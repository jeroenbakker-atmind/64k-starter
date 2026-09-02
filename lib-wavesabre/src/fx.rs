//! Effect devices ported from `WaveSabreCore`: Scissor, Crusher, Echo,
//! Leveller, Smasher, Chamber, Cathedral, Twister.
//!
//! Each effect mutates the stereo sample in place (`next`), mirroring how the
//! core effects overwrite the per-sample output buffer, and implements
//! `set_param` exactly like the corresponding `SetParam` switch.

use super::filters::{
    AllPass, AllPassDelay, BiquadFilter, BiquadType, Comb, DelayBuffer, ResampleBuffer,
    StateVariableFilter, SvfType,
};
use super::helpers;

// ===========================================================================
// Scissor (waveshaper)
// ===========================================================================

#[derive(Clone, Copy)]
enum ShaperType {
    Clipper = 0,
    Sine = 1,
    Parabola = 2,
}

/// `WaveSabreCore/Scissor.cpp`.
pub struct Scissor {
    ty: ShaperType,
    drive: f32,
    threshold: f32,
    foldover: f32,
    dry_wet: f32,
    oversampling: usize,
    last: [f32; 2],
}

impl Scissor {
    pub fn new(chunk: &[u8]) -> Scissor {
        let mut s = Scissor {
            ty: ShaperType::Clipper,
            drive: 0.2,
            threshold: 0.8,
            foldover: 0.0,
            dry_wet: 1.0,
            oversampling: 0,
            last: [0.0; 2],
        };
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.drive = value,
            1 => self.threshold = value,
            2 => self.foldover = value,
            3 => self.dry_wet = value,
            4 => self.ty = match ((value * 2.0) as usize) % 3 {
                0 => ShaperType::Clipper,
                1 => ShaperType::Sine,
                _ => ShaperType::Parabola,
            },
            5 => self.oversampling = ((value * 2.0) as usize) % 3,
            _ => {}
        }
    }

    fn distort(&self, v: f32, drive_scalar: f32) -> f32 {
        let mut v = v / self.threshold;
        v *= drive_scalar;
        match self.ty {
            ShaperType::Clipper => {
                if self.foldover > 0.0 {
                    if v < -1.0 {
                        v = -1.0 + (-1.0 - v) * self.foldover;
                    } else if v > 1.0 {
                        v = 1.0 + (1.0 - v) * self.foldover;
                    }
                }
            }
            ShaperType::Sine => v *= 3.141592 / 2.0,
            ShaperType::Parabola => v = v * v * v,
        }
        if v < -1.0 {
            v = -1.0;
        } else if v > 1.0 {
            v = 1.0;
        }
        v * self.threshold
    }

    pub fn next(&mut self, _sr: f64, l: &mut f32, r: &mut f32) {
        let drive_scalar = if self.drive < 0.2 {
            1.0 - (0.2 - self.drive)
        } else {
            1.0 + helpers::pow2f((self.drive - 0.2) * 5.0) * 5.0
        };
        for ch in 0..2 {
            let input = if ch == 0 { *l } else { *r };
            let mut v = self.distort(input, drive_scalar);
            match self.oversampling {
                1 => {
                    let input_mid = (self.last[ch] + input) * 0.5;
                    let v_mid = self.distort(input_mid, drive_scalar);
                    v = (v_mid + v) * 0.5;
                }
                2 => {
                    let input_mid = (self.last[ch] + input) * 0.5;
                    let input_q1 = (self.last[ch] + input_mid) * 0.5;
                    let input_q2 = (input_mid + input) * 0.5;
                    let v_q1 = self.distort(input_q1, drive_scalar);
                    let v_mid = self.distort(input_mid, drive_scalar);
                    let v_q2 = self.distort(input_q2, drive_scalar);
                    v = (v_q1 + v_mid + v_q2 + v) * 0.25;
                }
                _ => {}
            }
            let out = helpers::mix(input, v, self.dry_wet);
            if ch == 0 {
                *l = out;
            } else {
                *r = out;
            }
            self.last[ch] = input;
        }
    }
}

// ===========================================================================
// Crusher (bit/sample-rate reduction)
// ===========================================================================

/// `WaveSabreCore/Crusher.cpp`.
pub struct Crusher {
    vertical: f32,
    horizontal: f32,
    dry_wet: f32,
    phase: [f32; 2],
    hold: [f32; 2],
}

impl Crusher {
    pub fn new(chunk: &[u8]) -> Crusher {
        let mut s = Crusher {
            vertical: 0.0,
            horizontal: 0.0,
            dry_wet: 1.0,
            phase: [0.0; 2],
            hold: [0.0; 2],
        };
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.vertical = value,
            1 => self.horizontal = value,
            2 => self.dry_wet = value,
            _ => {}
        }
    }

    pub fn next(&mut self, _sr: f64, l: &mut f32, r: &mut f32) {
        let step = 1.0 / helpers::pow2f((1.0 - self.vertical) * 15.0 + 1.0);
        let freq = helpers::pow2f(1.0 - self.horizontal);
        let inputs = [*l, *r];
        let mut outs = [*l, *r];
        for ch in 0..2 {
            let input = inputs[ch];
            self.phase[ch] += freq;
            if self.phase[ch] >= 1.0 {
                self.phase[ch] -= 1.0;
                self.hold[ch] = (input / step + 0.5).floor() * step;
            }
            outs[ch] = helpers::mix(input, self.hold[ch], self.dry_wet);
        }
        *l = outs[0];
        *r = outs[1];
    }
}

// ===========================================================================
// Echo (stereo delay)
// ===========================================================================

/// `WaveSabreCore/Echo.cpp`.
pub struct Echo {
    left_delay_coarse: f32,
    left_delay_fine: f32,
    right_delay_coarse: f32,
    right_delay_fine: f32,
    low_cut_freq: f32,
    high_cut_freq: f32,
    feedback: f32,
    cross: f32,
    dry_wet: f32,
    tempo: f64,
    left_buffer: DelayBuffer,
    right_buffer: DelayBuffer,
    low_cut: [StateVariableFilter; 2],
    high_cut: [StateVariableFilter; 2],
}

impl Echo {
    pub fn new(chunk: &[u8], tempo: f64) -> Echo {
        let mut s = Echo {
            left_delay_coarse: 3.0,
            left_delay_fine: 0.0,
            right_delay_coarse: 4.0,
            right_delay_fine: 0.0,
            low_cut_freq: 20.0,
            high_cut_freq: 19980.0,
            feedback: 0.5,
            cross: 0.0,
            dry_wet: 0.5,
            tempo,
            left_buffer: DelayBuffer::new(),
            right_buffer: DelayBuffer::new(),
            low_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
            high_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
        };
        s.low_cut[0].set_type(SvfType::Highpass);
        s.low_cut[1].set_type(SvfType::Highpass);
        s.high_cut[0].set_type(SvfType::Lowpass);
        s.high_cut[1].set_type(SvfType::Lowpass);
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.left_delay_coarse = value,
            1 => self.left_delay_fine = value,
            2 => self.right_delay_coarse = value,
            3 => self.right_delay_fine = value,
            4 => self.low_cut_freq = helpers::param_to_frequency(value),
            5 => self.high_cut_freq = helpers::param_to_frequency(value),
            6 => self.feedback = value,
            7 => self.cross = value,
            8 => self.dry_wet = value,
            _ => {}
        }
    }

    pub fn next(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        let delay_scalar = 120.0 / self.tempo.max(1.0) / 8.0 * 1000.0;
        let left_ms = self.left_delay_coarse as f64 * delay_scalar + self.left_delay_fine as f64;
        let right_ms = self.right_delay_coarse as f64 * delay_scalar + self.right_delay_fine as f64;
        self.left_buffer.set_length(left_ms as f32, sr);
        self.right_buffer.set_length(right_ms as f32, sr);

        self.low_cut[0].set_freq(self.low_cut_freq);
        self.low_cut[1].set_freq(self.low_cut_freq);
        self.high_cut[0].set_freq(self.high_cut_freq);
        self.high_cut[1].set_freq(self.high_cut_freq);

        let left_input = *l;
        let right_input = *r;

        let left_delay =
            self.low_cut[0].next(self.high_cut[0].next(self.left_buffer.read_sample(), sr), sr);
        let right_delay =
            self.low_cut[1].next(self.high_cut[1].next(self.right_buffer.read_sample(), sr), sr);

        let left_feed = (left_delay * (1.0 - self.cross) + right_delay * self.cross) * self.feedback;
        let right_feed = (right_delay * (1.0 - self.cross) + left_delay * self.cross) * self.feedback;
        self.left_buffer
            .write_sample(left_input + left_feed);
        self.right_buffer
            .write_sample(right_input + right_feed);

        *l = left_input * (1.0 - self.dry_wet) + left_delay * self.dry_wet;
        *r = right_input * (1.0 - self.dry_wet) + right_delay * self.dry_wet;
    }
}

// ===========================================================================
// Leveller (biquad EQ)
// ===========================================================================

/// `WaveSabreCore/Leveller.cpp`.
pub struct Leveller {
    low_cut_freq: f32,
    low_cut_q: f32,
    peak1_freq: f32,
    peak1_gain: f32,
    peak1_q: f32,
    peak2_freq: f32,
    peak2_gain: f32,
    peak2_q: f32,
    peak3_freq: f32,
    peak3_gain: f32,
    peak3_q: f32,
    high_cut_freq: f32,
    high_cut_q: f32,
    master: f32,
    highpass: [BiquadFilter; 2],
    peak1: [BiquadFilter; 2],
    peak2: [BiquadFilter; 2],
    peak3: [BiquadFilter; 2],
    lowpass: [BiquadFilter; 2],
}

impl Leveller {
    pub fn new(chunk: &[u8]) -> Leveller {
        let mut s = Leveller {
            low_cut_freq: 20.0,
            low_cut_q: 1.0,
            peak1_freq: 1000.0,
            peak1_gain: 0.0,
            peak1_q: 1.0,
            peak2_freq: 3000.0,
            peak2_gain: 0.0,
            peak2_q: 1.0,
            peak3_freq: 7000.0,
            peak3_gain: 0.0,
            peak3_q: 1.0,
            high_cut_freq: 20000.0,
            high_cut_q: 1.0,
            master: 1.0,
            highpass: [BiquadFilter::new(), BiquadFilter::new()],
            peak1: [BiquadFilter::new(), BiquadFilter::new()],
            peak2: [BiquadFilter::new(), BiquadFilter::new()],
            peak3: [BiquadFilter::new(), BiquadFilter::new()],
            lowpass: [BiquadFilter::new(), BiquadFilter::new()],
        };
        for i in 0..2 {
            s.highpass[i].set_type(BiquadType::Highpass);
            s.peak1[i].set_type(BiquadType::Peak);
            s.peak2[i].set_type(BiquadType::Peak);
            s.peak3[i].set_type(BiquadType::Peak);
        }
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.low_cut_freq = helpers::param_to_frequency(value),
            1 => self.low_cut_q = helpers::param_to_q(value),
            2 => self.peak1_freq = helpers::param_to_frequency(value),
            3 => self.peak1_gain = helpers::param_to_db(value, 12.0),
            4 => self.peak1_q = helpers::param_to_q(value),
            5 => self.peak2_freq = helpers::param_to_frequency(value),
            6 => self.peak2_gain = helpers::param_to_db(value, 12.0),
            7 => self.peak2_q = helpers::param_to_q(value),
            8 => self.peak3_freq = helpers::param_to_frequency(value),
            9 => self.peak3_gain = helpers::param_to_db(value, 12.0),
            10 => self.peak3_q = helpers::param_to_q(value),
            11 => self.high_cut_freq = helpers::param_to_frequency(value),
            12 => self.high_cut_q = helpers::param_to_q(value),
            13 => self.master = value,
            _ => {}
        }
    }

    pub fn next(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        for i in 0..2 {
            self.highpass[i].set_freq(self.low_cut_freq);
            self.highpass[i].set_q(self.low_cut_q);
            self.lowpass[i].set_freq(self.high_cut_freq);
            self.lowpass[i].set_q(self.high_cut_q);
            self.peak1[i].set_freq(self.peak1_freq);
            self.peak1[i].set_gain(self.peak1_gain);
            self.peak1[i].set_q(self.peak1_q);
            self.peak2[i].set_freq(self.peak2_freq);
            self.peak2[i].set_gain(self.peak2_gain);
            self.peak2[i].set_q(self.peak2_q);
            self.peak3[i].set_freq(self.peak3_freq);
            self.peak3[i].set_gain(self.peak3_gain);
            self.peak3[i].set_q(self.peak3_q);

            let samples = [*l, *r];
            let mut sample = samples[i];
            sample = self.highpass[i].next(sample, sr);
            if self.peak1_gain != 0.0 {
                sample = self.peak1[i].next(sample, sr);
            }
            if self.peak2_gain != 0.0 {
                sample = self.peak2[i].next(sample, sr);
            }
            if self.peak3_gain != 0.0 {
                sample = self.peak3[i].next(sample, sr);
            }
            sample = self.lowpass[i].next(sample, sr);
            let sample = sample * self.master;
            if i == 0 {
                *l = sample;
            } else {
                *r = sample;
            }
        }
    }
}

// ===========================================================================
// Smasher (compressor / limiter)
// ===========================================================================

/// `WaveSabreCore/Smasher.cpp` (sidechain unsupported -> normal detection).
pub struct Smasher {
    _sidechain: bool,
    input_gain: f32,
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    output_gain: f32,
    peak: f32,
    left_buffer: DelayBuffer,
    right_buffer: DelayBuffer,
}

impl Smasher {
    pub fn new(chunk: &[u8]) -> Smasher {
        let mut s = Smasher {
            _sidechain: false,
            input_gain: 0.0,
            threshold: 0.0,
            ratio: 2.0,
            attack: 1.0,
            release: 200.0,
            output_gain: 0.0,
            peak: 0.0,
            left_buffer: DelayBuffer::new(),
            right_buffer: DelayBuffer::new(),
        };
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self._sidechain = helpers::param_to_boolean(value),
            1 => self.input_gain = helpers::param_to_db(value, 12.0),
            2 => self.threshold = helpers::param_to_db(value / 2.0, 36.0),
            3 => self.ratio = value * value * 18.0 + 2.0,
            4 => self.attack = helpers::scalar_to_env_value(value) / 5.0,
            5 => self.release = helpers::scalar_to_env_value(value),
            6 => self.output_gain = helpers::param_to_db(value, 12.0),
            _ => {}
        }
    }

    pub fn next(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        self.left_buffer.set_length(2.0, sr);
        self.right_buffer.set_length(2.0, sr);

        let input_gain_scalar = helpers::db_to_scalar(self.input_gain);
        let output_gain_scalar = helpers::db_to_scalar(self.output_gain);

        let env_coeff = (1000.0 / sr) as f32;
        let attack_scalar = env_coeff / self.attack;
        let release_scalar = env_coeff / self.release;
        let threshold_scalar = helpers::db_to_scalar(self.threshold);

        self.left_buffer.write_sample(*l * input_gain_scalar);
        self.right_buffer.write_sample(*r * input_gain_scalar);
        let input_left = *l * input_gain_scalar;
        let input_right = *r * input_gain_scalar;
        let input_left_level = input_left.abs();
        let input_right_level = input_right.abs();
        let input_level = input_left_level.max(input_right_level);

        if input_level > self.peak {
            self.peak += attack_scalar;
            if self.peak > input_level {
                self.peak = input_level;
            }
        } else {
            self.peak -= release_scalar;
            if self.peak < input_level {
                self.peak = input_level;
            }
        }

        let mut gain_scalar = output_gain_scalar;
        if self.peak > threshold_scalar {
            gain_scalar *= (threshold_scalar + (self.peak - threshold_scalar) / self.ratio) / self.peak;
        }

        *l = self.left_buffer.read_sample() * gain_scalar;
        *r = self.right_buffer.read_sample() * gain_scalar;
    }
}

// ===========================================================================
// Chamber (multi-tap feedback delay reverb)
// ===========================================================================

const CHAMBER_DELAYS: [f32; 8] = [7.0, 21.0, 17.0, 13.0, 3.0, 11.0, 23.0, 31.0];
const CHAMBER_MULTIPLIERS: [f32; 3] = [1.0, 5.0, 10.0];

/// `WaveSabreCore/Chamber.cpp`.
pub struct Chamber {
    mode: usize,
    feedback: f32,
    low_cut_freq: f32,
    high_cut_freq: f32,
    dry_wet: f32,
    pre_delay: f32,
    delay_buffers: [DelayBuffer; 8],
    pre_delay_l: DelayBuffer,
    pre_delay_r: DelayBuffer,
    low_cut: [StateVariableFilter; 2],
    high_cut: [StateVariableFilter; 2],
}

impl Chamber {
    pub fn new(chunk: &[u8]) -> Chamber {
        let mut s = Chamber {
            mode: 1,
            feedback: 0.88,
            low_cut_freq: 200.0,
            high_cut_freq: 8000.0,
            dry_wet: 0.27,
            pre_delay: 0.0,
            delay_buffers: Default::default(),
            pre_delay_l: DelayBuffer::new(),
            pre_delay_r: DelayBuffer::new(),
            low_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
            high_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
        };
        s.low_cut[0].set_type(SvfType::Highpass);
        s.low_cut[1].set_type(SvfType::Highpass);
        s.high_cut[0].set_type(SvfType::Lowpass);
        s.high_cut[1].set_type(SvfType::Lowpass);
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.mode = (value * 2.0) as usize % 3,
            1 => self.feedback = value * 0.5 + 0.5,
            2 => self.low_cut_freq = helpers::param_to_frequency(value),
            3 => self.high_cut_freq = helpers::param_to_frequency(value),
            4 => self.dry_wet = value,
            5 => self.pre_delay = value,
            _ => {}
        }
    }

    pub fn next(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        for i in 0..8 {
            let ms = CHAMBER_DELAYS[i] * CHAMBER_MULTIPLIERS[self.mode];
            self.delay_buffers[i].set_length(ms, sr);
        }
        self.pre_delay_l.set_length(self.pre_delay * 500.0, sr);
        self.pre_delay_r.set_length(self.pre_delay * 500.0, sr);

        self.low_cut[0].set_freq(self.low_cut_freq);
        self.low_cut[1].set_freq(self.low_cut_freq);
        self.high_cut[0].set_freq(self.high_cut_freq);
        self.high_cut[1].set_freq(self.high_cut_freq);

        let left_input = *l;
        let right_input = *r;

        let filtered_l = if self.pre_delay > 0.0 {
            self.pre_delay_l.write_sample(left_input);
            self.low_cut[0].next(self.high_cut[0].next(self.pre_delay_l.read_sample(), sr), sr)
        } else {
            self.low_cut[0].next(self.high_cut[0].next(left_input, sr), sr)
        };
        let filtered_r = if self.pre_delay > 0.0 {
            self.pre_delay_r.write_sample(right_input);
            self.low_cut[1].next(self.high_cut[1].next(self.pre_delay_r.read_sample(), sr), sr)
        } else {
            self.low_cut[1].next(self.high_cut[1].next(right_input, sr), sr)
        };

        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;
        for j in 0..8 {
            let channel_index = if j < 4 { 0 } else { 1 };
            let feedback_sample = self.delay_buffers[7 - j].read_sample();
            let filtered = if channel_index == 0 {
                filtered_l
            } else {
                filtered_r
            };
            self.delay_buffers[j].write_sample(filtered + feedback_sample * self.feedback);
            let input = if channel_index == 0 { left_input } else { right_input };
            let out = input * (1.0 - self.dry_wet) + feedback_sample * self.dry_wet;
            if channel_index == 0 {
                out_l += out;
            } else {
                out_r += out;
            }
        }
        out_l /= 4.0;
        out_r /= 4.0;
        *l = out_l;
        *r = out_r;
    }
}

// ===========================================================================
// Cathedral (Freeverb-style reverb)
// ===========================================================================

/// `WaveSabreCore/Cathedral.cpp`.
pub struct Cathedral {
    freeze: bool,
    room_size: f32,
    damp: f32,
    width: f32,
    low_cut_freq: f32,
    high_cut_freq: f32,
    dry_wet: f32,
    pre_delay: f32,
    room_size1: f32,
    damp1: f32,
    wet1: f32,
    wet2: f32,
    gain: f32,
    low_cut: [StateVariableFilter; 2],
    high_cut: [StateVariableFilter; 2],
    comb_left: [Comb; 8],
    comb_right: [Comb; 8],
    all_pass_left: [AllPass; 4],
    all_pass_right: [AllPass; 4],
    pre_delay_buffer: DelayBuffer,
}

impl Cathedral {
    pub fn new(chunk: &[u8]) -> Cathedral {
        const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
        const STEREO_SPREAD: usize = 23;

        let mut s = Cathedral {
            freeze: false,
            room_size: 0.5,
            damp: 0.0,
            width: 1.0,
            low_cut_freq: 20.0,
            high_cut_freq: 19980.0,
            dry_wet: 0.25,
            pre_delay: 0.0,
            room_size1: 0.5,
            damp1: 0.0,
            wet1: 1.0,
            wet2: 0.0,
            gain: 0.015,
            low_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
            high_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
            comb_left: Default::default(),
            comb_right: Default::default(),
            all_pass_left: Default::default(),
            all_pass_right: Default::default(),
            pre_delay_buffer: DelayBuffer::new(),
        };
        s.low_cut[0].set_type(SvfType::Highpass);
        s.low_cut[1].set_type(SvfType::Highpass);
        s.high_cut[0].set_type(SvfType::Lowpass);
        s.high_cut[1].set_type(SvfType::Lowpass);

        for i in 0..8 {
            s.comb_left[i].set_buffer_size(COMB_TUNING[i]);
            s.comb_right[i].set_buffer_size(COMB_TUNING[i] + STEREO_SPREAD);
        }
        for i in 0..4 {
            s.all_pass_left[i].set_buffer_size(ALLPASS_TUNING[i]);
            s.all_pass_right[i].set_buffer_size(ALLPASS_TUNING[i] + STEREO_SPREAD);
            s.all_pass_left[i].set_feedback(s.room_size);
            s.all_pass_right[i].set_feedback(s.room_size);
        }
        s.update_params();
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    fn update_params(&mut self) {
        self.wet1 = self.width / 2.0 + 0.5;
        self.wet2 = (1.0 - self.width) / 2.0;
        if self.freeze {
            self.room_size1 = 1.0;
            self.damp1 = 0.0;
            self.gain = 0.0;
        } else {
            self.room_size1 = self.room_size;
            self.damp1 = self.damp;
            self.gain = 0.015;
        }
        for i in 0..8 {
            self.comb_left[i].set_feedback(self.room_size1);
            self.comb_right[i].set_feedback(self.room_size1);
            self.comb_left[i].set_damp(self.damp1);
            self.comb_right[i].set_damp(self.damp1);
        }
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => {
                self.freeze = helpers::param_to_boolean(value);
                self.update_params();
            }
            1 => {
                self.room_size = value;
                self.update_params();
            }
            2 => {
                self.damp = value;
                self.update_params();
            }
            3 => {
                self.width = value;
                self.update_params();
            }
            4 => self.low_cut_freq = helpers::param_to_frequency(value),
            5 => self.high_cut_freq = helpers::param_to_frequency(value),
            6 => self.dry_wet = value,
            7 => self.pre_delay = value,
            _ => {}
        }
    }

    pub fn next(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        for i in 0..2 {
            self.low_cut[i].set_freq(self.low_cut_freq);
            self.high_cut[i].set_freq(self.high_cut_freq);
        }
        self.pre_delay_buffer.set_length(self.pre_delay * 500.0, sr);

        let left_input = *l;
        let right_input = *r;
        let mut input = (left_input + right_input) * self.gain;

        if self.pre_delay > 0.0 {
            self.pre_delay_buffer.write_sample(input);
            input = self.pre_delay_buffer.read_sample();
        }

        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;
        for i in 0..8 {
            out_l += self.comb_left[i].process(input);
            out_r += self.comb_right[i].process(input);
        }
        for i in 0..4 {
            out_l = self.all_pass_left[i].process(out_l);
            out_r = self.all_pass_right[i].process(out_r);
        }

        out_l = self.low_cut[0].next(self.high_cut[0].next(out_l, sr), sr);
        out_r = self.low_cut[1].next(self.high_cut[1].next(out_r, sr), sr);

        let cross_l = out_l * self.wet1 + out_r * self.wet2;
        let cross_r = out_r * self.wet1 + out_l * self.wet2;

        *l = left_input * (1.0 - self.dry_wet) + cross_l * self.dry_wet;
        *r = right_input * (1.0 - self.dry_wet) + cross_r * self.dry_wet;
    }
}

// ===========================================================================
// Twister (flanger / chorus)
// ===========================================================================

/// `WaveSabreCore/Twister.cpp`.
pub struct Twister {
    ty: usize,
    amount: f32,
    feedback: f32,
    spread: usize,
    vibrato_freq: f64,
    vibrato_amount: f32,
    vibrato_phase: f64,
    low_cut_freq: f32,
    high_cut_freq: f32,
    dry_wet: f32,
    last_left: f32,
    last_right: f32,
    all_pass_left: [AllPassDelay; 6],
    all_pass_right: [AllPassDelay; 6],
    left_buffer: ResampleBuffer,
    right_buffer: ResampleBuffer,
    low_cut: [StateVariableFilter; 2],
    high_cut: [StateVariableFilter; 2],
}

impl Twister {
    pub fn new(chunk: &[u8]) -> Twister {
        let mut s = Twister {
            ty: 0,
            amount: 0.0,
            feedback: 0.0,
            spread: 0,
            vibrato_freq: helpers::param_to_vibrato_freq(0.0),
            vibrato_amount: 0.0,
            vibrato_phase: 0.0,
            low_cut_freq: 20.0,
            high_cut_freq: 19980.0,
            dry_wet: 0.5,
            last_left: 0.0,
            last_right: 0.0,
            all_pass_left: [AllPassDelay::new(); 6],
            all_pass_right: [AllPassDelay::new(); 6],
            left_buffer: ResampleBuffer::new(),
            right_buffer: ResampleBuffer::new(),
            low_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
            high_cut: [StateVariableFilter::new(), StateVariableFilter::new()],
        };
        s.low_cut[0].set_type(SvfType::Highpass);
        s.low_cut[1].set_type(SvfType::Highpass);
        s.high_cut[0].set_type(SvfType::Lowpass);
        s.high_cut[1].set_type(SvfType::Lowpass);
        s.left_buffer.set_length(1000.0, 44100.0);
        s.right_buffer.set_length(1000.0, 44100.0);
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.ty = ((value * 3.0) as usize) % 4,
            1 => self.amount = value,
            2 => self.feedback = value,
            3 => self.spread = helpers::param_to_spread(value) as usize,
            4 => self.vibrato_freq = helpers::param_to_vibrato_freq(value),
            5 => self.vibrato_amount = value,
            6 => self.low_cut_freq = helpers::param_to_frequency(value),
            7 => self.high_cut_freq = helpers::param_to_frequency(value),
            8 => self.dry_wet = value,
            _ => {}
        }
    }

    fn all_pass_update_left(&mut self, input: f32) -> f32 {
        let mut x = input;
        for ap in self.all_pass_left.iter_mut() {
            x = ap.update(x);
        }
        x
    }

    fn all_pass_update_right(&mut self, input: f32) -> f32 {
        let mut x = input;
        for ap in self.all_pass_right.iter_mut() {
            x = ap.update(x);
        }
        x
    }

    pub fn next(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        let vibrato_delta = (self.vibrato_freq / sr) * 0.25;

        for i in 0..2 {
            self.low_cut[i].set_freq(self.low_cut_freq);
            self.high_cut[i].set_freq(self.high_cut_freq);
        }

        let left_input = *l;
        let right_input = *r;

        let freq = self.vibrato_phase.sin() * self.vibrato_amount as f64;
        let (mut pos_l, mut pos_r) = match self.spread {
            1 => {
                let p = helpers::clamp(self.amount + freq as f32, 0.0, 1.0);
                (p, 1.0 - p)
            }
            2 => (
                helpers::clamp(self.amount + freq as f32, 0.0, 1.0),
                helpers::clamp(self.amount - freq as f32, 0.0, 1.0),
            ),
            _ => {
                let p = helpers::clamp(self.amount + freq as f32, 0.0, 1.0);
                (p, p)
            }
        };

        let out_l;
        let out_r;
        match self.ty {
            0 | 1 => {
                pos_l *= 132.0;
                pos_r *= 132.0;
                out_l = self
                    .high_cut[0]
                    .next(self.low_cut[0].next(self.left_buffer.read_position(pos_l + 2.0), sr), sr);
                out_r = self
                    .high_cut[1]
                    .next(self.low_cut[1].next(self.right_buffer.read_position(pos_r + 2.0), sr), sr);
                if self.ty == 0 {
                    self.left_buffer
                        .write_sample(left_input + out_l * self.feedback);
                    self.right_buffer
                        .write_sample(right_input + out_r * self.feedback);
                } else {
                    self.left_buffer
                        .write_sample(left_input - out_l * self.feedback);
                    self.right_buffer
                        .write_sample(right_input - out_r * self.feedback);
                }
            }
            2 | 3 => {
                for ap in self.all_pass_left.iter_mut() {
                    ap.delay(pos_l);
                }
                for ap in self.all_pass_right.iter_mut() {
                    ap.delay(pos_r);
                }
                let ap_l = self.all_pass_update_left(
                    left_input + if self.ty == 2 { self.last_left } else { -self.last_left } * self.feedback,
                );
                let ap_r = self.all_pass_update_right(
                    right_input + if self.ty == 2 { self.last_right } else { -self.last_right } * self.feedback,
                );
                out_l = self.high_cut[0].next(self.low_cut[0].next(ap_l, sr), sr);
                out_r = self.high_cut[1].next(self.low_cut[1].next(ap_r, sr), sr);
                self.last_left = out_l;
                self.last_right = out_r;
            }
            _ => {
                out_l = 0.0;
                out_r = 0.0;
            }
        }

        *l = left_input * (1.0 - self.dry_wet) + out_l * self.dry_wet;
        *r = right_input * (1.0 - self.dry_wet) + out_r * self.dry_wet;

        self.vibrato_phase += vibrato_delta;
    }
}