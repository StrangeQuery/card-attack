//! Walks, per frame of the dive: many walks at once (see walk). Deck: how many, then for each where it starts, as a
//! whole part, a whole amount taken off it and hundredths 0..99; a first step o (hundredths, within 449 either way);
//! a step q (hundredths, within 441 either way); and how many steps k (0..330). Punches, per walk,
//! start - amount + o + k q as a whole part rounded down and hundredths 0..99.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut whole = inp();
        whole -= inp();
        let mut part = inp() + inp();
        let step = inp();
        let mut steps = inp();
        // SAFETY: the host deals 0..330 steps
        unsafe { assume(steps >= 0 && steps <= 330) };
        // k q, k = 10 tens + units: ten steps at a time (10 q is q / 10 whole units and 10 (q mod 10) hundredths),
        // then single steps, each split as whole units and hundredths 0..99 (rounded down); the hundredths kept in
        // 0..99, carrying whole units
        let mut tens = steps / 10;
        let mut units = steps - tens * 10;
        let mut step_whole = step / 10;
        let mut step_part = (step - step_whole * 10) * 10;
        loop {
            while part >= 100 { part -= 100; whole += 1; }
            while part < 0 { part += 100; whole -= 1; }
            if tens > 0 {
                tens -= 1;
            } else {
                if units == 0 { break; }
                units -= 1;
                step_whole = step / 100;
                step_part = step - step_whole * 100;
            }
            whole += step_whole;
            part += step_part;
        }
        out(whole);
        out(part);
    }
}
