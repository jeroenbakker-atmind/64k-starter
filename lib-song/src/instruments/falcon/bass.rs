use common::{env_ms, DeviceId, Falcon};

pub fn bass() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(2);
    f.osc1_decay = env_ms(80.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.50;
    f.osc2_waveform = 0.25;
    f.osc2_feedback = 0.35;
    f.osc2_decay = env_ms(250.0);
    f.osc2_sustain = 0.40;
    f.osc2_release = env_ms(90.0);
    // Slap: a short downward pitch sweep pops every note for bounce.
    f.pitch_env_amt2 = Falcon::pitch_amt(10.0);
    f.pitch_decay = env_ms(40.0);
    f.pitch_sustain = 0.0;
    f.pitch_release = env_ms(40.0);
    f.master_level = 0.68;
    (DeviceId::Falcon, f.chunk())
}
