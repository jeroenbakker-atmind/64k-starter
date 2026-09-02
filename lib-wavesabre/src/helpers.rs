//! Shared math and parameter conversions from `WaveSabreCore/Helpers.cpp`.
//!
//! Everything here mirrors the C++ core. The `FastSin`/`Exp2`/etc. functions
//! are implemented with clean `f64` math instead of the MSVC x86/x87 tricks
//! (bit-exactness is not pursued; the existing Falcon renderer already did the
//! same). `CurrentSampleRate` / `CurrentTempo` statics are passed explicitly.

use std::sync::Mutex;

/// Global LCG state for `rand_float`, seeded like the core (`RandomSeed = 1`).
static RANDOM_SEED: Mutex<u64> = Mutex::new(1);

/// `Helpers::RandFloat`: 0..1 floats from a 32-bit LCG.
pub fn rand_float() -> f32 {
    let mut seed = RANDOM_SEED.lock().unwrap();
    *seed = seed.wrapping_mul(0x15a4e35) & 0xffff_ffff;
    ((*seed % 255) as f32) / 255.0
}

/// `Helpers::Exp2`.
pub fn exp2(x: f64) -> f64 {
    2f64.powf(x)
}

/// `Helpers::Exp2F` / `Pow2F`.
pub fn pow2f(x: f32) -> f32 {
    2f32.powf(x)
}

/// `Helpers::Exp10F` (used by Leveller's Biquad peak).
pub fn exp10f(x: f32) -> f32 {
    10f32.powf(x)
}

/// `Helpers::NoteToFreq`.
pub fn note_to_freq(note: f64) -> f64 {
    440.0 * exp2((note - 69.0) / 12.0)
}

/// `Helpers::FastSin` (returns a value whose magnitude is slightly damped by
/// the core's lookup table; clean `sin` is used here).
pub fn fast_sin(x: f64) -> f64 {
    x.sin()
}

/// `Helpers::FastCos`.
pub fn fast_cos(x: f64) -> f64 {
    x.cos()
}

/// `Helpers::Mix(a, b, m)` = `a + (b - a) * m`.
pub fn mix(a: f32, b: f32, m: f32) -> f32 {
    a + (b - a) * m
}

/// `Helpers::Clamp`.
pub fn clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// `Helpers::Pow4`.
pub fn pow4(v: f32) -> f32 {
    let v = v * v;
    v * v
}

/// `Helpers::DbToScalar`: `sqrt(pow2f(db / 6))`.
pub fn db_to_scalar(db: f32) -> f32 {
    pow2f(db / 6.0).sqrt()
}

/// `Helpers::VolumeToScalar`: `(v * 0.4)^2`.
pub fn volume_to_scalar(v: f32) -> f32 {
    let v = v * 0.4;
    v * v
}

/// `Helpers::EnvValueToScalar`: stores an envelope time in ms as a scalar.
pub fn env_value_to_scalar(ms: f32) -> f32 {
    ((ms - 1.0) / 5000.0).max(0.0).sqrt()
}

/// `Helpers::ScalarToEnvValue`: recovers the ms time from the stored scalar.
pub fn scalar_to_env_value(v: f32) -> f32 {
    v * v * 5000.0 + 1.0
}

/// `Helpers::ParamToFrequency`.
pub fn param_to_frequency(p: f32) -> f32 {
    20.0 + 19980.0 * p * p
}

/// `Helpers::ParamToQ`.
pub fn param_to_q(p: f32) -> f32 {
    1.0 + p * 10.0
}

/// `Helpers::ParamToDb(value, range)`.
pub fn param_to_db(v: f32, range: f32) -> f32 {
    (v * 2.0 - 1.0) * range
}

/// `Helpers::ParamToVibratoFreq`.
pub fn param_to_vibrato_freq(p: f32) -> f64 {
    ((p as f64) * (p as f64) + 0.1) * 70.0
}

/// `Helpers::ParamToBoolean`.
pub fn param_to_boolean(v: f32) -> bool {
    v > 0.5
}

/// `Helpers::ParamToUnisono`.
pub fn param_to_unisono(p: f32) -> usize {
    (p * 15.0) as usize + 1
}

/// `Helpers::ParamToVoiceMode` (f32 slice of the two-valued enum).
pub fn param_to_voice_mode(p: f32) -> u8 {
    if p < 0.5 {
        0 // Polyphonic
    } else {
        1 // MonoLegatoTrill
    }
}

/// `Helpers::ParamToStateVariableFilterType`.
pub fn param_to_state_variable_filter_type(p: f32) -> u8 {
    (p * 3.0) as u8 % 4
}

/// `Helpers::ParamToSpread`.
pub fn param_to_spread(p: f32) -> u8 {
    (p * 2.0) as u8 % 3
}

/// `Helpers::PanToScalarLeft`.
pub fn pan_to_scalar_left(pan: f32) -> f32 {
    (1.0 - pan).sqrt()
}

/// `Helpers::PanToScalarRight`.
pub fn pan_to_scalar_right(pan: f32) -> f32 {
    pan.sqrt()
}