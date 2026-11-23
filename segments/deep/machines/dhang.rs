//! Hang, once per jellyfish per frame of the dive. Deck: the jellyfish's height from the camera (tenths), how many
//! points of its tentacles, then for each how far down it is (0..12) and its tentacle's length (tenths). Punches,
//! per point, its height from the camera: the jellyfish's, less down / 12 of the length.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let height = inp();
    let count = inp();
    for _ in 0..count {
        let down = inp();
        let length = inp();
        let mut hang = 0;
        for _ in 0..down { hang += length; }
        out(height - hang / 12);
    }
}
