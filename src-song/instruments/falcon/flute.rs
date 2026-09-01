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

fn flute_cfg(
    waveform: f32,
    index: f32,
    growl: f32,
    air: f32,
    atk_ms: f32,
    dec_ms: f32,
    sus: f32,
    rel_ms: f32,
    vib_f: f32,
    vib_a: f32,
    uni: f32,
    det: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = growl;
    f.osc1_feed_forward = index;
    f.osc1_attack = env_ms(atk_ms);
    f.osc1_decay = env_ms(dec_ms);
    f.osc1_sustain = sus;
    f.osc1_release = env_ms(rel_ms);
    f.osc2_waveform = waveform;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = air;
    f.osc2_attack = env_ms(atk_ms * 0.7);
    f.osc2_decay = env_ms(dec_ms * 1.2);
    f.osc2_sustain = (sus + 0.25).min(0.95);
    f.osc2_release = env_ms(rel_ms * 1.3);
    f.vibrato_freq = vib_f;
    f.vibrato_amount = vib_a;
    f.voices_unisono = uni;
    f.voices_detune = det;
    f.master_level = master;
    (DeviceId::Falcon, f.chunk())
}

pub fn flute_soft() -> (DeviceId, Vec<u8>) {
    flute_cfg(0.00, 0.20, 0.06, 0.04, 28.0, 900.0, 0.68, 320.0, 4.6, 0.10, 0.0, 0.0, 0.58)
}

pub fn flute_airy() -> (DeviceId, Vec<u8>) {
    flute_cfg(0.02, 0.26, 0.12, 0.06, 60.0, 820.0, 0.80, 420.0, 4.7, 0.16, 0.0, 0.0, 0.64)
}

pub fn flute_bright() -> (DeviceId, Vec<u8>) {
    flute_cfg(0.03, 0.30, 0.10, 0.06, 24.0, 600.0, 0.60, 250.0, 4.4, 0.20, 0.0, 0.0, 0.62)
}
