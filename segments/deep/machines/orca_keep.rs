//! Orca hidden, for the orcas in the dive at 1536x768: whether each pixel of an orca shows. Deck: how many pixels,
//! then for each how far ahead the orca is there (whole units) and the dive's depth there (whole units, 999 for open
//! water). Punches, per pixel, 1 if the orca is no more than a unit further than the dive's depth (at most 99), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let ahead = inp();
        let mut depth = inp();
        if depth > 99 { depth = 99; }
        if ahead > depth + 1 { out(0); } else { out(1); }
    }
}
