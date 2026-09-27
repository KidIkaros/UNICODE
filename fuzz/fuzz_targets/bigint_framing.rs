//! Structured big-int framing. The input is a *specification* of chunk
//! streams — declared digit counts, declared byte counts, digit sources,
//! and single mutations of genuinely encoded chunks — so the header,
//! digit-count, and value-overflow paths of BigInt mode are explored
//! systematically instead of waiting for random bytes to synthesize
//! valid-looking 14-symbol headers.
//!
//! Oracles:
//! 1. decode never panics (existence of this target),
//! 2. decode into a `max_decoded_len`-sized buffer never returns
//!    `OutputTooSmall` (the sizing contract),
//! 3. every failure is a typed error,
//! 4. an unmutated encoding of real bytes round-trips exactly.

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicode16_blocks::layout::{Layout, Symbols};
use unicode16_blocks::BlockId;

const PREFIX_DIGITS: usize = 14;
const MAX_STREAM_SYMBOLS: usize = 8192;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    // Alphabet selection: BigInt-mode shapes only (non-power-of-two,
    // below the Dense threshold), including the base-3 worst case for
    // digit counts. NB: (Ogham, 2) is a power of two and would route to
    // BitPack — verified the hard way by the first fuzz session.
    let (block, count) = match data[0] & 3 {
        0 => (BlockId::Ogham, 3u32),
        1 => (BlockId::Runic, 24),
        2 => (BlockId::Runic, 63),
        _ => (BlockId::Runic, 96),
    };
    let layout = Layout::new(Symbols::Block { block, count }).expect("valid shape");
    debug_assert!(
        layout.bits_per_symbol().is_none() && !layout.supports_direct_byte_mapping(),
        "target premise: shape must select BigInt mode"
    );
    let radix = layout.symbol_count() as u64;

    if data[0] & 0x80 == 0 {
        synthetic_stream(&layout, radix, data);
    } else {
        mutated_real_chunk(&layout, radix, data);
    }
});

/// Builds a header's `PREFIX_DIGITS` symbols for `value`, clamped to what
/// 14 base-`radix` digits can represent (base 2 cannot express every
/// hostile value; that limit is part of the format).
fn header_symbols(layout: &Layout, radix: u64, value: u64, out: &mut Vec<u32>) {
    let cap = (radix as u128).pow(PREFIX_DIGITS as u32) - 1;
    let mut v = value.min(cap as u64);
    let mut digits = [0u64; PREFIX_DIGITS];
    for slot in digits.iter_mut().rev() {
        *slot = v % radix;
        v /= radix;
    }
    debug_assert_eq!(v, 0);
    for d in digits {
        out.push(layout.symbol(d as usize));
    }
}

/// Fully synthetic stream: each 3-byte chunk spec declares a header
/// value (digit count << 10 | byte count — hostile ranges included) and
/// a digit source; the digit symbols themselves are valid, maximal,
/// foreign, or noncharacters as specified.
fn synthetic_stream(layout: &Layout, radix: u64, data: &[u8]) {
    let mut symbols: Vec<u32> = Vec::new();
    let mut i = 1;
    while i + 3 <= data.len() && symbols.len() < MAX_STREAM_SYMBOLS {
        let h = u16::from_le_bytes([data[i], data[i + 1]]) as u64;
        let flags = data[i + 2];
        i += 3;

        header_symbols(layout, radix, h, &mut symbols);
        let nd_decl = (h >> 10) as usize;

        // Present digits: exactly `nd_decl` when the "truncate" flag is
        // clear (valid structure), half of them when set (ragged). Max
        // digits are synthesized without consuming input, so huge
        // declared counts are cheap to reach.
        let present = if flags & 0x04 != 0 {
            nd_decl / 2
        } else {
            nd_decl
        };
        // One chunk can push header + 2586 digits past the stream cap
        // (the while-condition only checks before the chunk), so the
        // budget must saturate.
        let budget = MAX_STREAM_SYMBOLS.saturating_sub(symbols.len());
        for d in 0..present.min(budget) {
            let sym = match flags & 0x03 {
                0 => layout.symbol(data[i % data.len()] as usize % radix as usize),
                1 => layout.symbol(radix as usize - 1), // maximal: value overflow
                2 => 0x0410,                            // foreign
                _ => 0x10_FFFE,                         // noncharacter
            };
            symbols.push(sym);
            if flags & 0x03 == 0 {
                i += 1;
            }
            let _ = d;
        }
    }

    let mut sink = vec![0u8; layout.max_decoded_len(symbols.len())];
    match layout.decode(&symbols, &mut sink) {
        Ok(_) => {}
        // Sized by the contract, so "buffer too small" is a violation.
        Err(e @ unicode16_blocks::LayoutError::OutputTooSmall { .. }) => {
            panic!("max_decoded_len contract violated: {e} (S={})", symbols.len())
        }
        Err(_) => {} // typed rejection: Corrupted / InvalidSymbol
    }
}

/// Ground-truth anchor: encode real bytes (BigInt mode), then apply one
/// structured mutation. Unmutated must round-trip; mutated must be `Ok`
/// or a typed error — never a panic.
fn mutated_real_chunk(layout: &Layout, radix: u64, data: &[u8]) {
    let payload_len = 1 + (data[1] as usize) % 64;
    let payload: Vec<u8> = data[2..2 + payload_len.min(data.len() - 2)].to_vec();

    let mut out = vec![0u32; layout.max_encoded_len(payload.len())];
    let n = layout.encode(&payload, &mut out).expect("BigInt encode");
    let mut stream = out[..n].to_vec();

    let mutation = data[0] & 0x0F;
    match mutation {
        0 => {} // unmutated: the round-trip oracle below applies
        1 => stream[0] = 0x0410,                     // foreign header symbol
        2 => stream[PREFIX_DIGITS - 1] = layout.symbol(radix as usize - 1), // header value bump
        3 => stream.push(layout.symbol(0)),          // extra digit: junk next-chunk header
        4 => {
            stream.pop(); // ragged digit count
            if stream.len() <= PREFIX_DIGITS {
                return;
            }
        }
        _ => {
            let digits_len = stream.len().saturating_sub(PREFIX_DIGITS);
            if digits_len == 0 {
                return;
            }
            let pos = PREFIX_DIGITS + (data[2] as usize) % digits_len;
            stream[pos] = layout.symbol(radix as usize - 1); // digit value bump
        }
    }

    let mut sink = vec![0u8; layout.max_decoded_len(stream.len())];
    match layout.decode(&stream, &mut sink) {
        Ok(m) if mutation == 0 => assert_eq!(
            &sink[..m],
            &payload,
            "unmutated BigInt encoding must round-trip"
        ),
        Ok(_) => {}                    // mutation happened to stay consistent
        Err(e @ unicode16_blocks::LayoutError::OutputTooSmall { .. }) => {
            panic!("max_decoded_len contract violated: {e}")
        }
        Err(_) => {}                   // typed rejection
    }
}
