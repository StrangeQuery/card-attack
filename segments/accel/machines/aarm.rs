//! Post arm pixels, as many as the host has, per frame of the accelerated drive, a channel at a time: the arm where it
//! is nearer, hazed by what is behind it. Deck: the sky's channel, how many, then each: the pixel's distance, the
//! post's (quarters), the haze of the pixel's distance (32nds; afog's), the pixel's channel, the arm's (tenths of a
//! step). Punches each the channel: where the post is nearer, the arm's moved toward the sky by that haze, rounded
//! toward the arm's; else the pixel's.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let sky = inp();
    for _ in 0..inp() {
        let nearer = inp() > inp();
        let k = inp();
        let mut from = inp();
        let arm = inp();
        if nearer {
            from = arm;
            let mut rest = abs(sky - arm);
            let mut step = 0;
            while rest >= 32 { rest -= 32; step += k; }
            let mut part = 0;
            for _ in 0..k { part += rest; if part >= 32 { part -= 32; step += 1; } }
            if sky < arm { from -= step; } else { from += step; }
        }
        out(from);
    }
}
