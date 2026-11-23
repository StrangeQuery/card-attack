//! Wall base, once per wall a column's ray meets per frame of the accelerated drive: its block's base and where up
//! the wall (texels and twentieths from the base) the horizon is. Deck: how many, then each: the grade at the block's
//! middle two entries of the plan (eighths), the camera's height (eighths and 64ths over). Punches each the base
//! (their mean, rounded down) and F0 = 10 (h - base) + 10 hf / 8 - 10 twentieths (rounded down) as a texel and
//! twentieths (0-19).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let pair = inp() + inp();
        let mut base = pair / 2;
        if base + base > pair { base -= 1; }
        out(base);
        let d = inp() - base;
        let over = inp();
        // 20 (d / 2) + 10 (d mod 2) + over + over / 4 - 10, the twentieths kept to 0-19
        let mut texel = d / 2;
        if texel + texel > d { texel -= 1; }
        let mut tw = 10 * (d - texel - texel) + over + over / 2 / 2 - 10;
        if tw < 0 { tw += 20; texel -= 1; }
        out(texel);
        out(tw);
    }
}
