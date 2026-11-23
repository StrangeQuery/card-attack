//! Post shade, as many as the host has, per frame of the accelerated drive: the post's colour shaded. Deck: the post's
//! red, green and blue (tenths of a step), how many, then each a shade (100ths). Punches each the three channels
//! times the shade over 100, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let red = inp();
    let green = inp();
    let blue = inp();
    for _ in 0..inp() {
        let shade = inp();
        for k in 0..3 {
            let v = if k == 0 { red } else if k == 1 { green } else { blue };
            // v times the shade over 100: the shade added v times, 100 taken away whenever the sum reaches it
            let mut q = 0;
            let mut sum = 0;
            for _ in 0..v { sum += shade; while sum >= 100 { sum -= 100; q += 1; } }
            out(q);
        }
    }
}
