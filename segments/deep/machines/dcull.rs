//! Cull, per frame of the dive for the things drawn over the seabed. Deck: how many points, then for each how far it
//! is ahead of the camera, across and up (tenths of a unit along its forward, right and up vectors). Punches, per
//! point, 1 if it is seen (at least 0.3 ahead, and no more than twice as far across, or as far up, as it is ahead:
//! on the screen or near it), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let ahead = inp();
        let across = abs(inp());
        let up = abs(inp());
        let mut seen = 0;
        if ahead >= 3 && across <= ahead + ahead && up <= ahead { seen = 1; }
        out(seen);
    }
}
