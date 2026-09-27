//! Hostile decode: arbitrary and malformed symbol streams must produce
//! `Ok` or a typed error — never a panic, an out-of-bounds write, or UB.

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicode16_blocks::layout::{Layout, Symbols};
use unicode16_blocks::BlockId;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    // Rotate across every codec family: BigInt (base-3), Dense 8-bit gap,
    // Dense 12-bit, BitPack 5-bit.
    let (block, count) = match data[0] >> 6 {
        0 => (BlockId::Ogham, 3u32),
        1 => (BlockId::CJKUnifiedIdeographs, 300),
        2 => (BlockId::CJKUnifiedIdeographs, 4096),
        _ => (BlockId::Runic, 32),
    };
    let layout = Layout::new(Symbols::Block { block, count }).expect("valid shape");

    // Build a hostile symbol stream: mostly in-alphabet, with foreign
    // symbols, noncharacters, and out-of-alphabet neighbors mixed in.
    let mut symbols = Vec::with_capacity(data.len());
    for &b in data {
        match b & 7 {
            0 => symbols.push(0x0410),               // foreign (Cyrillic А)
            1 => symbols.push(0x10_FFFE),            // noncharacter
            2 => symbols.push(layout.symbol(0) + 1), // neighbor, likely out
            _ => symbols.push(layout.symbol(b as usize % layout.symbol_count())),
        }
    }
    // Truncate 0–3 symbols to attack the length prefix / marker paths.
    let chop = (data[0] & 3) as usize;
    let symbols = &symbols[..symbols.len().saturating_sub(chop)];

    // Fixed stack buffer: sized above the largest decode any of these
    // layouts can produce for libFuzzer's default input bounds; a larger
    // stream simply yields OutputTooSmall, which is a valid outcome.
    let mut sink = [0u8; 8192];
    let _ = layout.decode(symbols, &mut sink);
});
