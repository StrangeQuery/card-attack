//! Top, once per screen row per frame of the dive. Deck: how many pixels of the row hit the near seabed, then for
//! each: the face it hit (0 a top, else a front or side), how many tenths of the slice in, and its block's top, the
//! block in front's and behind's (over the hit's row base, whole units). On a top, punches per pixel: the light in
//! 32nds, 29, less toward its back where the block behind rises over it (5 a tenth from 0.8 on) and 6 less when that
//! stands four units or more over it; how much its front edge catches the light over a lower block in front (14
//! 32nds, the first two tenths); and whether it is its back running into the wall behind (1, rubble, from 0.6 on) or
//! its front over a drop (2, a ledge's lip, to 0.2). On a front or side: 0, 0, 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let face = inp();
        let tenths = inp();
        let top = inp();
        let front = inp();
        let back = inp();
        let mut light = 0;
        let mut rim = 0;
        let mut detail = 0;
        if face == 0 {
            light = 29;
            if front < top {
                if tenths <= 1 { rim = 14; }
                if tenths <= 2 { detail = 2; }
            }
            if back > top {
                if tenths >= 6 { detail = 1; }
                if tenths >= 8 { light -= (tenths - 7) * 5; }
            }
            if back >= top + 4 { light -= 6; }
        }
        out(light);
        out(rim);
        out(detail);
    }
}
