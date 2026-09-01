//! "Amber" — a warm slow-brushed ballad: 4-on-1 kick, brushes on 2 & 4, an
//! oom-pah walking bass and a tenor sax that holds the chord in long notes
//! while the flute paints a few sparse figures. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_amber -- [--out <path>]`


use starter::instruments::slaughter;
use starter::instruments::{drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 84.0;
const SWING: f64 = 0.32;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_amber.bin");

const CHORDS: [Chord; 16] = [
    DM7, GM7, DM7, A7, // A: first verse
    DM7, GM7, DM7, A7, // B: second verse (build)
    DM7, GM7, A7, A7, //  C: peak
    DM7, GM7, A7, DM7, //  D: release
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 94);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 80);
    }
    // Soft brushes.
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 58);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 58);
    // Hats: offbeats only, then a tick on 1.
    on_off(&mut p.hat_c, grid, bar, 2, 1, HAT_CLOSED, 44);
    on_off(&mut p.hat_c, grid, bar, 4, 1, HAT_CLOSED, 44);
    on_off(&mut p.hat_c, grid, bar, 6, 1, HAT_CLOSED, 44);
    if sec >= 2 {
        on_off(&mut p.hat_c, grid, bar, 0, 1, HAT_CLOSED, 40);
    }
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 52);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 52);
    if bar == 0 || bar == 8 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 66);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let sec = bar / 4;
    if sec >= 1 {
        spread(
            &mut p.bass,
            grid,
            bar,
            &[(0, 3, root as u8), (2, 1, (root + 12) as u8), (3, 1, chord_notes(root, &tones, 0)[1]), (7, 1, approach as u8)],
            68,
        );
        return;
    }
    spread(
        &mut p.bass,
        grid,
        bar,
        &[(0, 3, root as u8), (3, 1, chord_notes(root, &tones, 0)[1]), (7, 1, approach as u8)],
        68,
    );
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);
    let sec = bar / 4;
    if sec == 0 {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, 3, 1, n, 50);
        }
        return;
    }
    if sec == 1 {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, 3, 1, n, 52);
            on_off_h(&mut p.piano, grid, bar, 7, 2, n, 52);
        }
        return;
    }
    for &(e, dur) in &[(3, 1), (4, 2)] {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, e, dur, n, 56);
        }
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let c = chord_notes(root, &tones, 0);
    match bar / 4 {
        0 => {
            on_off_h(&mut p.sax, grid, bar, 0, 4, c[2], 60);
        }
        1 => {
            on_off_h(&mut p.sax, grid, bar, 0, 4, c[1], 62);
            on_off_h(&mut p.sax, grid, bar, 4, 2, c[0], 62);
        }
        2 => {
            on_off_h(&mut p.sax, grid, bar, 0, 4, c[1], 64);
            on_off_h(&mut p.sax, grid, bar, 4, 2, c[2], 64);
            on_off_h(&mut p.sax, grid, bar, 6, 2, (root + 12) as u8, 58);
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
        0 => &[(0, 4, n2), (6, 1, n0)],
        1 => &[(0, 2, n0), (2, 2, n1), (6, 1, n2)],
        2 => &[(0, 1, n0), (1, 1, n1), (2, 1, n2), (4, 2, n0), (6, 2, n1)],
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
        "cargo run --release --example swing_amber -- [--out <path>]",
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
        Placed { name: "bass", events: p.bass, dev: slaughter::bass::bass() },
        Placed { name: "kick", events: p.kick, dev: drums::kick() },
        Placed { name: "snare", events: p.snare, dev: drums::snare() },
        Placed { name: "hat_c", events: p.hat_c, dev: drums::closed_hat() },
        Placed { name: "hat_o", events: p.hat_o, dev: drums::open_hat() },
        Placed { name: "crash", events: p.crash, dev: drums::crash() },
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_soft() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}