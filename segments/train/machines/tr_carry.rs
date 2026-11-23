//! Edges, for the train, the remainders: from one row to the next an edge's crossing moves by the run times 100 over
//! the height, its whole part and one more each time the remainders reach the height. Deck: the count; then each edge
//! (or piece of one): how many rows (a card), the remainder so far less the height (a long, below 0), the step's
//! remainder (a long) and the height (a long). Punches for each row after the first 1 where one more is carried, else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n = inp();
        let mut c_hi = inp();
        let mut c_lo = inp();
        let r_hi = inp();
        let r_lo = inp();
        let h_hi = inp();
        let h_lo = inp();
        for _ in 1..n {
            c_hi += r_hi;
            let over = c_lo - (999 - r_lo);
            if over > 0 { c_lo = over - 1; c_hi += 1; } else { c_lo = over + 999; }
            if c_hi >= 0 {
                c_hi -= h_hi;
                c_lo -= h_lo;
                if c_lo < 0 { c_lo += 999; c_lo += 1; c_hi -= 1; }
                out(1);
            } else {
                out(0);
            }
        }
    }
}
