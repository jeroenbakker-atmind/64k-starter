//! Slaughter flute — divergent from `falcon::flute`.
//!
//! Improvement: a genuinely breathy lead. Three oscillators at ±4..5 cents
//! form a micro-chorus, a touch of white noise is the "air", and the
//! low-pass filter opens with the blow-in via the mod env instead of relying
//! on Falcon's near-pure sine core.

use crate::format::{DeviceId, Slaughter, env_ms};

pub fn flute() -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Micro-chorus core: one on-pitch pulse, two slightly off it.
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.8;
    s.osc2_waveform = 0.0;
    s.osc2_pulse_width = Slaughter::pulse_width(0.5);
    s.osc2_volume = 0.55;
    s.osc2_detune_fine = Slaughter::detune_fine(5.0);
    s.osc3_waveform = 0.02;
    s.osc3_pulse_width = Slaughter::pulse_width(0.5);
    s.osc3_volume = 0.35;
    s.osc3_detune_fine = Slaughter::detune_fine(-4.0);
    // Breath.
    s.noise_volume = 0.08;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(2400.0);
    s.filter_resonance = Slaughter::resonance(0.12);
    s.filter_mod_amt = 0.6;
    s.mod_attack = env_ms(22.0);
    s.mod_decay = env_ms(250.0);
    s.mod_sustain = 0.55;
    s.amp_attack = env_ms(14.0);
    s.amp_decay = env_ms(420.0);
    s.amp_sustain = 0.50;
    s.amp_release = env_ms(160.0);
    s.vibrato_freq = 4.8;
    s.vibrato_amount = 0.13;
    s.master_level = 0.60;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}