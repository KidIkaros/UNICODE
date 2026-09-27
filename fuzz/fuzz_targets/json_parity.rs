//! JSON parity: whatever `write_layout_json` emits must (a) be exactly
//! the byte count it reports, (b) parse as valid JSON, (c) parse back to
//! a layout with the declared symbol count, and (d) be reproducible from
//! a smaller buffer's exact `OutputTooSmall` report. This is the
//! invariant class that would have caught the extra-brace bug.

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicode16_blocks::{layout_from_json, write_layout_json, LayoutError, SourceDesc};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let name: String = String::from_utf8_lossy(&data[1..]).into_owned(); // arbitrary, must be escaped
    let source = if data[0] & 1 == 0 {
        SourceDesc::Block {
            block: "Runic",
            count: 24,
        }
    } else {
        // Includes a control character, a quote and a backslash to stress
        // the escaper.
        SourceDesc::Chars("ᚠᚢ𓂀\u{1}\"\\")
    };

    // Generous buffer: worst case is 6 bytes per name byte (\u00XX).
    let mut buf = vec![0u8; 8192 + name.len() * 6 + 64];
    let n = write_layout_json(&name, source, &mut buf).expect("buffer oversized");
    let s = std::str::from_utf8(&buf[..n]).expect("writer emits UTF-8");

    // (b) Valid JSON — the invariant the two-pass length arithmetic broke once.
    let parsed: serde_json::Value =
        serde_json::from_str(s).unwrap_or_else(|e| panic!("invalid JSON ({e}): {s:?}"));
    assert_eq!(parsed["version"], 1);
    assert_eq!(parsed["name"], name.as_str());

    // (c) Parses back to the declared symbol count.
    let layout = layout_from_json(s).expect("parser accepts writer output");
    let expected = match source {
        SourceDesc::Block { count, .. } => count as usize,
        SourceDesc::Chars(syms) => syms.chars().count(),
    };
    assert_eq!(layout.symbol_count(), expected);

    // (d) The exact-need report from a too-small buffer matches reality.
    let mut zero = [0u8; 0];
    match write_layout_json(&name, source, &mut zero) {
        Err(LayoutError::OutputTooSmall { needed }) => assert_eq!(needed, n),
        other => panic!("expected OutputTooSmall from empty buffer, got {other:?}"),
    }
});
