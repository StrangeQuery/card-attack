//! Lit, once per leaf per frame of the dive (kelp, seagrass). Deck: whether it is a blade (1), how many pixels of it,
//! then for each how far across the width it is (hundredths, toward the light side above 0), its half width there
//! and how far it is from the leaf (tenths), and how far across it is (tenths). The leaf is lit on one side and in
//! shadow on the other, 0.62 + 0.38 across; a wide blade has a darker rib (0.72) and light at its rim. Punches, per
//! pixel, the light (32nds) and the rim's light (0 or 11, 32nds).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let blade = inp();
    let count = inp();
    for _ in 0..count {
        let mut side = inp();
        let width = inp();
        let distance = inp();
        let across = inp();
        if side > 100 { side = 100; }
        if side < -100 { side = -100; }
        let mut lit = 20 + side * 12 / 100;
        if blade == 1 && width > 12 && distance < 5 { lit = lit * 72 / 100; }
        let mut rim = 0;
        if width > 12 && distance > width * 8 / 10 && across > 0 { rim = 11; }
        out(lit);
        out(rim);
    }
}
