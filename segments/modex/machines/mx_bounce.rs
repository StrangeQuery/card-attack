//! Bounce, once per ball pixel of the Mode X ray tracer (320x240). Deck: the bounced ray's X and Y (out of 529), the
//! pixel's B (pixels up from the ball's centre), the centre's height over the floor. A ray up and to the left
//! (X at most -189, Y at least 243) catches the light (7); any other upward ray sees sky, high (4, Y over 270) or
//! low (5). A downward ray falls from height h = the centre's + B and meets the floor after t = 100 h / -Y
//! hundredths of its length (529 pixels); from 25 on the floor is lost in haze (6). Punches a pixel's look and a
//! point (0, 48) for a fixed colour, or -1, t and 0 for the hit machine.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let ray_x = inp();
    let ray_y = inp();
    let b = inp();
    let centre = inp();
    let mut look = 5;                  // the sky, low
    let mut t = 0;
    if ray_y >= 243 && ray_x <= -189 {
        look = 7;                      // up and to the left: the light
    } else if ray_y > 270 {
        look = 4;                      // the sky, high
    } else if ray_y < 0 {
        // 100 h / -Y without the product: h added a hundred times, -Y taken away whenever it fits, until t reaches
        // the haze
        let height = centre + b;
        let mut part = 0;
        for _ in 0..100 {
            part += height;
            while part + ray_y >= 0 {
                part += ray_y;
                t += 1;
            }
            if t >= 25 { break; }
        }
        look = if t >= 25 { 6 } else { -1 };   // too far: haze; or the floor, whose point the hit machine finds
    }
    out(look);
    if look < 0 {
        out(t);
        out(0);
    } else {
        out(0);
        out(48);
    }
}
