//! Speck, once per frame of the dive. Deck: how much the light lets the snow show, 11 + 4 light / 10 (32nds; light
//! in 32nds), how many specks are seen, then for each how far it is ahead (tenths) and how far the seabed is in its
//! pixel (whole units; 999 for open water). Punches, per speck, how much of the snow's colour to mix into its pixel
//! (32nds): fading out by 40 ahead, (40 - ahead) show / 40; 0 where the seabed is nearer.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let show = inp();
    let count = inp();
    for _ in 0..count {
        let ahead = inp() / 10;
        let seabed = inp();
        let mut amount = 0;
        if ahead < 40 && ahead <= seabed {
            let mut sum = 0;
            for _ in 0..show {
                sum += 40 - ahead;
                if sum >= 40 { sum -= 40; amount += 1; }
            }
        }
        out(amount);
    }
}
