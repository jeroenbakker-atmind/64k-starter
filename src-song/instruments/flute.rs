use crate::format::{DeviceId, Falcon, env_ms};

pub fn flute() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Flute: 1:1 ratio, very low modulation index for a pure, breathy tone.
    // The characteristic of a flute is its purity - keep the index low.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.10;
    f.osc1_feed_forward = 0.30;
    f.osc1_attack = env_ms(15.0);
    f.osc1_decay = env_ms(400.0);
    f.osc1_sustain = 0.35;
    f.osc1_release = env_ms(100.0);
    // Carrier: pure sine, breathy.
    f.osc2_waveform = 0.02;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.05;
    f.osc2_attack = env_ms(12.0);
    f.osc2_decay = env_ms(600.0);
    f.osc2_sustain = 0.80;
    f.osc2_release = env_ms(180.0);
    // Gentle vibrato for the characteristic flute wobble.
    f.vibrato_freq = 5.0;
    f.vibrato_amount = 0.12;
    f.master_level = 0.60;
    (DeviceId::Falcon, f.chunk())
}
