//! Renders a generated song `.bin` to WAV files: the full mix plus one stem
//! per instrument track (optionally grouped with `--stem name:0,1,2`).
//!
//! Generation and rendering are intentionally decoupled: the song generators
//! (`examples/swing_*.rs`, `examples/instrument_test.rs`) only write `.bin`
//! files; this app reads one back and exports audio.
//!
//! Usage:
//!   cargo run --release --example render_song -- <song.bin> [--out-dir <dir>]
//!       [--stem name:0,1,2 ...]
//!
//! Example:
//!   cargo run --release --example render_song -- src/swing_tea.bin \
//!       --stem bass:0 --stem drums:1,2,3,4,5,6 --stem piano:7

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{decode, DeviceId};
use starter::render;

fn main() {
    let mut args = env::args().skip(1).peekable();
    let song_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: cargo run --release --example render_song -- <song.bin> [--out-dir <dir>] [--stem name:0,1,2 ...]");
            std::process::exit(2);
        }
    };

    let mut out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("export")
        .to_string_lossy()
        .into_owned();
    let mut stems: Vec<(String, Vec<usize>)> = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out-dir" => out_dir = args.next().expect("--out-dir needs a path"),
            "--stem" => {
                let spec = args.next().expect("--stem needs 'name:0,1,2'");
                let (name, idxs) = spec.split_once(':').unwrap_or_else(|| {
                    eprintln!("--stem expects name:0,1,2");
                    std::process::exit(2);
                });
                let idxs: Vec<usize> = idxs
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.parse().expect("stem index must be a number"))
                    .collect();
                stems.push((name.to_string(), idxs));
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    let data = fs::read(&song_path).expect("failed to read song.bin");
    let parsed = decode(&data).expect("internal error: re-parsing the generated song");
    let sr = parsed.sample_rate.max(1) as f64;

    let stem_base = Path::new(&song_path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    // Device names by id, for the per-track listing.
    let dev_name = |id: DeviceId| -> &'static str {
        match id {
            DeviceId::Falcon => "falcon",
            DeviceId::Slaughter => "slaughter",
            DeviceId::Adultery => "adultery",
            DeviceId::Specimen => "specimen",
            DeviceId::Scissor => "scissor",
            DeviceId::Leveller => "leveller",
            DeviceId::Crusher => "crusher",
            DeviceId::Echo => "echo",
            DeviceId::Smasher => "smasher",
            DeviceId::Chamber => "chamber",
            DeviceId::Twister => "twister",
            DeviceId::Cathedral => "cathedral",
            DeviceId::Thunder => "thunder",
        }
    };

    println!(
        "song: tempo={} sample_rate={} length={:.3}s tracks={}",
        parsed.tempo,
        parsed.sample_rate,
        parsed.length,
        parsed.tracks.len()
    );
    for (ti, t) in parsed.tracks.iter().enumerate() {
        let devs: Vec<String> = t
            .device_indices
            .iter()
            .filter_map(|di| parsed.devices.get(*di as usize))
            .map(|d| dev_name(d.id).to_string())
            .collect();
        let recv: Vec<String> = t
            .receives
            .iter()
            .map(|r| format!("t{}->ch{}@{:.2}", r.sending_track, r.channel, r.volume))
            .collect();
        println!(
            "  track {ti}: vol={:.2} lane={} devices={}{}",
            t.volume,
            t.lane_id,
            devs.join(","),
            if recv.is_empty() {
                String::new()
            } else {
                format!(" receives: {}", recv.join(" "))
            },
        );
    }

    fs::create_dir_all(&out_dir).expect("failed to create out-dir");

    // Full mix (stereo).
    let mut mix = render::render_stereo(&parsed);
    render::normalize_stereo(&mut mix);
    let mix_path = Path::new(&out_dir).join(format!("{stem_base}.wav"));
    render::write_stereo_wav_at(&mix_path.to_string_lossy(), &mix, sr as u32);
    println!(
        "wrote {} ({:.2}s, {} samples)",
        mix_path.display(),
        mix.len() as f64 / sr,
        mix.len()
    );

    // Per-instrument stems (default: one per track, in track order).
    let groups: Vec<(String, Vec<usize>)> = if stems.is_empty() {
        let mut g = Vec::new();
        for ti in 0..parsed.tracks.len() {
            g.push((format!("t{}", ti), vec![ti]));
        }
        g
    } else {
        stems
    };

    for (name, tracks) in groups {
        if tracks.is_empty() {
            continue;
        }
        let mut buf: Vec<[f32; 2]> = Vec::new();
        for &ti in &tracks {
            let solo = render::render_solo_stereo(&parsed, ti);
            if buf.is_empty() {
                buf = solo;
            } else {
                for (i, s) in solo.iter().enumerate() {
                    buf[i][0] += s[0];
                    buf[i][1] += s[1];
                }
            }
        }
        render::normalize_stereo(&mut buf);
        let stem_path = Path::new(&out_dir).join(format!("{stem_base}.{name}.wav"));
        render::write_stereo_wav_at(&stem_path.to_string_lossy(), &buf, sr as u32);
        println!("wrote {} ({:.2}s)", stem_path.display(), buf.len() as f64 / sr);
    }
}