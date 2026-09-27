//! Dovahzul — the Skyrim dragon language — as Unicode.
//!
//! Dovahzul is written in 34 claw-runes. It has no official Unicode
//! block and no UCSUR assignment (thuum.org asked for one in 2015; as of
//! Unicode 16.0 nothing was allocated), so this module defines a
//! *documented private mapping* in Supplementary Private Use Area-A —
//! the range this crate already treats as the home of closed-system
//! cipher text.
//!
//! # The scheme
//!
//! 34 consecutive code points starting at [`RUNE_BASE`], in a stable
//! order:
//!
//! * indices 0..25 — the 25 single letters: the Latin alphabet minus C
//!   (Dovahzul has no C; loanwords spell it S or K), alphabetical —
//!   [`RUNIC_LETTERS`]
//! * indices 25..34 — the 9 digraph runes, alphabetical — [`DIGRAPHS`]:
//!   `aa ah ei ey ii ir oo ur uu`
//!
//! So rune *i* is `U+F0000 + i`: `A` → U+F0000, `D` → U+F0002, `aa` →
//! U+F0019. The whole alphabet as one string is [`RUNES`].
//!
//! # Layouts and cipher use
//!
//! [`dovahzul_layout`] builds the 34-symbol cipher layout. Private-use
//! code points are rejected by strict validation, so it is built with
//! [`Layout::new_permissive`]; the resulting big-int (base-34) layout
//! round-trips bytes like any other, and `layouts/dovahzul.json` ships
//! it as a shareable file:
//!
//! ```
//! use unicode16_blocks::dovahzul::dovahzul_layout;
//!
//! let layout = dovahzul_layout().unwrap();
//! assert_eq!(layout.symbol_count(), 34);
//!
//! let msg = b"Dovah";
//! let mut out = [0u32; 4096];
//! let n = layout.encode(msg, &mut out).unwrap();
//! let mut back = [0u8; 5]; // max_decoded_len(msg.len())
//! let m = layout.decode(&out[..n], &mut back).unwrap();
//! assert_eq!(&back[..m], msg);
//! ```
//!
//! # Transliteration
//!
//! [`transliterate`] converts Latin text to runes and [`derune`] back.
//! Digraph matching is greedy (`aaah` → the `aa` rune, then the `ah`
//! rune). Both directions are strict: only ASCII letters and spaces, no
//! C, no apostrophes, digits, or punctuation — matching the written
//! language, and keeping the mapping reversible.
//!
//! ```
//! use unicode16_blocks::dovahzul::{derune, transliterate};
//!
//! let runes = transliterate("Drem Yol Lok").unwrap();
//! assert_eq!(derune(&runes).unwrap(), "drem yol lok");
//! ```
//!
//! # Why private use, and why permissive
//!
//! Strict validation ([`Layout::new`]) rejects private use because PUA
//! text is meaningless on a system that lacks the mapping. This module
//! *is* the mapping — versioned in code and in `layouts/dovahzul.json` —
//! so the runes survive anywhere this crate exists. That is exactly the
//! closed-system case [`Layout::new_permissive`] documents, hence
//! [`layout_from_json_permissive`](crate::layout_json) and the CLI's
//! `--permissive` flag.

use crate::layout::{Layout, LayoutError, Symbols};
#[cfg(feature = "std")]
use crate::std::string::String;

/// First code point of the Dovahzul rune range: the scheme occupies
/// `U+F0000..=U+F0021` in Supplementary Private Use Area-A.
pub const RUNE_BASE: u32 = 0xF_0000;

/// Number of runes in the scheme: 25 single letters + 9 digraphs.
pub const RUNE_COUNT: usize = 34;

/// The Latin letters that have their own rune — the alphabet minus C —
/// in alphabetical order. Letter at index *i* is rune `U+F0000 + i`.
pub const RUNIC_LETTERS: &str = "ABDEFGHIJKLMNOPQRSTUVWXYZ";

/// The digraph runes, alphabetical. Digraph at index *i* is rune
/// `U+F0000 + 25 + i`.
pub const DIGRAPHS: [&str; 9] = ["aa", "ah", "ei", "ey", "ii", "ir", "oo", "ur", "uu"];

/// The 34 dragon runes as one symbol string, in layout order: the 25
/// single letters ([`RUNIC_LETTERS`]) then the 9 digraphs ([`DIGRAPHS`]).
/// Symbol *i* of this string encodes value *i* in [`dovahzul_layout`].
pub const RUNES: &str = concat!(
    "\u{F0000}\u{F0001}\u{F0002}\u{F0003}\u{F0004}\u{F0005}\u{F0006}\u{F0007}",
    "\u{F0008}\u{F0009}\u{F000A}\u{F000B}\u{F000C}\u{F000D}\u{F000E}\u{F000F}",
    "\u{F0010}\u{F0011}\u{F0012}\u{F0013}\u{F0014}\u{F0015}\u{F0016}\u{F0017}",
    "\u{F0018}\u{F0019}\u{F001A}\u{F001B}\u{F001C}\u{F001D}\u{F001E}\u{F001F}",
    "\u{F0020}\u{F0021}",
);

/// The rune at scheme index `index` (0..34): `U+F0000 + index`.
///
/// # Panics
///
/// Panics if `index >= RUNE_COUNT` — the scheme has exactly 34 runes.
pub fn rune(index: usize) -> char {
    char::from_u32(RUNE_BASE + index as u32).expect("index < 34 is a valid scalar")
}

/// Is `cp` one of the 34 Dovahzul rune code points?
pub fn is_dovahzul_rune(cp: u32) -> bool {
    (RUNE_BASE..RUNE_BASE + RUNE_COUNT as u32).contains(&cp)
}

/// The Latin letter or digraph that `rune` stands for, or `None` if it
/// is not a Dovahzul rune. Spaces are not runes — [`derune`] passes
/// them through separately.
pub fn latin_for(rune: char) -> Option<&'static str> {
    let cp = rune as u32;
    if !(RUNE_BASE..RUNE_BASE + RUNE_COUNT as u32).contains(&cp) {
        return None;
    }
    let index = (cp - RUNE_BASE) as usize;
    if index < RUNIC_LETTERS.len() {
        // ASCII, so byte offsets are char offsets and the slice is 'static.
        Some(&RUNIC_LETTERS[index..index + 1])
    } else {
        Some(DIGRAPHS[index - RUNIC_LETTERS.len()])
    }
}

/// The Dovahzul cipher layout: all 34 dragon runes, built with
/// [`Layout::new_permissive`] (private use is the point of the scheme).
/// 34 is not a power of two, so the layout encodes in big-int mode.
pub fn dovahzul_layout() -> Result<Layout, LayoutError> {
    Layout::new_permissive(Symbols::Chars(RUNES))
}

/// What can go wrong in Dovahzul transliteration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DovahzulError {
    /// A character that is neither an ASCII letter nor a space, at byte
    /// `offset`. Dovahzul writing has no digits or punctuation.
    UnsupportedChar { offset: usize, c: char },
    /// The letter C, which Dovahzul lacks, at byte `offset` — spell it
    /// S or K (the language's own convention) and try again.
    LetterC { offset: usize },
    /// A character that is not a Dovahzul rune or a space, at byte
    /// `offset`.
    NotARune { offset: usize, c: char },
}

impl core::fmt::Display for DovahzulError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            DovahzulError::UnsupportedChar { offset, c } => {
                write!(f, "byte {offset}: {c:?} is not a letter or space")
            }
            DovahzulError::LetterC { offset } => {
                write!(f, "byte {offset}: Dovahzul has no C — spell it S or K")
            }
            DovahzulError::NotARune { offset, c } => {
                write!(f, "byte {offset}: {c:?} is not a Dovahzul rune")
            }
        }
    }
}

/// Transliterates Latin text into Dovahzul runes. Digraphs are matched
/// greedily (`aaah` → `aa` + `ah`); letters are lowercased; spaces pass
/// through.
///
/// # Errors
///
/// [`DovahzulError::LetterC`] for C (spell it S or K);
/// [`DovahzulError::UnsupportedChar`] for anything that is not an ASCII
/// letter or a space.
#[cfg(feature = "std")]
pub fn transliterate(text: &str) -> Result<String, DovahzulError> {
    let mut out = String::new();
    let mut chars = text.char_indices().peekable();
    while let Some((offset, c)) = chars.next() {
        if c == ' ' {
            out.push(' ');
            continue;
        }
        let l = c.to_ascii_lowercase();
        if !l.is_ascii_alphabetic() {
            return Err(DovahzulError::UnsupportedChar { offset, c });
        }
        if l == 'c' {
            return Err(DovahzulError::LetterC { offset });
        }
        // Greedy digraph: `aa` wins over `a` + `a`.
        let mut digraph = false;
        if let Some((_, next)) = chars.peek() {
            let n = next.to_ascii_lowercase();
            if n.is_ascii_alphabetic() {
                if let Some(i) = DIGRAPHS
                    .iter()
                    .position(|d| d.as_bytes() == [l as u8, n as u8])
                {
                    out.push(rune(RUNIC_LETTERS.len() + i));
                    chars.next(); // consume the digraph's second letter
                    digraph = true;
                }
            }
        }
        if !digraph {
            // RUNIC_LETTERS is written uppercase; `l` is lowercased.
            let i = RUNIC_LETTERS
                .find(l.to_ascii_uppercase())
                .expect("every letter except C is in RUNIC_LETTERS");
            out.push(rune(i));
        }
    }
    Ok(out)
}

/// Transliterates Dovahzul runes back into lowercase Latin text.
/// Spaces pass through; every rune expands to its letter or digraph.
///
/// # Errors
///
/// [`DovahzulError::NotARune`] for a character that is not a rune or a
/// space.
#[cfg(feature = "std")]
pub fn derune(runes: &str) -> Result<String, DovahzulError> {
    let mut out = String::new();
    for (offset, c) in runes.char_indices() {
        if c == ' ' {
            out.push(' ');
        } else if let Some(latin) = latin_for(c) {
            // latin_for returns canonical form (uppercase singles); the
            // round-trip contract is lowercase text.
            out.push_str(&latin.to_ascii_lowercase());
        } else {
            return Err(DovahzulError::NotARune { offset, c });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::is_private_use;
    use std::vec::Vec;

    #[test]
    fn runes_are_34_consecutive_pua_code_points() {
        let cps: Vec<u32> = RUNES.chars().map(|c| c as u32).collect();
        assert_eq!(cps.len(), RUNE_COUNT);
        for (i, &cp) in cps.iter().enumerate() {
            assert_eq!(cp, RUNE_BASE + i as u32);
            assert!(is_private_use(cp), "U+{cp:04X} not private use");
            assert!(is_dovahzul_rune(cp));
        }
        for i in 0..cps.len() {
            for j in 0..i {
                assert_ne!(cps[i], cps[j]);
            }
        }
    }

    #[test]
    fn mapping_is_aligned_with_the_tables() {
        assert_eq!(rune(0), '\u{F0000}'); // the 'A' slot
        assert_eq!(latin_for(rune(0)), Some("A"));
        assert_eq!(latin_for(rune(1)), Some("B"));
        assert_eq!(latin_for(rune(2)), Some("D")); // C does not exist
        assert_eq!(latin_for(rune(25)), Some("aa"));
        assert_eq!(latin_for(rune(33)), Some("uu"));
        assert_eq!(latin_for('\u{F0022}'), None);
        assert_eq!(latin_for('ᚠ'), None);
        assert_eq!(latin_for(' '), None);
    }

    #[test]
    fn layout_is_bigint_base_34() {
        let l = dovahzul_layout().unwrap();
        assert_eq!(l.symbol_count(), 34);
        assert_eq!(l.symbol(0), RUNE_BASE);
        assert_eq!(l.symbol(33), RUNE_BASE + 33);
        assert!(l.bits_per_symbol().is_none());
        assert!(!l.supports_direct_byte_mapping());
    }

    #[test]
    fn strict_validation_rejects_the_layout() {
        assert!(matches!(
            Layout::new(Symbols::Chars(RUNES)),
            Err(LayoutError::UnsafeCodePoint {
                index: 0,
                cp: RUNE_BASE
            })
        ));
    }

    #[test]
    fn bytes_round_trip() {
        let l = dovahzul_layout().unwrap();
        let bytes: Vec<u8> = (0..512u32).map(|i| (i * 7 + 13) as u8).collect();
        let mut out = vec![0u32; l.max_encoded_len(bytes.len())];
        let n = l.encode(&bytes, &mut out).unwrap();
        let mut back = vec![0u8; l.max_decoded_len(n)];
        let m = l.decode(&out[..n], &mut back).unwrap();
        assert_eq!(&back[..m], &bytes);
    }

    #[test]
    fn transliterate_round_trips() {
        let runes = transliterate("Drem Yol Lok").unwrap();
        assert_eq!(runes.chars().count(), 12); // 10 letters + 2 spaces
        assert_eq!(derune(&runes).unwrap(), "drem yol lok");
    }

    #[test]
    fn digraphs_are_greedy_and_reversible() {
        let runes = transliterate("aaah").unwrap();
        assert_eq!(runes.chars().count(), 2); // "aa" + "ah", not a+a+ah
        assert_eq!(latin_for(runes.chars().next().unwrap()), Some("aa"));
        assert_eq!(latin_for(runes.chars().nth(1).unwrap()), Some("ah"));
        for d in DIGRAPHS {
            let r = transliterate(d).unwrap();
            assert_eq!(r.chars().count(), 1, "{d} must map to one rune");
            assert_eq!(derune(&r).unwrap(), d);
        }
    }

    #[test]
    fn errors_are_typed_and_positioned() {
        assert!(matches!(
            transliterate("C"),
            Err(DovahzulError::LetterC { offset: 0 })
        ));
        assert!(matches!(
            transliterate("x3"),
            Err(DovahzulError::UnsupportedChar { offset: 1, c: '3' })
        ));
        assert!(matches!(
            derune("ᚠ"),
            Err(DovahzulError::NotARune {
                offset: 0, c: 'ᚠ'
            })
        ));
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_strict_rejects_permissive_accepts() {
        use crate::{layout_from_json, layout_from_json_permissive, write_layout_json, SourceDesc};

        let mut buf = vec![0u8; 4096];
        let n = write_layout_json("dovahzul", SourceDesc::Chars(RUNES), &mut buf).unwrap();
        let json = core::str::from_utf8(&buf[..n]).unwrap();

        assert!(layout_from_json(json).is_err());
        let l = layout_from_json_permissive(json).unwrap();
        assert_eq!(l.symbol_count(), 34);
        assert_eq!(l.symbol(0), RUNE_BASE);
    }
}
