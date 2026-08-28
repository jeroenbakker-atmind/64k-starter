//! A tiny, platform-independent software renderer that plays a parsed song,
//! used by `examples/create_song --wav` and `examples/instrument_test --wav`
//! to preview a song on non-Windows machines.
//!
//! Tracks whose device is a WaveSabre `Falcon` (the 2-operator FM synth every
//! song builder uses by default) are rendered with a faithful port of
//! `WaveSabreCore/Falcon.cpp` (plus `Envelope.cpp` / `Helpers.cpp`), driven by
//! the actual serialized patch parameters. The export therefore matches the
//! Falcon instruments: FM index, feedback, oscillator ratios and waveform
//! blend, ADSR, pitch envelope and vibrato all follow the C++ core.
//!
//! Tracks using other devices (e.g. `Adultery` GM samples) fall back to simple
//! sine/saw/box oscillators, since their samples cannot be synthesized.

use crate::format::{DeviceId, ParsedSong};
use std::f64::consts::TAU;
use std::fs;
use std::path::Path;
use std::vec::Vec;

// ===========================================================================
// Falcon: faithful 2-operator FM renderer
// ===========================================================================

const FALCON_PARAMS: usize = 32;
const MAX_VOICES: usize = 256;

/// Raw Falcon params (indices 0..30 of `Falcon::ParamIndices`), exactly as
/// stored in the device chunk by `Falcon::to_params`. Value semantics are
/// documented in `src-song/format.rs`; the gain / ratio / envelope conversions
/// below mirror `Falcon.cpp`'s `SetParam`.
#[derive(Clone, Copy)]
struct FalconParams {
    osc1_waveform: f64,
    osc1_ratio_coarse: f64,
    osc1_ratio_fine: f64,
    osc1_feedback: f64,
    osc1_feed_forward: f64,
    osc1_attack: f64,
    osc1_decay: f64,
    osc1_sustain: f64,
    osc1_release: f64,
    osc2_waveform: f64,
    osc2_ratio_coarse: f64,
    osc2_ratio_fine: f64,
    osc2_feedback: f64,
    osc2_attack: f64,
    osc2_decay: f64,
    osc2_sustain: f64,
    osc2_release: f64,
    master_level: f64,
    vibrato_freq: f64,
    vibrato_amount: f64,
    rise: f64,
    pitch_attack: f64,
    pitch_decay: f64,
    pitch_sustain: f64,
    pitch_release: f64,
    pitch_env_amt1: f64,
    pitch_env_amt2: f64,
}

/// Decodes a Falcon device chunk into its params.
fn falcon_params(chunk: &[u8]) -> Option<FalconParams> {
    let p = crate::format::chunk_params(chunk);
    if p.len() < FALCON_PARAMS {
        return None;
    }
    let g = |i: usize| p[i] as f64;
    Some(FalconParams {
        osc1_waveform: g(0),
        osc1_ratio_coarse: g(1),
        osc1_ratio_fine: g(2),
        osc1_feedback: g(3),
        osc1_feed_forward: g(4),
        osc1_attack: g(5),
        osc1_decay: g(6),
        osc1_sustain: g(7),
        osc1_release: g(8),
        osc2_waveform: g(9),
        osc2_ratio_coarse: g(10),
        osc2_ratio_fine: g(11),
        osc2_feedback: g(12),
        osc2_attack: g(13),
        osc2_decay: g(14),
        osc2_sustain: g(15),
        osc2_release: g(16),
        master_level: g(17),
        vibrato_freq: g(21),
        vibrato_amount: g(22),
        rise: g(23),
        pitch_attack: g(24),
        pitch_decay: g(25),
        pitch_sustain: g(26),
        pitch_release: g(27),
        pitch_env_amt1: g(28),
        pitch_env_amt2: g(29),
    })
}

/// Envelope times held by the core are in milliseconds; the chunk stores the
/// invertible scalar (`Helpers::ScalarToEnvValue`: `ms = scalar^2 * 5000 + 1`).
fn env_ms(v: f64) -> f64 {
    v * v * 5000.0 + 1.0
}

fn note_to_freq(note: f64) -> f64 {
    440.0 * 2f64.powf((note - 69.0) / 12.0)
}

/// The third + fifth harmonic of a square waveform (used blended with the
/// fundamental by `*_waveform`). See `Helpers::Square35`.
fn square35(phase: f64) -> f64 {
    (phase * 3.0).sin() / 3.0 + (phase * 5.0).sin() / 5.0
}

/// Per-sample derived scalars, mirroring the setup at the top of
/// `Falcon::FalconVoice::Run`.
struct FalconCtx {
    osc1_ratio: f64,
    osc2_ratio: f64,
    osc1_fb: f64,
    osc2_fb: f64,
    osc1_ff: f64,
    master: f64,
    vibrato_per_sample: f64,
    vibrato_amount: f64,
    pamt1: f64,
    pamt2: f64,
    osc1_waveform: f64,
    osc2_waveform: f64,
    rise: f64,
}

impl FalconCtx {
    fn new(p: &FalconParams, sr: f64) -> FalconCtx {
        let ratio = |coarse: f64, fine: f64| {
            let fine_base = (fine - 0.5) * 2.0;
            1.0 + (coarse * 32.99).floor() + fine_base * fine_base * fine_base
        };
        let vib_freq = (p.vibrato_freq * p.vibrato_freq + 0.1) * 70.0;
        let master = p.master_level * 0.4;
        FalconCtx {
            osc1_ratio: ratio(p.osc1_ratio_coarse, p.osc1_ratio_fine),
            osc2_ratio: ratio(p.osc2_ratio_coarse, p.osc2_ratio_fine),
            osc1_fb: 0.5 * p.osc1_feedback * p.osc1_feedback,
            osc2_fb: 0.5 * p.osc2_feedback * p.osc2_feedback,
            osc1_ff: p.osc1_feed_forward * p.osc1_feed_forward,
            master: master * master,
            vibrato_per_sample: vib_freq / sr,
            vibrato_amount: p.vibrato_amount,
            pamt1: (p.pitch_env_amt1 - 0.5) * 72.0,
            pamt2: (p.pitch_env_amt2 - 0.5) * 72.0,
            osc1_waveform: p.osc1_waveform,
            osc2_waveform: p.osc2_waveform,
            rise: p.rise * 24.0,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum EnvState {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

/// ADSR envelope, ported from `WaveSabreCore/Envelope.cpp`. `pos` runs in
/// milliseconds (incremented by `1000 / sampleRate` per sample).
struct Env {
    state: EnvState,
    pos: f64,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    release_value: f64,
}

impl Env {
    fn set(&mut self, attack: f64, decay: f64, sustain: f64, release: f64) {
        self.attack = attack;
        self.decay = decay;
        self.sustain = sustain;
        self.release = release;
    }

    fn trigger(&mut self) {
        self.state = EnvState::Attack;
        self.pos = 0.0;
    }

    fn off(&mut self) {
        self.release_value = self.value();
        self.state = EnvState::Release;
        self.pos = 0.0;
    }

    fn value(&self) -> f64 {
        match self.state {
            EnvState::Attack => self.pos / self.attack,
            EnvState::Decay => {
                let f = 1.0 - self.pos / self.decay;
                let f2 = f * f;
                f2 + self.sustain * (1.0 - f2)
            }
            EnvState::Sustain => self.sustain,
            EnvState::Release => {
                let f = 1.0 - self.pos / self.release;
                let v = f.max(0.0);
                self.release_value * v * v
            }
            EnvState::Done => 0.0,
        }
    }

    fn next(&mut self, sr: f64) {
        if self.state == EnvState::Done {
            return;
        }
        let pos_delta = 1000.0 / sr;
        match self.state {
            EnvState::Attack => {
                self.pos += pos_delta;
                if self.pos >= self.attack {
                    self.state = EnvState::Decay;
                    self.pos -= self.attack;
                }
            }
            EnvState::Decay => {
                self.pos += pos_delta;
                if self.pos >= self.decay {
                    self.state = EnvState::Sustain;
                }
            }
            EnvState::Release => {
                self.pos += pos_delta;
                if self.pos >= self.release {
                    self.state = EnvState::Done;
                }
            }
            _ => {}
        }
    }
}

/// A single Falcon voice. Ported from `Falcon::FalconVoice::Run`; only
/// `osc2` (the carrier) is audible, `osc1` (the modulator) feeds it via
/// `osc1_ff` and is additionally boosted by the core's fixed 13.25x hotness.
struct FalconVoice {
    is_on: bool,
    note: f64,
    osc1_phase: f64,
    osc2_phase: f64,
    osc1_output: f64,
    osc2_output: f64,
    vibrato_phase: f64,
    osc1_env: Env,
    osc2_env: Env,
    pitch_env: Env,
}

impl FalconVoice {
    fn new(p: &FalconParams) -> FalconVoice {
        let mut v = FalconVoice {
            is_on: false,
            note: 0.0,
            osc1_phase: 0.0,
            osc2_phase: 0.0,
            osc1_output: 0.0,
            osc2_output: 0.0,
            vibrato_phase: 0.0,
            osc1_env: Env {
                state: EnvState::Done,
                pos: 0.0,
                attack: 1.0,
                decay: 1.0,
                sustain: 1.0,
                release: 1.0,
                release_value: 0.0,
            },
            osc2_env: Env {
                state: EnvState::Done,
                pos: 0.0,
                attack: 1.0,
                decay: 1.0,
                sustain: 1.0,
                release: 1.0,
                release_value: 0.0,
            },
            pitch_env: Env {
                state: EnvState::Done,
                pos: 0.0,
                attack: 1.0,
                decay: 1.0,
                sustain: 1.0,
                release: 1.0,
                release_value: 0.0,
            },
        };
        v.config(p);
        v
    }

    fn config(&mut self, p: &FalconParams) {
        self.osc1_env.set(
            env_ms(p.osc1_attack),
            env_ms(p.osc1_decay),
            p.osc1_sustain,
            env_ms(p.osc1_release),
        );
        self.osc2_env.set(
            env_ms(p.osc2_attack),
            env_ms(p.osc2_decay),
            p.osc2_sustain,
            env_ms(p.osc2_release),
        );
        self.pitch_env.set(
            env_ms(p.pitch_attack),
            env_ms(p.pitch_decay),
            p.pitch_sustain,
            env_ms(p.pitch_release),
        );
    }

    fn note_on(&mut self, note: u8) {
        self.is_on = true;
        self.note = note as f64;
        // The C++ core seeds the phases from a global RNG
        // (`Helpers::RandFloat`); zero is deterministic and inaudibly close.
        self.osc1_phase = 0.0;
        self.osc2_phase = 0.0;
        self.osc1_output = 0.0;
        self.osc2_output = 0.0;
        self.vibrato_phase = 0.0;
        self.osc1_env.trigger();
        self.osc2_env.trigger();
        self.pitch_env.trigger();
    }

    fn note_off(&mut self) {
        self.osc1_env.off();
        self.osc2_env.off();
        self.pitch_env.off();
    }

    /// One sample of output. Mirrors the exact statement order of
    /// `FalconVoice::Run` (including `osc2Env.Next()` before the frequency
    /// update reads `pitchEnv` / `vibratoPhase`).
    fn next_sample(&mut self, sr: f64, ctx: &FalconCtx) -> f64 {
        let base_note = self.note + ctx.rise;

        let osc1_input = self.osc1_phase / sr * TAU + self.osc1_output * ctx.osc1_fb;
        self.osc1_output = (osc1_input.sin() + square35(osc1_input) * ctx.osc1_waveform)
            * self.osc1_env.value()
            * 13.25;

        let osc2_input = self.osc2_phase / sr * TAU
            + self.osc2_output * ctx.osc2_fb * 13.25
            + self.osc1_output * ctx.osc1_ff;
        self.osc2_output =
            (osc2_input.sin() + square35(osc2_input) * ctx.osc2_waveform) * self.osc2_env.value();
        let out = self.osc2_output * ctx.master;

        self.osc2_env.next(sr);
        if self.osc2_env.state == EnvState::Done {
            self.is_on = false;
            return out;
        }

        let p_env = self.pitch_env.value();
        let vib = self.vibrato_phase.sin() * ctx.vibrato_amount;
        let f1 = note_to_freq(base_note + p_env * ctx.pamt1 + vib);
        let f2 = note_to_freq(base_note + p_env * ctx.pamt2 + vib);
        self.osc1_phase += f1 * ctx.osc1_ratio;
        self.osc2_phase += f2 * ctx.osc2_ratio;
        self.vibrato_phase += ctx.vibrato_per_sample;
        self.osc1_env.next(sr);
        self.pitch_env.next(sr);

        out
    }
}

/// Renders every Falcon track. Voices are polyphonic, one per note-on
/// (matching the core's default `VoiceMode::Polyphonic`). Note velocity does
/// not affect the Falcon output - the C++ voice code never reads it.
struct FalconSynth {
    params: FalconParams,
    ctx: FalconCtx,
    sr: f64,
    voices: Vec<FalconVoice>,
}

impl FalconSynth {
    fn new(params: FalconParams, sr: f64) -> FalconSynth {
        let ctx = FalconCtx::new(&params, sr);
        FalconSynth {
            params,
            ctx,
            sr,
            voices: Vec::new(),
        }
    }
}

// ===========================================================================
// Crude fallback (non-Falcon devices, e.g. Adultery GM samples)
// ===========================================================================

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Saw,
    Box,
    /// A few low harmonics summed (piano-like): a strong fundamental with
    /// decreasing overtones, so the part stays audible without the harshness
    /// of a full saw.
    Piano,
}

#[derive(Clone, Copy)]
struct Inst {
    wave: Wave,
    fixed: Option<f64>,
    pitch: Option<(f64, f64, f64)>,
    amp: f64,
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
}

#[derive(PartialEq)]
enum CrudeStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

struct CrudeEnv {
    stage: CrudeStage,
    t: f64,
    peak: f64,
    release_peak: f64,
}

impl CrudeEnv {
    fn new(_inst: &Inst, peak: f64) -> CrudeEnv {
        CrudeEnv {
            stage: CrudeStage::Attack,
            t: 0.0,
            peak,
            release_peak: 0.0,
        }
    }

    fn next(&mut self, inst: &Inst, dt: f64) -> f64 {
        if self.stage == CrudeStage::Done {
            return 0.0;
        }
        self.t += dt;
        match self.stage {
            CrudeStage::Attack => {
                let x = self.t / inst.attack;
                if x >= 1.0 {
                    self.t -= inst.attack;
                    self.stage = CrudeStage::Decay;
                    self.peak
                } else {
                    self.peak * x
                }
            }
            CrudeStage::Decay => {
                let x = self.t / inst.decay;
                if x >= 1.0 {
                    self.stage = CrudeStage::Sustain;
                    self.peak * inst.sustain
                } else {
                    self.peak - (self.peak - self.peak * inst.sustain) * x
                }
            }
            CrudeStage::Sustain => self.peak * inst.sustain,
            CrudeStage::Release => {
                let x = self.t / inst.release;
                let v = (1.0 - x).max(0.0);
                if x >= 1.0 {
                    self.stage = CrudeStage::Done;
                }
                self.release_peak * v * v
            }
            CrudeStage::Done => 0.0,
        }
    }

    fn release(&mut self, _inst: &Inst, current: f64) {
        if self.stage == CrudeStage::Done {
            return;
        }
        self.release_peak = current;
        self.stage = CrudeStage::Release;
        self.t = 0.0;
    }
}

struct CrudeVoice {
    note: u8,
    phase: f64,
    t: f64,
    env: CrudeEnv,
}

struct CrudeSynth {
    inst: Inst,
    sr: f64,
    voices: Vec<CrudeVoice>,
}

impl CrudeSynth {
    fn new(inst: Inst, sr: f64) -> CrudeSynth {
        CrudeSynth {
            inst,
            sr,
            voices: Vec::new(),
        }
    }
}

fn voice_freq(inst: &Inst, note: u8, t: f64) -> f64 {
    if let Some(f) = inst.fixed {
        f
    } else if let Some((f0, rate, floor)) = inst.pitch {
        f0 * (-rate * t).exp() + floor
    } else {
        440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
    }
}

fn osc(wave: &Wave, phase: f64) -> f64 {
    // `phase` accumulates in *cycles* (voice_freq returns Hz and is advanced by
    // `f * dt`), so Saw/Box wrap it with `fract()` directly. The sine-family
    // waves need radians, hence the TAU conversion at the trig calls below.
    match wave {
        Wave::Sine => (TAU * phase).sin(),
        Wave::Saw => 2.0 * phase.fract() - 1.0,
        Wave::Box => {
            if phase.fract() < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Piano => {
            // sum of the first six harmonics, roughly 1/k rolloff - normalized
            // so the peak stays close to a plain sine's.
            let s = (TAU * phase).sin()
                + 0.5 * (TAU * 2.0 * phase).sin()
                + 0.34 * (TAU * 3.0 * phase).sin()
                + 0.25 * (TAU * 4.0 * phase).sin()
                + 0.2 * (TAU * 5.0 * phase).sin()
                + 0.15 * (TAU * 6.0 * phase).sin();
            s / 2.4
        }
    }
}

// ===========================================================================
// Shared track driver
// ===========================================================================

trait TrackSynth {
    fn note_on(&mut self, note: u8, vel: f32);
    fn note_off(&mut self, note: u8);
    fn next_sample(&mut self) -> f64;
}

impl TrackSynth for FalconSynth {
    fn note_on(&mut self, note: u8, _vel: f32) {
        let params = self.params;
        if self.voices.len() >= MAX_VOICES {
            self.voices.remove(0);
        }
        let mut v = FalconVoice::new(&params);
        v.note_on(note);
        self.voices.push(v);
    }

    fn note_off(&mut self, note: u8) {
        for v in self.voices.iter_mut() {
            if v.is_on && v.note as u8 == note {
                v.note_off();
            }
        }
    }

    fn next_sample(&mut self) -> f64 {
        let sr = self.sr;
        let ctx = &self.ctx;
        let mut out = 0.0;
        let mut i = 0;
        while i < self.voices.len() {
            let v = &mut self.voices[i];
            out += v.next_sample(sr, ctx);
            if !v.is_on {
                self.voices.swap_remove(i);
            } else {
                i += 1;
            }
        }
        out
    }
}

impl TrackSynth for CrudeSynth {
    fn note_on(&mut self, note: u8, vel: f32) {
        // retrigger: any active voice on the same note releases immediately
        let inst = self.inst;
        let mut i = 0;
        while i < self.voices.len() {
            let v = &mut self.voices[i];
            if v.note == note {
                let cur = v.env.next(&inst, 0.0);
                v.env.release(&inst, cur);
                self.voices.swap_remove(i);
            } else {
                i += 1;
            }
        }
        if self.voices.len() >= 48 {
            self.voices.remove(0);
        }
        self.voices.push(CrudeVoice {
            note,
            phase: 0.0,
            t: 0.0,
            env: CrudeEnv::new(&self.inst, self.inst.amp * (vel / 127.0) as f64),
        });
    }

    fn note_off(&mut self, note: u8) {
        let inst = self.inst;
        for v in self.voices.iter_mut() {
            if v.note == note {
                let cur = v.env.next(&inst, 0.0);
                v.env.release(&inst, cur);
            }
        }
    }

    fn next_sample(&mut self) -> f64 {
        let dt = 1.0 / self.sr;
        let inst = self.inst;
        let mut out = 0.0;
        let mut i = 0;
        while i < self.voices.len() {
            let (finished, sample) = {
                let v = &mut self.voices[i];
                v.t += dt;
                let f = voice_freq(&inst, v.note, v.t);
                v.phase += f * dt;
                let gain = v.env.next(&inst, dt);
                let s = osc(&inst.wave, v.phase);
                (v.env.stage == CrudeStage::Done, s * gain)
            };
            out += sample;
            if finished {
                self.voices.swap_remove(i);
            } else {
                i += 1;
            }
        }
        out
    }
}

/// Steps `synth` against `events` (sorted by sample position) for `total`
/// samples, returning the mono mix.
fn run_track<S: TrackSynth>(
    mut synth: S,
    events: &[(usize, bool, u8, f32)],
    total: usize,
) -> Vec<f32> {
    let mut out = vec![0.0f32; total];
    let mut idx = 0;
    for (s, o) in out.iter_mut().enumerate() {
        while idx < events.len() && events[idx].0 <= s {
            let (_, on, note, vel) = events[idx];
            if on {
                synth.note_on(note, vel);
            } else {
                synth.note_off(note);
            }
            idx += 1;
        }
        *o += synth.next_sample() as f32;
    }
    out
}

/// Renders a parsed song to mono floats (one sample per unit of sample_rate).
pub fn render(song: &ParsedSong) -> Vec<f32> {
    let sr = song.sample_rate.max(1) as f64;
    let total = (song.length * sr) as usize;
    let mut master = vec![0.0f32; total];

    for ti in 0..song.tracks.len() {
        let lane = render_track(song, ti);
        let n = lane.len().min(total);
        for i in 0..n {
            master[i] += lane[i];
        }
    }

    master
}

/// Renders a single track (by index) to mono floats, used for per-instrument
/// WAV stems.
pub fn render_solo(song: &ParsedSong, ti: usize) -> Vec<f32> {
    render_track(song, ti)
}

fn render_track(song: &ParsedSong, ti: usize) -> Vec<f32> {
    let sr = song.sample_rate.max(1) as f64;
    let total = (song.length * sr) as usize;
    let out = vec![0.0f32; total];
    if ti >= song.tracks.len() {
        return out;
    }
    let track = &song.tracks[ti];
    let lane = &song.lanes[track.lane_id];
    if lane.is_empty() {
        return out;
    }

    let mut events: Vec<(usize, bool, u8, f32)> = lane
        .iter()
        .map(|e| (e.samples as usize, e.on, e.note, e.velocity as f32 / 127.0))
        .collect();
    events.sort_by_key(|e| e.0);

    // Falcon: render the actual FM patch.
    if let Some(di) = track.device_indices.first() {
        let dev = &song.devices[*di];
        if dev.id == DeviceId::Falcon {
            if let Some(p) = falcon_params(&dev.chunk) {
                return run_track(FalconSynth::new(p, sr), &events, total);
            }
        }
    }

    // Fallback for non-Falcon devices: simple oscillators keyed by track role.
    run_track(CrudeSynth::new(inst_for(ti), sr), &events, total)
}

fn inst_for(ti: usize) -> Inst {
    match ti {
        0 => Inst {
            // piano: rich partial stack, medium sustain, pokes through the mix
            wave: Wave::Piano,
            fixed: None,
            pitch: None,
            amp: 0.32,
            attack: 0.005,
            decay: 0.30,
            sustain: 0.60,
            release: 0.55,
        },
        1 => Inst {
            // bass: saw, short pluck
            wave: Wave::Saw,
            fixed: None,
            pitch: None,
            amp: 0.13,
            attack: 0.004,
            decay: 0.09,
            sustain: 0.30,
            release: 0.12,
        },
        2 => Inst {
            // kick: sine with a fast pitch drop
            wave: Wave::Sine,
            fixed: None,
            pitch: Some((150.0, 22.0, 42.0)),
            amp: 0.34,
            attack: 0.001,
            decay: 0.25,
            sustain: 0.0,
            release: 0.08,
        },
        3 => Inst {
            // snare: box at a low body frequency
            wave: Wave::Box,
            fixed: Some(185.0),
            pitch: None,
            amp: 0.16,
            attack: 0.001,
            decay: 0.14,
            sustain: 0.0,
            release: 0.08,
        },
        4 => Inst {
            // closed hat: box, very short
            wave: Wave::Box,
            fixed: Some(7200.0),
            pitch: None,
            amp: 0.075,
            attack: 0.001,
            decay: 0.05,
            sustain: 0.0,
            release: 0.04,
        },
        5 => Inst {
            // open hat: box, longer
            wave: Wave::Box,
            fixed: Some(7200.0),
            pitch: None,
            amp: 0.06,
            attack: 0.001,
            decay: 0.22,
            sustain: 0.0,
            release: 0.10,
        },
        _ => Inst {
            wave: Wave::Sine,
            fixed: None,
            pitch: None,
            amp: 0.0,
            attack: 0.001,
            decay: 0.1,
            sustain: 0.0,
            release: 0.05,
        },
    }
}

/// Scales the mix so its peak reaches ~0.95 (-0.45 dBFS).
pub fn normalize(samples: &mut Vec<f32>) {
    let peak = samples.iter().fold(0.0f32, |a, s| a.max(s.abs()));
    if peak > 0.0 {
        let scale = 0.95 / peak;
        for s in samples.iter_mut() {
            *s *= scale;
        }
    }
}

fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
        pcm.extend_from_slice(&v.to_le_bytes());
    }

    let data_len = pcm.len() as u32;
    let mut out = Vec::with_capacity(44 + pcm.len());

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(&pcm);

    fs::write(path, out).expect("failed to write wav");
}

/// Writes a mono 16-bit PCM WAV using the same base path as the song file but
/// with the `.wav` extension.
pub fn write_wav_at(base_path: &str, samples: &[f32], sample_rate: u32) {
    let path = Path::new(base_path).with_extension("wav");
    write_wav(&path, samples, sample_rate);
}
