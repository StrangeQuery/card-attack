//! Row of the floor, once per tile row of the Mode X ray tracer (320x240, rows of four 96-pixel tiles). Deck: the
//! frame's row y (0..239), the tile's place across, and the ball's shadow: the ball's x (floor units), the shadow's
//! radius and darkness (32nds). The floor is the plasma's plane tilted right back, at 5 / 3 of 192x96: the horizon
//! is row 100, the camera 200 pixels from the screen. Rows to 100 see sky (the band of their row: to 59, to 83, to
//! 100), rows to 119 haze; a row below meets the floor at v = 120 (200 - y) / (y - 100) (rounded down, under 0 below
//! row 200), at depth z = 120 + v, with fog (z - 240) / 7 (all haze from 32 on). Across it u = (x - 160) z / 200,
//! stepped by adding z to a remainder. Punches for each of the 96 pixels its look (a sky or haze code, or 100 + the
//! shadow's darkness times 32 if it is in the shadow, plus the fog), u and v (0 and 48 where there is no plasma).
//! In the shadow: |u - 15 x / 4| at most the shadow's half width in this row, the largest h with h^2 + (v - 105)^2
//! under radius^2. In six phases, to fit: the first reads the five cards and finds v; then z and the fog; where u starts;
//! the shadow's centre; its half width; the row.
// cardiac: phase-cards=5,0,0,0,0
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let y = inp();
    let tile_x = inp();
    let ball_x = inp();
    let radius = inp();
    let dark = inp();
    let mut fixed = 3;                 // a code if the row shows no plasma: haze
    let mut v = 48;
    if y <= 100 {
        fixed = if y < 60 { 0 } else if y < 84 { 1 } else { 2 };   // the sky's bands by the row
    }
    // 120 (200 - y) / (y - 100) without the product: 200 - y added 120 times, y - 100 taken away whenever it fits
    // (or added back below 0, rounding down); none of it above row 120
    let below = y - 100;
    let mut times = 0;
    if y >= 120 {
        times = 120;
        v = 0;
    }
    let mut part = 0;
    for _ in 0..times {
        part += 200 - y;
        while part >= below {
            part -= below;
            v += 1;
        }
        while part < 0 {
            part += below;
            v -= 1;
        }
    }
    overlay();                         // v is found; the depth and fog load over that
    let z = 120 + v;
    let mut fog = 0;
    if y >= 120 {
        let mut far = z - 240;
        while far >= 7 && fog < 32 {   // the fog, (z - 240) / 7, by taking away 7s
            far -= 7;
            fog += 1;
        }
        if fog < 32 { fixed = -1; }   // the plasma, not all haze
    }
    overlay();                         // the depth and fog are found; the start across loads over that
    // u at the tile's first pixel, (tile_x - 160) z / 200 rounded down, and its remainder: z added |tile_x - 160|
    // times, the 200s carried
    let offset = tile_x - 160;
    let mut u = 0;
    let mut part = 0;
    for _ in 0..abs(offset) {
        part += z;
        while part >= 200 {
            part -= 200;
            u += 1;
        }
    }
    if offset < 0 {
        u = -u;
        if part > 0 {                  // rounded down, below 0
            u -= 1;
            part = 200 - part;
        }
    }
    overlay();                         // the start across is found; the shadow's centre loads over that
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
            part += z;                 // u steps by z / 200 a pixel: the remainder carries into it
            while part >= 200 {
                part -= 200;
                u += 1;
            }
        }
    }
}
