//! A tiny, platform-independent software renderer that plays a parsed song
//! with simple oscillators (sine / saw / box), used by `examples/create_song
//! --wav` to preview a song on non-Windows machines.
//!
//! This is intentionally crude: it exists so the music can be *listened to*
//! before testing on the real Windows/WaveSabre target. It is not the sound
//! engine used by the demo.

use crate::format::ParsedSong;
use std::f64::consts::TAU;
use std::fs;
use std::path::Path;
use std::vec::Vec;

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
enum Stage {
    Attack,
    Decay,
    Sustain,
    Release,
    Done,
}

struct Env {
    stage: Stage,
    t: f64,
    peak: f64,
    release_peak: f64,
}

impl Env {
    fn new(inst: &Inst, peak: f64) -> Env {
        let _ = inst;
        Env {
            stage: Stage::Attack,
            t: 0.0,
            peak,
            release_peak: 0.0,
        }
    }

    fn next(&mut self, inst: &Inst, dt: f64) -> f64 {
        if self.stage == Stage::Done {
            return 0.0;
        }
        self.t += dt;
        match self.stage {
            Stage::Attack => {
                let x = self.t / inst.attack;
                if x >= 1.0 {
                    self.t -= inst.attack;
                    self.stage = Stage::Decay;
                    self.peak
                } else {
                    self.peak * x
                }
            }
            Stage::Decay => {
                let x = self.t / inst.decay;
                if x >= 1.0 {
                    self.stage = Stage::Sustain;
                    self.peak * inst.sustain
                } else {
                    self.peak - (self.peak - self.peak * inst.sustain) * x
                }
            }
            Stage::Sustain => self.peak * inst.sustain,
            Stage::Release => {
                let x = self.t / inst.release;
                let v = (1.0 - x).max(0.0);
                if x >= 1.0 {
                    self.stage = Stage::Done;
                }
                self.release_peak * v * v
            }
            Stage::Done => 0.0,
        }
    }

    fn release(&mut self, inst: &Inst, current: f64) {
        if self.stage == Stage::Done {
            return;
        }
        self.release_peak = current;
        self.stage = Stage::Release;
        self.t = 0.0;
        let _ = inst;
    }
}

struct Voice {
    note: u8,
    phase: f64,
    t: f64,
    env: Env,
}

struct Synth {
    events: Vec<(usize, bool, u8, f32)>, // (sample, on, note, normalized vel)
    event_idx: usize,
    inst: Inst,
    voices: Vec<Voice>,
}

impl Synth {
    fn next_sample(&mut self, sr: f64) -> f64 {
        let dt = 1.0 / sr;
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
                (v.env.stage == Stage::Done, s * gain)
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
        self.voices.push(Voice {
            note,
            phase: 0.0,
            t: 0.0,
            env: Env::new(&self.inst, self.inst.amp * (vel / 127.0) as f64),
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

/// Renders a parsed song to mono floats (one sample per unit of sample_rate).
pub fn render(song: &ParsedSong) -> Vec<f32> {
    let sr = song.sample_rate.max(1) as f64;
    let total = (song.length * sr) as usize;
    let mut master = vec![0.0f32; total];

    // Track index -> instrument. This matches the layout produced by
    // examples/create_song: piano, bass, kick, snare, hat_c, hat_o, master.
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
    let mut out = vec![0.0f32; total];
    if ti >= song.tracks.len() {
        return out;
    }
    let track = &song.tracks[ti];
    let lane = &song.lanes[track.lane_id];
    if lane.is_empty() {
        return out;
    }
    let inst = inst_for(ti);
    let mut events: Vec<(usize, bool, u8, f32)> = lane
        .iter()
        .map(|e| (e.samples as usize, e.on, e.note, e.velocity as f32 / 127.0))
        .collect();
    events.sort_by_key(|e| e.0);

    let mut synth = Synth {
        events,
        event_idx: 0,
        inst,
        voices: Vec::new(),
    };

    for s in 0..total {
        while synth.event_idx < synth.events.len() && synth.events[synth.event_idx].0 <= s {
            let (_, on, note, vel) = synth.events[synth.event_idx];
            if on {
                synth.note_on(note, vel);
            } else {
                synth.note_off(note);
            }
            synth.event_idx += 1;
        }
        out[s] += synth.next_sample(sr) as f32;
    }

    out
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