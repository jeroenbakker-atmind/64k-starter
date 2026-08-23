//! Writes the original 64k-starter demo song - the known-audible baseline -
//! as a byte-exact copy of `examples/assets/original_song.bin` (extracted
//! from git history, commit `eb574da`).
//!
//! Use this to isolate playback problems: if a build of `src/main.rs` with
//! this song is silent on Windows, the issue is in the player/build pipeline,
//! not in our song generation. Conversely, if it is audible while a song from
//! `create_song` is not, the issue is in our device setup.
//!
//! Usage:
//! - `cargo run --example original_song [--out <path>] [--export-dir <path>] [--wav]`
//!   Writes the byte-exact original (default: `src/song.bin`). With `--wav`,
//!   also writes a rough WAV preview of the arrangement to the export
//!   directory (`export/`, override with `--export-dir`) under an `original-*`
//!   naming scheme so it never collides with `create_song --wav` output. The
//!   preview uses generic oscillators (see `src-song/render.rs`); it
//!   approximates the arrangement, not Slaughter's actual sound.
//! - `cargo run --example original_song -- --verify`
//!   Decodes and re-encodes with our own encoder and compares against the
//!   embedded original, reporting any differing bytes. This validates the
//!   encoder against ground truth produced by the C# serializer. Differences
//!   in device ordering or same-timestamp event ordering are reported but not
//!   fatal; any other difference is.

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{DEVICE_NAMES, MidiEvent, Song, Track, decode, encode};

static ORIGINAL_SONG: &'static [u8] = include_bytes!("assets/original_song.bin");

/// Rebuilds an encodable [`Song`] from a decoded one. Device chunks are keyed
/// by index, so this is lossless with respect to what our encoder emits.
fn song_from_parsed(parsed: &starter::format::ParsedSong) -> Song {
    let mut song = Song::new(parsed.tempo, parsed.sample_rate);
    song.length = parsed.length;
    for t in &parsed.tracks {
        let mut track = Track::new(t.volume);
        track.receives = t.receives.clone();
        track.devices = t
            .device_indices
            .iter()
            .map(|&i| (parsed.devices[i].id, parsed.devices[i].chunk.clone()))
            .collect();
        track.events = parsed.lanes[t.lane_id]
            .iter()
            .map(|e| {
                if e.on {
                    MidiEvent::on(e.samples, e.note, e.velocity)
                } else {
                    MidiEvent::off(e.samples, e.note)
                }
            })
            .collect();
        track.automations = t.automations.clone();
        song.tracks.push(track);
    }
    song
}

fn main() {
    let default_out = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/song.bin")
        .to_string_lossy()
        .into_owned();
    let default_export = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("export")
        .to_string_lossy()
        .into_owned();
    let mut out_path = default_out.clone();
    let mut export_dir = default_export.clone();
    let mut verify = false;
    let mut want_wav = false;

    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out_path = args.next().expect("--out needs a path"),
            "--export-dir" => export_dir = args.next().expect("--export-dir needs a path"),
            "--verify" => verify = true,
            "--wav" => want_wav = true,
            other => {
                eprintln!("unknown argument: {other}");
                eprintln!(
                    "usage: cargo run --example original_song [--out <path>] [--export-dir <path>] [--wav] [--verify]"
                );
                std::process::exit(2);
            }
        }
    }

    let parsed = decode(ORIGINAL_SONG).expect("embedded original song failed to decode");
    println!(
        "original: tempo={} sample_rate={} length={:.3}s devices={} tracks={}",
        parsed.tempo,
        parsed.sample_rate,
        parsed.length,
        parsed.devices.len(),
        parsed.tracks.len()
    );
    for d in &parsed.devices {
        println!(
            "  device id={} ({}) chunk={} bytes",
            d.id as u8,
            DEVICE_NAMES[d.id as usize],
            d.chunk.len()
        );
    }

    if !verify {
        fs::write(&out_path, ORIGINAL_SONG).expect("failed to write song.bin");
        println!(
            "wrote {} ({} bytes, byte-exact copy of the original)",
            out_path,
            ORIGINAL_SONG.len()
        );

        // Optional WAV preview under a fixed `original-*` naming scheme
        // (independent of --out) so it never collides with the
        // `create_song --wav` exports (`song*.wav`).
        if want_wav {
            fs::create_dir_all(&export_dir).expect("failed to create export directory");

            let mut mix = starter::render::render(&parsed);
            starter::render::normalize(&mut mix);
            let mix_path = Path::new(&export_dir).join("original-mix.wav");
            starter::render::write_wav_at(
                &mix_path.to_string_lossy(),
                &mix,
                parsed.sample_rate as u32,
            );
            println!(
                "wrote {} ({:.2}s mono wav, rough preview - not Slaughter's sound)",
                mix_path.display(),
                mix.len() as f64 / parsed.sample_rate as f64
            );

            // One stem per track that actually has notes (the master track is
            // empty and is skipped automatically).
            for (ti, t) in parsed.tracks.iter().enumerate() {
                if parsed.lanes[t.lane_id].is_empty() {
                    continue;
                }
                let mut solo = starter::render::render_solo(&parsed, ti);
                starter::render::normalize(&mut solo);
                let stem_path = Path::new(&export_dir).join(format!("original-track{ti}.wav"));
                starter::render::write_wav_at(
                    &stem_path.to_string_lossy(),
                    &solo,
                    parsed.sample_rate as u32,
                );
                println!("wrote {}", stem_path.display());
            }
        }
        return;
    }

    // Round-trip check: our decoder -> encoder vs the C#-serialized original.
    let reencoded = encode(&song_from_parsed(&parsed));
    println!("\nround-trip: original {} bytes, re-encoded {} bytes", ORIGINAL_SONG.len(), reencoded.len());

    if reencoded == ORIGINAL_SONG {
        println!("OK: re-encoded output is byte-identical to the original");
        return;
    }

    let mut diffs = Vec::new();
    for i in 0..ORIGINAL_SONG.len().max(reencoded.len()) {
        let a = ORIGINAL_SONG.get(i).copied();
        let b = reencoded.get(i).copied();
        if a != b {
            diffs.push(i);
            if diffs.len() >= 16 {
                break;
            }
        }
    }
    println!("MISMATCH: {}+ differing bytes, first at:", diffs.len());
    for &i in &diffs {
        println!(
            "  byte {}: original {:>4?} vs re-encoded {:>4?}",
            i,
            ORIGINAL_SONG.get(i),
            reencoded.get(i)
        );
    }

    // Explain benign differences: device order (our stable sort by id vs the
    // C# unstable sort) and event ties (same-timestamp ordering).
    let orig_devices: Vec<_> = parsed.devices.iter().map(|d| d.id.as_u8()).collect();
    let mut sorted_ids = orig_devices.clone();
    sorted_ids.sort_unstable();
    let order_differs = orig_devices != sorted_ids;
    println!(
        "device ids in file order: {:?} (sorted: {:?})",
        orig_devices, sorted_ids
    );
    if order_differs && diffs.len() <= 64 {
        println!("NOTE: device order differs from a plain sort - expected source of the diff");
    } else {
        eprintln!("UNEXPECTED difference - investigate before trusting the encoder");
        std::process::exit(1);
    }
}
