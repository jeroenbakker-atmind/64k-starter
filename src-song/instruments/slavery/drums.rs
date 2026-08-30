//! Slaughter percussion set — intentional divergence from `falcon::drums`.
//!
//! Where Falcon made percussion from FM/phase-noise, the subtractive arch
//! gets its punch from detuned BLIT-pulse stacks + noise shaped by fast
//! state-variable-filter sweeps. Key improvements over the Falcon set:
//!
//! - kick: the mod env opens the filter onto the beater click, then the body
//!   pitches down +24 -> 0 semitones and the filter closes for a chest boom.
//! - snare: woody band-passed crack + ringing rim partial under the rattle.
//! - hats: three inharmonic detuned pulses (plus noise) through a bright
//!   band-pass instead of Falcon's two feedback FM ops, so they shimmer.

use crate::format::{DeviceId, Slaughter, env_ms};

/// Bright metallic stack: three inharmonic detuned pulses in a band-pass.
fn metal_stack(s: &mut Slaughter) {
    s.osc1_waveform = 0.50;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.85;
    s.osc1_detune_coarse = Slaughter::detune_coarse(29.0);
    s.osc1_detune_fine = Slaughter::detune_fine(-9.0);
    s.osc2_waveform = 0.50;
    s.osc2_pulse_width = Slaughter::pulse_width(0.45);
    s.osc2_volume = 0.65;
    s.osc2_detune_coarse = Slaughter::detune_coarse(24.0);
    s.osc2_detune_fine = Slaughter::detune_fine(5.0);
    s.osc3_waveform = 0.55;
    s.osc3_pulse_width = Slaughter::pulse_width(0.3);
    s.osc3_volume = 0.55;
    s.osc3_detune_coarse = Slaughter::detune_coarse(19.0);
    s.osc3_detune_fine = Slaughter::detune_fine(11.0);
    s.noise_volume = 0.35;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(10000.0);
    s.filter_resonance = Slaughter::resonance(0.35);
}

pub fn kick() -> (DeviceId, Vec<u8>) {
    // Kick: beater click through an opening filter, then the body pitches
    // down +24 -> 0 semitones while the filter closes around it.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 1.0;
    s.osc2_waveform = 0.0;
    s.osc2_pulse_width = Slaughter::pulse_width(0.25);
    s.osc2_volume = 0.5;
    s.osc2_detune_coarse = Slaughter::detune_coarse(24.0);
    s.noise_volume = 0.12;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(200.0);
    s.filter_resonance = Slaughter::resonance(0.30);
    s.filter_mod_amt = 0.55;
    s.mod_attack = env_ms(3.0);
    s.mod_decay = env_ms(90.0);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(280.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(70.0);
    s.pitch_attack = env_ms(1.0);
    s.pitch_decay = env_ms(42.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(30.0);
    s.pitch_env_amt = Slaughter::pitch_amt(24.0);
    s.master_level = 0.95;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn snare() -> (DeviceId, Vec<u8>) {
    // Snare: band-passed tonal crack with a ringing rim partial (up two
    // octaves) under the noise rattle; pitch slaps down for the body.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.85;
    s.osc3_waveform = 0.0;
    s.osc3_pulse_width = Slaughter::pulse_width(0.28);
    s.osc3_volume = 0.6;
    s.osc3_detune_coarse = Slaughter::detune_coarse(24.0);
    s.noise_volume = 0.6;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(3500.0);
    s.filter_resonance = Slaughter::resonance(0.45);
    s.filter_mod_amt = 0.5;
    s.mod_attack = env_ms(1.0);
    s.mod_decay = env_ms(55.0);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(170.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(80.0);
    s.pitch_attack = env_ms(1.0);
    s.pitch_decay = env_ms(30.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(30.0);
    s.pitch_env_amt = Slaughter::pitch_amt(10.0);
    s.master_level = 0.70;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn closed_hat() -> (DeviceId, Vec<u8>) {
    // Closed hat: metallic stack choked to ~30 ms for a tight swung tick.
    let mut s = Slaughter::default();
    metal_stack(&mut s);
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(30.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(25.0);
    s.master_level = 0.50;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn open_hat() -> (DeviceId, Vec<u8>) {
    // Open hat: same stack left to ring ~260 ms with a bit more noise air.
    let mut s = Slaughter::default();
    metal_stack(&mut s);
    s.noise_volume = 0.45;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(260.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(100.0);
    s.master_level = 0.48;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn shaker() -> (DeviceId, Vec<u8>) {
    // Shaker: soft band-passed noise pill with a sprinkle of detuned pulse —
    // the swung offbeat glue between the hats and the backbeat.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.40;
    s.osc1_pulse_width = Slaughter::pulse_width(0.35);
    s.osc1_volume = 0.30;
    s.osc1_detune_coarse = Slaughter::detune_coarse(24.0);
    s.osc1_detune_fine = Slaughter::detune_fine(-7.0);
    s.noise_volume = 0.50;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(8200.0);
    s.filter_resonance = Slaughter::resonance(0.40);
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(130.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(60.0);
    s.master_level = 0.38;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn crash() -> (DeviceId, Vec<u8>) {
    // Crash: big wash — three detuned pulses + heavy noise through a
    // high-pass that sizzles, with a long decay and a hint of sustain.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.60;
    s.osc1_pulse_width = Slaughter::pulse_width(0.4);
    s.osc1_volume = 0.8;
    s.osc1_detune_coarse = Slaughter::detune_coarse(24.0);
    s.osc1_detune_fine = Slaughter::detune_fine(3.0);
    s.osc2_waveform = 0.55;
    s.osc2_pulse_width = Slaughter::pulse_width(0.3);
    s.osc2_volume = 0.65;
    s.osc2_detune_coarse = Slaughter::detune_coarse(12.0);
    s.osc2_detune_fine = Slaughter::detune_fine(-17.0);
    s.osc3_waveform = 0.60;
    s.osc3_pulse_width = Slaughter::pulse_width(0.5);
    s.osc3_volume = 0.6;
    s.osc3_detune_coarse = Slaughter::detune_coarse(17.0);
    s.osc3_detune_fine = Slaughter::detune_fine(21.0);
    s.noise_volume = 0.75;
    s.filter_type = 1.0; // highpass
    s.filter_freq = Slaughter::filter_freq_hz(4000.0);
    s.filter_resonance = Slaughter::resonance(0.25);
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(900.0);
    s.amp_sustain = 0.03;
    s.amp_release = env_ms(700.0);
    s.master_level = 0.58;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}