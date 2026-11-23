//! Pools, once per sample column per frame of the accelerated drive: at each step of the march, the lamps' light on
//! the street and the haze. Deck: how many steps, then each: the falloffs along and across (32nds), how far ahead
//! (quarters). Punches each: along times across / 32, and the haze (32nds: none to 56 units, all by 150:
//! (a - 224) 4 / 47 rounded down, 0-32).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let steps = inp();
    for _ in 0..steps {
        let along = inp();
        let across = inp();
        let ahead = inp();
        // a product by adding, 32 taken away whenever the sum reaches it (32 times 32 would pass a card)
        let mut sum = 0;
        let mut pool = 0;
        for _ in 0..along {
            sum += across;
            while sum >= 32 { sum -= 32; pool += 1; }
        }
        out(pool);
        // (a - 224) 4 / 47: four additions, 47 taken away while it fits
        let mut haze = 0;
        let left = ahead - 224;
        let mut part = 0;
        if left > 0 {
            for _ in 0..4 {
                part += left;
                while part >= 47 { part -= 47; haze += 1; }
            }
        }
        out(if haze > 32 { 32 } else { haze });
    }
}
