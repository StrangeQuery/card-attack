//! Step, per tentacle segment of a jellyfish per frame of the dive (once for across, up and ahead). Deck: how many
//! segments, then for each its two ends' values and its steps (1..). Punches, per segment, steps + 1 values from one
//! end to the other, each a + (b - a) s / steps toward zero (a remainder carried, no product).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let change = inp() - a;
        let steps = inp();
        let mut gone = 0;
        let mut rest = 0;
        for _ in 0..steps + 1 {
            out(a + gone);
            rest += abs(change);
            while rest >= steps {
                rest -= steps;
                if change < 0 { gone -= 1; } else { gone += 1; }
            }
        }
    }
}
