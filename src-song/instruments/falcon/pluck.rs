//! Falcon violin pluck — pizzicato string, 5 stylings.
//!
//! A plucked violin string snaps: an almost instantaneous attack, a bright
//! transient, then a ring whose higher partials decay faster than the
//! fundamental. Falcon's FM expresses that cleanly — a ratio-1 modulator
//! feeding a ratio-1 carrier with a deep, *fast-decaying* FM index folds the
//! sidebands onto the full string harmonic series, and as the index falls the
//! tone naturally dulls into the body of the note.

use crate::format::{DeviceId, Falcon, env_ms};

fn pluck_cfg(
    index: f32,
    index_dec_ms: f32,
    index_sus: f32,
    waveform: f32,
    feedback: f32,
    atk_ms: f32,
    dec_ms: f32,
    sus: f32,
    rel_ms: f32,
    uni: f32,
    det: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    // Modulator: ratio 1 folds sidebands onto every harmonic; a hot, quick-decay
    // index is the pluck transient that settles into the ringing string.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc1_feed_forward = index;
    f.osc1_feedback = feedback;
    f.osc1_attack = env_ms(atk_ms);
    f.osc1_decay = env_ms(index_dec_ms);
    f.osc1_sustain = index_sus;
    f.osc1_release = env_ms(rel_ms * 0.6);
    // Carrier: at pitch, faint square partials for string brightness.
    f.osc2_waveform = waveform;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = feedback * 0.5;
    f.osc2_attack = env_ms(atk_ms);
    f.osc2_decay = env_ms(dec_ms);
    f.osc2_sustain = sus;
    f.osc2_release = env_ms(rel_ms);
    f.voices_unisono = uni;
    f.voices_detune = det;
    f.master_level = master;
    (DeviceId::Falcon, f.chunk())
}

pub fn pluck() -> (DeviceId, Vec<u8>) {
    pluck_warm()
}

pub fn pluck_warm() -> (DeviceId, Vec<u8>) {
    // Soft, warm pizz: gentler transient, quick settle, short body.
    pluck_cfg(0.65, 120.0, 0.12, 0.04, 0.08, 1.5, 900.0, 0.08, 220.0, 0.0, 0.0, 0.56)
}

pub fn pluck_picked() -> (DeviceId, Vec<u8>) {
    // Snappy picked bass-string: very fast sub-100ms transient, dry and tight.
    pluck_cfg(0.70, 60.0, 0.00, 0.02, 0.10, 0.5, 700.0, 0.00, 260.0, 0.0, 0.0, 0.58)
}
