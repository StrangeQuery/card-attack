//! Reach, per frame of the dive for the kelp's and seagrass's leaves. Deck: how many leaves, then for each its half
//! width w (tenths), its length L and how far it runs down (tenths). A leaf that runs down more than 2 (w + 10) is
//! steep: on each row its pixels lie within (w + 10) L / |down| across of its middle line. Punches, per leaf, 1 if
//! steep (else 0) and that reach (L added w + 10 times, the |down|s counted as they fill).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let width = inp() + 10;
        let length = inp();
        let down = abs(inp());
        let mut steep = 0;
        let mut reach = 0;
        if down > width + width {
            steep = 1;
            let mut sum = 0;
            // the length is at least |down|, so one |down| comes off each time first: the sum stays under a card
            for _ in 0..width {
                sum += length - down;
                reach += 1;
                while sum >= down { sum -= down; reach += 1; }
            }
        }
        out(steep);
        out(reach);
    }
}
