//! Mix, once per frame of the polys: the normals each triangle carries across itself, half each corner's own and
//! half the triangle's, in thousandths; and the count of triangles. Deck: for each triangle, for each part of the
//! normal (x, y, z) and each corner, the corner's (in hundredths) and the triangle's mean (from the dot machine);
//! then 999. Punches 5 (corner + mean) for each; at the end, the triangles counted, in thousands and the rest.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut thousands = 0;
    let mut count = 0;
    loop {
        let mut corner = inp();
        if corner == 999 { break; }
        for pair in 0..9 {
            if pair > 0 { corner = inp(); }
            out(5 * (corner + inp()));
        }
        if count == 999 { count = 0; thousands += 1; } else { count += 1; }
    }
    out(thousands);
    out(count);
}
