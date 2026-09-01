//! "Velvet" — a slow, smooth-lounge gratitude: lush two-feel piano shells,
//! brushing drums, a lazily walking bass and a tenor sax that sings the line
//! over a D-minor blues sheen. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_velvet -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 88.0;
const SWING: f64 = 0.30;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_velvet.bin");

const CHORDS: [Chord; 16] = [
    DM7, GM7, A7, FM7, // A
    DM7, GM7, A7, FM7, // B
    DM7, GM7, A7, A7, //  C: the turn
    DM7, GM7, A7, FM7, // D
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 90);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 78);
    }
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 56);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 56);
    on_off(&mut p.hat_c, grid, bar, 2, 1, HAT_CLOSED, 44);
    on_off(&mut p.hat_c, grid, bar, 4, 1, HAT_CLOSED, 44);
    on_off(&mut p.hat_c, grid, bar, 6, 1, HAT_CLOSED, 44);
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 50);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 50);
    if bar == 0 || bar == 8 || bar == 12 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 64);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let c = chord_notes(root, &tones, 0);
    let sec = bar / 4;
    if sec >= 2 {
        spread(
            &mut p.bass,
            grid,
            bar,
            &[(0, 2, root as u8), (2, 1, c[0]), (4, 2, c[1]), (6, 2, c[2]), (7, 1, approach as u8)],
            66,
        );
        return;
    }
    spread(
        &mut p.bass,
        grid,
        bar,
        &[(0, 4, root as u8), (4, 2, (root + 7) as u8), (7, 1, approach as u8)],
        66,
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
    let (root, tones) = CHORDS[bar as usize];
    let c = chord_notes(root, &tones, 0);
    match bar / 4 {
        0 => {
            on_off_h(&mut p.sax, grid, bar, 0, 3, c[0], 64);
            on_off_h(&mut p.sax, grid, bar, 4, 2, c[1], 64);
        }
        1 => {
            on_off_h(&mut p.sax, grid, bar, 0, 2, c[1], 64);
            on_off_h(&mut p.sax, grid, bar, 2, 2, c[2], 62);
        }
        2 => {
            on_off_h(&mut p.sax, grid, bar, 0, 1, c[1], 68);
            on_off_h(&mut p.sax, grid, bar, 1, 1, c[2], 66);
            on_off_h(&mut p.sax, grid, bar, 2, 2, c[0], 66);
            on_off_h(&mut p.sax, grid, bar, 5, 2, c[1], 64);
        }
        _ => {
            on_off_h(&mut p.sax, grid, bar, 6, 2, (root + 12) as u8, 58);
        }
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    // The flute only answers the sax with a single ringing note.
    if bar / 4 == 2 || bar / 4 == 3 {
        on_off_h(&mut p.flute, grid, bar, 7, 2, chord_degree(root, &tones, 4), 62);
    }
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_velvet -- [--out <path>]",
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