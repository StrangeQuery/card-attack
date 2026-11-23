//! Walks, for the dive's creatures at 1536x768: two places stepped together a pixel at a time across a row, each a
//! whole and thousandths, for the texel (or the strand's place) under each pixel. Deck: the count of walks; then each:
//! how many pixels less one (0..998), then for each of the two places its whole and thousandths (0..999) and its
//! step's whole and thousandths (0..999, a step below 0 a whole less). Punches, per pixel, the two places' wholes.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        // the pixels to punch, counted up from below 0
        let mut k = -inp() - 1;
        // each place's thousandths kept as a count, 999 less them: a step takes its thousandths off, and when the
        // count goes below 0 it comes back up by 1000 and the whole goes on by one more
        let mut uh = inp();
        let mut uc = 999 - inp();
        let us = inp();
        let us1 = us + 1;
        let uf = inp();
        let mut vh = inp();
        let mut vc = 999 - inp();
        let vs = inp();
        let vs1 = vs + 1;
        let vf = inp();
        loop {
            out(uh);
            out(vh);
            uc -= uf;
            if uc < 0 { uc += 999; uc += 1; uh += us1; } else { uh += us; }
            vc -= vf;
            if vc < 0 { vc += 999; vc += 1; vh += vs1; } else { vh += vs; }
            k += 1;
            if k >= 0 { break; }
        }
    }
}
