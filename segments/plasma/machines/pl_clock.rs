//! Clock, once per frame of the plasma at 192x96 (and of the ray's plasma floor). Deck: the plasma's time tf in
//! frames (0..359). Each wave moves at its own speed round its turn, and the colours round their ring of 256.
//! Punches where each starts this frame: the wave across 6 tf, the rolling one 2 tf and the slanting one
//! 77 + 2 tf (turns of 96), the rings -2 tf (a turn of 64), the colours 8 tf (of 256). Each from tf mod the frames
//! a turn takes, made 0 or more, so that nothing passes three digits.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let tf = inp();
    let t16 = remainder(tf, 16);
    let t32 = remainder(tf, 32);
    let t48 = remainder(tf, 48);
    out(6 * t16);                      // across
    out(t48 + t48);                    // rolling
    out(77 + t48 + t48);               // slanting
    out(64 - t32 - t32);               // the rings spread out
    out(8 * t32);                      // the colours turn
}
