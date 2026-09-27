//! JSON (de)serialization for layouts.
//!
//! The crate side *emits* JSON into a caller buffer with no allocation
//! and no dependencies ([`write_layout_json`]). Parsing needs owned
//! strings, so [`layout_from_json`] lives behind the `json` feature and
//! is built on `serde_json` — the CLI uses it; effects stay at the edges.

use crate::layout::LayoutError;
#[cfg(feature = "json")]
use crate::layout::{Layout, Symbols, MAX_LAYOUT_LEN};
#[cfg(feature = "json")]
use crate::std::borrow::ToOwned;

/// How the symbols of a layout are described in a layout file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceDesc<'a> {
    /// First `count` code points of the named block.
    Block { block: &'a str, count: u32 },
    /// The symbols themselves, in alphabet order.
    Chars(&'a str),
}

/// Serializes a layout description to JSON into `out`. Returns the byte
/// length written.
///
/// `name` is free-form metadata (escaped on output). The result is a
/// single JSON object of the form:
///
/// ```json
/// {"version":1,"name":"runic","source":{"type":"block","block":"Runic","count":24}}
/// ```
///
/// # Errors
///
/// [`LayoutError::OutputTooSmall`] with the exact needed size if `out`
/// is too small; the buffer is left untouched (two-pass write).
pub fn write_layout_json(
    name: &str,
    source: SourceDesc<'_>,
    out: &mut [u8],
) -> Result<usize, LayoutError> {
    // First pass: exact length, so a too-small buffer is rejected whole.
    let needed = layout_json_len(name, &source);
    if out.len() < needed {
        return Err(LayoutError::OutputTooSmall { needed });
    }
    let mut w = JsonWriter { buf: out, pos: 0 };
    w.write(b"{\"version\":1,\"name\":");
    w.write_json_string(name);
    w.write(b",\"source\":");
    match source {
        SourceDesc::Block { block, count } => {
            w.write(b"{\"type\":\"block\",\"block\":");
            w.write_json_string(block);
            w.write(b",\"count\":");
            let mut num = [0u8; 10];
            w.write(itoa(count as u64, &mut num));
            w.write(b"}");
        }
        SourceDesc::Chars(symbols) => {
            w.write(b"{\"type\":\"chars\",\"symbols\":");
            w.write_json_string(symbols);
            w.write(b"}");
        }
    }
    w.write(b"}");
    debug_assert_eq!(w.pos, needed);
    Ok(w.pos)
}

/// Exact JSON byte length for a layout description.
fn layout_json_len(name: &str, source: &SourceDesc<'_>) -> usize {
    let mut n = "{\"version\":1,\"name\":".len() + json_string_len(name) + ",\"source\":".len();
    match source {
        SourceDesc::Block { block, count } => {
            let mut num = [0u8; 10];
            n += "{\"type\":\"block\",\"block\":".len()
                + json_string_len(block)
                + ",\"count\":".len()
                + itoa(*count as u64, &mut num).len()
                + "}".len();
        }
        SourceDesc::Chars(symbols) => {
            n += "{\"type\":\"chars\",\"symbols\":".len() + json_string_len(symbols) + "}".len();
        }
    }
    n + "}".len()
}

/// Parses and validates a layout from its JSON representation.
///
/// Accepts the same shape [`write_layout_json`] produces. Block sources
/// are resolved by (loose) name via [`crate::block_by_name`]; char
/// sources are used as-is. The result is fully validated exactly as
/// [`Layout::new`] would validate it.
///
/// Requires the `json` feature.
#[cfg(feature = "json")]
pub fn layout_from_json(json: &str) -> Result<Layout, LayoutError> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|_| LayoutError::Corrupted)?;

    let source = v.get("source").ok_or(LayoutError::Corrupted)?;
    let kind = source
        .get("type")
        .and_then(|t| t.as_str())
        .ok_or(LayoutError::Corrupted)?;

    let symbols = match kind {
        "block" => {
            let block_name = source
                .get("block")
                .and_then(|b| b.as_str())
                .ok_or(LayoutError::Corrupted)?;
            let count = source
                .get("count")
                .and_then(|c| c.as_u64())
                .ok_or(LayoutError::Corrupted)?;
            if count > MAX_LAYOUT_LEN as u64 {
                return Err(LayoutError::TooManySymbols {
                    len: count as usize,
                });
            }
            let id = crate::block_by_name(block_name).ok_or(LayoutError::Corrupted)?;
            Symbols::Block {
                block: id,
                count: count as u32,
            }
        }
        "chars" => {
            let s = source
                .get("symbols")
                .and_then(|s| s.as_str())
                .ok_or(LayoutError::Corrupted)?;
            Symbols::String(s.to_owned())
        }
        _ => return Err(LayoutError::Corrupted),
    };

    Layout::new(symbols)
}

// -- JSON string helpers (no_std, allocation-free) ---------------------------

/// Length `s` will occupy inside a JSON string literal (escapes counted).
fn json_string_len(s: &str) -> usize {
    2 + s
        .chars()
        .map(|c| match c {
            '"' | '\\' | '\n' | '\r' | '\t' | '\u{08}' | '\u{0C}' => 2,
            c if (c as u32) < 0x20 => 6, // \u00XX
            _ => c.len_utf8(),
        })
        .sum::<usize>()
}

struct JsonWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl JsonWriter<'_> {
    fn write(&mut self, bytes: &[u8]) {
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
    }

    fn write_json_string(&mut self, s: &str) {
        self.buf[self.pos] = b'"';
        self.pos += 1;
        for c in s.chars() {
            match c {
                '"' | '\\' => {
                    self.buf[self.pos] = b'\\';
                    self.buf[self.pos + 1] = c as u8;
                    self.pos += 2;
                }
                '\n' | '\r' | '\t' | '\u{08}' | '\u{0C}' => {
                    let esc: &[u8; 2] = match c {
                        '\n' => b"\\n",
                        '\r' => b"\\r",
                        '\t' => b"\\t",
                        '\u{08}' => b"\\b",
                        _ => b"\\f",
                    };
                    self.buf[self.pos..self.pos + 2].copy_from_slice(esc);
                    self.pos += 2;
                }
                c if (c as u32) < 0x20 => {
                    self.buf[self.pos] = b'\\';
                    self.buf[self.pos + 1] = b'u';
                    self.buf[self.pos + 2..self.pos + 6].copy_from_slice(&format_hex4(c as u32));
                    self.pos += 6;
                }
                c => {
                    let mut tmp = [0u8; 4];
                    let enc = c.encode_utf8(&mut tmp);
                    self.write(enc.as_bytes());
                }
            }
        }
        self.buf[self.pos] = b'"';
        self.pos += 1;
    }
}

/// Formats `v` as a 4-hex-digit uppercase group, e.g. `001F`.
fn format_hex4(v: u32) -> [u8; 4] {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    [
        HEX[((v >> 12) & 0xF) as usize],
        HEX[((v >> 8) & 0xF) as usize],
        HEX[((v >> 4) & 0xF) as usize],
        HEX[(v & 0xF) as usize],
    ]
}

/// Writes the decimal form of `v` into `buf` (which must be at least as
/// long as the decimal representation of `v`) and returns the digit slice.
fn itoa(v: u64, buf: &mut [u8; 10]) -> &[u8] {
    let mut tmp = [0u8; 20];
    let mut i = tmp.len();
    let mut v = v;
    if v == 0 {
        i -= 1;
        tmp[i] = b'0';
    }
    while v > 0 {
        i -= 1;
        tmp[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    let digits = &tmp[i..];
    buf[..digits.len()].copy_from_slice(digits);
    &buf[..digits.len()]
}

#[cfg(all(test, feature = "json"))]
mod tests {
    use super::*;

    #[test]
    fn writer_len_matches_written() {
        let mut buf = vec![0u8; MAX_LAYOUT_LEN * 4 + 256];
        let n = write_layout_json(
            "my cipher alphabet",
            SourceDesc::Block {
                block: "Runic",
                count: 24,
            },
            &mut buf,
        )
        .unwrap();
        let s = core::str::from_utf8(&buf[..n]).unwrap();
        assert!(s.contains("\"type\":\"block\""));
        assert!(s.contains("\"count\":24"));
        assert!(s.contains("my cipher alphabet"));
        assert!(serde_json::from_str::<serde_json::Value>(s).is_ok());
    }

    #[test]
    fn escaping_round_trips_through_serde() {
        let mut buf = vec![0u8; MAX_LAYOUT_LEN * 4 + 256];
        let n = write_layout_json(
            "quote\"back\\nl\u{1}",
            SourceDesc::Chars("ᚠ\u{2}ᚢ"),
            &mut buf,
        )
        .unwrap();
        let s = core::str::from_utf8(&buf[..n]).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(s).unwrap();
        assert_eq!(parsed["name"], "quote\"back\\nl\u{1}");
        assert_eq!(parsed["source"]["symbols"], "ᚠ\u{2}ᚢ");
    }

    #[test]
    fn too_small_buffer_reports_exact_need() {
        let mut buf = [0u8; 32];
        match write_layout_json(
            "x",
            SourceDesc::Block {
                block: "Runic",
                count: 24,
            },
            &mut buf,
        ) {
            Err(LayoutError::OutputTooSmall { needed }) => {
                let real = layout_json_len(
                    "x",
                    &SourceDesc::Block {
                        block: "Runic",
                        count: 24,
                    },
                );
                assert_eq!(needed, real);
            }
            other => panic!("expected OutputTooSmall, got {other:?}"),
        }
    }

    #[test]
    fn parser_round_trips_block_source() {
        let mut buf = vec![0u8; 4096];
        let n = write_layout_json(
            "t",
            SourceDesc::Block {
                block: "Runic",
                count: 24,
            },
            &mut buf,
        )
        .unwrap();
        let json = core::str::from_utf8(&buf[..n]).unwrap();
        let l = layout_from_json(json).unwrap();
        assert_eq!(l.symbol_count(), 24);
        assert_eq!(l.symbol(0), 0x16A0);
    }

    #[test]
    fn parser_round_trips_chars_source() {
        let mut buf = vec![0u8; 4096];
        let n = write_layout_json("t", SourceDesc::Chars("ᚠᚢᚦ"), &mut buf).unwrap();
        let json = core::str::from_utf8(&buf[..n]).unwrap();
        let l = layout_from_json(json).unwrap();
        assert_eq!(l.symbol_count(), 3);
        assert_eq!(l.symbol(2), 0x16A6);
    }

    #[test]
    fn parser_rejects_garbage() {
        assert!(layout_from_json("not json").is_err());
        assert!(layout_from_json("{\"version\":1}").is_err());
        assert!(layout_from_json(
            "{\"source\":{\"type\":\"block\",\"block\":\"Nope\",\"count\":2}}"
        )
        .is_err());
        // Unsafe symbols (private use) must fail validation: serialize a
        // PUA alphabet (emitting is agnostic) and parse it back.
        let mut buf = vec![0u8; 4096];
        let n = write_layout_json("t", SourceDesc::Chars("ᚠ\u{E000}"), &mut buf).unwrap();
        let json = core::str::from_utf8(&buf[..n]).unwrap();
        assert!(layout_from_json(json).is_err());
    }
}
