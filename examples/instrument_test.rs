//! Generates a 16-bar electro swing instrument test song.
//!
//! Showcases: piano, bass, drums (kick/snare/hats/crash), trumpet, flute,
//! tenor saxophone — all via Falcon 2-operator FM patches.
//!
//! Usage: `cargo run --example instrument_test [--out <path>] [--wav] [--export-dir <path>]`

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{
    DeviceId, MidiEvent, Receive, Song, Track, decode, encode,
};
use starter::instruments::{bass, drums, flute, piano, saxophone, trumpet};
use starter::music::Grid;

const BPM: f64 = 120.0;
const SAMPLE_RATE: i64 = 44100;
const TAIL_SECS: f64 = 2.0;

// ---------------------------------------------------------------------------
// Chord definitions. Root is the bass note (MIDI); tones are semitone offsets.
// ---------------------------------------------------------------------------
type Chord = (i32, [i32; 4]);

const MAJ7: [i32; 4] = [0, 4, 7, 11];
const MIN7: [i32; 4] = [0, 3, 7, 10];
const DOM7: [i32; 4] = [0, 4, 7, 10];
const HALFDIM: [i32; 4] = [0, 3, 6, 10];

const CM7: Chord = (48, MIN7);
const FM7: Chord = (53, MIN7);
const G7: Chord = (55, DOM7);
const BM7: Chord = (59, MAJ7);
const DM7B5: Chord = (50, HALFDIM);

// 16 bars: 4-bar phrases, each stated twice with variation.
const CHORDS: [Chord; 16] = [
    CM7, FM7, CM7, G7,    // phrase A (1st)
    CM7, FM7, CM7, G7,    // phrase A (2nd)
    FM7, G7, BM7, CM7,    // phrase B (1st)
    DM7B5, G7, CM7, G7,   // phrase B (2nd, turnaround)
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
struct Parts {
    piano: Vec<MidiEvent>,
    bass: Vec<MidiEvent>,
    kick: Vec<MidiEvent>,
    snare: Vec<MidiEvent>,
    hat_c: Vec<MidiEvent>,
    hat_o: Vec<MidiEvent>,
    crash: Vec<MidiEvent>,
    trumpet: Vec<MidiEvent>,
    flute: Vec<MidiEvent>,
    sax: Vec<MidiEvent>,
}

fn note_on_off(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, e: i64, dur: i64, note: u8, vel: u8) {
    let start = grid.eighth(bar, e / 2, e % 2);
    let off = start + dur * grid.beat_samples / 2;
    events.push(MidiEvent::on(start, note, vel));
    events.push(MidiEvent::off(off, note));
}

fn chord_notes(root: i32, tones: &[i32; 4], octave_offset: i32) -> [u8; 3] {
    [
        (root + octave_offset * 12 + tones[1]) as u8,
        (root + octave_offset * 12 + tones[2]) as u8,
        (root + octave_offset * 12 + tones[3]) as u8,
    ]
}

// ---------------------------------------------------------------------------
// Drum patterns (electro swing: four-on-the-floor kick, swung hats)
// ---------------------------------------------------------------------------
fn drums_bar(grid: &Grid, bar: i64, p: &mut Parts, intensity: u8) {
    // Kick: four-on-the-floor
    for beat in 0..4 {
        let vel = if beat == 0 { 110 } else { 90 };
        note_on_off(&mut p.kick, grid, bar, beat * 2, 1, 36, vel);
    }
    // Snare: backbeat on 2 and 4
    note_on_off(&mut p.snare, grid, bar, 2, 1, 38, 88);
    note_on_off(&mut p.snare, grid, bar, 6, 1, 38, 95);
    // Ghost snare
    if intensity >= 1 {
        note_on_off(&mut p.snare, grid, bar, 3, 1, 38, 55);
    }
    // Hats: swung eighths, on-beat louder
    for e in 0..8 {
        let vel = if e % 2 == 0 {
            if intensity >= 2 { 92 } else { 78 }
        } else {
            if intensity >= 2 { 56 } else { 44 }
        };
        note_on_off(&mut p.hat_c, grid, bar, e, 1, 42, vel);
    }
    // Open hat accent on phrase boundaries
    if bar % 4 == 0 {
        note_on_off(&mut p.hat_o, grid, bar, 0, 2, 46, 76);
    }
}

// ---------------------------------------------------------------------------
// Piano: swing comping on offbeats
// ---------------------------------------------------------------------------
fn piano_bar(grid: &Grid, bar: i64, p: &mut Parts, pattern_idx: usize) {
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 3);

    // Comping patterns: hit slots (e in 0..8, dur in eighths)
    const PATTERNS: [[(i64, i64); 4]; 4] = [
        [(1, 2), (4, 1), (6, 2), (0, 0)],  // offbeat swing
        [(0, 1), (3, 2), (5, 1), (7, 1)],  // syncopated
        [(2, 2), (5, 2), (7, 1), (0, 0)],  // laid back
        [(1, 1), (4, 1), (6, 2), (0, 0)],  // sparse
    ];

    let pat = &PATTERNS[pattern_idx % 4];
    for &(e, dur) in pat {
        if dur == 0 { continue; }
        let vel = 68 + ((bar * 3 + e) % 5) as u8 * 2;
        for &n in &shell {
            note_on_off(&mut p.piano, grid, bar, e, dur, n, vel);
        }
    }
    // Melody colour: a high note answering the comp
    if bar % 4 == 3 {
        let answer_note = (root as u8) + 36 + 12; // octave above shell
        note_on_off(&mut p.piano, grid, bar, 6, 1, answer_note, 72);
    }
}

// ---------------------------------------------------------------------------
// Bass: walking quarters
// ---------------------------------------------------------------------------
fn bass_bar(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let third = root + if tones[1] == 3 { 3 } else { 4 };
    let fifth = root + 7;
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };

    let notes = [root, third, fifth, approach];
    for (i, &n) in notes.iter().enumerate() {
        let vel = match i {
            0 => 86,
            1 => 76,
            2 => 78,
            _ => 82,
        };
        let start = grid.at(bar, i as i64);
        let off = grid.at(bar, i as i64 + 1);
        p.bass.push(MidiEvent::on(start, n as u8, vel));
        p.bass.push(MidiEvent::off(off, n as u8));
    }
}

// ---------------------------------------------------------------------------
// Melody lines
// ---------------------------------------------------------------------------

/// Trumpet melody: phrase A (bars 4-7), confident lead
fn trumpet_phrase_a(grid: &Grid, bar: i64, p: &mut Parts) {
    let phrases: [[(i64, i64, u8, i64); 6]; 4] = [
        // bar 0 of phrase: rising figure
        [(0, 2, 72, 2), (2, 1, 75, 1), (3, 1, 76, 1), (4, 2, 79, 2), (6, 1, 76, 1), (7, 1, 0, 0)],
        // bar 1: peak and fall
        [(0, 2, 81, 2), (2, 1, 79, 1), (3, 1, 76, 1), (4, 3, 75, 3), (7, 1, 0, 0), (0, 0, 0, 0)],
        // bar 2: call
        [(0, 1, 67, 1), (1, 1, 71, 1), (2, 2, 72, 2), (4, 2, 76, 2), (6, 1, 75, 1), (7, 1, 0, 0)],
        // bar 3: resolve
        [(0, 2, 74, 2), (2, 1, 72, 1), (3, 1, 71, 1), (4, 3, 67, 3), (7, 1, 0, 0), (0, 0, 0, 0)],
    ];
    let local = (bar % 4) as usize;
    let vel_base = 88;
    for &(e, dur, note, _skip) in &phrases[local] {
        if note == 0 || dur == 0 { continue; }
        let vel = (vel_base + ((bar + e) % 3) as u8 * 2).min(108);
        note_on_off(&mut p.trumpet, grid, bar, e, dur, note, vel);
    }
}

/// Flute fills: answering the trumpet in bars 8-11
fn flute_phrase_b(grid: &Grid, bar: i64, p: &mut Parts) {
    let phrases: [[(i64, i64, u8); 5]; 4] = [
        [(0, 2, 84), (2, 1, 83), (4, 2, 81), (6, 1, 79), (7, 1, 0)],
        [(0, 1, 79), (1, 2, 81), (3, 1, 83), (4, 2, 84), (6, 2, 0)],
        [(0, 2, 86), (2, 1, 84), (3, 1, 83), (4, 2, 81), (6, 2, 0)],
        [(0, 3, 79), (3, 1, 77), (4, 2, 76), (6, 2, 0), (0, 0, 0)],
    ];
    let local = (bar % 4) as usize;
    for &(e, dur, note) in &phrases[local] {
        if note == 0 || dur == 0 { continue; }
        let vel = 74 + ((bar * 7 + e) % 4) as u8;
        note_on_off(&mut p.flute, grid, bar, e, dur, note, vel);
    }
}

/// Saxophone melody: bars 12-15, soulful resolution
fn sax_phrase_c(grid: &Grid, bar: i64, p: &mut Parts) {
    let phrases: [[(i64, i64, u8); 5]; 4] = [
        [(0, 1, 71), (1, 2, 74), (3, 1, 76), (4, 2, 79), (6, 2, 0)],
        [(0, 2, 78), (2, 1, 76), (3, 1, 74), (4, 3, 72), (7, 1, 0)],
        [(0, 2, 74), (2, 1, 76), (3, 2, 79), (5, 1, 81), (6, 2, 0)],
        [(0, 1, 79), (1, 1, 76), (2, 2, 74), (4, 3, 72), (7, 1, 0)],
    ];
    let local = (bar % 4) as usize;
    for &(e, dur, note) in &phrases[local] {
        if note == 0 || dur == 0 { continue; }
        let vel = 82 + ((bar * 5 + e) % 5) as u8;
        note_on_off(&mut p.sax, grid, bar, e, dur, note, vel);
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() {
    let default_out = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/song.bin")
        .to_string_lossy()
        .into_owned();
    let default_export = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("export")
        .to_string_lossy()
        .into_owned();

    let mut out_path = default_out;
    let mut export_dir = default_export;
    let mut want_wav = false;

    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out_path = args.next().expect("--out needs a path"),
            "--wav" => want_wav = true,
            "--export-dir" => export_dir = args.next().expect("--export-dir needs a path"),
            other => {
                eprintln!("usage: cargo run --example instrument_test [--out <path>] [--wav] [--export-dir <path>]");
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    let grid = Grid::new(BPM, SAMPLE_RATE);

    let mut p = Parts {
        piano: Vec::new(),
        bass: Vec::new(),
        kick: Vec::new(),
        snare: Vec::new(),
        hat_c: Vec::new(),
        hat_o: Vec::new(),
        crash: Vec::new(),
        trumpet: Vec::new(),
        flute: Vec::new(),
        sax: Vec::new(),
    };

    for bar in 0i64..16 {
        // Drums: full from bar 0, sparse in the last 2 bars
        let intensity = if bar >= 14 { 0 } else if bar >= 12 { 1 } else { 2 };
        drums_bar(&grid, bar, &mut p, intensity);

        // Crash on section boundaries
        if bar % 4 == 0 && bar > 0 {
            note_on_off(&mut p.crash, &grid, bar, 0, 2, 49, 80);
        }

        // Piano: comping throughout
        piano_bar(&grid, bar, &mut p, bar as usize);

        // Bass: walking from bar 0
        bass_bar(&grid, bar, &mut p);

        // Melody: call-and-response across the 4 sections
        match bar {
            0..=3 => {} // intro: rhythm section only
            4..=7 => trumpet_phrase_a(&grid, bar, &mut p),
            8..=11 => flute_phrase_b(&grid, bar, &mut p),
            12..=15 => sax_phrase_c(&grid, bar, &mut p),
            _ => {}
        }
    }

    // --- Assemble the song ---
    let mut song = Song::new(BPM as i32, SAMPLE_RATE as i32);

    // Instruments in track order. Each gets a unique Falcon device.
    let instruments: Vec<(&str, Vec<MidiEvent>, (DeviceId, Vec<u8>))> = vec![
        ("piano", p.piano, piano::piano()),
        ("bass", p.bass, bass::bass()),
        ("kick", p.kick, drums::kick()),
        ("snare", p.snare, drums::snare()),
        ("hat_c", p.hat_c, drums::closed_hat()),
        ("hat_o", p.hat_o, drums::open_hat()),
        ("crash", p.crash, drums::crash()),
        ("trumpet", p.trumpet, trumpet::trumpet()),
        ("flute", p.flute, flute::flute()),
        ("sax", p.sax, saxophone::tenor_sax()),
    ];

    let mut track_names: Vec<String> = Vec::new();
    for (name, events, (dev_id, dev_chunk)) in &instruments {
        let mut track = Track::new(1.0);
        track.devices.push((dev_id.clone(), dev_chunk.clone()));
        track.events = events.clone();
        song.tracks.push(track);
        track_names.push(name.to_string());
    }

    // Master: receives from all tracks
    let mut track_master = Track::new(1.0);
    for send in 0..instruments.len() {
        track_master
            .receives
            .push(Receive::new(send as i32, 0, 1.0));
    }
    song.tracks.push(track_master);

    // Duration
    let mut last_end: i64 = 0;
    for t in &song.tracks {
        for e in &t.events {
            if e.samples > last_end {
                last_end = e.samples;
            }
        }
    }
    song.length = last_end as f64 / SAMPLE_RATE as f64 + TAIL_SECS;

    let data = encode(&song);
    fs::write(&out_path, &data).expect("failed to write song.bin");

    // WAV preview
    if want_wav {
        let parsed = decode(&data).expect("internal error: re-parsing the generated song");
        let stem_base = Path::new(&out_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        fs::create_dir_all(&export_dir).expect("failed to create export directory");

        let mut mix = starter::render::render(&parsed);
        starter::render::normalize(&mut mix);
        let mix_path = Path::new(&export_dir).join(format!("{stem_base}.wav"));
        starter::render::write_wav_at(&mix_path.to_string_lossy(), &mix, SAMPLE_RATE as u32);
        println!(
            "wrote {} ({:.2}s mono wav)",
            mix_path.display(),
            mix.len() as f64 / SAMPLE_RATE as f64
        );

        // Per-instrument stems
        let stems: &[(&str, &[usize])] = &[
            ("piano", &[0]),
            ("bass", &[1]),
            ("drums", &[2, 3, 4, 5, 6]),
            ("trumpet", &[7]),
            ("flute", &[8]),
            ("sax", &[9]),
        ];
        for (name, tracks) in stems {
            let mut buf: Vec<f32> = Vec::new();
            for &ti in *tracks {
                let solo = starter::render::render_solo(&parsed, ti);
                if buf.is_empty() {
                    buf = solo;
                } else {
                    for (i, s) in solo.iter().enumerate() {
                        buf[i] += s;
                    }
                }
            }
            starter::render::normalize(&mut buf);
            let stem_path = Path::new(&export_dir).join(format!("{stem_base}.{name}.wav"));
            starter::render::write_wav_at(&stem_path.to_string_lossy(), &buf, SAMPLE_RATE as u32);
            println!("wrote {}", stem_path.display());
        }
    }

    // Summary
    println!("wrote {} ({} bytes)", out_path, data.len());
    println!(
        "tempo {} bpm | length {:.2}s | 16 bars | electro swing instrument test",
        BPM as i32, song.length,
    );
    for (i, t) in song.tracks[..instruments.len()].iter().enumerate() {
        println!(
            "  track {i}: {:<8} {} notes",
            track_names[i],
            t.events.iter().filter(|e| e.on).count()
        );
    }
    println!("\nPreview with --wav for a mono WAV (approximates arrangement, not FM timbres).");
}
