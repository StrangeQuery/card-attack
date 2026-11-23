//! Glow amounts, as many lights as the host has, per frame of the accelerated drive. Deck: how many, then each: the
//! strength (thousandths) and the colour's three parts K (tenths of a step: 8-bit v 200 / 255 of the channel's
//! steps). Punches each channel's amount, s K / 1000 rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let s = inp();
        for _ in 0..3 {
            let k = inp();
            // the smaller counts the additions of the other; 500 taken away whenever the sum reaches it (kept below
            // 0 by starting at -500), and the 500ths halved
            let (mut times, mut each) = (s, k);
            if k < s { times = k; each = s; }
            let mut q = 0;
            let mut sum = -500;
            for _ in 0..times {
                sum += each;
                while sum >= 0 { sum -= 500; q += 1; }
            }
            out(q / 2);
        }
    }
}
