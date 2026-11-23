//! Street dash ends, once per row of the street per frame of the accelerated drive, for the pixels on the stripe: how
//! much of the pixel's length lies past each end of the dash (16ths of a quarter: 0 and 240 of 480). Deck: how many,
//! then each: the place (16ths), the end, the pixel's length (64ths of a unit, aslong's). Punches each
//! (z - l) 32 / w + 16, the quotient toward zero, kept to 0-32.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let d = inp() - inp();
        let w = inp();
        // |d| 32 / w by adding: 32 added |d| times, w taken away whenever the sum reaches it, stopping once past 16
        let mut q = 0;
        let mut sum = -w;
        let mut times = abs(d);
        while times > 0 && q <= 16 {
            sum += 32;
            while sum >= 0 { sum -= w; q += 1; }
            times -= 1;
        }
        if d < 0 { q = -q; }
        q += 16;
        out(if q > 32 { 32 } else if q < 0 { 0 } else { q });
    }
}
