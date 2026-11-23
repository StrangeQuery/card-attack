//! Corner, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: the face (0 a top, else a front or side), the hit's x within its unit (hundredths), its block's top and the
//! blocks either side across (whole units), and the light and rim so far (32nds). Where a top meets a higher block
//! across (within a fifth of a unit of it) it is 5 darker; where it falls to a lower one its edge catches 7 more.
//! Punches, per pixel, the light and the rim.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let face = inp();
        let x_part = inp();
        let top = inp();
        let left = inp();
        let right = inp();
        let mut light = inp();
        let mut rim = inp();
        if face == 0 {
            let mut side = top;
            if x_part < 20 { side = left; }
            if x_part >= 80 { side = right; }
            if side > top { light -= 5; }
            if side < top { rim += 7; }
        }
        out(light);
        out(rim);
    }
}
