//! Wave, per screen row per frame of the dive (twice, for the two god-ray waves). Deck: how many, then steps 0..59
//! of a 60-step turn. Punches, per step, 20 + 20 sin(6 step degrees): 0..40, from a quarter-wave table.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let quarter: [i16; 16] = load_resource("sine");   // 20 sin(6 k degrees), k = 0..15
    let count = inp();
    for _ in 0..count {
        let mut i = inp();
        let below = i >= 30;
        if below { i -= 30; }
        if i > 15 { i = 30 - i; }
        // SAFETY: 0..15 now: the host deals steps 0..59
        unsafe { assume(i >= 0 && i <= 15) };
        let v = quarter[i as usize];
        if below { out(20 - v); } else { out(20 + v); }
    }
}
