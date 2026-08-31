//! Renders one 5-second WAV per instrument patch: a single held note, so each
//! patch in the library can be auditioned in isolation.
//!
//! Nothing is stored as `.bin` — each patch is built, wrapped in a minimal
//! song, encoded/decoded in memory and rendered straight to WAV.
//!
//! Usage:
//!   cargo run --release --example instrument_sounds -- [--out-dir <dir>]
//!
//! Writes `<out-dir>/<engine>-<instrument>[-<variation>].wav` (default
//! out-dir is the repo `export/instruments`).

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{DeviceId, MidiEvent, Receive, Song, Track, decode, encode};
use starter::instruments::{falcon, slaughter};
use starter::render;

const SAMPLE_RATE: u32 = 44100;
const NOTE_SECS: f64 = 5.0;

type Patch = fn() -> (DeviceId, Vec<u8>);

struct Entry {
    name: String,
    patch: Patch,
    note: u8,
}

fn push(cat: &mut Vec<Entry>, engine: &str, inst: &str, note: u8, patch: Patch) {
    cat.push(Entry {
        name: format!("{inst}-{engine}"),
        patch,
        note,
    });
}

fn push_vars(cat: &mut Vec<Entry>, engine: &str, inst: &str, note: u8, vars: &[&str], ps: &[Patch]) {
    for (v, p) in vars.iter().zip(ps) {
        cat.push(Entry {
            name: format!("{inst}-{v}-{engine}"),
            patch: *p,
            note,
        });
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let mut out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("export")
        .join("instruments");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out-dir" => {
                out_dir = Path::new(
                    &args.next().expect("--out-dir needs a path"),
                ).to_path_buf();
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    let mut cat: Vec<Entry> = Vec::new();

    // Core falcon kit.
    push(&mut cat, "falcon", "bass", 48, falcon::bass::bass);
    push(&mut cat, "falcon", "kick", 36, falcon::drums::kick);
    push(&mut cat, "falcon", "snare", 38, falcon::drums::snare);
    push(&mut cat, "falcon", "closed-hat", 42, falcon::drums::closed_hat);
    push(&mut cat, "falcon", "open-hat", 46, falcon::drums::open_hat);
    push(&mut cat, "falcon", "crash", 49, falcon::drums::crash);
    push(&mut cat, "falcon", "shaker", 54, falcon::drums::shaker);
    push(&mut cat, "falcon", "flute", 60, falcon::flute::flute);
    push(&mut cat, "falcon", "piano", 60, falcon::piano::piano);
    push(&mut cat, "falcon", "tenor-sax", 57, falcon::saxophone::tenor_sax);
    push(&mut cat, "falcon", "alto-sax", 60, falcon::saxophone::alto_sax);

    // Core slaughter kit.
    push(&mut cat, "slaughter", "bass", 48, slaughter::bass::bass);
    push(&mut cat, "slaughter", "kick", 36, slaughter::drums::kick);
    push(&mut cat, "slaughter", "snare", 38, slaughter::drums::snare);
    push(&mut cat, "slaughter", "closed-hat", 42, slaughter::drums::closed_hat);
    push(&mut cat, "slaughter", "open-hat", 46, slaughter::drums::open_hat);
    push(&mut cat, "slaughter", "crash", 49, slaughter::drums::crash);
    push(&mut cat, "slaughter", "shaker", 54, slaughter::drums::shaker);
    push(&mut cat, "slaughter", "piano", 60, slaughter::piano::piano);
    push(&mut cat, "slaughter", "tenor-sax", 57, slaughter::saxophone::tenor_sax);
    push(&mut cat, "slaughter", "alto-sax", 60, slaughter::saxophone::alto_sax);

    // 20 variations per family (10 falcon + 10 slaughter).
    let flute_vars: [&str; 3] = ["1","2","3"];
    let piano_vars: [&str; 10] = ["1","2","3","4","5","6","7","8","9","10"];
    let sax_vars: [&str; 10] = ["1","2","3","4","5","6","7","8","9","10"];
    let clarinet_vars: [&str; 4] = ["dark","vibrato","legato","ballad"];
    let falcon_clarinets = [
        falcon::clarinet::clarinet_dark, falcon::clarinet::clarinet_vibrato,
        falcon::clarinet::clarinet_legato, falcon::clarinet::clarinet_ballad,
    ];
    let slaughter_clarinet_vars: [&str; 1] = ["legato"];
    let slaughter_clarinets: [Patch; 1] = [
        slaughter::clarinet::clarinet_legato,
    ];
    let falcon_flutes = [
        falcon::flute::flute_v1, falcon::flute::flute_v2, falcon::flute::flute_v3,
    ];
    let falcon_pianos = [
        falcon::piano::piano_v1, falcon::piano::piano_v2, falcon::piano::piano_v3,
        falcon::piano::piano_v4, falcon::piano::piano_v5, falcon::piano::piano_v6,
        falcon::piano::piano_v7, falcon::piano::piano_v8, falcon::piano::piano_v9,
        falcon::piano::piano_v10,
    ];
    let falcon_saxes = [
        falcon::saxophone::sax_v1, falcon::saxophone::sax_v2, falcon::saxophone::sax_v3,
        falcon::saxophone::sax_v4, falcon::saxophone::sax_v5, falcon::saxophone::sax_v6,
        falcon::saxophone::sax_v7, falcon::saxophone::sax_v8, falcon::saxophone::sax_v9,
        falcon::saxophone::sax_v10,
    ];
    let slaughter_pianos = [
        slaughter::piano::piano_v1, slaughter::piano::piano_v2, slaughter::piano::piano_v3,
        slaughter::piano::piano_v4, slaughter::piano::piano_v5, slaughter::piano::piano_v6,
        slaughter::piano::piano_v7, slaughter::piano::piano_v8, slaughter::piano::piano_v9,
        slaughter::piano::piano_v10,
    ];
    let slaughter_saxes = [
        slaughter::saxophone::sax_v1, slaughter::saxophone::sax_v2, slaughter::saxophone::sax_v3,
        slaughter::saxophone::sax_v4, slaughter::saxophone::sax_v5, slaughter::saxophone::sax_v6,
        slaughter::saxophone::sax_v7, slaughter::saxophone::sax_v8, slaughter::saxophone::sax_v9,
        slaughter::saxophone::sax_v10,
    ];

    push_vars(&mut cat, "falcon", "flute", 60, &flute_vars, &falcon_flutes);
    push_vars(&mut cat, "falcon", "piano", 60, &piano_vars, &falcon_pianos);
    push_vars(&mut cat, "falcon", "sax", 60, &sax_vars, &falcon_saxes);
    push_vars(&mut cat, "falcon", "clarinet", 60, &clarinet_vars, &falcon_clarinets);
    push_vars(&mut cat, "slaughter", "clarinet", 60, &slaughter_clarinet_vars, &slaughter_clarinets);
    push_vars(&mut cat, "slaughter", "piano", 60, &piano_vars, &slaughter_pianos);
    push_vars(&mut cat, "slaughter", "sax", 60, &sax_vars, &slaughter_saxes);

    println!("rendering {} patches to {}", cat.len(), out_dir.display());

    for (i, e) in cat.iter().enumerate() {
        let (device, chunk) = (e.patch)();

        // One track holding the device + a single held note, plus a master
        // that receives it (same routing the other examples use).
        let mut song = Song::new(120, SAMPLE_RATE as i32);
        let mut track = Track::new(1.0);
        track.devices.push((device, chunk));
        track.events.push(MidiEvent::on(0, e.note, 100));
        track.events.push(MidiEvent::off(
            (NOTE_SECS - 0.3) as i64 * SAMPLE_RATE as i64,
            e.note,
        ));
        let mut master = Track::new(1.0);
        master.receives.push(Receive::new(0, 0, 1.0));
        song.tracks = vec![track, master];
        song.length = NOTE_SECS;

        let parsed = decode(&encode(&song)).expect("internal error: re-parsing the generated song");
        let mut buf = render::render_stereo(&parsed);
        render::normalize_stereo(&mut buf);

        let base = out_dir.join(&e.name).to_string_lossy().into_owned();
        render::write_stereo_wav_at(&base, &buf, SAMPLE_RATE);

        println!("[{:>2}/{}] {}", i + 1, cat.len(), e.name);
    }

    println!("done: {} files in {}", cat.len(), out_dir.display());
}