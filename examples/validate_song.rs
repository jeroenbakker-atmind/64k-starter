//! Validates `src/song.bin` against the WaveSabre serialized-song format and
//! dumps a summary (structure, note grids by track).
//!
//! Usage: `cargo run --example validate_song [<path>]`

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{DEVICE_NAMES, DeviceId, decode};
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
    let mut problems = 0;
    for (i, d) in song.devices.iter().enumerate() {
        let params = starter::format::chunk_params(&d.chunk);
        match d.id {
            DeviceId::Adultery => describe_adultery(i, &params),
            DeviceId::Falcon => describe_falcon(i, &params),
            _ => println!(
                "  [{i}] {} chunk={} bytes ({} params)",
                DEVICE_NAMES[d.id as usize],
                d.chunk.len(),
                params.len()
            ),
        }
        problems += check_device(d.id, &params);
    }

    // Backend inventory
    let mut counts: Vec<(DeviceId, usize)> = Vec::new();
    for d in &song.devices {
        match counts.iter_mut().find(|(id, _)| *id == d.id) {
            Some((_, n)) => *n += 1,
            None => counts.push((d.id, 1)),
        }
    }
    counts.sort_by_key(|(id, _)| *id as u8);
    let inventory = counts
        .iter()
        .map(|(id, n)| format!("{} x{}", DEVICE_NAMES[*id as usize], n))
        .collect::<Vec<_>>()
        .join(", ");
    println!("\nbackend: {}", inventory);

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
    let mut errors = problems;

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

/// Maps a normalized enum param to its variant name (`(int)(v * (n - 1))`).
fn fmt_enum3(v: f32, a: &str, b: &str, c: &str) -> String {
    let i = (v * 2.0).round() as i32;
    match i {
        0 => a.into(),
        1 => b.into(),
        _ => c.into(),
    }
}

fn describe_adultery(i: usize, p: &[f32]) {
    if p.len() < 28 {
        println!("  [{i}] Adultery MALFORMED ({} params, expected 28)", p.len());
        return;
    }
    println!(
        "  [{i}] Adultery sample_index={} loop_mode={} master={}",
        p[0],
        fmt_enum3(p[6], "Disabled", "Repeat", "PingPong"),
        p[25]
    );
}

fn falcon_ratio(coarse: f32, fine: f32) -> f64 {
    let c = coarse as f64;
    let fb = ((fine - 0.5) as f64) * 2.0;
    1.0 + (c * 32.99).floor() + fb * fb * fb
}

fn describe_falcon(i: usize, p: &[f32]) {
    if p.len() < 32 {
        println!("  [{i}] Falcon MALFORMED ({} params, expected 32)", p.len());
        return;
    }
    // Carrier = osc2 (the only audible operator); modulator = osc1.
    let r1 = falcon_ratio(p[1], p[2]);
    let r2 = falcon_ratio(p[10], p[11]);
    let ff = (p[4] as f64).powi(2);
    let fb2 = (p[12] as f64).powi(2);
    println!(
        "  [{i}] Falcon o1 r={:.2} o2 r={:.2} ff^2={:.3} fb2^2*13.25/2={:.2} sustain2={} master={}",
        r1, r2, ff, fb2 * 13.25 / 2.0, p[15], p[17]
    );
}

/// Semantic checks against WaveSabreCore's actual parameter mappings.
/// Returns the number of problems found.
fn check_device(id: DeviceId, p: &[f32]) -> usize {
    const EPS: f32 = 1e-4;
    let mut n = 0;
    match id {
        DeviceId::Adultery => {
            if p.len() < 28 {
                return 1;
            }
            let loop_ok = (p[6] - 0.0).abs() < EPS || (p[6] - 0.5).abs() < EPS;
            if !loop_ok {
                println!(
                    "    WARN Adultery loop_mode={} decodes to {} - did you mean Repeat (0.5)?",
                    p[6],
                    fmt_enum3(p[6], "Disabled", "Repeat", "PingPong")
                );
                n += 1;
            }
            let si = p[0].round();
            if si < 1.0 {
                println!("    WARN Adultery sample_index={} = no sample loaded (silent device)", p[0]);
            } else if si > 495.0 {
                println!(
                    "    WARN Adultery sample_index={} likely exceeds the gm.dls wave pool",
                    p[0]
                );
                n += 1;
            }
        }
        DeviceId::Falcon => {
            if p.len() < 32 {
                return 1;
            }
            for (k, v) in p.iter().enumerate() {
                if !(-EPS..=1.0 + EPS).contains(v) {
                    println!("    WARN Falcon param {k} out of [0,1]: {v}");
                    n += 1;
                    break;
                }
            }
        }
        _ => {}
    }
    n
}