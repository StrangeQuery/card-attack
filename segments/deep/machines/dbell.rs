//! Bell, once per jellyfish per frame of the dive. Deck: how many points of its bell, then for each how side-on it
//! is seen (32nds, 0..31), whether it is on one of the bell's canals (1) and whether it is seen from inside (1). The
//! bell glows most at its edge, more along the canals, and less seen from inside: (0.25 + 0.75 edge^2, from a
//! table, + 0.35 canal) (0.55 inside). Punches, per point, how much white to mix into the glow (0.6 edge, 32nds)
//! and the point's strength (1.5 times the glow, 32nds, for the shade machine).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let glow = tables::BELL_GLOW;
    let count = inp();
    for _ in 0..count {
        let edge = inp();
        // SAFETY: the edge machine punches 0..31
        unsafe { assume(edge >= 0 && edge <= 31) };
        let mut bright = glow[edge as usize] + inp() * 11;
        if inp() == 1 { bright = bright * 5 / 10 + bright * 5 / 100; }
        out(edge * 6 / 10);
        out(bright + bright * 5 / 10);
    }
}
