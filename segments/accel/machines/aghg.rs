//! Ghost middles, per ghost per frame of the accelerated drive. Deck: how many, then each: the light's pixel p, the
//! way (1 toward a middle further on, -1 back), aghc's q and whether anything was left. Punches each p + q, or p - q
//! less 1 more if anything was left (rounded down), or 999 for q past 995 or a middle past 990 pixels either way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let p = inp();
        let way = inp();
        let mut q = inp();
        let left = inp();
        if q > 995 { out(999); continue; }
        if way < 0 {
            q += left;
            // p - q, below -990 or past 990 (each sum kept within a card)
            if p < 0 { if q > 990 + p { out(999); continue; } } else if q - p > 990 { out(999); continue; }
            if p - q > 990 { out(999); continue; }
            out(p - q);
        } else {
            if p > 0 { if q > 990 - p { out(999); continue; } } else if q + p > 990 || q + p < -990 { out(999); continue; }
            out(p + q);
        }
    }
}
