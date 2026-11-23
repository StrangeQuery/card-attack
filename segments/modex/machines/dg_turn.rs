//! Turn, three times per row of the 320x240 frame per frame of the jolt (once for each colour): each pixel's angle
//! turned on by the colour's phase. Deck: the phase as a quarter turn (0-3) and tenths (0-899), how many; then each
//! angle the same way. Punches each sum the same way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let quarters = inp();
    let tenths = inp();
    let count = inp();
    let gap = 900 - tenths;
    for _ in 0..count {
        let mut quarter = inp() + quarters;
        let mut rest = inp();
        if rest >= gap { rest -= gap; quarter += 1; } else { rest += tenths; }
        if quarter >= 4 { quarter -= 4; }
        out(quarter);
        out(rest);
    }
}
