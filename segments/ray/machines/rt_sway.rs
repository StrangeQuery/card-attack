//! The ray's ball's sway, once a frame: 28 t / 999 rounded, t 999 sin of the frame's 120ths of a turn (the
//! printed table, data/sine120.txt). Deck: t. Punches the ball's x. With |t| = 36 a + b, 28 |t| = 999 a + 9 a + 28 b:
//! a, and one more when 9 a + 28 b reaches 500 (never a tie: 56 t is even, 999 odd).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let t = inp();
    let neg = t < 0;
    let mut a = 0;
    let mut b = if neg { -t } else { t };
    while b >= 36 { b -= 36; a += 1; }
    let x = 28 * b - 500 + 9 * a;
    let q = if x >= 0 { a + 1 } else { a };
    out(if neg { -q } else { q });
}
