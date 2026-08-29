//! Encoding/decoding of the WaveSabre serialized song format.
//!
//! The byte layout mirrors `WaveSabreConvert/Serializer.cs` on the write side
//! and the reader in `WaveSabrePlayerLib/SongRenderer.cpp`:
//!
//! ```text
//! int  tempo
//! int  sample rate
//! double song length (seconds)
//! int  device count
//!   byte  device id
//!   int   chunk size
//!   byte[] chunk
//! int  midi lane count
//!   int  event count
//!     int  samples from last event (or 0 if first)
//!     byte note (msb set = note off)
//!     byte velocity (note on only)
//! int  track count
//!   float track volume
//!   int  receive count
//!     int sending track index
//!     int receiving channel index
//!     float volume
//!   int  device count
//!     int device index
//!   int  midi lane id
//!   int  automation count
//!     int device index
//!     int param id
//!     int point count
//!       int samples from last point
//!       byte quantized value (0-255)
//! ```
//!
//! Device chunk format (`Device::GetChunk`): `numParams` little-endian `f32`
//! followed by a little-endian `i32` holding the chunk size.

use std::vec::Vec;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum DeviceId {
    Falcon,
    Slaughter,
    Thunder,
    Scissor,
    Leveller,
    Crusher,
    Echo,
    Smasher,
    Chamber,
    Twister,
    Cathedral,
    Adultery,
    Specimen,
}

pub const DEVICE_NAMES: [&str; 13] = [
    "Falcon",
    "Slaughter",
    "Thunder",
    "Scissor",
    "Leveller",
    "Crusher",
    "Echo",
    "Smasher",
    "Chamber",
    "Twister",
    "Cathedral",
    "Adultery",
    "Specimen",
];

impl DeviceId {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(b: u8) -> Option<DeviceId> {
        Some(match b {
            0 => DeviceId::Falcon,
            1 => DeviceId::Slaughter,
            2 => DeviceId::Thunder,
            3 => DeviceId::Scissor,
            4 => DeviceId::Leveller,
            5 => DeviceId::Crusher,
            6 => DeviceId::Echo,
            7 => DeviceId::Smasher,
            8 => DeviceId::Chamber,
            9 => DeviceId::Twister,
            10 => DeviceId::Cathedral,
            11 => DeviceId::Adultery,
            12 => DeviceId::Specimen,
            _ => return None,
        })
    }
}

/// Builds a device chunk from raw float parameters.
pub fn build_chunk(params: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(params.len() * 4 + 4);
    for p in params {
        out.extend_from_slice(&p.to_le_bytes());
    }
    out.extend_from_slice(&((params.len() * 4 + 4) as i32).to_le_bytes());
    out
}

/// Extracts float parameters from a device chunk (empty if malformed).
pub fn chunk_params(chunk: &[u8]) -> Vec<f32> {
    if chunk.len() < 4 {
        return Vec::new();
    }
    let size = i32::from_le_bytes(chunk[chunk.len() - 4..].try_into().unwrap()) as usize;
    if size != chunk.len() || (size - 4) % 4 != 0 {
        return Vec::new();
    }
    let n = (size - 4) / 4;
    let mut params = Vec::with_capacity(n);
    for i in 0..n {
        params.push(f32::from_le_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap()));
    }
    params
}

/// Encodes an envelope time in milliseconds to the normalized scalar WaveSabre
/// devices expect (`Helpers::ScalarToEnvValue`: `ms = scalar^2 * 5000 + 1`).
/// Shared by every device with ADSR time params (Adultery, Falcon, ...).
pub fn env_ms(ms: f32) -> f32 {
    ((ms - 1.0) / 5000.0).max(0.0).sqrt()
}

/// Adultery (GM sample player) parameters in index order, see `Adultery.h`.
///
/// Enum-valued params are normalized 0..1 across their enum range
/// (`(int)(value * (numValues - 1))`), which matters when numValues > 2:
#[derive(Clone, Copy, Debug)]
pub struct Adultery {
    pub sample_index: f32, // 1-based (1..495); 0 = no sample
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,
    pub sample_start: f32,
    /// Normalized over `LoopMode { Disabled=0, Repeat=1, PingPong=2 }`:
    /// 0.0 = disabled, **0.5 = repeat**, 1.0 = ping-pong.
    pub loop_mode: f32,
    /// Normalized over `LoopBoundaryMode { FromSample=0, Manual=1 }`.
    pub loop_boundary_mode: f32,
    pub loop_start: f32,
    pub loop_length: f32,
    pub reverse: f32,
    pub interpolation_mode: f32,
    pub coarse_tune: f32,
    pub fine_tune: f32,
    pub filter_type: f32,
    pub filter_freq: f32,
    pub filter_resonance: f32,
    pub filter_mod_amt: f32,
    pub mod_attack: f32,
    pub mod_decay: f32,
    pub mod_sustain: f32,
    pub mod_release: f32,
    pub voices_unisono: f32,
    pub voices_detune: f32,
    pub voices_pan: f32,
    pub master: f32,
    pub voice_mode: f32,
    pub slide_time: f32,
}

impl Default for Adultery {
    fn default() -> Self {
        Adultery {
            sample_index: 1.0,
            amp_attack: Adultery::env_ms(2.0),
            amp_decay: Adultery::env_ms(500.0),
            amp_sustain: 1.0,
            amp_release: Adultery::env_ms(3000.0),
            sample_start: 0.0,
            loop_mode: 0.5,          // Repeat (see field docs: 0.5 of 3 modes)
            loop_boundary_mode: 0.0, // FromSample (use DLS loop points)
            loop_start: 0.0,
            loop_length: 1.0,
            reverse: 0.0,
            interpolation_mode: 1.0, // Linear
            coarse_tune: 0.5,
            fine_tune: 0.5,
            filter_type: 0.0,    // Lowpass
            filter_freq: 1.0,    // open
            filter_resonance: 1.0,
            filter_mod_amt: 0.5,
            mod_attack: Adultery::env_ms(2.0),
            mod_decay: Adultery::env_ms(50.0),
            mod_sustain: 1.0,
            mod_release: Adultery::env_ms(500.0),
            voices_unisono: 0.0,
            voices_detune: 0.0,
            voices_pan: 0.5,
            master: 0.5,
            voice_mode: 0.0, // Polyphonic
            slide_time: 0.0,
        }
    }
}

impl Adultery {
    /// Encodes envelope time in ms; see the module-level [`env_ms`].
    pub fn env_ms(ms: f32) -> f32 {
        env_ms(ms)
    }

    pub fn to_params(&self) -> [f32; 28] {
        [
            self.sample_index,
            self.amp_attack,
            self.amp_decay,
            self.amp_sustain,
            self.amp_release,
            self.sample_start,
            self.loop_mode,
            self.loop_boundary_mode,
            self.loop_start,
            self.loop_length,
            self.reverse,
            self.interpolation_mode,
            self.coarse_tune,
            self.fine_tune,
            self.filter_type,
            self.filter_freq,
            self.filter_resonance,
            self.filter_mod_amt,
            self.mod_attack,
            self.mod_decay,
            self.mod_sustain,
            self.mod_release,
            self.voices_unisono,
            self.voices_detune,
            self.voices_pan,
            self.master,
            self.voice_mode,
            self.slide_time,
        ]
    }

    pub fn chunk(&self) -> Vec<u8> {
        build_chunk(&self.to_params())
    }
}

/// Falcon (2-operator FM synth) parameters in index order, see `Falcon.h`.
///
/// Signal path: osc1 (modulator) feeds osc2 (carrier) via `feed_forward`; only
/// osc2 is audible. Value semantics from `Falcon.cpp`:
///
/// - `*_waveform`: raw 0..1 blend, 0 = pure sine, 1 = sine + square partials.
/// - `*_ratio_coarse`: frequency ratio = `1 + floor(coarse * 32.99) +
///   ((fine - 0.5) * 2)^3`, so coarse quantizes to integer ratio offsets
///   0..32 above unity and fine sweeps ±1 with a cubic curve.
/// - `osc1_feedback` / `osc2_feedback`: self-FM index = `v^2 / 2` (osc2's is
///   additionally scaled by 13.25 internally), so small values already bite.
/// - `osc1_feed_forward`: FM index from osc1 into osc2 = `v^2` (times osc1's
///   hot 13.25x output), so moderate values give classic FM brightness.
/// - Envelope times use [`env_ms`]; sustains are raw levels.
/// - `master_level`: perceived gain `(v * 0.4)^2`, i.e. quadratic falloff.
/// - `pitch_env_amt1/2`: semitones = `(value - 0.5) * 72` (±36 range).
#[derive(Clone, Copy, Debug)]
pub struct Falcon {
    pub osc1_waveform: f32,
    pub osc1_ratio_coarse: f32,
    pub osc1_ratio_fine: f32,
    pub osc1_feedback: f32,
    pub osc1_feed_forward: f32,
    pub osc1_attack: f32,
    pub osc1_decay: f32,
    pub osc1_sustain: f32,
    pub osc1_release: f32,
    pub osc2_waveform: f32,
    pub osc2_ratio_coarse: f32,
    pub osc2_ratio_fine: f32,
    pub osc2_feedback: f32,
    pub osc2_attack: f32,
    pub osc2_decay: f32,
    pub osc2_sustain: f32,
    pub osc2_release: f32,
    pub master_level: f32,
    pub voices_unisono: f32,
    pub voices_detune: f32,
    pub voices_pan: f32,
    pub vibrato_freq: f32,
    pub vibrato_amount: f32,
    pub rise: f32,
    pub pitch_attack: f32,
    pub pitch_decay: f32,
    pub pitch_sustain: f32,
    pub pitch_release: f32,
    pub pitch_env_amt1: f32,
    pub pitch_env_amt2: f32,
    pub voice_mode: f32,
    pub slide_time: f32,
}

impl Default for Falcon {
    fn default() -> Self {
        Falcon {
            osc1_waveform: 0.0,
            osc1_ratio_coarse: 0.0,
            osc1_ratio_fine: 0.5,
            osc1_feedback: 0.0,
            osc1_feed_forward: 0.0,
            osc1_attack: env_ms(1.0),
            osc1_decay: env_ms(1.0),
            osc1_sustain: 1.0,
            osc1_release: env_ms(1.0),
            osc2_waveform: 0.0,
            osc2_ratio_coarse: 0.0,
            osc2_ratio_fine: 0.5,
            osc2_feedback: 0.0,
            osc2_attack: env_ms(1.0),
            osc2_decay: env_ms(500.0),
            osc2_sustain: 0.75,
            osc2_release: env_ms(500.0),
            master_level: 0.8,
            voices_unisono: 0.0,
            voices_detune: 0.0,
            voices_pan: 0.5,
            vibrato_freq: 0.0,
            vibrato_amount: 0.0,
            rise: 0.0,
            pitch_attack: env_ms(1.0),
            pitch_decay: env_ms(500.0),
            pitch_sustain: 0.5,
            pitch_release: env_ms(1500.0),
            pitch_env_amt1: 0.5,
            pitch_env_amt2: 0.5,
            voice_mode: 0.0, // Polyphonic
            slide_time: 0.0,
        }
    }
}

impl Falcon {
    /// Coarse param for an integer carrier/modulator frequency ratio >= 1.
    /// `floor(coarse * 32.99)` must decode to `ratio - 1`; aiming at the
    /// midpoint of its quantization bin keeps that robust under f32 rounding
    /// (a bare `(ratio - 1) / 32.99` can land an ulp below and floor to the
    /// next-lower ratio). Keep `ratio_fine` at 0.5 so its cubic term vanishes.
    pub fn ratio_coarse(ratio: i32) -> f32 {
        ((ratio.max(1) - 1) as f32 + 0.5) / 32.99
    }

    /// Pitch envelope amount param for `semitones` within ±36.
    pub fn pitch_amt(semitones: f32) -> f32 {
        semitones / 72.0 + 0.5
    }

    pub fn to_params(&self) -> [f32; 32] {
        [
            self.osc1_waveform,
            self.osc1_ratio_coarse,
            self.osc1_ratio_fine,
            self.osc1_feedback,
            self.osc1_feed_forward,
            self.osc1_attack,
            self.osc1_decay,
            self.osc1_sustain,
            self.osc1_release,
            self.osc2_waveform,
            self.osc2_ratio_coarse,
            self.osc2_ratio_fine,
            self.osc2_feedback,
            self.osc2_attack,
            self.osc2_decay,
            self.osc2_sustain,
            self.osc2_release,
            self.master_level,
            self.voices_unisono,
            self.voices_detune,
            self.voices_pan,
            self.vibrato_freq,
            self.vibrato_amount,
            self.rise,
            self.pitch_attack,
            self.pitch_decay,
            self.pitch_sustain,
            self.pitch_release,
            self.pitch_env_amt1,
            self.pitch_env_amt2,
            self.voice_mode,
            self.slide_time,
        ]
    }

    pub fn chunk(&self) -> Vec<u8> {
        build_chunk(&self.to_params())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MidiEvent {
    pub samples: i64,
    pub note: u8,
    pub velocity: u8,
    pub on: bool,
}

impl MidiEvent {
    pub fn on(samples: i64, note: u8, velocity: u8) -> MidiEvent {
        MidiEvent {
            samples,
            note,
            velocity,
            on: true,
        }
    }

    pub fn off(samples: i64, note: u8) -> MidiEvent {
        MidiEvent {
            samples,
            note,
            velocity: 0,
            on: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Receive {
    pub sending_track: i32,
    pub channel: i32,
    pub volume: f32,
}

impl Receive {
    pub fn new(sending_track: i32, channel: i32, volume: f32) -> Receive {
        Receive {
            sending_track,
            channel,
            volume,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Automation {
    pub device_index: usize,
    pub param_id: i32,
    pub points: Vec<(i64, u8)>,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub volume: f32,
    pub receives: Vec<Receive>,
    pub devices: Vec<(DeviceId, Vec<u8>)>,
    pub events: Vec<MidiEvent>,
    pub automations: Vec<Automation>,
}

impl Track {
    pub fn new(volume: f32) -> Track {
        Track {
            volume,
            receives: Vec::new(),
            devices: Vec::new(),
            events: Vec::new(),
            automations: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Song {
    pub tempo: i32,
    pub sample_rate: i32,
    pub length: f64,
    pub tracks: Vec<Track>,
}

impl Song {
    pub fn new(tempo: i32, sample_rate: i32) -> Song {
        Song {
            tempo,
            sample_rate,
            length: 0.0,
            tracks: Vec::new(),
        }
    }
}

/// Mirrors `Song.Restructure` / `DeltaEncode` / `Serializer.CreateBinary`.
pub fn encode(song: &Song) -> Vec<u8> {
    let mut out = Vec::new();

    // Song settings
    out.extend_from_slice(&song.tempo.to_le_bytes());
    out.extend_from_slice(&song.sample_rate.to_le_bytes());
    out.extend_from_slice(&song.length.to_le_bytes());

    // Devices: collect all (track order), then sort by id. Rust's sort_by_key
    // is stable, so equal ids keep their relative order (matches the C#
    // List.Sort in Restructure closely enough that the exact order does not
    // matter - any id-sorted order with coherent indices is valid).
    let mut all_devices: Vec<(DeviceId, Vec<u8>)> = Vec::new();
    for t in &song.tracks {
        for d in &t.devices {
            all_devices.push(d.clone());
        }
    }
    all_devices.sort_by_key(|d| d.0);

    // Map every track device to a unique index into the sorted device list.
    let mut used = vec![false; all_devices.len()];
    let mut track_indices: Vec<Vec<usize>> = Vec::with_capacity(song.tracks.len());
    for t in &song.tracks {
        let mut idxs = Vec::with_capacity(t.devices.len());
        for d in &t.devices {
            for (i, ad) in all_devices.iter().enumerate() {
                if !used[i] && ad.0 == d.0 && ad.1 == d.1 {
                    used[i] = true;
                    idxs.push(i);
                    break;
                }
            }
        }
        track_indices.push(idxs);
    }

    // Device list
    out.extend_from_slice(&(all_devices.len() as i32).to_le_bytes());
    for d in &all_devices {
        out.push(d.0.as_u8());
        out.extend_from_slice(&(d.1.len() as i32).to_le_bytes());
        out.extend_from_slice(&d.1);
    }

    // Midi lanes: one per track, delta coded, chronologically sorted.
    out.extend_from_slice(&(song.tracks.len() as i32).to_le_bytes());
    for t in &song.tracks {
        let mut sorted = t.events.clone();
        sorted.sort_by_key(|e| e.samples);
        out.extend_from_slice(&(sorted.len() as i32).to_le_bytes());
        let mut last = 0i64;
        for e in sorted {
            out.extend_from_slice(&((e.samples - last) as i32).to_le_bytes());
            if e.on {
                out.push(e.note & 0x7f);
                out.push(e.velocity);
            } else {
                out.push((e.note & 0x7f) | 0x80);
            }
            last = e.samples;
        }
    }

    // Tracks
    out.extend_from_slice(&(song.tracks.len() as i32).to_le_bytes());
    for (ti, t) in song.tracks.iter().enumerate() {
        out.extend_from_slice(&t.volume.to_le_bytes());

        out.extend_from_slice(&(t.receives.len() as i32).to_le_bytes());
        for r in &t.receives {
            out.extend_from_slice(&r.sending_track.to_le_bytes());
            out.extend_from_slice(&r.channel.to_le_bytes());
            out.extend_from_slice(&r.volume.to_le_bytes());
        }

        let idxs = &track_indices[ti];
        out.extend_from_slice(&(idxs.len() as i32).to_le_bytes());
        for i in idxs {
            out.extend_from_slice(&(*i as i32).to_le_bytes());
        }

        out.extend_from_slice(&(ti as i32).to_le_bytes()); // midi lane id

        out.extend_from_slice(&(t.automations.len() as i32).to_le_bytes());
        for a in &t.automations {
            out.extend_from_slice(&(a.device_index as i32).to_le_bytes());
            out.extend_from_slice(&a.param_id.to_le_bytes());
            out.extend_from_slice(&(a.points.len() as i32).to_le_bytes());
            let mut last = 0i64;
            for (samples, value) in &a.points {
                out.extend_from_slice(&((*samples - last) as i32).to_le_bytes());
                out.push(*value);
                last = *samples;
            }
        }
    }

    out
}

#[derive(Clone, Debug)]
pub struct ParsedDevice {
    pub id: DeviceId,
    pub chunk: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ParsedEvent {
    pub samples: i64, // absolute sample position
    pub note: u8,
    pub velocity: u8,
    pub on: bool,
}

#[derive(Clone, Debug)]
pub struct ParsedTrack {
    pub volume: f32,
    pub receives: Vec<Receive>,
    pub device_indices: Vec<usize>,
    pub lane_id: usize,
    pub automations: Vec<Automation>,
}

#[derive(Clone, Debug)]
pub struct ParsedSong {
    pub tempo: i32,
    pub sample_rate: i32,
    pub length: f64,
    pub devices: Vec<ParsedDevice>,
    pub lanes: Vec<Vec<ParsedEvent>>,
    pub tracks: Vec<ParsedTrack>,
}

/// Decodes a serialized song blob (used by `validate_song`).
pub fn decode(data: &[u8]) -> Result<ParsedSong, String> {
    let mut pos = 0usize;
    macro_rules! need {
        ($n:expr) => {
            if data.len() < pos + $n {
                return Err(format!("unexpected end of data at byte {}", pos));
            }
        };
    }
    macro_rules! take_i32 {
        () => {{
            need!(4);
            let v = i32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;
            v
        }};
    }
    macro_rules! take_f32 {
        () => {{
            need!(4);
            let v = f32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
            pos += 4;
            v
        }};
    }
    macro_rules! take_f64 {
        () => {{
            need!(8);
            let v = f64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;
            v
        }};
    }
    macro_rules! take_u8 {
        () => {{
            need!(1);
            let v = data[pos];
            pos += 1;
            v
        }};
    }

    let tempo = take_i32!();
    let sample_rate = take_i32!();
    let length = take_f64!();

    let num_devices = take_i32!();
    if num_devices < 0 {
        return Err(format!("invalid device count {}", num_devices));
    }
    let mut devices = Vec::with_capacity(num_devices as usize);
    for _ in 0..num_devices {
        let id = DeviceId::from_u8(take_u8!())
            .ok_or_else(|| format!("unknown device id at byte {}", pos - 1))?;
        let size = take_i32!();
        if size < 0 {
            return Err(format!("negative chunk size {}", size));
        }
        need!(size as usize);
        let chunk = data[pos..pos + size as usize].to_vec();
        pos += size as usize;
        devices.push(ParsedDevice { id, chunk });
    }

    let num_lanes = take_i32!();
    if num_lanes < 0 {
        return Err(format!("invalid lane count {}", num_lanes));
    }
    let mut lanes = Vec::with_capacity(num_lanes as usize);
    for lane_idx in 0..num_lanes {
        let num_events = take_i32!();
        if num_events < 0 {
            return Err(format!("invalid event count {}", num_events));
        }
        let mut events = Vec::with_capacity(num_events as usize);
        let mut last = 0i64;
        for ev_idx in 0..num_events {
            let delta = take_i32!();
            if delta < 0 {
                return Err(format!(
                    "negative event delta {} (lane {lane_idx}, event {ev_idx}, byte {})",
                    delta,
                    pos - 4
                ));
            }
            last += delta as i64;
            let note = take_u8!();
            if note & 0x80 == 0 {
                let velocity = take_u8!();
                events.push(ParsedEvent {
                    samples: last,
                    note: note & 0x7f,
                    velocity,
                    on: true,
                });
            } else {
                events.push(ParsedEvent {
                    samples: last,
                    note: note & 0x7f,
                    velocity: 0,
                    on: false,
                });
            }
        }
        lanes.push(events);
    }

    let num_tracks = take_i32!();
    if num_tracks < 0 {
        return Err(format!("invalid track count {}", num_tracks));
    }
    let mut tracks = Vec::with_capacity(num_tracks as usize);
    for _ in 0..num_tracks {
        let volume = take_f32!();

        let num_receives = take_i32!();
        if num_receives < 0 {
            return Err(format!("invalid receive count {}", num_receives));
        }
        let mut receives = Vec::with_capacity(num_receives as usize);
        for _ in 0..num_receives {
            receives.push(Receive {
                sending_track: take_i32!(),
                channel: take_i32!(),
                volume: take_f32!(),
            });
        }

        let num_devices = take_i32!();
        if num_devices < 0 {
            return Err(format!("invalid track device count {}", num_devices));
        }
        let mut device_indices = Vec::with_capacity(num_devices as usize);
        for _ in 0..num_devices {
            let idx = take_i32!();
            if idx < 0 || idx as usize >= devices.len() {
                return Err(format!("track device index {} out of range", idx));
            }
            device_indices.push(idx as usize);
        }

        let lane_id = take_i32!();
        if lane_id < 0 || lane_id as usize >= lanes.len() {
            return Err(format!("lane id {} out of range", lane_id));
        }

        let num_autos = take_i32!();
        if num_autos < 0 {
            return Err(format!("invalid automation count {}", num_autos));
        }
        let mut automations = Vec::with_capacity(num_autos as usize);
        for _ in 0..num_autos {
            let device_index = take_i32!();
            let param_id = take_i32!();
            let num_points = take_i32!();
            if num_points <= 0 {
                return Err(format!("invalid automation point count {}", num_points));
            }
            let mut points = Vec::with_capacity(num_points as usize);
            let mut last = 0i64;
            for _ in 0..num_points {
                let delta = take_i32!();
                if delta < 0 {
                    return Err(format!("negative automation delta {}", delta));
                }
                last += delta as i64;
                let value = take_u8!();
                points.push((last, value));
            }
            automations.push(Automation {
                device_index: device_index as usize,
                param_id,
                points,
            });
        }

        tracks.push(ParsedTrack {
            volume,
            receives,
            device_indices,
            lane_id: lane_id as usize,
            automations,
        });
    }

    Ok(ParsedSong {
        tempo,
        sample_rate,
        length,
        devices,
        lanes,
        tracks,
    })
}