//! Slaughter trumpet — a single bold brass voice, 5 stylings.
//!
//! A solo trumpet is one powerful voice rich in the 2nd-4th harmonics, with a
//! moderately fast "lip" attack and a formant that opens into the note. On
//! Slaughter that's a saw-ward pulse core through a resonant low-pass whose
//! cutoff opens with the swell, a small downward pitch scoop settling in
//! (lip), and a light growl. Light detune keeps it a solo instrument — the
//! wide chorus lives in `brass`.

use crate::format::{DeviceId, Slaughter, env_ms};

pub struct Trumpet {
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub detune_cent: f32,
    pub growl: f32,
    pub scoop: f32,
    pub attack_ms: f32,
    pub swell_ms: f32,
    pub sustain: f32,
    pub release_ms: f32,
    pub vibrato_hz: f32,
    pub vibrato_amt: f32,
    pub master: f32,
    pub noise: f32,
}

fn trumpet(cfg: &Trumpet) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Saw-ward pulse core (rich brass overtones to hit), mostly on osc1, a
    // light stack on osc2/osc3 for body without a section's wide chorus.
    s.osc1_waveform = 0.62;
    s.osc1_pulse_width = Slaughter::pulse_width(0.45);
    s.osc1_volume = 0.95;
    s.osc2_waveform = 0.55;
    s.osc2_pulse_width = Slaughter::pulse_width(0.45);
    s.osc2_volume = 0.55;
    s.osc2_detune_fine = Slaughter::detune_fine(cfg.detune_cent);
    s.osc3_waveform = 0.45;
    s.osc3_pulse_width = Slaughter::pulse_width(0.5);
    s.osc3_volume = 0.35;
    s.osc3_detune_fine = Slaughter::detune_fine(-cfg.detune_cent * 0.8);
    // Breath + lip noise.
    s.noise_volume = cfg.noise;
    // The brass formant: resonant low-pass that opens with the swell.
    s.filter_type = 0.0;
    s.filter_freq = Slaughter::filter_freq_hz(cfg.cutoff_hz);
    s.filter_resonance = Slaughter::resonance(cfg.resonance);
    s.filter_mod_amt = 0.62;
    s.mod_attack = env_ms(cfg.attack_ms * 1.2);
    s.mod_decay = env_ms(cfg.swell_ms);
    s.mod_sustain = 0.55;
    // Lip scoop: settle down into the note.
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
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn trumpet_v1() -> (DeviceId, Vec<u8>) {
    // Lead trumpet: bold, bright, quick attack, confident sustain.
    trumpet(&Trumpet {
        cutoff_hz: 2400.0, resonance: 0.32, detune_cent: 5.0, growl: 0.0, scoop: 1.5,
        attack_ms: 12.0, swell_ms: 450.0, sustain: 0.82, release_ms: 180.0,
        vibrato_hz: 5.2, vibrato_amt: 0.16, master: 0.60, noise: 0.03,
    })
}

pub fn trumpet_v2() -> (DeviceId, Vec<u8>) {
    // Mellow clarino: rounder, warmer cutoff, gentle attack and vibrato.
    trumpet(&Trumpet {
        cutoff_hz: 1500.0, resonance: 0.40, detune_cent: 4.0, growl: 0.0, scoop: 1.0,
        attack_ms: 28.0, swell_ms: 650.0, sustain: 0.86, release_ms: 260.0,
        vibrato_hz: 4.9, vibrato_amt: 0.22, master: 0.56, noise: 0.04,
    })
}

pub fn trumpet_v3() -> (DeviceId, Vec<u8>) {
    // Bright staccato: fast attack and tight release, cutting through a mix.
    trumpet(&Trumpet {
        cutoff_hz: 3200.0, resonance: 0.28, detune_cent: 6.0, growl: 0.0, scoop: 0.5,
        attack_ms: 6.0, swell_ms: 320.0, sustain: 0.78, release_ms: 110.0,
        vibrato_hz: 5.4, vibrato_amt: 0.10, master: 0.64, noise: 0.03,
    })
}

pub fn trumpet_v4() -> (DeviceId, Vec<u8>) {
    // Growly pedal: extra lip buzz, deeper resonance, more air.
    trumpet(&Trumpet {
        cutoff_hz: 1800.0, resonance: 0.48, detune_cent: 5.0, growl: 0.0, scoop: 2.0,
        attack_ms: 18.0, swell_ms: 520.0, sustain: 0.80, release_ms: 220.0,
        vibrato_hz: 5.0, vibrato_amt: 0.20, master: 0.56, noise: 0.08,
    })
}

pub fn trumpet_v5() -> (DeviceId, Vec<u8>) {
    // Soft ballad: pillowy, unhurried entrance, shy vibrato, long tail.
    trumpet(&Trumpet {
        cutoff_hz: 1200.0, resonance: 0.44, detune_cent: 3.0, growl: 0.0, scoop: 1.0,
        attack_ms: 42.0, swell_ms: 880.0, sustain: 0.88, release_ms: 360.0,
        vibrato_hz: 4.7, vibrato_amt: 0.26, master: 0.52, noise: 0.05,
    })
}
