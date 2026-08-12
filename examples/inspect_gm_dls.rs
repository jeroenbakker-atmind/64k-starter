//! Inspects the Windows General MIDI DLS bank (`gm.dls`) and lists the wave
//! pool the way the `Adultery` device walks it (see `Adultery.cpp`), so you can
//! pick `SampleIndex` values for the piano/bass/drum voices.
//!
//! Usage:
//!   cargo run --example inspect_gm_dls [--gm-dls <path>] [--filter piano|bass|drums]
//!
//! On the target Windows machine the file lives at
//! `C:\Windows\System32\drivers\gm.dls` (falling back to
//! `...\drivers\etc\gm.dls`). A copy may also be inspected on any OS.

use std::env;
use std::fs;
use std::path::Path;

const WINDOWS_PATHS: [&str; 2] = [
    r"C:\Windows\System32\drivers\gm.dls",
    r"C:\Windows\System32\drivers\etc\gm.dls",
];

struct Wave {
    index: u32, // 1-based, i.e. the Adultery SampleIndex
    unity_note: u16,
    fine_tune: i16,
    loop_count: u32,
    loop_start: u32,
    loop_length: u32,
    samples: u32,
    list_size: u32,
}

fn find_wvpl(data: &[u8]) -> Option<&[u8]> {
    if data.len() < 12 || &data[0..4] != b"RIFF" {
        return None;
    }
    let mut pos = 12usize;
    while pos + 8 <= data.len() {
        let tag = &data[pos..pos + 4];
        let size = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().ok()?) as usize;
        let payload = pos + 8;
        if tag == b"LIST" {
            let ltype = data.get(payload..payload + 4)?;
            if ltype == b"wvpl" {
                // LIST size includes the 4-byte form type.
                if payload + 4 + size > data.len() {
                    return None;
                }
                return Some(&data[payload + 4..payload + size]);
            }
        }
        pos = payload + size;
    }
    None
}

fn parse_wsmp(payload: &[u8]) -> (u16, i16, u32, u32, u32) {
    let mut unity = 0xffffu16;
    let mut fine = 0i16;
    let mut loops = 0u32;
    let mut loop_start = 0u32;
    let mut loop_length = 0u32;
    if payload.len() >= 20 {
        unity = u16::from_le_bytes(payload[0..2].try_into().unwrap());
        fine = i16::from_le_bytes(payload[2..4].try_into().unwrap());
        loops = u32::from_le_bytes(payload[16..20].try_into().unwrap());
        if loops > 0 && payload.len() >= 36 {
            loop_start = u32::from_le_bytes(payload[28..32].try_into().unwrap());
            loop_length = u32::from_le_bytes(payload[32..36].try_into().unwrap());
        }
    }
    (unity, fine, loops, loop_start, loop_length)
}

fn walk_waves(content: &[u8]) -> Vec<Wave> {
    let mut out = Vec::new();
    let mut p = 0usize;
    let mut index = 0u32;
    while p + 8 <= content.len() {
        let tag = &content[p..p + 4];
        let size = u32::from_le_bytes(content[p + 4..p + 8].try_into().unwrap()) as usize;
        let payload = p + 8;
        if tag == b"LIST" && content.get(payload..payload + 4) == Some(&b"wave"[..]) {
            let wave = &content[payload + 4..(payload + size).min(content.len())];
            let mut unity = 0xffffu16;
            let mut fine = 0i16;
            let mut loops = 0u32;
            let mut loop_start = 0u32;
            let mut loop_length = 0u32;
            let mut samples = 0u32;
            let mut i = 0usize;
            while i + 8 <= wave.len() {
                let wtag = &wave[i..i + 4];
                let wsize = u32::from_le_bytes(wave[i + 4..i + 8].try_into().unwrap()) as usize;
                let wpayload = &wave[i + 8..(i + 8 + wsize).min(wave.len())];
                match wtag {
                    b"wsmp" => {
                        let (u, f, lc, ls, ll) = parse_wsmp(wpayload);
                        unity = u;
                        fine = f;
                        loops = lc;
                        loop_start = ls;
                        loop_length = ll;
                    }
                    b"data" => samples = (wsize / 2) as u32,
                    _ => {}
                }
                i += 8 + wsize;
            }
            index += 1;
            out.push(Wave {
                index,
                unity_note: unity,
                fine_tune: fine,
                loop_count: loops,
                loop_start,
                loop_length,
                samples,
                list_size: size as u32,
            });
        }
        p = payload + size;
    }
    out
}

fn main() {
    let mut args = env::args().skip(1).peekable();
    let mut gm_dls: Option<String> = None;
    let mut filter = "all".to_string();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--gm-dls" => gm_dls = Some(args.next().expect("--gm-dls needs a path")),
            "--filter" => filter = args.next().expect("--filter needs a value"),
            _ => {
                eprintln!("usage: cargo run --example inspect_gm_dls [--gm-dls <path>] [--filter piano|bass|drums]");
                std::process::exit(2);
            }
        }
    }

    let path = gm_dls
        .or_else(|| WINDOWS_PATHS.iter().find(|p| Path::new(p).exists()).map(|s| s.to_string()))
        .expect("gm.dls not found - pass --gm-dls <path> (or run on Windows where it lives under C:\\Windows\\System32\\drivers\\gm.dls)");

    let data = fs::read(&path).expect("failed to read gm.dls");
    println!("opened {} ({} bytes)\n", path, data.len());

    let wvpl = find_wvpl(&data).expect("could not locate the 'wvpl' wave pool");
    println!("found wave pool at LIST 'wvpl' ({} bytes)", wvpl.len());

    let waves = walk_waves(wvpl);
    println!("{} waves in pool\n", waves.len());

    if filter == "all" {
        println!(" idx  unity note | len(ms) |  loop (start/len) |  size  | guess");
    }

    let mut melodic = 0u32;
    let mut oneshot = 0u32;
    for w in &waves {
        let ms = w.samples as f64 / 44.1; // 44100 samples/sec -> ms
        let looping = w.loop_count > 0;
        let note_str = if w.unity_note < 128 {
            note_name(w.unity_note as u8)
        } else {
            "  -".to_string()
        };
        let guess = if looping && (48..=84).contains(&w.unity_note) {
            melodic += 1;
            "pitched/loop (piano range)"
        } else if !looping && ms < 3000.0 {
            oneshot += 1;
            "one-shot (percussion?)"
        } else if looping {
            melodic += 1;
            "pitched/loop"
        } else {
            "one-shot/long"
        };

        let show = if filter == "piano" && (looping && (50..=84).contains(&w.unity_note) && ms > 300.0) {
            true
        } else if filter == "bass" && (looping && (28..=55).contains(&w.unity_note)) {
            true
        } else if filter == "drums" && (!looping && ms < 3000.0) {
            true
        } else {
            filter == "all"
        };

        if show {
            if looping {
                println!(
                    " {:4}  {:>4} {:<4} | {:7.0} | start {:<6} len {:<6} | {:6} | {}",
                    w.index,
                    w.unity_note,
                    note_str,
                    ms,
                    w.loop_start,
                    w.loop_length,
                    w.list_size,
                    guess
                );
            } else {
                println!(
                    " {:4}  {:>4} {:<4} ft {} | {:7.0} |  no loop           | {:6} | {}",
                    w.index, w.unity_note, note_str, w.fine_tune, ms, w.list_size, guess
                );
            }
        }
    }

    println!(
        "\nsummary: {} melodic/looping waves, {} percussion-like one-shots",
        melodic, oneshot
    );
    println!(
        "percussion sounds are typically the NON-looping short waves near the end of the bank."
    );
    println!(
        "Update the CONFIG constants in examples/create_song.rs with chosen indices and re-run it."
    );
}

fn note_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i32 / 12) - 1;
    format!("{}{}", NAMES[(note % 12) as usize], octave)
}