//! Faithful port of `WaveSabreCore/Falcon.cpp` (2-operator FM synth),
//! including the full `SynthDevice` behavior (unisono, mono-legato, slide) and
//! per-voice panning.

use std::f64::consts::TAU;

use super::envelope::Envelope;
use super::helpers;
use super::voice::{SynthCore, SynthVoice, VoiceBase};

#[derive(Clone, Copy, Default)]
pub struct FalconCtx {
    ratio1: f64,
    ratio2: f64,
    fb1: f64,
    fb2: f64,
    ff: f32,
    master: f32,
    vib_per_sample: f64,
    vib_amount: f32,
    pamt1: f64,
    pamt2: f64,
    osc1_waveform: f64,
    osc2_waveform: f64,
    rise: f64,
}

/// `Falcon::FalconVoice`.
pub struct FalconVoice {
    base: VoiceBase,
    ctx: FalconCtx,
    osc1_phase: f64,
    osc2_phase: f64,
    osc1_output: f64,
    osc2_output: f64,
    osc1_env: Envelope,
    osc2_env: Envelope,
    pitch_env: Envelope,
}

impl FalconVoice {
    fn new(ctx: FalconCtx) -> FalconVoice {
        FalconVoice {
            base: VoiceBase::new(),
            ctx,
            osc1_phase: 0.0,
            osc2_phase: 0.0,
            osc1_output: 0.0,
            osc2_output: 0.0,
            osc1_env: Envelope::new(),
            osc2_env: Envelope::new(),
            pitch_env: Envelope::new(),
        }
    }
}

impl SynthVoice for FalconVoice {
    fn base(&self) -> &VoiceBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut VoiceBase {
        &mut self.base
    }

    fn note_off(&mut self) {
        self.osc1_env.off();
        self.osc2_env.off();
        self.pitch_env.off();
    }

    fn next_sample(&mut self, sr: f64) -> (f32, f32) {
        // baseNote = GetNote() + Detune + Rise * 24
        let base_note = self.base.get_note() + self.base.detune as f64 + self.ctx.rise;

        let osc1_input =
            self.osc1_phase / sr * TAU + self.osc1_output * self.ctx.fb1;
        self.osc1_output = (osc1_input.sin() + square35(osc1_input) * self.ctx.osc1_waveform)
            * self.osc1_env.value() as f64
            * 13.25;

        let osc2_input = self.osc2_phase / sr * TAU
            + self.osc2_output * self.ctx.fb2 * 13.25
            + self.osc1_output * self.ctx.ff as f64;
        self.osc2_output = (osc2_input.sin() + square35(osc2_input) * self.ctx.osc2_waveform)
            * self.osc2_env.value() as f64;

        let out = self.osc2_output * self.ctx.master as f64;

        let pan_l = helpers::pan_to_scalar_left(self.base.pan);
        let pan_r = helpers::pan_to_scalar_right(self.base.pan);
        let l = (out * pan_l as f64) as f32;
        let r = (out * pan_r as f64) as f32;

        self.osc2_env.next(sr);
        if self.osc2_env.state == super::envelope::EnvState::Finished {
            self.base.is_on = false;
            return (l, r);
        }

        let p_env = self.pitch_env.value();
        let vib = self.base.vibrato_phase.sin() * self.ctx.vib_amount as f64;
        let freq1 = helpers::note_to_freq(base_note + p_env as f64 * self.ctx.pamt1 + vib)
            * self.ctx.ratio1;
        let freq2 = helpers::note_to_freq(base_note + p_env as f64 * self.ctx.pamt2 + vib)
            * self.ctx.ratio2;
        self.osc1_phase += freq1;
        self.osc2_phase += freq2;
        self.base.vibrato_phase += self.ctx.vib_per_sample;
        self.osc1_env.next(sr);
        self.pitch_env.next(sr);

        (l, r)
    }
}

/// `Helpers::Square35`: third + fifth harmonic of a square, blended with the
/// fundamental by the `*_waveform` params.
fn square35(phase: f64) -> f64 {
    (phase * 3.0).sin() / 3.0 + (phase * 5.0).sin() / 5.0
}

/// `Falcon` device. Params are decoded/set exactly like `Falcon::SetParam`;
/// axis: 32 floats stored in the chunk.
pub struct FalconSynth {
    osc1_waveform: f32,
    osc1_ratio_coarse: f32,
    osc1_ratio_fine: f32,
    osc1_feedback: f32,
    osc1_feed_forward: f32,
    osc1_attack: f32,
    osc1_decay: f32,
    osc1_sustain: f32,
    osc1_release: f32,
    osc2_waveform: f32,
    osc2_ratio_coarse: f32,
    osc2_ratio_fine: f32,
    osc2_feedback: f32,
    osc2_attack: f32,
    osc2_decay: f32,
    osc2_sustain: f32,
    osc2_release: f32,
    master_level: f32,
    voices_unisono: usize,
    voices_detune: f32,
    voices_pan: f32,
    vibrato_freq: f64,
    vibrato_amount: f32,
    rise: f32,
    pitch_attack: f32,
    pitch_decay: f32,
    pitch_sustain: f32,
    pitch_release: f32,
    pitch_env_amt1: f64,
    pitch_env_amt2: f64,
    slide: f32,
    core: SynthCore,
}

impl FalconSynth {
    pub fn new(chunk: &[u8]) -> FalconSynth {
        let mut s = FalconSynth {
            osc1_waveform: 0.0,
            osc1_ratio_coarse: 0.0,
            osc1_ratio_fine: 0.5,
            osc1_feedback: 0.0,
            osc1_feed_forward: 0.0,
            osc1_attack: 1.0,
            osc1_decay: 1.0,
            osc1_sustain: 1.0,
            osc1_release: 1.0,
            osc2_waveform: 0.0,
            osc2_ratio_coarse: 0.0,
            osc2_ratio_fine: 0.5,
            osc2_feedback: 0.0,
            osc2_attack: 1.0,
            osc2_decay: 5.0,
            osc2_sustain: 0.75,
            osc2_release: 1.5,
            master_level: 0.8,
            voices_unisono: 1,
            voices_detune: 0.0,
            voices_pan: 0.5,
            vibrato_freq: helpers::param_to_vibrato_freq(0.0),
            vibrato_amount: 0.0,
            rise: 0.0,
            pitch_attack: 1.0,
            pitch_decay: 5.0,
            pitch_sustain: 0.5,
            pitch_release: 1.5,
            pitch_env_amt1: 0.0,
            pitch_env_amt2: 0.0,
            slide: 0.0,
            core: SynthCore::new(),
        };
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    /// `Falcon::SetParam`.
    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.osc1_waveform = value,
            1 => self.osc1_ratio_coarse = value,
            2 => self.osc1_ratio_fine = value,
            3 => self.osc1_feedback = value,
            4 => self.osc1_feed_forward = value,
            5 => self.osc1_attack = helpers::scalar_to_env_value(value),
            6 => self.osc1_decay = helpers::scalar_to_env_value(value),
            7 => self.osc1_sustain = value,
            8 => self.osc1_release = helpers::scalar_to_env_value(value),
            9 => self.osc2_waveform = value,
            10 => self.osc2_ratio_coarse = value,
            11 => self.osc2_ratio_fine = value,
            12 => self.osc2_feedback = value,
            13 => self.osc2_attack = helpers::scalar_to_env_value(value),
            14 => self.osc2_decay = helpers::scalar_to_env_value(value),
            15 => self.osc2_sustain = value,
            16 => self.osc2_release = helpers::scalar_to_env_value(value),
            17 => self.master_level = value,
            18 => self.voices_unisono = helpers::param_to_unisono(value),
            19 => self.voices_detune = value,
            20 => self.voices_pan = value,
            21 => self.vibrato_freq = helpers::param_to_vibrato_freq(value),
            22 => self.vibrato_amount = value,
            23 => self.rise = value,
            24 => self.pitch_attack = helpers::scalar_to_env_value(value),
            25 => self.pitch_decay = helpers::scalar_to_env_value(value),
            26 => self.pitch_sustain = value,
            27 => self.pitch_release = helpers::scalar_to_env_value(value),
            28 => self.pitch_env_amt1 = (value - 0.5) as f64 * 72.0,
            29 => self.pitch_env_amt2 = (value - 0.5) as f64 * 72.0,
            30 => {
                let mode = helpers::param_to_voice_mode(value);
                self.core.set_voice_mode(mode);
            }
            31 => self.slide = value,
            _ => {}
        }
    }

    pub fn note_on(&mut self, note: u8, vel: u8, sr: f64) {
        // Ratio scalar from `Helpers::ratioScalar`.
        let ratio = |coarse: f64, fine: f64| {
            let fine_base = (fine - 0.5) * 2.0;
            1.0 + (coarse * 32.99).floor() + fine_base * fine_base * fine_base
        };
        let ctx = FalconCtx {
            ratio1: ratio(self.osc1_ratio_coarse as f64, self.osc1_ratio_fine as f64),
            ratio2: ratio(self.osc2_ratio_coarse as f64, self.osc2_ratio_fine as f64),
            fb1: 0.5 * (self.osc1_feedback as f64) * (self.osc1_feedback as f64),
            fb2: 0.5 * (self.osc2_feedback as f64) * (self.osc2_feedback as f64),
            ff: self.osc1_feed_forward * self.osc1_feed_forward,
            master: helpers::volume_to_scalar(self.master_level),
            vib_per_sample: self.vibrato_freq / sr,
            vib_amount: self.vibrato_amount,
            pamt1: self.pitch_env_amt1,
            pamt2: self.pitch_env_amt2,
            osc1_waveform: self.osc1_waveform as f64,
            osc2_waveform: self.osc2_waveform as f64,
            rise: self.rise as f64 * 24.0,
        };
        // Copy env times + trigger, exactly like `FalconVoice::NoteOn`.
        let spawn = {
            let ctx = ctx;
            let osc1_attack = self.osc1_attack;
            let osc1_decay = self.osc1_decay;
            let osc1_sustain = self.osc1_sustain;
            let osc1_release = self.osc1_release;
            let osc2_attack = self.osc2_attack;
            let osc2_decay = self.osc2_decay;
            let osc2_sustain = self.osc2_sustain;
            let osc2_release = self.osc2_release;
            let pa = self.pitch_attack;
            let pd = self.pitch_decay;
            let ps = self.pitch_sustain;
            let pr = self.pitch_release;
            move |note: u8, detune: f32, pan: f32| -> Box<dyn SynthVoice> {
                let mut v = FalconVoice::new(ctx);
                v.base.note_on(note, detune, pan);
                v.osc1_phase = helpers::rand_float() as f64;
                v.osc2_phase = v.osc1_phase;
                v.osc1_output = 0.0;
                v.osc2_output = 0.0;
                v.osc1_env.attack = osc1_attack;
                v.osc1_env.decay = osc1_decay;
                v.osc1_env.sustain = osc1_sustain;
                v.osc1_env.release = osc1_release;
                v.osc1_env.trigger();
                v.osc2_env.attack = osc2_attack;
                v.osc2_env.decay = osc2_decay;
                v.osc2_env.sustain = osc2_sustain;
                v.osc2_env.release = osc2_release;
                v.osc2_env.trigger();
                v.pitch_env.attack = pa;
                v.pitch_env.decay = pd;
                v.pitch_env.sustain = ps;
                v.pitch_env.release = pr;
                v.pitch_env.trigger();
                Box::new(v)
            }
        };
        let unisono = self.voices_unisono;
        let detune = self.voices_detune;
        let pan = self.voices_pan;
        let slide = self.slide;
        let mut spawn = spawn;
        self.core.note_on(note, vel, unisono, detune, pan, slide, sr, &mut spawn);
    }

    pub fn note_off(&mut self, note: u8, sr: f64) {
        let slide = self.slide;
        self.core.note_off(note, slide, sr);
    }

    pub fn all_notes_off(&mut self) {
        self.core.all_notes_off();
    }

    pub fn next_sample(&mut self, sr: f64) -> (f32, f32) {
        self.core.next_sample(sr)
    }
}

impl Default for FalconSynth {
    fn default() -> Self {
        FalconSynth::new(&[])
    }
}