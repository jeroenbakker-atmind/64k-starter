use common::{DeviceId, Falcon, env_ms};

pub fn tenor_sax() -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Tenor sax v2: keeps the 1:1 FM core (rich harmonic series) but adds
    // the things the v1 lacked - a breathy attack scoop, a hotter FM index
    // so the low register cuts, reedy square-partial body, a growl, and a
    // rounder tail.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feedback = 0.42; // self-FM growl on the modulator (blow edge)
    f.osc1_feed_forward = 0.68; // FM index into the carrier (brightness bite)
    f.osc1_attack = env_ms(12.0); // soft "blow-in", no key click
    f.osc1_decay = env_ms(620.0); // index settles slowly during the note
    f.osc1_sustain = 0.56; // long notes round off instead of screaming
    f.osc1_release = env_ms(180.0); // breathy tail
    // Carrier: sine + square partials for reedy body, mild self-FM for air.
    f.osc2_waveform = 0.22;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = 0.24;
    f.osc2_attack = env_ms(9.0);
    f.osc2_decay = env_ms(900.0);
    f.osc2_sustain = 0.68;
    f.osc2_release = env_ms(220.0);
    // Expressive vocal vibrato, a touch deeper than v1.
    f.vibrato_freq = 5.6;
    f.vibrato_amount = 0.26;
    f.master_level = 0.55;
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

fn sax_cfg(
    index: f32,
    growl: f32,
    waveform: f32,
    atk_ms: f32,
    dec_ms: f32,
    sus: f32,
    rel_ms: f32,
    vib_f: f32,
    vib_a: f32,
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
    f.osc2_feedback = growl * 0.55;
    f.osc2_attack = env_ms(atk_ms * 0.75);
    f.osc2_decay = env_ms(dec_ms * 1.4);
    f.osc2_sustain = (sus + 0.15).min(0.95);
    f.osc2_release = env_ms(rel_ms * 1.2);
    f.vibrato_freq = vib_f;
    f.vibrato_amount = vib_a;
    f.master_level = master;
    (DeviceId::Falcon, f.chunk())
}

pub fn sax_v1() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.55, 0.30, 0.18, 14.0, 620.0, 0.58, 190.0, 5.6, 0.24, 0.56)
}

pub fn sax_v2() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.62, 0.50, 0.20, 8.0, 550.0, 0.55, 160.0, 5.8, 0.30, 0.58)
}

pub fn sax_v3() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.75, 0.30, 0.30, 6.0, 450.0, 0.50, 140.0, 5.5, 0.22, 0.62)
}

pub fn sax_v4() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.45, 0.20, 0.10, 20.0, 900.0, 0.68, 300.0, 5.2, 0.18, 0.54)
}

pub fn sax_v5() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.70, 0.35, 0.25, 4.0, 480.0, 0.55, 140.0, 5.5, 0.20, 0.66)
}

pub fn sax_v6() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.50, 0.42, 0.15, 16.0, 700.0, 0.60, 220.0, 5.4, 0.20, 0.56)
}

pub fn sax_v7() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.40, 0.22, 0.00, 10.0, 700.0, 0.50, 150.0, 5.6, 0.14, 0.60)
}

pub fn sax_v8() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.58, 0.30, 0.18, 40.0, 900.0, 0.62, 300.0, 5.3, 0.32, 0.58)
}

pub fn sax_v9() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.90, 0.55, 0.35, 2.0, 380.0, 0.48, 120.0, 5.9, 0.30, 0.60)
}

pub fn sax_v10() -> (DeviceId, Vec<u8>) {
    sax_cfg(0.60, 0.32, 0.16, 26.0, 800.0, 0.65, 260.0, 5.4, 0.34, 0.56)
}
