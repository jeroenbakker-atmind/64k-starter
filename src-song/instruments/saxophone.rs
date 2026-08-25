use crate::format::{DeviceId, Falcon, env_ms};

pub fn tenor_sax() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Tenor sax: 1:1 ratio for the rich harmonic series, moderate index.
    // The slow-decaying modulator envelope creates the characteristic
    // timbral evolution during sustained notes.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.35;
    f.osc1_feed_forward = 0.65;
    f.osc1_attack = env_ms(5.0);
    f.osc1_decay = env_ms(500.0);
    f.osc1_sustain = 0.50;
    f.osc1_release = env_ms(140.0);
    // Carrier with some square partials for reedy body.
    f.osc2_waveform = 0.12;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.18;
    f.osc2_attack = env_ms(4.0);
    f.osc2_decay = env_ms(800.0);
    f.osc2_sustain = 0.65;
    f.osc2_release = env_ms(200.0);
    f.vibrato_freq = 5.2;
    f.vibrato_amount = 0.22;
    f.master_level = 0.70;
    (DeviceId::Falcon, f.chunk())
}

pub fn alto_sax() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Alto sax: brighter, slightly higher index than tenor.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.40;
    f.osc1_feed_forward = 0.72;
    f.osc1_attack = env_ms(4.0);
    f.osc1_decay = env_ms(450.0);
    f.osc1_sustain = 0.55;
    f.osc1_release = env_ms(130.0);
    f.osc2_waveform = 0.15;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.22;
    f.osc2_attack = env_ms(3.0);
    f.osc2_decay = env_ms(700.0);
    f.osc2_sustain = 0.70;
    f.osc2_release = env_ms(180.0);
    f.vibrato_freq = 5.5;
    f.vibrato_amount = 0.20;
    f.master_level = 0.68;
    (DeviceId::Falcon, f.chunk())
}
