//! Pick, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: whose block it hit (0 the column a slice before, 1 the column at the hit), the x of each (whole units), and
//! the blocks of the hit's row in and either side of each column (three and three). Punches, per pixel: the hit
//! block's column (x, whole units), its top and the blocks either side of it across.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let last = inp();
        let mut x = inp();
        let x_last = inp();
        let mut left = inp();
        let mut top = inp();
        let mut right = inp();
        let left_last = inp();
        let top_last = inp();
        let right_last = inp();
        if last == 1 { x = x_last; left = left_last; top = top_last; right = right_last; }
        out(x);
        out(top);
        out(left);
        out(right);
    }
}
