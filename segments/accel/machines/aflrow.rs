//! Flare rows, per row, side and channel of a flare per frame of the accelerated drive: the falloff's three indexes at
//! each pixel out from the light's column. Deck: the row's gy, y1 and y2, the channel's streak index and star index,
//! how many pixels, then each its gx, x2 and x1 (aflax's, afsq's). Punches each pixel's gy + gx + streak, y1 + x2 +
//! star and x1 + y2 + star, and stops at the first where all three are 192 or more (none further out reaches it).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let gy = inp();
    let y1 = inp();
    let y2 = inp();
    let sv = inp();
    let cv = inp();
    for _ in 0..inp() {
        let t1 = gy + inp() + sv;
        let t2 = y1 + inp() + cv;
        let t3 = inp() + y2 + cv;
        if t1 >= 192 && t2 >= 192 && t3 >= 192 { break; }
        out(t1);
        out(t2);
        out(t3);
    }
}
