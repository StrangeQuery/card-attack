//! The screen's walk, for the turn: from one of the screen's grid points toward the next (20 pixels on, down a grid
//! column or across a row), the place on the picture at each pixel: the first plus the change times the pixel's count
//! over 20, rounded down, the change 20 a + b (tn_step's): a step adds a, and a thousandth more each time the sum of
//! the b's passes 20. Deck: the count; then each walk: 1 to punch the places (else the texels), the first place (a
//! long, thousandths of a texel), a (a long) and b, how many pixels to pass before the first punched and how many to
//! punch. Punches each pixel's place (a long) or its texel (the place's thousands).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let places = inp();
        let mut whole = inp();
        let mut part = inp();
        let step_whole = inp();
        let step_part = inp();
        let twentieth = inp();
        let pass = inp();
        let punch = inp();
        // what the step's part leaves of a thousand, so a sum is never formed past a card
        let room = 999 - step_part;
        let mut sum = 0;
        for k in 0..pass + punch {
            if k >= pass {
                out(whole);
                if places > 0 { out(part); }
            }
            whole += step_whole;
            if part > room { part -= room; part -= 1; whole += 1; } else { part += step_part; }
            sum += twentieth;
            if sum > 19 {
                sum -= 20;
                if part == 999 { part = 0; whole += 1; } else { part += 1; }
            }
        }
    }
}
