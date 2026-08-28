use crate::format::{DeviceId, Falcon, env_ms};

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
