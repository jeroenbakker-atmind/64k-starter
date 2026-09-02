//! "Late" — smoky late-night lounge: slow, deeply swung, broken-up drums
//! (kick on 1, brush snare, open hats on "2&" / "4&"), sustained bass, lush
//! piano comps and a smoky tenor sax lead with the flute as a distant sigh.
//! 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_late -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 96.0;
const SWING: f64 = 0.28; // deep lounge swing
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_late.bin");

const CHORDS: [Chord; 16] = [
    DM7, GM7, DM7, A7, // A: first verse
    GM7, FM7, DM7, A7, // B: build (iv - bVI)
    DM7, GM7, DM7, A7, // C: peak
    FM7, GM7, A7, A7, //  D: turnaround (bVI - iv - V V)
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    // Kick: just 1, plus 3 at the peak.
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 92);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 76);
    }
    // Brush snare on beats 2 & 4, ghost on "4&" at the peak.
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 66);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 70);
    if sec == 2 {
        on_off(&mut p.snare, grid, bar, 7, 1, SNARE, 42);
    }
    // Tick on the downbeats, splashy opens on "2&" and "4&".
    on_off(&mut p.hat_c, grid, bar, 0, 1, HAT_CLOSED, 40);
    on_off(&mut p.hat_c, grid, bar, 4, 1, HAT_CLOSED, 36);
    // Soft shaker brush.
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 44);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 40);
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _) = CHORDS[bar as usize];
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let sec = bar / 4;
    if bar == 15 {
        // Hold the A pedal through the turnaround.
        spread(&mut p.bass, grid, bar, &[(0, 4, root as u8), (7, 1, approach as u8)], 76);
        return;
    }
    match sec {
        2 => {
            spread(
                &mut p.bass,
                grid,
                bar,
                &[
                    (0, 3, root as u8),
                    (4, 1, (root + 7) as u8),
                    (6, 2, approach as u8),
                ],
                78,
            );
        }
        _ => {
            spread(
                &mut p.bass,
                grid,
                bar,
                &[(0, 3, root as u8), (6, 2, (root - 2) as u8), (7, 1, approach as u8)],
                74,
            );
        }
    }
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    if bar / 4 == 3 {
        return; // let the sax resolve alone
    }
    let shell = chord_notes(root, &tones, 0);
    let hits: &[(i64, i64)] = if bar / 4 == 0 { &[(3, 1)] } else { &[(3, 1), (7, 2)] };
    for &(e, dur) in hits {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, e, dur, n, 58);
        }
    }
}

fn sax_lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let vel = match bar / 4 {
        0 => 72,
        1 => 76,
        2 => 82,
        _ => 74,
    };
    let row: &[(i64, i64, u8)] = match bar {
        0 => &[(0, 3, 62), (4, 3, 65)],
        1 => &[(2, 3, 67), (6, 2, 65)],
        2 => &[(0, 3, 62), (4, 2, 69), (7, 1, 67)],
        3 => &[(0, 2, 65), (2, 2, 67), (4, 2, 69), (6, 2, 65)],
        4 => &[(0, 4, 62), (6, 2, 69)],
        5 => &[(0, 3, 67), (4, 2, 72), (7, 1, 69)],
        6 => &[(0, 3, 69), (4, 3, 65)],
        7 => &[(0, 2, 62), (2, 2, 74), (4, 2, 72)],
        8 => &[(0, 1, 74), (1, 1, 72), (2, 2, 69), (4, 2, 74), (7, 1, 72)],
        9 => &[(0, 3, 69), (4, 3, 72)],
        10 => &[(0, 2, 74), (2, 2, 69), (6, 2, 65)],
        11 => &[(0, 3, 62), (4, 1, 69), (5, 1, 72), (6, 2, 74)],
        12 => &[(0, 4, 65), (6, 2, 62)],
        13 => &[(0, 3, 62), (4, 2, 65), (7, 1, 64)],
        14 => &[(0, 3, 62), (4, 3, 69)],
        _ => &[(0, 3, 74), (4, 1, 72), (5, 1, 69), (6, 2, 67)],
    };
    spread(&mut p.sax, grid, bar, row, vel);
}

fn flute_sigh(grid: &Grid, bar: i64, p: &mut Parts) {
    // A distant high sigh on each phrase end.
    let note = match bar {
        3 => 81,
        7 => 84,
        11 => 84,
        15 => 86,
        _ => return,
    };
    spread(&mut p.flute, grid, bar, &[(6, 2, note)], 70);
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_late -- [--out <path>]",
        DEFAULT_OUT,
    );
    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = SWING;

    let mut p = Parts::new();
    for bar in 0i64..16 {
        groove(&grid, bar, &mut p);
        bass_line(&grid, bar, &mut p);
        comp(&grid, bar, &mut p);
        sax_lead(&grid, bar, &mut p);
        flute_sigh(&grid, bar, &mut p);
    }

    let placed = vec![
        Placed { name: "bass", events: p.bass, dev: bass::bass() },
        Placed { name: "kick", events: p.kick, dev: drums::kick() },
        Placed { name: "snare", events: p.snare, dev: drums::snare() },
        Placed { name: "hat_c", events: p.hat_c, dev: drums::closed_hat() },
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_soft() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}