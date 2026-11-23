//! Rounding down, for the swarm's long division: a number divided by its size's quotient and remainder, rounded down
//! when it was below 0 (one more taken away, and the remainder counted from the other end, unless it divided
//! exactly). Deck: the count; then each: 1 if the number was below 0 (else 0), the quotient and remainder of its size,
//! and the divisor (longs). Punches the quotient as three cards (millions, thousands, rest) and the remainder (0 to
//! the divisor less 1, a long).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let negative = inp();
        let (mut qh, mut ql) = (inp(), inp());
        let (mut rh, mut rl) = (inp(), inp());
        let (dh, dl) = (inp(), inp());
        if negative > 0 {
            if rh > 0 || rl > 0 {
                if ql == 999 { ql = 0; qh += 1; } else { ql += 1; }
                rh = dh - rh;
                rl = dl - rl;
                if rl < 0 { rl += 999; rl += 1; rh -= 1; }
            }
            qh = -qh;
            if ql > 0 { ql = 999 - ql + 1; qh -= 1; }
            if qh < 0 { out(-1); qh += 999; qh += 1; } else { out(0); }
        } else {
            out(0);
        }
        out(qh); out(ql); out(rh); out(rl);
    }
}
