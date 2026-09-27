# Sample layouts

Ready-made cipher-alphabet layouts. Each file is a JSON layout in the
format documented in the main README. Validate one, then encode:

```bash
cargo build --features json --bin layout
BIN=target/debug/layout
$BIN validate layouts/braille-256.json
$BIN encode layouts/braille-256.json --in message.bin --out cipher.txt
$BIN decode layouts/braille-256.json --in cipher.txt --out message.bin
```

Every layout below was verified by encoding 4,096 random bytes and
decoding back byte-identical; the size columns are measured from that run.

| File | Symbols | Source | Mode | Cipher symbols | Cipher bytes (UTF-8) | Look |
|---|---|---|---|---|---|---|
| `braille-256.json` | 256 | Braille Patterns | direct | 4,096 | 12,288 (3.0×) | dense dotted cells |
| `tibetan-256.json` | 256 | Tibetan | direct | 4,096 | 12,288 (3.0×) | script-like text |
| `byzantine-256.json` | 256 | Byzantine Musical Symbols | direct | 4,096 | 16,388 (4.0×) | musical neumes |
| `cjk-4096.json` | 4,096 | CJK Unified Ideographs (first 4,096) | dense (12-bit) | 2,733 | 8,199 (2.0×) | hanzi |
| `futhark.json` | 24 | Elder Futhark rune string | big-int (base-24) | 7,260 | 21,780 (5.3×) | true runes |
| `dovahzul.json` | 34 | Dovahzul dragon runes (Supplementary PUA-A) | big-int (base-34) | 6,553 | 19,659 (4.8×) | dragon runes (font required) |

## Reading the table

* **Cipher symbols** is the information cost: 256-symbol layouts map 1
  symbol per byte; `cjk-4096` uses dense 12-bit packing (3 bytes → 2
  symbols, plus a 1-symbol tail marker); `futhark` expands information
  (~1.77×) because 24 < 256.
* **Cipher bytes** is what the file weighs on disk: UTF-8 gives BMP code
  points 3 bytes and plane-1 code points (like Byzantine, U+1D000…) 4
  bytes. If storage matters, pick a BMP layout.

## Gotchas

* `braille-256`: byte `0x00` maps to U+2800, the *blank* braille cell.
  It renders like a space but is a real character — copies and
  round-trips fine, it just looks invisible in some editors.
* `byzantine-256`: neume glyphs render inconsistently across fonts
  (some are spacing-sensitive). Storage and round-trips are unaffected.
* `cjk-4096`: dense mode packs 12 payload bits per symbol (3 bytes → 2
  symbols); all 4,096 symbols participate. Byte `0x00`…`0xFF` never
  appear alone — the marker symbol for short tails is `一` (U+4E00).
* `futhark`: the base-24 big-int mode adds a 14-symbol length header per
  512-byte chunk — negligible for real messages.
* `dovahzul`: symbols are private-use code points (`U+F0000..=U+F0021`,
  the Skyrim dragon-language mapping documented in `src/dovahzul.rs`),
  so it only validates permissively — `layout validate --permissive` —
  and needs a font covering SPUA-A to *render*. Round-trips are
  font-independent; base-34 is denser than futhark's base-24.

## Make your own

```bash
$BIN from-block --block "Ogham" --count 32 --name ogham32 > layouts/ogham.json
$BIN from-symbols --symbols "ᚠᚢᚦ𓂀ᚨ" --name mixed > layouts/mixed.json
```

The files are plain JSON — edit, diff, and share them freely. Any
modification can be checked with `layout validate`.
