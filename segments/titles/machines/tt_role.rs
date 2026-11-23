//! Role: what each pixel of a screen row is in a lower-third title, as a colour code for the title's palette, once
//! per screen row per title. Deck: outline (0 or 1), the shadow's length (0 none, 1 a drop shadow, up to 6 a long
//! one), then for each column from the one before the first to the one after the last (194): the mask there as
//! one card of three digits, above, here and below (each 1 on a letter, 0 off one and past the screen's edges),
//! and the run there on the row above (0..7, from this machine; 7 above the screen and beside it). A pixel on a
//! letter is fill. Off one, its run is one more than the run up and to the left, at most 7: how many steps up the
//! diagonal to the nearest letter. It is outline if the title has one and a letter touches it (any of the 8
//! around it), else shadow if its run is within the shadow's length. Punches for each of the 192 pixels its code
//! (0 none, 1..4 fill at x mod 4 = 0..3, 5 outline, 6 shadow) and its run.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let outline = inp();
    let shadow = inp();
    // the column before the first, then the first
    let mut left = inp();
    let mut left_run = inp();
    let mut middle = inp();
    let mut middle_run = inp();
    for _ in 0..48 {
        for fill in 1..5 {
            let right = inp();
            let right_run = inp();
            let mut run = left_run + 1;
            if run > 7 { run = 7; }
            let mut code = 0;
            if digit(middle, 1) > 0 {
                code = fill;
                run = 0;
            } else if outline > 0 && left + middle + right > 0 {
                code = 5;
            } else if run <= shadow {
                code = 6;
            }
            out(code);
            out(run);
            left = middle;
            middle = right;
            left_run = middle_run;
            middle_run = right_run;
        }
    }
}
