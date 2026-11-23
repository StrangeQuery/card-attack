//! The swerve past the slow car ahead, in the camera's path and our car's lane: each frame, which entry of the swerve
//! table (data/laneswerve.txt) applies. The frames are k = 12 t from -3 on, as aphase's. Deck: how many frames.
//! Punches each frame's entry, 0-21; the host fetches it.
//!
//! The swerve past a slow car is smooth(26, 14, u) times smooth(-10, -2, u), where u is how far ahead the car is
//! (its place at frame 0, less k). One of the two is always 0 or 1: from u = 26 down to 15 the first rises
//! (entries 1-11, a = 26 - u), and from u = 14 down to -9 the second falls (entries 13-21, 13 + b, b = u + 10 up to
//! 8). Outside that, entry 0, no swerve. The cars are 55 units apart, so only one is ever swerved round.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The slow cars' places at frame 0, in units.
const SLOW: [i16; 5] = [40, 95, 150, 205, 250];

pub fn main() {
    let frames = inp();
    for i in 0..frames as usize {
        let k = i as i16 - 3;
        let mut entry = 0;
        for c in 0..5 {
            let u = SLOW[c] - k;
            if u < 26 && u > -10 {
                if u > 14 { entry = 26 - u; } else if u > -2 { entry = 21; } else { entry = u + 23; }
            }
        }
        out(entry);
    }
}
