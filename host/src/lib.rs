//! The host for Card Attack. Every pixel is computed on CARDIAC: the host loads machines, hands each its cards,
//! carries what one punches to the next, and files the paint machines' punched colours as frames. It also runs
//! each segment's independent reference renderer and checks the frames byte for byte. Running machines, dealing
//! decks, counting and checking are the cardiac-host crate's (`runtime/`); this crate is the demo: its segments
//! (`fx`), their tables and timeline, the live player (`play`), and its machines built natively (`machines`).
pub use cardiac_host::{asm, cardiac, live, native, overlay};
pub use cardiac_host::{build, carrying, compiled, finish, hand, machine_file, numbers, one, patched, read, reference, rust, rust_or, Compiled, Host, Machine};
pub mod json;
/// The card sorter: cards put in order by the numbers they hold, a column a pass.
pub mod sorter;
pub mod tables;
pub mod fx;
pub mod machines;
pub mod play;

use std::path::{Path, PathBuf};

pub const W: usize = 96;
pub const H: usize = 48;
pub const FRAME: usize = W * H * 3;

/// A video of 96x48 frames.
pub type Video = cardiac_host::Video<W, H>;


/// Frames at twice the resolution, 192x96, as four 96x48 tiles: the effects drawn that way, and the cut.
pub const W2: usize = 2 * W;
pub const H2: usize = 2 * H;
pub const FRAME2: usize = W2 * H2 * 3;
pub type Video2 = cardiac_host::Video<W2, H2>;

/// The segments folder: beside the executable when there is one (a packaged copy), else found from this crate's
/// location. Every path the host uses is relative to it.
pub fn demo_dir() -> PathBuf {
    let packaged = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("segments"))).filter(|d| d.is_dir());
    packaged.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("segments")).canonicalize().unwrap()
}
