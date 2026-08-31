//! "Dune" — a slow desert drone over an open D-dorian modal space: straight
//! (uns swung) eighths, a held root-fifth bass, long shimmering piano fifths
//! and a flute that floats long tones over the heat-haze. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_dune -- [--out <path>]`


use starter::instruments::{falcon, slavery};
use starter::swingkit::*;

const BPM: f64 = 76.0;
const SWING: f64 = 0.0; // straight, timeless
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_dune.bin");

const CHORDS: [Chord; 16] = [
    DM7, DM7, GM7, DM7, // A: drifting
    DM7, DM7, GM7, A7, //  B: tension gathers
    DM7, GM7, DM7, A7, //  C: the crest
    DM7, DM7, GM7, DM7, //  D: return to the desert
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 86);
    if sec >= 1 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 72);
    }
    // A lone rim tap drifts in from the B section on.
    if sec >= 2 && bar % 2 == 0 {
        on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 44);
    }
    // Distant shaker shimmer on the strong offbeats.
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 52);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 52);
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    // Open drone: root whole bar, alternating with the fifth on dry bars.
    if bar % 2 == 0 {
        spread(&mut p.bass, grid, bar, &[(0, 8, root as u8)], 62);
    } else {
        if sec == 2 {
            spread(&mut p.bass, grid, bar, &[(0, 8, root as u8), (6, 2, (root + 7) as u8)], 62);
        } else {
            spread(&mut p.bass, grid, bar, &[(0, 8, root as u8)], 62);
        }
    }
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let c = chord_notes(root, &tones, 0);
    // Shimmering open fifths, only on the two-bar phrase starts.
    if bar % 2 == 1 {
        return;
    }
    let sec = bar / 4;
    let vel = if sec == 2 { 50 } else { 44 };
    if bar % 4 == 0 {
        on_off_h(&mut p.piano, grid, bar, 0, 4, c[0], vel);
        on_off_h(&mut p.piano, grid, bar, 0, 4, c[1], vel);
        on_off_h(&mut p.piano, grid, bar, 4, 4, c[1], vel - 4);
    } else {
        on_off_h(&mut p.piano, grid, bar, 0, 4, c[0] + 12, vel);
        on_off_h(&mut p.piano, grid, bar, 0, 4, c[1] + 12, vel);
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let c = chord_notes(root, &tones, 0);
    // Tenor winds one long line across each two-bar phrase.
    if bar % 2 == 0 {
        on_off_h(&mut p.sax, grid, bar, 0, 8, c[1], 58);
    } else {
        on_off_h(&mut p.sax, grid, bar, 0, 4, (root + 12) as u8, 56);
        on_off_h(&mut p.sax, grid, bar, 4, 4, c[2], 56);
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    let n0 = chord_degree(root, &tones, (2 + bar % 3) as u8);
    let n1 = chord_degree(root, &tones, (3 + bar % 3) as u8);
    let n2 = chord_degree(root, &tones, (4 + bar % 3) as u8);
    // Long floating notes; a raised sixth (the dorian colour) resolves to the
    // seventh on the dry bars.
    let row: &[(i64, i64, u8)] = if bar % 2 == 0 {
        match sec {
            0 => &[(0, 6, n2), (6, 2, n0)],
            2 => &[(0, 4, n1), (4, 4, n2), (6, 2, n0)],
            _ => &[(0, 8, n1)],
        }
    } else {
        match sec {
            0 => &[(0, 6, n0), (6, 2, n2)],
            2 => &[(0, 4, n2), (4, 4, n1), (6, 2, n0)],
            _ => &[(0, 6, n2), (6, 2, n1)],
        }
    };
    spread(&mut p.flute, grid, bar, row, match sec {
        0 => 64,
        1 => 66,
        2 => 72,
        _ => 62,
    });
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_dune -- [--out <path>]",
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
        Placed { name: "bass", events: p.bass, dev: slavery::bass::bass() },
        Placed { name: "kick", events: p.kick, dev: slavery::drums::kick() },
        Placed { name: "snare", events: p.snare, dev: slavery::drums::snare() },
        Placed { name: "hat_c", events: p.hat_c, dev: slavery::drums::closed_hat() },
        Placed { name: "hat_o", events: p.hat_o, dev: slavery::drums::open_hat() },
        Placed { name: "crash", events: p.crash, dev: slavery::drums::crash() },
        Placed { name: "shake", events: p.shake, dev: slavery::drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: slavery::piano::piano() },
        Placed { name: "flute", events: p.flute, dev: falcon::flute::flute_v3() },
        Placed { name: "sax", events: p.sax, dev: slavery::saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}