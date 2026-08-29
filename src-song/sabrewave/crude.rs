//! Fallback oscillator synth for non-Falcon / non-Slaughter devices (Adultery
//! GM samples, Specimen, Thunder), which cannot be synthesized from the chunk.
//! Simple sine/saw/box/piano oscillators keyed by track role (the old
//! `render.rs` fallback), now outputting stereo.

use std::vec::Vec;

#[derive(Clone, Copy)]
pub enum Wave {
    Sine,
    Saw,
    Box,
    /// A few low harmonics summed (piano-like), so the part stays audible
    /// without the harshness of a full saw.
    Piano,
}

#[derive(Clone, Copy)]
pub struct Inst {
    pub wave: Wave,
    pub fixed: Option<f64>,
    pub pitch: Option<(f64, f64, f64)>,
    pub amp: f64,
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
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
    fn new(peak: f64) -> CrudeEnv {
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
        fn quad(x: f64) -> f64 {
            let v = (1.0 - x).max(0.0);
            v * v
        }
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
                if x >= 1.0 {
                    self.stage = CrudeStage::Done;
                }
                self.release_peak * quad(x)
            }
            CrudeStage::Done => 0.0,
        }
    }

    fn release(&mut self, current: f64) {
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
    use std::f64::consts::TAU;
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

pub struct CrudeSynth {
    pub inst: Inst,
    sr: f64,
    voices: Vec<CrudeVoice>,
}

impl CrudeSynth {
    pub fn new(inst: Inst, sr: f64) -> CrudeSynth {
        CrudeSynth {
            inst,
            sr,
            voices: Vec::new(),
        }
    }

    pub fn note_on(&mut self, note: u8, vel: u8) {
        let inst = self.inst;
        // retrigger: any active voice on the same note releases immediately
        let mut i = 0;
        while i < self.voices.len() {
            let v = &mut self.voices[i];
            if v.note == note {
                let cur = v.env.next(&inst, 0.0);
                v.env.release(cur);
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
            env: CrudeEnv::new(self.inst.amp * (vel as f64 / 127.0)),
        });
    }

    pub fn note_off(&mut self, note: u8) {
        let inst = self.inst;
        for v in self.voices.iter_mut() {
            if v.note == note {
                let cur = v.env.next(&inst, 0.0);
                v.env.release(cur);
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        let inst = self.inst;
        for v in self.voices.iter_mut() {
            if v.env.stage != CrudeStage::Done {
                let cur = v.env.next(&inst, 0.0);
                v.env.release(cur);
            }
        }
    }

    pub fn next_sample(&mut self) -> (f32, f32) {
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
        let out = out as f32;
        (out, out)
    }
}

/// Legacy track-role instrument table (used when a track's device isn't
/// rendered by Falcon/Slaughter).
pub fn inst_for(ti: usize) -> Inst {
    match ti {
        0 => Inst {
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