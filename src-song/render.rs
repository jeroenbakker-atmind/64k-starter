//! A tiny, platform-independent software renderer that plays a parsed song,
//! used by `examples/create_song --wav` and `examples/instrument_test --wav`
//! to preview a song on non-Windows machines.
//!
//! The actual engine lives in `crate::sabrewave` (a Rust port of the
//! WaveSabre core: Falcon + Slaughter synths, the effect devices, per-track
//! chains with receives and automation). This module is a thin facade so
//! existing callers keep working about the same public function names:
//! `render`, `render_solo`, `normalize` and `write_wav_at`.

use crate::format::ParsedSong;
use std::vec::Vec;

pub use crate::sabrewave::{normalize, render, render_solo, write_wav_at};

/// Renders the full song mix to interleaved stereo (see
/// `crate::sabrewave::render_stereo`).
pub fn render_stereo(song: &ParsedSong) -> Vec<[f32; 2]> {
    crate::sabrewave::render_stereo(song)
}

/// Renders a single track (by index) to stereo, ignoring its receives so the
/// stem contains only that track + its own device chain.
pub fn render_solo_stereo(song: &ParsedSong, ti: usize) -> Vec<[f32; 2]> {
    crate::sabrewave::render_solo_stereo(song, ti)
}

/// Stereo variant of `normalize`.
pub fn normalize_stereo(samples: &mut [[f32; 2]]) {
    crate::sabrewave::normalize_stereo(samples);
}

/// Writes a stereo 16-bit PCM WAV using the same base path as the song file
/// but with the `.wav` extension.
pub fn write_stereo_wav_at(base_path: &str, samples: &[[f32; 2]], sample_rate: u32) {
    crate::sabrewave::write_stereo_wav_at(base_path, samples, sample_rate);
}