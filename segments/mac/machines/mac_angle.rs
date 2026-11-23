//! Angle, once per row of the one-bit tunnel's blocks (setup): each block's angle around the middle, 800 a turn, from
//! the right, down first. Deck: for each of the row's 128 blocks, its angle within its octant from the slope table
//! (0-100) and its octant (mac_offset's: 1 steeper than 45 degrees, 2 more left, 4 more above). Punches each block's
//! angle around (0-799).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..128 {
        let mut theta = inp();
        let mut octant = inp();
        let up = octant >= 4;
        if up { octant -= 4; }
        let left = octant >= 2;
        if left { octant -= 2; }
        // steeper than 45 degrees: the angle from the vertical
        if octant > 0 { theta = 200 - theta; }
        if left {
            if up { theta = 400 + theta; } else { theta = 400 - theta; }
        } else if up && theta > 0 {
            theta = 800 - theta;
        }
        out(theta);
    }
}
