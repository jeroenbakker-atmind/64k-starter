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

fn flute_cfg(
    cutoff_hz: f32,
    resonance: f32,
    noise: f32,
    osc2_cent: f32,
    osc3_cent: f32,
    mod_amt: f32,
    mod_atk: f32,
    mod_dec: f32,
    mod_sus: f32,
    atk_ms: f32,
    dec_ms: f32,
    sus: f32,
    rel_ms: f32,
    vib_f: f32,
    vib_a: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Micro-chorus core: one on-pitch pulse, two slightly off it.
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.8;
    s.osc2_waveform = 0.0;
    s.osc2_pulse_width = Slaughter::pulse_width(0.5);
    s.osc2_volume = 0.55;
    s.osc2_detune_fine = Slaughter::detune_fine(osc2_cent);
    s.osc3_waveform = 0.02;
    s.osc3_pulse_width = Slaughter::pulse_width(0.5);
    s.osc3_volume = 0.35;
    s.osc3_detune_fine = Slaughter::detune_fine(osc3_cent);
    // Breath.
    s.noise_volume = noise;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(cutoff_hz);
    s.filter_resonance = Slaughter::resonance(resonance);
    s.filter_mod_amt = mod_amt;
    s.mod_attack = env_ms(mod_atk);
    s.mod_decay = env_ms(mod_dec);
    s.mod_sustain = mod_sus;
    s.amp_attack = env_ms(atk_ms);
    s.amp_decay = env_ms(dec_ms);
    s.amp_sustain = sus;
    s.amp_release = env_ms(rel_ms);
    s.vibrato_freq = vib_f;
    s.vibrato_amount = vib_a;
    s.master_level = master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn flute_v1() -> (DeviceId, Vec<u8>) {
    flute_cfg(3200.0, 0.10, 0.14, 6.0, -5.0, 0.65, 12.0, 360.0, 0.50, 10.0, 420.0, 0.50, 140.0, 5.0, 0.14, 0.60)
}

pub fn flute_v2() -> (DeviceId, Vec<u8>) {
    flute_cfg(1600.0, 0.18, 0.06, 3.0, -2.0, 0.50, 24.0, 500.0, 0.65, 30.0, 600.0, 0.70, 260.0, 4.6, 0.10, 0.56)
}

pub fn flute_v3() -> (DeviceId, Vec<u8>) {
    flute_cfg(2400.0, 0.12, 0.08, 9.0, -9.0, 0.60, 14.0, 420.0, 0.55, 16.0, 500.0, 0.55, 160.0, 4.9, 0.15, 0.62)
}

pub fn flute_v4() -> (DeviceId, Vec<u8>) {
    flute_cfg(2200.0, 0.14, 0.18, 5.0, -4.0, 0.60, 14.0, 450.0, 0.55, 12.0, 550.0, 0.55, 200.0, 5.0, 0.10, 0.58)
}

pub fn flute_v5() -> (DeviceId, Vec<u8>) {
    flute_cfg(4000.0, 0.08, 0.10, 4.0, -3.0, 0.70, 4.0, 300.0, 0.45, 4.0, 260.0, 0.45, 110.0, 5.4, 0.12, 0.62)
}

pub fn flute_v6() -> (DeviceId, Vec<u8>) {
    flute_cfg(2000.0, 0.14, 0.06, 5.0, -4.0, 0.55, 40.0, 700.0, 0.80, 50.0, 800.0, 0.80, 400.0, 4.5, 0.16, 0.62)
}

pub fn flute_v7() -> (DeviceId, Vec<u8>) {
    flute_cfg(4200.0, 0.05, 0.04, 3.0, -2.0, 0.55, 10.0, 400.0, 0.60, 14.0, 500.0, 0.60, 150.0, 5.2, 0.08, 0.58)
}

pub fn flute_v8() -> (DeviceId, Vec<u8>) {
    flute_cfg(2200.0, 0.16, 0.10, 8.0, -8.0, 0.55, 16.0, 460.0, 0.55, 18.0, 550.0, 0.55, 200.0, 4.3, 0.20, 0.62)
}

pub fn flute_v9() -> (DeviceId, Vec<u8>) {
    flute_cfg(3600.0, 0.10, 0.08, 5.0, -3.0, 0.60, 6.0, 300.0, 0.40, 6.0, 300.0, 0.40, 100.0, 5.6, 0.12, 0.62)
}

pub fn flute_v10() -> (DeviceId, Vec<u8>) {
    flute_cfg(5200.0, 0.06, 0.16, 6.0, -5.0, 0.68, 8.0, 320.0, 0.48, 8.0, 320.0, 0.48, 120.0, 5.2, 0.12, 0.60)
}