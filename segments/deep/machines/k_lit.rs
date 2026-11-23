//! Lit, for the dive's ending at 1536x768: a channel of a texel of the knob once the dial powers up. Deck: the glow's
//! gain over 9 (32nds), how many; then for each the texel's channel without ink and what the dial's glow adds there at
//! full strength. Punches the channel with fl(glow gain / 32) added: the larger added as many times as the smaller,
//! the 32s carried out.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let gain = inp();
    for _ in 0..inp() {
        let mut c = inp();
        let glow = inp();
        let (mut times, mut add) = (gain, glow);
        if glow < gain { times = glow; add = gain; }
        let mut acc = 0;
        for _ in 0..times {
            acc += add;
            while acc >= 32 { acc -= 32; c += 1; }
        }
        out(c);
    }
}
