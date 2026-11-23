//! Bulge, once per frame of the polys: how far each corner of the blob is pushed in or out. Deck: the count, then
//! for each corner p (99 sin A times 99 cos B, in hundredths) and 99 sin C. The push, in thousandths, is 0.22 sin A
//! cos B + 0.08 sin C: 2.2 p + 0.8 (99 sin C), each part cut toward zero. Punches its tenths and its units.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let p = inp();
        let push = 2 * p + 2 * p / 10 + 8 * inp() / 10;
        out(push / 10);
        out(push % 10);
    }
}
