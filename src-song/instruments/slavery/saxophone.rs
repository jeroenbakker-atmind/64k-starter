//! Slaughter saxophones — divergent from `falcon::saxophone`.
//!
//! Improvement: Falcon's reeds are FM index + vibrato; here the reed itself
//! is the filter. A resonant low-pass on detuned pulse overtones gives the
//! woody/reedy formant, a pitch scoop bends into each note like a breath
//! attack, and white noise adds the air. Voices stay polyphonic so brass
//! section lines with simultaneous notes keep their voicings.

use crate::format::{DeviceId, Slaughter, env_ms};

fn reed(
    cutoff_hz: f32,
    resonance: f32,
    detune: f32,
    scoop: f32,
    attack_ms: f32,
    release_ms: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Saw-ward pulse timbre (rich odd/even overtones to hit), lightly stacked.
    s.osc1_waveform = 0.60;
    s.osc1_pulse_width = Slaughter::pulse_width(0.42);
    s.osc1_volume = 0.9;
    s.osc2_waveform = 0.55;
    s.osc2_pulse_width = Slaughter::pulse_width(0.42);
    s.osc2_volume = 0.6;
    s.osc2_detune_fine = Slaughter::detune_fine(7.0);
    s.osc3_waveform = 0.50;
    s.osc3_pulse_width = Slaughter::pulse_width(0.48);
    s.osc3_volume = 0.4;
    s.osc3_detune_fine = Slaughter::detune_fine(-6.0);
    // Breath.
    s.noise_volume = 0.06;
    // The reed formant: resonant low-pass that opens past the cutoff during
    // the swell, then settles warm.
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(cutoff_hz);
    s.filter_resonance = Slaughter::resonance(resonance);
    s.filter_mod_amt = 0.62;
    s.mod_attack = env_ms(20.0);
    s.mod_decay = env_ms(300.0);
    s.mod_sustain = 0.55;
    // Attack "breath scoop": a quick bend up into the note, then settle.
    s.pitch_attack = env_ms(2.0);
    s.pitch_decay = env_ms(180.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(60.0);
    s.pitch_env_amt = Slaughter::pitch_amt(scoop);
    s.amp_attack = env_ms(attack_ms);
    s.amp_decay = env_ms(650.0);
    s.amp_sustain = 0.60;
    s.amp_release = env_ms(release_ms);
    s.vibrato_freq = 5.6;
    s.vibrato_amount = detune;
    s.master_level = master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn tenor_sax() -> (DeviceId, Vec<u8>) {
    reed(1100.0, 0.35, 0.28, 2.0, 15.0, 200.0, 0.50)
}

pub fn alto_sax() -> (DeviceId, Vec<u8>) {
    reed(1800.0, 0.30, 0.22, 1.5, 5.0, 180.0, 0.62)
}