//! Face spans, once per row per frame of the shaded cube. Two phases, each with its own cards. First: y, then the edge rasterizer's output (for each of the 12 edges: its top row, the number of rows
//! n, then n spans); keeps the span of each edge that falls on row y. Second: for each of the 6 faces, its
//! brightness (0 = hidden) and its 4 edges. A face's span on this row runs from the leftmost of its edges' spans to
//! the rightmost. Punches (left, right) per face; empty (900, -1) if hidden.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    // each edge's span on this row; an empty span has left > right
    let mut left = [900i16; 12];
    let mut right = [-1i16; 12];
    let y = inp();
    for edge in 0..12 {
        let wanted = y - inp();         // this row is span number y - top of the edge
        let spans = inp();
        for span in 0..spans {
            let a = inp();
            let b = inp();
            if span == wanted { left[edge] = a; right[edge] = b; }
        }
    }
    overlay();                          // phase 2: the faces
    for _ in 0..6 {                     // the faces
        let brightness = inp();
        let mut face_left = 900;
        let mut face_right = -1;
        for _ in 0..4 {                 // its four edges: the leftmost left, the rightmost right
            let edge = inp();
            // SAFETY: the deck's edge numbers are 0..11
            unsafe {
                assume(edge >= 0);
                assume(edge < 12);
            }
            if left[edge as usize] < face_left { face_left = left[edge as usize]; }
            if right[edge as usize] >= face_right { face_right = right[edge as usize]; }
        }
        if brightness >= 1 { out(face_left); out(face_right); } else { out(900); out(-1); }   // hidden: empty
    }
}
