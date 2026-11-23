//! Limit, once per screen row per frame of the dive. Deck: for each of the row's 192 pixels its ray's forward part
//! d (100..441). A ray that has gone k + 1 units ahead of the camera is 300 (k + 1) / d along the view. Punches, per
//! pixel, the last slice before it is further than 110: 11 d / 30 - 1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..192 {
        // 11 d / 30 without the product, which can pass a card: d = 30 thirties + rest
        let mut rest = inp();
        let mut thirties = 0;
        while rest >= 30 { rest -= 30; thirties += 1; }
        out(thirties * 11 + rest * 11 / 30 - 1);
    }
}
