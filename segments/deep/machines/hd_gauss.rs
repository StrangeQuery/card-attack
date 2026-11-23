//! Gauss, for the dive's jellyfish halos at 1536x768: where each 4 x 4 pixels' fall-off is in its table. Deck: how
//! many, then for each how far across and up it is from the middle (hundredths of the halo's width). Punches, per
//! one, half the squared distance in tenths (each square halved by itself, from tenths toward zero), or -1 when it is
//! more than 31 tenths either way.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// n^2 / 2, n 0..31: the square by adding n n times.
#[inline(never)]
fn half_square(n: i16) -> i16 {
    let mut square = 0;
    for _ in 0..n { square += n; }
    square / 2
}

pub fn main() {
    for _ in 0..inp() {
        let across = abs(inp() / 10);
        let up = abs(inp() / 10);
        if across > 31 || up > 31 { out(-1); } else { out(half_square(across) + half_square(up)); }
    }
}
