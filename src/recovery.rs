//! Recovery-alphabet checks: what a symbol set must satisfy to back a
//! recovery phrase, and how long a phrase it produces.
//!
//! A recovery phrase is a seed backup, so its alphabet has a stricter
//! contract than a general cipher alphabet. [`is_cipher_safe`](crate::is_cipher_safe)
//! is not sufficient on its own: it rejects surrogates, noncharacters,
//! private use, variation selectors, and `No_Block` gaps, but it deliberately
//! carries no general-category data — so it accepts every control character,
//! every invisible format character, and every combining mark, because those
//! live *inside* allocated blocks.
//!
//! [`recovery_safe`] composes the missing checks on top of it, so a layout can
//! be validated here before it is used as a recovery alphabet.
//!
//! This check is independent of the SDK's codepoint policy and is deliberately
//! *not* kept in lockstep with it: the SDK is authoritative, so a layout that
//! passes here may still be refused by the SDK, and where the two disagree the
//! SDK's verdict is the one that governs a phrase.
//!
//! ## Residual limitation
//!
//! Only combining ranges *dedicated* to combining marks are listed below.
//! Mixed script blocks (Tibetan, Devanagari, Arabic, ...) also contain them,
//! and separating those requires general-category data this crate does not
//! carry. For a recovery alphabet, prefer symbol blocks that are wholly
//! assigned and non-combining (Braille Patterns, Runic, CJK Unified
//! Ideographs) and validate the decoded output against the alphabet itself.

use crate::MAX_LAYOUT_LEN;

/// Longest phrase the recovery codec will produce.
///
/// Beyond this a phrase cannot be transcribed or checked by hand, so an
/// alphabet that needs more symbols is not a usable recovery alphabet.
pub const MAX_PHRASE_SYMBOLS: usize = 64;

/// Entropy widths the recovery codec accepts, in bytes.
pub const SUPPORTED_ENTROPY_BYTES: [usize; 3] = [16, 24, 32];

/// Limbs needed to compare `radix^digits` against the largest message (288
/// bits). Ten limbs is 320 bits, above any recovery message.
const POWER_LIMBS: usize = 10;

/// Is `cp` acceptable as a symbol of a **recovery** alphabet?
///
/// Rejects everything [`is_cipher_safe`](crate::is_cipher_safe) rejects, plus
/// control characters, invisible format characters, combining marks in the
/// ranges dedicated to them, the variation-selector supplement, the tag
/// characters, and whitespace (which cannot survive the canonical phrase
/// text form).
///
/// Usable in `const` contexts.
///
/// # Examples
///
/// ```
/// use unicode16_blocks::{is_cipher_safe, recovery_safe};
///
/// // Both accept an ordinary symbol.
/// assert!(recovery_safe(0x16A0)); // ᚠ RUNIC LETTER FEHU
///
/// // Block coverage alone is not enough: NUL sits inside Basic Latin.
/// assert!(is_cipher_safe(0x0000));
/// assert!(!recovery_safe(0x0000));
///
/// // Invisible characters are rejected even though they are assigned.
/// assert!(!recovery_safe(0x200D)); // ZWJ
/// assert!(!recovery_safe(0x0301)); // COMBINING ACUTE ACCENT
/// ```
#[inline]
pub const fn recovery_safe(cp: u32) -> bool {
    if !crate::is_cipher_safe(cp) {
        return false;
    }
    // C0 and C1 controls.
    if cp <= 0x1F || (cp >= 0x7F && cp <= 0x9F) {
        return false;
    }
    // Unicode White_Space that is not a control (C0/C1 whitespace was
    // rejected above): the canonical phrase text separates symbols with
    // whitespace and parsing drops it, so a whitespace symbol would silently
    // vanish from its own phrase.
    if cp == 0x0020
        || cp == 0x00A0
        || cp == 0x1680
        || (cp >= 0x2000 && cp <= 0x200A)
        || cp == 0x2028
        || cp == 0x2029
        || cp == 0x202F
        || cp == 0x205F
        || cp == 0x3000
    {
        return false;
    }
    // Soft hyphen: renders as nothing but is a distinct codepoint.
    if cp == 0x00AD {
        return false;
    }
    // Combining marks, in the ranges dedicated to them.
    if (cp >= 0x0300 && cp <= 0x036F)
        || (cp >= 0x0483 && cp <= 0x0489)
        || (cp >= 0x1AB0 && cp <= 0x1AFF)
        || (cp >= 0x1DC0 && cp <= 0x1DFF)
        || (cp >= 0x20D0 && cp <= 0x20FF)
        || (cp >= 0xFE20 && cp <= 0xFE2F)
    {
        return false;
    }
    // Variation Selectors Supplement, and the tag characters.
    if (cp >= 0xE0100 && cp <= 0xE01EF) || (cp >= 0xE0000 && cp <= 0xE007F) {
        return false;
    }
    // Joiners, BIDI controls, and the default-ignorable format ranges.
    if cp == 0x061C
        || cp == 0x180E
        || cp == 0x200B
        || cp == 0x200C
        || cp == 0x200D
        || cp == 0x200E
        || cp == 0x200F
        || cp == 0xFEFF
        || (cp >= 0x202A && cp <= 0x202E)
        || (cp >= 0x2060 && cp <= 0x206F)
    {
        return false;
    }
    true
}

/// Bits in `entropy ‖ checksum` for this entropy width.
///
/// The checksum is one bit per byte of entropy, so a 32-byte seed carries a
/// 32-bit checksum and the message is 36 bytes.
///
/// Returns `None` for a width outside [`SUPPORTED_ENTROPY_BYTES`].
pub const fn recovery_message_bits(entropy_bytes: usize) -> Option<usize> {
    if entropy_bytes == 16 || entropy_bytes == 24 || entropy_bytes == 32 {
        Some((entropy_bytes + entropy_bytes / 8) * 8)
    } else {
        None
    }
}

/// Symbols a phrase needs for this alphabet and entropy width.
///
/// Exact: the smallest `digits` with `radix^digits >= 2^message_bits`. Returns
/// `None` if the alphabet size is out of range or the entropy width is
/// unsupported.
///
/// This mirrors the SDK recovery codec's sizing; the values are pinned
/// against the SDK's committed golden vectors in the tests below.
pub fn recovery_symbol_count(radix: usize, entropy_bytes: usize) -> Option<usize> {
    if !(2..=MAX_LAYOUT_LEN).contains(&radix) {
        return None;
    }
    let bits = recovery_message_bits(entropy_bytes)?;
    if radix.is_power_of_two() {
        let width = radix.trailing_zeros() as usize;
        return Some(crate::layout::ceil_div(bits, width));
    }
    let mut digits = 1usize;
    while !power_covers(radix, digits, bits) {
        digits += 1;
    }
    Some(digits)
}

/// Whether this alphabet can back a recovery phrase at any supported width.
///
/// True when at least one width yields a phrase of at most
/// [`MAX_PHRASE_SYMBOLS`] symbols.
pub fn recovery_usable(radix: usize) -> bool {
    SUPPORTED_ENTROPY_BYTES.iter().any(|&entropy_bytes| {
        matches!(recovery_symbol_count(radix, entropy_bytes), Some(n) if n <= MAX_PHRASE_SYMBOLS)
    })
}

/// First symbol that is not [`recovery_safe`], as `(index, codepoint)`.
///
/// `None` means every symbol passed.
pub fn first_unsafe_symbol(symbols: &[u32]) -> Option<(usize, u32)> {
    symbols
        .iter()
        .enumerate()
        .find(|(_, &cp)| !recovery_safe(cp))
        .map(|(index, &cp)| (index, cp))
}

/// Whether `radix^exponent >= 2^bits`, computed exactly.
fn power_covers(radix: usize, exponent: usize, bits: usize) -> bool {
    let mut limbs = [0u32; POWER_LIMBS];
    limbs[0] = 1;
    for _ in 0..exponent {
        let mut carry = 0u64;
        for limb in limbs.iter_mut() {
            let product = (*limb as u64) * (radix as u64) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            // Past 320 bits, which is above any recovery message.
            return true;
        }
    }
    !fits_bits(&limbs, bits)
}

/// Whether a little-endian limb value is below `2^bits`.
fn fits_bits(limbs: &[u32; POWER_LIMBS], bits: usize) -> bool {
    let whole = bits / 32;
    let partial = bits % 32;
    for (index, &limb) in limbs.iter().enumerate() {
        if index < whole {
            continue;
        }
        let in_range = if index == whole && partial != 0 {
            limb >> partial == 0
        } else {
            limb == 0
        };
        if !in_range {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::is_cipher_safe;

    #[test]
    fn rejects_what_block_coverage_accepts() {
        // Every one of these is inside an allocated block, so `is_cipher_safe`
        // accepts it; a recovery alphabet must not.
        for cp in [
            0x0000u32, 0x0007, 0x0009, 0x001B, 0x007F, 0x0085, // controls
            0x00AD, // soft hyphen
            0x200D, 0x200E, 0x200F, 0x2028, 0x2029, 0xFEFF, // format
            0x0020, 0x00A0, 0x1680, 0x2007, 0x202F, 0x205F, 0x3000, // whitespace
            0x0301, 0x0483, 0x1AB0, 0x1DC0, 0x20D0, 0xFE20,  // combining
            0xE0041, // tag character
        ] {
            assert!(is_cipher_safe(cp), "U+{cp:04X} should be block-covered");
            assert!(!recovery_safe(cp), "U+{cp:04X} must not be recovery-safe");
        }
    }

    #[test]
    fn accepts_symbols_and_scripts() {
        for cp in [
            0x16A0u32, 0x16FF, // Runic
            0x4E00, 0x9FFF, // CJK
            0x2800, 0x28FF, // Braille
            0x41, 0x30, // Basic Latin letters/digits
        ] {
            assert!(recovery_safe(cp), "U+{cp:04X} should be recovery-safe");
        }
    }

    #[test]
    fn still_rejects_what_cipher_safety_rejects() {
        for cp in [
            0xD800u32, 0xFFFE, 0xF8FF, 0xFE0F, 0xE0100, 0x40000, 0x110000,
        ] {
            assert!(!recovery_safe(cp), "U+{cp:04X}");
        }
    }

    /// The sizing must agree with the SDK recovery codec's committed golden
    /// vectors, which are independently reproduced by a Python
    /// implementation of the same format.
    #[test]
    fn symbol_counts_match_the_sdk_golden_vectors() {
        // (alphabet size, entropy bytes, symbols in the SDK's golden vector)
        let pinned = [
            (24usize, 32usize, 63usize),
            (33, 16, 29),
            (256, 16, 18),
            (4096, 32, 24),
            (2048, 32, 27),
            (2048, 16, 14),
            (16, 16, 36),
        ];
        for (radix, entropy_bytes, expected) in pinned {
            assert_eq!(
                recovery_symbol_count(radix, entropy_bytes),
                Some(expected),
                "radix {radix}, {entropy_bytes} bytes"
            );
        }
    }

    #[test]
    fn message_bits_match_the_sdk_layout() {
        assert_eq!(recovery_message_bits(16), Some(144));
        assert_eq!(recovery_message_bits(24), Some(216));
        assert_eq!(recovery_message_bits(32), Some(288));
        assert_eq!(recovery_message_bits(20), None);
        assert_eq!(recovery_message_bits(0), None);
    }

    #[test]
    fn rejects_unusable_alphabet_sizes() {
        assert_eq!(recovery_symbol_count(1, 16), None);
        assert_eq!(recovery_symbol_count(0, 16), None);
        assert_eq!(recovery_symbol_count(MAX_LAYOUT_LEN + 1, 16), None);
        // 2 symbols needs 144 at the smallest width: not usable.
        assert!(!recovery_usable(2));
        // 16 symbols is over the limit at 256-bit (72) but fine at 128-bit.
        assert!(recovery_usable(16));
        assert!(matches!(recovery_symbol_count(16, 32), Some(n) if n > MAX_PHRASE_SYMBOLS));
        assert!(recovery_usable(24));
        assert!(recovery_usable(33));
        assert!(recovery_usable(256));
        assert!(recovery_usable(4096));
    }

    #[test]
    fn power_covers_is_exact_at_boundaries() {
        // 24^63 >= 2^288 but 24^62 < 2^288.
        assert!(power_covers(24, 63, 288));
        assert!(!power_covers(24, 62, 288));
        // 3^5 = 243 >= 128 but 3^4 = 81 < 128.
        assert!(power_covers(3, 5, 7));
        assert!(!power_covers(3, 4, 7));
        // Exact powers of two.
        assert!(power_covers(2, 288, 288));
        assert!(!power_covers(2, 287, 288));
    }

    #[test]
    fn finds_the_first_unsafe_symbol() {
        assert_eq!(first_unsafe_symbol(&[0x16A0, 0x16A1]), None);
        assert_eq!(first_unsafe_symbol(&[0x16A0, 0x0000]), Some((1, 0x0000)));
        assert_eq!(first_unsafe_symbol(&[]), None);
    }
}
