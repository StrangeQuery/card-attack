//! Edge walks, the remainders: an edge's crossing moves a whole step a row, and a hundredth more when the remainder
//! reaches the height. Deck: the count; then each edge: its height and the first row's remainder and the step's
//! (longs), and how many rows it walks. Punches for each row on from the first, 1 when the hundredth more is taken
//! (else 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hh = inp();
        let hl = inp();
        let mut rh = inp();
        let mut rl = inp();
        let sh = inp();
        let sl = inp();
        for _ in 0..inp() {
            // the remainder less the height, plus the step's: kept when not below 0
            rh += sh - hh;
            rl -= hl;
            if rl < 0 { rl += 999; rl += 1; rh -= 1; }
            rl -= 999 - sl;
            if rl > 0 { rl -= 1; rh += 1; } else { rl += 999; }
            if rh < 0 {
                rh += hh;
                rl -= 999 - hl;
                if rl > 0 { rl -= 1; rh += 1; } else { rl += 999; }
                out(0);
            } else {
                out(1);
            }
        }
    }
}
