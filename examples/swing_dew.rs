//! "Dew" — a light morning swing: a warm FM7-GM7-A7-DM7 wash, soft brushes,
//! a slow root-fifth bass and an airy flute that drifts arpeggios over the top.
//! 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_dew -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 100.0;
const SWING: f64 = 0.28;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_dew.bin");

const CHORDS: [Chord; 16] = [
    FM7, GM7, A7, DM7, // A
    FM7, GM7, A7, DM7, // B
    FM7, GM7, A7, A7, //  C: the lift is the dominant
    FM7, GM7, A7, DM7, // D
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 90);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 78);
    }
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 50);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 50);
    on_off(&mut p.hat_c, grid, bar, 3, 1, HAT_CLOSED, 42);
    on_off(&mut p.hat_c, grid, bar, 7, 1, HAT_CLOSED, 42);
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 48);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 48);
    if bar == 0 || bar == 8 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 60);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    if sec == 2 {
        for e in [0i64, 2, 4, 6] {
            on_off_h(&mut p.bass, grid, bar, e, 1, (root + (if e % 4 == 0 { 0 } else { 7 })) as u8, 66);
        }
        return;
    }
    spread(
        &mut p.bass,
        grid,
        bar,
        &[(0, 4, root as u8), (4, 3, (root + 7) as u8)],
        64,
    );
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);
    let sec = bar / 4;
    let rows: &[(i64, i64)] = match sec {
        0 => &[(3, 1), (4, 2)],
        1 => &[(3, 2)],
        2 => &[(3, 1), (4, 2), (7, 2)],
        _ => &[(3, 2)],
    };
    for &(e, dur) in rows {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, e, dur, n, if sec == 2 { 54 } else { 48 });
        }
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    // The sax only hums a drone-like long note at the section starts.
    if bar == 0 || bar == 4 || bar == 8 || bar == 12 {
        on_off_h(&mut p.sax, grid, bar, 0, 4, chord_notes(root, &tones, 0)[1], 56);
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    let n0 = chord_degree(root, &tones, (2 + bar % 3) as u8);
    let n1 = chord_degree(root, &tones, (3 + bar % 3) as u8);
    let n2 = chord_degree(root, &tones, (4 + bar % 3) as u8);
    let row: &[(i64, i64, u8)] = match sec {
        0 => &[(0, 2, n0), (2, 1, n1), (4, 3, n2)],
        1 => &[(0, 1, n2), (2, 2, n0), (5, 2, n1)],
        2 => &[(0, 1, n0), (1, 1, n1), (2, 1, n2), (4, 2, n1), (6, 2, n2)],
        _ => &[(0, 3, n2), (4, 2, n1)],
    };
    spread(&mut p.flute, grid, bar, row, match sec {
        0 => 66,
        1 => 70,
        2 => 76,
        _ => 66,
    });
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_dew -- [--out <path>]",
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
        Placed { name: "flute", events: p.flute, dev: flute::flute_bright() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}