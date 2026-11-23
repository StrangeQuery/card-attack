//! Traffic, once per frame of the accelerated drive: how far ahead each car is (the drive's dcarz, for any number of
//! cars). Deck: the frame f, the camera's place in its entry of the plan (quarters) and the entry, how many cars, then
//! each: slow (0) or oncoming (1) and its number. A slow car, going 5 quarters a frame to our 9, is its number - 4 f
//! ahead; an oncoming one, going 7, meets us 28 quarters ahead at the frame its number says, so is 28 + 16 (number -
//! f) ahead, or 828 (out of sight) if that is more than 50 frames off. Punches for each car how far ahead
//! (quarters), the plan's entry there (at least the camera's) and how far into it (quarters).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let frame = inp();
    let into = inp();
    let entry = inp();
    for _ in 0..inp() {
        let oncoming = inp() > 0;
        let number = inp();
        let ahead = if oncoming {
            let frames = number - frame;
            if frames > 50 || frames < -50 { 828 } else { 28 + 16 * frames }
        } else { number - 4 * frame };
        out(ahead);
        let mut place = into + if ahead > 0 { ahead } else { 0 };
        let mut at = entry;
        while place >= 8 { place -= 8; at += 1; }
        out(at);
        out(place);
    }
}
