//! ADSR envelope, ported from `WaveSabreCore/Envelope.cpp`.
//!
//! `pos` runs in milliseconds; `Next` advances it by `1000 / sample_rate`.
//! Times are held in ms (the device stores them as `EnvValueToScalar` scalars;
//! the synth copies/scales them at note-on, exactly like the core).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvState {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

#[derive(Clone, Copy, Debug)]
pub struct Envelope {
    pub state: EnvState,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pos: f32,
    release_value: f32,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            state: EnvState::Finished,
            attack: 1.0,
            decay: 5.0,
            sustain: 0.5,
            release: 1.5,
            pos: 0.0,
            release_value: 0.0,
        }
    }

    pub fn trigger(&mut self) {
        self.state = EnvState::Attack;
        self.pos = 0.0;
    }

    pub fn off(&mut self) {
        self.release_value = self.value();
        self.state = EnvState::Release;
        self.pos = 0.0;
    }

    pub fn value(&self) -> f32 {
        match self.state {
            EnvState::Attack => self.pos / self.attack,
            EnvState::Decay => {
                let f = 1.0 - self.pos / self.decay;
                f * f + self.sustain * (1.0 - f * f)
            }
            EnvState::Sustain => self.sustain,
            EnvState::Release => {
                let f = 1.0 - self.pos / self.release;
                self.release_value * f * f
            }
            EnvState::Finished => 0.0,
        }
    }

    pub fn next(&mut self, sr: f64) {
        if self.state == EnvState::Finished {
            return;
        }
        let pos_delta = (1000.0 / sr) as f32;
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
                    self.state = EnvState::Finished;
                }
            }
            EnvState::Sustain | EnvState::Finished => {}
        }
    }
}