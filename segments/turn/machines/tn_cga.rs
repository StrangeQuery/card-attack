//! The CGA tunnel, for the turn, a row a frame: each pixel's colour (0..3 of the palette). Deck: the frame's travel
//! along the tunnel (fifth-hundredths of a ring, 0..999: two rings) and the fade (eight times its step, 0 for none);
//! then each of the 320 pixels: its colour, or 2 for a check; for a check its place round the tunnel, the turn at its
//! depth (tn_twist's) and its place along it (fifth-hundredths, 0..999 each); when fading, its place in the 4x4 dither
//! (6 b + 3). A check is colour 3 when its places round (turned) and along (travelled) are in the same half of their
//! two (both checks even or both odd), else 2; fading, a pixel whose dither is under the fade is black. Punches each
//! pixel's colour.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let travel = inp();
    let fade = inp();
    let room = 999 - travel;
    for _ in 0..320 {
        let mut colour = inp();
        if colour == 2 {
            let mut round = inp();
            let turn = inp();
            let mut along = inp();
            // a sum of two places wrapped to 0..999: it would pass a card, so past what the second leaves it wraps
            if round > 999 - turn { round -= 999 - turn; round -= 1; } else { round += turn; }
            if along > room { along -= room; along -= 1; } else { along += travel; }
            if (round > 499) == (along > 499) { colour = 3; }
        }
        if fade > 0 {
            let dither = inp();
            if dither < fade { colour = 0; }
        }
        out(colour);
    }
}
