//! Swing, per frame of the dive for the kelp. Deck: how many points up the stalks, then for each its height up the
//! stalk (tenths) and the sway's wave there, 20 + 20 sin (0..40). The stalk swings further the higher it is:
//! punches, per point, how far across (tenths), height sin 0.15 as (20 sin) height / 100 7 / 10.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let height = inp();
        let wave = inp() - 20;
        let size = abs(wave);
        let mut part = 0;
        for k in 0..3 {
            for _ in 0..digit(height, 2 - k) { part += size; }
            if k < 2 { part /= 10; }
        }
        if wave < 0 { part = -part; }
        out(part * 7 / 10);
    }
}
