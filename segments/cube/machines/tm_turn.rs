//! The tumble's turn, once a frame (the tumble's frame numbers were the host's). Deck: the frame f. Punches the
//! cube's angle (60ths of a turn): flat and fast for 16 frames (22.5 degrees a frame), then 18 degrees a frame. (Its
//! axis and tilt: tm_axis.)
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

const FLAT: i16 = 16;
const TILT: i16 = 15;

/// x mod m, by taking m away (x 0 or more).
fn modulo(mut x: i16, m: i16) -> i16 {
    while x >= m { x -= m; }
    x
}

/// x / m for x 0 or more, by taking m away.
fn over(mut x: i16, m: i16) -> i16 {
    let mut q = 0;
    while x >= m { x -= m; q += 1; }
    q
}

pub fn main() {
    let f = inp();
    let k = f - FLAT;
    let spun;
    if f < FLAT {
        // 15 (f + 1) / 4, the fifteen as adds
        let mut x = 0;
        for _ in 0..f + 1 { x += 15; }
        spun = over(x, 4);
    } else {
        // from the tilt on, 3 (u + 1) for u the frames since the flat part or since the tilt
        let u = if k < TILT { k + 1 } else { k - TILT + 1 };
        spun = u + u + u;
    }
    out(modulo(spun, 60));
}
