//! Music tooling for the dev-time examples (song generators, instrument
//! patches, grids, swing helper kit) plus a thin facade over the `wavesabre`
//! renderer. This library is *not* linked into the 64k binary.

pub mod instruments;
pub mod music;
pub mod render;
pub mod swingkit;

pub mod format {
    pub use common::{
        Adultery, Automation, DEVICE_NAMES, DeviceId, Falcon, MidiEvent, ParsedDevice,
        ParsedEvent, ParsedSong, ParsedTrack, Receive, Slaughter, Song, Track, build_chunk,
        chunk_params, decode, encode, env_ms,
    };
}

pub mod sabrewave {
    pub use wavesabre::*;
}
