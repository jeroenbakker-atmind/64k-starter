//! Slaughter brass section — a thick detuned horn ensemble, 5 stylings.
//!
//! A brass section is several voices locked together: detuned for a muscular
//! chorus sheen, wider and heavier than a solo trumpet, with the classic
//! horn swell — the resonant low-pass opening on the attack and settling into
//! a full, slightly gritty sustain. Polyphonic so section chords voicing
//! multiple notes stay intact.
//!
//! Each patch is a full 3-oscillator stack with a wider detune spread than
//! the solo `trumpet`, and some variations add a touch of unison for extra
//! section body.

use common::{DeviceId, Slaughter, env_ms};

pub struct Brass {
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub spread_cent: f32,
    pub osc3_partial: f32,
    pub scoop: f32,
    pub attack_ms: f32,
    pub swell_ms: f32,
    pub sustain: f32,
    pub release_ms: f32,
    pub vibrato_hz: f32,
    pub vibrato_amt: f32,
    pub unisono: f32,
    pub detune: f32,
    pub master: f32,
    pub noise: f32,
}

fn brass(cfg: &Brass) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Full three-oscillator stack: rich overtone ladder for the section mass.
    s.osc1_waveform = 0.60;
    s.osc1_pulse_width = Slaughter::pulse_width(0.45);
    s.osc1_volume = 0.95;
    s.osc2_waveform = 0.55;
    s.osc2_pulse_width = Slaughter::pulse_width(0.45);
    s.osc2_volume = 0.65;
    s.osc2_detune_fine = Slaughter::detune_fine(cfg.spread_cent);
    s.osc3_waveform = 0.50;
    s.osc3_pulse_width = Slaughter::pulse_width(0.48);
    s.osc3_volume = 0.45;
    s.osc3_detune_coarse = Slaughter::detune_coarse(cfg.osc3_partial);
    s.osc3_detune_fine = Slaughter::detune_fine(-cfg.spread_cent);
    // Airy wind noise under the section.
    s.noise_volume = cfg.noise;
    // The horn swell: resonant low-pass that opens wide on the attack.
    s.filter_type = 0.0;
    s.filter_freq = Slaughter::filter_freq_hz(cfg.cutoff_hz);
    s.filter_resonance = Slaughter::resonance(cfg.resonance);
    s.filter_mod_amt = 0.66;
    s.mod_attack = env_ms(cfg.attack_ms * 1.3);
    s.mod_decay = env_ms(cfg.swell_ms);
    s.mod_sustain = 0.60;
    // Lip scoop settles into the chord.
    s.pitch_attack = env_ms(2.0);
    s.pitch_decay = env_ms(160.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(60.0);
    s.pitch_env_amt = Slaughter::pitch_amt(cfg.scoop);
    s.amp_attack = env_ms(cfg.attack_ms);
    s.amp_decay = env_ms(cfg.swell_ms);
    s.amp_sustain = cfg.sustain;
    s.amp_release = env_ms(cfg.release_ms);
    s.vibrato_freq = cfg.vibrato_hz;
    s.vibrato_amount = cfg.vibrato_amt;
    s.master_level = cfg.master;
    s.voices_unisono = cfg.unisono;
    s.voices_detune = cfg.detune;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn brass_fat_low() -> (DeviceId, Vec<u8>) {
    // Fat low brass: deep resonance, wide spread, growly underbelly.
    brass(&Brass {
        cutoff_hz: 1400.0, resonance: 0.48, spread_cent: 11.0, osc3_partial: 24.0, scoop: 2.0,
        attack_ms: 18.0, swell_ms: 550.0, sustain: 0.82, release_ms: 240.0,
        vibrato_hz: 4.8, vibrato_amt: 0.18, unisono: 0.20, detune: 0.22, master: 0.58, noise: 0.08,
    })
}

pub fn brass_cinematic_swell() -> (DeviceId, Vec<u8>) {
    // Soft cinematic swell: wide but gentle, generous attack and long tail.
    brass(&Brass {
        cutoff_hz: 1700.0, resonance: 0.40, spread_cent: 12.0, osc3_partial: 12.0, scoop: 1.0,
        attack_ms: 40.0, swell_ms: 900.0, sustain: 0.88, release_ms: 380.0,
        vibrato_hz: 4.7, vibrato_amt: 0.22, unisono: 0.13, detune: 0.16, master: 0.54, noise: 0.06,
    })
}
