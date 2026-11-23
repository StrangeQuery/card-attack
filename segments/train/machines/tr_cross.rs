//! Edges, for the train, the crossings: the first row's crossing, then from row to row the step's whole part and the
//! carry tr_carry punched. Deck: the count; then each edge (or piece of one): how many rows (a card), the first row's
//! crossing and the step's whole part (longs), then the carries (one fewer than the rows). Punches each row's
//! crossing, a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let mut x_hi = inp();
        let mut x_lo = inp();
        let s_hi = inp();
        let s_lo = inp();
        out(x_hi);
        out(x_lo);
        for _ in 1..n {
            x_hi += s_hi;
            let over = x_lo - (999 - s_lo);
            if over > 0 { x_lo = over - 1; x_hi += 1; } else { x_lo = over + 999; }
            if inp() > 0 { if x_lo == 999 { x_lo = 0; x_hi += 1; } else { x_lo += 1; } }
            out(x_hi);
            out(x_lo);
        }
    }
}
