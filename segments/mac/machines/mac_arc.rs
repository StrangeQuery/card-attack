//! Arc, once a frame of the Mac's desk while the pointer moves: its place on the glide, off to one side on a slight
//! arc. Deck: the start's x and y, the end's x and y, and mac_glide's four: what is left of the way across and down,
//! and what is left of those (the bows' parts, each within 124). Punches the pointer's x and y: the start, plus the
//! way less what is left, and the bow, 0.32 u (1 - u) of the way the other way, to the right of the way it goes: what
//! is left less what is left of that, times 8 over 25 (32 over 100), toward zero.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let x0 = inp();
    let y0 = inp();
    let across = inp() - x0;
    let down = inp() - y0;
    let left_across = inp();
    let left_down = inp();
    let bow_across = (left_across - inp()) * 8 / 25;
    let bow_down = (left_down - inp()) * 8 / 25;
    out(x0 + across - left_across - bow_down);
    out(y0 + down - left_down + bow_across);
}
