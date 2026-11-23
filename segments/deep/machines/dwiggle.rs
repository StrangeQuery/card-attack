//! Wiggle, once per jellyfish per frame of the dive. Deck: the tentacles' wave this frame (a step of 60), how many
//! points, then for each its tentacle's phase (a step 0..59) and how far down it is (0..12). The wave runs down each
//! tentacle: punches, per point, the step of the wave there, 4 down + phase + wave, round the 60.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let wave = inp();
    let count = inp();
    for _ in 0..count {
        let mut i = inp() + wave;
        let down = inp();
        i += down * 4;
        while i >= 60 { i -= 60; }
        out(i);
    }
}
