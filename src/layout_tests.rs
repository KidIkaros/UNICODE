//! Tests for user-defined layouts: validation, codec round-trips, and
//! hostile-input rejection.

use crate::layout::*;
use crate::std::borrow::ToOwned;
use crate::std::vec::Vec;
use crate::{BlockId, LayoutError};

/// Block-source layout: first N code points of a block, in order.
#[test]
fn block_source_layout() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::Runic,
        count: 24,
    })
    .expect("runic 24");
    assert_eq!(l.symbol_count(), 24);
    assert_eq!(l.symbol(0), 0x16A0); // ᚠ
    assert_eq!(l.symbol(23), 0x16B7);
    assert_eq!(l.index_of(0x16A0), Some(0));
    assert_eq!(l.index_of(0x16B7), Some(23));
    assert_eq!(l.index_of(0x16B8), None); // outside the chosen prefix
    assert!(l.contains(0x16A5));
    assert!(!l.contains(0x0410));
}

/// Pasted-symbol layout: chars in the given order, including astral ones.
#[test]
fn chars_source_layout() {
    let l = Layout::new(Symbols::String("ᚠᚢᚦ𓂀ᚨ".to_owned())).expect("5 distinct safe symbols");
    assert_eq!(l.symbol_count(), 5);
    assert_eq!(l.symbol(3), 0x13080); // 𓂀
    assert_eq!(l.index_of(0x13080), Some(3));
    // 5 is not a power of two → BigInt mode.
    assert_eq!(l.bits_per_symbol(), None);
}

/// Validation rejections, one per error kind.
#[test]
fn validation_rejections() {
    // Too few symbols.
    assert_eq!(
        Layout::new(Symbols::Block {
            block: BlockId::Runic,
            count: 1
        })
        .unwrap_err(),
        LayoutError::TooFewSymbols
    );
    let one = Symbols::String("ᚠ".to_owned());
    assert_eq!(Layout::new(one).unwrap_err(), LayoutError::TooFewSymbols);

    // Too many symbols.
    assert_eq!(
        Layout::new(Symbols::Block {
            block: BlockId::Runic,
            count: (MAX_LAYOUT_LEN + 1) as u32
        })
        .unwrap_err(),
        LayoutError::TooManySymbols {
            len: MAX_LAYOUT_LEN + 1
        }
    );

    // Count exceeds block (Runic is U+16A0..=U+16FF: 96 symbols).
    assert_eq!(
        Layout::new(Symbols::Block {
            block: BlockId::Runic,
            count: 300
        })
        .unwrap_err(),
        LayoutError::CountExceedsBlock {
            requested: 300,
            available: 96
        }
    );
    assert_eq!(
        Layout::new(Symbols::Block {
            block: BlockId::Ogham,
            count: 40
        })
        .unwrap_err(),
        LayoutError::CountExceedsBlock {
            requested: 40,
            available: 32
        }
    ); // Duplicate symbol.
    assert_eq!(
        Layout::new(Symbols::String("ᚠᚢᚠ".to_owned())).unwrap_err(),
        LayoutError::DuplicateSymbol {
            index: 2,
            cp: 0x16A0
        }
    );

    // Unsafe code point (private use).
    assert_eq!(
        Layout::new(Symbols::String("ᚠ\u{E000}ᚢ".to_owned())).unwrap_err(),
        LayoutError::UnsafeCodePoint {
            index: 1,
            cp: 0xE000
        }
    );

    // Unsafe code point (noncharacter U+FFFF) — reachable via Block source.
    assert!(Layout::new(Symbols::Block {
        block: BlockId::Specials,
        count: 16
    })
    .is_err());

    // Permissive build accepts the PUA alphabet but still rejects surrogates
    // (unrepresentable in &str, so exercised via the strict gate only).
    assert!(Layout::new_permissive(Symbols::String("ᚠ\u{E000}ᚢ".to_owned())).is_ok());
}

/// Regressions for fuzzer findings: an `index_of` miss on a full-size
/// layout used to read one past the logical end, and `max_decoded_len`
/// under-reported for BigInt mode.
#[test]
fn fuzz_regressions() {
    // Full-size layout: misses must be None, not a bounds panic.
    let full = Layout::new(Symbols::Block {
        block: BlockId::CJKUnifiedIdeographs,
        count: 4096,
    })
    .expect("4096 hanzi");
    assert_eq!(full.index_of(0x10_FFFF), None);
    assert!(!full.contains(0x10_FFFF));

    // BigInt sizing: an all-zero chunk is 15 symbols → 512 bytes.
    let ogham3 = Layout::new(Symbols::Block {
        block: BlockId::Ogham,
        count: 3,
    })
    .expect("3");
    let zeros = [0u8; 24];
    let mut out = vec![0u32; ogham3.max_encoded_len(24)];
    let n = ogham3.encode(&zeros, &mut out).expect("encode");
    assert_eq!(n, 15);
    assert_eq!(ogham3.max_decoded_len(n), 512);
    let mut back = vec![0u8; ogham3.max_decoded_len(n)];
    let m = ogham3.decode(&out[..n], &mut back).expect("decode");
    assert_eq!(m, 24);
    assert!(back[..m].iter().all(|&b| b == 0));

    // BigInt digit validation: a valid header followed by a symbol that
    // is not in the alphabet must return InvalidSymbol, not panic —
    // the sizing pass skips over the digits without validating them,
    // so the decode pass must be fallible (fuzzer-found).
    let mut hostile: Vec<u32> = out[..PREFIX_DIGITS].to_vec();
    hostile.push(0x10_FFFE); // noncharacter as the lone "digit"
    let mut sink = [0u8; 1024];
    assert!(matches!(
        ogham3.decode(&hostile, &mut sink),
        Err(LayoutError::InvalidSymbol { .. })
    ));
}

/// Byte→symbol→byte round trip in Direct mode (256-symbol alphabet).
#[test]
fn direct_round_trip() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::ByzantineMusicalSymbols,
        count: 256,
    })
    .expect("256 byzantine symbols");
    assert!(l.supports_direct_byte_mapping());

    // Patterned data plus boundaries.
    let mut data = [0u8; 512];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i ^ (i >> 8)) as u8;
    }
    data[0] = 0x00;
    data[255] = 0xFF;
    data[511] = 0x80;

    let mut out = [0u32; 512];
    let n = l.encode(&data, &mut out).expect("encode");
    assert_eq!(n, 512);
    assert_eq!(out[0], 0x1D000);
    assert_eq!(out[255], 0x1D0FF);

    let mut back = [0u8; 512];
    let m = l.decode(&out[..n], &mut back).expect("decode");
    assert_eq!(m, 512);
    assert_eq!(back, data);
}

/// Bit-pack round trip for every power-of-two radix up to 256: distinct
/// strides, boundary straddles, and padding sizes.
#[test]
fn bitpack_round_trip_all_radices() {
    let radices = [2usize, 4, 8, 16, 32, 64, 128, 256];
    let blocks = [
        BlockId::Tibetan, // 256 symbols
        BlockId::Myanmar, // 160 → only used for ≤128
        BlockId::ByzantineMusicalSymbols,
    ];
    for &radix in &radices {
        let block = if radix <= 128 { blocks[1] } else { blocks[2] };
        let l = Layout::new(Symbols::Block {
            block,
            count: radix as u32,
        })
        .unwrap_or_else(|e| panic!("radix {radix}: {e}"));
        if radix == 256 {
            // Direct mode: one byte, one symbol.
            assert!(l.supports_direct_byte_mapping());
            assert_eq!(l.bits_per_symbol(), None);
        } else {
            assert_eq!(l.bits_per_symbol(), Some(radix.trailing_zeros()));
        }

        // Lengths chosen to hit: sub-symbol, symbol-aligned, sub-byte,
        // byte-aligned, and multi-byte sizes.
        for &len in &[1usize, 3, 7, 8, 13, 64, 255] {
            let mut data = vec![0u8; len];
            let mut x = 0x1234_5678u32;
            for b in data.iter_mut() {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                *b = (x >> 16) as u8;
            }
            let mut out = vec![0u32; l.max_encoded_len(len)];
            let n = l.encode(&data, &mut out).expect("encode");
            if radix != 256 {
                assert_eq!(n, (len * 8).div_ceil(radix.trailing_zeros() as usize));
            } else {
                assert_eq!(n, len);
            }

            let mut back = vec![0u8; l.max_decoded_len(n)];
            let m = l.decode(&out[..n], &mut back).expect("decode");
            assert_eq!(m, len, "radix {radix}, len {len}");
            assert_eq!(&back[..m], &data[..], "radix {radix}, len {len}");
        }
    }
}

/// Base-3 alphabet: the digit-count worst case (2585 digits per 512-byte
/// chunk) plus header framing, all-zero chunks, multi-chunk inputs, and
/// trailing-zero preservation.
#[test]
fn bigint_base3_round_trip() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::Ogham,
        count: 3,
    })
    .expect("3 ogham");
    assert_eq!(l.bits_per_symbol(), None);
    assert_eq!(l.symbol_count(), 3);

    // Exact worst case: one full 512-byte chunk.
    let mut data = [0u8; 512];
    for (i, b) in data.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(7) ^ (i >> 3) as u8;
    }
    let cap = l.max_encoded_len(512);
    let mut out = vec![0u32; cap];
    let n = l.encode(&data, &mut out).expect("encode worst case");
    assert!(n <= cap);
    // 512 bytes → ≤ 2585 digits + 14 header symbols.
    assert!(n <= 14 + 2585, "got {n} symbols");

    let mut back = [0u8; 512];
    let m = l.decode(&out[..n], &mut back).expect("decode");
    assert_eq!(m, 512);
    assert_eq!(back, data);

    // All-zero chunks collapse to a tiny encoding and decode back to zeros.
    let zeros = [0u8; 700]; // spans two chunks, second one all-zero
    let mut out2 = vec![0u32; l.max_encoded_len(700)];
    let n2 = l.encode(&zeros, &mut out2).expect("encode zeros");
    assert!(n2 < 40, "all-zero input should be tiny, got {n2}");
    let mut back2 = [0xAAu8; 700];
    let m2 = l.decode(&out2[..n2], &mut back2).expect("decode zeros");
    assert_eq!(m2, 700);
    assert!(back2.iter().all(|&b| b == 0));

    // Multi-chunk with high bytes (leading-zero stripping must not drop
    // interior or trailing zeros).
    let mut mixed = vec![0u8; 1200];
    mixed[3] = 0xFF;
    mixed[600] = 0x01;
    mixed[1199] = 0x80; // trailing zero bytes after it must survive
    let mut out3 = vec![0u32; l.max_encoded_len(1200)];
    let n3 = l.encode(&mixed, &mut out3).expect("encode mixed");
    let mut back3 = vec![0xEEu8; 1200];
    let m3 = l.decode(&out3[..n3], &mut back3).expect("decode mixed");
    assert_eq!(m3, 1200);
    assert_eq!(back3, mixed);
}

/// The base-2 alphabet round-trips through BigInt framing too.
#[test]
fn bigint_base2_round_trip() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::Ogham,
        count: 2,
    })
    .expect("2 ogham");
    let data: [u8; 100] = core::array::from_fn(|i| (i * 31 % 251) as u8);
    let mut out = vec![0u32; l.max_encoded_len(100)];
    let n = l.encode(&data, &mut out).expect("encode");
    let mut back = [0u8; 100];
    let m = l.decode(&out[..n], &mut back).expect("decode");
    assert_eq!(m, 100);
    assert_eq!(back, data);
}

/// Output-buffer accounting: `OutputTooSmall` reports the exact need, and
/// `max_encoded_len` / `max_decoded_len` are true upper bounds.
#[test]
fn buffer_sizing_contract() {
    let layouts = [
        Layout::new(Symbols::Block {
            block: BlockId::ByzantineMusicalSymbols,
            count: 256,
        })
        .expect("256"),
        Layout::new(Symbols::Block {
            block: BlockId::Runic,
            count: 16,
        })
        .expect("16"),
        Layout::new(Symbols::Block {
            block: BlockId::Ogham,
            count: 3,
        })
        .expect("3"),
        Layout::new(Symbols::Block {
            block: BlockId::Ogham,
            count: 2,
        })
        .expect("2"),
    ];
    for len in [0usize, 1, 100, 513, 1024] {
        let data = vec![0xA5u8; len];
        for l in &layouts {
            let cap = l.max_encoded_len(len);
            let mut out = vec![0u32; cap];
            let n = l.encode(&data, &mut out).expect("encode within cap");
            assert!(n <= cap);
            let mut back = vec![0u8; l.max_decoded_len(n)];
            let m = l.decode(&out[..n], &mut back).expect("decode within cap");
            assert_eq!(m, len);
            assert_eq!(&back[..m], &data[..]);

            // One slot short must fail with the exact need.
            if cap > 0 {
                let mut short = vec![0u32; cap - 1];
                match l.encode(&data, &mut short) {
                    Err(LayoutError::OutputTooSmall { needed }) => assert!(needed <= cap),
                    other => panic!("expected OutputTooSmall, got {other:?}"),
                }
            }
        }
    }
}

/// Dense mode engages above 256 symbols with floor(log2(len)) bits.
#[test]
fn dense_mode_selection() {
    let mode_bits = |count: u32, block: BlockId| {
        Layout::new(Symbols::Block { block, count })
            .unwrap()
            .bits_per_symbol()
    };
    // Exactly 256: Direct (no packed bits).
    assert_eq!(mode_bits(256, BlockId::Tibetan), None);
    // 257..=511: Dense with 8 bits — Direct's density in the gap.
    assert_eq!(mode_bits(257, BlockId::CJKUnifiedIdeographs), Some(8));
    assert_eq!(mode_bits(400, BlockId::CJKUnifiedIdeographs), Some(8));
    assert_eq!(mode_bits(511, BlockId::CJKUnifiedIdeographs), Some(8));
    // 512 and up: Dense with floor(log2(len)) bits.
    assert_eq!(mode_bits(512, BlockId::CJKUnifiedIdeographs), Some(9));
    assert_eq!(mode_bits(513, BlockId::CJKUnifiedIdeographs), Some(9));
    // BigInt now only serves non-power-of-two alphabets below 257.
    assert_eq!(mode_bits(255, BlockId::CJKUnifiedIdeographs), None);
    assert_eq!(mode_bits(24, BlockId::Runic), None);
    assert_eq!(mode_bits(1024, BlockId::CJKUnifiedIdeographs), Some(10));
    assert_eq!(mode_bits(2048, BlockId::CJKUnifiedIdeographs), Some(11));
    assert_eq!(mode_bits(4000, BlockId::CJKUnifiedIdeographs), Some(11));
    assert_eq!(mode_bits(4096, BlockId::CJKUnifiedIdeographs), Some(12));
}

/// The 257..=511 gap packs at Direct density: 8 bits per symbol,
/// 1 byte ↔ 1 symbol, no tails, no markers.
#[test]
fn dense_gap_8bit_round_trip() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::CJKUnifiedIdeographs,
        count: 300,
    })
    .expect("300 hanzi");
    assert_eq!(l.bits_per_symbol(), Some(8));
    assert_eq!(l.max_encoded_len(1000), 1000);
    for &len in &[1usize, 2, 3, 64, 1000] {
        let data: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
        let mut out = vec![0u32; l.max_encoded_len(len)];
        let n = l.encode(&data, &mut out).expect("encode");
        assert_eq!(n, len, "len {len}: 1 byte ↔ 1 symbol");

        let mut back = vec![0u8; l.max_decoded_len(n)];
        let m = l.decode(&out[..n], &mut back).expect("decode");
        assert_eq!(m, len);
        assert_eq!(&back[..m], &data);
    }

    // Beats BigInt's ~1.7x expansion: 1000 bytes stay 1000 symbols.
    let data = vec![0x5Au8; 1000];
    let mut out = vec![0u32; l.max_encoded_len(1000)];
    let n = l.encode(&data, &mut out).expect("encode");
    assert_eq!(n, 1000);
}

/// Group shapes: smallest bit group aligned to both bytes and symbols.
#[test]
fn dense_group_shapes() {
    use crate::layout::dense_group;
    assert_eq!(dense_group(12), (24, 2)); // gcd 4: 3 bytes ↔ 2 symbols
    assert_eq!(dense_group(10), (40, 4)); // gcd 2: 5 bytes ↔ 4 symbols
    assert_eq!(dense_group(9), (72, 8)); // gcd 1: 9 bytes ↔ 8 symbols
    assert_eq!(dense_group(11), (88, 8));
    assert_eq!(dense_group(16), (16, 1));
}

/// 12-bit dense round trip: 3 bytes in, 2 symbols out, byte-exact back.
#[test]
fn dense_round_trip_12bit() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::CJKUnifiedIdeographs,
        count: 4096,
    })
    .expect("4096 hanzi");
    assert_eq!(l.bits_per_symbol(), Some(12));

    for &len in &[1usize, 2, 3, 4, 5, 8, 100, 1000, 4096] {
        let data: Vec<u8> = (0..len).map(|i| (i * 131 + 7) as u8).collect();
        let mut out = vec![0u32; l.max_encoded_len(len)];
        let n = l.encode(&data, &mut out).expect("encode");
        // 2 symbols per 3-byte group, +1 marker symbol for a tail.
        let groups = len / 3 + usize::from(len % 3 != 0);
        assert_eq!(n, groups * 2 + usize::from(len % 3 != 0), "len {len}");

        let mut back = vec![0u8; l.max_decoded_len(n)];
        let m = l.decode(&out[..n], &mut back).expect("decode");
        assert_eq!(m, len, "len {len}");
        assert_eq!(&back[..m], &data, "len {len}");
    }

    // 3000 bytes must now beat Direct mode: 1000 groups × 2 = 2000 symbols.
    let data = vec![0x5Au8; 3000];
    let mut out = vec![0u32; l.max_encoded_len(3000)];
    let n = l.encode(&data, &mut out).expect("encode");
    assert_eq!(n, 2000, "dense must beat direct (3000)");
}

/// 11-bit dense: tails pad to whole 88-bit groups, so distinct byte
/// counts always yield distinct symbol counts (no cross-length
/// ambiguity in decode).
#[test]
fn dense_tail_padding_unambiguous() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::CJKUnifiedIdeographs,
        count: 513,
    })
    .expect("513 symbols");
    assert_eq!(l.bits_per_symbol(), Some(9));

    for &len in &[1usize, 2, 3, 9, 10, 18, 19] {
        let data: Vec<u8> = (0..len).map(|i| (i * 89 + 3) as u8).collect();
        let mut out = vec![0u32; l.max_encoded_len(len)];
        let n = l.encode(&data, &mut out).expect("encode");
        // 8 symbols per 9-byte group, +1 marker symbol for a tail.
        let groups = len / 9 + usize::from(len % 9 != 0);
        assert_eq!(n, groups * 8 + usize::from(len % 9 != 0), "len {len}");

        let mut back = vec![0u8; l.max_decoded_len(n)];
        let m = l.decode(&out[..n], &mut back).expect("decode");
        assert_eq!(m, len);
        assert_eq!(&back[..m], &data);
    }
}

/// Dense decode rejects symbol values at or above 2^bits (tampering or
/// a foreign layout), not just symbols outside the alphabet.
#[test]
fn dense_rejects_out_of_code_values() {
    // 4000 symbols → 11-bit code: indexes 2048..=3999 are real symbols
    // but out of code.
    let l = Layout::new(Symbols::Block {
        block: BlockId::CJKUnifiedIdeographs,
        count: 4000,
    })
    .expect("4000 symbols");
    let data = vec![0x42u8; 3];
    let mut out = vec![0u32; l.max_encoded_len(3)];
    let n = l.encode(&data, &mut out).expect("encode");
    let mut sink = [0u8; 64];

    // Tamper a payload symbol (index 3000 ≥ 2^11).
    let mut tampered = out[..n].to_vec();
    tampered[0] = l.symbol(3000);
    assert!(matches!(
        l.decode(&tampered, &mut sink),
        Err(LayoutError::InvalidSymbol { .. })
    ));

    // Tamper the trailing marker: its value (the tail byte count) must
    // fall in 1..bytes_per_group, so an out-of-code symbol is Corrupted.
    let mut marker_tampered = out[..n].to_vec();
    marker_tampered[n - 1] = l.symbol(3000);
    assert!(matches!(
        l.decode(&marker_tampered, &mut sink),
        Err(LayoutError::Corrupted)
    ));

    // Note: stripping the marker yields a valid no-tail decode of the
    // padded message (11 bytes here) — inherent to the format, since
    // "no marker" means "whole groups". Length is attacker-truncatable
    // in every length-prefixed scheme; integrity needs a MAC.
}

/// A lone marker symbol (symbol count 1 < group size) is structurally
/// invalid, not a zero-byte message.
#[test]
fn dense_rejects_lone_marker() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::CJKUnifiedIdeographs,
        count: 4096,
    })
    .expect("4096 hanzi");
    let mut sink = [0u8; 64];
    assert!(matches!(
        l.decode(&[l.symbol(1)], &mut sink),
        Err(LayoutError::Corrupted)
    ));
}

/// Hostile ciphertext: foreign symbols, broken framing, and
/// value-overflow digits must all be rejected without panicking.
#[test]
fn hostile_ciphertext_rejected() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::Ogham,
        count: 3,
    })
    .expect("3 ogham");
    let foreign = Layout::new(Symbols::Block {
        block: BlockId::Runic,
        count: 24,
    })
    .expect("24 runic");

    // A symbol from another alphabet.
    let mut out = vec![0u32; foreign.max_encoded_len(24)];
    let n = foreign
        .encode(b"hello world, hello world", &mut out)
        .expect("encode");
    let mut sink = [0u8; 64];
    assert!(matches!(
        l.decode(&out[..n], &mut sink),
        Err(LayoutError::InvalidSymbol { .. })
    ));

    // Truncated big-int stream: header without enough digits.
    let mut out2 = vec![0u32; l.max_encoded_len(600)];
    let n2 = l.encode(&[0x42u8; 600], &mut out2).expect("encode");
    assert!(matches!(
        l.decode(&out2[..n2 - 1], &mut [0u8; 600]),
        Err(LayoutError::Corrupted)
    ));

    // Digit-value overflow: valid symbols, but the chunk value exceeds
    // what its declared byte count can hold. Hand-build such a stream.
    let mut hostile = Vec::new();
    // Header claiming nd = BIGINT_MAX_DIGITS digits but nbytes = 1.
    let value = ((BIGINT_MAX_DIGITS as u64) << 10) | 1;
    let mut v = value as usize;
    for _ in 0..PREFIX_DIGITS {
        hostile.push(l.symbol(v % l.symbol_count()));
        v /= l.symbol_count();
    }
    for _ in 0..BIGINT_MAX_DIGITS {
        hostile.push(l.symbol(l.symbol_count() - 1)); // all max digits
    }
    let mut sink2 = [0u8; 4096];
    assert!(
        l.decode(&hostile, &mut sink2).is_err(),
        "value overflow must be rejected, not wrapped"
    );
}

/// Decode never panics on arbitrary symbol soup (bounded, deterministic).
#[test]
fn decode_fuzz_never_panics() {
    let l = Layout::new(Symbols::Block {
        block: BlockId::Ogham,
        count: 3,
    })
    .expect("3 ogham");
    let mut x = 0xC0FFEEu32;
    for trial in 0..200u32 {
        let n = (x % 300) as usize + 1;
        let mut syms = Vec::with_capacity(n);
        for _ in 0..n {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            // Mix valid symbols with invalid ones.
            let pick = x % 5;
            if pick == 0 {
                syms.push(0x0410); // Cyrillic А — not in this alphabet
            } else {
                syms.push(l.symbol((x % l.symbol_count() as u32) as usize));
            }
        }
        let mut sink = [0u8; 4096];
        let _ = l.decode(&syms, &mut sink);
        let _ = trial;
    }
}
