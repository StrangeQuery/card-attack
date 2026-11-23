//! Gather for the tumble's cube, once per frame. Deck: the big cube's 8 corners as seen (x, y), the very corners
//! its edges are drawn from, then corner numbers, two pairs per face for the 6 faces: its diagonals, a to c and b
//! to d. Punches each diagonal, from its first corner to its second (x, y).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    // corner k's x is corners[2k] and its y corners[2k + 1]
    let mut corners = [0i16; 16];
    for i in 0..16 { corners[i] = inp(); }
    for _ in 0..12 {                    // two diagonals for each of the 6 faces
        let from = inp();
        let to = inp();
        // SAFETY: the deck's corner numbers are 0..7
        unsafe {
            assume(from >= 0);
            assume(from < 8);
            assume(to >= 0);
            assume(to < 8);
        }
        let (at_from, at_to) = ((from + from) as usize, (to + to) as usize);
        out(corners[at_to] - corners[at_from]);
        out(corners[at_to + 1] - corners[at_from + 1]);
    }
}
