//! "Mocha" — a slow bossa swipe through A-minor: straight uns swung eighths,
//! a jaunty two-feel bass, offbeat piano "chiks", shaker sizzle and a bright
//! alto sax topline with a breathy flute reply. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_mocha -- [--out <path>]`


use song::instruments::{bass, drums, flute, piano, saxophone};
use song::swingkit::*;

const BPM: f64 = 92.0;
const SWING: f64 = 0.10;
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_mocha.bin");

const CHORDS: [Chord; 16] = [
    AM7, DM7, E7, AM7, // A
    AM7, DM7, E7, AM7, // B
    AM7, DM7, FM7, E7, // C: peak colours
    AM7, DM7, E7, AM7, // D
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 90);
    if sec == 2 {
        on_off(&mut p.kick, grid, bar, 2, 1, KICK, 78);
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 78);
    }
    // Brushes sweep on 2 & 4.
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 54);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 54);
    // Shaker on every offbeat: the bossa heartbeat.
    on_off(&mut p.shake, grid, bar, 1, 1, SHAKER, 56);
    on_off(&mut p.shake, grid, bar, 3, 1, SHAKER, 56);
    on_off(&mut p.shake, grid, bar, 5, 1, SHAKER, 56);
    on_off(&mut p.shake, grid, bar, 7, 1, SHAKER, 56);
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    // Jaunty two-feel: root, octave pop, root, fifth.
    let mut hits: Vec<(i64, i64, u8)> = vec![
        (0, 1, root as u8),
        (1, 1, (root + 12) as u8),
        (4, 1, root as u8),
        (5, 1, (root + 7) as u8),
    ];
    if sec == 2 {
        hits[1] = (1, 1, (root + 7) as u8);
        hits.insert(2, (2, 1, root as u8));
        hits.insert(3, (3, 1, (root + 12) as u8));
    }
    for &(e, dur, n) in &hits {
        on_off_h(&mut p.bass, grid, bar, e, dur, n, 70);
    }
}

fn chik(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);
    let sec = bar / 4;
    let eights: &[i64] = if sec == 2 { &[1, 3, 5, 7] } else { &[3, 7] };
    for e in eights {
        for &n in shell.iter().take(2) {
            on_off_h(&mut p.piano, grid, bar, *e, 1, n, if sec == 2 { 56 } else { 50 });
        }
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let c = chord_notes(root, &tones, 0);
    // Alto sax: the melody itself, plus staccato support hits.
    if bar == 4 || bar == 12 {
        on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 66);
        on_off_h(&mut p.sax, grid, bar, 4, 1, (root + 12) as u8, 66);
    } else if bar / 4 == 2 {
        on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 60);
        on_off_h(&mut p.sax, grid, bar, 2, 1, c[1], 60);
        on_off_h(&mut p.sax, grid, bar, 4, 1, c[2], 60);
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    let n0 = chord_degree(root, &tones, (2 + bar % 3) as u8);
    let n1 = chord_degree(root, &tones, (3 + bar % 3) as u8);
    let n2 = chord_degree(root, &tones, (4 + bar % 3) as u8);
    let row: &[(i64, i64, u8)] = match sec {
        0 => &[(0, 2, n1), (2, 1, n2), (3, 1, n1), (6, 2, n0)],
        1 => &[(0, 1, n1), (2, 2, n0), (5, 2, n1), (7, 1, n2)],
        2 => &[(0, 1, n0), (1, 1, n1), (2, 2, n2), (4, 1, n1), (5, 1, n0), (6, 2, n1)],
        _ => &[(0, 2, n1), (2, 1, n2), (3, 1, n1), (6, 2, n0)],
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
        "cargo run --release --example swing_mocha -- [--out <path>]",
        DEFAULT_OUT,
    );
    let mut grid = Grid::new(BPM, SAMPLE_RATE);
    grid.swing = SWING;

    let mut p = Parts::new();
    for bar in 0i64..16 {
        groove(&grid, bar, &mut p);
        bass_line(&grid, bar, &mut p);
        chik(&grid, bar, &mut p);
        brass(&grid, bar, &mut p);
        lead(&grid, bar, &mut p);
    }

    let placed = vec![
        Placed { name: "bass", events: p.bass, dev: bass::bass() },
        Placed { name: "kick", events: p.kick, dev: drums::kick() },
        Placed { name: "snare", events: p.snare, dev: drums::snare() },
        Placed { name: "hat_c", events: p.hat_c, dev: drums::closed_hat() },
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_airy() },
        Placed { name: "sax", events: p.sax, dev: saxophone::alto_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}