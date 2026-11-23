//! Slope, once per row of the one-bit tunnel's blocks (setup): how steep each block's offset from the middle is, the
//! step r = 128 shorter / longer, rounded down (0-127), for the slope table. Deck: for each of the row's 128 blocks,
//! the longer offset and the shorter (mac_offset's). Punches each block's r.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..128 {
        let longer = inp();
        let mut rest = inp();
        // a bit at a time (shorter < longer): the remainder doubled, and the longer taken off when it fits. The
        // double would pass a card, so the test is rest >= longer - rest.
        let mut slope = 0;
        for _ in 0..7 {
            slope += slope;
            let other = longer - rest;
            if rest >= other {
                rest -= other;
                slope += 1;
            } else {
                rest += rest;
            }
        }
        out(slope);
    }
}
