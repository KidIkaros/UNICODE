# unicode16-blocks

Every **Unicode 16.0** block — all **338** of them — as typed, `no_std`,
zero-dependency Rust data, designed for cipher usage.

The block table plus the `No_Block` gaps between blocks **partition the
entire Unicode code space** (`U+0000..=U+10FFFF`, 1,114,112 code points)
with no overlap and no holes.

## API

| Item | Description |
|---|---|
| `BLOCKS` | `const [Block; 338]`, ascending by code point |
| `BlockId` | one enum variant per block; `Copy + Eq + Ord + Hash` |
| `block_of(char) -> Option<&'static Block>` | O(log n) lookup; `const fn` |
| `block_of_cp(u32) -> Option<&'static Block>` | same, for raw code points; `const fn` |
| `block_by_id(BlockId) -> &'static Block` | reverse lookup |
| `block_by_name(&str) -> Option<BlockId>` | exact + UAX #44-loose name matching |
| `Block::first` / `Block::last` | inclusive `u32` bounds (`u32`, not `char`, so surrogate blocks are representable) |
| `Block::size_chars()` | code points in the block; `const fn` |
| `is_surrogate(u32)` | `U+D800..=U+DFFF` |
| `is_noncharacter(u32)` | `U+FDD0..=U+FDEF` and all `U+xFFFE`/`U+xFFFF` |
| `is_private_use(u32)` | BMP PUA + both supplementary PUA planes |
| `is_variation_selector(u32)` | `U+FE00..=U+FE0F`, `U+E0100..=U+E01EF` |
| `is_cipher_safe(u32)` | conservative gate for cipher alphabets (see below) |

All predicates are `const fn`.

## Layouts: your own cipher alphabets

> Ready-made samples live in [`layouts/`](layouts/README.md) — braille-256,
> tibetan-256, byzantine-256, cjk-4096, and a 24-rune futhark — all
> validated and round-trip-tested; grab one and go.

A [`Layout`](src/layout.rs) is an ordered set of distinct code points — a
user-defined alphabet. Build one three ways:

```rust
use unicode16_blocks::{BlockId, Layout, Symbols};

// From a block prefix:
let l = Layout::new(Symbols::Block { block: BlockId::Runic, count: 96 })?;

// From pasted symbols (each char = one slot, order preserved):
let l = Layout::new(Symbols::Chars("ᚠᚢᚦ𓂀ᚨ"))?;
```

Validation rejects duplicates, unsafe code points (surrogates,
noncharacters, private use, variation selectors, gaps), and degenerate
sizes. `Layout::new_permissive` deliberately allows private use for
closed systems.

Encoding picks the best mode automatically:

| Alphabet | Mode | Payload per symbol |
|---|---|---|
| exactly 256 | direct | 1 byte → 1 symbol |
| power of two ≤ 256 | bit-packed | log₂(n) bits |
| > 256 (257–4096) | dense | floor(log₂ n) bits — 257–511 carry 8 bits (direct density), 512+ pack wider: 4096 symbols carry 12 bits, 3 bytes ↔ 2 symbols |
| everything else | big-int chunks | base-n with length-prefixed chunks |

Codecs are allocation-free (`encode`/`decode_into`-style buffer APIs;
see `Layout::max_encoded_len` for sizing) and round-trip byte-for-byte.
Decoding is hostile-input-hard: foreign symbols, truncation, and
value-overflow digits all return typed errors, never panic.

### Layout files (JSON)

```bash
# create + share (requires the `json` feature for the CLI)
cargo run --features json -- from-block --block Runic --count 96 --name runic96 > runic.json
cargo run --features json -- from-symbols --symbols "ᚠᚢᚦ𓂀" > futhark.json
cargo run --features json -- validate runic.json
cargo run --features json -- encode runic.json --in msg.bin --out cipher.txt
cargo run --features json -- decode runic.json --in cipher.txt --out back.bin
cargo run --features json -- list-blocks
cargo run --features json -- recovery-check layouts/futhark.json
```

The file format is a single object:

```json
{"version":1,"name":"runic96","source":{"type":"block","block":"Runic","count":96}}
```

Emit it with [`write_layout_json`](src/layout_json.rs) (allocation-free,
`no_std`) or parse it with [`layout_from_json`] (feature `json`). For
layouts whose symbols sit in private use (see the Dovahzul section
below), use [`layout_from_json_permissive`] or the CLI's `--permissive`
flag — strict validation rejects private-use symbols by design.

## Using blocks as cipher alphabets

A block is a ready-made substitution alphabet. Pick a fully assigned
block and map your plaintext domain onto its code points:

* byte (0..256) → `Byzantine Musical Symbols` (`U+1D000..=U+1D0FF`) or
  `Braille Patterns` (`U+2800..=U+28FF`) — 256 symbols each
* 16-bit value → `CJK Unified Ideographs` (20,992 symbols),
  `Yi Syllables` (11,656), or `Hangul Syllables` (11,184)
* full 20-bit value → `Supplementary Private Use Area-A/B` (65,536 each;
  note `is_cipher_safe` rejects PUA — fine for a fixed internal alphabet,
  rejected only if you pipe ciphertext through interchange validators)

On decode, validate that every code point landed back in the expected
block (`block_of_cp(cp).map(|b| b.id) == Some(mine)`); anything in a
`No_Block` gap or a different block is corruption or forgery.

`is_cipher_safe(cp)` rejects surrogates (invalid scalars), noncharacters
(break interchange), private-use (render by agreement), variation
selectors (invisible, breaks signatures/hashes), out-of-range values, and
`No_Block` gaps (unassigned). Code points inside an allocated block can
still be individually unassigned — the crate intentionally does not carry
general-category data; validate against your own alphabet for that.

## Using a layout as a recovery alphabet

A recovery phrase is a seed backup, so its alphabet has a stricter contract
than a general cipher alphabet: every symbol has to survive transcription and
interchange, and the phrase has to be short enough to write down by hand.

`recovery-check` applies that contract and reports the phrase length:

```bash
cargo run --features json -- recovery-check layouts/futhark.json
```

```text
layouts/futhark.json: 24 symbols
recovery policy: every symbol passes
  16-byte entropy: 32 symbols (usable, limit 64)
  24-byte entropy: 48 symbols (usable, limit 64)
  32-byte entropy: 63 symbols (usable, limit 64)
```

[`recovery_safe`](src/recovery.rs) is the predicate behind that verdict. It
composes the checks [`is_cipher_safe`] does not make: control characters,
invisible format characters, combining marks in the ranges dedicated to them,
the variation-selector supplement, tag characters, and line/paragraph
separators are all rejected — they sit *inside* allocated blocks, so block
coverage alone accepts them. `is_cipher_safe(0x0000)` is `true`;
`recovery_safe(0x0000)` is `false`.

An alphabet needing more than [`MAX_PHRASE_SYMBOLS`] (64) symbols is reported
as over the limit rather than silently accepted: a 288-symbol phrase is not a
backup anyone can transcribe. The SDK's `RecoveryFormat` is the codec that
consumes such an alphabet.

## Case study: Dovahzul (the dragon language) in Unicode

[`dovahzul`](src/dovahzul.rs) is a worked example of giving a conlang
with no official encoding a real home in Unicode. Skyrim's dragon
language is written in 34 claw-runes — 25 single letters (the Latin
alphabet minus C; loanwords spell C as S or K) plus 9 digraph runes
(`aa ah ei ey ii ir oo ur uu`). thuum.org asked for a Unicode block in
2015; as of Unicode 16.0 none exists (nor a UCSUR slot), so the module
defines a *documented private mapping* in Supplementary Private Use
Area-A, `U+F0000..=U+F0021`, in a stable order: single letters
alphabetical, then digraphs alphabetical:

```rust
use unicode16_blocks::dovahzul::{dovahzul_layout, transliterate, derune};

// "Drem Yol Lok" (greetings) — digraphs match greedily.
let runes = transliterate("Drem Yol Lok").unwrap();
assert_eq!(derune(&runes).unwrap(), "drem yol lok");

// The 34-rune cipher layout (big-int mode, built permissively).
let layout = dovahzul_layout().unwrap();
assert_eq!(layout.symbol_count(), 34);
```

The mapping is versioned in the module and in `layouts/dovahzul.json`,
so the runes survive anywhere this crate exists — that is what makes
private use appropriate here (rendering is by agreement, and the
agreement is this module). `layout translit` / `layout untranslit`
do the Latin ↔ rune conversion from the shell; `--permissive` is
required to validate/load the layout file.

## Data source & verification

Ranges and names transcribed from the official UCD file
[Blocks-16.0.0.txt](https://www.unicode.org/Public/16.0.0/ucd/Blocks.txt)
(dated 2024-02-02, © Unicode®, Inc.,
[Unicode License](https://www.unicode.org/terms_of_use.html)).

The test suite verifies the table against a second, independent
transcription pinned in `src/tests.rs`, and checks:

* **exhaustiveness** — every one of the 1,114,112 code points resolves
  (block or gap), and block coverage (299,056) + gap coverage (815,056)
  equals the full code space exactly
* **disjointness** — no two blocks share a code point
* **official alignment** — every start ≡ 0 (mod 16), every end ≡ 15 (mod 16)
* **name/id integrity** — unique names and ids, entry order matches
  `BlockId` declaration order, every official name resolves via
  `block_by_name`
* **cipher-safety consistency** — `is_cipher_safe` agrees with the
  composed predicates across the whole code space

```text
cargo test          # 52 unit tests + 11 doc-tests (59 unit with --features json)
```

## License

Code: MIT OR Apache-2.0. Block data © Unicode®, Inc., under the
[Unicode Terms of Use](https://www.unicode.org/terms_of_use.html).
