//! Gather: picks corners out of the projected list. Deck: the 8 projected corners (x, y), then corner numbers, three
//! per face for the 6 faces. Punches each numbered corner's (x, y).
// cardiac: replaces cube/original/face_gather.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    // corner k's x is corners[2k] and its y corners[2k + 1]
    let mut corners = [0i16; 16];
    for i in 0..16 { corners[i] = inp(); }
    for _ in 0..18 {                    // three corners for each of the 6 faces
        let corner = inp();
        // SAFETY: the deck's corner numbers are 0..7
        unsafe {
            assume(corner >= 0);
            assume(corner < 8);
        }
        let at = (corner + corner) as usize;
        out(corners[at]);
        out(corners[at + 1]);
    }
}
