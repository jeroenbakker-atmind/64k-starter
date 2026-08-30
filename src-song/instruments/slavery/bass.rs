use crate::format::{env_ms, DeviceId, Slaughter};

/// Slaughter bass: two detuned pulses through a short low-pass wobble, with a
/// soft noise thump on the attack and a light detuned unison for body.
pub fn bass() -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Osc1: main body (saw-ish pulse width 0.5).
    s.osc1_waveform = 0.50;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.9;
    // Osc3: bright sub-color an octave-low synced feel, slightly detuned.
    s.osc3_waveform = 0.0;
    s.osc3_pulse_width = Slaughter::pulse_width(0.5);
    s.osc3_volume = 0.35;
    s.osc3_detune_coarse = Slaughter::detune_coarse(0.0);
    s.osc3_detune_fine = Slaughter::detune_fine(-8.0);
    // Noise click to define the pluck.
    s.noise_volume = 0.06;
    // Fast low-pass wobble: open on attack, closing down onto the body.
    s.filter_type = 0.0;
    s.filter_freq = Slaughter::filter_freq_hz(420.0);
    s.filter_resonance = Slaughter::resonance(0.35);
    s.filter_mod_amt = 0.35;
    s.mod_attack = env_ms(2.0);
    s.mod_decay = env_ms(90.0);
    s.mod_sustain = 0.0;
    // Amp: snappy pluck that decays to a held note.
    s.amp_attack = env_ms(2.0);
    s.amp_decay = env_ms(220.0);
    s.amp_sustain = 0.55;
    s.amp_release = env_ms(70.0);
    // Slight downward settle after the attack.
    s.pitch_attack = env_ms(1.0);
    s.pitch_decay = env_ms(50.0);
    s.pitch_sustain = 0.0;
    s.pitch_env_amt = Slaughter::pitch_amt(-0.8);
    // Body: two unison voices, a touch of detune, centered pan.
    s.master_level = 0.62;
    s.voices_unisono = 0.13; // (0.13 * 15 + 1) = 2
    s.voices_detune = 0.10;
    s.voices_pan = 0.5;
    s.rise = 0.0;
    s.voice_mode = 0.0; // Polyphonic
    s.slide = 0.0;
    (DeviceId::Slaughter, s.chunk())
}