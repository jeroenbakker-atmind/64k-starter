//! Slaughter piano — a proper harmonic stack.
//!
//! Issue with the v1: three detuned oscillators *at the fundamental* just
//! sounded like a plucked saw. A piano is its partials: here osc2 and osc3 sit
//! on the 2nd and 3rd harmonics (+12 / +19 semitones below band-limit), each
//! with the tiny "string stretch" inharmonicity a real hammer introduces. The
//! hammer is the noise burst; the filter pops bright on attack and settles
//! warm over ~1.2s, then the voice decays off without a long sustain tail.

use crate::format::{DeviceId, Slaughter, env_ms};

pub fn piano() -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Partial stack: 1st (fundamental), 2nd, 3rd harmonics, quieter upward.
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.9;
    s.osc2_waveform = 0.0;
    s.osc2_pulse_width = Slaughter::pulse_width(0.5);
    s.osc2_volume = 0.45;
    s.osc2_detune_coarse = Slaughter::detune_coarse(12.0);
    s.osc2_detune_fine = Slaughter::detune_fine(-8.0);
    s.osc3_waveform = 0.12;
    s.osc3_pulse_width = Slaughter::pulse_width(0.42);
    s.osc3_volume = 0.30;
    s.osc3_detune_coarse = Slaughter::detune_coarse(19.0);
    s.osc3_detune_fine = Slaughter::detune_fine(6.0);
    // Hammer: broadband smack that the filter instantly dulls.
    s.noise_volume = 0.12;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(6000.0);
    s.filter_resonance = Slaughter::resonance(0.15);
    // Sparkle pop on the attack, then settle warm while the string rings.
    s.filter_mod_amt = 0.62;
    s.mod_attack = env_ms(2.0);
    s.mod_decay = env_ms(1200.0);
    s.mod_sustain = 0.35;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(2600.0);
    s.amp_sustain = 0.03;
    s.amp_release = env_ms(250.0);
    s.master_level = 0.62;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

fn piano_cfg(
    cutoff_hz: f32,
    resonance: f32,
    noise: f32,
    osc2_cent: f32,
    osc3_cent: f32,
    osc3_partial: f32,
    mod_amt: f32,
    mod_dec: f32,
    dec_ms: f32,
    sus: f32,
    rel_ms: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Partial stack: 1st (fundamental), 2nd, 3rd harmonics, quieter upward.
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.9;
    s.osc2_waveform = 0.0;
    s.osc2_pulse_width = Slaughter::pulse_width(0.5);
    s.osc2_volume = 0.45;
    s.osc2_detune_coarse = Slaughter::detune_coarse(12.0);
    s.osc2_detune_fine = Slaughter::detune_fine(osc2_cent);
    s.osc3_waveform = 0.12;
    s.osc3_pulse_width = Slaughter::pulse_width(0.42);
    s.osc3_volume = 0.30;
    s.osc3_detune_coarse = Slaughter::detune_coarse(osc3_partial);
    s.osc3_detune_fine = Slaughter::detune_fine(osc3_cent);
    // Hammer: broadband smack that the filter instantly dulls.
    s.noise_volume = noise;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(cutoff_hz);
    s.filter_resonance = Slaughter::resonance(resonance);
    s.filter_mod_amt = mod_amt;
    s.mod_attack = env_ms(2.0);
    s.mod_decay = env_ms(mod_dec);
    s.mod_sustain = 0.35;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(dec_ms);
    s.amp_sustain = sus;
    s.amp_release = env_ms(rel_ms);
    s.master_level = master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn piano_v1() -> (DeviceId, Vec<u8>) {
    piano_cfg(6000.0, 0.15, 0.12, -8.0, 6.0, 19.0, 0.62, 1200.0, 2600.0, 0.03, 250.0, 0.62)
}

pub fn piano_v2() -> (DeviceId, Vec<u8>) {
    piano_cfg(9000.0, 0.12, 0.14, -5.0, 4.0, 19.0, 0.60, 1000.0, 2000.0, 0.02, 200.0, 0.64)
}

pub fn piano_v3() -> (DeviceId, Vec<u8>) {
    piano_cfg(3800.0, 0.22, 0.10, -10.0, 8.0, 19.0, 0.55, 1200.0, 2200.0, 0.05, 300.0, 0.58)
}

pub fn piano_v4() -> (DeviceId, Vec<u8>) {
    piano_cfg(5200.0, 0.18, 0.10, -16.0, 12.0, 19.0, 0.50, 900.0, 2600.0, 0.04, 280.0, 0.60)
}

pub fn piano_v5() -> (DeviceId, Vec<u8>) {
    piano_cfg(6500.0, 0.15, 0.12, -8.0, 6.0, 24.0, 0.62, 1200.0, 2800.0, 0.03, 320.0, 0.62)
}

pub fn piano_v6() -> (DeviceId, Vec<u8>) {
    piano_cfg(4200.0, 0.20, 0.08, -8.0, 6.0, 19.0, 0.50, 700.0, 1200.0, 0.08, 200.0, 0.56)
}

pub fn piano_v7() -> (DeviceId, Vec<u8>) {
    piano_cfg(5000.0, 0.25, 0.10, -6.0, 5.0, 19.0, 0.45, 500.0, 900.0, 0.02, 150.0, 0.60)
}

pub fn piano_v8() -> (DeviceId, Vec<u8>) {
    piano_cfg(6200.0, 0.15, 0.14, -22.0, 20.0, 19.0, 0.60, 1100.0, 2400.0, 0.04, 250.0, 0.62)
}

pub fn piano_v9() -> (DeviceId, Vec<u8>) {
    piano_cfg(5500.0, 0.42, 0.10, -8.0, 6.0, 19.0, 0.60, 1400.0, 3400.0, 0.02, 400.0, 0.58)
}

pub fn piano_v10() -> (DeviceId, Vec<u8>) {
    piano_cfg(4800.0, 0.18, 0.08, -10.0, 8.0, 19.0, 0.50, 900.0, 2000.0, 0.06, 250.0, 0.56)
}