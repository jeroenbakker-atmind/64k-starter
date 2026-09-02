//! Song composition — the train-to-tea blend.
//!
//! Exposes the composition as a library function so both `examples/create_song`
//! and `bin-starter/build.rs` can generate the song without a nested cargo run.

use crate::instruments::{bass, drums, flute, piano, saxophone};
use crate::swingkit::*;

const BPM: f64 = 104.0;
const SWING: f64 = 0.3;
const TAIL_SECS: f64 = 2.5;

// ---------------------------------------------------------------------------
// Section boundaries (32 bars total).
//   0..=3   intro:   solo piano, sparse train harmony
//   4..=7   train:   bass chug, kick 1&3, sax calls
//   8..=11  train B: flute answers, piano colour joins
//   12..=15 trans:   drums shift, oom-pah bass, FM7 introduced
//   16..=19 tea A:   piano arpeggios, oom-pah, flute obligato
//   20..=23 tea B:   sax swells, peak arc
//   24..=27 outro:   train callback, instruments drop, final chord
// ---------------------------------------------------------------------------
const TRAIN_CHORDS: [Chord; 3] = [DM7, A7, GM7]; // bars 0-11 (with A7 pivot)
const TRANS_CHORDS: [Chord; 4] = [DM7, GM7, FM7, A7]; // bars 12-15
const TEA_CHORDS: [Chord; 12] = [
    DM7, GM7, DM7, GM7, // tea A (16-19)
    FM7, GM7, A7, DM7, // tea B (20-23)
    DM7, GM7, FM7, A7,
];
const OUTRO_CHORDS: [Chord; 4] = [DM7, FM7, GM7, DM7]; // bars 24-27

fn chord_at(bar: i64) -> Chord {
    match bar {
        0..=11 => TRAIN_CHORDS[bar as usize % 3],
        12..=15 => TRANS_CHORDS[(bar - 12) as usize],
        16..=23 => TEA_CHORDS[(bar - 16) as usize],
        24..=27 => OUTRO_CHORDS[(bar - 24) as usize],
        _ => DM7,
    }
}

// ---------------------------------------------------------------------------
// Train (bars 4-7) + train B (bars 8-11)
// ---------------------------------------------------------------------------

fn train_groove(grid: &Grid, bar: i64, p: &mut Parts) {
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 96);
    on_off(&mut p.kick, grid, bar, 4, 1, KICK, 88);
    for e in 0..8 {
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, if e % 4 == 2 { 52 } else { 44 });
    }
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 54);
}

fn train_bass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    for beat in 0..4 {
        on_off_h(&mut p.bass, grid, bar, beat * 2, 1, root as u8, 62);
        on_off_h(&mut p.bass, grid, bar, beat * 2 + 1, 1, (root + 7) as u8, 56);
    }
}

fn train_sax(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    if bar % 4 == 0 {
        on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 68);
        on_off_h(&mut p.sax, grid, bar, 4, 1, (root + 12) as u8, 64);
    } else if bar % 4 == 2 {
        on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 66);
    }
}

fn train_b_flute(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    if bar % 4 == 1 || bar % 4 == 3 {
        on_off_h(&mut p.flute, grid, bar, 2, 2, (root + 9) as u8, 66);
        on_off_h(&mut p.flute, grid, bar, 6, 2, (root + 7) as u8, 64);
    }
}

fn train_b_piano(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = chord_at(bar);
    let shell = chord_notes(root, &tones, 1);
    if bar % 2 == 0 {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, 3, 1, n, 48);
        }
    }
}

// ---------------------------------------------------------------------------
// Transition (bars 12-15)
// ---------------------------------------------------------------------------

fn trans_groove(grid: &Grid, bar: i64, p: &mut Parts) {
    match bar - 12 {
        0 => {
            on_off(&mut p.kick, grid, bar, 0, 1, KICK, 92);
            on_off(&mut p.kick, grid, bar, 4, 1, KICK, 80);
            for e in 0..8 {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, if e % 4 == 2 { 50 } else { 42 });
            }
            on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 50);
        }
        1 => {
            on_off(&mut p.kick, grid, bar, 0, 1, KICK, 88);
            for e in 0..8 {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, if e % 2 == 1 { 48 } else { 42 });
            }
            on_off(&mut p.snare, grid, bar, 3, 1, SNARE, 44);
            on_off(&mut p.snare, grid, bar, 7, 1, SNARE, 40);
        }
        2 => {
            on_off(&mut p.kick, grid, bar, 0, 1, KICK, 84);
            for e in [1i64, 3, 5, 7] {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, 46);
            }
            on_off(&mut p.snare, grid, bar, 3, 1, SNARE, 42);
            on_off(&mut p.snare, grid, bar, 7, 1, SNARE, 38);
        }
        _ => {
            on_off(&mut p.kick, grid, bar, 0, 1, KICK, 82);
            for e in [1i64, 3, 5, 7] {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, 48);
            }
            on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 42);
            on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 40);
        }
    }
}

fn trans_bass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    match bar - 12 {
        0 => {
            for beat in 0..4 {
                on_off_h(&mut p.bass, grid, bar, beat * 2, 1, root as u8, 58);
                on_off_h(&mut p.bass, grid, bar, beat * 2 + 1, 1, (root + 7) as u8, 52);
            }
        }
        1 => {
            spread(&mut p.bass, grid, bar, &[(0, 4, root as u8)], 64);
        }
        _ => {
            spread(
                &mut p.bass,
                grid,
                bar,
                &[
                    (0, 2, root as u8),
                    (4, 2, root as u8),
                    (6, 1, (root + 7) as u8),
                ],
                70,
            );
        }
    }
}

fn trans_piano(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = chord_at(bar);
    match bar - 12 {
        0 => {
            let shell = chord_notes(root, &tones, 1);
            for &n in &shell {
                on_off_h(&mut p.piano, grid, bar, 3, 1, n, 50);
            }
        }
        1 => {
            let shell = chord_notes(root, &tones, 1);
            for (i, &n) in shell.iter().enumerate() {
                on_off_h(&mut p.piano, grid, bar, 1 + i as i64 * 2, 1, n, 48);
            }
        }
        _ => {
            let shell = chord_notes(root, &tones, 1);
            let row: [(i64, i64, u8); 4] = [
                (0, 1, root as u8 + 12),
                (2, 1, shell[2]),
                (4, 1, shell[1]),
                (6, 1, shell[0]),
            ];
            for &(e, dur, n) in &row {
                on_off_h(&mut p.piano, grid, bar, e, dur, n, 52);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tea (bars 16-23) — A dance, B peak
// ---------------------------------------------------------------------------

fn tea_groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = (bar - 16) / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 82);
    if sec == 1 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 68);
    }
    on_off(&mut p.snare, grid, bar, 3, 1, SNARE, 42);
    on_off(&mut p.snare, grid, bar, 7, 1, SNARE, 38);
    for e in [1i64, 3, 5, 7] {
        let vel = if e == 3 { 52 } else { 46 };
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, vel);
    }
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 42);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 40);
}

fn tea_bass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    spread(
        &mut p.bass,
        grid,
        bar,
        &[
            (0, 2, root as u8),
            (4, 2, root as u8),
            (6, 1, (root + 7) as u8),
        ],
        72,
    );
}

fn tea_arpeggio(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = chord_at(bar);
    let shell = chord_notes(root, &tones, 1);
    let sec = (bar - 16) / 4;
    if sec == 0 {
        for e in [2i64, 6] {
            for &n in &shell {
                on_off_h(&mut p.piano, grid, bar, e, 1, n, 46);
            }
        }
    } else {
        let row: [(i64, i64, u8); 6] = [
            (0, 1, root as u8 + 12),
            (1, 1, shell[2]),
            (2, 1, shell[1]),
            (3, 1, shell[0]),
            (4, 2, root as u8 + 12),
            (6, 1, shell[2]),
        ];
        for &(e, dur, n) in &row {
            on_off_h(&mut p.piano, grid, bar, e, dur, n, 52);
        }
    }
}

fn tea_flute(grid: &Grid, bar: i64, p: &mut Parts) {
    let vel = if bar >= 20 { 76 } else { 70 };
    let row: &[(i64, i64, u8)] = match bar {
        16 => &[(0, 2, 62), (2, 1, 65), (3, 1, 67), (4, 2, 69)],
        17 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        18 => &[(0, 2, 69), (2, 1, 72), (3, 2, 74), (6, 1, 72), (7, 1, 69)],
        19 => &[(0, 2, 74), (2, 1, 72), (3, 1, 69), (4, 2, 67)],
        20 => &[(0, 2, 62), (2, 1, 65), (3, 1, 67), (4, 2, 69), (6, 2, 72)],
        21 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        22 => &[(0, 2, 69), (2, 1, 72), (3, 1, 74), (4, 2, 69), (6, 2, 67)],
        23 => &[(0, 2, 74), (2, 1, 72), (3, 2, 69), (6, 2, 65)],
        _ => &[(0, 3, 62)],
    };
    spread(&mut p.flute, grid, bar, row, vel);
}

fn tea_swell(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    match bar {
        17 | 19 => {
            on_off_h(&mut p.sax, grid, bar, 0, 2, (root + 12) as u8, 62);
        }
        20 => {
            on_off_h(&mut p.sax, grid, bar, 0, 2, (root + 12) as u8, 70);
            let (_, tones) = chord_at(bar);
            on_off_h(&mut p.sax, grid, bar, 4, 1, chord_notes(root, &tones, 0)[1], 68);
        }
        21 | 22 => {
            on_off_h(&mut p.sax, grid, bar, 0, 2, (root + 12) as u8, 72);
            if bar == 22 {
                on_off_h(&mut p.sax, grid, bar, 6, 2, (root + 24) as u8, 66);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Outro (bars 24-27)
// ---------------------------------------------------------------------------

fn outro_groove(grid: &Grid, bar: i64, p: &mut Parts) {
    match bar - 24 {
        0 => {
            on_off(&mut p.kick, grid, bar, 0, 1, KICK, 90);
            on_off(&mut p.kick, grid, bar, 4, 1, KICK, 82);
            for e in 0..8 {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, if e % 4 == 2 { 48 } else { 40 });
            }
            on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 48);
        }
        1 => {
            on_off(&mut p.kick, grid, bar, 0, 1, KICK, 78);
            for e in [1i64, 3, 5, 7] {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, 42);
            }
        }
        _ => {
            for e in [1i64, 5] {
                on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, 36);
            }
        }
    }
}

fn outro_bass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = chord_at(bar);
    match bar - 24 {
        0 => {
            for beat in 0..4 {
                on_off_h(&mut p.bass, grid, bar, beat * 2, 1, root as u8, 56);
            }
        }
        _ => {
            spread(&mut p.bass, grid, bar, &[(0, 4, root as u8)], 50);
        }
    }
}

fn outro_piano(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = chord_at(bar);
    match bar - 24 {
        0 => {
            let shell = chord_notes(root, &tones, 1);
            for (i, &n) in shell.iter().enumerate() {
                on_off_h(&mut p.piano, grid, bar, 1 + i as i64 * 2, 1, n, 44);
            }
        }
        1 => {
            let shell = chord_notes(root, &tones, 1);
            for &n in &shell {
                on_off_h(&mut p.piano, grid, bar, 0, 6, n, 42);
            }
        }
        2 => {
            let shell = chord_notes(root, &tones, 1);
            for &n in &shell {
                on_off_h(&mut p.piano, grid, bar, 0, 7, n, 40);
            }
            on_off_h(&mut p.piano, grid, bar, 2, 5, 74, 58);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compose the train-to-tea song.
///
/// Returns `(track_names, encoded_bytes)`, where `track_names` is the track
/// list in `Placed` order (index = track index, minus the final master track).
/// `bin-starter` build.rs calls `encode` itself on the same bytes.
pub fn compose_placed() -> (Vec<&'static str>, Vec<u8>) {
    let names: Vec<&'static str> = [
        "bass",
        "kick",
        "snare",
        "hat_c",
        "shake",
        "piano",
        "flute",
        "sax",
    ]
    .to_vec();

    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = SWING;

    let mut p = Parts::new();
    for bar in 0i64..28 {
        match bar {
            0..=3 => {
                let (root, tones) = chord_at(bar);
                let shell = chord_notes(root, &tones, 1);
                for &n in &shell {
                    on_off_h(&mut p.piano, &grid, bar, 0, 4, n, 56);
                }
                let melody = [62u8, 65, 67, 70];
                on_off_h(&mut p.piano, &grid, bar, 2, 2, melody[bar as usize], 68);
            }
            4..=7 => {
                train_groove(&grid, bar, &mut p);
                train_bass(&grid, bar, &mut p);
                train_sax(&grid, bar, &mut p);
            }
            8..=11 => {
                train_groove(&grid, bar, &mut p);
                train_bass(&grid, bar, &mut p);
                train_sax(&grid, bar, &mut p);
                train_b_flute(&grid, bar, &mut p);
                train_b_piano(&grid, bar, &mut p);
            }
            12..=15 => {
                trans_groove(&grid, bar, &mut p);
                trans_bass(&grid, bar, &mut p);
                trans_piano(&grid, bar, &mut p);
            }
            16..=23 => {
                tea_groove(&grid, bar, &mut p);
                tea_bass(&grid, bar, &mut p);
                tea_arpeggio(&grid, bar, &mut p);
                tea_flute(&grid, bar, &mut p);
                tea_swell(&grid, bar, &mut p);
            }
            24..=27 => {
                outro_groove(&grid, bar, &mut p);
                outro_bass(&grid, bar, &mut p);
                outro_piano(&grid, bar, &mut p);
            }
            _ => {}
        }
    }

    let placed = vec![
        Placed { name: "bass", events: p.bass, dev: bass::bass() },
        Placed { name: "kick", events: p.kick, dev: drums::kick() },
        Placed { name: "snare", events: p.snare, dev: drums::snare() },
        Placed { name: "hat_c", events: p.hat_c, dev: drums::closed_hat() },
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_airy() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    let data = assemble(BPM, TAIL_SECS, placed);
    (names, data)
}

/// Compose the train-to-tea song and return the encoded WaveSabre blob.
pub fn compose() -> Vec<u8> {
    compose_placed().1
}
