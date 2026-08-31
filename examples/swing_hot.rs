//! "Hot" — gypsy-minor electro swing: a tarantella pulse (running bass
//! eighths, offbeat rhythm-guitar chiks, backbeat rim) in A minor with
//! "Spanish" E7 colour. 16 bars, A B C D.
//!
//! Usage: `cargo run --release --example swing_hot -- [--out <path>]`


use starter::instruments::{bass, drums, flute, piano, saxophone};
use starter::swingkit::*;

const BPM: f64 = 138.0;
const SWING: f64 = 0.12; // straighter, driving tarantella
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_hot.bin");

const CHORDS: [Chord; 16] = [
    AM7, D7, AM7, E7, // A
    AM7, D7, AM7, E7, // B
    AM7, E7, FM7, E7, // C: peak (i-V-bVI-V)
    AM7, D7, FM7, E7, // D: turnaround
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    let sec = bar / 4;
    // Kick: light pulse; 4-floor at the peak.
    if sec == 2 {
        for beat in 0..4 {
            on_off(&mut p.kick, grid, bar, beat * 2, 1, KICK, if beat == 0 { 98 } else { 82 });
        }
    } else {
        on_off(&mut p.kick, grid, bar, 0, 1, KICK, 90);
        on_off(&mut p.kick, grid, bar, 4, 1, KICK, 80);
        if sec >= 1 {
            on_off(&mut p.kick, grid, bar, 6, 1, KICK, 68); // "4&" push
        }
    }
    on_off(&mut p.snare, grid, bar, 2, 1, SNARE, 84);
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 88);
    for e in 0..8 {
        let vel = if e % 2 == 0 { 54 } else { 74 };
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, vel);
    }
    // Tambourine: the two strong offbeats only.
    for e in [3i64, 7] {
        on_off(&mut p.shake, grid, bar, e, 1, SHAKER, 64);
    }
    if bar == 8 || bar == 12 {
        on_off(&mut p.crash, grid, bar, 0, 2, CRASH, 84);
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    let sec = bar / 4;
    let chord = chord_notes(root, &tones, 0);
    let mut hits: Vec<(i64, i64, u8)> = vec![
        (0, 1, root as u8),
        (1, 1, chord[2] as _), // seventh just above the root = octave pop
        (2, 1, root as u8),
        (3, 1, root as u8 + 12),
        (4, 1, root as u8),
        (5, 1, root as u8 + 12),
        (6, 1, root as u8),
        (7, 1, root as u8 + 12),
    ];
    if sec == 1 {
        // Colour the offbeats with chord tones.
        hits[1] = (1, 1, chord[0]);
        hits[5] = (5, 1, chord[1]);
    }
    if bar == 11 || bar == 15 {
        // Run-up into the next section's downbeat.
        hits[6] = (6, 1, root as u8 + 11);
        hits[7] = (7, 1, root as u8 + 12);
    }
    let mut vel = if sec == 2 { 78 } else { 72 };
    for &(e, dur, n) in &hits {
        on_off_h(&mut p.bass, grid, bar, e, dur, n, vel);
        vel = 72; // keep subsequent hits steadier
    }
}

fn chik(grid: &Grid, bar: i64, p: &mut Parts) {
    // Rhythm-guitar style offbeat stabs on every "&".
    let (root, tones) = CHORDS[bar as usize];
    let shell = chord_notes(root, &tones, 1);
    let vel = if CHORDS[bar as usize].0 == 52 { 60 } else { 54 };
    for e in [1i64, 3, 5, 7] {
        for &n in shell.iter().take(2) {
            on_off_h(&mut p.piano, grid, bar, e, 1, n, vel);
        }
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
            on_off_h(&mut p.sax, grid, bar, 7, 2, 64, 86); // E5 into A
            on_off_h(&mut p.sax, grid, bar, 7, 2, 76, 82);
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
        0 => &[(0, 2, 69), (2, 1, 67), (3, 1, 65), (4, 2, 62)],
        1 => &[(0, 2, 65), (2, 1, 64), (3, 1, 60), (4, 2, 57)],
        2 => &[(0, 2, 69), (2, 1, 67), (3, 2, 65), (6, 1, 64), (7, 1, 62)],
        3 => &[(0, 1, 62), (1, 1, 64), (2, 1, 65), (3, 2, 68), (6, 2, 69)],
        4 => &[(0, 2, 69), (2, 1, 72), (3, 1, 69), (4, 2, 65)],
        5 => &[(0, 2, 72), (2, 1, 69), (3, 1, 71), (4, 2, 65)],
        6 => &[(0, 2, 65), (2, 1, 67), (3, 1, 69), (4, 2, 72), (6, 1, 69), (7, 1, 67)],
        7 => &[(0, 2, 67), (2, 1, 69), (3, 2, 65), (6, 2, 62)],
        8 => &[(0, 1, 69), (1, 1, 72), (2, 1, 76), (3, 1, 72), (4, 2, 69)],
        9 => &[(0, 1, 76), (1, 1, 72), (2, 2, 69), (4, 1, 68), (5, 1, 69), (6, 2, 72)],
        10 => &[(0, 1, 69), (1, 1, 67), (2, 1, 65), (3, 1, 62), (4, 2, 60)],
        11 => &[(0, 2, 57), (2, 1, 65), (3, 1, 68), (5, 2, 72), (7, 1, 69)],
        12 => &[(0, 2, 69), (2, 2, 65), (4, 1, 64), (5, 1, 62)],
        13 => &[(0, 2, 69), (2, 1, 67), (3, 1, 64), (4, 2, 60)],
        14 => &[(0, 2, 65), (2, 1, 64), (3, 2, 62), (6, 1, 60), (7, 1, 57)],
        _ => &[(0, 2, 57), (2, 1, 69), (3, 2, 68), (6, 2, 69)],
    };
    spread(&mut p.flute, grid, bar, row, vel);
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_hot -- [--out <path>]",
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
        Placed { name: "hat_o", events: p.hat_o, dev: drums::open_hat() },
        Placed { name: "crash", events: p.crash, dev: drums::crash() },
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_v2() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}