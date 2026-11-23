//! Ink, for the dive's ending at 1536x768: how far to mix a texel of the knob toward its ink's colour once the dial
//! powers up (32nds). Deck: how many; then for each its ink (32nds) and how much its number's light multiplies it
//! (tenths: 10 + fl(14 lit / 10), lit 0..10 tenths; -1 for the knob's pointer marks). Punches none without ink; the
//! ink for the pointer's marks; else fl(22 amount / 32), at most 32, for the amount fl(ink times / 10).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let ink = inp();
        let times = inp();
        let mut k = 0;
        if ink > 0 {
            if times < 0 { k = ink; } else {
                let mut amount = 0;
                let mut acc = 0;
                for _ in 0..ink { acc += times; while acc >= 10 { acc -= 10; amount += 1; } }
                if amount > 46 { k = 32; } else {
                    // fl(22 amount / 32) = fl(11 amount / 16)
                    acc = 0;
                    for _ in 0..amount { acc += 11; while acc >= 16 { acc -= 16; k += 1; } }
                }
            }
        }
        out(k);
    }
}
