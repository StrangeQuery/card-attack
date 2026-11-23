//! Twist, once per picture of the one-bit tunnel: how far each depth twist turns the wall, for the ring table's twists
//! 0..60 (the depth's share, 2400 / R by 0.6 in 50ths of a check). Deck: the picture's weight, 16 sin(0.15 j)
//! (mac_turn's). Punches for each twist 0..60 the twist times the weight over 8, toward zero.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let weight = inp();
    // the product by adding (within 960), eighths by halving three times
    let mut twisted = 0;
    for _ in 0..61 {
        out(twisted / 2 / 2 / 2);
        twisted += weight;
    }
}
