//! Musical layout helpers used by `examples/create_song`.

/// Time grid for arranging notes.
pub struct Grid {
    pub sample_rate: i64,
    pub beat_samples: i64,
    /// Swing amount for the second eighth of a pair, in fractions of a beat
    /// (0 = straight eighths, ~1/6 = triplet swing).
    pub swing: f64,
}

impl Grid {
    pub fn new(bpm: f64, sample_rate: i64) -> Grid {
        Grid {
            sample_rate,
            beat_samples: (sample_rate as f64 * 60.0 / bpm).round() as i64,
            swing: 1.0 / 6.0,
        }
    }

    /// Sample position of a beat given as (bar, beat-in-bar). Bar of 4 beats.
    pub fn at(&self, bar: i64, beat: i64) -> i64 {
        ((bar * 4 + beat) * self.beat_samples) as i64
    }

    /// Position of an eighth subdivision: `e` is 0 (on-beat) or 1 ("and").
    /// The second eighth is swung by `swing * beat_samples`.
    pub fn eighth(&self, bar: i64, beat: i64, e: i64) -> i64 {
        let base = self.at(bar, beat);
        base + if e == 0 {
            0
        } else {
            (self.swing * self.beat_samples as f64).round() as i64
        }
    }
}

/// Converts a MIDI note to a printable name (e.g. 60 -> "C4").
pub fn note_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i32 / 12) - 1;
    format!("{}{}", NAMES[(note % 12) as usize], octave)
}

/// Converts a sample position to "mm:ss.cs".
pub fn fmt_time(samples: i64, sample_rate: i64) -> String {
    let secs = samples as f64 / sample_rate as f64;
    let m = (secs / 60.0) as i32;
    let s = secs as i32 % 60;
    let cs = ((secs - secs.floor()) * 100.0) as i32;
    format!("{:02}:{:02}.{:02}", m, s, cs)
}