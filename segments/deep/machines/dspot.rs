//! Spot, once per pixel per frame of the dive where the seabed is hit. Deck: the unit along x where the ray crossed
//! the seabed, then the glowing spots between the map row before the hit and the next: how many, and for each its
//! unit along x and its kind and pulse (10 kind + pulse). Punches the kind of the spot in the hit's unit square (0
//! for none, 1 teal, 2 violet) and its pulse (0..9).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let x = inp();
    let count = inp();
    let mut code = 0;
    for _ in 0..count {
        let spot = inp();
        let this = inp();
        if spot == x { code = this; }
    }
    out(digit(code, 1));
    out(digit(code, 2));
}
