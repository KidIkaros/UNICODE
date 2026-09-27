//! # unicode16-blocks
//!
//! Every **Unicode 16.0** block — all 338 of them — as typed Rust data,
//! built for cipher work.
//!
//! The table is exhaustive: the 338 block ranges plus the `No_Block` gaps
//! between them partition the entire code space `U+0000..=U+10FFFF`
//! (1,114,112 code points) with no overlap and no holes.
//!
//! ## What you get
//!
//! * [`BLOCKS`] — a `const` array of all 338 blocks in ascending order.
//! * [`BlockId`] — one enum variant per block (copyable, orderable, hashable).
//! * [`block_of`] — O(log n) block lookup for any `char`.
//! * [`block_by_id`] / [`block_by_name`] — reverse lookup (UAX #44-loose
//!   name matching, so `"cyrillic supplement"` finds `Cyrillic Supplement`).
//! * [`Layout`] — user-defined cipher alphabets: build one from a block or
//!   a symbol string, then encode/decode bytes with allocation-free codecs.
//! * Cipher-safety predicates: [`is_surrogate`], [`is_noncharacter`],
//!   [`is_private_use`], [`is_variation_selector`], [`is_cipher_safe`].
//!
//! ## Cipher usage patterns
//!
//! **1. Unicode alphabets.** Any block is a ready-made cipher alphabet.
//! A block of 256 symbols maps one code point per byte:
//!
//! ```
//! use unicode16_blocks::{block_by_id, BlockId, is_cipher_safe};
//!
//! let block = block_by_id(BlockId::ByzantineMusicalSymbols);
//! assert_eq!(block.size_chars(), 256);
//!
//! // byte 0x41 → code point U+1D041
//! let cp = block.first + 0x41;
//! assert!(is_cipher_safe(cp));
//! ```
//!
//! The same trick scales: 16-bit values fit `CJK Unified Ideographs`
//! (20,992 symbols) or `Yi Syllables` (11,656); full 20-bit values fit
//! `Supplementary Private Use Area-B` (65,536 per block, and the
//! supplementary PUA pair gives 131,072).
//!
//! **2. Tamper detection.** Ciphertext that must stay in its block is
//! easy to validate on decode — anything mapping to the wrong block, or
//! to a `No_Block` gap, is corruption or forgery:
//!
//! ```
//! use unicode16_blocks::{block_of, BlockId};
//!
//! let c = '⣿'; // U+28FF
//! assert_eq!(block_of(c).map(|b| b.id), Some(BlockId::BraillePatterns));
//! ```
//!
//! **3. Stream validation.** [`is_cipher_safe`] rejects everything that
//! breaks text pipelines (surrogates, noncharacters, variation selectors)
//! plus private-use and unassigned code points, before they reach
//! encoders, databases, or logs.
//!
//! ## `no_std`
//!
//! The crate is `#![no_std]` with zero dependencies: the table is static
//! data and the lookup is a plain binary search.
//!
//! ## Out of scope
//!
//! General-category data (assigned vs. unassigned *inside* a block),
//! confusables, and case folding are other UCD properties and are not
//! included. [`is_cipher_safe`] therefore treats an unassigned code point
//! that sits *inside* an allocated block as safe; code points in the
//! `No_Block` gaps are unassigned and rejected.

#![no_std]

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;

mod block_id;
pub mod layout;
pub mod layout_json;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod tests;
mod unicode16_blocks;

pub use block_id::BlockId;
pub use layout::{Layout, LayoutError, Symbols, MAX_LAYOUT_LEN};
#[cfg(feature = "json")]
pub use layout_json::layout_from_json;
pub use layout_json::{write_layout_json, SourceDesc};

/// Highest Unicode code point (`U+10FFFF`).
pub const UNICODE_LAST: char = '\u{10FFFF}';

/// Total number of Unicode code points, including surrogates: 1,114,112.
pub const TOTAL_CODE_POINTS: u32 = 0x11_0000;

/// Number of blocks in [`BLOCKS`] — all of Unicode 16.0.
pub const BLOCK_COUNT: usize = 338;

/// One Unicode block: an inclusive code-point range plus its name and id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Block {
    /// First code point of the block (always ≡ 0 mod 16).
    ///
    /// A `u32` rather than `char` because the surrogate blocks consist
    /// entirely of code points that are not valid scalar values.
    pub first: u32,
    /// Last code point of the block (always ≡ 15 mod 16), inclusive.
    pub last: u32,
    /// Official Unicode block name, e.g. `"Cyrillic"`.
    pub name: &'static str,
    /// Typed identifier for this block.
    pub id: BlockId,
}

impl Block {
    /// Number of code points in the block (assigned or not).
    ///
    /// # Examples
    ///
    /// ```
    /// use unicode16_blocks::{block_by_id, BlockId};
    ///
    /// assert_eq!(block_by_id(BlockId::BasicLatin).size_chars(), 128);
    /// assert_eq!(block_by_id(BlockId::BraillePatterns).size_chars(), 256);
    /// ```
    #[inline]
    pub const fn size_chars(&self) -> u32 {
        self.last - self.first + 1
    }

    /// Is `c` inside this block's range?
    #[inline]
    pub const fn contains(&self, c: char) -> bool {
        let cp = c as u32;
        cp >= self.first && cp <= self.last
    }
}

/// The complete Unicode 16.0 block table, ascending by code point.
///
/// 338 entries; see [`block_of`] for lookup. Guaranteed by tests to be
/// sorted, pairwise disjoint, and exhaustive (together with the
/// `No_Block` gaps) over `U+0000..=U+10FFFF`.
pub use unicode16_blocks::BLOCKS;

/// Returns the block containing `c`, or `None` if `c` lies in one of the
/// `No_Block` gaps between blocks (unassigned ranges such as
/// `U+323B0..=U+DFFFF`). Usable in `const` contexts.
///
/// Binary search: O(log 338) ≈ 9 comparisons.
///
/// # Examples
///
/// ```
/// use unicode16_blocks::{block_of, BlockId};
///
/// assert_eq!(block_of('A').map(|b| b.id), Some(BlockId::BasicLatin));
/// assert_eq!(block_of('д').map(|b| b.id), Some(BlockId::Cyrillic));
/// assert_eq!(block_of('\u{4E2D}').map(|b| b.id), Some(BlockId::CJKUnifiedIdeographs));
/// assert_eq!(block_of('\u{40000}'), None); // No_Block gap
/// ```
#[inline]
pub const fn block_of(c: char) -> Option<&'static Block> {
    block_of_cp(c as u32)
}

/// Numeric-code-point flavor of [`block_of`].
///
/// Accepts any `u32`; surrogates (`U+D800..=U+DFFF`) and out-of-range
/// values still resolve against the table (they fall inside the surrogate
/// blocks) rather than being rejected, because the block table covers
/// them. Use [`is_cipher_safe`] when you need scalar validation.
/// Binary search: O(log 338) ≈ 9 comparisons. Usable in `const` contexts.
#[inline]
pub const fn block_of_cp(cp: u32) -> Option<&'static Block> {
    if cp > 0x10_FFFF {
        return None;
    }
    // Manual binary search (const-stable): count the blocks whose start is
    // <= cp; the candidate is the block just before that count. BLOCKS[0]
    // starts at U+0000, so the subtraction cannot underflow.
    let mut lo = 0usize;
    let mut hi = BLOCKS.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if BLOCKS[mid].first <= cp {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let b = &BLOCKS[lo - 1];
    if cp <= b.last {
        Some(b)
    } else {
        None
    }
}

/// Returns the block record for a [`BlockId`].
///
/// # Examples
///
/// ```
/// use unicode16_blocks::{block_by_id, BlockId};
///
/// let b = block_by_id(BlockId::Cyrillic);
/// assert_eq!(b.name, "Cyrillic");
/// assert_eq!(b.first, 0x0400);
/// assert_eq!(b.last, 0x04FF);
/// ```
#[inline]
pub fn block_by_id(id: BlockId) -> &'static Block {
    &BLOCKS[id.index()]
}

/// Looks a block up by its official Unicode name.
///
/// Names match exactly, or after UAX #44-style "loose" normalization
/// (case, space, hyphen, and underscore are ignored), per the
/// recommendation in [UAX #44 §4.2][uax44] for block-property values.
///
/// [uax44]: https://www.unicode.org/reports/tr44/#Matching_Rules
///
/// # Examples
///
/// ```
/// use unicode16_blocks::{block_by_name, BlockId};
///
/// assert_eq!(block_by_name("Cyrillic"), Some(BlockId::Cyrillic));
/// assert_eq!(block_by_name("cyrillic"), Some(BlockId::Cyrillic));
/// assert_eq!(block_by_name("cyrillic supplement"), Some(BlockId::CyrillicSupplement));
/// assert_eq!(block_by_name("Latin Extended-A"), Some(BlockId::LatinExtendedA));
/// assert_eq!(block_by_name("no such block"), None);
/// ```
pub fn block_by_name(name: &str) -> Option<BlockId> {
    // Exact names short-circuit before the loose comparison runs.
    BLOCKS
        .iter()
        .find(|b| b.name == name || names_match_loose(b.name, name))
        .map(|b| b.id)
}

/// Pure loose-match relation between two block names (UAX #44 style):
/// case, space, hyphen, and underscore are ignored; non-ASCII characters
/// can never match because no official block name contains one.
fn names_match_loose(a: &str, b: &str) -> bool {
    let mut ia = a
        .chars()
        .filter(|c| c.is_ascii() && !matches!(c, ' ' | '-' | '_'));
    let mut ib = b
        .chars()
        .filter(|c| c.is_ascii() && !matches!(c, ' ' | '-' | '_'));
    loop {
        match (ia.next(), ib.next()) {
            (Some(x), Some(y)) if x.eq_ignore_ascii_case(&y) => continue,
            (Some(_), Some(_)) => return false,
            (None, None) => return true,
            _ => return false,
        }
    }
}

// --------------------------------------------------------------------------
// Cipher-safety predicates (numeric code points, since ciphers usually
// operate on numbers before any char encoding happens).
// --------------------------------------------------------------------------

/// Is `cp` a UTF-16 surrogate (`U+D800..=U+DFFF`)?
///
/// Surrogates are invalid as scalar values: Rust's `char` can never hold
/// one, and most encoders (UTF-8, JSON, XML) reject or replace them.
/// They occupy the three blocks `High Surrogates`, `High Private Use
/// Surrogates`, and `Low Surrogates`.
#[inline]
pub const fn is_surrogate(cp: u32) -> bool {
    0xD800 <= cp && cp <= 0xDFFF
}

/// Is `cp` a noncharacter?
///
/// Noncharacters are permanently reserved for internal use and are never
/// valid in interchange: `U+FDD0..=U+FDEF` and every `U+xFFFE`/`U+xFFFF`.
/// They look like ordinary visible code points to naive filters, which
/// makes them a classic cipher-protocol bug (they survive str→bytes
/// round-trips on some platforms and are silently replaced on others).
#[inline]
pub const fn is_noncharacter(cp: u32) -> bool {
    (0xFDD0 <= cp && cp <= 0xFDEF) || (cp & 0xFFFE) == 0xFFFE
}

/// Is `cp` in a private-use area?
///
/// Private-use code points (`U+E000..=U+F8FF`, `U+F0000..=U+FFFFD`,
/// `U+100000..=U+10FFFD`) are *assigned by agreement*, not by the
/// standard. Two implementations can render the same character as
/// completely different glyphs, so ciphertext that leaks them into
/// user-visible text produces mojibake that no font or normalizer can fix.
#[inline]
pub const fn is_private_use(cp: u32) -> bool {
    (0xE000 <= cp && cp <= 0xF8FF)
        || (0xF0000 <= cp && cp <= 0xFFFFD)
        || (0x100000 <= cp && cp <= 0x10FFFD)
}

/// Is `cp` a variation selector (`U+FE00..=U+FE0F` or `U+E0100..=U+E01EF`)?
///
/// Variation selectors are combining marks that alter the preceding
/// character invisibly — a text-width channel. Cipher output that
/// contains them renders identical in a terminal but hashes differently,
/// which breaks signature verification and visual comparison.
#[inline]
pub const fn is_variation_selector(cp: u32) -> bool {
    (0xFE00 <= cp && cp <= 0xFE0F) || (0xE0100 <= cp && cp <= 0xE01EF)
}

/// Conservative "safe to put in a cipher alphabet / ciphertext" check.
///
/// Rejects a code point when any of the following holds:
///
/// * it is a surrogate ([`is_surrogate`]) — not a valid scalar value;
/// * it is a noncharacter ([`is_noncharacter`]) — never valid in interchange;
/// * it is private-use ([`is_private_use`]) — rendering is by agreement;
/// * it is a variation selector ([`is_variation_selector`]) — invisible;
/// * it is out of range (`> U+10FFFF`);
/// * it falls in a `No_Block` gap — those code points are unassigned.
///
/// Code points *inside* an allocated block can still be unassigned
/// (Unicode does not allocate every slot); excluding those needs
/// general-category data, which this crate deliberately does not carry.
/// For a cipher alphabet, prefer mapping onto [`BlockId`]s whose block is
/// fully assigned (e.g. `Byzantine Musical Symbols`, `Braille Patterns`),
/// or validate decoded output against your alphabet itself.
///
/// # Examples
///
/// ```
/// use unicode16_blocks::is_cipher_safe;
///
/// assert!(is_cipher_safe(0x0041));          // 'A'
/// assert!(!is_cipher_safe(0xD800));         // surrogate
/// assert!(!is_cipher_safe(0xFFFE));         // noncharacter
/// assert!(!is_cipher_safe(0xF8FF));         // private use
/// assert!(!is_cipher_safe(0xFE0F));         // variation selector
/// assert!(!is_cipher_safe(0x40000));        // No_Block gap (unassigned)
/// assert!(!is_cipher_safe(0x110000));       // out of range
/// ```
#[inline]
pub const fn is_cipher_safe(cp: u32) -> bool {
    if cp > 0x10_FFFF
        || is_surrogate(cp)
        || is_noncharacter(cp)
        || is_private_use(cp)
        || is_variation_selector(cp)
    {
        return false;
    }
    block_of_cp(cp).is_some()
}
