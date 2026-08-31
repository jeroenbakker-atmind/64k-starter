/// Instrument patches, grouped by the synth device they run on.
///
/// The Falcon-family patches (bass, drums, flute, piano, saxophone) are the
/// established set and are re-exported at this crate root, so
/// `use starter::instruments::bass;` keeps working unchanged.
pub mod falcon;
pub mod slaughter;

pub use falcon::{bass, clarinet, drums, flute, piano, saxophone};