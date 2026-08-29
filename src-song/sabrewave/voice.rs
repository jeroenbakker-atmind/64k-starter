//! Voice base state + the shared polyphonic / mono-legato voice pool driver,
//! ported from `WaveSabreCore/SynthDevice.cpp`.

use super::helpers;

/// `SynthDevice::maxVoices`.
pub const MAX_VOICES: usize = 256;
const MAX_ACTIVE_NOTES: usize = 128;

pub struct VoiceBase {
    pub is_on: bool,
    pub note: u8,
    pub detune: f32,
    pub pan: f32,
    pub vibrato_phase: f64,
    slide_active: bool,
    slide_delta: f64,
    slide_samples: i64,
    destination_note: u8,
    current_note: f64,
}

impl VoiceBase {
    pub fn new() -> VoiceBase {
        VoiceBase {
            is_on: false,
            note: 0,
            detune: 0.0,
            pan: 0.5,
            vibrato_phase: 0.0,
            slide_active: false,
            slide_delta: 0.0,
            slide_samples: 0,
            destination_note: 0,
            current_note: 0.0,
        }
    }

    /// `SynthDevice::Voice::NoteOn`.
    pub fn note_on(&mut self, note: u8, detune: f32, pan: f32) {
        self.is_on = true;
        self.note = note;
        self.detune = detune;
        self.pan = pan;
        self.current_note = note as f64;
        self.slide_active = false;
    }

    /// `SynthDevice::Voice::NoteSlide`; `slide_time` is the device `Slide` param
    /// (0..1), which the core scales with `10 * Pow4(slide)`.
    pub fn note_slide(&mut self, note: u8, slide: f32, sr: f64) {
        self.slide_active = true;
        self.destination_note = note;

        let slide_time = 10.0 * helpers::pow4(slide);
        let sr_s = sr.max(1.0);
        self.slide_delta = (note as f64 - self.current_note) / (sr_s * slide_time as f64);
        self.slide_samples = (sr_s * slide_time as f64) as i64;
    }

    /// `SynthDevice::Voice::GetNote`.
    pub fn get_note(&mut self) -> f64 {
        if self.slide_active {
            self.current_note += self.slide_delta;
            self.slide_samples -= 1;
            if self.slide_samples < 0 {
                self.note = self.destination_note;
                self.slide_active = false;
                self.current_note = self.destination_note as f64;
            }
        }
        self.current_note
    }
}

/// A voice owned by a `SynthCore`. Implementations render one stereo sample
/// per `next_sample` call (a per-sample equivalent of the core's block `Run`).
pub trait SynthVoice {
    fn base(&self) -> &VoiceBase;
    fn base_mut(&mut self) -> &mut VoiceBase;

    /// `Voice::NoteOff` (device-specific: releases the envs).
    fn note_off(&mut self);

    /// `Voice::NoteSlide`, shared by all synths.
    fn note_slide(&mut self, note: u8, slide: f32, sr: f64) {
        self.base_mut().note_slide(note, slide, sr);
    }

    /// One stereo sample. The implementation sets `base().is_on = false` when
    /// the note finishes; the pool then removes it.
    fn next_sample(&mut self, sr: f64) -> (f32, f32);
}

/// Shared voice pool + note-mode logic from `SynthDevice`. Voices are spawned
/// lazily up to `MAX_VOICES` (the core preallocates 256 slots and scans for a
/// free one; `deltaSamples` events are merged into the engine's per-sample
/// dispatch, so no event queue is needed).
pub struct SynthCore {
    pub voices: Vec<Box<dyn SynthVoice>>,
    /// 0 = Polyphonic, 1 = MonoLegatoTrill.
    pub voice_mode: u8,
    mono_active: bool,
    note_count: usize,
    active_notes: [bool; MAX_ACTIVE_NOTES],
    note_log: [u8; MAX_ACTIVE_NOTES],
}

impl SynthCore {
    pub fn new() -> SynthCore {
        SynthCore {
            voices: Vec::new(),
            voice_mode: 0,
            mono_active: false,
            note_count: 0,
            active_notes: [false; MAX_ACTIVE_NOTES],
            note_log: [0; MAX_ACTIVE_NOTES],
        }
    }

    pub fn set_voice_mode(&mut self, mode: u8) {
        if self.voice_mode == mode {
            return;
        }
        self.all_notes_off();
        self.voice_mode = mode;
    }

    /// `SynthDevice::NoteOn` (poly) / mono-legato branch. `spawn(note, detune,
    /// pan)` creates one voice.
    #[allow(clippy::too_many_arguments)]
    pub fn note_on(
        &mut self,
        note: u8,
        _vel: u8,
        unisono: usize,
        detune: f32,
        pan: f32,
        slide: f32,
        sr: f64,
        spawn: &mut dyn FnMut(u8, f32, f32) -> Box<dyn SynthVoice>,
    ) {
        if self.voice_mode == 0 {
            let count = if unisono > 1 { (unisono - 1) as f32 } else { 1.0 };
            let pan_at = |f: f32| -> f32 { (f - 0.5) * (pan * 2.0 - 1.0) + 0.5 };
            let mut j = unisono;

            for k in 0..self.voices.len() {
                if j == 0 {
                    break;
                }
                if !self.voices[k].base().is_on {
                    j -= 1;
                    let f = j as f32 / count;
                    self.voices[k] = spawn(note, f * detune, pan_at(f));
                }
            }
            while j > 0 && self.voices.len() < MAX_VOICES {
                j -= 1;
                let f = j as f32 / count;
                self.voices.push(spawn(note, f * detune, pan_at(f)));
            }
        } else {
            self.active_notes[note as usize] = true;
            self.note_log[self.note_count] = note;
            self.note_count += 1;

            if !self.mono_active {
                self.mono_active = true;
                let count = if unisono > 1 { (unisono - 1) as f32 } else { 1.0 };
                let pan_at = |f: f32| -> f32 { (f - 0.5) * (pan * 2.0 - 1.0) + 0.5 };
                let mut j = unisono;

                for k in 0..self.voices.len() {
                    if j == 0 {
                        break;
                    }
                    if !self.voices[k].base().is_on {
                        j -= 1;
                        let f = j as f32 / count;
                        self.voices[k] = spawn(note, f * detune, pan_at(f));
                    }
                }
                while j > 0 && self.voices.len() < MAX_VOICES {
                    j -= 1;
                    let f = j as f32 / count;
                    self.voices.push(spawn(note, f * detune, pan_at(f)));
                }
            } else {
                for v in self.voices.iter_mut() {
                    if v.base().is_on {
                        v.note_slide(note, slide, sr);
                    }
                }
            }
        }
    }

    /// `SynthDevice::NoteOff` (poly) / mono-legato branch.
    pub fn note_off(&mut self, note: u8, slide: f32, sr: f64) {
        if self.voice_mode == 0 {
            for v in self.voices.iter_mut() {
                if v.base().is_on && v.base().note == note {
                    v.note_off();
                }
            }
        } else {
            self.active_notes[note as usize] = false;
            if self.note_count > 0 && note == self.note_log[self.note_count - 1] {
                let mut found_active = false;
                while self.note_count > 0 {
                    self.note_count -= 1;
                    if self.active_notes[self.note_log[self.note_count] as usize] {
                        let target = self.note_log[self.note_count];
                        for v in self.voices.iter_mut() {
                            if v.base().is_on {
                                v.note_slide(target, slide, sr);
                            }
                        }
                        found_active = true;
                        break;
                    }
                }
                if !found_active {
                    self.mono_active = false;
                    for a in self.active_notes.iter_mut() {
                        *a = false;
                    }
                    for v in self.voices.iter_mut() {
                        if v.base().is_on {
                            v.note_off();
                        }
                    }
                }
            }
        }
    }

    /// Next stereo sample from all active voices; finished voices are removed.
    pub fn next_sample(&mut self, sr: f64) -> (f32, f32) {
        let mut l = 0.0f32;
        let mut r = 0.0f32;
        let mut dead = Vec::with_capacity(4);
        for (i, v) in self.voices.iter_mut().enumerate() {
            if !v.base().is_on {
                continue;
            }
            let (a, b) = v.next_sample(sr);
            l += a;
            r += b;
            if !v.base().is_on {
                dead.push(i);
            }
        }
        for i in dead.into_iter().rev() {
            self.voices.swap_remove(i);
        }
        (l, r)
    }

    pub fn all_notes_off(&mut self) {
        for v in self.voices.iter_mut() {
            if v.base().is_on {
                v.note_off();
            }
        }
        self.mono_active = false;
        self.note_count = 0;
        for a in self.active_notes.iter_mut() {
            *a = false;
        }
    }
}