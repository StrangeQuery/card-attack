//! Edge sorter. Deck: the 16 numbers the vertex machine punched (x, y of each corner), then 12 edges as corner pairs.
//! Punches each edge as ytop, ybottom, xtop, xbottom.
// cardiac: replaces cube/original/edge_sort.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    // corner k's x is corners[2k] and its y corners[2k + 1]
    let mut corners = [0i16; 16];
    for i in 0..16 { corners[i] = inp(); }
    for _ in 0..12 {                    // the edges
        let a = inp();
        let b = inp();
        // SAFETY: the deck's corner numbers are 0..7
        unsafe {
            assume(a >= 0);
            assume(a < 8);
            assume(b >= 0);
            assume(b < 8);
        }
        let (at_a, at_b) = ((a + a) as usize, (b + b) as usize);
        let (xa, ya) = (corners[at_a], corners[at_a + 1]);
        let (xb, yb) = (corners[at_b], corners[at_b + 1]);
        if yb < ya {                    // the top end first
            out(yb); out(ya); out(xb); out(xa);
        } else {
            out(ya); out(yb); out(xa); out(xb);
        }
    }
}
