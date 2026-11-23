//! Outlines, for the turn: a row of the 3D frames, cut where what it or the rows above and below show changes, each
//! piece's pixels marked when a neighbour (above, below, left, right) shows something other than they do. Deck: the
//! count; then each piece of the row: its first pixel and the one after its last (-768..768), what the row above, the
//! row and the row below show there (a face's way 0..5, or 6 for nothing), and what the row shows just left of its
//! first pixel and just right of its last. Punches whether its first pixel is marked (1, else 0), its middle ones (when
//! it has three or more) and its last (when it has two or more). (A marked object's pixel is an outline, a marked
//! empty one the rim.)
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let first = inp();
        let after = inp();
        let above = inp();
        let shows = inp();
        let below = inp();
        let left = inp();
        let right = inp();
        // the middle pixels by the rows above and below; the ends by their side too
        let mut middle = 0;
        if above != shows || below != shows { middle = 1; }
        let mut first_mark = middle;
        if left != shows { first_mark = 1; }
        let mut last_mark = middle;
        if right != shows { last_mark = 1; }
        // its last pixel (the count of its pixels could pass a card)
        let last = after - 1;
        if last == first {
            if last_mark > first_mark { first_mark = last_mark; }
            out(first_mark);
        } else {
            out(first_mark);
            if last - 1 > first { out(middle); }
            out(last_mark);
        }
    }
}
