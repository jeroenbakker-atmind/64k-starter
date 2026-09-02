//! Slaughter clarinet — the reed and the bore as a resonator.
//!
//! A clarinet is a closed cylinder: hollow, woody, ringing on the odd
//! harmonics. Slaughter models that with a resonant filter sitting on top of
//! the oscillator stack — the filter formant IS the chalumeau body. A small
//! pitch-bend envelope scopes into each note like a breath attack, and the
//! `mod_env` opens the cutoff with the swell.

use common::{DeviceId, Slaughter, env_ms};

pub fn clarinet_legato() -> (DeviceId, Vec<u8>) {
    // Smooth, full-bodied swell - unhurried entrance, round tail.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.28;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.9;
    s.osc2_waveform = 0.22;
    s.osc2_pulse_width = Slaughter::pulse_width(0.5);
    s.osc2_volume = 0.55;
    s.osc2_detune_fine = Slaughter::detune_fine(6.0);
    s.osc3_waveform = 0.18;
    s.osc3_pulse_width = Slaughter::pulse_width(0.5);
    s.osc3_volume = 0.35;
    s.osc3_detune_fine = Slaughter::detune_fine(-5.0);
    s.noise_volume = 0.05;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(1650.0);
    s.filter_resonance = Slaughter::resonance(0.26);
    s.filter_mod_amt = 0.50;
    s.mod_attack = env_ms(45.0);
    s.mod_decay = env_ms(760.0);
    s.mod_sustain = 0.76;
    s.pitch_attack = env_ms(2.0);
    s.pitch_decay = env_ms(160.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(60.0);
    s.pitch_env_amt = Slaughter::pitch_amt(1.0);
    s.amp_attack = env_ms(46.0);
    s.amp_decay = env_ms(800.0);
    s.amp_sustain = 0.76;
    s.amp_release = env_ms(330.0);
    s.vibrato_freq = 4.9;
    s.vibrato_amount = 0.22;
    s.master_level = 0.56;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}
