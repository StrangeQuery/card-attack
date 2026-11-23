//! Mix, as many as the host has, per frame of the accelerated drive, a channel at a time: a colour laid over by
//! others in turn. Deck: how many, then each: how many layers, the channel (tenths of a step), then each layer: its
//! channel and its cover k (32nds). Punches each channel, every layer moving it toward the layer's by k 32nds of the
//! way, rounded toward where it was: |to - from| = 32 q + r, so k times it over 32 is k q + k r / 32 (the product
//! would pass a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let layers = inp();
        let mut from = inp();
        for _ in 0..layers {
            let to = inp();
            let k = inp();
            if k == 32 { from = to; }
            if k == 0 || k == 32 { continue; }
            let mut rest = abs(to - from);
            let mut step = 0;
            while rest >= 32 { rest -= 32; step += k; }
            let mut part = 0;
            for _ in 0..k {
                part += rest;
                if part >= 32 { part -= 32; step += 1; }
            }
            if to < from { from -= step; } else { from += step; }
        }
        out(from);
    }
}
