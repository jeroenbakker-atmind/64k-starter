//! A Rust port of the WaveSabre rendering engine ("sabrewave"): devices
//! (Falcon, Slaughter + effects), track chains, receives and automation, with
//! per-sample stereo processing mirroring `SongRenderer.Track.cpp`.
//!
//! Public API: `render` / `render_solo` (mono, for `examples/create_song`
//! previews) and `render_stereo` / `render_solo_stereo` (the full render used
//! by `examples/render_song`), plus `normalize*` and the WAV writers.

pub mod crude;
pub mod devices;
pub mod envelope;
pub mod falcon;
pub mod filters;
pub mod fx;
pub mod helpers;
pub mod slaughter;
pub mod voice;

use std::fs;
use std::path::Path;
use std::vec::Vec;

use common::{Automation, ParsedSong, ParsedTrack};

pub use devices::Device;

// ===========================================================================
// Rendering engine
// ===========================================================================

/// Interpolated automation playback for one `Automation` in a track.
struct AutomationState {
    device_slot: Option<usize>,
    param_id: usize,
    points: Vec<(usize, u8)>,
    next_i: usize,
    from: f32,
    to: f32,
    steps: i64,
    step: i64,
}

impl AutomationState {
    fn new(
        track: &ParsedTrack,
        automation: &Automation,
    ) -> AutomationState {
        let device_slot = track
            .device_indices
            .iter()
            .position(|d| *d == automation.device_index);
        AutomationState {
            device_slot,
            param_id: automation.param_id as usize,
            points: automation
                .points
                .iter()
                .map(|&(t, v)| (t.max(0) as usize, v))
                .collect(),
            next_i: 0,
            from: 0.0,
            to: 0.0,
            steps: 0,
            step: 0,
        }
    }

    fn step(&mut self, s: usize, devices: &mut [Device]) {
        let Some(slot) = self.device_slot else {
            return;
        };
        if self.steps > 0 {
            self.step = (self.step + 1).min(self.steps);
            let v = self.from + (self.to - self.from) * (self.step as f32 / self.steps as f32);
            devices[slot].set_param(self.param_id, v);
        }
        if self.next_i < self.points.len() && self.points[self.next_i].0 <= s {
            let (t, byte) = self.points[self.next_i];
            self.from = self.to;
            self.to = byte as f32 / 255.0;
            let end = if self.next_i + 1 < self.points.len() {
                self.points[self.next_i + 1].0
            } else {
                t
            };
            self.steps = (end as i64 - t as i64).max(1);
            self.step = 0;
            self.next_i += 1;
        }
    }
}

struct Renderer<'a> {
    song: &'a ParsedSong,
    sr: f64,
    total: usize,
    include_receives: bool,
    cache: Vec<Option<Vec<[f32; 2]>>>,
}

impl<'a> Renderer<'a> {
    fn new(song: &'a ParsedSong) -> Renderer<'a> {
        Renderer::with_receives(song, true)
    }

    fn with_receives(song: &'a ParsedSong, include_receives: bool) -> Renderer<'a> {
        let sr = song.sample_rate.max(1) as f64;
        let total = (song.length * sr) as usize;
        Renderer {
            song,
            sr,
            total,
            include_receives,
            cache: (0..song.tracks.len()).map(|_| None).collect(),
        }
    }

    fn tempo(&self) -> f64 {
        self.song.tempo.max(1) as f64
    }

    /// Renders every track once (topological order seeded by the master /
    /// referencing tracks) so receives can pull from fully-rendered senders.
    fn render_all(&mut self) {
        let mut visiting = vec![false; self.song.tracks.len()];
        let mut order = Vec::with_capacity(self.song.tracks.len());
        for ti in self.all_mix_tracks() {
            self.visit(ti, &mut visiting, &mut order);
        }
        for ti in order {
            if self.cache[ti].is_none() {
                let buf = self.render_track(ti);
                self.cache[ti] = Some(buf);
            }
        }
    }

    fn visit(&self, ti: usize, visiting: &mut [bool], order: &mut Vec<usize>) {
        if visiting[ti] || self.cache[ti].is_some() || order.contains(&ti) {
            return;
        }
        visiting[ti] = true;
        if ti < self.song.tracks.len() {
            for rc in &self.song.tracks[ti].receives {
                let snd = rc.sending_track;
                if snd >= 0 {
                    self.visit(snd as usize, visiting, order);
                }
            }
        }
        visiting[ti] = false;
        order.push(ti);
    }

    /// Index of the receiving/master track (no devices), or all tracks when
    /// none exists.
    fn all_mix_tracks(&self) -> Vec<usize> {
        if let Some(master) = self
            .song
            .tracks
            .iter()
            .position(|t| t.device_indices.is_empty())
        {
            vec![master]
        } else {
            (0..self.song.tracks.len()).collect()
        }
    }

    fn render_track(&mut self, ti: usize) -> Vec<[f32; 2]> {
        let mut buf = vec![[0.0f32, 0.0f32]; self.total];
        if ti >= self.song.tracks.len() {
            return buf;
        }
        let track = &self.song.tracks[ti];
        let sr = self.sr;
        let tempo = self.tempo();

        if track.device_indices.is_empty() {
            // Pure routing track (master): just aggregate receives.
            for s in 0..self.total {
                let mut l = 0.0f32;
                let mut r = 0.0f32;
                if self.include_receives {
                    for rc in &track.receives {
                        let snd = rc.sending_track;
                        if snd < 0 || snd as usize >= self.song.tracks.len() {
                            continue;
                        }
                        let src = self.require(snd as usize);
                        let src = &src[s];
                        match rc.channel {
                            0 => {
                                l += src[0] * rc.volume;
                                r += src[1] * rc.volume;
                            }
                            1 => r += src[0] * rc.volume,
                            _ => {}
                        }
                    }
                }
                buf[s] = [l * track.volume, r * track.volume];
            }
            return buf;
        }

        let mut devices: Vec<Device> = track
            .device_indices
            .iter()
            .filter_map(|di| {
                let dev = self.song.devices.get(*di)?;
                Some(Device::build(dev.id, &dev.chunk, ti, tempo, sr))
            })
            .collect();

        let mut automations: Vec<AutomationState> = track
            .automations
            .iter()
            .map(|a| AutomationState::new(track, a))
            .collect();

        let mut events: Vec<(usize, bool, u8, u8)> = self.song.lanes[track.lane_id]
            .iter()
            .map(|e| (e.samples.max(0) as usize, e.on, e.note, e.velocity))
            .collect();
        events.sort_by_key(|e| e.0);

        let mut idx = 0;
        for (s, slot) in buf.iter_mut().enumerate() {
            let mut l = 0.0f32;
            let mut r = 0.0f32;

            // Receives seed the buffer before the device chain runs.
            if self.include_receives {
                for rc in &track.receives {
                    let snd = rc.sending_track;
                    if snd < 0 || snd as usize >= self.song.tracks.len() {
                        continue;
                    }
                    let src = self.require(snd as usize);
                    let src = &src[s];
                    match rc.channel {
                        0 => {
                            l += src[0] * rc.volume;
                            r += src[1] * rc.volume;
                        }
                        1 => r += src[0] * rc.volume,
                        _ => {}
                    }
                }
            }

            // Automations, then MIDI events due at this sample.
            for a in automations.iter_mut() {
                a.step(s, &mut devices);
            }
            while idx < events.len() && events[idx].0 <= s {
                let (_, on, note, vel) = events[idx];
                for d in devices.iter_mut() {
                    if on {
                        d.note_on(note, vel, sr);
                    } else {
                        d.note_off(note, sr);
                    }
                }
                idx += 1;
            }

            for d in devices.iter_mut() {
                d.next_sample(sr, &mut l, &mut r);
            }

            l *= track.volume;
            r *= track.volume;
            slot[0] = l;
            slot[1] = r;
        }
        buf
    }

    /// Returns a rendered track buffer, rendering it on demand.
    fn require(&mut self, ti: usize) -> &Vec<[f32; 2]> {
        if self.cache[ti].is_none() {
            let buf = self.render_track(ti);
            self.cache[ti] = Some(buf);
        }
        self.cache[ti].as_ref().unwrap()
    }
}

fn downmix(buf: &[[f32; 2]]) -> Vec<f32> {
    let mut out = Vec::with_capacity(buf.len());
    for s in buf {
        out.push((s[0] + s[1]) * 0.5);
    }
    out
}

/// Renders the full song mix to mono floats (sabrewave engine, downmixed).
pub fn render(song: &ParsedSong) -> Vec<f32> {
    downmix(&render_stereo(song))
}

/// Renders one track (by index) alone to mono floats, ignoring its receives.
pub fn render_solo(song: &ParsedSong, ti: usize) -> Vec<f32> {
    downmix(&render_solo_stereo(song, ti))
}

/// Renders the full song mix to interleaved stereo.
pub fn render_stereo(song: &ParsedSong) -> Vec<[f32; 2]> {
    let mut r = Renderer::new(song);
    r.render_all();
    let mut mix = vec![[0.0f32, 0.0f32]; r.total];
    for ti in r.all_mix_tracks() {
        let buf = r.require(ti);
        for (i, s) in buf.iter().enumerate() {
            mix[i][0] += s[0];
            mix[i][1] += s[1];
        }
    }
    mix
}

/// Renders a single track (by index) to stereo, ignoring its receives so a
/// stem contains only that track + its own device chain.
pub fn render_solo_stereo(song: &ParsedSong, ti: usize) -> Vec<[f32; 2]> {
    if ti >= song.tracks.len() {
        return Vec::new();
    }
    let mut r = Renderer::with_receives(song, false);
    let buf = r.require(ti).clone();
    buf
}

// ===========================================================================
// Post-processing + WAV output
// ===========================================================================

/// Scales the (mono) mix so its peak reaches ~0.95 (-0.45 dBFS).
pub fn normalize(samples: &mut Vec<f32>) {
    let peak = samples.iter().fold(0.0f32, |a, s| a.max(s.abs()));
    if peak > 0.0 {
        let scale = 0.95 / peak;
        for s in samples.iter_mut() {
            *s *= scale;
        }
    }
}

/// Stereo variant of `normalize`.
pub fn normalize_stereo(samples: &mut [[f32; 2]]) {
    let peak = samples
        .iter()
        .fold(0.0f32, |a, s| a.max(s[0].abs()).max(s[1].abs()));
    if peak > 0.0 {
        let scale = 0.95 / peak;
        for s in samples.iter_mut() {
            s[0] *= scale;
            s[1] *= scale;
        }
    }
}

fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
        pcm.extend_from_slice(&v.to_le_bytes());
    }

    let data_len = pcm.len() as u32;
    let mut out = Vec::with_capacity(44 + pcm.len());

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(&pcm);

    fs::write(path, out).expect("failed to write wav");
}

/// Writes a mono 16-bit PCM WAV using the same base path as the song file but
/// with the `.wav` extension.
pub fn write_wav_at(base_path: &str, samples: &[f32], sample_rate: u32) {
    let path = Path::new(base_path).with_extension("wav");
    write_wav(&path, samples, sample_rate);
}

fn write_wav_stereo(path: &Path, samples: &[[f32; 2]], sample_rate: u32) {
    let mut pcm = Vec::with_capacity(samples.len() * 4);
    for s in samples {
        let l = (s[0] * 32767.0).clamp(-32768.0, 32767.0) as i16;
        let r = (s[1] * 32767.0).clamp(-32768.0, 32767.0) as i16;
        pcm.extend_from_slice(&l.to_le_bytes());
        pcm.extend_from_slice(&r.to_le_bytes());
    }

    let data_len = pcm.len() as u32;
    let mut out = Vec::with_capacity(44 + pcm.len());

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&2u16.to_le_bytes()); // stereo
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 4).to_le_bytes()); // byte rate
    out.extend_from_slice(&4u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(&pcm);

    fs::write(path, out).expect("failed to write wav");
}

/// Writes a stereo 16-bit PCM WAV (interleaved) to `base_path` with a `.wav`
/// extension.
pub fn write_stereo_wav_at(base_path: &str, samples: &[[f32; 2]], sample_rate: u32) {
    let path = Path::new(base_path).with_extension("wav");
    write_wav_stereo(&path, samples, sample_rate);
}