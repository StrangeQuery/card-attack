//! Amplitude, once per jellyfish per frame of the dive. Deck: how many points of its tentacles, then for each the
//! wave there, 20 + 20 sin (0..40), and how far down it is (0..12). The wave grows toward the tip: punches, per
//! point, (20 sin) down / 100 in tenths of a unit, about 0.25 (down / 12) sin.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let wave = inp() - 20;
        let down = inp();
        let mut v = 0;
        for _ in 0..down { v += wave; }
        out(v / 100);
    }
}
