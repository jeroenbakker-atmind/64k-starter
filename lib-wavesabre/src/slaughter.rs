//! Faithful port of `WaveSabreCore/Slaughter.cpp` (subtractive synth with
//! three BLIT oscillators, state-variable filter and amp/mod/pitch envelopes),
//! including the full `SynthDevice` behavior.

use super::envelope::Envelope;
use super::filters::{StateVariableFilter, SvfType};
use super::helpers;
use super::voice::{SynthCore, SynthVoice, VoiceBase};
use std::f64::consts::PI;

/// `Slaughter::SlaughterVoice::Oscillator`: a poly-BLIT band-limited pulse.
struct Oscillator {
    phase: f64,
    integral: f64,
}

impl Oscillator {
    fn next(&mut self, note: f64, waveform: f32, pulse_width: f32, sr: f64) -> f32 {
        let phase_max = sr * 0.5 / helpers::note_to_freq(note);
        let dc_offset = -0.498 / phase_max;
        let pw = pulse_width as f64;
        let wf = waveform as f64;

        let phase2 =
            (self.phase + 2.0 * phase_max * pw).rem_euclid(phase_max * 2.0) - phase_max;
        self.phase = (self.phase + 1.0).rem_euclid(phase_max * 2.0);
        let tmp_phase = self.phase - phase_max;

        let eps = 0.0000001;
        let blit1 = if tmp_phase.abs() > eps {
            let t = tmp_phase * PI;
            helpers::fast_sin(t) / t
        } else {
            1.0
        };
        let blit2 = if phase2.abs() > eps {
            let t = phase2 * PI;
            helpers::fast_sin(t) / t
        } else {
            1.0
        };

        self.integral =
            0.998 * self.integral + dc_offset * (1.0 - wf) + blit1 - blit2 * wf;
        self.integral as f32
    }
}

fn coarse_detune(detune: f32) -> f64 {
    (detune * 24.99).floor() as f64
}

/// `Slaughter::SlaughterVoice`.
pub struct SlaughterVoice {
    base: VoiceBase,
    osc1: Oscillator,
    osc2: Oscillator,
    osc3: Oscillator,
    filter: StateVariableFilter,
    amp_env: Envelope,
    mod_env: Envelope,
    pitch_env: Envelope,
    // Per-note snapshots (computed from the device at note-on, like the core
    // which reads the members at the top of each Run block).
    amp: f32,
    pan_left: f32,
    pan_right: f32,
    osc1_detune: f64,
    osc2_detune: f64,
    osc3_detune: f64,
    osc1_vol: f32,
    osc2_vol: f32,
    osc3_vol: f32,
    noise_vol: f32,
    osc1_waveform: f32,
    osc1_pulse_width: f32,
    osc2_waveform: f32,
    osc2_pulse_width: f32,
    osc3_waveform: f32,
    osc3_pulse_width: f32,
    filter_type: SvfType,
    filter_res: f32,
    filter_freq: f32,
    filter_mod_amt: f32,
    pitch_env_amt: f64,
    vibrato_freq: f64,
    vibrato_amount: f32,
    rise: f64,
}

impl SlaughterVoice {
    fn new() -> SlaughterVoice {
        SlaughterVoice {
            base: VoiceBase::new(),
            osc1: Oscillator {
                phase: helpers::rand_float() as f64 * 2.0 * PI,
                integral: 0.0,
            },
            osc2: Oscillator {
                phase: helpers::rand_float() as f64 * 2.0 * PI,
                integral: 0.0,
            },
            osc3: Oscillator {
                phase: helpers::rand_float() as f64 * 2.0 * PI,
                integral: 0.0,
            },
            filter: StateVariableFilter::new(),
            amp_env: Envelope::new(),
            mod_env: Envelope::new(),
            pitch_env: Envelope::new(),
            amp: 0.0,
            pan_left: 0.0,
            pan_right: 0.0,
            osc1_detune: 0.0,
            osc2_detune: 0.0,
            osc3_detune: 0.0,
            osc1_vol: 1.0,
            osc2_vol: 0.0,
            osc3_vol: 0.0,
            noise_vol: 0.0,
            osc1_waveform: 0.0,
            osc1_pulse_width: 0.5,
            osc2_waveform: 0.0,
            osc2_pulse_width: 0.5,
            osc3_waveform: 0.0,
            osc3_pulse_width: 0.5,
            filter_type: SvfType::Lowpass,
            filter_res: 0.0,
            filter_freq: 19980.0,
            filter_mod_amt: 0.0,
            pitch_env_amt: 0.0,
            vibrato_freq: 0.0,
            vibrato_amount: 0.0,
            rise: 0.0,
        }
    }
}

impl SynthVoice for SlaughterVoice {
    fn base(&self) -> &VoiceBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut VoiceBase {
        &mut self.base
    }

    fn note_off(&mut self) {
        self.amp_env.off();
        self.mod_env.off();
        self.pitch_env.off();
    }

    fn next_sample(&mut self, sr: f64) -> (f32, f32) {
        // baseNote = GetNote() + Detune + pitchEnv * pitchEnvAmt
        //            + sin(vibratoPhase) * VibratoAmount + Rise * 24
        let asep = self.filter_mod_amt * 2.0 - 1.0;
        let cutoff = helpers::clamp(
            self.filter_freq + self.mod_env.value() * 19980.0 * asep,
            0.0,
            19980.0,
        );
        self.filter.set_freq(cutoff);

        let base_note = self.base.get_note()
            + self.base.detune as f64
            + self.pitch_env.value() as f64 * self.pitch_env_amt
            + self.base.vibrato_phase.sin() * self.vibrato_amount as f64
            + self.rise;

        let mut osc_mix = 0.0f32;
        if self.osc1_vol > 0.0 {
            osc_mix += self.osc1.next(
                base_note + self.osc1_detune,
                self.osc1_waveform,
                self.osc1_pulse_width,
                sr,
            ) * self.osc1_vol;
        }
        if self.osc2_vol > 0.0 {
            osc_mix += self.osc2.next(
                base_note + self.osc2_detune,
                self.osc2_waveform,
                self.osc2_pulse_width,
                sr,
            ) * self.osc2_vol;
        }
        if self.osc3_vol > 0.0 {
            osc_mix += self.osc3.next(
                base_note + self.osc3_detune,
                self.osc3_waveform,
                self.osc3_pulse_width,
                sr,
            ) * self.osc3_vol;
        }
        if self.noise_vol > 0.0 {
            osc_mix += helpers::rand_float() * self.noise_vol;
        }

        let out = self.filter.next(osc_mix, sr) * self.amp_env.value() * self.amp;
        let l = out * self.pan_left;
        let r = out * self.pan_right;

        self.amp_env.next(sr);
        if self.amp_env.state == super::envelope::EnvState::Finished {
            self.base.is_on = false;
            return (l, r);
        }
        self.base.vibrato_phase += self.vibrato_freq / sr;
        self.mod_env.next(sr);
        self.pitch_env.next(sr);

        (l, r)
    }
}

/// `Slaughter` device. Params follow `Slaughter::ParamIndices` (42 floats).
pub struct SlaughterSynth {
    osc1_waveform: f32,
    osc1_pulse_width: f32,
    osc1_volume: f32,
    osc1_detune_coarse: f32,
    osc1_detune_fine: f32,
    osc2_waveform: f32,
    osc2_pulse_width: f32,
    osc2_volume: f32,
    osc2_detune_coarse: f32,
    osc2_detune_fine: f32,
    osc3_waveform: f32,
    osc3_pulse_width: f32,
    osc3_volume: f32,
    osc3_detune_coarse: f32,
    osc3_detune_fine: f32,
    noise_volume: f32,
    filter_type: SvfType,
    filter_freq: f32,
    filter_resonance: f32,
    filter_mod_amt: f32,
    amp_attack: f32,
    amp_decay: f32,
    amp_sustain: f32,
    amp_release: f32,
    mod_attack: f32,
    mod_decay: f32,
    mod_sustain: f32,
    mod_release: f32,
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
    pitch_env_amt: f64,
    slide: f32,
    core: SynthCore,
}

impl SlaughterSynth {
    pub fn new(chunk: &[u8]) -> SlaughterSynth {
        let mut s = SlaughterSynth {
            osc1_waveform: 0.0,
            osc1_pulse_width: 0.5,
            osc1_volume: 1.0,
            osc1_detune_coarse: 0.0,
            osc1_detune_fine: 0.0,
            osc2_waveform: 0.0,
            osc2_pulse_width: 0.5,
            osc2_volume: 0.0,
            osc2_detune_coarse: 0.0,
            osc2_detune_fine: 0.0,
            osc3_waveform: 0.0,
            osc3_pulse_width: 0.5,
            osc3_volume: 0.0,
            osc3_detune_coarse: 0.0,
            osc3_detune_fine: 0.0,
            noise_volume: 0.0,
            filter_type: SvfType::Lowpass,
            filter_freq: 19980.0,
            filter_resonance: 0.0,
            filter_mod_amt: 0.5,
            amp_attack: 1.0,
            amp_decay: 5.0,
            amp_sustain: 0.5,
            amp_release: 1.5,
            mod_attack: 1.0,
            mod_decay: 5.0,
            mod_sustain: 1.0,
            mod_release: 1.5,
            master_level: 0.5,
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
            pitch_env_amt: 0.0,
            slide: 0.0,
            core: SynthCore::new(),
        };
        let params = common::chunk_params(chunk);
        for (i, v) in params.iter().enumerate() {
            s.set_param(i, *v);
        }
        s
    }

    /// `Slaughter::SetParam`.
    pub fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.osc1_waveform = value,
            1 => self.osc1_pulse_width = 1.0 - value,
            2 => self.osc1_volume = value,
            3 => self.osc1_detune_coarse = value,
            4 => self.osc1_detune_fine = value,
            5 => self.osc2_waveform = value,
            6 => self.osc2_pulse_width = 1.0 - value,
            7 => self.osc2_volume = value,
            8 => self.osc2_detune_coarse = value,
            9 => self.osc2_detune_fine = value,
            10 => self.osc3_waveform = value,
            11 => self.osc3_pulse_width = 1.0 - value,
            12 => self.osc3_volume = value,
            13 => self.osc3_detune_coarse = value,
            14 => self.osc3_detune_fine = value,
            15 => self.noise_volume = value,
            16 => {
                self.filter_type = match helpers::param_to_state_variable_filter_type(value) {
                    0 => SvfType::Lowpass,
                    1 => SvfType::Highpass,
                    2 => SvfType::Bandpass,
                    _ => SvfType::Notch,
                }
            }
            17 => self.filter_freq = helpers::param_to_frequency(value),
            18 => self.filter_resonance = 1.0 - value,
            19 => self.filter_mod_amt = value,
            20 => self.amp_attack = helpers::scalar_to_env_value(value),
            21 => self.amp_decay = helpers::scalar_to_env_value(value),
            22 => self.amp_sustain = value,
            23 => self.amp_release = helpers::scalar_to_env_value(value),
            24 => self.mod_attack = helpers::scalar_to_env_value(value),
            25 => self.mod_decay = helpers::scalar_to_env_value(value),
            26 => self.mod_sustain = value,
            27 => self.mod_release = helpers::scalar_to_env_value(value),
            28 => self.master_level = value,
            29 => self.voices_unisono = helpers::param_to_unisono(value),
            30 => self.voices_detune = value,
            31 => self.voices_pan = value,
            32 => self.vibrato_freq = helpers::param_to_vibrato_freq(value),
            33 => self.vibrato_amount = value,
            34 => self.rise = value,
            35 => self.pitch_attack = helpers::scalar_to_env_value(value),
            36 => self.pitch_decay = helpers::scalar_to_env_value(value),
            37 => self.pitch_sustain = value,
            38 => self.pitch_release = helpers::scalar_to_env_value(value),
            39 => self.pitch_env_amt = (value - 0.5) as f64 * 72.0,
            40 => {
                let mode = helpers::param_to_voice_mode(value);
                self.core.set_voice_mode(mode);
            }
            41 => self.slide = value,
            _ => {}
        }
    }

    pub fn note_on(&mut self, note: u8, vel: u8, sr: f64) {
        let spawn = {
            let a = self.clone_fields();
            move |note: u8, detune: f32, pan: f32| -> Box<dyn SynthVoice> {
                let mut v = SlaughterVoice::new();
                v.base.note_on(note, detune, pan);
                v.amp = -16.0 * helpers::volume_to_scalar(a.master_level);
                v.pan_left = helpers::pan_to_scalar_left(pan);
                v.pan_right = helpers::pan_to_scalar_right(pan);
                v.osc1_detune = coarse_detune(a.osc1_detune_coarse) + a.osc1_detune_fine as f64;
                v.osc2_detune = coarse_detune(a.osc2_detune_coarse) + a.osc2_detune_fine as f64;
                v.osc3_detune = coarse_detune(a.osc3_detune_coarse) + a.osc3_detune_fine as f64;
                v.osc1_vol = a.osc1_volume * a.osc1_volume;
                v.osc2_vol = a.osc2_volume * a.osc2_volume;
                v.osc3_vol = a.osc3_volume * a.osc3_volume;
                v.noise_vol = a.noise_volume * a.noise_volume;
                v.osc1_waveform = a.osc1_waveform;
                v.osc1_pulse_width = a.osc1_pulse_width;
                v.osc2_waveform = a.osc2_waveform;
                v.osc2_pulse_width = a.osc2_pulse_width;
                v.osc3_waveform = a.osc3_waveform;
                v.osc3_pulse_width = a.osc3_pulse_width;
                v.filter_type = a.filter_type;
                v.filter_res = a.filter_resonance;
                v.filter_freq = a.filter_freq;
                v.filter_mod_amt = a.filter_mod_amt;
                v.pitch_env_amt = a.pitch_env_amt;
                v.vibrato_freq = a.vibrato_freq;
                v.vibrato_amount = a.vibrato_amount;
                v.rise = a.rise as f64 * 24.0;
                v.filter.set_type(a.filter_type);
                v.filter.set_q(a.filter_resonance);
                v.amp_env.attack = a.amp_attack;
                v.amp_env.decay = a.amp_decay;
                v.amp_env.sustain = a.amp_sustain;
                v.amp_env.release = a.amp_release;
                v.amp_env.trigger();
                v.mod_env.attack = a.mod_attack;
                v.mod_env.decay = a.mod_decay;
                v.mod_env.sustain = a.mod_sustain;
                v.mod_env.release = a.mod_release;
                v.mod_env.trigger();
                v.pitch_env.attack = a.pitch_attack;
                v.pitch_env.decay = a.pitch_decay;
                v.pitch_env.sustain = a.pitch_sustain;
                v.pitch_env.release = a.pitch_release;
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

/// Copy of the device members consumed by a voice at note-on.
#[derive(Clone)]
struct SlaughterFields {
    master_level: f32,
    osc1_detune_coarse: f32,
    osc1_detune_fine: f32,
    osc2_detune_coarse: f32,
    osc2_detune_fine: f32,
    osc3_detune_coarse: f32,
    osc3_detune_fine: f32,
    osc1_volume: f32,
    osc2_volume: f32,
    osc3_volume: f32,
    noise_volume: f32,
    osc1_waveform: f32,
    osc1_pulse_width: f32,
    osc2_waveform: f32,
    osc2_pulse_width: f32,
    osc3_waveform: f32,
    osc3_pulse_width: f32,
    filter_type: SvfType,
    filter_resonance: f32,
    filter_freq: f32,
    filter_mod_amt: f32,
    pitch_env_amt: f64,
    vibrato_freq: f64,
    vibrato_amount: f32,
    rise: f32,
    amp_attack: f32,
    amp_decay: f32,
    amp_sustain: f32,
    amp_release: f32,
    mod_attack: f32,
    mod_decay: f32,
    mod_sustain: f32,
    mod_release: f32,
    pitch_attack: f32,
    pitch_decay: f32,
    pitch_sustain: f32,
    pitch_release: f32,
}

impl SlaughterSynth {
    fn clone_fields(&self) -> SlaughterFields {
        SlaughterFields {
            master_level: self.master_level,
            osc1_detune_coarse: self.osc1_detune_coarse,
            osc1_detune_fine: self.osc1_detune_fine,
            osc2_detune_coarse: self.osc2_detune_coarse,
            osc2_detune_fine: self.osc2_detune_fine,
            osc3_detune_coarse: self.osc3_detune_coarse,
            osc3_detune_fine: self.osc3_detune_fine,
            osc1_volume: self.osc1_volume,
            osc2_volume: self.osc2_volume,
            osc3_volume: self.osc3_volume,
            noise_volume: self.noise_volume,
            osc1_waveform: self.osc1_waveform,
            osc1_pulse_width: self.osc1_pulse_width,
            osc2_waveform: self.osc2_waveform,
            osc2_pulse_width: self.osc2_pulse_width,
            osc3_waveform: self.osc3_waveform,
            osc3_pulse_width: self.osc3_pulse_width,
            filter_type: self.filter_type,
            filter_resonance: self.filter_resonance,
            filter_freq: self.filter_freq,
            filter_mod_amt: self.filter_mod_amt,
            pitch_env_amt: self.pitch_env_amt,
            vibrato_freq: self.vibrato_freq,
            vibrato_amount: self.vibrato_amount,
            rise: self.rise,
            amp_attack: self.amp_attack,
            amp_decay: self.amp_decay,
            amp_sustain: self.amp_sustain,
            amp_release: self.amp_release,
            mod_attack: self.mod_attack,
            mod_decay: self.mod_decay,
            mod_sustain: self.mod_sustain,
            mod_release: self.mod_release,
            pitch_attack: self.pitch_attack,
            pitch_decay: self.pitch_decay,
            pitch_sustain: self.pitch_sustain,
            pitch_release: self.pitch_release,
        }
    }
}

impl Default for SlaughterSynth {
    fn default() -> Self {
        SlaughterSynth::new(&[])
    }
}