//! The jet's size and height, once per frame of the accelerated drive while it is in the strip of sky (frames 104 to
//! 139). Deck: k (the frame less 103). Punches 4 k k / 9 (16ths: its half span past 4 pixels), and 160 k k / 81 16ths
//! (how far it has come down) as 10 k k / 81 pixels and 16 r / 81 16ths, r what that left (all rounded down).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let k = inp();
    let mut s4 = 0;
    let mut s4r = 0;
    let mut yp = 0;
    let mut yr = 0;
    for _ in 0..k { for _ in 0..k {
        s4r += 4;
        if s4r >= 9 { s4r -= 9; s4 += 1; }
        yr += 10;
        if yr >= 81 { yr -= 81; yp += 1; }
    } }
    out(s4);
    out(yp);
    let mut yf = 0;
    let mut sum = 0;
    for _ in 0..yr { sum += 16; if sum >= 81 { sum -= 81; yf += 1; } }
    out(yf);
}
