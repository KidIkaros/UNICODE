//! Name lookup: transformations that UAX #44 loose matching must
//! tolerate (case, separators) never change the resolved block, and
//! arbitrary strings never panic.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let idx = (data[0] as usize) % unicode16_blocks::BLOCKS.len();
    let name = unicode16_blocks::BLOCKS[idx].name;

    // Mangle the official name with equality-preserving transformations.
    let mut mangled = String::new();
    for (i, c) in name.chars().enumerate() {
        match (data[1].wrapping_add(i as u8)) % 4 {
            0 => mangled.extend(c.to_uppercase()),
            2 => {
                mangled.push(c);
                mangled.push('-');
            }
            3 => {
                mangled.push('_');
                mangled.push(c);
            }
            _ => mangled.push(c),
        }
    }

    let direct = unicode16_blocks::block_by_name(name).expect("official name resolves");
    let loose = unicode16_blocks::block_by_name(&mangled)
        .unwrap_or_else(|| panic!("mangling broke the match: {mangled:?}"));
    assert_eq!(direct, loose, "mangled {mangled:?} resolved differently");

    // Arbitrary bytes as a string: lookup must not panic.
    if let Ok(arbitrary) = std::str::from_utf8(&data[2..]) {
        let _ = unicode16_blocks::block_by_name(arbitrary);
    }
});
