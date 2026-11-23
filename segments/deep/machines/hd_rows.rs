//! Rows, for the dive's creatures at 1536x768: where each row of a box starts its walk (see hd_walk): two places
//! stepped together a row at a time down the box's first column. Deck: the count; then each: how many rows less one
//! (0..998), then for each of the two places its whole and thousandths (0..999) and its step's whole and thousandths
//! (0..999, a step below 0 a whole less). Punches, per row, the two places (whole and thousandths each).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut k = -inp() - 1;
        let mut uh = inp();
        let mut ul = inp();
        let us = inp();
        let uf = inp();
        let mut vh = inp();
        let mut vl = inp();
        let vs = inp();
        let vf = inp();
        loop {
            out(uh);
            out(ul);
            out(vh);
            out(vl);
            // the thousandths and the step's: their sum passes 999 just when one is more than 999 less the other
            if ul > 999 - uf { ul -= 999 - uf; ul -= 1; uh += 1; } else { ul += uf; }
            uh += us;
            if vl > 999 - vf { vl -= 999 - vf; vl -= 1; vh += 1; } else { vl += vf; }
            vh += vs;
            k += 1;
            if k >= 0 { break; }
        }
    }
}
