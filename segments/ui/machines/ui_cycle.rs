//! Cycle, once a frame of a UI layer with a busy spinner: each spoke's grey. Deck: the spinner's phase (0 or more),
//! how many spokes, the darkest grey and the lightest. Punches each spoke's grey: the spoke at the phase the
//! darkest, each one after it round a step lighter, the one before it the lightest.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let phase = inp();
    let spokes = inp();
    let dark = inp();
    let light = inp();
    let last = spokes - 1;
    // the first spoke's place after the phase, round the spinner
    let mut place = -phase;
    while place < 0 { place += spokes; }
    for _ in 0..spokes {
        out(dark + muldiv(place, light - dark, last));
        place += 1;
        if place == spokes { place = 0; }
    }
}
