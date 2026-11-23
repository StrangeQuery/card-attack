//! Angle, once per list of angles per frame of the polys (and per size of the mesh), for the sine machine. Deck: a
//! base angle, the count, then the angles, each in 960ths of a turn, base and angle together -960..959. Punches for
//! each, base + angle brought into the first quarter turn (0..240): the second half turn is the first's negative,
//! the second quarter the first's mirror. Two cards: the angle there, and 1 for the second half turn (else 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let base = inp();
    let count = inp();
    for _ in 0..count {
        let mut angle = base + inp();
        if angle < 0 { angle += 960; }
        let mut below = false;
        if angle >= 480 { angle -= 480; below = true; }
        if angle > 240 { angle = 480 - angle; }
        out(angle);
        out(if below { 1 } else { 0 });
    }
}
