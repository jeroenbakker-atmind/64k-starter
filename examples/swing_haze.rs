//! "Haze" — a low, smoky film-noir quite: A-minor i-iv-V-i, brushes down to a
//! whisper, a slow two-feel bass and a tenor that smokes over the bars while
//! the flute drips a few blue responses. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_haze -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 78.0;
const SWING: f64 = 0.34;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_haze.bin");

const CHORDS: [Chord; 16] = [
    AM7, DM7, E7, AM7, // A
    AM7, DM7, E7, AM7, // B
    AM7, DM7, E7, AM7, // C: peak
    AM7, DM7, E7, AM7, // D: resolve
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 92);
    if sec == 3 {
        on_off(&mut p.kick, grid, bar, 6, 1, KICK, 70); // "4&" push home
    }
    // Brushes at a whisper.
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 52);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 52);
    // Ticking hats on the downbeats, opening up slightly at the peak.
    on_off(&mut p.hat_c, grid, bar, 0, 1, HAT_CLOSED, 46);
    on_off(&mut p.hat_c, grid, bar, 4, 1, HAT_CLOSED, 46);
    if sec >= 2 {
        on_off(&mut p.hat_c, grid, bar, 2, 1, HAT_CLOSED, 48);
        on_off(&mut p.hat_c, grid, bar, 6, 1, HAT_CLOSED, 48);
    }
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 50);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 50);
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let next_root = CHORDS[((bar + 1) % 16) as usize].0;
    let approach = if next_root > root { next_root - 1 } else { next_root + 1 };
    let c = chord_notes(root, &tones, 0);
    let sec = bar / 4;
    if sec == 2 {
        // Slow walk: root, 3rd, 5th, 7th.
        spread(
            &mut p.bass,
            grid,
            bar,
            &[(0, 1, root as u8), (2, 1, c[0]), (4, 1, c[1]), (6, 1, c[2]), (7, 1, approach as u8)],
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
        0 => &[(3, 1)],
        1 => &[(3, 1), (7, 2)],
        2 => &[(0, 1), (3, 1)],
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
    let c = chord_notes(root, &tones, 0);
    match bar / 4 {
        0 => {
            on_off_h(&mut p.sax, grid, bar, 0, 4, c[1], 62);
            on_off_h(&mut p.sax, grid, bar, 4, 4, c[2], 60);
        }
        1 => {
            on_off_h(&mut p.sax, grid, bar, 0, 4, c[1], 62);
            on_off_h(&mut p.sax, grid, bar, 4, 2, c[0], 62);
        }
        2 => {
            on_off_h(&mut p.sax, grid, bar, 2, 1, c[1], 66);
            on_off_h(&mut p.sax, grid, bar, 6, 2, (root + 12) as u8, 60);
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
        0 => &[(0, 3, n2), (4, 1, n1)],
        1 => &[(0, 2, n0), (2, 1, n2), (6, 2, n1)],
        2 => &[(0, 1, n0), (1, 1, n1), (2, 2, n2), (4, 1, n1), (5, 1, n0), (6, 2, n2)],
        _ => &[(0, 3, n2), (4, 2, n1)],
    };
    spread(&mut p.flute, grid, bar, row, match sec {
        0 => 62,
        1 => 66,
        2 => 72,
        _ => 62,
    });
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_haze -- [--out <path>]",
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
        Placed { name: "flute", events: p.flute, dev: flute::flute_airy() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}