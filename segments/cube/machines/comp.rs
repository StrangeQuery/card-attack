//! Row compositor, once per row per frame of the intro. Two phases, each with its own cards. First: y, the cube's fizzle level, then the edge rasterizer's output (for each of the 12 edges: its top
//! row, the number of rows n, then n spans); it keeps the span of each edge that falls on row y. Second: the
//! typesetter's 96 text bits for this row; punches one code per pixel: 1 for text (it wins), 2 where an edge's span
//! covers the pixel and the cube's fizzle shows it, 0 for plasma. Each pixel's dither threshold steps by 7 mod 16.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    // each edge's span on this row; an empty span has left > right
    let mut left = [900i16; 12];
    let mut right = [-1i16; 12];
    let y = inp();
    let fizzle = inp();
    let mut threshold = remainder(y * 3, 16);   // the dither threshold starts at 3y mod 16
    for edge in 0..12 {
        let wanted = y - inp();         // this row is span number y - top of the edge
        let spans = inp();
        for span in 0..spans {
            let a = inp();
            let b = inp();
            if span == wanted { left[edge] = a; right[edge] = b; }
        }
    }
    overlay();                          // phase 2: the pixels
    for x in 0..96 {
        threshold += 7;
        if threshold >= 16 { threshold -= 16; }
        if inp() >= 1 {
            out(1);                     // text wins
        } else {
            let mut code = 0;           // plasma
            for edge in 0..12 {
                if x >= left[edge] && right[edge] >= x {   // the first edge whose span covers x
                    if threshold < fizzle { code = 2; }
                    break;
                }
            }
            out(code);
        }
    }
}
