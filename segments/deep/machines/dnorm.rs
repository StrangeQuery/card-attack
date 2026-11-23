//! Norm, once per pixel for the whole dive (the camera turns, but a pixel's ray is always the same length).
//! Deck: the pixel's column and row. Its ray is 300 (f + a r + b u) with a = A / 200, b = B / 200, A = 2 column -
//! 191 and B = 95 - 2 row, and f, r, u at right angles: 300 sqrt(1 + a^2 + b^2) long, half the root of
//! 360,000 + 9 (A^2 + B^2). Punches that number, which runs to 769,554, as thousands and units, for the root.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let across = abs(inp() * 2 - 191);
    let up = abs(95 - inp() * 2);
    let mut thousands = 360;
    let mut units = 0;
    // 9 A^2 + 9 B^2: nine times over, A added A times and B added B times; units kept under 1000 by taking 500
    // off first
    for _ in 0..9 {
        for i in 0..across + up {
            let mut add = up;
            if i < across { add = across; }
            units += add - 500;
            if units >= 500 { units -= 500; thousands += 1; } else { units += 500; }
        }
    }
    out(thousands);
    out(units);
}
