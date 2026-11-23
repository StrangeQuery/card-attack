//! Rays, once per frame of the accelerated drive, twice: the camera's 768 columns (and a 769th, the last sample
//! column's) as directions on the street. Deck: the cosine and sine of the turn, times 400, and 0 for the rays across
//! or 1 for the rays along. Across, a and b are the cosine and the sine; along, minus the sine and the cosine.
//! Punches for each column x, 0-768, the direction floor((880 b + (2 x - 767) a) / 800) (our pixels at the
//! screen's distance, 440).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The sum, kept as 800 whole + rest (0-799), which a card can't hold whole, with step added (|step| <= 400).
#[inline(never)]
fn add(whole: i16, rest: i16, step: i16) -> (i16, i16) {
    let mut whole = whole;
    let mut rest = rest + step;
    if rest >= 800 { rest -= 800; whole += 1; }
    if rest < 0 { rest += 800; whole -= 1; }
    (whole, rest)
}

pub fn main() {
    let cos = inp();
    let sin = inp();
    let along = inp();
    let (a, b) = if along == 0 { (cos, sin) } else { (-sin, cos) };
    // 880 b, back 767 a to column 0, then 2 a a column
    let mut sum = (0, 0);
    for _ in 0..880 { sum = add(sum.0, sum.1, b); }
    for _ in 0..767 { sum = add(sum.0, sum.1, -a); }
    for _ in 0..769 {
        out(sum.0);
        sum = add(sum.0, sum.1, a);
        sum = add(sum.0, sum.1, a);
    }
}
