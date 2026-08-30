use crate::format::{env_ms, DeviceId, Falcon};

pub fn flute() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Flute v3: a clean, airy lead voice - a near-pure 1:1 core with just a
    // whisper of self-FM "breath", a soft blown attack and a warm vibrato.
    // Purer than v2 so the topline sits clearly on top.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.10;
    f.osc1_feed_forward = 0.30;
    f.osc1_attack = env_ms(18.0); // soft blown attack
    f.osc1_decay = env_ms(400.0);
    f.osc1_sustain = 0.42;
    f.osc1_release = env_ms(140.0);
    // Carrier: near-sine core with a hint of air.
    f.osc2_waveform = 0.03;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.12; // clean breath, not raspy
    f.osc2_attack = env_ms(14.0);
    f.osc2_decay = env_ms(600.0);
    f.osc2_sustain = 0.80;
    f.osc2_release = env_ms(200.0);
    // Warm but subtle vibrato.
    f.vibrato_freq = 4.8;
    f.vibrato_amount = 0.14;
    f.master_level = 0.62;
    (DeviceId::Falcon, f.chunk())
}
