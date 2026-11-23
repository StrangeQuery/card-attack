//! Across the street, once per sample column per frame of the accelerated drive: at each step of the march, how far
//! the ray is from the street's middle. Deck: the camera's place across (eighths), the ray's side (-1 left, 1 right),
//! how many steps, then each: the street's middle at the step's entry (eighths) and its 64ths further (afine's), how
//! far across the ray is (eighths, at most 400, and 64ths over). Punches each: how far from the middle (64ths,
//! signed; 999 past 117 eighths), and the place in the lamps' across falloff (sixteenths of a unit, at most 160).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let camera = inp();
    let side = inp();
    let steps = inp();
    for _ in 0..steps {
        let middle = inp();
        let lean = inp();
        let mut whole = inp();
        let mut over = inp();
        if side < 0 { whole = -whole; over = -over; }
        whole += camera - middle;
        let mut off = 999;
        if whole < -117 { off = -999; } else if whole <= 117 { off = 8 * whole + over - lean; }
        out(off);
        let quarter = abs(off) / 2 / 2;
        out(if quarter > 160 { 160 } else { quarter });
    }
}
