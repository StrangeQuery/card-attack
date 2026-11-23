//! Light, once per frame of the jolt: how light each colour on the frame is, for the swirl. Deck: how many; then
//! each colour, red, green and blue. Punches each one's red + green + blue.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let p = read_pixel();
        out(p.r + p.g + p.b);
    }
}
