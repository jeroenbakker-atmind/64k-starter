//! Generates a 32-bar electro swing track written from scratch.
//!
//! Follows the Lyric Assistant guide "How to write electro swing songs":
//!   - Swung groove: one grid, every part swings (long-short eighth pairs).
//!   - Steady anchors (kick, backbeat snare, backbeat ghost) under swung
//!     hats and lazy-quantized piano/bass/brass.
//!   - D blues harmony: Dm7 home, Gm7 iv, A7 blues seven, FM7 relative major.
//!     Every chord stays in the blues family so melody and harmony always
//!     agree (no borrowed dominants that fight the fixed topline).
//!   - Arrangement recipe: Intro (piano rag motif + brass) -> Verse (spare)
//!     -> Pre-chorus (build) -> Chorus (full brass + hook) -> Breakdown
//!     (kick + hat) -> Final chorus with a resolve.
//!   - Voice roles: the flute owns the topline; the tenor sax is a quiet
//!     third voice with short stabs and spare call-and-response answers.
//!
//! Showcases the Falcon FM instruments: piano, bass, drums, flute, tenor sax.
//!
//! Usage: `cargo run --example instrument_test [--out <path>] [--wav] [--export-dir <path>]`

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{decode, encode, DeviceId, MidiEvent, Receive, Song, Track};
use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::music::Grid;

const BPM: f64 = 122.0;
const SAMPLE_RATE: i64 = 44100;
const TAIL_SECS: f64 = 2.5;

// ---------------------------------------------------------------------------
// Harmony: D blues (D F G Ab A C). Dm7 is home; A7 is the blues seven;
// Gm7 is the iv; FM7 is the relative major. All chords stay inside the blues
// family so the fixed melody and the harmony always agree.
// ---------------------------------------------------------------------------
type Chord = (i32, [i32; 4]);

const MIN7: [i32; 4] = [0, 3, 7, 10];
const DOM7: [i32; 4] = [0, 4, 7, 10];
const MAJ7: [i32; 4] = [0, 4, 7, 11];

const DM7: Chord = (50, MIN7); // Dm7: tonic
const GM7: Chord = (55, MIN7); // Gm7: iv
const A7: Chord = (57, DOM7); // A7: V (blues seven)
const FM7: Chord = (53, MAJ7); // FM7: relative major

// 32 bars: Intro(4) Verse(8) Pre-chorus(4) Chorus(8) Breakdown(4) Final(4).
const CHORDS: [Chord; 32] = [
    DM7, GM7, DM7, A7, // intro
    DM7, GM7, DM7, A7, // verse (1st)
    FM7, A7, A7, DM7, // verse (2nd: blues A7 keeps it idiomatic)
    DM7, A7, GM7, A7, // pre-chorus build
    DM7, GM7, DM7, A7, // chorus (1st)
    FM7, DM7, A7, A7, // chorus (2nd)
    DM7, GM7, DM7, A7, // breakdown
    DM7, GM7, DM7, A7, // final chorus + turnaround
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
    shake: Vec<MidiEvent>,
    flute: Vec<MidiEvent>,
    sax: Vec<MidiEvent>,
}

fn note_on_off(
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

/// Deterministic timing jitter in samples (±~0.9 ms) for lazy quantization.
fn jitter_samples(bar: i64, e: i64, note: u8) -> i64 {
    let x = (bar * 131 + e * 17 + note as i64 * 7) as u32 & 0xffff;
    (x % 81) as i64 - 40
}

/// Deterministic velocity jitter (base ±4) so the "live" parts breathe.
fn jitter_vel(vel: u8, bar: i64, e: i64, note: u8) -> u8 {
    let x = (bar * 37 + e * 11 + note as i64 * 5) as u32 & 0xff;
    (vel as i64 + (x % 9) as i64 - 4).clamp(30, 127) as u8
}

/// Humanized version for piano/bass/horns: lazy quantization keeps them loose
/// on the swung grid, per the "quantize less for live elements" groove tip.
fn note_on_off_h(
    events: &mut Vec<MidiEvent>,
    grid: &Grid,
    bar: i64,
    e: i64,
    dur: i64,
    note: u8,
    vel: u8,
) {
    let prev = events.last().map_or(0, |ev| ev.samples);
    let start = (grid.eighth(bar, e / 2, e % 2) + jitter_samples(bar, e, note)).max(prev + 1);
    let off = start + dur * grid.beat_samples / 2;
    events.push(MidiEvent::on(start, note, jitter_vel(vel, bar, e, note)));
    events.push(MidiEvent::off(off, note));
}

fn chord_notes(root: i32, tones: &[i32; 4], octave_offset: i32) -> [u8; 3] {
    [
        (root + octave_offset * 12 + tones[1]) as u8,
        (root + octave_offset * 12 + tones[2]) as u8,
        (root + octave_offset * 12 + tones[3]) as u8,
    ]
}

fn section(bar: i64) -> &'static str {
    match bar {
        0..=3 => "intro",
        4..=11 => "verse",
        12..=15 => "pre",
        16..=23 => "chorus",
        24..=27 => "breakdown",
        _ => "outro",
    }
}

// ---------------------------------------------------------------------------
// Drums: steady anchors (kick, backbeat, ghost) with swung hats; the
// pre-chorus/chorus add a tighter 16th-hat build. Breakdown = kick + hat.
// ---------------------------------------------------------------------------
fn drums_bar(grid: &Grid, bar: i64, p: &mut Parts, intensity: u8) {
    for beat in 0..4 {
        let vel = if beat == 0 { 108 } else { 88 };
        note_on_off(&mut p.kick, grid, bar, beat * 2, 1, 36, vel);
    }
    if intensity >= 2 {
        note_on_off(&mut p.snare, grid, bar, 2, 1, 38, 86);
        note_on_off(&mut p.snare, grid, bar, 6, 1, 38, 93);
        if intensity >= 3 {
            note_on_off(&mut p.snare, grid, bar, 3, 1, 38, 52);
        }
    }
    // Swung eighths: offbeats accented (the swung "&" is the groove's engine).
    for e in 0..8 {
        let vel = if e % 2 == 0 {
            if intensity == 1 {
                58
            } else {
                70
            }
        } else {
            if intensity == 1 {
                74
            } else {
                92
            }
        };
        note_on_off(&mut p.hat_c, grid, bar, e, 1, 42, vel);
    }
    // Tighter 16th-hat build in the back half of pre-chorus and chorus bars.
    if intensity >= 3 {
        for k in 0..8 {
            let t = grid.at(bar, 2) + k * grid.beat_samples / 4;
            let vel = 58 + k as u8 * 4;
            p.hat_c.push(MidiEvent::on(t, 42, vel));
            p.hat_c.push(MidiEvent::off(t + grid.beat_samples / 4, 42));
        }
    }
}

/// 16th-note snare roll into the next section's downbeat (biggie on drops).
fn fill_bar(grid: &Grid, bar: i64, p: &mut Parts) {
    for k in 0..8 {
        let t = grid.at(bar, 2) + k * grid.beat_samples / 4;
        let vel = 50 + k as u8 * 8;
        let n = if k == 7 { 36 } else { 38 };
        let events = if k == 7 { &mut p.kick } else { &mut p.snare };
        events.push(MidiEvent::on(t, n, if k == 7 { vel + 10 } else { vel }));
        events.push(MidiEvent::off(t + grid.beat_samples / 4, n));
    }
}

/// Swung shaker glue: a soft shaker riding every offbeat ("&"). This is the
/// shuffled percussion layer that makes the swing audible over the steady
/// kick/snare anchors (groove tip: add shaker, tambourine, or brushes).
fn shaker_bar(grid: &Grid, bar: i64, p: &mut Parts) {
    for e in [1i64, 3, 5, 7] {
        let v = if e == 3 || e == 7 { 84 } else { 72 };
        note_on_off(&mut p.shake, grid, bar, e, 1, 54, v);
    }
}

// ---------------------------------------------------------------------------
// Bass: D blues bounce by default; chromatic walk-downs on the "fresh"
// middle-section bars; sparse root pulses through the breakdown.
// ---------------------------------------------------------------------------
fn bass_bar(grid: &Grid, bar: i64, p: &mut Parts, walk: bool, sparse: bool) {
    let (root, tones) = CHORDS[bar as usize];
    let fifth = root + 7;
    let next_root = CHORDS[((bar + 1) % 32) as usize].0;
    let approach = if next_root > root {
        next_root - 1
    } else {
        next_root + 1
    };

    let mut hits: Vec<(i64, i32, i64, u8)>;
    if sparse {
        hits = vec![
            (0, root, 2, 82),
            (4, root + if tones[1] == 3 { 3 } else { 4 }, 2, 74),
        ];
    } else if walk {
        // Old-school chromatic walk-down, one step per beat.
        hits = vec![
            (0, root, 2, 86),
            (2, root - 1, 1, 74),
            (4, root - 2, 1, 74),
            (6, root - 3, 1, 82),
        ];
    } else {
        // Swing bounce: root / 2& fifth pop / chromatic push into next root.
        hits = vec![(0, root, 2, 86), (3, fifth, 1, 95), (7, approach, 1, 100)];
    }
    for (e, n, dur, vel) in hits.drain(..) {
        note_on_off_h(&mut p.bass, grid, bar, e, dur, n as u8, vel);
    }
}

// ---------------------------------------------------------------------------
// Piano: ragtime motif on the intro (the "Ragtime Flip"), then a swung comp
// (chords on "2&" and a ringing "& of 4"). Quiet in the breakdown.
// ---------------------------------------------------------------------------
fn piano_rag(grid: &Grid, bar: i64, p: &mut Parts) {
    // One-bar D blues riff at eighth density, accents marking the rag bounce.
    let rag: [(i64, u8, u8); 8] = [
        (0, 62, 82),
        (1, 65, 70),
        (2, 67, 70),
        (3, 69, 78),
        (4, 69, 84),
        (5, 67, 70),
        (6, 65, 70),
        (7, 62, 80),
    ];
    for (e, note, vel) in rag {
        note_on_off_h(&mut p.piano, grid, bar, e, 1, note, vel);
    }
}

fn piano_comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);

    // Hit "2&" staccato, ring "& of 4" into the next bar.
    for &(e, dur) in &[(3, 1), (7, 2)] {
        let vel = 64 + ((bar % 3) as u8) * 2;
        for &n in &shell {
            note_on_off_h(&mut p.piano, grid, bar, e, dur, n, vel);
        }
    }
    // Ringing top answer on phrase ends.
    if bar % 4 == 3 {
        note_on_off_h(&mut p.piano, grid, bar, 6, 1, (root + 24) as u8, 70);
    }
}

// ---------------------------------------------------------------------------
// Brass: short stabs on strong beats, octave-doubled tenor sax roots that
// punch with the downbeat. Intro/outro Charleston, pre-chorus pushes, chorus
// downbeat accents, and the final A7 -> Dm turnaround blast.
// ---------------------------------------------------------------------------
fn brass_stab(grid: &Grid, bar: i64, e: i64, dur: i64, p: &mut Parts) {
    let (root, _) = CHORDS[bar as usize];
    for &n in &[(root + 12) as u8, (root + 24) as u8] {
        note_on_off_h(&mut p.sax, grid, bar, e, dur, n, 92);
    }
}

fn brass_section(grid: &Grid, bar: i64, p: &mut Parts) {
    match bar {
        2 | 3 => {
            brass_stab(grid, bar, 0, 1, p);
            brass_stab(grid, bar, 4, 1, p);
        }
        12..=15 => {
            // Build: an offbeat push every bar, leaning into the next.
            brass_stab(grid, bar, 7, 1, p);
        }
        16..=23 => {
            // Chorus: short unison accents on the strong beats (with the snare).
            brass_stab(grid, bar, 0, 1, p);
            if bar == 19 || bar == 23 {
                brass_stab(grid, bar, 7, 1, p);
            }
        }
        28 | 29 | 30 => {
            brass_stab(grid, bar, 0, 1, p);
            brass_stab(grid, bar, 4, 1, p);
        }
        31 => {
            // Turnaround on the dominant: D-F anticipating, then the A7 -> Dm
            // resolve blast on the "& of 4" with a 16th-hat riser into it.
            note_on_off_h(&mut p.sax, grid, bar, 2, 1, 62, 96);
            note_on_off_h(&mut p.sax, grid, bar, 2, 1, 69, 90);
            for k in 0..8 {
                let t = grid.at(bar, 3) + k * grid.beat_samples / 4;
                let vel = 50 + k as u8 * 8;
                p.hat_c.push(MidiEvent::on(t, 42, vel));
                p.hat_c.push(MidiEvent::off(t + grid.beat_samples / 4, 42));
            }
            for n in [62u8, 65, 69] {
                note_on_off_h(&mut p.sax, grid, bar, 7, 4, n, 110);
            }
            note_on_off_h(&mut p.flute, grid, bar, 7, 4, 74, 96);
            note_on_off(&mut p.crash, grid, bar, 7, 4, 49, 100);
            note_on_off(&mut p.hat_o, grid, bar, 7, 4, 46, 100);
            note_on_off_h(&mut p.bass, grid, bar, 7, 4, 50, 92);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Voice roles: the flute owns the topline (motif-based, D blues); the tenor
// sax is a quiet third voice that answers with spare fragments and fills.
// All pitch content is D blues (D F G Ab A C).
// ---------------------------------------------------------------------------
fn phrase_on(
    events: &mut Vec<MidiEvent>,
    grid: &Grid,
    bar: i64,
    phrase: &[(i64, i64, u8); 5],
    vel_base: u8,
    transpose: i64,
) {
    for &(e, dur, note) in phrase {
        if note == 0 || dur == 0 {
            continue;
        }
        let n = (note as i64 + transpose) as u8;
        let vel = vel_base + ((bar * 7 + e) % 4) as u8;
        note_on_off_h(events, grid, bar, e, dur, n, vel);
    }
}

/// Sax/flute per-section tables; each returns the rows for bars in the
/// section. (Rows are 5-event phrases; a zero note means rest.)
fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    // Intro riff (bars 2-3).
    let intro_sax: [[(i64, i64, u8); 5]; 2] = [
        [(0, 2, 62), (2, 1, 65), (3, 1, 67), (4, 2, 69), (6, 2, 0)],
        [(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 62), (6, 2, 0)],
    ];
    let intro_flute: [[(i64, i64, u8); 5]; 2] = [
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 2, 79), (7, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 2, 81), (7, 0, 0)],
    ];
    // Verse: spare topline; the sax answers late in the bar.
    let verse_sax: [[(i64, i64, u8); 5]; 8] = [
        [(0, 2, 62), (3, 1, 65), (4, 2, 67), (6, 2, 69), (7, 0, 0)],
        [(0, 3, 69), (4, 2, 67), (6, 1, 65), (7, 0, 0), (0, 0, 0)],
        [(0, 2, 68), (2, 1, 67), (3, 1, 65), (4, 3, 62), (7, 0, 0)],
        [(0, 2, 67), (2, 1, 69), (3, 1, 72), (4, 3, 74), (7, 0, 0)],
        [(0, 1, 74), (1, 2, 72), (3, 1, 69), (4, 2, 67), (6, 2, 0)],
        [(0, 3, 65), (4, 2, 68), (6, 1, 67), (7, 0, 0), (0, 0, 0)],
        [(0, 2, 69), (2, 1, 68), (3, 1, 67), (4, 2, 65), (6, 2, 0)],
        [(0, 3, 62), (4, 1, 69), (5, 1, 67), (6, 1, 65), (7, 0, 0)],
    ];
    let verse_flute: [[(i64, i64, u8); 5]; 8] = [
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 1, 79), (7, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 1, 77), (7, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 2, 79), (7, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 2, 81), (7, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (4, 1, 79), (6, 2, 77)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (4, 2, 80), (6, 2, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (4, 1, 81), (6, 2, 79)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 2, 74), (7, 0, 0)],
    ];
    // Pre-chorus: rising, denser, more urgent (fewer rests).
    let pre_sax: [[(i64, i64, u8); 5]; 4] = [
        [(0, 1, 62), (1, 1, 65), (2, 1, 67), (3, 2, 69), (6, 2, 0)],
        [(0, 1, 69), (1, 1, 72), (2, 2, 74), (4, 2, 72), (6, 1, 69)],
        [(0, 1, 72), (1, 1, 74), (2, 2, 72), (4, 1, 69), (6, 1, 68)],
        [(0, 2, 74), (2, 1, 72), (3, 1, 69), (4, 3, 67), (7, 0, 0)],
    ];
    let pre_flute: [[(i64, i64, u8); 5]; 4] = [
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (4, 2, 84), (6, 2, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (4, 2, 81), (6, 1, 79)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (4, 2, 80), (6, 2, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (6, 2, 84), (7, 0, 0)],
    ];
    // Chorus: a repeatable hook on the flute; the sax adds spare colour.
    let chorus_sax: [[(i64, i64, u8); 5]; 8] = [
        [(0, 2, 74), (2, 1, 72), (3, 1, 74), (4, 2, 69), (6, 2, 67)],
        [(0, 2, 65), (2, 2, 68), (4, 2, 67), (6, 1, 65), (7, 0, 0)],
        [(0, 2, 69), (2, 1, 67), (3, 1, 68), (4, 2, 69), (6, 2, 72)],
        [(0, 3, 74), (3, 1, 72), (4, 3, 69), (7, 0, 0), (0, 0, 0)],
        [(0, 2, 74), (2, 1, 72), (3, 1, 74), (4, 2, 72), (6, 2, 69)],
        [(0, 2, 68), (2, 2, 65), (4, 3, 62), (7, 0, 0), (0, 0, 0)],
        [(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 68), (6, 2, 67)],
        [(0, 4, 72), (5, 1, 74), (6, 2, 0), (0, 0, 0), (0, 0, 0)],
    ];
    let chorus_flute: [[(i64, i64, u8); 5]; 8] = [
        [(0, 2, 81), (2, 1, 79), (3, 1, 77), (4, 2, 74), (6, 2, 0)],
        [(0, 2, 77), (2, 2, 80), (4, 2, 79), (6, 2, 0), (0, 0, 0)],
        [(0, 2, 79), (2, 1, 77), (3, 1, 74), (4, 2, 72), (6, 2, 0)],
        [(0, 3, 79), (3, 2, 84), (6, 2, 0), (0, 0, 0), (0, 0, 0)],
        [(0, 1, 79), (1, 1, 81), (2, 2, 84), (4, 2, 81), (6, 2, 0)],
        [(0, 2, 80), (2, 2, 77), (4, 2, 74), (6, 2, 0), (0, 0, 0)],
        [(0, 2, 77), (2, 1, 80), (3, 1, 79), (4, 2, 74), (6, 2, 0)],
        [(0, 1, 84), (1, 1, 86), (2, 1, 84), (3, 2, 81), (6, 2, 0)],
    ];
    // Breakdown: stripped to kick + hat, then a lone pickup announcing the
    // final chorus.
    let breakdown_sax: [[(i64, i64, u8); 5]; 4] = [
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0)],
        [(0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0)],
        [(6, 1, 69), (7, 1, 0), (0, 0, 0), (0, 0, 0), (0, 0, 0)],
    ];
    // Final chorus: one last statement with the flute obligato.
    let outro_sax: [[(i64, i64, u8); 5]; 4] = [
        [(0, 2, 69), (2, 1, 74), (3, 1, 72), (4, 2, 69), (6, 2, 67)],
        [(0, 2, 68), (2, 1, 67), (3, 1, 65), (4, 3, 62), (7, 0, 0)],
        [(0, 2, 65), (2, 2, 68), (4, 2, 74), (6, 2, 0), (0, 0, 0)],
        [(0, 3, 69), (3, 1, 72), (4, 3, 74), (7, 0, 0), (0, 0, 0)],
    ];
    let outro_flute: [[(i64, i64, u8); 5]; 4] = [
        [(0, 1, 79), (1, 1, 81), (2, 1, 84), (3, 2, 81), (6, 2, 0)],
        [(0, 2, 80), (2, 2, 84), (4, 2, 81), (6, 2, 0), (0, 0, 0)],
        [(0, 2, 81), (2, 1, 84), (3, 2, 86), (6, 2, 0), (0, 0, 0)],
        [(0, 3, 84), (3, 1, 86), (4, 3, 84), (7, 0, 0), (0, 0, 0)],
    ];

    let local = (bar % 4) as usize;
    match section(bar) {
        "intro" if bar >= 2 => {
            let row = (bar - 2) as usize;
            phrase_on(&mut p.flute, grid, bar, &intro_sax[row], 88, 0);
            phrase_on(&mut p.sax, grid, bar, &intro_flute[row], 60, -12);
            return;
        }
        "intro" => return,
        "verse" => {
            let row = (bar - 4) as usize;
            phrase_on(&mut p.flute, grid, bar, &verse_sax[row], 76, 0);
            phrase_on(&mut p.sax, grid, bar, &verse_flute[row], 58, -12);
            return;
        }
        "pre" => {
            phrase_on(&mut p.flute, grid, bar, &pre_sax[local], 86, 0);
            phrase_on(&mut p.sax, grid, bar, &pre_flute[local], 62, -12);
            return;
        }
        "chorus" => {
            let row = (bar - 16) as usize;
            phrase_on(&mut p.flute, grid, bar, &chorus_sax[row], 88, 0);
            phrase_on(&mut p.sax, grid, bar, &chorus_flute[row], 58, -12);
            return;
        }
        "breakdown" => {
            phrase_on(&mut p.flute, grid, bar, &breakdown_sax[local], 82, 0);
            return;
        }
        _ => {
            let row = (bar - 28) as usize;
            phrase_on(&mut p.flute, grid, bar, &outro_sax[row], 88, 0);
            phrase_on(&mut p.sax, grid, bar, &outro_flute[row], 60, -12);
            return;
        }
    };
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

    // One grid, one feel: everything swings on the same long-short pulse.
    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = 0.2;

    let mut p = Parts {
        piano: Vec::new(),
        bass: Vec::new(),
        kick: Vec::new(),
        snare: Vec::new(),
        hat_c: Vec::new(),
        hat_o: Vec::new(),
        crash: Vec::new(),
        shake: Vec::new(),
        flute: Vec::new(),
        sax: Vec::new(),
    };

    for bar in 0i64..32 {
        // Intensity by section: intro light, verse mid, pre/chorus full,
        // breakdown stripped back to kick + hat, outro full.
        let intensity = match section(bar) {
            "intro" => 1,
            "verse" => 2,
            "pre" => 3,
            "chorus" => 3,
            "breakdown" => 1,
            _ => 3,
        };
        drums_bar(&grid, bar, &mut p, intensity);
        shaker_bar(&grid, bar, &mut p);

        // Fills dropping each section into the next.
        let fills = [3i64, 11, 15, 23, 27];
        if fills.contains(&bar) {
            fill_bar(&grid, bar, &mut p);
        }

        // Crashes marking the verse, chorus, and outro downbeats.
        if bar == 4 || bar == 16 || bar == 28 {
            note_on_off(&mut p.crash, &grid, bar, 0, 2, 49, 80);
        }

        // Bass: walk-downs on the fresh middle-section bars, sparse in the
        // breakdown, swing bounce elsewhere.
        let walk = (8..=11).contains(&bar) || (20..=21).contains(&bar);
        bass_bar(&grid, bar, &mut p, walk, (24..=27).contains(&bar));

        // Piano: ragtime motif on the intro, swung comp otherwise, quiet in
        // the breakdown.
        match section(bar) {
            "intro" => piano_rag(&grid, bar, &mut p),
            "breakdown" => {}
            _ => piano_comp(&grid, bar, &mut p),
        }

        // Brass stabs (the personality layer).
        brass_section(&grid, bar, &mut p);

        // Lead sax topline + flute counter-melody.
        lead(&grid, bar, &mut p);
    }

    // --- Assemble the song ---
    let mut song = Song::new(BPM as i32, SAMPLE_RATE as i32);

    let instruments: Vec<(&str, Vec<MidiEvent>, (DeviceId, Vec<u8>))> = vec![
        ("piano", p.piano, piano::piano()),
        ("bass", p.bass, bass::bass()),
        ("kick", p.kick, drums::kick()),
        ("snare", p.snare, drums::snare()),
        ("hat_c", p.hat_c, drums::closed_hat()),
        ("hat_o", p.hat_o, drums::open_hat()),
        ("crash", p.crash, drums::crash()),
        ("shake", p.shake, drums::shaker()),
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
            ("drums", &[2, 3, 4, 5, 6, 7]),
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
        let _ = track_names;
    }
}
