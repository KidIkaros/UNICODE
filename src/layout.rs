//! User-defined Unicode "layouts": custom cipher alphabets over code points.
//!
//! A [`Layout`] is an ordered, distinct set of Unicode code points. It is
//! an alphabet: position *i* in the layout is the encoding of value *i*.
//!
//! Four encoding modes are selected automatically from the symbol count:
//!
//! | Symbols | Mode | Meaning |
//! |---|---|---|
//! | exactly 256 | [`Mode::Direct`] | one byte → one symbol (`symbols[byte]`) |
//! | power of two ≤ 256 | [`Mode::BitPack`] | *n*-bit symbols packed bit-level |
//! | > 256 (257+) | [`Mode::Dense`] | floor(log2(len)) payload bits per symbol (8 in the 257–511 gap, 9–12 beyond), packed in zero-waste byte↔symbol groups |
//! | non-power-of-two < 257 (e.g. 24) | [`Mode::BigInt`] | base-*len* number, chunked with length prefixes |
//!
//! All three round-trip byte-for-byte. The core is allocation-free
//! (`no_std`): codecs write into caller-provided buffers, so effects and
//! allocation stay at the edges (see the `layout` CLI in `main.rs`).

#[cfg(feature = "std")]
use crate::std::string::String;
use crate::{is_cipher_safe, is_surrogate, BlockId};

/// Hard cap on layout size: the largest power-of-two-friendly alphabet we
/// support without allocation.
pub const MAX_LAYOUT_LEN: usize = 4096;

/// Big-int mode framing: input is processed in `BIGINT_CHUNK`-byte
/// chunks; each chunk becomes a base-`len` digit string (at most
/// `BIGINT_MAX_DIGITS` digits — the base-3 worst case for 512 bytes)
/// preceded by a fixed-width `PREFIX_DIGITS`-symbol header carrying the
/// digit count and the chunk's byte count.
const BIGINT_CHUNK: usize = 512;
pub(crate) const BIGINT_MAX_DIGITS: usize = 2586; // ceil(4096 bits / log2 3) + 1
pub(crate) const PREFIX_DIGITS: usize = 14; // 3^14 > (2585 << 10) | 512

/// Where a layout's symbols come from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Symbols {
    /// The first `count` code points of a block, ascending. `count` must
    /// be at least 2 and at most the block's size.
    Block { block: BlockId, count: u32 },
    /// An explicit sequence of code points, in the given order. The
    /// string must outlive the layout, so it is `&'static`: build it from
    /// a literal or a `static`.
    Chars(&'static str),
    /// An owned symbol string (feature `std`). The layout copies the code
    /// points at construction time, so nothing leaks and the string is
    /// dropped. Prefer this over leaking a [`Symbols::Chars`] at an
    /// owning boundary.
    #[cfg(feature = "std")]
    String(String),
}

/// Everything that can go wrong while building or using a layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// Fewer than 2 distinct symbols — nothing to encode with.
    TooFewSymbols,
    /// More than [`MAX_LAYOUT_LEN`] symbols.
    TooManySymbols { len: usize },
    /// The same code point appears twice.
    DuplicateSymbol { index: usize, cp: u32 },
    /// `Symbols::Block { count }` exceeds the block's size.
    CountExceedsBlock { requested: u32, available: u32 },
    /// A code point is not cipher-safe (surrogate, noncharacter, private
    /// use, variation selector, or a `No_Block` gap).
    UnsafeCodePoint { index: usize, cp: u32 },
    /// A code point is above `U+10FFFF`.
    CodePointOutOfRange { index: usize, cp: u32 },
    /// The caller's output buffer is too small; `needed` is the exact size.
    OutputTooSmall { needed: usize },
    /// A symbol that is not in the alphabet appeared in ciphertext.
    InvalidSymbol { index: usize, cp: u32 },
    /// The ciphertext is structurally broken (bad padding, incoherent
    /// length prefix, digit overflow).
    Corrupted,
}

impl core::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            LayoutError::TooFewSymbols => {
                write!(f, "a layout needs at least 2 distinct symbols")
            }
            LayoutError::TooManySymbols { len } => {
                write!(f, "{len} symbols exceeds the maximum of {MAX_LAYOUT_LEN}")
            }
            LayoutError::DuplicateSymbol { index, cp } => {
                write!(f, "duplicate symbol U+{cp:04X} at position {index}")
            }
            LayoutError::CountExceedsBlock {
                requested,
                available,
            } => {
                write!(
                    f,
                    "requested {requested} symbols but the block has {available}"
                )
            }
            LayoutError::UnsafeCodePoint { index, cp } => {
                write!(
                    f,
                    "code point U+{cp:04X} at position {index} is not cipher-safe"
                )
            }
            LayoutError::CodePointOutOfRange { index, cp } => {
                write!(
                    f,
                    "code point U+{cp:08X} at position {index} is above U+10FFFF"
                )
            }
            LayoutError::OutputTooSmall { needed } => {
                write!(f, "output buffer too small: {needed} slots needed")
            }
            LayoutError::InvalidSymbol { index, cp } => {
                write!(
                    f,
                    "symbol U+{cp:04X} at position {index} is not in this layout"
                )
            }
            LayoutError::Corrupted => write!(f, "ciphertext is structurally corrupted"),
        }
    }
}

/// Encoding mode, derived from the symbol count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Exactly 256 symbols: one byte per symbol.
    Direct,
    /// `len` a power of two ≤ 256: `bits`-bit symbols, bit-packed.
    BitPack(u32),
    /// `len` > 256: the low `bits = floor(log2(len))` bits of each symbol
    /// carry payload, packed byte↔symbol in whole groups (zero waste).
    /// For non-power-of-two alphabets the top symbols are unreachable in
    /// a fixed-width code and simply unused.
    Dense(u32),
    /// All other sizes (e.g. 24): base-`len` big integer in
    /// `BIGINT_CHUNK`-byte chunks with length prefixes.
    BigInt,
}

/// An ordered cipher alphabet of distinct Unicode code points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// Symbols in alphabet order: index *i* encodes value *i*.
    symbols: [u32; MAX_LAYOUT_LEN],
    /// The same symbols, sorted ascending, for O(log n) membership
    /// lookups.
    sorted: [u32; MAX_LAYOUT_LEN],
    /// `order[p]` is the alphabet index of `sorted[p]`, so a lookup hit
    /// maps back to the value that symbol encodes even when the user's
    /// symbol string is not in ascending order.
    order: [u16; MAX_LAYOUT_LEN],
    len: usize,
}

impl Layout {
    /// Builds and fully validates a layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use unicode16_blocks::{BlockId, Layout, Symbols};
    ///
    /// let layout = Layout::new(Symbols::Block { block: BlockId::Runic, count: 24 })
    ///     .expect("24 runic symbols");
    /// assert_eq!(layout.symbol_count(), 24);
    /// assert_eq!(layout.symbol(0), 0x16A0); // ᚠ
    /// ```
    pub fn new(symbols: Symbols) -> Result<Self, LayoutError> {
        Self::build(symbols, true)
    }

    /// Like [`Layout::new`], but tolerates private-use code points,
    /// noncharacters, and variation selectors — for closed systems that
    /// deliberately encode into those ranges. Surrogates are still
    /// rejected: they are not valid scalar values and could never be
    /// emitted as text.
    pub fn new_permissive(symbols: Symbols) -> Result<Self, LayoutError> {
        Self::build(symbols, false)
    }

    fn build(symbols: Symbols, strict: bool) -> Result<Self, LayoutError> {
        let mut layout = Layout {
            symbols: [0; MAX_LAYOUT_LEN],
            sorted: [0; MAX_LAYOUT_LEN],
            order: [0; MAX_LAYOUT_LEN],
            len: 0,
        };
        match symbols {
            Symbols::Block { block, count } => {
                let b = block.block();
                if count < 2 {
                    return Err(LayoutError::TooFewSymbols);
                }
                if count as usize > MAX_LAYOUT_LEN {
                    return Err(LayoutError::TooManySymbols {
                        len: count as usize,
                    });
                }
                if count > b.size_chars() {
                    return Err(LayoutError::CountExceedsBlock {
                        requested: count,
                        available: b.size_chars(),
                    });
                }
                for (i, slot) in layout.symbols[..count as usize].iter_mut().enumerate() {
                    *slot = b.first + i as u32;
                }
                layout.len = count as usize;
            }
            Symbols::Chars(s) => {
                let mut len = 0usize;
                push_char_symbols(&mut layout, s, &mut len, strict)?;
                layout.len = len;
            }
            #[cfg(feature = "std")]
            Symbols::String(s) => {
                // Owned source: copy the code points in, then drop the
                // string. Nothing leaks.
                let mut len = 0usize;
                push_char_symbols(&mut layout, &s, &mut len, strict)?;
                layout.len = len;
            }
        }

        if strict {
            for (index, &cp) in layout.symbols[..layout.len].iter().enumerate() {
                if !is_cipher_safe(cp) {
                    return Err(LayoutError::UnsafeCodePoint { index, cp });
                }
            }
        }
        if layout.len < 2 {
            return Err(LayoutError::TooFewSymbols);
        }

        layout.sorted[..layout.len].copy_from_slice(&layout.symbols[..layout.len]);
        for (i, o) in layout.order[..layout.len].iter_mut().enumerate() {
            *o = i as u16;
        }
        insertion_sort_with_order(
            &mut layout.sorted[..layout.len],
            &mut layout.order[..layout.len],
        );
        Ok(layout)
    }

    /// Number of symbols in the alphabet (the radix of the encoding).
    pub fn symbol_count(&self) -> usize {
        self.len
    }

    /// The code point at alphabet position `index` (the encoding of value
    /// `index`).
    ///
    /// # Panics
    ///
    /// Panics if `index >= symbol_count()`.
    pub fn symbol(&self, index: usize) -> u32 {
        self.symbols[index]
    }

    /// Iterates the alphabet's code points in order.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.symbols[..self.len].iter().copied()
    }

    /// Alphabet position of `cp` — i.e. the value that symbol encodes —
    /// or `None` if it is not in the layout.
    pub fn index_of(&self, cp: u32) -> Option<usize> {
        let mut lo = 0usize;
        let mut hi = self.len;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.sorted[mid] < cp {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        // NB: `then_some` evaluates its argument eagerly, so an if/else
        // is required here: on a miss, `lo == len` and `order[lo]` would
        // index one past the logical end (a silent garbage read for
        // len < MAX, a hard panic at len == MAX).
        if lo < self.len && self.sorted[lo] == cp {
            Some(self.order[lo] as usize)
        } else {
            None
        }
    }

    /// Is `cp` part of this alphabet?
    pub fn contains(&self, cp: u32) -> bool {
        self.index_of(cp).is_some()
    }

    /// Payload bits per symbol in the packed modes.
    ///
    /// For [`Mode::Dense`] this is `floor(log2(symbol_count))`: a 4096
    /// symbol alphabet carries 12 bits per symbol, but a 4000-symbol one
    /// carries 11 — the top symbols would be unreachable in a fixed-width
    /// code, so they are unused.
    pub fn bits_per_symbol(&self) -> Option<u32> {
        match self.mode() {
            Mode::BitPack(bits) | Mode::Dense(bits) => Some(bits),
            _ => None,
        }
    }

    /// True when one byte maps to exactly one symbol.
    pub fn supports_direct_byte_mapping(&self) -> bool {
        self.mode() == Mode::Direct
    }

    fn mode(&self) -> Mode {
        if self.len == 256 {
            Mode::Direct
        } else if self.len.is_power_of_two() && self.len <= 256 {
            Mode::BitPack(self.len.trailing_zeros())
        } else if self.len > 256 {
            // Dense packing carries floor(log2(len)) bits per symbol: 8
            // bits across the 257..=511 gap — Direct's density, far ahead
            // of BigInt's ~1.7x — then 9..12 bits beyond 512. Width-
            // independent: usize may be 32- or 64-bit, so anchor to
            // usize::BITS.
            Mode::Dense((usize::BITS - 1) - self.len.leading_zeros())
        } else {
            Mode::BigInt
        }
    }

    /// Upper bound on the symbol count [`Layout::encode`] can produce for
    /// `byte_len` input bytes. Size output buffers with this.
    pub fn max_encoded_len(&self, byte_len: usize) -> usize {
        match self.mode() {
            Mode::Direct => byte_len,
            Mode::BitPack(bits) => ceil_div(byte_len * 8, bits as usize),
            Mode::Dense(bits) => {
                let (group_bits, syms) = dense_group(bits);
                let bytes_per_group = group_bits as usize / 8;
                let groups = ceil_div(byte_len, bytes_per_group);
                groups * syms as usize + usize::from(byte_len % bytes_per_group != 0)
            }
            Mode::BigInt => ceil_div(byte_len, BIGINT_CHUNK) * (BIGINT_MAX_DIGITS + PREFIX_DIGITS),
        }
    }

    /// Upper bound on the byte count [`Layout::decode`] can produce for
    /// `symbol_count` input symbols.
    ///
    /// In [`Mode::Dense`] this can *exceed* `symbol_count`: symbols are
    /// wider than bytes there (e.g. 2 twelve-bit symbols decode to 3
    /// bytes). Ragged symbol counts (not a whole number of groups) are
    /// rejected by `decode` with [`LayoutError::Corrupted`].
    ///
    /// In [`Mode::BigInt`] the bound is genuinely larger than the symbol
    /// count: a chunk encodes to as few as `PREFIX_DIGITS + 1` symbols
    /// (all-zero content) yet decodes to `BIGINT_CHUNK` bytes, so the
    /// tight upper bound is `ceil_div(S, PREFIX_DIGITS + 1) × BIGINT_CHUNK`.
    pub fn max_decoded_len(&self, symbol_count: usize) -> usize {
        match self.mode() {
            Mode::Dense(bits) => {
                let (group_bits, syms_per_group) = dense_group(bits);
                symbol_count / syms_per_group as usize * (group_bits as usize / 8)
            }
            Mode::BigInt => ceil_div(symbol_count, PREFIX_DIGITS + 1) * BIGINT_CHUNK,
            _ => symbol_count,
        }
    }

    /// Encodes `bytes` into `out` as alphabet code points. Returns the
    /// number of symbols written.
    ///
    /// # Errors
    ///
    /// [`LayoutError::OutputTooSmall`] if `out` cannot hold the result;
    /// [`LayoutError::CodePointOutOfRange`] if `bytes` produced a symbol
    /// beyond the code space (cannot happen with `MAX_LAYOUT_LEN` symbol
    /// alphabets, but the codec is total).
    pub fn encode(&self, bytes: &[u8], out: &mut [u32]) -> Result<usize, LayoutError> {
        match self.mode() {
            Mode::Direct => self.encode_direct(bytes, out),
            Mode::BitPack(bits) => self.encode_bitpack(bytes, bits, out),
            Mode::Dense(bits) => self.encode_dense(bytes, bits, out),
            Mode::BigInt => self.encode_bigint(bytes, out),
        }
    }

    /// Decodes alphabet code points back into bytes. Returns the number
    /// of bytes written.
    ///
    /// # Errors
    ///
    /// [`LayoutError::InvalidSymbol`] for a code point outside the
    /// alphabet, [`LayoutError::Corrupted`] for broken padding or length
    /// prefixes, [`LayoutError::OutputTooSmall`] for a small buffer.
    pub fn decode(&self, symbols: &[u32], out: &mut [u8]) -> Result<usize, LayoutError> {
        match self.mode() {
            Mode::Direct => self.decode_direct(symbols, out),
            Mode::BitPack(bits) => self.decode_bitpack(symbols, bits, out),
            Mode::Dense(bits) => self.decode_dense(symbols, bits, out),
            Mode::BigInt => self.decode_bigint(symbols, out),
        }
    }

    // -- direct ------------------------------------------------------------

    fn encode_direct(&self, bytes: &[u8], out: &mut [u32]) -> Result<usize, LayoutError> {
        if out.len() < bytes.len() {
            return Err(LayoutError::OutputTooSmall {
                needed: bytes.len(),
            });
        }
        for (i, &b) in bytes.iter().enumerate() {
            let cp = self.symbols[b as usize];
            if cp > 0x10_FFFF {
                return Err(LayoutError::CodePointOutOfRange { index: i, cp });
            }
            out[i] = cp;
        }
        Ok(bytes.len())
    }

    fn decode_direct(&self, symbols: &[u32], out: &mut [u8]) -> Result<usize, LayoutError> {
        if out.len() < symbols.len() {
            return Err(LayoutError::OutputTooSmall {
                needed: symbols.len(),
            });
        }
        for (i, &cp) in symbols.iter().enumerate() {
            let idx = self
                .index_of(cp)
                .ok_or(LayoutError::InvalidSymbol { index: i, cp })?;
            if idx >= 256 {
                return Err(LayoutError::InvalidSymbol { index: i, cp });
            }
            out[i] = idx as u8;
        }
        Ok(symbols.len())
    }

    // -- bit packing ---------------------------------------------------------

    // Reads the byte stream MSB-first in `bits`-wide groups; a group that
    // straddles two bytes is gathered across the boundary. A group that
    // straddles past the final byte reads zero pad bits, which decode
    // discards.
    fn encode_bitpack(
        &self,
        bytes: &[u8],
        bits: u32,
        out: &mut [u32],
    ) -> Result<usize, LayoutError> {
        let needed = ceil_div(bytes.len() * 8, bits as usize);
        if out.len() < needed {
            return Err(LayoutError::OutputTooSmall { needed });
        }
        let mut written = 0usize;
        let mut byte_pos = 0usize;
        let mut bit_pos = 0u32; // next bit to read within bytes[byte_pos]
        while written < needed {
            let mut value: u32 = 0;
            let mut got: u32 = 0;
            while got < bits {
                let avail = 8 - bit_pos;
                let take = avail.min(bits - got);
                let shift = 8 - bit_pos - take;
                // Pad bits (zero) once the stream is exhausted: the final
                // symbol may straddle past the last byte.
                let byte = if byte_pos < bytes.len() {
                    bytes[byte_pos]
                } else {
                    0
                };
                let chunk = (byte >> shift) as u32 & ((1 << take) - 1);
                value = (value << take) | chunk;
                got += take;
                bit_pos += take;
                if bit_pos == 8 {
                    bit_pos = 0;
                    byte_pos += 1;
                }
            }
            out[written] = self.symbols[value as usize];
            written += 1;
        }
        Ok(written)
    }

    // Writes symbols back into a bit stream, MSB-first, flushing whole
    // bytes. The final partial byte (encoder padding) is discarded, so the
    // output is exactly floor(symbols × bits / 8) bytes — which equals the
    // original input length for anything `encode_bitpack` produced.
    fn decode_bitpack(
        &self,
        symbols: &[u32],
        bits: u32,
        out: &mut [u8],
    ) -> Result<usize, LayoutError> {
        let needed = symbols.len() * bits as usize / 8;
        if out.len() < needed {
            return Err(LayoutError::OutputTooSmall { needed });
        }
        let mut written = 0usize;
        let mut acc: u32 = 0; // pending bits, low-aligned
        let mut acc_bits: u32 = 0;
        for (i, &s) in symbols.iter().enumerate() {
            let value =
                self.index_of(s)
                    .ok_or(LayoutError::InvalidSymbol { index: i, cp: s })? as u32;
            acc = (acc << bits) | value;
            acc_bits += bits;
            if acc_bits >= 8 {
                acc_bits -= 8;
                out[written] = (acc >> acc_bits) as u8;
                written += 1;
            }
        }
        debug_assert_eq!(written, needed);
        Ok(written)
    }

    // -- dense packing (len > 256) ------------------------------------------

    // Packs the byte stream MSB-first into `bits`-wide symbols, grouped so
    // that whole groups align with whole bytes. If the input does not fill
    // a whole number of groups, the tail group is zero-padded and ONE
    // extra marker symbol is appended, carrying the tail's real byte
    // count `r` (1..bytes_per_group-1; always encodable since
    // r < bytes_per_group <= bits). Decoding detects the marker by symbol
    // count: S % symbols_per_group == 1 (symbols_per_group is always >= 2
    // here, because MAX_LAYOUT_LEN keeps bits in 9..=12), so the original
    // byte length is recovered exactly — pad bytes are never mistaken
    // for data.
    fn encode_dense(&self, bytes: &[u8], bits: u32, out: &mut [u32]) -> Result<usize, LayoutError> {
        let (group_bits, syms_per_group) = dense_group(bits);
        let bytes_per_group = group_bits as usize / 8;
        let groups = ceil_div(bytes.len(), bytes_per_group);
        let has_tail = bytes.len() % bytes_per_group != 0;
        let needed = groups * syms_per_group as usize + usize::from(has_tail);
        if out.len() < needed {
            return Err(LayoutError::OutputTooSmall { needed });
        }

        let mut bit_pos = 0usize; // absolute bit index, MSB-first
        for g in 0..groups {
            for s in 0..syms_per_group as usize {
                let mut value: u32 = 0;
                for _ in 0..bits {
                    // Zero pad once the byte stream is exhausted (tail
                    // groups only).
                    let byte = if bit_pos / 8 < bytes.len() {
                        bytes[bit_pos / 8]
                    } else {
                        0
                    };
                    value = (value << 1) | ((byte >> (7 - (bit_pos % 8))) & 1) as u32;
                    bit_pos += 1;
                }
                out[g * syms_per_group as usize + s] = self.symbols[value as usize];
            }
        }
        debug_assert_eq!(bit_pos, groups * group_bits as usize);
        if has_tail {
            let r = bytes.len() % bytes_per_group;
            out[needed - 1] = self.symbols[r];
        }
        Ok(needed)
    }

    // Inverse of [`Layout::encode_dense`]. A trailing marker symbol
    // (detected as `S % symbols_per_group == 1`) recovers the tail's real
    // byte count; its padding is dropped, so the output is exactly the
    // original input length.
    fn decode_dense(
        &self,
        symbols: &[u32],
        bits: u32,
        out: &mut [u8],
    ) -> Result<usize, LayoutError> {
        let (group_bits, syms_per_group) = dense_group(bits);
        let s = syms_per_group as usize;
        let bytes_per_group = group_bits as usize / 8;

        let has_tail = symbols.len() % s == 1;
        let groups = if has_tail {
            (symbols.len() - 1) / s
        } else {
            symbols.len() / s
        };
        if has_tail && groups == 0 {
            return Err(LayoutError::Corrupted);
        }
        let mut total = groups * bytes_per_group;
        if has_tail {
            let marker = symbols[symbols.len() - 1];
            let r = self.index_of(marker).ok_or(LayoutError::InvalidSymbol {
                index: symbols.len() - 1,
                cp: marker,
            })?;
            if r == 0 || r >= bytes_per_group {
                return Err(LayoutError::Corrupted);
            }
            total = (groups - 1) * bytes_per_group + r;
        }
        if out.len() < total {
            return Err(LayoutError::OutputTooSmall { needed: total });
        }

        let payload = if has_tail {
            &symbols[..symbols.len() - 1]
        } else {
            symbols
        };
        let mut written = 0usize;
        let mut cur: u8 = 0;
        let mut cur_bits = 0u32;
        for (i, &sym) in payload.iter().enumerate() {
            if written == total {
                break; // remaining symbols are tail padding
            }
            let value = self
                .index_of(sym)
                .ok_or(LayoutError::InvalidSymbol { index: i, cp: sym })?
                as u32;
            // Dense codes only use the low `bits` bits. A higher index is
            // either tampering or a foreign layout.
            if value >> bits != 0 {
                return Err(LayoutError::InvalidSymbol { index: i, cp: sym });
            }
            for k in (0..bits).rev() {
                cur = (cur << 1) | ((value >> k) & 1) as u8;
                cur_bits += 1;
                if cur_bits == 8 {
                    out[written] = cur;
                    written += 1;
                    cur = 0;
                    cur_bits = 0;
                    if written == total {
                        break; // tail padding follows; drop it
                    }
                }
            }
        }
        debug_assert_eq!(written, total);
        Ok(written)
    }

    // -- big integer ---------------------------------------------------------

    fn encode_bigint(&self, bytes: &[u8], out: &mut [u32]) -> Result<usize, LayoutError> {
        let chunks = ceil_div(bytes.len(), BIGINT_CHUNK);
        let needed = chunks * (BIGINT_MAX_DIGITS + PREFIX_DIGITS);
        if out.len() < needed {
            return Err(LayoutError::OutputTooSmall { needed });
        }
        let mut written = 0usize;
        for chunk in bytes.chunks(BIGINT_CHUNK) {
            written += self.encode_bigint_chunk(chunk, &mut out[written..])?;
        }
        Ok(written)
    }

    /// Converts one chunk to base-`len` digits: repeated division of the
    /// chunk (held little-endian, so the significant window shrinks from
    /// the top) collecting remainders — O(digits × chunk) time, 512 bytes
    /// of stack, no allocation.
    fn encode_bigint_chunk(&self, chunk: &[u8], out: &mut [u32]) -> Result<usize, LayoutError> {
        let len = self.len as u32;

        // Strip leading zero bytes (most significant in big-endian order).
        let mut start = 0usize;
        while start < chunk.len() && chunk[start] == 0 {
            start += 1;
        }

        let mut nd = 0usize;
        if start == chunk.len() {
            // All-zero chunk still encodes one zero digit.
            out[PREFIX_DIGITS] = self.symbols[0];
            nd = 1;
        } else {
            let mut num = [0u8; BIGINT_CHUNK];
            for (i, &b) in chunk[start..].iter().rev().enumerate() {
                num[i] = b; // little-endian: num[0] is least significant
            }
            let mut region = chunk.len() - start;
            while region > 0 {
                let mut rem: u32 = 0;
                for i in (0..region).rev() {
                    let cur = (rem << 8) | num[i] as u32;
                    num[i] = (cur / len) as u8;
                    rem = cur % len;
                }
                out[PREFIX_DIGITS + nd] = self.symbols[rem as usize];
                nd += 1;
                if num[region - 1] == 0 {
                    region -= 1;
                }
            }
            debug_assert!(nd <= BIGINT_MAX_DIGITS);
            // Digits were collected least-significant first; emit
            // most-significant first.
            out[PREFIX_DIGITS..PREFIX_DIGITS + nd].reverse();
        }

        // Header: (digit_count << 10) | byte_count, in exactly
        // PREFIX_DIGITS base-`len` digits, most significant first.
        let value = ((nd as u64) << 10) | chunk.len() as u64;
        let mut v = value as usize;
        for i in (0..PREFIX_DIGITS).rev() {
            out[i] = self.symbols[v % self.len];
            v /= self.len;
        }
        debug_assert_eq!(v, 0, "prefix overflow");
        Ok(PREFIX_DIGITS + nd)
    }

    /// Walks the chunk framing and returns the total decoded byte count.
    fn bigint_total_bytes(&self, symbols: &[u32]) -> Result<usize, LayoutError> {
        let len = self.len as u64;
        let maxv: u64 = ((BIGINT_MAX_DIGITS as u64) << 10) | BIGINT_CHUNK as u64;
        let mut pos = 0usize;
        let mut total = 0usize;
        while pos < symbols.len() {
            if symbols.len() - pos < PREFIX_DIGITS {
                return Err(LayoutError::Corrupted);
            }
            let mut value: u64 = 0;
            for (k, &s) in symbols[pos..pos + PREFIX_DIGITS].iter().enumerate() {
                let idx = self.index_of(s).ok_or(LayoutError::InvalidSymbol {
                    index: pos + k,
                    cp: s,
                })? as u64;
                value = value * len + idx;
                if value > maxv {
                    return Err(LayoutError::Corrupted);
                }
            }
            let nd = (value >> 10) as usize;
            let nbytes = (value & 0x3FF) as usize;
            if nd == 0 || nd > BIGINT_MAX_DIGITS || nbytes == 0 || nbytes > BIGINT_CHUNK {
                return Err(LayoutError::Corrupted);
            }
            pos += PREFIX_DIGITS;
            if symbols.len() - pos < nd {
                return Err(LayoutError::Corrupted);
            }
            pos += nd;
            total += nbytes;
        }
        Ok(total)
    }

    fn decode_bigint(&self, symbols: &[u32], out: &mut [u8]) -> Result<usize, LayoutError> {
        let total = self.bigint_total_bytes(symbols)?;
        if out.len() < total {
            return Err(LayoutError::OutputTooSmall { needed: total });
        }
        let len = self.len as u32;
        let mut pos = 0usize;
        let mut written = 0usize;
        while pos < symbols.len() {
            // Re-read the header; bigint_total_bytes validated the header
            // fields. NB: it deliberately does NOT validate the digit
            // symbols below (it skips over them), so digit lookups must
            // be fallible here — an `.expect` here is a hostile-input
            // panic (fuzzer-found).
            let mut value: u64 = 0;
            for (k, &s) in symbols[pos..pos + PREFIX_DIGITS].iter().enumerate() {
                let idx = self.index_of(s).ok_or(LayoutError::InvalidSymbol {
                    index: pos + k,
                    cp: s,
                })? as u64;
                value = value * self.len as u64 + idx;
            }
            let nd = (value >> 10) as usize;
            let nbytes = (value & 0x3FF) as usize;
            pos += PREFIX_DIGITS;

            // Multiply-add the digits (most significant first) into a
            // little-endian byte window, then right-align to `nbytes`.
            let mut k = 0usize; // little-endian bytes so far
            for (di, &s) in symbols[pos..pos + nd].iter().enumerate() {
                let mut carry = self.index_of(s).ok_or(LayoutError::InvalidSymbol {
                    index: pos + di,
                    cp: s,
                })? as u32;
                for b in out[written..written + k].iter_mut() {
                    let v = (*b as u32) * len + carry;
                    *b = (v & 0xFF) as u8;
                    carry = v >> 8;
                }
                while carry > 0 {
                    // Hostile input can present digits whose value exceeds
                    // the declared chunk size; refuse rather than write
                    // past it.
                    if k >= nbytes {
                        return Err(LayoutError::Corrupted);
                    }
                    out[written + k] = (carry & 0xFF) as u8;
                    carry >>= 8;
                    k += 1;
                }
            }
            pos += nd;

            debug_assert!(k <= nbytes, "chunk value overflows its declared size");
            out[written..written + k].reverse(); // to big-endian
            if k < nbytes {
                out.copy_within(written..written + k, written + nbytes - k);
                out[written..written + nbytes - k].fill(0);
            }
            written += nbytes;
        }
        debug_assert_eq!(written, total);
        Ok(written)
    }
}

/// Copies the code points of `s` into `layout.symbols` starting at 0,
/// validating range, safety, and distinctness as it goes. `len` tracks
/// how many slots are filled.
fn push_char_symbols(
    layout: &mut Layout,
    s: &str,
    len: &mut usize,
    strict: bool,
) -> Result<(), LayoutError> {
    for (index, c) in s.chars().enumerate() {
        if index >= MAX_LAYOUT_LEN {
            return Err(LayoutError::TooManySymbols {
                len: s.chars().count(),
            });
        }
        let cp = c as u32;
        if cp > 0x10_FFFF {
            return Err(LayoutError::CodePointOutOfRange { index, cp });
        }
        if is_surrogate(cp) {
            // Unreachable from &str (chars are scalars); kept for
            // symmetry with the raw-code-point paths.
            return Err(LayoutError::UnsafeCodePoint { index, cp });
        }
        if strict && !is_cipher_safe(cp) {
            return Err(LayoutError::UnsafeCodePoint { index, cp });
        }
        // Alphabet symbols must be distinct or decoding is ambiguous.
        // n ≤ 4096, so a linear scan over previous entries costs at most
        // ~8M comparisons once, at construction time.
        if layout.symbols[..*len].contains(&cp) {
            return Err(LayoutError::DuplicateSymbol { index, cp });
        }
        layout.symbols[index] = cp;
        *len += 1;
    }
    Ok(())
}

/// Ceiling division. (Keeps the crate's 1.63 MSRV; `div_ceil` needs 1.73.)
fn ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

/// Greatest common divisor.
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Byte↔symbol group shape for [`Mode::Dense`]: the smallest bit group
/// that is a whole number of both bytes and `bits`-bit symbols —
/// `(group_bits, symbols_per_group)`. Every symbol carries exactly
/// `bits` payload bits; no bits are wasted on padding inside a group.
pub(crate) fn dense_group(bits: u32) -> (u32, u32) {
    let g = gcd(8, bits);
    (8 * bits / g, 8 / g)
}

fn insertion_sort_with_order(s: &mut [u32], order: &mut [u16]) {
    for i in 1..s.len() {
        let key = s[i];
        let key_order = order[i];
        let mut j = i;
        while j > 0 && s[j - 1] > key {
            s[j] = s[j - 1];
            order[j] = order[j - 1];
            j -= 1;
        }
        s[j] = key;
        order[j] = key_order;
    }
}
