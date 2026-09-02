//! Falcon clarinet — 10 single-reed patches, designed from the synth's raw FM
//! mechanics rather than ported from any existing library member.
//!
//! Physics: a clarinet is a *closed* cylinder, so it resonates on the odd
//! harmonics (1st, 3rd, 5th…) and its tone is hollow and woody, not the open
//! smearing of a saxophone.
//!
//! How the synth expresses that:
//! - osc1 is the *modulator* (its feed-forward is the FM index into the carrier
//!   below). Driving it at ratio 2 (one octave up) folds its sidebands onto the
//!   odd partials — the classic "FM clarinet" core — and a modest index keeps
//!   the tone hollow rather than brass-like.
//! - osc2 is the *carrier*. A little `waveform` mixes in the square's 3rd+5th
//!   (square35 is odd-only), which reinforces the odd-harmonic ring without
//!   adding even harmonics, further away from any sax family.
//!
//! The ten variations are stylings (bright/dark, fast/slow attack, dry/lush
//! vibrato, breathy, growly) all sharing that one clarinet identity.

use common::{DeviceId, Falcon, env_ms};

fn clarinet_cfg(
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
    // Modulator: one octave up -> folds the FM onto odd harmonics (the hollow,
    // woody clarinet core). Feedback softens the reed into a growl when raised.
    f.osc1_ratio_coarse = Falcon::ratio_coarse(2);
    f.osc1_feedback = growl;
    f.osc1_feed_forward = index;
    f.osc1_attack = env_ms(atk_ms);
    f.osc1_decay = env_ms(dec_ms);
    f.osc1_sustain = sus;
    f.osc1_release = env_ms(rel_ms);
    // Carrier: at pitch, odd-harmonic square blend bolsters the ring.
    f.osc2_waveform = waveform;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(1);
    f.osc2_feedback = index * 0.45;
    f.osc2_attack = env_ms(atk_ms * 0.85);
    f.osc2_decay = env_ms(dec_ms * 1.3);
    f.osc2_sustain = (sus + 0.25).min(0.95);
    f.osc2_release = env_ms(rel_ms * 1.3);
    f.vibrato_freq = vib_f;
    f.vibrato_amount = vib_a;
    f.master_level = master;
    (DeviceId::Falcon, f.chunk())
}

pub fn clarinet_dark() -> (DeviceId, Vec<u8>) {
    // Dark: low, smoky throat register - thick and covered, the clarinet's
    // melancholy low end.
    clarinet_cfg(0.28, 0.10, 0.02, 20.0, 680.0, 0.80, 260.0, 4.5, 0.08, 0.50)
}

pub fn clarinet_vibrato() -> (DeviceId, Vec<u8>) {
    // Vibrato: a lyrical, singing line with the lip-vibrato turned up.
    clarinet_cfg(0.42, 0.14, 0.08, 10.0, 460.0, 0.65, 190.0, 5.2, 0.30, 0.58)
}

pub fn clarinet_legato() -> (DeviceId, Vec<u8>) {
    // Legato: a smooth, full-bodied swell, unhurried entrance and round tail.
    clarinet_cfg(0.50, 0.16, 0.10, 44.0, 760.0, 0.72, 320.0, 4.9, 0.22, 0.58)
}

pub fn clarinet_ballad() -> (DeviceId, Vec<u8>) {
    // Ballad: a hushed, pillowy tone that floats in gently and lingers.
    clarinet_cfg(0.32, 0.08, 0.04, 32.0, 800.0, 0.75, 300.0, 4.6, 0.14, 0.48)
}
