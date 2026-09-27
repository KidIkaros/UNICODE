# Fuzzing

libFuzzer targets via [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
(nightly toolchain, ASan + debug assertions):

```bash
cd fuzz
RUSTUP_TOOLCHAIN=nightly cargo fuzz run roundtrip -- -max_total_time=60
```

## Targets

| Target | Invariant |
|---|---|
| `roundtrip` | encode → decode reproduces arbitrary payloads in every mode |
| `decode_never_panics` | hostile symbol streams yield `Ok` or typed errors, never panics/UB |
| `name_lookup` | UAX #44-equality-preserving mangling never changes lookup results |
| `json_parity` | writer output is valid JSON, parses back to the declared layout, and honors the exact-need buffer contract |
| `bigint_framing` | structured generator for BigInt framing: hostile headers (digit count × byte count ranges), digit sources (valid/maximal/foreign/noncharacter), single mutations of real encodings; asserts the `max_decoded_len` contract holds for every stream |

## Corpus

`corpus/*/regress-*` files are committed crash inputs from real findings —
they run first on every invocation, so past bugs cannot silently return.
The generated corpus (`corpus/*` without the `regress-` prefix) is
gitignored; it rebuilds in seconds.

## Findings so far

All three were found within minutes of the first sessions and are locked
by unit tests in `src/layout_tests.rs` (`fuzz_regressions`):

1. `index_of` miss read one past the logical end (`then_some` evaluates
   its argument eagerly) — silent garbage read for small layouts, panic
   on a full 4096-symbol layout.
2. `max_decoded_len` under-reported for BigInt mode: an all-zero chunk
   encodes to 15 symbols but decodes to 512 bytes.
3. `decode_bigint` asserted digits were pre-validated; the sizing pass
   actually skips them, so hostile symbols hit an `.expect` panic.

The structured `bigint_framing` target also documented itself twice
before the library paid out: `(Ogham, 2)` is a power of two and routes
to BitPack, not BigInt (caught by the generator's first session — shapes
are now asserted to select BigInt mode), and one chunk of hostile header
digits can exceed the stream cap (caught as a target-side underflow).
Both lessons are pinned in the target's comments.

`layout_from_json` also leaked the symbol string (`Box::leak`) — found
as an LSan leak, fixed by the owned `Symbols::String` variant.

## CI

`.github/workflows/fuzz.yml` runs a 5-minute smoke of every target
nightly (plus manual dispatch). Past findings re-verify daily via the
committed regression seeds.
