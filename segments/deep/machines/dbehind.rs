//! Behind, once per screen row per frame of the dive while the portal shows. Deck: how far ahead the portal's plane
//! is (tenths of a unit), the first slice's place ahead of the camera (tenths), how many pixels, then for each where
//! it meets the seabed: the slice (-1 for open water) and the tenths of a slice it crossed after the one before.
//! Punches, per pixel, 1 if the portal is seen (the seabed, 10 (k - 1) + s + tenths ahead, is not nearer; further
//! than 50 slices it is behind the portal), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let ahead = inp();
    let start = inp();
    let count = inp();
    for _ in 0..count {
        let slice = inp();
        let tenths = inp();
        let mut seen = 1;
        if slice >= 0 && slice <= 50 && ahead > slice * 10 - 10 + start + tenths { seen = 0; }
        out(seen);
    }
}
