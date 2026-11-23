//! Jelly fade, once per frame of the dive while the jellyfish show. Deck: how much the light lets them glow
//! (32nds), how many, then how far each is ahead (tenths). One between 1 and 60 units ahead is seen; it fades with
//! distance, 64 exp(-d / 14) from a table every 2 units, by the light. Punches, per jellyfish, its fade (64ths;
//! 0 when it is not seen).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let fade = tables::JELLY_FADE;
    let light = inp();
    let count = inp();
    for _ in 0..count {
        let ahead = inp();
        let mut shown = 0;
        if ahead > 10 && ahead < 600 {
            let mut sum = 0;
            for _ in 0..light {
                sum += fade[(ahead / 20) as usize];
                while sum >= 32 { sum -= 32; shown += 1; }
            }
        }
        out(shown);
    }
}
