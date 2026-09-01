//! "Tea" — elegant tea-dance swing: minuet piano arpeggios, light brushed
//! drums (kick on 1 only, idle offbeat brushes), a graceful bass oom-pah and a
//! delicate flute obligato. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_tea -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 104.0;
const SWING: f64 = 0.3; // gracious, deep swing
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_tea.bin");

const CHORDS: [Chord; 16] = [
    DM7, GM7, DM7, GM7, // A
    FM7, GM7, A7, DM7, // B
    DM7, GM7, FM7, A7, // C: peak
    DM7, FM7, GM7, A7, // D: cadence out
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    // Kick: a gentle step on 1 (plus 3 at the peak).
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 82);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 68);
    }
    // Idle brush pattern: snare ticks on "2&" and "& of 4".
    on_off(&mut p.snare, grid, bar, 3, 1, SNARE, 42);
    on_off(&mut p.snare, grid, bar, 7, 1, SNARE, 38);
    // Crisp offbeat hats.
    for e in [1i64, 3, 5, 7] {
        let vel = if e == 3 { 52 } else { 46 };
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, vel);
    }
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 42);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 40);
    if bar == 8 {
        on_off(&mut p.crash, grid, bar, 0, 3, CRASH, 54);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = CHORDS[bar as usize];
    let sec = bar / 4;
    if sec == 3 {
        // Fade to a held root.
        spread(&mut p.bass, grid, bar, &[(0, 4, root as u8)], 70);
        return;
    }
    // Oom-pah: root on 1, root on 3 with a fifth lift.
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

fn arpeggio(grid: &Grid, bar: i64, p: &mut Parts) {
    // Minuet figuration: a light broken-chord pattern in the chandelier
    // octave.
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);
    let sec = bar / 4;
    if sec == 3 {
        // Soft sustained chord before the cadence.
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, 0, 6, n, 48);
        }
        return;
    }
    match sec {
        0 => {
            for e in [2i64, 6] {
                for &n in &shell {
                    on_off_h(&mut p.piano, grid, bar, e, 1, n, 46);
                }
            }
        }
        _ => {
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
}

fn swell(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = CHORDS[bar as usize];
    match bar / 4 {
        2 => {
            on_off_h(&mut p.sax, grid, bar, 0, 2, (root + 12) as u8, 66);
            if bar == 11 {
                on_off_h(&mut p.sax, grid, bar, 6, 2, (root + 24) as u8, 62);
            }
        }
        _ if bar == 15 => {
            // Gentle fanfare into the loop's top.
            for e in [0i64, 2] {
                on_off_h(&mut p.sax, grid, bar, e, 2, (root + 12) as u8, 70);
            }
        }
        _ => {}
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let vel = match bar / 4 {
        2 => 76,
        _ => 70,
    };
    let row: &[(i64, i64, u8)] = match bar {
        0 => &[(0, 2, 62), (2, 1, 65), (3, 1, 67), (4, 2, 69)],
        1 => &[(0, 2, 65), (2, 1, 67), (3, 1, 65), (4, 2, 62)],
        2 => &[(0, 2, 69), (2, 1, 67), (3, 2, 65), (6, 1, 67), (7, 1, 69)],
        3 => &[(0, 2, 67), (2, 1, 65), (3, 1, 64), (4, 2, 62)],
        4 => &[(0, 2, 62), (2, 1, 65), (3, 1, 69), (4, 2, 72)],
        5 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        6 => &[(0, 2, 69), (2, 1, 72), (3, 2, 74), (6, 1, 72), (7, 1, 69)],
        7 => &[(0, 2, 74), (2, 1, 72), (3, 1, 69), (4, 2, 67)],
        8 => &[(0, 2, 62), (2, 1, 65), (3, 1, 67), (4, 2, 69), (6, 2, 72)],
        9 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        10 => &[(0, 2, 69), (2, 1, 72), (3, 1, 74), (4, 2, 69), (6, 2, 67)],
        11 => &[(0, 2, 74), (2, 1, 72), (3, 2, 69), (6, 2, 65)],
        12 => &[(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 62)],
        13 => &[(0, 2, 60), (2, 1, 62), (3, 1, 65), (4, 2, 67), (6, 2, 69)],
        14 => &[(0, 2, 69), (2, 1, 67), (3, 2, 65), (6, 2, 62)],
        _ => &[(0, 3, 62), (4, 2, 69), (6, 2, 72)],
    };
    spread(&mut p.flute, grid, bar, row, vel);
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_tea -- [--out <path>]",
        DEFAULT_OUT,
    );
    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = SWING;

    let mut p = Parts::new();
    for bar in 0i64..16 {
        groove(&grid, bar, &mut p);
        bass_line(&grid, bar, &mut p);
        arpeggio(&grid, bar, &mut p);
        swell(&grid, bar, &mut p);
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
        Placed { name: "flute", events: p.flute, dev: flute::flute_airy() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}