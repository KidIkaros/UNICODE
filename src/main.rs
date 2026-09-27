//! `layout` — command-line tool for user-defined Unicode cipher layouts.
//!
//! ```text
//! layout from-block  --block Runic --count 24 [--name my]     → layout JSON
//! layout from-symbols --symbols "ᚠᚢᚦ" [--name my]             → layout JSON
//! layout validate    layout.json                              → summary
//! layout encode      layout.json [--in data.bin] [--out text] → cipher text
//! layout decode      layout.json [--in text] [--out data.bin] → original bytes
//! layout list-blocks                                          → block table
//! layout recovery-check layout.json                           → recovery verdict
//! ```
//!
//! Requires the `json` feature (run with `cargo run --features json`).

#![cfg_attr(not(feature = "json"), allow(unused))]
#[cfg(not(feature = "json"))]
compile_error!(
    "the `layout` CLI requires the `json` feature: cargo run --features json -- layout ..."
);

use std::io::{Read, Write};
use std::process::ExitCode;

use unicode16_blocks::layout::{Layout, Symbols, MAX_LAYOUT_LEN};
use unicode16_blocks::{
    block_by_name, layout_from_json, layout_from_json_permissive, recovery_symbol_count,
    recovery_usable, write_layout_json, LayoutError, SourceDesc, MAX_PHRASE_SYMBOLS,
    SUPPORTED_ENTROPY_BYTES,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("from-block") => cmd_from_block(&args[1..]),
        Some("from-symbols") => cmd_from_symbols(&args[1..]),
        Some("validate") => {
            let (permissive, rest) = take_permissive(&args[1..]);
            cmd_validate(rest.first().map(String::as_str), permissive)
        }
        Some("encode") => {
            let (permissive, rest) = take_permissive(&args[1..]);
            cmd_encode(
                rest.first().map(String::as_str),
                rest.get(1..).unwrap_or(&[]),
                permissive,
            )
        }
        Some("decode") => {
            let (permissive, rest) = take_permissive(&args[1..]);
            cmd_decode(
                rest.first().map(String::as_str),
                rest.get(1..).unwrap_or(&[]),
                permissive,
            )
        }
        Some("list-blocks") => cmd_list_blocks(),
        Some("translit") => cmd_translit(&args[1..]),
        Some("untranslit") => cmd_untranslit(&args[1..]),
        Some("recovery-check") => {
            let (permissive, rest) = take_permissive(&args[1..]);
            cmd_recovery_check(rest.first().map(String::as_str), permissive)
        }
        Some("--help") | Some("-h") | Some("help") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown command: {other}\n");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "usage: layout <command> [args]

commands:
  from-block   --block <name> --count <n> [--name <label>] [--permissive]
               Create a layout from the first n code points of a block.
  from-symbols --symbols <string> [--name <label>] [--permissive]
               Create a layout from pasted symbols (each char = one slot).
  validate     <layout.json> [--permissive]
               Check a layout file and print a summary.
  encode       <layout.json> [--in <file>] [--out <file>] [--permissive]
               Encode bytes (file or stdin) into cipher symbols.
  decode       <layout.json> [--in <file>] [--out <file>] [--permissive]
               Decode cipher symbols back into bytes.
  list-blocks  Print every Unicode 16.0 block with its range and size.
  translit     <latin text>
               Transliterate Latin text into Dovahzul dragon runes
               (Supplementary PUA-A, U+F0000..; see the dovahzul module).
  untranslit   <runes>
               Transliterate Dovahzul dragon runes back into Latin text.
  recovery-check <layout.json> [--permissive]
               Check a layout against the recovery-phrase policy and report
               the phrase length it produces at each supported entropy width.

--permissive switches validation from Layout::new to
Layout::new_permissive, accepting private-use symbols (e.g. the
dovahzul dragon-rune layout), noncharacters, and variation selectors.

The layout JSON produced by from-block/from-symbols is the shareable
format; feed it back to validate/encode/decode."
    );
}

// -- flag helpers -------------------------------------------------------------

fn flag_value<'a>(args: &'a [String], name: &str) -> Result<Option<&'a str>, String> {
    match args.iter().position(|a| a == name) {
        None => Ok(None),
        Some(i) if i + 1 < args.len() => Ok(Some(args[i + 1].as_str())),
        Some(_) => Err(format!("missing value for {name}")),
    }
}

fn require_flag<'a>(args: &'a [String], name: &str) -> Result<&'a str, String> {
    flag_value(args, name)?.ok_or_else(|| format!("missing required flag {name}"))
}

/// Boolean switch: `--permissive` anywhere in the argument list.
/// Returns the flag's presence and the list with it stripped, so the
/// commands below keep their positional/flag layout.
fn take_permissive(args: &[String]) -> (bool, Vec<String>) {
    let on = args.iter().any(|a| a == "--permissive");
    (
        on,
        args.iter()
            .filter(|a| a.as_str() != "--permissive")
            .cloned()
            .collect(),
    )
}

// -- symbol sources ------------------------------------------------------------

fn cmd_from_block(args: &[String]) -> ExitCode {
    run_cmd(|| {
        let (permissive, args) = take_permissive(args);
        let block_name = require_flag(&args, "--block")?;
        let count: u32 = require_flag(&args, "--count")?
            .parse()
            .map_err(|_| "--count must be a number".to_string())?;
        let name = flag_value(&args, "--name")?.unwrap_or("layout");

        // Validate by building before emitting anything.
        let id = block_by_name(block_name)
            .ok_or_else(|| format!("unknown block {block_name:?} (see `layout list-blocks`)"))?;
        let symbols = Symbols::Block { block: id, count };
        let _layout = if permissive {
            Layout::new_permissive(symbols)
        } else {
            Layout::new(symbols)
        }
        .map_err(err)?;

        print_json(
            &SourceDesc::Block {
                block: block_name,
                count,
            },
            name,
        )
    })
}

fn cmd_from_symbols(args: &[String]) -> ExitCode {
    run_cmd(|| {
        let (permissive, args) = take_permissive(args);
        let symbols = require_flag(&args, "--symbols")?;
        let name = flag_value(&args, "--name")?.unwrap_or("layout");
        let owned = Symbols::String(symbols.to_owned());
        let _layout = if permissive {
            Layout::new_permissive(owned)
        } else {
            Layout::new(owned)
        }
        .map_err(err)?;
        print_json(&SourceDesc::Chars(symbols), name)
    })
}

fn print_json(source: &SourceDesc<'_>, name: &str) -> Result<(), String> {
    let mut buf = vec![0u8; MAX_LAYOUT_LEN * 4 + 256];
    let n = write_layout_json(name, *source, &mut buf).map_err(err)?;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&buf[..n]).map_err(io_err)?;
    stdout.write_all(b"\n").map_err(io_err)?;
    Ok(())
}

// -- layout loading ------------------------------------------------------------

fn load_layout(path: &str, permissive: bool) -> Result<Layout, String> {
    let json = std::fs::read_to_string(path).map_err(io_err)?;
    if permissive {
        layout_from_json_permissive(&json)
    } else {
        layout_from_json(&json)
    }
    .map_err(err)
}

fn layout_summary(l: &Layout) -> String {
    let mode = if l.supports_direct_byte_mapping() {
        "direct (1 byte → 1 symbol)".to_string()
    } else if let Some(bits) = l.bits_per_symbol() {
        if l.symbol_count() > 256 {
            format!("dense ({bits} bits per symbol)")
        } else {
            format!("bit-packed ({bits} bits per symbol)")
        }
    } else {
        format!("big-int (base-{} chunks)", l.symbol_count())
    };
    let sample: String = l.iter().take(12).filter_map(char::from_u32).collect();
    format!(
        "{} symbols, {} mode\nfirst symbols: {sample}{}",
        l.symbol_count(),
        mode,
        if l.symbol_count() > 12 { "…" } else { "" }
    )
}

fn cmd_validate(path: Option<&str>, permissive: bool) -> ExitCode {
    run_cmd(|| {
        let path = path.ok_or("usage: layout validate <layout.json> [--permissive]")?;
        let l = load_layout(path, permissive)?;
        println!("{}: valid", path);
        println!("{}", layout_summary(&l));
        Ok(())
    })
}

// -- encode / decode -------------------------------------------------------------

fn cmd_encode(path: Option<&str>, args: &[String], permissive: bool) -> ExitCode {
    run_cmd(|| {
        let path = path
            .ok_or("usage: layout encode <layout.json> [--in file] [--out file] [--permissive]")?;
        let l = load_layout(path, permissive)?;
        let bytes = read_input(flag_value(args, "--in")?)?;

        let mut out = vec![0u32; l.max_encoded_len(bytes.len())];
        let n = l.encode(&bytes, &mut out).map_err(err)?;
        let text: String = out[..n]
            .iter()
            .map(|&cp| char::from_u32(cp).expect("layout symbols are valid scalars"))
            .collect();

        write_text(flag_value(args, "--out")?, &text)?;
        eprintln!("encoded {} bytes → {} symbols", bytes.len(), n);
        Ok(())
    })
}

fn cmd_decode(path: Option<&str>, args: &[String], permissive: bool) -> ExitCode {
    run_cmd(|| {
        let path = path
            .ok_or("usage: layout decode <layout.json> [--in file] [--out file] [--permissive]")?;
        let l = load_layout(path, permissive)?;
        let text = read_text(flag_value(args, "--in")?)?;
        let symbols: Vec<u32> = text.chars().map(|c| c as u32).collect();

        let mut out = vec![0u8; l.max_decoded_len(symbols.len())];
        let n = l.decode(&symbols, &mut out).map_err(err)?;

        write_output(flag_value(args, "--out")?, &out[..n])?;
        eprintln!("decoded {} symbols → {} bytes", symbols.len(), n);
        Ok(())
    })
}

// -- dovahzul transliteration ----------------------------------------------------

fn cmd_translit(args: &[String]) -> ExitCode {
    run_cmd(|| {
        let text = args.first().ok_or("usage: layout translit <latin text>")?;
        let runes = unicode16_blocks::dovahzul::transliterate(text).map_err(|e| e.to_string())?;
        println!("{runes}");
        Ok(())
    })
}

fn cmd_untranslit(args: &[String]) -> ExitCode {
    run_cmd(|| {
        let runes = args.first().ok_or("usage: layout untranslit <runes>")?;
        let latin = unicode16_blocks::dovahzul::derune(runes).map_err(|e| e.to_string())?;
        println!("{latin}");
        Ok(())
    })
}

// -- recovery check ---------------------------------------------------------------

/// Report whether a layout can back a recovery phrase.
///
/// A recovery alphabet has a stricter contract than a general cipher
/// alphabet: symbols must survive transcription and interchange, so controls,
/// invisible format characters, and combining marks are rejected even though
/// they sit inside allocated blocks. The phrase length is reported per
/// supported entropy width, because an alphabet that needs more than
/// [`MAX_PHRASE_SYMBOLS`] symbols is not transcribable by hand.
/// Cap the per-symbol dump; a bad layout can reject hundreds of symbols.
const MAX_OFFENDERS_SHOWN: usize = 8;

fn cmd_recovery_check(path: Option<&str>, permissive: bool) -> ExitCode {
    run_cmd(|| {
        let path = path.ok_or("usage: layout recovery-check <layout.json> [--permissive]")?;
        let layout = load_layout(path, permissive)?;
        let symbols: Vec<u32> = layout.iter().collect();
        println!("{path}: {} symbols", symbols.len());

        let offenders: Vec<(usize, u32)> = symbols
            .iter()
            .enumerate()
            .filter(|(_, &cp)| !unicode16_blocks::recovery_safe(cp))
            .map(|(index, &cp)| (index, cp))
            .collect();
        if offenders.is_empty() {
            println!("recovery policy: every symbol passes");
        } else {
            println!("recovery policy: {} symbol(s) rejected", offenders.len());
            for (index, cp) in offenders.iter().take(MAX_OFFENDERS_SHOWN) {
                println!("  position {index}: U+{cp:04X} is not recovery-safe");
            }
            if offenders.len() > MAX_OFFENDERS_SHOWN {
                println!("  ... and {} more", offenders.len() - MAX_OFFENDERS_SHOWN);
            }
        }

        for entropy_bytes in SUPPORTED_ENTROPY_BYTES {
            match recovery_symbol_count(symbols.len(), entropy_bytes) {
                Some(count) => {
                    let verdict = if count <= MAX_PHRASE_SYMBOLS {
                        "usable"
                    } else {
                        "over the limit"
                    };
                    println!(
                        "  {entropy_bytes}-byte entropy: {count} symbols ({verdict}, limit {MAX_PHRASE_SYMBOLS})"
                    );
                }
                None => continue,
            }
        }

        if offenders.is_empty() && recovery_usable(symbols.len()) {
            Ok(())
        } else {
            Err("not a usable recovery alphabet".to_string())
        }
    })
}

// -- block listing ----------------------------------------------------------------

fn cmd_list_blocks() -> ExitCode {
    for b in unicode16_blocks::BLOCKS.iter() {
        println!(
            "U+{:04X}..U+{:04X}  {:>6}  {}",
            b.first,
            b.last,
            b.size_chars(),
            b.name
        );
    }
    ExitCode::SUCCESS
}

// -- plumbing ----------------------------------------------------------------------

fn read_input(path: Option<&str>) -> Result<Vec<u8>, String> {
    match path {
        Some(p) => std::fs::read(p).map_err(io_err),
        None => {
            let mut buf = Vec::new();
            std::io::stdin()
                .lock()
                .read_to_end(&mut buf)
                .map_err(io_err)?;
            Ok(buf)
        }
    }
}

fn read_text(path: Option<&str>) -> Result<String, String> {
    match path {
        Some(p) => std::fs::read_to_string(p).map_err(io_err),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .lock()
                .read_to_string(&mut buf)
                .map_err(io_err)?;
            Ok(buf)
        }
    }
}

fn write_text(path: Option<&str>, text: &str) -> Result<(), String> {
    match path {
        Some(p) => std::fs::write(p, text).map_err(io_err),
        None => std::io::stdout()
            .lock()
            .write_all(text.as_bytes())
            .map_err(io_err),
    }
}

fn write_output(path: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    match path {
        Some(p) => std::fs::write(p, bytes).map_err(io_err),
        None => std::io::stdout().lock().write_all(bytes).map_err(io_err),
    }
}

fn io_err(e: std::io::Error) -> String {
    format!("io error: {e}")
}

fn err(e: LayoutError) -> String {
    format!("layout error: {e}")
}

fn run_cmd(f: impl FnOnce() -> Result<(), String>) -> ExitCode {
    match f() {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}
