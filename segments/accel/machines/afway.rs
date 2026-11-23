//! A lens flare's ways out, once a light an axis (the host worked them out). Deck: the light's fraction of a
//! pixel along the axis (f, 0..15) and its reach (r). Punches where the samples start on the far side, 16 - f, and
//! on the near, 16 + f, and the reach eight times over (along x the streak is 8 r long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let f = inp();
    let r = inp();
    out(16 - f);
    out(16 + f);
    let r2 = r + r;
    let r4 = r2 + r2;
    out(r4 + r4);
}
