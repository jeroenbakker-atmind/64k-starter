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

fn piano_cfg(
    index: f32,
    growl: f32,
    waveform: f32,
    atk_ms: f32,
    index_dec_ms: f32,
    index_sus: f32,
    dec_ms: f32,
    sus: f32,
    rel_ms: f32,
    uni: f32,
    det: f32,
    vib_f: f32,
    vib_a: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = growl;
    f.osc1_attack = env_ms(atk_ms);
    f.osc1_decay = env_ms(index_dec_ms);
    f.osc1_sustain = index_sus;
    f.osc1_release = env_ms(120.0);
    f.osc1_feed_forward = index;
    f.osc2_waveform = waveform;
    f.osc2_feedback = atk_ms.clamp(1.0, 500.0) / 2000.0;
    f.osc2_attack = env_ms(atk_ms);
    f.osc2_decay = env_ms(dec_ms);
    f.osc2_sustain = sus;
    f.osc2_release = env_ms(rel_ms);
    f.vibrato_freq = vib_f;
    f.vibrato_amount = vib_a;
    f.voices_unisono = uni;
    f.voices_detune = det;
    f.master_level = master;
    (DeviceId::Falcon, f.chunk())
}

pub fn piano_v1() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.55, 0.15, 0.03, 1.0, 260.0, 0.30, 2600.0, 0.08, 700.0, 0.0, 0.0, 0.0, 0.0, 0.60)
}

pub fn piano_v2() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.85, 0.10, 0.15, 1.0, 100.0, 0.00, 1800.0, 0.06, 400.0, 0.0, 0.0, 0.0, 0.0, 0.62)
}

pub fn piano_v3() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.95, 0.12, 0.00, 1.0, 90.0, 0.00, 2400.0, 0.00, 900.0, 0.0, 0.0, 0.0, 0.0, 0.58)
}

pub fn piano_v4() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.35, 0.10, 0.02, 8.0, 300.0, 0.30, 1400.0, 0.10, 400.0, 0.0, 0.0, 0.0, 0.0, 0.58)
}

pub fn piano_v5() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.60, 0.15, 0.04, 1.0, 160.0, 0.10, 2600.0, 0.06, 600.0, 0.50, 0.30, 0.0, 0.0, 0.62)
}

pub fn piano_v6() -> (DeviceId, Vec<u8>) {
    piano_cfg(1.00, 0.10, 0.22, 1.0, 120.0, 0.00, 3000.0, 0.00, 1200.0, 0.0, 0.0, 0.0, 0.0, 0.60)
}

pub fn piano_v7() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.42, 0.30, 0.00, 2.0, 140.0, 0.40, 2000.0, 0.12, 500.0, 0.0, 0.0, 0.0, 0.0, 0.58)
}

pub fn piano_v8() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.50, 0.10, 0.00, 1.0, 70.0, 0.00, 700.0, 0.00, 300.0, 0.0, 0.0, 6.8, 0.05, 0.60)
}

pub fn piano_v9() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.70, 0.50, 0.08, 1.0, 150.0, 0.05, 3200.0, 0.02, 800.0, 0.0, 0.0, 0.0, 0.0, 0.60)
}

pub fn piano_v10() -> (DeviceId, Vec<u8>) {
    piano_cfg(0.80, 0.12, 0.10, 0.5, 60.0, 0.00, 2200.0, 0.04, 500.0, 0.0, 0.0, 0.0, 0.0, 0.68)
}
