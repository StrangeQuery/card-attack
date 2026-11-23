//! Paint, as many as the host has, per frame of the accelerated drive, a channel at a time: a thing laid over a pixel
//! where it is nearer. Deck: how many, then each: the pixel's distance, the thing's (quarters), its cover (32nds),
//! the pixel's channel, the thing's (tenths of a step). Punches each the channel (moved toward the thing's by the
//! cover's 32nds of the way, rounded toward where it was, where the thing is nearer) and the distance (the thing's
//! where it is nearer and covers more than half).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut far = inp();
        let near = inp();
        let k = inp();
        let mut from = inp();
        let to = inp();
        if far > near {
            // |to - from| = 32 q + r: k q + k r / 32 (the product would pass a card)
            let mut rest = abs(to - from);
            let mut step = 0;
            while rest >= 32 { rest -= 32; step += k; }
            let mut part = 0;
            for _ in 0..k { part += rest; if part >= 32 { part -= 32; step += 1; } }
            if to < from { from -= step; } else { from += step; }
            if k > 16 { far = near; }
        }
        out(from);
        out(far);
    }
}
