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