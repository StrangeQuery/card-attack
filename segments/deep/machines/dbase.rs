//! Base, once per frame of the dive for the kelp and seagrass. Deck: the camera's height (tenths), how many roots,
//! then for each its map row's base (whole units) and the seabed's height over the base under it (tenths). Punches,
//! per root, its height from the camera in tenths: 10 base + height - camera.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let camera = inp();
    let count = inp();
    for _ in 0..count {
        let base = inp();
        out(base * 10 + inp() - camera);
    }
}
