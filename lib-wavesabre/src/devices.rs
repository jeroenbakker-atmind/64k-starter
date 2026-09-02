//! The `Device` enum: a uniform handle over every synth / effect, built from a
//! `format::DeviceId` + chunk, mirroring how `SongRenderer.Track` builds its
//! device chain (synths accumulate output, effects overwrite in place).

use common::DeviceId;

use super::crude::inst_for;
use super::crude::CrudeSynth;
use super::falcon::FalconSynth;
use super::fx::{Cathedral, Chamber, Crusher, Echo, Leveller, Scissor, Smasher, Twister};
use super::slaughter::SlaughterSynth;

pub enum Device {
    Falcon(FalconSynth),
    Slaughter(SlaughterSynth),
    Crude(CrudeSynth),
    Scissor(Scissor),
    Crusher(Crusher),
    Echo(Echo),
    Smasher(Smasher),
    Leveller(Leveller),
    Chamber(Chamber),
    Cathedral(Cathedral),
    Twister(Twister),
}

impl Device {
    /// Builds a device from its serialized chunk. `track_index` keys the crude
    /// fallback's instrument choice; `tempo` feeds tempo-relative delays.
    pub fn build(id: DeviceId, chunk: &[u8], track_index: usize, tempo: f64, sr: f64) -> Device {
        match id {
            DeviceId::Falcon => Device::Falcon(FalconSynth::new(chunk)),
            DeviceId::Slaughter => Device::Slaughter(SlaughterSynth::new(chunk)),
            DeviceId::Scissor => Device::Scissor(Scissor::new(chunk)),
            DeviceId::Crusher => Device::Crusher(Crusher::new(chunk)),
            DeviceId::Echo => Device::Echo(Echo::new(chunk, tempo)),
            DeviceId::Smasher => Device::Smasher(Smasher::new(chunk)),
            DeviceId::Leveller => Device::Leveller(Leveller::new(chunk)),
            DeviceId::Chamber => Device::Chamber(Chamber::new(chunk)),
            DeviceId::Cathedral => Device::Cathedral(Cathedral::new(chunk)),
            DeviceId::Twister => Device::Twister(Twister::new(chunk)),
            // Not synthesizable from the chunk (GM samples / deprecated):
            DeviceId::Thunder | DeviceId::Adultery | DeviceId::Specimen => {
                Device::Crude(CrudeSynth::new(inst_for(track_index), sr))
            }
        }
    }

    pub fn note_on(&mut self, note: u8, vel: u8, sr: f64) {
        match self {
            Device::Falcon(f) => f.note_on(note, vel, sr),
            Device::Slaughter(s) => s.note_on(note, vel, sr),
            Device::Crude(c) => c.note_on(note, vel),
            _ => {}
        }
    }

    pub fn note_off(&mut self, note: u8, sr: f64) {
        match self {
            Device::Falcon(f) => f.note_off(note, sr),
            Device::Slaughter(s) => s.note_off(note, sr),
            Device::Crude(c) => c.note_off(note),
            _ => {}
        }
    }

    pub fn all_notes_off(&mut self) {
        match self {
            Device::Falcon(f) => f.all_notes_off(),
            Device::Slaughter(s) => s.all_notes_off(),
            Device::Crude(c) => c.all_notes_off(),
            _ => {}
        }
    }

    /// Synths accumulate into `l/r` (their per-voice output is summed onto the
    /// buffer); effects overwrite `l/r`, like the core.
    pub fn next_sample(&mut self, sr: f64, l: &mut f32, r: &mut f32) {
        match self {
            Device::Falcon(f) => {
                let (a, b) = f.next_sample(sr);
                *l += a;
                *r += b;
            }
            Device::Slaughter(s) => {
                let (a, b) = s.next_sample(sr);
                *l += a;
                *r += b;
            }
            Device::Crude(c) => {
                let (a, b) = c.next_sample();
                *l += a;
                *r += b;
            }
            Device::Scissor(d) => d.next(sr, l, r),
            Device::Crusher(d) => d.next(sr, l, r),
            Device::Echo(d) => d.next(sr, l, r),
            Device::Smasher(d) => d.next(sr, l, r),
            Device::Leveller(d) => d.next(sr, l, r),
            Device::Chamber(d) => d.next(sr, l, r),
            Device::Cathedral(d) => d.next(sr, l, r),
            Device::Twister(d) => d.next(sr, l, r),
        }
    }

    /// Applies an automation value to the device (param id per the C++ enum).
    pub fn set_param(&mut self, index: usize, value: f32) {
        match self {
            Device::Falcon(f) => f.set_param(index, value),
            Device::Slaughter(s) => s.set_param(index, value),
            Device::Scissor(d) => d.set_param(index, value),
            Device::Crusher(d) => d.set_param(index, value),
            Device::Echo(d) => d.set_param(index, value),
            Device::Smasher(d) => d.set_param(index, value),
            Device::Leveller(d) => d.set_param(index, value),
            Device::Chamber(d) => d.set_param(index, value),
            Device::Cathedral(d) => d.set_param(index, value),
            Device::Twister(d) => d.set_param(index, value),
            Device::Crude(_) => {}
        }
    }
}