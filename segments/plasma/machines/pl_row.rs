//! Row of the plane, once per tile row of the plasma at 192x96 and of the ray's plasma floor. Deck: the frame's
//! row y (0..95), the tile's place across (0 or 96), the plane's tilt as 600 cos and 100 sin, and the ball's
//! shadow: the ball's x (floor units), the shadow's radius (0 for none) and darkness (32nds). The plane stands on
//! the floor where the bottom of the screen meets it and tilts back about that line; tilted right back it is the
//! floor. The row's slope k5 = 600 cos - 5 (36 - y) sin / 100 is where the row meets the plane: at or below 0 it
//! sees sky (the band of its row), under 60 haze; otherwise the row is at v = 600 (96 - y) / k5 along the plane, at
//! depth z = 120 + v sin / 100, with fog (z - 240) / 7 (all haze from 32 on). Across it u = (x - 96) z / 120, stepped
//! by adding z to a remainder. Punches for each of the 96 pixels its look (a sky or haze code, or 100 + the
//! shadow's darkness times 32 if it is in the shadow, plus the fog), u and v (0 and 48 where there is no plasma).
//! In the shadow: |u - 15 x / 4| at most the shadow's half width in this row, the largest h with h^2 + (v - 105)^2
//! under radius^2. In six phases, to fit: the first reads the seven cards and finds the slope; then v; z and the
//! fog; where u starts and the shadow's centre; the shadow's half width; the row.
// cardiac: phase-cards=7,0,0,0,0
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// a b / 100 rounded down, for a 0..999 and b 0..100, without the product: a's hundreds and the rest of it added b
/// times, a hundred of the rest carried.
fn percent(a: i16, b: i16) -> i16 {
    let (hundreds, rest) = divide(a, 100);
    let mut whole = 0;
    let mut part = 0;
    for _ in 0..b {
        whole += hundreds;
        part += rest;
        if part >= 100 {
            part -= 100;
            whole += 1;
        }
    }
    whole
}

pub fn main() {
    let y = inp();
    let tile_x = inp();
    let cos600 = inp();
    let sin100 = inp();
    let ball_x = inp();
    let radius = inp();
    let dark = inp();
    let above = 36 - y;                // rows above the horizon
    let lean = percent(5 * abs(above), sin100);
    let k5 = if above >= 0 { cos600 - lean } else { cos600 + lean };
    overlay();                         // the slope is found; where the row meets the plane loads over that
    let mut fixed = 3;                 // a code if the row shows no plasma: haze
    let mut v = 48;
    let mut z = 0;
    let mut fog = 0;
    if k5 <= 0 {
        fixed = y / 2;                 // the sky's bands by the row
        fixed = if fixed < 6 { 0 } else if fixed < 13 { 1 } else if fixed < 19 { 2 } else { 3 };
    } else if k5 >= 60 {
        // 600 (96 - y) / k5 without the product: 96 - y added 600 times, k5 taken away whenever it fits
        let mut part = 0;
        v = 0;
        for _ in 0..600 {
            part += 96 - y;
            while part >= k5 {
                part -= k5;
                v += 1;
            }
        }
    }
    overlay();                         // v is found; the depth and fog load over that
    if k5 >= 60 {
        z = 120 + percent(v, sin100);
        let mut far = z - 240;
        while far >= 7 && fog < 32 {   // the fog, (z - 240) / 7, by taking away 7s
            far -= 7;
            fog += 1;
        }
        if fog < 32 { fixed = -1; }   // the plasma, not all haze
    }
    overlay();                         // the depth and fog are found; the start across loads over that
    let mut u = 0;
    let mut part = 0;                  // the remainder of (x - 96) z / 120
    if tile_x < 96 {                   // from x = 0: u = z / 5 - z, the remainder 24 (z mod 5)
        let (fifth, rest) = divide(z, 5);
        u = fifth - z;
        part = 24 * rest;
    }
    let mut centre = 15 * ball_x;      // the shadow's centre across, 15 x / 4 rounded down
    let mut quarters = 0;
    while centre < 0 { centre += 4; quarters -= 1; }
    while centre >= 4 { centre -= 4; quarters += 1; }
    let shade = 100 + 32 * dark + fog;
    overlay();                         // the start across and the shadow's centre are found; its width loads over that
    let off = abs(v - 105);
    let mut half_width = -1;
    if off < radius {
        // the largest h with h^2 < radius^2 - off^2: with 968 added so that it fits a card, 968 - (radius - off)
        // (radius + off) by subtracting, then odd numbers added while they keep it under 968
        let mut gap = 968;
        for _ in 0..radius - off { gap -= radius + off; }
        let mut odd = 1;
        while gap + odd < 968 {
            gap += odd;
            odd += 2;
        }
        half_width = odd / 2;
    }
    overlay();                         // the row is set; the stepping across loads over that
    for _ in 0..96 {
        if fixed >= 0 {
            out(fixed);
            out(0);
            out(48);
        } else {
            if abs(u - quarters) <= half_width { out(shade); } else { out(shade - 32 * dark); }
            out(u);
            out(v);
            part += z;                 // u steps by z / 120 a pixel: the remainder carries into it
            while part >= 120 {
                part -= 120;
                u += 1;
            }
        }
    }
}
