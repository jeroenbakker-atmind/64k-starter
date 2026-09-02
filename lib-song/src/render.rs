//! A tiny, platform-independent software renderer that plays a parsed song,
//! used by `examples/create_song --wav` and `examples/instrument_test --wav`
//! to preview a song on non-Windows machines.
//!
//! The actual engine lives in the `wavesabre` crate (a Rust port of the
//! WaveSabre core: Falcon + Slaughter synths, the effect devices, per-track
//! chains with receives and automation). This module is a thin facade so
//! existing callers keep working about the same public function names:
//! `render`, `render_solo`, `normalize` and `write_wav_at`.

pub use wavesabre::{normalize, render, render_solo, render_stereo, normalize_stereo, render_solo_stereo, write_wav_at, write_stereo_wav_at};
