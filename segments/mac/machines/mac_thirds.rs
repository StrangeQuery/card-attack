//! A title card's dissolve, once a frame of the tunnel's titles: how many thirds of it show. Deck: the frame's
//! place in the card's showing (k, from 0), and how long it shows (n), or 0 for a card that stays. Punches the
//! thirds: one more each frame in, one fewer each frame out at the end, at most three.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let k = inp();
    let n = inp();
    let mut t = k + 1;
    if n > 0 && n - k < t { t = n - k; }
    if t > 3 { t = 3; }
    out(t);
}
