//! Depth, for the mesh kit: the nearest of the pieces of triangles on each pixel. Deck: the count of pixels; then
//! each: how many pieces (at least one), and each piece's depth and three values, in the order the triangles are
//! drawn (sorted to their pixels by the host, as a card sorter would). Punches for each pixel the nearest piece's
//! values: the least depth, the first of equals.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let pieces = inp();
        let mut nearest = inp();
        let (mut a, mut b, mut c) = (inp(), inp(), inp());
        for _ in 1..pieces {
            let depth = inp();
            let (x, y, z) = (inp(), inp(), inp());
            if depth < nearest { nearest = depth; a = x; b = y; c = z; }
        }
        out(a);
        out(b);
        out(c);
    }
}
