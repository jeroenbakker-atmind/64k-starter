//! Slaughter percussion set — intentional divergence from `falcon::drums`.
//!
//! Where Falcon made percussion from FM/phase-noise, the subtractive arch
//! gets its punch from detuned BLIT-pulse stacks + noise shaped by fast
//! state-variable-filter sweeps. Key improvements over the Falcon set:
//!
//! - kick: the mod env opens the filter onto the beater click, then the body
//!   pitches down +24 -> 0 semitones and the filter closes for a chest boom.
//! - snare: woody band-passed crack + ringing rim partial under the rattle.
//! - hats: three inharmonic detuned pulses (plus noise) through a bright
//!   band-pass instead of Falcon's two feedback FM ops, so they shimmer.

use crate::format::{DeviceId, Slaughter, env_ms};

/// Bright metallic stack: three inharmonic detuned pulses in a band-pass.
fn metal_stack(s: &mut Slaughter) {
    s.osc1_waveform = 0.50;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.85;
    s.osc1_detune_coarse = Slaughter::detune_coarse(29.0);
    s.osc1_detune_fine = Slaughter::detune_fine(-9.0);
    s.osc2_waveform = 0.50;
    s.osc2_pulse_width = Slaughter::pulse_width(0.45);
    s.osc2_volume = 0.65;
    s.osc2_detune_coarse = Slaughter::detune_coarse(24.0);
    s.osc2_detune_fine = Slaughter::detune_fine(5.0);
    s.osc3_waveform = 0.55;
    s.osc3_pulse_width = Slaughter::pulse_width(0.3);
    s.osc3_volume = 0.55;
    s.osc3_detune_coarse = Slaughter::detune_coarse(19.0);
    s.osc3_detune_fine = Slaughter::detune_fine(11.0);
    s.noise_volume = 0.35;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(10000.0);
    s.filter_resonance = Slaughter::resonance(0.35);
}

pub fn kick() -> (DeviceId, Vec<u8>) {
    // Kick: beater click through an opening filter, then the body pitches
    // down +24 -> 0 semitones while the filter closes around it.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 1.0;
    s.osc2_waveform = 0.0;
    s.osc2_pulse_width = Slaughter::pulse_width(0.25);
    s.osc2_volume = 0.5;
    s.osc2_detune_coarse = Slaughter::detune_coarse(24.0);
    s.noise_volume = 0.12;
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(200.0);
    s.filter_resonance = Slaughter::resonance(0.30);
    s.filter_mod_amt = 0.55;
    s.mod_attack = env_ms(3.0);
    s.mod_decay = env_ms(90.0);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(280.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(70.0);
    s.pitch_attack = env_ms(1.0);
    s.pitch_decay = env_ms(42.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(30.0);
    s.pitch_env_amt = Slaughter::pitch_amt(24.0);
    s.master_level = 0.95;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn snare() -> (DeviceId, Vec<u8>) {
    // Snare: band-passed tonal crack with a ringing rim partial (up two
    // octaves) under the noise rattle; pitch slaps down for the body.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.85;
    s.osc3_waveform = 0.0;
    s.osc3_pulse_width = Slaughter::pulse_width(0.28);
    s.osc3_volume = 0.6;
    s.osc3_detune_coarse = Slaughter::detune_coarse(24.0);
    s.noise_volume = 0.6;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(3500.0);
    s.filter_resonance = Slaughter::resonance(0.45);
    s.filter_mod_amt = 0.5;
    s.mod_attack = env_ms(1.0);
    s.mod_decay = env_ms(55.0);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(170.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(80.0);
    s.pitch_attack = env_ms(1.0);
    s.pitch_decay = env_ms(30.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(30.0);
    s.pitch_env_amt = Slaughter::pitch_amt(10.0);
    s.master_level = 0.70;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn closed_hat() -> (DeviceId, Vec<u8>) {
    // Closed hat: metallic stack choked to ~30 ms for a tight swung tick.
    let mut s = Slaughter::default();
    metal_stack(&mut s);
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(30.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(25.0);
    s.master_level = 0.50;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn snare_shaker() -> (DeviceId, Vec<u8>) {
    // Shaker: soft band-passed noise pill with a sprinkle of detuned pulse —
    // sits as a hushed, rattly snare rather than a pitched crack.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.40;
    s.osc1_pulse_width = Slaughter::pulse_width(0.35);
    s.osc1_volume = 0.30;
    s.osc1_detune_coarse = Slaughter::detune_coarse(24.0);
    s.osc1_detune_fine = Slaughter::detune_fine(-7.0);
    s.noise_volume = 0.50;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(8200.0);
    s.filter_resonance = Slaughter::resonance(0.40);
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(130.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(60.0);
    s.master_level = 0.38;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

// ---------------------------------------------------------------------------
// Slaughter body family. These build off the subtractive strengths: a pitched
// pulse body whose pitch drops fast (kick/tom/snare/perc) through a filter
// sweep, plus a bursty-noise/band-pass builder for the clap and the passive
// band-pass click for the rim. Each is a named one-shot patch.
// ---------------------------------------------------------------------------

/// Pitched drum body: one or two pulse oscs through a low-pass/band-pass that
/// closes on the body, with a fast downward pitch drop. The core of kick,
/// snare body, tom and perc stabs.
struct PitchedBody {
    osc2_partial: f32,   // second osc detune (semitones) above
    noise: f32,
    filter_type: f32,
    cutoff_hz: f32,
    resonance: f32,
    mod_amt: f32,
    mod_dec_ms: f32,
    amp_dec_ms: f32,
    amp_sus: f32,
    amp_rel_ms: f32,
    pitch_drop: f32,     // semitones dropped on attack
    pitch_dec_ms: f32,
    master: f32,
}

fn pitched_body(cfg: &PitchedBody) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 1.0;
    if cfg.osc2_partial > 0.0 {
        s.osc2_waveform = 0.0;
        s.osc2_pulse_width = Slaughter::pulse_width(0.3);
        s.osc2_volume = 0.5;
        s.osc2_detune_coarse = Slaughter::detune_coarse(cfg.osc2_partial);
    }
    s.noise_volume = cfg.noise;
    s.filter_type = cfg.filter_type;
    s.filter_freq = Slaughter::filter_freq_hz(cfg.cutoff_hz);
    s.filter_resonance = Slaughter::resonance(cfg.resonance);
    s.filter_mod_amt = cfg.mod_amt;
    s.mod_attack = env_ms(3.0);
    s.mod_decay = env_ms(cfg.mod_dec_ms);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(cfg.amp_dec_ms);
    s.amp_sustain = cfg.amp_sus;
    s.amp_release = env_ms(cfg.amp_rel_ms);
    s.pitch_attack = env_ms(1.0);
    s.pitch_decay = env_ms(cfg.pitch_dec_ms);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(cfg.pitch_dec_ms * 0.6);
    s.pitch_env_amt = Slaughter::pitch_amt(cfg.pitch_drop);
    s.master_level = cfg.master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn kick_deep() -> (DeviceId, Vec<u8>) {
    // Sub-heavy long boom: big drop, rolling body.
    pitched_body(&PitchedBody {
        osc2_partial: 0.0, noise: 0.05, filter_type: 0.0, cutoff_hz: 200.0, resonance: 0.25,
        mod_amt: 0.55, mod_dec_ms: 120.0, amp_dec_ms: 380.0, amp_sus: 0.0, amp_rel_ms: 90.0,
        pitch_drop: 28.0, pitch_dec_ms: 55.0, master: 0.95,
    })
}

pub fn kick_tight() -> (DeviceId, Vec<u8>) {
    // Short electro/808 click: tight body, moderate drop.
    pitched_body(&PitchedBody {
        osc2_partial: 0.0, noise: 0.10, filter_type: 0.0, cutoff_hz: 260.0, resonance: 0.30,
        mod_amt: 0.50, mod_dec_ms: 70.0, amp_dec_ms: 220.0, amp_sus: 0.0, amp_rel_ms: 50.0,
        pitch_drop: 22.0, pitch_dec_ms: 35.0, master: 0.92,
    })
}

pub fn kick_gated() -> (DeviceId, Vec<u8>) {
    // Abrupt cut: super short amp tail, snappy.
    pitched_body(&PitchedBody {
        osc2_partial: 0.0, noise: 0.06, filter_type: 0.0, cutoff_hz: 240.0, resonance: 0.28,
        mod_amt: 0.48, mod_dec_ms: 60.0, amp_dec_ms: 140.0, amp_sus: 0.0, amp_rel_ms: 30.0,
        pitch_drop: 24.0, pitch_dec_ms: 25.0, master: 0.90,
    })
}

pub fn snare_roomy() -> (DeviceId, Vec<u8>) {
    // Bigger body + tail: lower band-pass, longer decay.
    pitched_body(&PitchedBody {
        osc2_partial: 24.0, noise: 0.65, filter_type: 2.0, cutoff_hz: 2400.0, resonance: 0.35,
        mod_amt: 0.55, mod_dec_ms: 90.0, amp_dec_ms: 260.0, amp_sus: 0.0, amp_rel_ms: 120.0,
        pitch_drop: 12.0, pitch_dec_ms: 40.0, master: 0.66,
    })
}

pub fn snare_trap() -> (DeviceId, Vec<u8>) {
    // Sharp, short, vinyl: bright crack, minimal rattle.
    pitched_body(&PitchedBody {
        osc2_partial: 24.0, noise: 0.40, filter_type: 2.0, cutoff_hz: 3800.0, resonance: 0.45,
        mod_amt: 0.40, mod_dec_ms: 30.0, amp_dec_ms: 110.0, amp_sus: 0.0, amp_rel_ms: 40.0,
        pitch_drop: 8.0, pitch_dec_ms: 22.0, master: 0.70,
    })
}

pub fn snare_soft() -> (DeviceId, Vec<u8>) {
    // Brush, gentle: soft body, high band-pass, quiet rattle.
    pitched_body(&PitchedBody {
        osc2_partial: 24.0, noise: 0.35, filter_type: 2.0, cutoff_hz: 2800.0, resonance: 0.32,
        mod_amt: 0.42, mod_dec_ms: 60.0, amp_dec_ms: 200.0, amp_sus: 0.0, amp_rel_ms: 90.0,
        pitch_drop: 8.0, pitch_dec_ms: 30.0, master: 0.58,
    })
}

pub fn tom_floor() -> (DeviceId, Vec<u8>) {
    // Low, deep: big drop, wide body.
    pitched_body(&PitchedBody {
        osc2_partial: 7.0, noise: 0.08, filter_type: 0.0, cutoff_hz: 300.0, resonance: 0.35,
        mod_amt: 0.55, mod_dec_ms: 100.0, amp_dec_ms: 260.0, amp_sus: 0.0, amp_rel_ms: 90.0,
        pitch_drop: 18.0, pitch_dec_ms: 40.0, master: 0.80,
    })
}

pub fn tom_rototom() -> (DeviceId, Vec<u8>) {
    // Tuned, pitched: strongest sustained pitch (small drop).
    pitched_body(&PitchedBody {
        osc2_partial: 7.0, noise: 0.05, filter_type: 0.0, cutoff_hz: 600.0, resonance: 0.45,
        mod_amt: 0.48, mod_dec_ms: 90.0, amp_dec_ms: 240.0, amp_sus: 0.0, amp_rel_ms: 80.0,
        pitch_drop: 6.0, pitch_dec_ms: 30.0, master: 0.76,
    })
}

pub fn tom_gated() -> (DeviceId, Vec<u8>) {
    // Tight electronic: short amp tail, snappy drop.
    pitched_body(&PitchedBody {
        osc2_partial: 7.0, noise: 0.12, filter_type: 0.0, cutoff_hz: 450.0, resonance: 0.28,
        mod_amt: 0.48, mod_dec_ms: 55.0, amp_dec_ms: 130.0, amp_sus: 0.0, amp_rel_ms: 30.0,
        pitch_drop: 16.0, pitch_dec_ms: 22.0, master: 0.80,
    })
}

// ---------------------------------------------------------------------------
// Rim: a bright band-pass click, near-bodyless. Pure stick on the rim.
// ---------------------------------------------------------------------------
fn rim_click(cutoff_hz: f32, resonance: f32, noise: f32, amp_dec_ms: f32, master: f32) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.8;
    s.noise_volume = noise;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(cutoff_hz);
    s.filter_resonance = Slaughter::resonance(resonance);
    s.filter_mod_amt = 0.40;
    s.mod_attack = env_ms(1.0);
    s.mod_decay = env_ms(40.0);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(amp_dec_ms);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(25.0);
    s.master_level = master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn rim_ting() -> (DeviceId, Vec<u8>) {
    // High partial: strong pitch body, resonant band.
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.0;
    s.osc1_pulse_width = Slaughter::pulse_width(0.5);
    s.osc1_volume = 0.8;
    s.osc3_waveform = 0.0;
    s.osc3_pulse_width = Slaughter::pulse_width(0.2);
    s.osc3_volume = 0.6;
    s.osc3_detune_coarse = Slaughter::detune_coarse(24.0);
    s.noise_volume = 0.12;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(6000.0);
    s.filter_resonance = Slaughter::resonance(0.55);
    s.filter_mod_amt = 0.40;
    s.mod_attack = env_ms(1.0);
    s.mod_decay = env_ms(50.0);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(90.0);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(40.0);
    s.master_level = 0.60;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn rim_damped() -> (DeviceId, Vec<u8>) {
    rim_click(3000.0, 0.40, 0.35, 70.0, 0.50)
}

// ---------------------------------------------------------------------------
// Clap: layered bursty noise through a band-pass that opens. No pitched body.
// ---------------------------------------------------------------------------
fn clap_burst(cutoff_hz: f32, resonance: f32, noise: f32, mod_dec_ms: f32, amp_dec_ms: f32, master: f32) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    s.osc1_waveform = 0.40;
    s.osc1_pulse_width = Slaughter::pulse_width(0.35);
    s.osc1_volume = 0.30;
    s.osc1_detune_coarse = Slaughter::detune_coarse(24.0);
    s.osc1_detune_fine = Slaughter::detune_fine(-7.0);
    s.noise_volume = noise;
    s.filter_type = 2.0; // bandpass
    s.filter_freq = Slaughter::filter_freq_hz(cutoff_hz);
    s.filter_resonance = Slaughter::resonance(resonance);
    s.filter_mod_amt = 0.45;
    s.mod_attack = env_ms(1.0);
    s.mod_decay = env_ms(mod_dec_ms);
    s.mod_sustain = 0.0;
    s.amp_attack = env_ms(1.0);
    s.amp_decay = env_ms(amp_dec_ms);
    s.amp_sustain = 0.0;
    s.amp_release = env_ms(60.0);
    s.master_level = master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn clap_tight() -> (DeviceId, Vec<u8>) {
    clap_burst(5000.0, 0.50, 0.70, 40.0, 90.0, 0.55)
}

pub fn clap_roomy() -> (DeviceId, Vec<u8>) {
    clap_burst(3000.0, 0.35, 0.80, 90.0, 200.0, 0.52)
}

pub fn clap_soft() -> (DeviceId, Vec<u8>) {
    clap_burst(4000.0, 0.42, 0.50, 60.0, 150.0, 0.48)
}