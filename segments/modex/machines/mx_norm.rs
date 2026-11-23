//! Normal, once per ball pixel of the Mode X ray tracer (320x240). Deck: the pixel's offset from the ball's centre,
//! A across and B up. The ball's surface faces the camera by C = isqrt(2116 - A^2 - B^2) (radius 46). A ray from
//! the eye bounces off it in the direction (C A, C B, 1058 - C^2) out of 1058; halved (toward zero), so that it
//! fits a card, (C A / 2, C B / 2, 529 - C^2 / 2) out of 529. Punches C and that direction. In three phases, to fit: the root,
//! X and Y, Z.
// cardiac: phase-cards=2,0
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// c v / 2 toward zero, without the product (it can pass a card): c added |v| / 2 times, half of c more if |v| is odd.
#[inline(never)]
fn half_product(c: i16, v: i16) -> i16 {
    let (pairs, odd) = divide(abs(v), 2);
    let mut sum = 0;
    for _ in 0..pairs { sum += c; }
    if odd > 0 { sum += c / 2; }
    if v < 0 { -sum } else { sum }
}

pub fn main() {
    let a = inp();
    let b = inp();
    // 2116 - A^2 - B^2 passes a card: it is kept as hundreds and units, the squares taken away as odd numbers
    // (A^2 is the first |A| of them), then the root counted as the odd numbers that can still be taken away
    let mut hundreds = 21;
    let mut units = 16;
    let mut odd = 1;
    for _ in 0..abs(a) {
        units -= odd;
        if units < 0 { units += 100; hundreds -= 1; }
        odd += 2;
    }
    odd = 1;
    for _ in 0..abs(b) {
        units -= odd;
        if units < 0 { units += 100; hundreds -= 1; }
        odd += 2;
    }
    let mut c = 0;
    odd = 1;
    while hundreds > 0 || units >= odd {
        units -= odd;
        if units < 0 { units += 100; hundreds -= 1; }
        odd += 2;
        c += 1;
    }
    overlay();                         // C is found; the direction loads over it
    out(c);
    out(half_product(c, a));
    out(half_product(c, b));
    overlay();                         // X and Y are punched; Z loads over them
    // 529 - C^2 / 2 (rounded down) is 529 - (C + C mod 2) (C / 2): taken away a step at a time, from 529 down to
    // no less than -529
    let (half, odd_c) = divide(c, 2);
    let mut z = 529;
    for _ in 0..half { z -= c + odd_c; }
    out(z);
}
