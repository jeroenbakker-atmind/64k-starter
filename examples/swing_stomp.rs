//! "Stomp" — driving big-band electro swing: 4-on-the-floor chorus, walking
//! bass, brass stabs and a dance-hall flute hook. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_stomp -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 126.0;
const SWING: f64 = 0.2;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_stomp.bin");

const CHORDS: [Chord; 16] = [
    DM7, GM7, DM7, A7, // A: first verse
    DM7, GM7, DM7, A7, // B: second verse (build)
    DM7, GM7, A7, A7, //  C: peak (two V7s for lift)
    DM7, GM7, A7, A7, //  D: turnaround
];

fn candimate_bar(bar: i64) -> (i32, [i32; 4]) {
    CHORDS[bar as usize]
}

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    // Kick: 4-on-the-floor at the peak, 1 & 3 elsewhere.
    if sec == 2 {
        for beat in 0..4 {
            on_off(&mut p.kick, grid, bar, beat * 2, 1, KICK, if beat == 0 { 100 } else { 84 });
        }
    } else {
        on_off(&mut p.kick, grid, bar, 0, 1, KICK, 94);
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 82);
    }
    // Backbeat.
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 86);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 90);
    // Swung eighths with accented offbeats; lighter on the turnaround.
    for e in 0..8 {
        let vel = if e % 2 == 0 {
            if sec == 3 { 52 } else { 58 }
        } else if sec == 3 {
            62
        } else {
            76
        };
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, vel);
    }
    // 16th-hat riser into the turnaround.
    if bar == 11 {
        for k in 8..16 {
            on_off_16(&mut p.hat_c, grid, bar, k, 1, HAT_CLOSED, 46 + k as u8 * 3);
        }
    }
    // Shaker glue on the strong offbeats.
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 66);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 70);
    // Section crashes.
    if bar == 0 || bar == 8 || bar == 12 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 76);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = candimate_bar(bar);
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let sec = bar / 4;

    match sec {
        0 => {
            // Swing bounce.
            spread(
                &mut p.bass,
                grid,
                bar,
                &[(0, 2, root as u8), (3, 1, (root + 7) as u8), (7, 1, approach as u8)],
                74,
            );
        }
        1 => {
            // Chordal bounce with a root skip on "2&".
            spread(
                &mut p.bass,
                grid,
                bar,
                &[(0, 2, root as u8), (2, 1, (root + 12) as u8), (3, 1, (root + 7) as u8), (7, 1, approach as u8)],
                74,
            );
        }
        2 => {
            // Walking quarters: root, 3rd, 5th, 7th.
            let c = chord_notes(root, &tones, 0);
            spread(
                &mut p.bass,
                grid,
                bar,
                &[
                    (0, 1, root as u8),
                    (2, 1, c[0]),
                    (4, 1, c[1]),
                    (6, 1, c[2]),
                ],
                74,
            );
        }
        _ => {
            // Turnaround: held roots with a fifth pop on "4&".
            spread(
                &mut p.bass,
                grid,
                bar,
                &[(0, 3, root as u8), (6, 1, (root + 7) as u8), (7, 1, approach as u8)],
                74,
            );
        }
    }
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = candimate_bar(bar);
    if bar == 15 {
        return; // let the horns blast
    }
    let shell = chord_notes(root, &tones, 1);
    match bar / 4 {
        0 => {
            // Single "2&" hit on the first verse.
            for &n in &shell {
                on_off_h(&mut p.piano, grid, bar, 3, 1, n, 56);
            }
        }
        1 | 2 => {
            // "2&" staccato + "& of 4" ring into the next bar.
            for &(e, dur) in &[(3, 1), (7, 2)] {
                for &n in &shell {
                    on_off_h(&mut p.piano, grid, bar, e, dur, n, 60);
                }
            }
        }
        _ => {}
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = candimate_bar(bar);
    match bar / 4 {
        0 | 1 => {
            // Downbeat punches.
            on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 74);
        }
        2 => {
            // Offbeat push accents (the stomp riff), quieter so the flute can lead.
            for e in [1i64, 3, 5, 7] {
                on_off_h(&mut p.sax, grid, bar, e, 1, (root + 12) as u8, 60);
            }
        }
        _ if bar == 15 => {
            // A7 -> Dm turnaround blast.
            on_off_h(&mut p.sax, grid, bar, 7, 2, 69, 84);
            on_off_h(&mut p.sax, grid, bar, 7, 2, 81, 78);
        }
        _ => {}
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let vel = match bar / 4 {
        0 => 68,
        1 => 72,
        2 => 78,
        _ => 70,
    };
    let row: &[(i64, i64, u8)] = match bar {
        0 => &[(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 62)],
        1 => &[(0, 2, 65), (2, 1, 65), (3, 1, 67), (4, 2, 69)],
        2 => &[(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 62), (6, 2, 69)],
        3 => &[(0, 2, 67), (2, 1, 69), (3, 2, 72), (6, 1, 69), (7, 1, 67)],
        4 => &[(0, 2, 62), (2, 1, 65), (3, 1, 69), (4, 2, 72)],
        5 => &[(0, 2, 69), (2, 1, 72), (3, 2, 74), (6, 1, 72), (7, 1, 69)],
        6 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        7 => &[(0, 2, 69), (2, 1, 72), (3, 1, 74), (4, 2, 72), (6, 2, 69)],
        8 => &[(0, 1, 69), (1, 1, 72), (2, 1, 74), (3, 1, 72), (4, 2, 69)],
        9 => &[(0, 1, 69), (1, 1, 72), (2, 2, 74), (4, 1, 72), (5, 1, 69)],
        10 => &[(0, 1, 69), (1, 1, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65)],
        11 => &[(0, 2, 62), (2, 1, 65), (3, 1, 69), (5, 2, 74), (7, 1, 72)],
        12 => &[(0, 2, 62), (2, 2, 65), (4, 1, 67), (5, 1, 69)],
        13 => &[(0, 2, 72), (2, 1, 69), (3, 1, 67), (4, 2, 65), (6, 2, 62)],
        14 => &[(0, 2, 69), (2, 1, 72), (3, 1, 74), (4, 2, 69)],
        _ => &[(0, 2, 69), (2, 1, 72), (3, 2, 74), (6, 2, 72)],
    };
    spread(&mut p.flute, grid, bar, row, vel);
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_stomp -- [--out <path>]",
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
        Placed { name: "flute", events: p.flute, dev: flute::flute_soft() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}