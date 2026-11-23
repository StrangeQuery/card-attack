//! Solid compositor, once per row per frame of the shaded cube. Two phases, each with its own cards. First: y, the faces' fizzle level, then the 6 faces' spans on this row (left, right).
//! Second: the wireframe compositor's 96 codes for this row. Where a face covers the pixel and the faces' fizzle
//! shows it, punches 3 + the face's number (the first face wins); else passes the code on. Same dither as the
//! wireframe: the threshold starts at 3y mod 16 and steps by 7 mod 16 every pixel.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut left = [0i16; 6];           // each face's span on this row
    let mut right = [0i16; 6];
    let y = inp();
    let fizzle = inp();
    // SAFETY: the deck's row is a screen row, 0..47
    unsafe { assume(y >= 0) };
    let mut threshold = remainder(y * 3, 16);   // the dither threshold starts at 3y mod 16
    for face in 0..6 {
        left[face] = inp();
        right[face] = inp();
    }
    overlay();                          // phase 2: the pixels
    for x in 0..96 {
        threshold += 7;
        if threshold >= 16 { threshold -= 16; }
        let mut code = inp();           // the wireframe compositor's code, unless a face covers x
        if threshold < fizzle {
            for face in 0..6 {
                if x >= left[face] && right[face] >= x { code = face as i16 + 3; break; }   // the first face wins
            }
        }
        out(code);
    }
}
