//! Edge, once per jellyfish per frame of the dive. Deck: how many points of its bell, then for each how much it
//! faces away from the camera (hundredths, -100..100: below zero toward it). Punches, per point, how side-on it is
//! seen, 32 (100 - |away|) / 100 by tens and units, within 0..31, and 1 if it faces away (seen from inside).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let away = inp();
        let mut side = 100 - abs(away);
        if side < 0 { side = 0; }
        let mut edge = (side / 10 * 32 + side % 10 * 32 / 10) / 10;
        if edge > 31 { edge = 31; }
        out(edge);
        out(if away > 0 { 1 } else { 0 });
    }
}
