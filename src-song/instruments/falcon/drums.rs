use crate::format::{env_ms, DeviceId, Falcon};

pub fn kick() -> (DeviceId, Vec<u8>) {
    // Kick: a sine body whose pitch envelope drops 24 semitones (261 Hz ->
    // the 65 Hz fundamental in ~40 ms), with a short ratio-2 modulator tick
    // for the beater click.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(2);
    f.osc1_decay = env_ms(12.0);
    f.osc1_sustain = 0.0;
    f.osc1_release = env_ms(8.0);
    f.osc1_feed_forward = 0.70;
    f.osc2_waveform = 0.02;
    f.osc2_feedback = 0.12;
    f.osc2_decay = env_ms(260.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(60.0);
    f.pitch_env_amt2 = Falcon::pitch_amt(24.0);
    f.pitch_decay = env_ms(40.0);
    f.pitch_sustain = 0.0;
    f.pitch_release = env_ms(30.0);
    f.master_level = 0.95;
    (DeviceId::Falcon, f.chunk())
}

pub fn snare() -> (DeviceId, Vec<u8>) {
    // Snare: a high-ratio modulator crack over a noise-burst body (heavy
    // carrier self-feedback), whose pitch slaps down ~14 semitones for the
    // tonal thump under the rattle.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(6);
    f.osc1_feedback = 0.25;
    f.osc1_decay = env_ms(20.0);
    f.osc1_sustain = 0.0;
    f.osc1_release = env_ms(10.0);
    f.osc1_feed_forward = 0.85;
    f.osc2_waveform = 0.15;
    f.osc2_feedback = 0.72;
    f.osc2_decay = env_ms(150.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(70.0);
    f.pitch_env_amt2 = Falcon::pitch_amt(14.0);
    f.pitch_decay = env_ms(45.0);
    f.pitch_sustain = 0.0;
    f.pitch_release = env_ms(35.0);
    f.master_level = 0.72;
    (DeviceId::Falcon, f.chunk())
}

pub fn closed_hat() -> (DeviceId, Vec<u8>) {
    // Closed hat: high ratios + heavy feedback on both operators for a bright
    // metallic sizzle, choked to ~45 ms for a tight, swung tick.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(16);
    f.osc1_waveform = 0.5;
    f.osc1_decay = env_ms(28.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.92;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(10);
    f.osc2_waveform = 0.55;
    f.osc2_feedback = 0.82;
    f.osc2_decay = env_ms(45.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(40.0);
    f.master_level = 0.55;
    (DeviceId::Falcon, f.chunk())
}

pub fn open_hat() -> (DeviceId, Vec<u8>) {
    // Open hat: same metallic recipe but left to ring ~300 ms.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(16);
    f.osc1_waveform = 0.5;
    f.osc1_decay = env_ms(140.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.92;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(10);
    f.osc2_waveform = 0.55;
    f.osc2_feedback = 0.82;
    f.osc2_decay = env_ms(300.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(120.0);
    f.master_level = 0.50;
    (DeviceId::Falcon, f.chunk())
}

pub fn shaker() -> (DeviceId, Vec<u8>) {
    // Shaker: a soft bright noise burst that rings ~100 ms — the swung
    // offbeat glue between the hats and the backbeat.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(12);
    f.osc1_waveform = 0.4;
    f.osc1_decay = env_ms(90.0);
    f.osc1_sustain = 0.0;
    f.osc1_feed_forward = 0.90;
    f.osc1_feedback = 0.30;
    f.osc2_waveform = 0.6;
    f.osc2_feedback = 0.30;
    f.osc2_decay = env_ms(140.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(80.0);
    f.master_level = 0.40;
    (DeviceId::Falcon, f.chunk())
}

pub fn crash() -> (DeviceId, Vec<u8>) {
    // Crash: two high-ratio noisy operators with a longer decay for a washier
    // sheet-metal shimmer.
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(24);
    f.osc1_waveform = 0.7;
    f.osc1_decay = env_ms(160.0);
    f.osc1_sustain = 0.04;
    f.osc1_feed_forward = 0.95;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(15);
    f.osc2_waveform = 0.6;
    f.osc2_feedback = 0.88;
    f.osc2_decay = env_ms(850.0);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(600.0);
    f.master_level = 0.60;
    (DeviceId::Falcon, f.chunk())
}
