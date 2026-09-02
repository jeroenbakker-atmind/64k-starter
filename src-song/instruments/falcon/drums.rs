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

// ---------------------------------------------------------------------------
// FM cymbal family (closed/open hat, crash, ride). The FM engine's high,
// ratio-offset operators with heavy feedback make a glassier, more inharmonic
// metallic tone than the Slaughter pulse stacks — hats and cymbals are where
// Falcon genuinely wins. Each is a named one-shot patch.
// ---------------------------------------------------------------------------

/// Shared metallic two-op recipe: high ratio-offset operators, heavy
/// feedback, a noisy waveform blend, and sculpted decay/release.
fn fm_metal(
    ratio1: f32,
    ratio2: f32,
    wf1: f32,
    wf2: f32,
    ff1: f32,
    fb2: f32,
    dec1_ms: f32,
    dec2_ms: f32,
    rel_ms: f32,
    sus: f32,
    atk_ms: f32,
    master: f32,
) -> (DeviceId, Vec<u8>) {
    let mut f = Falcon::default();
    f.osc1_ratio_coarse = Falcon::ratio_coarse(ratio1 as i32);
    f.osc1_waveform = wf1;
    f.osc1_attack = env_ms(atk_ms);
    f.osc1_decay = env_ms(dec1_ms);
    f.osc1_sustain = sus;
    f.osc1_feed_forward = ff1;
    f.osc2_ratio_coarse = Falcon::ratio_coarse(ratio2 as i32);
    f.osc2_waveform = wf2;
    f.osc2_attack = env_ms(atk_ms);
    f.osc2_feedback = fb2;
    f.osc2_decay = env_ms(dec2_ms);
    f.osc2_sustain = 0.0;
    f.osc2_release = env_ms(rel_ms);
    f.master_level = master;
    (DeviceId::Falcon, f.chunk())
}

pub fn closed_hat_dark() -> (DeviceId, Vec<u8>) {
    // Dulled metallic: lower ratios, heavier feedback for a thuddier tick.
    fm_metal(14.0, 9.0, 0.45, 0.5, 0.88, 0.90, 35.0, 60.0, 45.0, 0.0, 2.0, 0.48)
}

pub fn closed_hat_openish() -> (DeviceId, Vec<u8>) {
    // Slightly longer: a hat a touch more open than choked.
    fm_metal(16.0, 10.0, 0.5, 0.55, 0.92, 0.82, 60.0, 110.0, 70.0, 0.0, 2.0, 0.50)
}

pub fn ride_ping() -> (DeviceId, Vec<u8>) {
    // Clear bell ping: stable high partial, dryish body.
    fm_metal(9.0, 5.0, 0.5, 0.45, 0.80, 0.55, 60.0, 700.0, 500.0, 0.0, 3.0, 0.52)
}

pub fn ride_stick() -> (DeviceId, Vec<u8>) {
    // Dry stick hit: more noise, less sustained bell.
    fm_metal(9.0, 5.0, 0.7, 0.6, 0.92, 0.70, 50.0, 450.0, 350.0, 0.0, 3.0, 0.54)
}

pub fn ride_washy() -> (DeviceId, Vec<u8>) {
    // Bell + wide wash: brighter bell over a long shimmer.
    fm_metal(10.0, 5.0, 0.6, 0.65, 0.90, 0.75, 80.0, 900.0, 650.0, 0.0, 3.0, 0.52)
}

pub fn ride_darkbell() -> (DeviceId, Vec<u8>) {
    // Muted bell: lower, denser, darker.
    fm_metal(8.0, 4.0, 0.5, 0.5, 0.85, 0.80, 70.0, 800.0, 550.0, 0.0, 3.0, 0.48)
}

pub fn ride_sizzle() -> (DeviceId, Vec<u8>) {
    // Rivets shimmer: noisy, light feedback, long sparkle.
    fm_metal(9.0, 5.0, 0.85, 0.8, 0.95, 0.60, 90.0, 1000.0, 700.0, 0.03, 3.0, 0.54)
}
