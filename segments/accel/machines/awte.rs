//! Wall place across an end wall, once per end wall a column's ray meets per frame of the accelerated drive. Deck: how
//! many, then each: the ray's place across at the block's start (quarters) and tenths over (amul's), the face's
//! |distance| across (eighths). Punches each the texel across from the face and tenths from its middle (at least 0).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut texel = inp();
        let mut tenths = inp() - 5;
        let e = inp();
        let half = e / 2;
        texel -= half;
        if e > half + half { tenths -= 5; }
        while tenths < 0 { tenths += 10; texel -= 1; }
        if texel < 0 { texel = 0; tenths = 0; }
        out(texel);
        out(tenths);
    }
}
