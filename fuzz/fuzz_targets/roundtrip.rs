//! Round-trip oracle: for arbitrary payloads and layout shapes, encode →
//! decode must reproduce the bytes exactly. Layout selection is steered
//! to cover all four modes (BitPack, Direct, Dense 8-bit gap, Dense 9–12).

#![no_main]

use libfuzzer_sys::fuzz_target;
use unicode16_blocks::layout::{Layout, Symbols};
use unicode16_blocks::BlockId;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }
    // Deterministic layout shape from the first two bytes.
    let mut count = 2 + (data[0] as usize) % 4095; // 2..=4096
    if data[1] & 1 == 0 {
        // Half the corpus snaps to power-of-two sizes to hit BitPack.
        count = count.next_power_of_two().min(4096);
    }
    let blocks = [
        BlockId::CJKUnifiedIdeographs,
        BlockId::Runic,
        BlockId::ByzantineMusicalSymbols,
        BlockId::Ogham,
    ];
    let block = blocks[(data[1] >> 6) as usize];
    let payload = &data[2..];

    let Ok(layout) = Layout::new(Symbols::Block {
        block,
        count: count as u32,
    }) else {
        return; // count exceeds the block's size; nothing to test
    };

    let mut out = vec![0u32; layout.max_encoded_len(payload.len())];
    let Ok(n) = layout.encode(payload, &mut out) else {
        return;
    };

    let mut back = vec![0u8; layout.max_decoded_len(n)];
    let m = layout
        .decode(&out[..n], &mut back)
        .expect("decode of own encode must succeed");
    assert_eq!(
        &back[..m],
        payload,
        "round-trip violated (block={block:?}, count={count}, len={})",
        payload.len()
    );
});
