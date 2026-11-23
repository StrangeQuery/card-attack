//! Wall column, once per wall a column's ray meets per frame of the accelerated drive, a channel at a time: each row's
//! colour, between the texel row under it and the one above (rows of several walls one after another: a wall's first
//! row is always new). Deck: how many rows, then each: 1 when its texel rows
//! are new (then their two channels, tenths of a step: under, above; awblend's), else 0; then the row's twentieths of
//! the way up (0-19). Punches each row's channel: under + (above - under) w / 20, the step rounded toward under; while
//! the texel rows stay the same the step is carried on from the last row's (|above - under| = 20 q + r: the step is
//! q w + r w / 20, w grown a twentieth at a time).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let rows = inp();
    let mut under = 0;
    let mut down = false;
    let mut q = 0;
    let mut r = 0;
    let mut w = 0;
    let mut step = 0;
    let mut part = 0;
    for _ in 0..rows {
        if inp() > 0 {
            under = inp();
            let above = inp();
            down = above < under;
            let size = abs(above - under);
            q = size / 20;
            r = size - 20 * q;
            w = 0;
            step = 0;
            part = 0;
        }
        let to = inp();
        while w < to {
            w += 1;
            step += q;
            part += r;
            if part >= 20 { part -= 20; step += 1; }
        }
        out(if down { under - step } else { under + step });
    }
}
