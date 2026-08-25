use crate::format::{DeviceId, Falcon, env_ms};

pub fn kick() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc2_decay = env_ms(280.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(60.0);
    f.pitch_env_amt2 = Falcon::pitch_amt(18.0);
    f.pitch_decay = env_ms(50.0);
    f.pitch_sustain = 0.0;
    f.pitch_release = env_ms(30.0);
    f.master_level = 0.95;
    (DeviceId::Falcon, f.chunk())
}

pub fn snare() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(4);
    f.osc1_decay = env_ms(60.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.85;
    f.osc2_feedback = 0.72;
    f.osc2_decay = env_ms(120.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(80.0);
    f.master_level = 0.70;
    (DeviceId::Falcon, f.chunk())
}

pub fn closed_hat() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(16);
    f.osc1_waveform = 0.5;
    f.osc1_decay = env_ms(30.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.90;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(8);
    f.osc2_waveform = 0.5;
    f.osc2_feedback = 0.80;
    f.osc2_decay = env_ms(40.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(40.0);
    f.master_level = 0.55;
    (DeviceId::Falcon, f.chunk())
}

pub fn open_hat() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(16);
    f.osc1_waveform = 0.5;
    f.osc1_decay = env_ms(120.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.90;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(8);
    f.osc2_waveform = 0.5;
    f.osc2_feedback = 0.80;
    f.osc2_decay = env_ms(240.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(100.0);
    f.master_level = 0.50;
    (DeviceId::Falcon, f.chunk())
}

pub fn crash() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(21);
    f.osc1_waveform = 0.7;
    f.osc1_decay = env_ms(200.0);
    f.osc1_sustain = 0.1;
    f.osc1_feed_forward = 0.95;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(13);
    f.osc2_waveform = 0.6;
    f.osc2_feedback = 0.85;
    f.osc2_decay = env_ms(800.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(500.0);
    f.master_level = 0.60;
    (DeviceId::Falcon, f.chunk())
}
