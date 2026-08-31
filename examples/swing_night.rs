//! "Night" — a slow night-drive electro groove at the edge of the city:
//! swinging eight­hs, an octave-bounce bass, hats that tick and splash, sparse
//! piano lights and a flute that drifts the high lines while the tenor hums
//! low under them. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_night -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 86.0;
const SWING: f64 = 0.32;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_night.bin");

const CHORDS: [Chord; 16] = [
    DM7, GM7, A7, A7, //  A: holding the V
    DM7, GM7, A7, A7, //  B
    DM7, GM7, A7, DM7, // C: slipping home
    DM7, GM7, A7, DM7, // D
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 98);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 86);
    }
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 80);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 80);
    for e in 0..8 {
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, if e % 2 == 0 { 50 } else { 72 });
    }
    if sec >= 1 {
        on_off(&mut p.hat_o, grid, bar, 3, 1, HAT_OPEN, 64);
        on_off(&mut p.hat_o, grid, bar, 7, 1, HAT_OPEN, 68);
    }
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 56);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 56);
    if bar == 0 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 64);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _tones) = CHORDS[bar as usize];
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let sec = bar / 4;
    if sec >= 2 {
        spread(
            &mut p.bass,
            grid,
            bar,
            &[(0, 2, root as u8), (2, 1, (root + 12) as u8), (3, 1, (root + 7) as u8), (5, 1, (root + 12) as u8), (7, 1, approach as u8)],
            74,
        );
        return;
    }
    spread(
        &mut p.bass,
        grid,
        bar,
        &[(0, 3, root as u8), (3, 1, (root + 12) as u8), (6, 1, (root + 7) as u8), (7, 1, approach as u8)],
        72,
    );
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);
    let sec = bar / 4;
    let rows: &[(i64, i64)] = match sec {
        0 => &[(0, 2), (4, 2)],
        1 => &[(0, 2), (4, 2), (6, 1)],
        2 => &[(0, 2), (2, 1), (4, 2), (6, 1)],
        _ => &[(0, 2), (4, 2)],
    };
    for &(e, dur) in rows {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, e, dur, n, if sec == 2 { 56 } else { 52 });
        }
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _tones) = CHORDS[bar as usize];
    match bar / 4 {
        0 | 1 => {
            on_off_h(&mut p.sax, grid, bar, 3, 1, (root + 12) as u8, 58);
            on_off_h(&mut p.sax, grid, bar, 7, 1, (root + 12) as u8, 58);
        }
        2 => {
            on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 60);
            on_off_h(&mut p.sax, grid, bar, 4, 1, (root + 12) as u8, 60);
        }
        _ => {}
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    let n0 = chord_degree(root, &tones, (2 + bar % 3) as u8);
    let n1 = chord_degree(root, &tones, (3 + bar % 3) as u8);
    let n2 = chord_degree(root, &tones, (4 + bar % 3) as u8);
    let row: &[(i64, i64, u8)] = match sec {
        0 => &[(0, 3, n1), (4, 2, n2)],
        1 => &[(0, 2, n0), (4, 2, n2), (6, 1, n1)],
        2 => &[(0, 1, n0), (1, 1, n1), (4, 2, n2), (6, 2, n0)],
        _ => &[(0, 3, n2), (4, 2, n1)],
    };
    spread(&mut p.flute, grid, bar, row, match sec {
        0 => 64,
        1 => 68,
        2 => 74,
        _ => 64,
    });
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_night -- [--out <path>]",
        DEFAULT_OUT,
    );
    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = SWING;

    let mut p = Parts::new();
    for bar in 0i64..16 {
        groove(&grid, bar, &mut p);
        bass_line(&grid, bar, &mut p);
        comp(&grid, bar, &mut p);
        brass(&grid, bar, &mut p);
        lead(&grid, bar, &mut p);
    }

    let placed = vec![
        Placed { name: "bass", events: p.bass, dev: bass::bass() },
        Placed { name: "kick", events: p.kick, dev: drums::kick() },
        Placed { name: "snare", events: p.snare, dev: drums::snare() },
        Placed { name: "hat_c", events: p.hat_c, dev: drums::closed_hat() },
        Placed { name: "hat_o", events: p.hat_o, dev: drums::open_hat() },
        Placed { name: "crash", events: p.crash, dev: drums::crash() },
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_v3() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}