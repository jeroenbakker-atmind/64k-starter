//! Generates `src/song.bin` - a 72-bar tune for piano, bass and drums voiced
//! with General MIDI samples through the `Adultery` device.
//!
//! The arrangement is an energy arc (104 bpm, driving funk-jazz). Every
//! section is twice as long as a compact single - each 8-bar phrase is stated
//! twice, with the second pass resolving it:
//!
//! ```text
//! bars 0-3    intro: solo piano, almost nothing
//! bars 4-19   A1: quiet, half-time bass; hats join at bar 8
//! bars 20-35  A2: build - kick joins, groove locks in
//! bar  35     drum break (band out, kit does a fill)
//! bars 36-51  B: full groove, melody an octave up, funk kick
//! bar  43     reduced breath mid-B
//! bar  51     reduced break under a piano fill
//! bars 52-67  A3: full groove again
//! bar  67     drum fill
//! bars 68-71  outro: entrance hit, then a slow solo-piano cadence
//! ```
//!
//! Everything plays with the beat rather than on it: the melody pushes its
//! phrase starts onto the swung and-of-4, the last bar of each phrase drops
//! behind the beat, comping is parked on the offbeats (Brad Mehldau style)
//! with two-note right-hand "answers" after each phrase, and the piano never
//! walks the same idle groove as the bass.
//!
//! Usage: `cargo run --example create_song [--out <path>] [--wav] [--export-dir <path>]`
//!
//! With `--wav`, a mono WAV preview and one stem WAV per instrument group
//! (`<stem>.wav`, `<stem>.piano.wav`, `<stem>.bass.wav` and
//! `<stem>.drums.wav`) are written to the export directory (default
//! `export/`, override with `--export-dir`), rendered with simple
//! sine/saw/box/partial oscillators (see `src-song/render.rs`).
//!
//! The arrangement is composed entirely from the data below; regenerating is
//! deterministic (byte-identical output).

use std::env;
use std::fs;
use std::path::Path;

use starter::format::{Adultery, DeviceId, MidiEvent, Receive, Song, Track, decode, encode};
use starter::music::Grid;

// ---------------------------------------------------------------------------
// CONFIG
// ---------------------------------------------------------------------------
// These are the GM sample indices from the runtime-loaded gm.dls bank
// (`C:\Windows\System32\drivers\gm.dls`). They start at 1.
//
// IMPORTANT: values below are *educated guesses*. On the target Windows
// machine run `cargo run --example inspect_gm_dls` to list the bank and adjust
// these to taste, then re-run create_song.

const PIANO_SAMPLE: f32 = 1.0; // Acoustic Grand Piano layer (guess)
const BASS_SAMPLE: f32 = 45.0; // double/finger bass region (guess)
const KICK_SAMPLE: f32 = 465.0; // percussion: kick (guess)
const SNARE_SAMPLE: f32 = 470.0; // percussion: snare (guess)
const HAT_C_SAMPLE: f32 = 480.0; // percussion: closed hi-hat (guess)
const HAT_O_SAMPLE: f32 = 481.0; // percussion: open hi-hat (guess)

const BPM: f64 = 104.0;
const SAMPLE_RATE: i64 = 44100;
const TAIL_SECS: f64 = 3.0;

// Transposes the entire piano score down one octave (comp, melody, fills,
// outro). The composed register above stays untouched - only the emitted
// midi notes shift.
const PIANO_SHIFT: i32 = -12;

// ---------------------------------------------------------------------------
// Deterministic humanization. A fixed seed keeps regeneration byte-identical:
// every velocity/timing quirk derives from a hash of the note's coordinates
// and feel class, so emission order never matters and results reproduce.
// ---------------------------------------------------------------------------
const HU_SEED: u64 = 0x2F6E_2B1E_57B7_9A44;

// Feel classes: the musical role a note plays. Each gets its own timing bias
// (see felt_eighth below) and a small independent velocity/timing jitter.
const F_PUSH: u64 = 0; // anticipating into a section or change
const F_ON: u64 = 1; // melody on a straight eighth, sitting behind the beat
const F_SWUNG: u64 = 2; // melody on a swung "and"
const F_LAZY: u64 = 3; // phrase-final, laid back
const F_CON: u64 = 4; // comp on-beat
const F_CSWUNG: u64 = 5; // comp swung "and"
const F_ANSWER: u64 = 6; // right-hand pickup answers
const F_OUTRO: u64 = 7; // outro tag
const F_BASS: u64 = 8; // bass (light touch only)

fn hu_final(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Seeded value in [0,1) for the note at (bar, e) playing feel `cls`.
fn hu(bar: i64, e: i64, cls: u64) -> f64 {
    let h = hu_final(
        HU_SEED
            ^ (bar as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (e as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ cls.wrapping_mul(0x94D0_49BB_1331_11EB),
    );
    (h >> 11) as f64 / 9_007_199_254_740_992.0
}

/// Velocity jitter: `±amp` MIDI velocity levels.
fn hu_vel(bar: i64, e: i64, cls: u64, amp: f64) -> i64 {
    ((hu(bar, e, cls) * 2.0 - 1.0) * amp).round() as i64
}

/// Signed timing offset in samples: `bias_ms` plus jitter within `spread_ms`.
fn hu_time(bar: i64, e: i64, cls: u64, bias_ms: f64, spread_ms: f64) -> i64 {
    let ms = bias_ms + (hu(bar, e, cls) * 2.0 - 1.0) * spread_ms;
    (ms * SAMPLE_RATE as f64 / 1000.0).round() as i64
}

/// Varies the amount of swing on a swung "and", relative to the fixed 1/6 the
/// grid already applies; returns a signed offset in samples.
fn hu_swing(grid: &Grid, bar: i64, e: i64, cls: u64) -> i64 {
    let delta = (hu(bar, e, cls) * 2.0 - 1.0) * 0.018; // ~±10ms at 104bpm
    (delta * grid.beat_samples as f64).round() as i64
}

/// Eighth-slot start (bar, e in 0..8) with the feel class's timing applied.
fn felt_eighth(grid: &Grid, bar: i64, e: i64, cls: u64) -> i64 {
    let beat = e / 2;
    let start = grid.eighth(bar, beat, e % 2);
    start
        + (if e % 2 == 1 {
            match cls {
                F_PUSH => hu_swing(grid, bar, e, cls) + hu_time(bar, e, cls, -4.0, 2.0),
                _ => hu_swing(grid, bar, e, cls) + hu_time(bar, e, cls, -1.0, 2.0),
            }
        } else {
            match cls {
                F_PUSH => hu_time(bar, e, cls, -4.0, 2.0),
                F_LAZY => hu_time(bar, e, cls, 7.0, 3.0),
                F_ANSWER => hu_time(bar, e, cls, 2.0, 2.0),
                F_CON => hu_time(bar, e, cls, 1.0, 1.5),
                F_BASS => hu_time(bar, e, cls, 0.0, 1.5),
                _ => hu_time(bar, e, cls, 1.5, 2.0),
            }
        })
        .max(0)
}

// ---------------------------------------------------------------------------
// Chords. Root is the bass note (MIDI); tones is [root, 3rd, 5th, 7th] as
// semitone offsets from the root. C = 36 (C2).
// ---------------------------------------------------------------------------
const MAJ7: [i32; 4] = [0, 4, 7, 11];
const MIN7: [i32; 4] = [0, 3, 7, 10];
const DOM7: [i32; 4] = [0, 4, 7, 10];

const C: (i32, [i32; 4]) = (36, MAJ7);
const AM: (i32, [i32; 4]) = (45, MIN7);
const F: (i32, [i32; 4]) = (41, MAJ7);
const DM: (i32, [i32; 4]) = (38, MIN7);
const G: (i32, [i32; 4]) = (43, DOM7);
const EM: (i32, [i32; 4]) = (40, MIN7);

// 4-bar intro, 4x16-bar AABA, 4-bar outro = 72 bars.
// Each 8-bar phrase is stated twice; its second pass resolves the cadence.
const CHORDS: &[(i32, [i32; 4])] = &[
    C, C, C, C, // intro
    C, AM, F, DM, C, DM, G, G, // A1 (1st statement, half cadence)
    C, AM, F, DM, C, DM, G, C, // A1 (2nd statement, full cadence)
    C, AM, F, DM, C, DM, G, C, // A2 (1st statement)
    C, AM, F, DM, C, DM, G, C, // A2 (2nd statement, into the break)
    F, G, EM, AM, DM, G, C, G, // B (1st statement)
    F, G, EM, AM, DM, G, C, G, // B (2nd statement)
    C, AM, F, DM, C, DM, G, C, // A3 (1st statement)
    C, AM, F, DM, C, DM, G, C, // A3 (2nd statement, into the fill)
    C, C, C, C, // outro
];

// ---------------------------------------------------------------------------
// Melody. `e` is the eighth subdivision within the bar (0..7), `dur` is the
// note length in (straight) eighths.
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct Note {
    e: i64,
    note: u8,
    dur: i64,
}

// ---------------------------------------------------------------------------
// Jazz comping vocabulary (Brad Mehldau flavour).
// A comping pattern is a list of chord hits; each hit is an eighth slot
// (0..7) plus a duration in eighths. `e == 8` means "and-of-4 of the previous
// bar" - i.e. the chord is pushed *into* this bar. Hits on odd eighths land on
// the swung "and", which pulls the piano behind the beat.
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
struct CompHit {
    e: i64,
    dur: i64,
}

const fn h(e: i64, dur: i64) -> CompHit {
    CompHit { e, dur }
}

type CompPat = &'static [CompHit];

// A1 (bars 4-19): airy, phrased on the offbeats - pushes and swung "ands",
// never a steady walk. Each 8-bar statement has its own flavour; the answer
// bars (idx 4 and 12) clear e6/e7 for the right-hand pickup.
const A1P: [CompPat; 16] = [
    &[h(8, 1), h(5, 2)], // push into the section, settle on the "and of 3"
    &[h(1, 1), h(4, 1), h(7, 1)],
    &[h(8, 1), h(2, 1), h(5, 2)],
    &[h(1, 2)], // laid-back phrase-end bar
    &[h(0, 1), h(3, 1)], // clears for the answer
    &[h(8, 1), h(2, 1), h(5, 1)],
    &[h(1, 1), h(4, 1), h(7, 1)],
    &[h(8, 2), h(3, 1)], // half-cadence push
    &[h(8, 1), h(5, 2)], // 2nd statement
    &[h(1, 1), h(4, 1), h(7, 1)],
    &[h(0, 1), h(2, 1), h(5, 2)],
    &[h(1, 2)],
    &[h(8, 1), h(3, 1)], // clears for the answer
    &[h(1, 1), h(4, 1), h(7, 1)],
    &[h(0, 1), h(2, 1), h(5, 1)],
    &[h(8, 2), h(3, 1)], // full-cadence push into A2
];

// A2 (bars 20-35): rhythms heat up, still nothing on the straight quarter grid.
// The last bar (idx 15) rests with the drum break.
const A2P: [CompPat; 16] = [
    &[h(8, 1), h(5, 2), h(7, 1)],
    &[h(1, 1), h(3, 1), h(6, 1)],
    &[h(0, 1), h(2, 1), h(5, 2)],
    &[h(1, 2)], // laid-back phrase-end bar
    &[h(8, 1), h(3, 1)], // leaves room for the right-hand answer
    &[h(1, 1), h(4, 2), h(6, 1)],
    &[h(8, 1), h(2, 1), h(5, 2)],
    &[h(1, 1), h(4, 1), h(7, 1)], // 2nd statement
    &[h(8, 1), h(5, 2), h(7, 1)],
    &[h(1, 1), h(3, 1), h(6, 1)],
    &[h(0, 1), h(2, 1), h(5, 2)],
    &[h(1, 2)],
    &[h(8, 1), h(3, 1)], // clears for the answer
    &[h(1, 1), h(4, 2), h(6, 1)],
    &[h(0, 1), h(3, 1), h(5, 1)], // drive into the break
    &[], // drum break
];

// B (bars 36-51): busiest, "sprinkle" phrases. The lazy bars (idx 3, 11)
// and the fill bar (idx 15) drop out - the octave-up melody leads alone.
const BP: [CompPat; 16] = [
    &[h(0, 1), h(3, 1), h(5, 1), h(7, 1)], // downbeat entrance on the crash
    &[h(1, 1), h(4, 1), h(6, 1)],
    &[h(8, 1), h(2, 2), h(5, 1)],
    &[], // melody only
    &[h(8, 1), h(3, 1)], // clears for the answer
    &[h(1, 1), h(4, 2), h(7, 1)],
    &[h(8, 1), h(2, 2)],
    &[h(1, 1), h(4, 1), h(6, 1)], // 2nd statement
    &[h(0, 1), h(3, 1), h(5, 1)],
    &[h(1, 1), h(4, 1), h(6, 1)],
    &[h(8, 1), h(2, 2), h(5, 1)],
    &[], // melody only
    &[h(8, 1), h(3, 1)], // clears for the answer
    &[h(1, 1), h(4, 2), h(7, 1)],
    &[h(8, 1), h(2, 2)],
    &[], // piano fill bar
];

// A3 (bars 52-67): re-stated with variation, offbeat punches throughout.
const C3P: [CompPat; 16] = [
    &[h(0, 1), h(3, 1), h(5, 1), h(7, 1)], // entrance with the crash
    &[h(1, 1), h(4, 2), h(6, 1)],
    &[h(8, 1), h(2, 1), h(5, 1)],
    &[h(1, 2)], // laid-back phrase-end bar
    &[h(8, 1), h(3, 1)], // clears for the answer
    &[h(1, 1), h(4, 1), h(7, 1)],
    &[h(8, 1), h(2, 2), h(5, 1)],
    &[h(1, 2), h(6, 1)], // 2nd statement
    &[h(8, 1), h(5, 2), h(7, 1)],
    &[h(1, 1), h(3, 1), h(6, 1)],
    &[h(0, 1), h(2, 1), h(5, 2)],
    &[h(1, 2)],
    &[h(8, 1), h(3, 1)], // clears for the answer
    &[h(1, 1), h(4, 2), h(6, 1)],
    &[h(0, 1), h(3, 1), h(5, 1)], // drive into the fill
    &[h(8, 2), h(3, 1)], // push into the outro tag
];

// A section, half-cadence ending (leads back into the repeat / bridge).
const A_HALF: [&[Note]; 8] = [
    &[
        Note { e: 0, note: 64, dur: 1 },
        Note { e: 1, note: 62, dur: 1 },
        Note { e: 2, note: 60, dur: 1 },
        Note { e: 4, note: 64, dur: 3 },
    ], // C
    &[
        Note { e: 0, note: 62, dur: 1 },
        Note { e: 1, note: 60, dur: 1 },
        Note { e: 2, note: 57, dur: 1 },
        Note { e: 4, note: 60, dur: 3 },
    ], // Am
    &[
        Note { e: 0, note: 55, dur: 1 },
        Note { e: 1, note: 57, dur: 1 },
        Note { e: 2, note: 59, dur: 1 },
        Note { e: 4, note: 62, dur: 3 },
    ], // F
    &[
        Note { e: 0, note: 60, dur: 1 },
        Note { e: 1, note: 59, dur: 1 },
        Note { e: 2, note: 57, dur: 1 },
        Note { e: 4, note: 55, dur: 2 },
        Note { e: 6, note: 57, dur: 1 },
    ], // Dm
    &[
        Note { e: 0, note: 55, dur: 1 },
        Note { e: 2, note: 60, dur: 2 },
        Note { e: 4, note: 64, dur: 2 },
    ], // C
    &[
        Note { e: 0, note: 62, dur: 1 },
        Note { e: 2, note: 57, dur: 1 },
        Note { e: 4, note: 53, dur: 2 },
        Note { e: 6, note: 57, dur: 1 },
    ], // Dm
    &[
        Note { e: 0, note: 55, dur: 1 },
        Note { e: 2, note: 59, dur: 1 },
        Note { e: 4, note: 62, dur: 3 },
    ], // G
    &[
        Note { e: 0, note: 60, dur: 1 },
        Note { e: 2, note: 59, dur: 1 },
        Note { e: 4, note: 62, dur: 3 },
    ], // G (half cadence)
];

// A section, full cadence ending (last bar resolves to long C).
const A_FULL: [&[Note]; 8] = [
    A_HALF[0],
    A_HALF[1],
    A_HALF[2],
    A_HALF[3],
    A_HALF[4],
    A_HALF[5],
    A_HALF[6],
    &[
        Note { e: 0, note: 50, dur: 1 },
        Note { e: 2, note: 52, dur: 1 },
        Note { e: 4, note: 60, dur: 4 },
    ], // C cadence
];

// Bridge (8 bars).
const B_SEC: [&[Note]; 8] = [
    &[
        Note { e: 0, note: 57, dur: 1 },
        Note { e: 1, note: 60, dur: 1 },
        Note { e: 2, note: 65, dur: 1 },
        Note { e: 4, note: 64, dur: 2 },
    ], // F
    &[
        Note { e: 0, note: 62, dur: 1 },
        Note { e: 2, note: 65, dur: 1 },
        Note { e: 4, note: 64, dur: 2 },
    ], // G
    &[
        Note { e: 0, note: 64, dur: 1 },
        Note { e: 2, note: 59, dur: 1 },
        Note { e: 4, note: 55, dur: 1 },
        Note { e: 5, note: 59, dur: 1 },
        Note { e: 6, note: 64, dur: 1 },
    ], // Em
    &[
        Note { e: 0, note: 64, dur: 2 },
        Note { e: 2, note: 60, dur: 1 },
        Note { e: 4, note: 57, dur: 2 },
    ], // Am
    &[
        Note { e: 0, note: 65, dur: 1 },
        Note { e: 2, note: 64, dur: 1 },
        Note { e: 4, note: 62, dur: 2 },
    ], // Dm
    &[
        Note { e: 0, note: 62, dur: 1 },
        Note { e: 2, note: 60, dur: 1 },
        Note { e: 4, note: 59, dur: 2 },
    ], // G
    &[
        Note { e: 0, note: 60, dur: 2 },
        Note { e: 2, note: 64, dur: 2 },
    ], // C
    &[
        Note { e: 0, note: 65, dur: 1 },
        Note { e: 2, note: 62, dur: 1 },
        Note { e: 4, note: 59, dur: 3 },
    ], // G (turnaround)
];

// A doubled (16 bars): the 8-bar phrase stated twice - first pass ends on a
// half cadence (G), second pass resolves to the full cadence (C).
const A_DOUBLED: [&[Note]; 16] = [
    A_HALF[0], A_HALF[1], A_HALF[2], A_HALF[3],
    A_HALF[4], A_HALF[5], A_HALF[6], A_HALF[7],
    A_FULL[0], A_FULL[1], A_FULL[2], A_FULL[3],
    A_FULL[4], A_FULL[5], A_FULL[6], A_FULL[7],
];

// B doubled (16 bars): the turnaround is stated twice.
const B_DOUBLED: [&[Note]; 16] = [
    B_SEC[0], B_SEC[1], B_SEC[2], B_SEC[3],
    B_SEC[4], B_SEC[5], B_SEC[6], B_SEC[7],
    B_SEC[0], B_SEC[1], B_SEC[2], B_SEC[3],
    B_SEC[4], B_SEC[5], B_SEC[6], B_SEC[7],
];

// Intro: solo piano colour, no bass/drums. Four bars, slowly rising.
const INTRO_MELODY: [&[Note]; 4] = [
    &[Note { e: 2, note: 76, dur: 8 }], // E5
    &[Note { e: 2, note: 74, dur: 8 }], // D5
    &[Note { e: 2, note: 76, dur: 8 }], // E5
    &[Note { e: 0, note: 79, dur: 8 }], // G5
];

// The B -> A3 piano fill (bar 25): a rising lick over the turnaround G.
const PIANO_FILL: [Note; 8] = [
    Note { e: 0, note: 79, dur: 1 },
    Note { e: 1, note: 81, dur: 1 },
    Note { e: 2, note: 83, dur: 1 },
    Note { e: 3, note: 84, dur: 1 },
    Note { e: 4, note: 86, dur: 1 },
    Note { e: 5, note: 88, dur: 1 },
    Note { e: 6, note: 90, dur: 1 },
    Note { e: 7, note: 91, dur: 1 },
];

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------
struct Parts {
    piano: Vec<MidiEvent>,
    bass: Vec<MidiEvent>,
    kick: Vec<MidiEvent>,
    snare: Vec<MidiEvent>,
    hat_c: Vec<MidiEvent>,
    hat_o: Vec<MidiEvent>,
}

fn comp_chord(root: i32, tones: &[i32; 4]) -> [u8; 3] {
    // Shell voicing: 3rd, 5th, 7th, three octaves above the bass root.
    [
        (root as u8).wrapping_add(36).wrapping_add(tones[1] as u8),
        (root as u8).wrapping_add(36).wrapping_add(tones[2] as u8),
        (root as u8).wrapping_add(36).wrapping_add(tones[3] as u8),
    ]
}

fn comp_chord9(root: i32, tones: &[i32; 4]) -> [u8; 3] {
    // 3rd, 7th, 9th - used for the intro/outro colour chords.
    [
        (root as u8).wrapping_add(36).wrapping_add(tones[1] as u8),
        (root as u8).wrapping_add(36).wrapping_add(tones[3] as u8),
        (root as u8).wrapping_add(36).wrapping_add(14),
    ]
}

fn add_note(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, e: i64, dur_eighths: i64, note: u8, vel: u8) {
    let start = grid.eighth(bar, e / 2, e % 2);
    let off = start + dur_eighths * grid.beat_samples / 2;
    events.push(MidiEvent::on(start, note, vel));
    events.push(MidiEvent::off(off, note));
}

/// Adds a single piano note, transposed by `PIANO_SHIFT` and played with the
/// velocity jitter + timing of feel `cls`.
#[allow(clippy::too_many_arguments)]
fn add_felt_piano_note(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, e: i64, dur_eighths: i64, note: u8, vel: u8, cls: u64) {
    let start = felt_eighth(grid, bar, e, cls);
    let off = start + dur_eighths * grid.beat_samples / 2;
    let v = ((vel as i64 + hu_vel(bar, e, cls, 3.0)).clamp(1, 127)) as u8;
    let note = (note as i32 + PIANO_SHIFT) as u8;
    events.push(MidiEvent::on(start, note, v));
    events.push(MidiEvent::off(off, note));
}

/// Adds a piano chord (struck together) on a beat, transposed by `PIANO_SHIFT`
/// and played with the feel of `cls`.
#[allow(clippy::too_many_arguments)]
fn add_felt_piano_chord(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, beat: i64, dur_eighths: i64, notes: &[u8], vel: u8, cls: u64) {
    let start = (grid.at(bar, beat) + hu_time(bar, beat, cls, 1.0, 1.5)).max(0);
    let off = start + dur_eighths * grid.beat_samples / 2;
    let v = ((vel as i64 + hu_vel(bar, beat, cls, 3.0)).clamp(1, 127)) as u8;
    for n in notes {
        let note = (*n as i32 + PIANO_SHIFT) as u8;
        events.push(MidiEvent::on(start, note, v));
        events.push(MidiEvent::off(off, note));
    }
}

/// Adds a bass note with a light humanization (the bass locks the groove, so
/// the touch is much gentler than the piano's).
fn add_felt_bass_note(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, e: i64, dur_eighths: i64, note: u8, vel: u8) {
    let start = felt_eighth(grid, bar, e, F_BASS);
    let off = start + dur_eighths * grid.beat_samples / 2;
    let v = ((vel as i64 + hu_vel(bar, e, F_BASS, 2.0)).clamp(1, 127)) as u8;
    events.push(MidiEvent::on(start, note, v));
    events.push(MidiEvent::off(off, note));
}

/// One shuffled-in comping chord. `hit.e == 8` means the chord is pushed onto
/// the swung and-of-4 of the previous bar (anticipation into `bar`).
fn comp_hit(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, hit: CompHit, notes: &[u8]) {
    let (abar, e) = if hit.e == 8 { (bar - 1, 7) } else { (bar, hit.e) };
    let cls = if hit.e == 8 { F_PUSH } else if hit.e % 2 == 1 { F_CSWUNG } else { F_CON };
    // pushed hits lead in a little louder, swung "ands" ghost-soft; the two
    // statements of each section answer each other (call/response dynamics).
    let sect = match bar {
        4..=19 => 4,
        20..=35 => 20,
        36..=51 => 36,
        _ => 52,
    };
    let stmt_swing = if (bar - sect) / 8 % 2 == 0 { 3 } else { -2 };
    let base = if hit.e == 8 { 74 } else if hit.e % 2 == 1 { 54 } else { 64 };
    let start = felt_eighth(grid, abar, e, cls);
    let off = start + hit.dur * grid.beat_samples / 2;
    let v = ((base + stmt_swing + hu_vel(bar, e, cls, 2.0)).clamp(1, 127)) as u8;
    for n in notes {
        let note = (*n as i32 + PIANO_SHIFT) as u8;
        events.push(MidiEvent::on(start, note, v));
        events.push(MidiEvent::off(off, note));
    }
}

fn main() {
    let grid = Grid::new(BPM, SAMPLE_RATE);

    let piano = Vec::new();
    let bass = Vec::new();
    let kick = Vec::new();
    let snare = Vec::new();
    let hat_c = Vec::new();
    let hat_o = Vec::new();

    let mut p = Parts {
        piano,
        bass,
        kick,
        snare,
        hat_c,
        hat_o,
    };

    const BODY_START: i64 = 4;
    const BODY_END: i64 = 68;

    // --- Piano: intro (bars 0-3): a colour chord under a slow rising motif ---
    for (bi, notes) in INTRO_MELODY.iter().enumerate() {
        let bar = bi as i64;
        let chord = comp_chord9(CHORDS[bar as usize].0, &CHORDS[bar as usize].1);
        add_felt_piano_chord(&mut p.piano, &grid, bar, 0, 8, &chord, 56, F_CON);
        for n in *notes {
            let cls = if n.e % 2 == 1 { F_SWUNG } else { F_ON };
            add_felt_piano_note(&mut p.piano, &grid, bar, n.e, n.dur, n.note, 72, cls);
        }
    }

    // --- Piano: melody, one doubled phrase per section. B sings an octave
    //     above the A sections; the break bars (35, 51, 67) let it breathe. ---
    let mut melody_map: Vec<(&[&[Note]; 16], i64, u8, i32)> = Vec::new();
    melody_map.push((&A_DOUBLED, 4, 82, 12)); // A1: quiet, upper register
    melody_map.push((&A_DOUBLED, 20, 92, 12)); // A2
    melody_map.push((&B_DOUBLED, 36, 100, 24)); // B: brighter, an octave above A
    melody_map.push((&A_DOUBLED, 52, 98, 12)); // A3

    for (section, offset, vel, shift) in &melody_map {
        for (bi, notes) in section.iter().enumerate() {
            let bar = offset + bi as i64;
            let stmt = bi / 8;
            let bi8 = bi % 8;
            // bar 35: full drum break (piano rests). bar 51: the piano fill
            // plays instead. bar 67: drum fill (piano rests).
            if (offset == &20 && bi == 15) || (offset == &36 && bi == 15) || (offset == &52 && bi == 15) {
                continue;
            }
            // Energy arc across each 8-bar statement, so the second statement
            // of a section starts a touch stronger now the first has set it up.
            let arc: [i32; 8] = if stmt == 0 {
                [0, 1, 2, 3, 2, 1, 0, -2]
            } else {
                [2, 3, 4, 4, 3, 2, 1, 0]
            };
            let lazy = bi8 == 3;
            for (ni, n) in notes.iter().enumerate() {
                let note = n.note.wrapping_add(*shift as u8);
                // Phrase contour: accent the peak (and the beat-1 pickup), soften
                // the tail note that rolls off the phrase.
                let peak = notes.iter().map(|x| x.note).max().unwrap();
                let contour = if n.note == peak && ni > 0 {
                    4
                } else if ni == 0 {
                    3
                } else if ni == notes.len() - 1 {
                    -3
                } else {
                    0
                };
                // Anti-metronomic phrasing: statement 1 pushes its opening
                // downbeat onto the swung and-of-4; statement 2 leans the same
                // phrase in on the nose, a beat later.
                let pushed = bi8 == 0 && stmt == 0 && ni == 0;
                let (ebar, e, cls) = if pushed {
                    (bar - 1, 7, F_PUSH)
                } else if lazy {
                    (bar, n.e + 1, F_LAZY)
                } else if n.e % 2 == 1 {
                    (bar, n.e, F_SWUNG)
                } else {
                    (bar, n.e, F_ON)
                };
                let v = ((*vel as i64 + arc[bi8] as i64 + contour as i64 + hu_vel(ebar, e, cls, 3.0)).clamp(1, 127)) as u8;
                add_felt_piano_note(&mut p.piano, &grid, ebar, e, n.dur, note, v, cls);
            }
        }
    }

    // --- Piano: right-hand "answers". Mehldau answers a finished phrase with a
    //     little two-note pickup in the last two eighths of the bar, a register
    //     away from the left-hand shells - once per 8-bar statement. ---
    for (bar, lick) in [
        (8, [73, 74]),
        (16, [73, 74]),
        (24, [73, 74]),
        (32, [73, 74]),
        (40, [76, 74]),
        (48, [76, 74]),
        (56, [73, 74]),
        (64, [73, 74]),
    ] {
        for (k, note) in lick.iter().enumerate() {
            // Appoggiatura shape: the pickup note leans in soft, the resolving
            // one lands accented.
            let v = (62 + if k == 0 { -5 } else { 5 } + hu_vel(bar, 6 + k as i64, F_ANSWER, 2.0)).clamp(1, 127) as u8;
            add_felt_piano_note(&mut p.piano, &grid, bar, 6 + k as i64, 1, *note as u8, v, F_ANSWER);
        }
    }

    // --- Piano: the bar-51 fill, carrying the B -> A3 break ---
    for n in PIANO_FILL {
        let cls = if n.e % 2 == 1 { F_SWUNG } else { F_ON };
        add_felt_piano_note(&mut p.piano, &grid, 51, n.e, n.dur, n.note, 86, cls);
    }

    // --- Piano: comping, Brad Mehldau style. Syncopated, pushed and phrase-based,
    //     never locked to beats 1 & 3. `e == 8` hits anticipate into the bar. ---
    for bar in BODY_START..BODY_END {
        let (root, tones) = CHORDS[bar as usize];
        let shell = comp_chord(root, &tones);
        let pat: CompPat = match bar {
            4..=19 => A1P[(bar - 4) as usize],
            20..=35 => A2P[(bar - 20) as usize],
            36..=51 => BP[(bar - 36) as usize],
            _ => C3P[(bar - 52) as usize],
        };
        for hit in pat {
            // the B section tosses 9ths on the top off-beats for a spicier chord
            let notes: &[u8] = if (36..=51).contains(&bar) && hit.e >= 6 {
                &comp_chord9(root, &tones)
            } else {
                &shell
            };
            comp_hit(&mut p.piano, &grid, bar, *hit, notes);
        }
    }

    // --- Piano: outro tag (bars 68-71): a slow solo cadence resolving to a
    //     wide final C(add9). ---
    {
        let chord9 = comp_chord9(36, &MAJ7);
        add_felt_piano_chord(&mut p.piano, &grid, 68, 0, 8, &chord9, 50, F_OUTRO);
        add_felt_piano_note(&mut p.piano, &grid, 68, 2, 4, 84, 72, F_OUTRO); // C5
        add_felt_piano_note(&mut p.piano, &grid, 68, 6, 1, 86, 74, F_OUTRO); // D5
        add_felt_piano_chord(&mut p.piano, &grid, 69, 0, 8, &chord9, 50, F_OUTRO);
        add_felt_piano_note(&mut p.piano, &grid, 69, 2, 4, 88, 72, F_OUTRO); // E5
        add_felt_piano_chord(&mut p.piano, &grid, 70, 0, 8, &chord9, 50, F_OUTRO);
        add_felt_piano_note(&mut p.piano, &grid, 70, 2, 4, 91, 72, F_OUTRO); // G5
        add_felt_piano_chord(&mut p.piano, &grid, 71, 0, 8, &[72, 76, 79, 84, 88, 96], 74, F_OUTRO);
    }

    // --- Bass ---
    for bar in BODY_START..BODY_END {
        let (root, tones) = CHORDS[bar as usize];
        let _ = tones;
        match bar {
            35 | 51 => {} // drop out with the breaks
            // A1 (1st statement): half-time roots keep the start calm
            4..=11 => {
                add_felt_bass_note(&mut p.bass, &grid, bar, 0, 2, root as u8, 64);
                add_felt_bass_note(&mut p.bass, &grid, bar, 4, 2, (root + 7) as u8, 58);
            }
            // A1 (2nd statement): walking begins, still feathering in
            12..=19 => {
                let next_root = CHORDS[bar as usize + 1].0;
                walk_bar(&mut p.bass, &grid, bar, root, next_root, 74);
            }
            // everything from A2 on: walking quarters
            _ => {
                let next_root = CHORDS[bar as usize + 1].0;
                walk_bar(&mut p.bass, &grid, bar, root, next_root, 86);
            }
        }
    }
    // ...and a deep C under the final chord.
    add_felt_bass_note(&mut p.bass, &grid, 71, 0, 8, 24, 58);

    // --- Drums: an energy arc across the doubled sections, with breaks and a
    //     fill into the tag ---
    for bar in 0..72 {
        let level = match bar {
            0..=7 => Drums::Off,       // intro + A1 first statement: none
            8..=15 => Drums::Hat,      // A1: hats only
            16..=19 => Drums::KickHat, // A1 tail: kick joins
            20..=27 => Drums::KickHat, // A2 first statement
            28..=34 => Drums::Full,    // A2 lock-in
            35 => Drums::Break,        // drum break into B
            36..=42 => Drums::Full,    // B: the groove in full
            43 => Drums::Sparse,       // reduced breath mid-B
            44..=50 => Drums::Full,    // B second statement
            51 => Drums::Sparse,       // reduced break under the piano fill
            52..=66 => Drums::Full,    // A3: drive home
            67 => Drums::Fill,         // fill into the outro
            _ => Drums::Off,           // outro
        };
        let accent = matches!(bar, 15 | 19 | 27 | 34 | 43 | 50 | 66);
        drum_bar(&grid, bar, &mut p, level, accent);
    }
    // Entrance hit announcing the tag, and a closing hit under the final chord.
    add_note(&mut p.kick, &grid, 68, 0, 1, 36, 110);
    add_note(&mut p.snare, &grid, 68, 0, 1, 38, 96);
    add_note(&mut p.hat_o, &grid, 68, 0, 1, 46, 80);
    add_note(&mut p.kick, &grid, 71, 0, 1, 36, 108);
    add_note(&mut p.hat_o, &grid, 71, 0, 2, 46, 74);

    // --- Assemble the song ---
    let mut song = Song::new(BPM as i32, SAMPLE_RATE as i32);

    let piano_dev = {
        let mut a = Adultery::default();
        a.sample_index = PIANO_SAMPLE;
        a.amp_attack = Adultery::env_ms(2.0);
        a.amp_decay = Adultery::env_ms(700.0);
        a.amp_sustain = 0.9;
        a.amp_release = Adultery::env_ms(2200.0);
        a.loop_mode = 1.0; // Repeat (DLS loop points)
        a.master = 0.75;
        (DeviceId::Adultery, a.chunk())
    };
    let bass_dev = {
        let mut a = Adultery::default();
        a.sample_index = BASS_SAMPLE;
        a.amp_attack = Adultery::env_ms(3.0);
        a.amp_decay = Adultery::env_ms(450.0);
        a.amp_sustain = 0.8;
        a.amp_release = Adultery::env_ms(500.0);
        a.loop_mode = 1.0;
        a.master = 0.55;
        (DeviceId::Adultery, a.chunk())
    };
    let kick_dev = {
        let mut a = Adultery::default();
        a.sample_index = KICK_SAMPLE;
        a.amp_attack = Adultery::env_ms(1.0);
        a.amp_decay = Adultery::env_ms(50.0);
        a.amp_sustain = 0.8;
        a.amp_release = Adultery::env_ms(150.0);
        a.loop_mode = 0.0; // Disabled
        a.master = 0.5;
        (DeviceId::Adultery, a.chunk())
    };
    let snare_dev = {
        let mut a = Adultery::default();
        a.sample_index = SNARE_SAMPLE;
        a.amp_attack = Adultery::env_ms(1.0);
        a.amp_decay = Adultery::env_ms(60.0);
        a.amp_sustain = 0.7;
        a.amp_release = Adultery::env_ms(120.0);
        a.loop_mode = 0.0;
        a.master = 0.45;
        (DeviceId::Adultery, a.chunk())
    };
    let hat_c_dev = {
        let mut a = Adultery::default();
        a.sample_index = HAT_C_SAMPLE;
        a.amp_attack = Adultery::env_ms(1.0);
        a.amp_decay = Adultery::env_ms(30.0);
        a.amp_sustain = 0.4;
        a.amp_release = Adultery::env_ms(80.0);
        a.loop_mode = 0.0;
        a.master = 0.38;
        (DeviceId::Adultery, a.chunk())
    };
    let hat_o_dev = {
        let mut a = Adultery::default();
        a.sample_index = HAT_O_SAMPLE;
        a.amp_attack = Adultery::env_ms(1.0);
        a.amp_decay = Adultery::env_ms(250.0);
        a.amp_sustain = 0.4;
        a.amp_release = Adultery::env_ms(150.0);
        a.loop_mode = 0.0;
        a.master = 0.34;
        (DeviceId::Adultery, a.chunk())
    };

    let mut track_piano = Track::new(1.0);
    track_piano.devices.push(piano_dev);
    track_piano.events = p.piano;

    let mut track_bass = Track::new(1.0);
    track_bass.devices.push(bass_dev);
    track_bass.events = p.bass;

    let mut track_kick = Track::new(1.0);
    track_kick.devices.push(kick_dev);
    track_kick.events = p.kick;

    let mut track_snare = Track::new(1.0);
    track_snare.devices.push(snare_dev);
    track_snare.events = p.snare;

    let mut track_hat_c = Track::new(1.0);
    track_hat_c.devices.push(hat_c_dev);
    track_hat_c.events = p.hat_c;

    let mut track_hat_o = Track::new(1.0);
    track_hat_o.devices.push(hat_o_dev);
    track_hat_o.events = p.hat_o;

    // Master: sums everything, no devices.
    let mut track_master = Track::new(1.0);
    for send in 0..6 {
        track_master
            .receives
            .push(Receive::new(send as i32, 0, 1.0));
    }

    song.tracks = vec![
        track_piano,
        track_bass,
        track_kick,
        track_snare,
        track_hat_c,
        track_hat_o,
        track_master,
    ];

    // Duration: end of the last event plus a short tail.
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

    // Parse CLI: --out <path> (default src/song.bin), --wav, --export-dir <path>
    let default_out = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/song.bin")
        .to_string_lossy()
        .into_owned();
    let mut out_path = default_out.clone();
    let default_export = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("export")
        .to_string_lossy()
        .into_owned();
    let mut export_dir = default_export.clone();
    let mut want_wav = false;

    let mut args = env::args().skip(1).peekable();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out_path = args.next().expect("--out needs a path"),
            "--wav" => want_wav = true,
            "--export-dir" => export_dir = args.next().expect("--export-dir needs a path"),
            other => {
                eprintln!("unknown argument: {other}");
                eprintln!("usage: cargo run --example create_song [--out <path>] [--wav] [--export-dir <path>]");
                std::process::exit(2);
            }
        }
    }

    fs::write(&out_path, &data).expect("failed to write song.bin");

    // Optional cross-platform WAV preview + one WAV per instrument, written to
    // the export directory (never mixed into src/).
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

        // Stems: piano, bass and one combined "drums" kit.
        let stems: &[(&str, &[usize])] = &[
            ("piano", &[0]),
            ("bass", &[1]),
            ("drums", &[2, 3, 4, 5]),
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
        "tempo {} bpm | length {:.2}s | {} bars",
        BPM as i32,
        song.length,
        CHORDS.len()
    );
    let names = ["piano", "bass", "kick", "snare", "hat_c", "hat_o"];
    for (i, t) in song.tracks[..6].iter().enumerate() {
        println!("  track {i}: {:<8} {} notes", names[i], t.events.iter().filter(|e| e.on).count());
    }

    // Warning about the GM sample indices
    println!("\nNOTE: the Adultery sample indices in 'CONFIG' above are guesses.");
    println!("On the target Windows machine run:");
    println!("    cargo run --example inspect_gm_dls");
    println!("then update PIANO_SAMPLE/BASS_SAMPLE/KICK_SAMPLE/... and re-run this example.");
}

/// How much of the kit plays in a given bar.
#[derive(Clone, Copy)]
enum Drums {
    Off,
    Hat,
    KickHat,
    Sparse,
    Full,
    Break,
    Fill,
}

/// Places one bar of drums. `accent` opens the hi-hat at the end of the bar.
fn drum_bar(grid: &Grid, bar: i64, p: &mut Parts, level: Drums, accent: bool) {
    const HAT: u8 = 42;
    const KICK: u8 = 36;
    const SNARE: u8 = 38;

    let hats = |c: &mut Vec<MidiEvent>, on: u8, off: u8| {
        for e in 0..8 {
            let vel = if e % 2 == 0 { on } else { off };
            add_note(c, grid, bar, e, 1, HAT, vel);
        }
    };

    match level {
        Drums::Off => {}
        Drums::Hat => hats(&mut p.hat_c, 84, 48),
        Drums::Sparse => {
            hats(&mut p.hat_c, 74, 44);
            add_note(&mut p.snare, grid, bar, 2, 1, SNARE, 78);
            add_note(&mut p.snare, grid, bar, 6, 1, SNARE, 86);
        }
        Drums::KickHat => {
            hats(&mut p.hat_c, 92, 52);
            // building kick: 1, "and of 2", 3, "and of 4"
            add_note(&mut p.kick, grid, bar, 0, 1, KICK, 104);
            add_note(&mut p.kick, grid, bar, 5, 1, KICK, 72);
            add_note(&mut p.kick, grid, bar, 4, 1, KICK, 100);
            add_note(&mut p.kick, grid, bar, 7, 1, KICK, 58);
        }
        Drums::Full => {
            hats(&mut p.hat_c, 98, 58);
            // funk kick: 1, "and of 2", 3, plus a pickup on "and of 4"
            add_note(&mut p.kick, grid, bar, 0, 1, KICK, 108);
            add_note(&mut p.kick, grid, bar, 5, 1, KICK, 76);
            add_note(&mut p.kick, grid, bar, 4, 1, KICK, 100);
            add_note(&mut p.kick, grid, bar, 7, 1, KICK, 62);
            // snare: ghost on "and of 2", backbeat on 2 & 4
            add_note(&mut p.snare, grid, bar, 3, 1, SNARE, 60);
            add_note(&mut p.snare, grid, bar, 2, 1, SNARE, 86);
            add_note(&mut p.snare, grid, bar, 6, 1, SNARE, 98);
            // crash cymbal into the B and A3 downbeats
            if bar == 18 || bar == 26 {
                add_note(&mut p.hat_o, grid, bar, 0, 2, 46, 80);
            }
        }
        Drums::Break => {
            hats(&mut p.hat_c, 92, 54);
            add_note(&mut p.hat_o, grid, bar, 0, 1, 46, 70); // crash in
            add_note(&mut p.kick, grid, bar, 0, 1, KICK, 102);
            add_note(&mut p.kick, grid, bar, 4, 1, KICK, 102);
            add_note(&mut p.snare, grid, bar, 2, 1, SNARE, 82);
            // snare roll through beat 4
            for e in 5..8 {
                add_note(&mut p.snare, grid, bar, e, 1, SNARE, (84 + 4 * (e - 5)) as u8);
            }
        }
        Drums::Fill => {
            hats(&mut p.hat_c, 94, 56);
            add_note(&mut p.kick, grid, bar, 0, 1, KICK, 104);
            add_note(&mut p.kick, grid, bar, 4, 1, KICK, 96);
            add_note(&mut p.snare, grid, bar, 2, 1, SNARE, 82);
            // roll into the tag
            for e in 5..8 {
                add_note(&mut p.snare, grid, bar, e, 1, SNARE, (86 + 3 * (e - 5)) as u8);
            }
        }
    }

    if accent {
        add_note(&mut p.hat_o, grid, bar, 7, 2, 46, 56);
    }
}

/// Composes four walking-bass quarters for one bar.
fn walk_bar(events: &mut Vec<MidiEvent>, grid: &Grid, bar: i64, root: i32, next_root: i32, vel: u8) {
    let third = root + if is_minor_third(root) { 3 } else { 4 };
    let approach = if next_root > root {
        next_root - 1
    } else if next_root < root {
        next_root + 1
    } else {
        root - 1
    };
    // Middle note: a fifth above the root, or a stepwise "connector" towards
    // the chromatic approach tone if that keeps the line smoother.
    let mid = if (approach - third).abs() <= 4 {
        root + 7
    } else if approach > third {
        approach - 2
    } else {
        approach + 2
    };
    let notes = [root, third, mid, approach];
    for (i, n) in notes.iter().enumerate() {
        // Walking bounce: downbeat root lifts, the chromatic approach lands a
        // touch louder, the third sits back - plus light velocity/timing jitter.
        let contour = match i {
            0 => 3,
            1 => -2,
            3 => 1,
            _ => 0,
        };
        let start = (grid.at(bar, i as i64) + hu_time(bar, i as i64, F_BASS, 0.0, 1.5)).max(0);
        let off = grid.at(bar, i as i64 + 1);
        let v = ((vel as i64 + contour + hu_vel(bar, i as i64, F_BASS, 2.0)).clamp(1, 127)) as u8;
        events.push(MidiEvent::on(start, *n as u8, v));
        events.push(MidiEvent::off(off, *n as u8));
    }
}

/// Crude check for the minor-third quality based on the root pitch class.
fn is_minor_third(root: i32) -> bool {
    // m7 chords used here: A (45), D (38), E (40) -> pitch classes 9, 2, 4.
    matches!(root % 12, 2 | 4 | 9)
}