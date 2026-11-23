//! Turn, once per move of the Rubik's cube's scramble and solve. Deck: the move's side (-1 or 1) and quarter turns
//! clockwise looking at that face (1, -1 or 2); then the 54 stickers, each its cubie's place, its outward normal
//! (each along the move's axis first, then the two after it in turn: x y z, y z x or z x y) and its colour. A
//! sticker on the layer at that side turns about the axis: its place and normal a quarter turn anticlockwise
//! looking down the axis, (i, j) to (-j, i), -side q times, mod 4. Punches the 54 stickers in the same order.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let side = inp();
    let turns = inp();
    // -side turns, mod 4: anticlockwise quarter turns
    let mut quarters = turns;
    if side > 0 { quarters = -turns; }
    if quarters < 0 { quarters += 4; }
    for _ in 0..54 {
        let along = inp();
        let mut i = inp();
        let mut j = inp();
        let normal = inp();
        let mut ni = inp();
        let mut nj = inp();
        let colour = inp();
        if along == side {
            for _ in 0..quarters {
                let t = i;
                i = -j;
                j = t;
                let t = ni;
                ni = -nj;
                nj = t;
            }
        }
        out(along);
        out(i);
        out(j);
        out(normal);
        out(ni);
        out(nj);
        out(colour);
    }
}
