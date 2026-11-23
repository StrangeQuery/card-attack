//! Rows and pixels, for the train: fl(y + 49, 100) of a long y in hundredths of a pixel (the first row, or pixel,
//! whose middle at 100 k + 50 is at or past y). Deck: the count, then the longs. Punches each as a card.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let hi = inp();
        let lo = inp();
        // (hi 1000 + lo + 49) / 100 = 10 hi + (lo + 49) / 100, lo + 49 never below 0
        let s = lo - 951;
        out(if s >= 0 { hi * 10 + 10 } else { hi * 10 + (lo + 49) / 100 });
    }
}
