//! Post column shade, as many as the host has, per frame of the accelerated drive. Deck: how many, then each: d (16ths
//! from the post's middle), its half width hw (16ths), the post's side (-1 left, 1 right). Punches each the shade,
//! 100ths: 70 plus (on the street's side of the middle) or less 60 |d| / hw (at most 30): -side 60 d / hw toward
//! zero, kept to -30..30.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let d = inp();
        let size = abs(d);
        let hw = inp();
        let lit = (inp() < 0) != (d < 0);
        // 60 |d| / hw by adding, at most 30
        let mut tilt = 0;
        let mut sum = -hw;
        for _ in 0..size { sum += 60; while sum >= 0 && tilt <= 30 { sum -= hw; tilt += 1; } }
        if tilt > 30 { tilt = 30; }
        out(if lit { 70 + tilt } else { 70 - tilt });
    }
}
