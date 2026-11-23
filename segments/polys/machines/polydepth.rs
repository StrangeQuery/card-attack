//! Depth, once per frame of the polys: the nearest of the triangles covering each pixel. Deck: for each pixel, how
//! many pieces of triangles cover it, then each piece's depth, normal (three parts) and blob (1 or 2), in the order
//! the triangles are drawn (sorted to their pixels by the host, as a card sorter would); then 999. Punches for each
//! pixel the blob, depth and normal of its nearest piece (the smallest depth, the first of equals), or five zeros if
//! none.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let pieces = inp();
        if pieces == 999 { break; }
        let mut nearest = 999;
        let (mut blob, mut x, mut y, mut z) = (0, 0, 0, 0);
        for _ in 0..pieces {
            let depth = inp();
            let nx = inp();
            let ny = inp();
            let nz = inp();
            let which = inp();
            if depth < nearest { nearest = depth; x = nx; y = ny; z = nz; blob = which; }
        }
        out(blob);
        out(if pieces > 0 { nearest } else { 0 });
        out(x);
        out(y);
        out(z);
    }
}
