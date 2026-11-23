//! Wobble, per frame of the rotozoomer's wobble: how far each row is pushed sideways. Deck: how far it has grown in
//! (0..15 of 15), then the number of rows n, then each row's 999 sin of its wave's angle. Punches each row's push, 5
//! times the growth over 15 times the sine, in pixels (-5..5), rounded half away from zero.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let grown = inp();
    let n = inp();
    for _ in 0..n {
        let sine = inp();
        // grown |sine| / 2997 rounded: the 999s of grown |sine| + 1498 counted (the sine added grown times, a 999
        // carried whenever the rest reaches it, starting at 1498 as one 999 and 499), then a third of them, cut down
        let add = abs(sine);
        let gap = 999 - add;
        let mut rest = 499;
        let mut nines = 1;
        for _ in 0..grown {
            if rest >= gap { rest -= gap; nines += 1; } else { rest += add; }
        }
        let mut push = 0;
        while nines >= 3 { nines -= 3; push += 1; }
        if sine < 0 { push = -push; }
        out(push);
    }
}
