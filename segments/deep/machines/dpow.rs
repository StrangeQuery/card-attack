//! Power, per screen row per frame of the dive (twice, for the two god-ray waves). Deck: whether to the tenth power
//! (else the sixth), how many, then values v in 40ths (0..40). Punches v^6 or v^10 in 40ths: a wave sharpened into
//! narrow bright shafts.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let tenth = inp();
    let count = inp();
    for _ in 0..count {
        let v = inp();
        let square = by(v, v);
        let fourth = by(square, square);
        let mut sharp = by(fourth, square);
        if tenth == 1 { sharp = by(sharp, fourth); }
        out(sharp);
    }
}

/// a b / 40 rounded down, for a, b 0..40: a added b times, the 40s counted as they fill.
#[inline(never)]
fn by(a: i16, b: i16) -> i16 {
    let mut part = 0;
    let mut whole = 0;
    for _ in 0..b {
        part += a;
        if part >= 40 { part -= 40; whole += 1; }
    }
    whole
}
