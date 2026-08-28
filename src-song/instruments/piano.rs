use crate::format::{env_ms, DeviceId, Falcon};

pub fn piano() -> (DeviceId, Vec<u8>) {
    // Acoustic-grand-ish FM recipe: a deep, fast-decaying index (ratio 1, so
    // the partials ring like a vibrating string) settles into a long, decaying
    // carrier body with no real sustain.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.15;
    f.osc1_attack = env_ms(1.0);
    f.osc1_decay = env_ms(160.0);
    f.osc1_sustain = 0.10;
    f.osc1_release = env_ms(120.0);
    f.osc1_feed_forward = 0.60;
    f.osc2_waveform = 0.04;
    f.osc2_feedback = 0.08;
    f.osc2_attack = env_ms(1.0);
    f.osc2_decay = env_ms(2800.0);
    f.osc2_sustain = 0.04;
    f.osc2_release = env_ms(700.0);
    f.master_level = 0.60;
    (DeviceId::Falcon, f.chunk())
}
