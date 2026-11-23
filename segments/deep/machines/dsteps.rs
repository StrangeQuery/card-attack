//! Steps, per tentacle segment of a jellyfish per frame of the dive. Deck: how many segments, then for each its two
//! ends' places on the screen from the middle (tenths of a pixel, across and up). The segment is dotted at least
//! two dots a pixel: punches, per segment, (2 longer side + 9) / 10 steps, at least 1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let y = inp();
        let wide = abs(inp() - x);
        let tall = abs(inp() - y);
        let mut longer = wide;
        if tall > longer { longer = tall; }
        let mut steps = (longer + longer + 9) / 10;
        if steps < 1 { steps = 1; }
        out(steps);
    }
}
