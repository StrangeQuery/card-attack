//! Tenths, per screen row per frame of the dive. Deck: how many, then pairs of s (0..10) and a step q (hundredths,
//! within 441 either way). Punches, per pair, s q / 10 rounded down: s tenths of a step.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let tenths = inp();
        let step = inp();
        // SAFETY: the host deals 0..10
        unsafe { assume(tenths >= 0 && tenths <= 10) };
        // s q can be up to 4,410: q is 10 a + b (b -9..9), added s times as tens and units, the units kept in 0..9
        let step_tens = step / 10;
        let step_units = step - step_tens * 10;
        let mut tens = 0;
        let mut units = 0;
        for _ in 0..tenths {
            tens += step_tens;
            units += step_units;
            if units >= 10 { units -= 10; tens += 1; }
            if units < 0 { units += 10; tens -= 1; }
        }
        out(tens);
    }
}
