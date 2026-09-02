//! Shared plumbing for the 16-bar electro-swing test songs (`examples/`).
//! Each song is its own example; this module supplies the time grid, event
//! helpers, chord vocabulary and track assembly so the song files only carry
//! musical content. Generators write the encoded `.bin` only — rendering to
//! WAV is `examples/render_song`, a separate app that reads the `.bin`.

use std::env;
use std::fs;

use common::{encode, DeviceId, MidiEvent, Receive, Song, Track};
pub use crate::music::Grid;

pub const SAMPLE_RATE: i64 = 44100;
pub const TAIL_SECS: f64 = 2.5;

// ---------------------------------------------------------------------------
// Harmony: a small electro-swing chord vocabulary (root, + tones from the
// root). Electro swing leans on minor-seventh and dominant-seventh colours,
// so the whole kit stays in the D-blues / relative-minor family.
// ---------------------------------------------------------------------------

pub type Chord = (i32, [i32; 4]);

pub const MIN7: [i32; 4] = [0, 3, 7, 10];
pub const DOM7: [i32; 4] = [0, 4, 7, 10];
pub const MAJ7: [i32; 4] = [0, 4, 7, 11];

pub const DM7: Chord = (50, MIN7); // Dm7:  home (D blue note register)
pub const GM7: Chord = (55, MIN7); // Gm7:  iv
pub const A7: Chord = (57, DOM7); // A7:   V (blues seven)
pub const FM7: Chord = (53, MAJ7); // FM7:  relative major
pub const AM7: Chord = (45, MIN7); // Am7:  gypsy home
pub const D7: Chord = (50, DOM7); // D7:   V of gypsy home
pub const E7: Chord = (52, DOM7); // E7:   "Spanish" dominant

// ---------------------------------------------------------------------------
// Parts: every song shares the same Falcon instrument kit; songs simply leave
// parts empty when they do not want an instrument.
// ---------------------------------------------------------------------------
#[derive(Default)]
pub struct Parts {
    pub kick: Vec<MidiEvent>,
    pub snare: Vec<MidiEvent>,
    pub hat_c: Vec<MidiEvent>,
    pub shake: Vec<MidiEvent>,
    pub bass: Vec<MidiEvent>,
    pub piano: Vec<MidiEvent>,
    pub flute: Vec<MidiEvent>,
    pub sax: Vec<MidiEvent>,
}

impl Parts {
    pub fn new() -> Parts {
        Parts::default()
    }
}

pub const KICK: u8 = 36;
pub const SNARE: u8 = 38;
pub const HAT_CLOSED: u8 = 42;
pub const SHAKER: u8 = 54;

// ---------------------------------------------------------------------------
// Event helpers. `e` is an eighth index within the bar (0..8, `dur` in
// eighths); `e16` is a sixteenth index (0..16). The `_h` variant is lazy-
// quantized with deterministic jitter, for "live" parts.
// ---------------------------------------------------------------------------
pub fn on_off(
    events: &mut Vec<MidiEvent>,
    grid: &Grid,
    bar: i64,
    e: i64,
    dur: i64,
    note: u8,
    vel: u8,
) {
    let start = grid.eighth(bar, e / 2, e % 2);
    let off = start + dur * grid.beat_samples / 2;
    events.push(MidiEvent::on(start, note, vel));
    events.push(MidiEvent::off(off, note));
}

pub fn on_off_16(
    events: &mut Vec<MidiEvent>,
    grid: &Grid,
    bar: i64,
    e16: i64,
    dur16: i64,
    note: u8,
    vel: u8,
) {
    let start = grid.at(bar, 0) + e16 * grid.beat_samples / 4;
    let off = start + dur16 * grid.beat_samples / 4;
    events.push(MidiEvent::on(start, note, vel));
    events.push(MidiEvent::off(off, note));
}

/// Deterministic timing jitter in samples (±~0.9 ms) for lazy quantization.
fn jitter_samples(bar: i64, e: i64, note: u8) -> i64 {
    let x = (bar * 131 + e * 17 + note as i64 * 7) as u32 & 0xffff;
    (x % 81) as i64 - 40
}

/// Deterministic velocity jitter (base ±4) so "live" parts breathe.
fn jitter_vel(vel: u8, bar: i64, e: i64, note: u8) -> u8 {
    let x = (bar * 37 + e * 11 + note as i64 * 5) as u32 & 0xff;
    (vel as i64 + (x % 9) as i64 - 4).clamp(30, 127) as u8
}

pub fn on_off_h(
    events: &mut Vec<MidiEvent>,
    grid: &Grid,
    bar: i64,
    e: i64,
    dur: i64,
    note: u8,
    vel: u8,
) {
    // No monotonicity clamp: the encoder sorts events by sample, and clamping
    // a note-on to the previous note-off would cascade later chord voices off
    // the beat. Jitter is only ±40 samples, far shorter than any note length.
    // We only clamp the first notes of the song so jitter can't dip below 0
    // (a negative sample would delta-code as a negative event).
    let start = (grid.eighth(bar, e / 2, e % 2) + jitter_samples(bar, e, note)).max(0);
    let off = start + dur * grid.beat_samples / 2;
    events.push(MidiEvent::on(start, note, jitter_vel(vel, bar, e, note)));
    events.push(MidiEvent::off(off, note));
}

/// Three chord tones (3rd, 5th, 7th) above the root at an octave offset:
/// `chord_notes(root, &tones, 0)` is the tight voicing, `1` an octave up.
pub fn chord_notes(root: i32, tones: &[i32; 4], octave: i32) -> [u8; 3] {
    [
        (root + octave * 12 + tones[1]) as u8,
        (root + octave * 12 + tones[2]) as u8,
        (root + octave * 12 + tones[3]) as u8,
    ]
}

/// Maps a chord degree to a note guaranteed inside the chord: 0 = root +1
/// octave, 1 = root +2 octaves, 2..4 = 3rd/5th/7th above the root, 5..7 the
/// same one octave up. Keeps song melodies diatonic by construction.
pub fn chord_degree(root: i32, tones: &[i32; 4], d: u8) -> u8 {
    match d {
        0 => (root + 12) as u8,
        1 => (root + 24) as u8,
        2 => (root + 12 + tones[1]) as u8,
        3 => (root + 12 + tones[2]) as u8,
        4 => (root + 12 + tones[3]) as u8,
        5 => (root + 24 + tones[1]) as u8,
        6 => (root + 24 + tones[2]) as u8,
        _ => (root + 24 + tones[3]) as u8,
    }
}

/// Lays a row of `(eighth, dur_eighths, note)` notes into `events`, played
/// lazily out of the grid (humanized). A row is one bar of melody.
pub fn spread(
    events: &mut Vec<MidiEvent>,
    grid: &Grid,
    bar: i64,
    notes: &[(i64, i64, u8)],
    vel: u8,
) {
    for &(e, dur, n) in notes {
        on_off_h(events, grid, bar, e, dur, n, vel);
    }
}

// ---------------------------------------------------------------------------
// Assembly: one track per instrument, then a master track that receives from
// all of them (matching `examples/instrument_test`). Returns the encoded
// `song.bin` bytes.
// ---------------------------------------------------------------------------
pub struct Placed {
    pub name: &'static str,
    pub events: Vec<MidiEvent>,
    pub dev: (DeviceId, Vec<u8>),
}

pub fn assemble(bpm: f64, tail_secs: f64, placed: Vec<Placed>) -> Vec<u8> {
    let mut song = Song::new(bpm as i32, SAMPLE_RATE as i32);
    for p in placed {
        let mut track = Track::new(1.0);
        track.devices.push(p.dev.clone());
        track.events = p.events;
        song.tracks.push(track);
    }
    let n = song.tracks.len();
    let mut master = Track::new(1.0);
    for i in 0..n {
        master.receives.push(Receive::new(i as i32, 0, 1.0));
    }
    song.tracks.push(master);

    let mut last_end: i64 = 0;
    for t in &song.tracks {
        for e in &t.events {
            if e.samples > last_end {
                last_end = e.samples;
            }
        }
    }
    song.length = last_end as f64 / SAMPLE_RATE as f64 + tail_secs;
    encode(&song)
}

// ---------------------------------------------------------------------------
// Options (generators write a `.bin` only; rendering is `examples/render_song`).
// ---------------------------------------------------------------------------
pub struct Opts {
    pub out: String,
}

pub fn parse_args(usage: &str, default_out: &str) -> Opts {
    let mut out = default_out.to_string();

    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = args.next().expect("--out needs a path"),
            other => {
                eprintln!("usage: {usage}");
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    Opts { out }
}

/// Writes the encoded song bytes to `path`.
pub fn write_bin(data: &[u8], path: &str) {
    fs::write(path, data).expect("failed to write song.bin");
    let bytes = data.len();
    println!("wrote {path} ({bytes} bytes)");
}

/// Assembles and writes a song plus a `<path>.md` manifest listing its placed
/// track names (track index per line), so `render_song` can name the stems
/// (`song-...t3-piano.wav`) and export the manifest alongside the audio.
pub fn write_song(bpm: f64, tail_secs: f64, placed: Vec<Placed>, path: &str) {
    let names: Vec<&str> = placed.iter().map(|p| p.name).collect();
    let data = assemble(bpm, tail_secs, placed);
    write_bin(&data, path);
    let md = format!(
        "# Track list\n\n{}",
        names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{i}: {n}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(format!("{path}.md"), md).expect("failed to write track manifest");
    println!("wrote {path}.md");
}