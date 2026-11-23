//! Sine, once per list of angles per frame of the polys (and per size of the mesh). Its program deck carries a
//! quarter turn of the trig table, 99 sin of 0, 3.75 ... 90 degrees (every other card of it, and one more past the
//! end), in `quarter`. Deck: the count, then the angles as the angle machine punches them: in the first quarter
//! turn (0..240, in 960ths of a turn), and 1 for the second half turn (else 0). Punches 99 sin of each, read from the
//! table between its steps (ten to a step) by a straight line.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let quarter: [i16; 26] = load_resource("quarter");   // the host punches the table in here
    let count = inp();
    for _ in 0..count {
        let angle = inp();
        let below = inp();
        let step = angle / 10;
        // SAFETY: the angle is 0..240, so step is 0..24 and the next 1..25
        unsafe { assume(step >= 0); assume(step <= 24); }
        let at = quarter[step as usize];
        let sine = at + demokit::mul::times(quarter[step as usize + 1] - at, angle % 10) / 10;
        if below > 0 { out(-sine); } else { out(sine); }
    }
}
