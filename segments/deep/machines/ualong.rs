//! Along, once per leaf per frame of the dive (kelp, seagrass). Deck: how many pixels, then for each how far along
//! the leaf it is (hundredths). Punches, per pixel, that in 32nds: 8 along / 25, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        out(inp() * 8 / 25);
    }
}
