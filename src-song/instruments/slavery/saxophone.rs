//! Slaughter saxophones — divergent from `falcon::saxophone`.
//!
//! Improvement: Falcon's reeds are FM index + vibrato; here the reed itself
//! is the filter. A resonant low-pass on detuned pulse overtones gives the
//! woody/reedy formant, a pitch scoop bends into each note like a breath
//! attack, and white noise adds the air. Voices stay polyphonic so brass
//! section lines with simultaneous notes keep their voicings.

use crate::format::{DeviceId, Slaughter, env_ms};

pub struct Reed {
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub spread: f32,
    pub vibrato_hz: f32,
    pub vibrato_amt: f32,
    pub scoop: f32,
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub release_ms: f32,
    pub master: f32,
    pub noise: f32,
}

fn reed(cfg: &Reed) -> (DeviceId, Vec<u8>) {
    let mut s = Slaughter::default();
    // Saw-ward pulse timbre (rich odd/even overtones to hit), lightly stacked.
    s.osc1_waveform = 0.60;
    s.osc1_pulse_width = Slaughter::pulse_width(0.42);
    s.osc1_volume = 0.9;
    s.osc2_waveform = 0.55;
    s.osc2_pulse_width = Slaughter::pulse_width(0.42);
    s.osc2_volume = 0.6;
    s.osc2_detune_fine = Slaughter::detune_fine(cfg.spread);
    s.osc3_waveform = 0.50;
    s.osc3_pulse_width = Slaughter::pulse_width(0.48);
    s.osc3_volume = 0.4;
    s.osc3_detune_fine = Slaughter::detune_fine(-cfg.spread * 0.9);
    // Breath.
    s.noise_volume = cfg.noise;
    // The reed formant: resonant low-pass that opens past the cutoff during
    // the swell, then settles warm.
    s.filter_type = 0.0; // lowpass
    s.filter_freq = Slaughter::filter_freq_hz(cfg.cutoff_hz);
    s.filter_resonance = Slaughter::resonance(cfg.resonance);
    s.filter_mod_amt = 0.62;
    s.mod_attack = env_ms(20.0);
    s.mod_decay = env_ms(300.0);
    s.mod_sustain = 0.55;
    // Attack "breath scoop": a quick bend up into the note, then settle.
    s.pitch_attack = env_ms(2.0);
    s.pitch_decay = env_ms(180.0);
    s.pitch_sustain = 0.0;
    s.pitch_release = env_ms(60.0);
    s.pitch_env_amt = Slaughter::pitch_amt(cfg.scoop);
    s.amp_attack = env_ms(cfg.attack_ms);
    s.amp_decay = env_ms(cfg.decay_ms);
    s.amp_sustain = 0.60;
    s.amp_release = env_ms(cfg.release_ms);
    s.vibrato_freq = cfg.vibrato_hz;
    s.vibrato_amount = cfg.vibrato_amt;
    s.master_level = cfg.master;
    s.voices_pan = 0.5;
    (DeviceId::Slaughter, s.chunk())
}

pub fn tenor_sax() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1100.0,
        resonance: 0.35,
        spread: 7.0,
        vibrato_hz: 5.6,
        vibrato_amt: 0.28,
        scoop: 2.0,
        attack_ms: 15.0,
        decay_ms: 650.0,
        release_ms: 200.0,
        master: 0.50,
        noise: 0.06,
    })
}

pub fn alto_sax() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1800.0,
        resonance: 0.30,
        spread: 7.0,
        vibrato_hz: 5.5,
        vibrato_amt: 0.22,
        scoop: 1.5,
        attack_ms: 5.0,
        decay_ms: 650.0,
        release_ms: 180.0,
        master: 0.62,
        noise: 0.06,
    })
}

pub fn sax_v1() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1000.0, resonance: 0.40, spread: 7.0, vibrato_hz: 5.6, vibrato_amt: 0.26,
        scoop: 2.0, attack_ms: 20.0, decay_ms: 750.0, release_ms: 260.0, master: 0.50, noise: 0.05,
    })
}

pub fn sax_v2() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 900.0, resonance: 0.55, spread: 8.0, vibrato_hz: 5.8, vibrato_amt: 0.34,
        scoop: 2.5, attack_ms: 4.0, decay_ms: 550.0, release_ms: 160.0, master: 0.52, noise: 0.08,
    })
}

pub fn sax_v3() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1800.0, resonance: 0.30, spread: 6.0, vibrato_hz: 5.5, vibrato_amt: 0.20,
        scoop: 1.5, attack_ms: 4.0, decay_ms: 620.0, release_ms: 180.0, master: 0.64, noise: 0.05,
    })
}

pub fn sax_v4() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1200.0, resonance: 0.28, spread: 6.0, vibrato_hz: 5.2, vibrato_amt: 0.18,
        scoop: 1.0, attack_ms: 28.0, decay_ms: 850.0, release_ms: 320.0, master: 0.52, noise: 0.04,
    })
}

pub fn sax_v5() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1300.0, resonance: 0.32, spread: 7.0, vibrato_hz: 5.4, vibrato_amt: 0.22,
        scoop: 1.5, attack_ms: 18.0, decay_ms: 700.0, release_ms: 240.0, master: 0.54, noise: 0.16,
    })
}

pub fn sax_v6() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 2500.0, resonance: 0.35, spread: 5.0, vibrato_hz: 5.7, vibrato_amt: 0.16,
        scoop: 1.0, attack_ms: 3.0, decay_ms: 500.0, release_ms: 140.0, master: 0.66, noise: 0.04,
    })
}

pub fn sax_v7() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 950.0, resonance: 0.45, spread: 5.0, vibrato_hz: 5.1, vibrato_amt: 0.38,
        scoop: 3.0, attack_ms: 40.0, decay_ms: 900.0, release_ms: 350.0, master: 0.48, noise: 0.06,
    })
}

pub fn sax_v8() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1100.0, resonance: 0.50, spread: 9.0, vibrato_hz: 5.7, vibrato_amt: 0.30,
        scoop: 1.0, attack_ms: 6.0, decay_ms: 600.0, release_ms: 200.0, master: 0.55, noise: 0.12,
    })
}

pub fn sax_v9() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1400.0, resonance: 0.25, spread: 5.0, vibrato_hz: 5.4, vibrato_amt: 0.20,
        scoop: 0.5, attack_ms: 12.0, decay_ms: 700.0, release_ms: 200.0, master: 0.60, noise: 0.05,
    })
}

pub fn sax_v10() -> (DeviceId, Vec<u8>) {
    reed(&Reed {
        cutoff_hz: 1000.0, resonance: 0.60, spread: 10.0, vibrato_hz: 6.0, vibrato_amt: 0.36,
        scoop: 2.0, attack_ms: 2.0, decay_ms: 500.0, release_ms: 120.0, master: 0.56, noise: 0.14,
    })
}