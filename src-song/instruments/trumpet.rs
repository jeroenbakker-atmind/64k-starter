use crate::format::{DeviceId, Falcon, env_ms};

pub fn trumpet() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // 1:1 ratio gives sawtooth-like harmonics - the core brass character.
    // Heavy feed-forward creates the bright, brassy attack.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.45;
    f.osc1_feed_forward = 0.78;
    f.osc1_attack = env_ms(3.0);
    f.osc1_decay = env_ms(180.0);
    f.osc1_sustain = 0.55;
    f.osc1_release = env_ms(120.0);
    // Carrier: mostly sine with a hint of square for body.
    f.osc2_waveform = 0.05;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.20;
    f.osc2_attack = env_ms(2.0);
    f.osc2_decay = env_ms(300.0);
    f.osc2_sustain = 0.70;
    f.osc2_release = env_ms(150.0);
    // Vibrato for natural brass wobble.
    f.vibrato_freq = 5.5;
    f.vibrato_amount = 0.18;
    f.master_level = 0.72;
    (DeviceId::Falcon, f.chunk())
}

pub fn trumpet_mute() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Muted trumpet: lower index, more nasal with ratio-2 modulator.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(2);
    f.osc1_feedback = 0.30;
    f.osc1_feed_forward = 0.55;
    f.osc1_attack = env_ms(4.0);
    f.osc1_decay = env_ms(200.0);
    f.osc1_sustain = 0.40;
    f.osc1_release = env_ms(100.0);
    f.osc2_waveform = 0.10;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.35;
    f.osc2_attack = env_ms(3.0);
    f.osc2_decay = env_ms(250.0);
    f.osc2_sustain = 0.60;
    f.osc2_release = env_ms(120.0);
    f.vibrato_freq = 6.0;
    f.vibrato_amount = 0.25;
    f.master_level = 0.65;
    (DeviceId::Falcon, f.chunk())
}
