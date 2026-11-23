//! Put, per frame of the dive for the kelp, seagrass and fish. Deck: how far the thing is ahead (tenths), how many
//! pixels, then for each how far the nearest thing already in it is (whole units; 999 for open water). Punches, per
//! pixel, 1 where the thing is not further (it is painted) and the distance to keep in it, (ahead + 9) / 10 whole
//! units, else 0 and the distance there.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let ahead = inp();
    let count = inp();
    for _ in 0..count {
        let seen = inp();
        let mut near = seen;
        if near > 99 { near = 99; }
        if near * 10 >= ahead { out(1); out((ahead + 9) / 10); } else { out(0); out(seen); }
    }
}
