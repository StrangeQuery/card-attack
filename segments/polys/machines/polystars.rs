//! Stars, once for the polys' sky: the random numbers that place and light its 140 stars. Deck: nothing. Punches
//! 420 numbers, a tenth of each seed in turn from 801 (0..99): three a star, for its x, y and brightness.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut seed = 801;
    for _ in 0..420 {
        seed = lcg::next(seed);
        out(seed / 10);
    }
}
