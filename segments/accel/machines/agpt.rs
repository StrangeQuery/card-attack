//! Glow places, once per row of a light's glow per frame of the accelerated drive: how far out its profile is at
//! each pixel of the row, an octagon close to a circle. Deck: the row's t and 3 t / 8 (agaxis's), how many pixels,
//! then each its t and 3 t / 8 across. Punches each the greater t and 3 / 8 of the lesser (more than 192: none).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let ty = inp();
    let ey = inp();
    for _ in 0..inp() {
        let tx = inp();
        let ex = inp();
        out(if tx >= ty { tx + ey } else { ty + ex });
    }
}
