use crate::format::{DeviceId, Falcon, env_ms};

pub fn piano() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(2);
    f.osc1_feedback = 0.30;
    f.osc1_decay = env_ms(350.0);
    f.osc1_sustain = 0.25;
    f.osc1_feed_forward = 0.62;
    f.osc2_waveform = 0.08;
    f.osc2_feedback = 0.12;
    f.osc2_decay = env_ms(1400.0);
    f.osc2_sustain = 0.45;
    f.osc2_release = env_ms(450.0);
    f.master_level = 0.75;
    (DeviceId::Falcon, f.chunk())
}
