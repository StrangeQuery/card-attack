//! Spec, once per frame of the polys: the highlight. Deck: the count, then each pixel's dot with the half-way vector
//! (in thousandths). Punches the dot to the 24th power, in thousandths: 0 below 0.8, else from the table (every
//! 0.01), between its steps by a straight line (a tenth of the step's rise per thousandth).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");
use demokit::mul::times;

pub fn main() {
    let table: [i16; 21] = load_resource("specular");  // the host punches demokit's table in here
    let count = inp();
    for _ in 0..count {
        let dot = inp();
        let mut s = 0;
        if dot >= 800 {
            let h = dot - 800;
            let step = h / 10;
            // SAFETY: the dot is at most 999, so step is 0..19 and the next 1..20
            unsafe { assume(step >= 0); assume(step <= 19); }
            let at = table[step as usize];
            s = at + times((table[step as usize + 1] - at) / 10, h % 10);
        }
        out(s);
    }
}
