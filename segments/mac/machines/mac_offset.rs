//! Offset, once per row of the one-bit tunnel's 128x86 blocks (setup): each block's offset from the middle, in half
//! pixels, dx = 8 ix - 507, dy = 8 iy - 337 (odd, so never 0). Deck: the row, iy. Punches for each of its 128 blocks:
//! the longer of |dx| and |dy|, the shorter, and the octant: 1 if it is steeper than 45 degrees, 2 more if left of the
//! middle, 4 more if above it.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let dy = inp() * 8 - 337;
    let down = abs(dy);
    let mut above = 0;
    if dy < 0 { above = 4; }
    let mut dx = -507;
    for _ in 0..128 {
        let across = abs(dx);
        let mut octant = above;
        if dx < 0 { octant += 2; }
        if across < down {
            out(down);
            out(across);
            octant += 1;
        } else {
            out(across);
            out(down);
        }
        out(octant);
        dx += 8;
    }
}
