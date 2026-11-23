//! Sine, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for each
//! the three caustic waves' steps (0..59). Punches, per pixel, how far the waves' sum is from cancelling: |sum| of
//! 20 sin(6 step degrees), from a quarter-wave table.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let quarter: [i16; 16] = load_resource("sine");   // 20 sin(6 k degrees), k = 0..15
    let count = inp();
    for _ in 0..count {
        let mut sum = 0;
        for _ in 0..3 {
            // the quarter wave, turned for each quarter
            let mut i = inp();
            let below = i >= 30;
            if below { i -= 30; }
            if i > 15 { i = 30 - i; }
            // SAFETY: 0..15 now: the host deals steps 0..59
            unsafe { assume(i >= 0 && i <= 15) };
            if below { sum -= quarter[i as usize]; } else { sum += quarter[i as usize]; }
        }
        out(abs(sum));
    }
}
