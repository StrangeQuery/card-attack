//! Pocket, once per move of the Rubik's cube's scramble and solve. Deck: the 54 stickers, each its cubie's place
//! (x, y, z) and its outward normal (x, y, z), each part -1, 0 or 1. Punches each one's pocket: the cubie's number
//! 9 (x + 1) + 3 (y + 1) + z + 1 times 6, plus its face, 2 axis + 1 if the normal points up the axis (0-161), so
//! the host can file the stickers as a card sorter would and deal each face its own.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..54 {
        // the place, 54 x + 18 y + 6 z, from 78 (the middle cubie), by adding
        let mut pocket = 78;
        let mut weight = 54;
        for _ in 0..3 {
            let part = inp();
            if part < 0 { pocket -= weight; }
            if part > 0 { pocket += weight; }
            weight = weight / 3;
        }
        // the face: 0 or 1 along x, 2 or 3 along y, 4 or 5 along z
        let mut face = 0;
        for _ in 0..3 {
            let part = inp();
            if part > 0 { pocket += face + 1; }
            if part < 0 { pocket += face; }
            face += 2;
        }
        out(pocket);
    }
}
