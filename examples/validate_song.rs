//! Validates `src/song.bin` against the WaveSabre serialized-song format and
//! dumps a summary (structure, note grids by track).
//!
//! Usage: `cargo run --example validate_song [<path>]`

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{DEVICE_NAMES, decode};
use starter::music::{fmt_time, note_name};

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/song.bin")
                .to_string_lossy()
                .into_owned()
        });

    let data = fs::read(&path).expect("failed to read song.bin");
    println!("reading {} ({} bytes)\n", path, data.len());

    let song = match decode(&data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("INVALID: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "tempo={} sample_rate={} length={:.2}s (={} samples)",
        song.tempo,
        song.sample_rate,
        song.length,
        (song.length * song.sample_rate as f64) as i64
    );

    println!("\ndevices ({}):", song.devices.len());
    for (i, d) in song.devices.iter().enumerate() {
        let params = starter::format::chunk_params(&d.chunk);
        let sample = params.first().copied().unwrap_or(0.0);
        println!(
            "  [{i}] id={} ({}) chunk={} bytes sample_index={}",
            d.id as u8,
            DEVICE_NAMES[d.id as usize],
            d.chunk.len(),
            sample
        );
    }

    println!("\ntracks ({}):", song.tracks.len());
    for (i, t) in song.tracks.iter().enumerate() {
        println!(
            "  track {i}: vol={} receives={:?} devices={:?} lane={} autos={}",
            t.volume,
            t.receives
                .iter()
                .map(|r| format!("({}->{}, {})", r.sending_track, r.channel, r.volume))
                .collect::<Vec<_>>()
                .join(","),
            t.device_indices,
            t.lane_id,
            t.automations.len()
        );
    }

    // Structural checks
    let mut errors = 0;

    // every note-on should be paired with a note-off
    for (li, lane) in song.lanes.iter().enumerate() {
        let mut on_pending: Vec<u8> = Vec::new();
        for (ei, e) in lane.iter().enumerate() {
            if e.on {
                on_pending.push(e.note);
            } else {
                if let Some(pos) = on_pending.iter().position(|n| *n == e.note) {
                    on_pending.remove(pos);
                } else {
                    println!(
                        "  WARN lane {li} event {ei}: note-off {} without a matching note-on",
                        note_name(e.note)
                    );
                    errors += 1;
                }
            }
        }
        if !on_pending.is_empty() {
            for n in &on_pending {
                println!(
                    "  WARN lane {li}: note-on {} never released",
                    note_name(*n)
                );
                errors += 1;
            }
        }
    }

    // all events must fit within the declared song length
    let length_samples = (song.length * song.sample_rate as f64) as i64;
    for (li, lane) in song.lanes.iter().enumerate() {
        for e in lane {
            if e.samples > length_samples {
                println!(
                    "  WARN lane {li}: event at {} exceeds song length",
                    fmt_time(e.samples, song.sample_rate as i64)
                );
                errors += 1;
            }
        }
    }

    if errors > 0 {
        eprintln!("\n{} problem(s) found", errors);
    } else {
        println!("\nOK: structure looks valid");
    }

    // Note grid dump (compact, chronologically merged per track)
    println!("\nnote grid:");
    for (ti, t) in song.tracks.iter().enumerate() {
        let lane = &song.lanes[t.lane_id];
        if lane.is_empty() {
            continue;
        }
        let name = if ti == song.tracks.len() - 1 {
            "master"
        } else {
            &["piano", "bass", "kick", "snare", "hat_c", "hat_o"][ti]
        };
        println!("  track {ti} ({name}, lane {}):", t.lane_id);
        let mut cur: Option<u8> = None;
        let mut cur_start = 0i64;
        for e in lane {
            if e.on {
                if let Some(n) = cur {
                    println!(
                        "    {:>10} {:<4} {:>10}",
                        fmt_time(cur_start, song.sample_rate as i64),
                        note_name(n),
                        fmt_time(e.samples - cur_start, song.sample_rate as i64)
                    );
                }
                cur_start = e.samples;
                cur = Some(e.note);
            } else if cur == Some(e.note) {
                println!(
                    "    {:>10} {:<4} {:>10}",
                    fmt_time(cur_start, song.sample_rate as i64),
                    note_name(e.note),
                    fmt_time(e.samples - cur_start, song.sample_rate as i64)
                );
                cur = None;
            }
        }
    }
}