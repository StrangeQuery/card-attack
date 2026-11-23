//! Fine cull, per frame of the dive for the jellyfish. Deck: how many points, then for each how far it is ahead of
//! the camera, across and up (tenths). Punches, per point, 1 if it is at least 0.3 ahead and less far across and up
//! than ahead (so its place on the screen is within 100 pixels of the middle, to a tenth of a pixel), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let ahead = inp();
        let across = abs(inp());
        let up = abs(inp());
        let mut seen = 0;
        if ahead >= 3 && across < ahead && up < ahead { seen = 1; }
        out(seen);
    }
}
