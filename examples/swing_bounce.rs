//! "Bounce" — funky breakbeat swing: swung breakbeat (snare 2 & 4, kicks on
//! "2&" and "4&"), funk bass with octave pops, clavinet-style stabs and a
//! staccato call-and-answer brass + flute hook. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_bounce -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 118.0;
const SWING: f64 = 0.18;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_bounce.bin");

const CHORDS: [Chord; 16] = [
    DM7, A7, GM7, A7, // A
    DM7, A7, GM7, A7, // B
    DM7, GM7, DM7, A7, // C: peak
    FM7, GM7, A7, A7, //  D: turnaround
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    // Swung breakbeat: kicks on 1, "2&", 3 and "4&" (the "4&" arrives in B).
    let kicks: &[i64] = if sec == 0 { &[0, 3, 4] } else { &[0, 3, 4, 6] };
    for e in kicks {
        on_off(&mut p.kick, grid, bar, *e, 1, KICK, if *e == 0 { 98 } else { 80 });
    }
    // Backbeat snare.
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 88);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 92);
    // Swung hats with an open splash on the strong offbeats.
    for e in 0..8 {
        let vel = if e % 2 == 0 { 58 } else { 76 };
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, vel);
    }
    if sec >= 1 {
        on_off(&mut p.hat_o, grid, bar, 3, 1, HAT_OPEN, 68);
        on_off(&mut p.hat_o, grid, bar, 7, 1, HAT_OPEN, 74);
    }
    // Shaker on 2& and 4&.
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 60);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 64);
    if bar == 0 || bar == 8 || bar == 12 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 70);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let sec = bar / 4;
    if sec == 3 {
        // Simpler groove on the way out.
        spread(
            &mut p.bass,
            grid,
            bar,
            &[(0, 3, root as u8), (5, 1, (root + 7) as u8), (7, 1, approach as u8)],
            76,
        );
        return;
    }
    let mut hits: Vec<(i64, i64, u8)> = vec![
        (0, 2, root as u8),
        (3, 1, (root + 7) as u8),
        (6, 1, root as u8 + 12),
        (7, 1, approach as u8),
    ];
    if sec == 2 {
        if bar % 2 == 1 {
            hits[3] = (3, 1, chord_notes(root, &tones, 0)[0]); // third instead of fifth
        }
    }
    for &(e, dur, n) in &hits {
        on_off_h(&mut p.bass, grid, bar, e, dur, n, 74);
    }
}

fn claws(grid: &Grid, bar: i64, p: &mut Parts) {
    // Clavinet-style stabs with the 10th colour.
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 0);
    let notes = vec![shell[0], shell[2], root as u8 + 12];
    let sec = bar / 4;
    let eights: &[i64] = if sec == 2 { &[1, 3, 5, 7] } else { &[3, 7] };
    let mut vel = if sec == 2 { 62 } else { 56 };
    for e in eights {
        for &n in &notes {
            on_off_h(&mut p.piano, grid, bar, *e, 1, n, vel);
        }
        vel = 60;
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = CHORDS[bar as usize];
    match bar / 4 {
        0 | 1 => {
            on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 72);
        }
        2 => {
            on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 64);
            on_off_h(&mut p.sax, grid, bar, 4, 1, (root + 12) as u8, 64);
        }
        _ if bar == 15 => {
            on_off_h(&mut p.sax, grid, bar, 7, 2, 69, 84);
            on_off_h(&mut p.sax, grid, bar, 7, 2, 81, 78);
        }
        _ => {}
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let vel = match bar / 4 {
        0 => 66,
        1 => 70,
        2 => 78,
        _ => 70,
    };
    let row: &[(i64, i64, u8)] = match bar {
        0 => &[(0, 1, 62), (1, 1, 65), (3, 1, 67), (4, 2, 69)],
        1 => &[(0, 1, 69), (2, 2, 72), (5, 1, 69), (7, 1, 67)],
        2 => &[(0, 1, 65), (1, 1, 67), (2, 1, 69), (3, 1, 65), (4, 2, 62)],
        3 => &[(0, 1, 62), (2, 1, 67), (3, 1, 69), (4, 2, 72), (7, 1, 69)],
        4 => &[(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 62)],
        5 => &[(0, 1, 69), (1, 1, 72), (2, 1, 74), (3, 1, 72), (4, 2, 69)],
        6 => &[(0, 1, 74), (2, 2, 72), (5, 2, 69)],
        7 => &[(0, 2, 67), (2, 1, 68), (3, 1, 65), (4, 2, 62)],
        8 => &[(0, 1, 62), (1, 1, 65), (2, 1, 67), (3, 1, 65), (4, 2, 69), (7, 1, 67)],
        9 => &[(0, 1, 69), (1, 1, 72), (2, 2, 74), (4, 2, 69)],
        10 => &[(0, 1, 62), (1, 1, 65), (2, 1, 62), (3, 1, 65), (4, 1, 67), (6, 2, 69)],
        11 => &[(0, 2, 69), (2, 1, 72), (3, 1, 74), (4, 1, 72), (5, 1, 69), (6, 2, 74)],
        12 => &[(0, 2, 62), (2, 1, 65), (3, 1, 67), (4, 2, 69)],
        13 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        14 => &[(0, 2, 69), (2, 1, 72), (3, 1, 74), (4, 2, 69)],
        _ => &[(0, 3, 74), (4, 1, 72), (5, 1, 69), (6, 2, 67)],
    };
    spread(&mut p.flute, grid, bar, row, vel);
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_bounce -- [--out <path>]",
        DEFAULT_OUT,
    );
    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = SWING;

    let mut p = Parts::new();
    for bar in 0i64..16 {
        groove(&grid, bar, &mut p);
        bass_line(&grid, bar, &mut p);
        claws(&grid, bar, &mut p);
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
        Placed { name: "flute", events: p.flute, dev: flute::flute() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}