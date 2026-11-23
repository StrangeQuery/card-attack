//! Lift, once per frame of the polys: a pixel's light brought up toward a unit normal's. Deck: the count, then for
//! each a value (in thousandths) and the lift. Punches value + value / 10 x lift / 100, held within -999..999 (the
//! value tested against the limit less the step, as the sum itself may not fit on a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");
use demokit::mul::by100;

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let v = inp();
        let step = by100(v / 10, inp());
        let lifted = if step < 0 {
            if v >= -999 - step { v + step } else { -999 }
        } else if v <= 999 - step {
            v + step
        } else {
            999
        };
        out(lifted);
    }
}
