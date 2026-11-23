//! Muldiv, a few times per frame of the die and the Rubik's cube: a b / c, for the cube's place and sizes scaled to
//! its distance. Deck: how many; then each a (-999..999), b (0..999) and c (1..999). Punches each a b / c, its size
//! rounded half up.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let a = inp();
        let b = inp();
        let c = inp();
        // |a| b / c without the product, which would pass a card: |a| is q c + r, so it is q b, and r added b
        // times, c taken away whenever the rest reaches it, the rest starting at half of c to round
        let mut r = abs(a);
        let mut q = 0;
        while r >= c { r -= c; q += 1; }
        let gap = c - r;                // the rest at or past this carries
        let mut rest = c / 2;
        let mut product = 0;
        for _ in 0..b {
            product += q;
            if rest >= gap { rest -= gap; product += 1; } else { rest += r; }
        }
        if a < 0 { product = -product; }
        out(product);
    }
}
