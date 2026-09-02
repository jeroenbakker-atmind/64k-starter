//! Filter / delay primitives ported from `WaveSabreCore`.

use super::helpers;

// ===========================================================================
// StateVariableFilter
// ===========================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SvfType {
    Lowpass = 0,
    Highpass = 1,
    Bandpass = 2,
    Notch = 3,
}

/// `WaveSabreCore/StateVariableFilter.cpp`. Per-sample bilinear-ish SVF that
/// averages two `run`s (one with the previous+current input averaged).
pub struct StateVariableFilter {
    recalculate: bool,
    pub ty: SvfType,
    pub freq: f32,
    pub q: f32,
    last_input: f32,
    low: f32,
    band: f32,
    f: f32,
}

impl StateVariableFilter {
    pub fn new() -> StateVariableFilter {
        StateVariableFilter {
            recalculate: true,
            ty: SvfType::Lowpass,
            freq: 20.0,
            q: 1.0,
            last_input: 0.0,
            low: 0.0,
            band: 0.0,
            f: 0.0,
        }
    }

    pub fn set_type(&mut self, ty: SvfType) {
        if ty == self.ty {
            return;
        }
        self.ty = ty;
        self.recalculate = true;
    }

    pub fn set_freq(&mut self, freq: f32) {
        if freq == self.freq {
            return;
        }
        self.freq = freq;
        self.recalculate = true;
    }

    pub fn set_q(&mut self, q: f32) {
        if q == self.q {
            return;
        }
        self.q = q;
        self.recalculate = true;
    }

    pub fn next(&mut self, input: f32, sr: f64) -> f32 {
        if self.recalculate {
            self.f = (1.5 * helpers::fast_sin(3.141592 * (self.freq as f64) / 2.0 / sr)) as f32;
            self.recalculate = false;
        }
        let ret =
            (self.run((self.last_input + input) / 2.0) + self.run(input)) / 2.0;
        self.last_input = input;
        ret
    }

    fn run(&mut self, input: f32) -> f32 {
        self.low = self.low + self.f * self.band;
        let high = self.q * (input - self.band) - self.low;
        self.band = self.band + self.f * high;
        match self.ty {
            SvfType::Lowpass => self.low,
            SvfType::Highpass => high,
            SvfType::Bandpass => self.band,
            SvfType::Notch => self.low + high,
        }
    }
}

// ===========================================================================
// BiquadFilter
// ===========================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BiquadType {
    Lowpass = 0,
    Highpass = 1,
    Peak = 2,
}

/// `WaveSabreCore/BiquadFilter.cpp` (RBJ-style coefficients, `Next` with
/// `lastInput/lastOutput` terms).
pub struct BiquadFilter {
    recalculate: bool,
    ty: BiquadType,
    freq: f32,
    q: f32,
    gain: f32,
    last_input: f32,
    last_last_input: f32,
    last_output: f32,
    last_last_output: f32,
    c1: f32,
    c2: f32,
    c3: f32,
    c4: f32,
    c5: f32,
}

impl BiquadFilter {
    pub fn new() -> BiquadFilter {
        BiquadFilter {
            recalculate: true,
            ty: BiquadType::Lowpass,
            freq: 1000.0,
            q: 1.0,
            gain: 0.0,
            last_input: 0.0,
            last_last_input: 0.0,
            last_output: 0.0,
            last_last_output: 0.0,
            c1: 0.0,
            c2: 0.0,
            c3: 0.0,
            c4: 0.0,
            c5: 0.0,
        }
    }

    pub fn set_type(&mut self, ty: BiquadType) {
        if ty == self.ty {
            return;
        }
        self.ty = ty;
        self.recalculate = true;
    }

    pub fn set_freq(&mut self, freq: f32) {
        if freq == self.freq {
            return;
        }
        self.freq = freq;
        self.recalculate = true;
    }

    pub fn set_q(&mut self, q: f32) {
        if q == self.q {
            return;
        }
        self.q = q;
        self.recalculate = true;
    }

    pub fn set_gain(&mut self, gain: f32) {
        if gain == self.gain {
            return;
        }
        self.gain = gain;
        self.recalculate = true;
    }

    pub fn next(&mut self, input: f32, sr: f64) -> f32 {
        if self.recalculate {
            let w0 = 2.0 * 3.141592 * (self.freq as f64) / sr;
            let alpha = (helpers::fast_sin(w0) / (2.0 * (self.q as f64))) as f32;

            let (a1, a2, b0, b1, b2, a0) = match self.ty {
                BiquadType::Lowpass => {
                    let cosw0 = (helpers::fast_cos(w0) as f32) * -2.0;
                    let bb = 1.0 - (helpers::fast_cos(w0) as f32);
                    (
                        cosw0,
                        1.0 - alpha,
                        bb / 2.0,
                        bb,
                        bb / 2.0,
                        1.0 + alpha,
                    )
                }
                BiquadType::Highpass => {
                    let cosw0 = (helpers::fast_cos(w0) as f32) * -2.0;
                    let bb = 1.0 + (helpers::fast_cos(w0) as f32);
                    (
                        cosw0,
                        1.0 - alpha,
                        bb / 2.0,
                        -bb,
                        bb / 2.0,
                        1.0 + alpha,
                    )
                }
                BiquadType::Peak => {
                    let a = helpers::exp10f(self.gain / 40.0);
                    (
                        -2.0 * (helpers::fast_cos(w0) as f32),
                        1.0 - alpha / a,
                        1.0 + alpha * a,
                        -2.0 * (helpers::fast_cos(w0) as f32),
                        1.0 - alpha * a,
                        1.0 + alpha / a,
                    )
                }
            };

            self.c1 = b0 / a0;
            self.c2 = b1 / a0;
            self.c3 = b2 / a0;
            self.c4 = a1 / a0;
            self.c5 = a2 / a0;
            self.recalculate = false;
        }

        let output = self.c1 * input
            + self.c2 * self.last_input
            + self.c3 * self.last_last_input
            - self.c4 * self.last_output
            - self.c5 * self.last_last_output;

        self.last_last_input = self.last_input;
        self.last_input = input;
        self.last_last_output = self.last_output;
        self.last_output = output;

        output
    }
}

// ===========================================================================
// DelayBuffer
// ===========================================================================

/// `WaveSabreCore/DelayBuffer.cpp`. A plain circular delay; `SetLength(ms)`
/// reallocates (ringing-flush) when the length changes.
#[derive(Default)]
pub struct DelayBuffer {
    length: usize,
    buffer: Vec<f32>,
    position: usize,
}

impl DelayBuffer {
    pub fn new() -> DelayBuffer {
        DelayBuffer {
            length: 0,
            buffer: Vec::new(),
            position: 0,
        }
    }

    pub fn set_length(&mut self, length_ms: f32, sr: f64) {
        let samples = (length_ms as f64 * sr / 1000.0) as usize;
        self.set_length_samples(samples);
    }

    pub fn set_length_samples(&mut self, samples: usize) {
        let samples = samples.max(1);
        if samples != self.length || self.buffer.is_empty() {
            self.buffer = vec![0.0; samples];
            self.position = 0;
            self.length = samples;
        }
    }

    pub fn write_sample(&mut self, sample: f32) {
        self.buffer[self.position] = sample;
        self.position += 1;
        if self.position >= self.length {
            self.position = 0;
        }
    }

    pub fn read_sample(&self) -> f32 {
        if self.buffer.is_empty() {
            0.0
        } else {
            self.buffer[self.position]
        }
    }
}

// ===========================================================================
// ResampleBuffer
// ===========================================================================

/// `WaveSabreCore/ResampleBuffer.cpp`. Circular buffer that writes backwards
/// and reads a fractional position with linear interpolation.
pub struct ResampleBuffer {
    length: usize,
    buffer: Vec<f32>,
    current_position: usize,
}

impl ResampleBuffer {
    pub fn new() -> ResampleBuffer {
        ResampleBuffer {
            length: 0,
            buffer: Vec::new(),
            current_position: 0,
        }
    }

    pub fn set_length(&mut self, length_ms: f32, sr: f64) {
        let samples = (length_ms as f64 * sr / 1000.0) as usize;
        self.set_length_samples(samples);
    }

    pub fn set_length_samples(&mut self, samples: usize) {
        let samples = samples.max(1);
        if samples != self.length || self.buffer.is_empty() {
            self.buffer = vec![0.0; samples];
            self.current_position = 0;
            self.length = samples;
        }
    }

    pub fn write_sample(&mut self, sample: f32) {
        self.buffer[self.current_position] = sample;
        self.current_position = if self.current_position == 0 {
            self.length - 1
        } else {
            self.current_position - 1
        };
    }

    pub fn read_position(&self, position: f32) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        let len = self.length;
        let sample_pos = (self.current_position + position as usize) % len;
        let fraction = position - position.floor();
        let s0 = self.buffer[sample_pos];
        let s1 = if sample_pos > 0 {
            self.buffer[sample_pos - 1]
        } else {
            self.buffer[len - 1]
        };
        s0 + fraction * (s1 - s0)
    }
}

// ===========================================================================
// AllPass
// ===========================================================================

/// `WaveSabreCore/AllPass.cpp` (used by Cathedral's reverb network).
#[derive(Default)]
pub struct AllPass {
    feedback: f32,
    buffer: Vec<f32>,
    buffer_index: usize,
}

impl AllPass {
    pub fn new() -> AllPass {
        AllPass {
            feedback: 0.0,
            buffer: Vec::new(),
            buffer_index: 0,
        }
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        let size = size.max(1);
        self.buffer = vec![0.0; size];
        self.buffer_index = 0;
    }

    pub fn set_feedback(&mut self, v: f32) {
        self.feedback = v;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let buffer_out = self.buffer[self.buffer_index];
        let output = -input + buffer_out;
        self.buffer[self.buffer_index] = input + buffer_out * self.feedback;
        self.buffer_index = (self.buffer_index + 1) % self.buffer.len();
        output
    }
}

// ===========================================================================
// Comb
// ===========================================================================

/// `WaveSabreCore/Comb.cpp` (used by Cathedral's reverb network).
#[derive(Default)]
pub struct Comb {
    feedback: f32,
    damp1: f32,
    damp2: f32,
    filter_store: f32,
    buffer: Vec<f32>,
    buffer_index: usize,
}

impl Comb {
    pub fn new() -> Comb {
        Comb {
            feedback: 0.0,
            damp1: 0.0,
            damp2: 1.0,
            filter_store: 0.0,
            buffer: Vec::new(),
            buffer_index: 0,
        }
    }

    pub fn set_buffer_size(&mut self, size: usize) {
        let size = size.max(1);
        self.buffer = vec![0.0; size];
        self.buffer_index = 0;
    }

    pub fn set_damp(&mut self, v: f32) {
        self.damp1 = v;
        self.damp2 = 1.0 - v;
    }

    pub fn set_feedback(&mut self, v: f32) {
        self.feedback = v;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.buffer_index];
        self.filter_store = output * self.damp2 + self.filter_store * self.damp1;
        self.buffer[self.buffer_index] = input + self.filter_store * self.feedback;
        self.buffer_index = (self.buffer_index + 1) % self.buffer.len();
        output
    }
}

// ===========================================================================
// AllPassDelay
// ===========================================================================

/// `WaveSabreCore/AllPassDelay.cpp` (used by Twister's chorusing allpasses).
#[derive(Clone, Copy, Default)]
pub struct AllPassDelay {
    a1: f32,
    zm1: f32,
}

impl AllPassDelay {
    pub fn new() -> AllPassDelay {
        AllPassDelay { a1: 0.0, zm1: 0.0 }
    }

    pub fn delay(&mut self, delay: f32) {
        self.a1 = (1.0 - delay) / (1.0 + delay);
    }

    pub fn update(&mut self, input: f32) -> f32 {
        let y = input * -self.a1 + self.zm1;
        self.zm1 = y * self.a1 + input;
        y
    }
}