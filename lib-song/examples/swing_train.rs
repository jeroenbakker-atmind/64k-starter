//! "Train" — a slow night-train roll: straight chugging eighths (bass + hats),
//! kick on 1 & 3, a rim tap on 4, and a tenor sax that calls in short staccato
//! phrases while the flute answers from the far end of the car. 16 bars.
//!
//! Usage: `cargo run --release --example swing_train -- [--out <path>]`


use song::instruments::{bass, drums, flute, piano, saxophone};
use song::swingkit::*;

const BPM: f64 = 68.0;
const SWING: f64 = 0.0; // straight chug, no groove
const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/swing_train.bin");

const CHORDS: [Chord; 16] = [
    DM7, A7, GM7, A7, // A
    DM7, A7, GM7, A7, // B
    DM7, GM7, A7, A7, // C: full-steam
    DM7, A7, GM7, A7, // D
];

fn groove(grid: &Grid, bar: i64, p: &mut Parts) {
    on_off(&mut p.kick, grid, bar, 0, 1, KICK, 92);
    on_off(&mut p.kick, grid, bar, 4, 1, KICK, 86);
    // Rim tap on the "4" — the brake feel.
    on_off(&mut p.snare, grid, bar, 6, 1, SNARE, 60);
    // Hats tick every eighth, just louder on the backbeat.
    for e in 0..8 {
        on_off(&mut p.hat_c, grid, bar, e, 1, HAT_CLOSED, if e % 4 == 2 { 50 } else { 42 });
    }
}

fn bass_line(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, _tones) = CHORDS[bar as usize];
    // The chug: root on the beats, fifth on the offbeats.
    let sec = bar / 4;
    let vel = if sec == 2 { 62 } else { 56 };
    for beat in 0..4 {
        on_off_h(&mut p.bass, grid, bar, beat * 2, 1, root as u8, vel);
        on_off_h(&mut p.bass, grid, bar, beat * 2 + 1, 1, (root + 7) as u8, vel - 6);
    }
}

fn comp(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    // Sparse downstroke piano on "2&" and "4&" only — the rhythm's already busy.
    let shell = chord_notes(root, &tones, 1);
    if bar / 4 == 2 {
        for &n in &shell {
            on_off_h(&mut p.piano, grid, bar, 3, 1, n, 50);
            on_off_h(&mut p.piano, grid, bar, 7, 1, n, 50);
        }
    }
}

fn brass(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    // The CALL: short, strong, on the beat.
    match bar / 4 {
        0 | 1 => {
            on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 66);
            on_off_h(&mut p.sax, grid, bar, 4, 1, (root + 12) as u8, 64);
        }
        2 => {
            for e in [0i64, 2, 4, 6] {
                on_off_h(&mut p.sax, grid, bar, e, 1, chord_notes(root, &tones, 0)[1], 64);
            }
        }
        _ => {
            on_off_h(&mut p.sax, grid, bar, 0, 1, (root + 12) as u8, 62);
        }
    }
}

fn lead(grid: &Grid, bar: i64, p: &mut Parts) {
    let (root, tones) = CHORDS[bar as usize];
    // The ANSWER: a falling figure that lands on the next chord's note.
    let n2 = chord_degree(root, &tones, 4);
    let n1 = chord_degree(root, &tones, 3);
    let n0 = chord_degree(root, &tones, 2);
    match bar / 4 {
        0 => {
            on_off_h(&mut p.flute, grid, bar, 2, 2, n2, 66);
            on_off_h(&mut p.flute, grid, bar, 6, 2, n1, 66);
        }
        1 => {
            on_off_h(&mut p.flute, grid, bar, 2, 2, n1, 68);
            on_off_h(&mut p.flute, grid, bar, 6, 2, n0, 68);
        }
        2 => {
            on_off_h(&mut p.flute, grid, bar, 1, 1, n2, 72);
            on_off_h(&mut p.flute, grid, bar, 2, 1, n1, 70);
            on_off_h(&mut p.flute, grid, bar, 5, 2, n2, 72);
            on_off_h(&mut p.flute, grid, bar, 7, 1, n0, 68);
        }
        _ => {
            on_off_h(&mut p.flute, grid, bar, 2, 3, n2, 64);
        }
    }
}

fn main() {
    let opts = parse_args(
        "cargo run --release --example swing_train -- [--out <path>]",
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
        Placed { name: "shake", events: p.shake, dev: drums::shaker() },
        Placed { name: "piano", events: p.piano, dev: piano::piano() },
        Placed { name: "flute", events: p.flute, dev: flute::flute_bright() },
        Placed { name: "sax", events: p.sax, dev: saxophone::tenor_sax() },
    ];
    write_song(BPM, TAIL_SECS, placed, &opts.out);
}