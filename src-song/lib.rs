//! Shared code for the music tooling (`examples/create_song`,
//! `examples/validate_song`, `examples/inspect_gm_dls`), living in `src-song/`
//! so it never mixes with the 64k executable in `src/`.
//!
//! This library is *not* linked into the 64k binary; it is only used by the
//! dev-time examples.

pub mod format;
pub mod instruments;
pub mod music;
pub mod render;