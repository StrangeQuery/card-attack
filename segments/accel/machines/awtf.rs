//! Wall place along a face, once per face a column's ray meets per frame of the accelerated drive. Deck: how many,
//! then each: the quarters ahead where the ray meets it and the half over (awx's), 5 times the division's leftover
//! over |across| (amul's), the block's start. Punches each the texel along the face (0-103) and tenths from its
//! middle (5 X - 10 start - 5 tenths, kept to 0-1030).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let met = inp();
        let mut tenths = 5 * inp() + inp() - 5;
        let mut texel = met - inp();
        if tenths < 0 { tenths += 10; texel -= 1; }
        if texel < 0 { texel = 0; tenths = 0; }
        if texel > 102 { texel = 103; tenths = 0; }
        out(texel);
        out(tenths);
    }
}
