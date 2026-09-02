//! Generates `src/song.bin` — a 28-bar tune blending a driving train ride with
//! an elegant tea-dance, connected by a gradual transition.
//!
//! Structure:
//! ```text
//! bars 0-3    intro:  solo piano, sparse train chords
//! bars 4-7    train:  bass root-chug, kick on 1 & 3, sax staccato calls
//! bars 8-11   trainB: flute answers join, piano fills
//! bars 12-15  trans:  drums shift, bass goes oom-pah, FM7 introduced
//! bars 16-19  tea A:  piano arpeggios, oom-pah bass, flute obligato
//! bars 20-23  tea B:  sax swells, peak arc
//! bars 24-27  outro:  train callback, instruments drop, final chord
//! ```
//!
//! Usage: `cargo run --release --example create_song -- [--out <path>]`

use song::compose::compose_placed;
use song::swingkit::{parse_args, write_bin};

const DEFAULT_OUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/song.bin");

fn main() {
    let opts = parse_args(
        "cargo run --release --example create_song -- [--out <path>]",
        DEFAULT_OUT,
    );
    let (names, data) = compose_placed();
    write_bin(&data, &opts.out);
    let md = format!(
        "# Track list\n\n{}",
        names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{i}: {n}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    std::fs::write(format!("{}.md", opts.out), md).expect("failed to write track manifest");
    println!("wrote {}.md", opts.out);
}
