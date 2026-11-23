//! Sum, for the dive's ending at 1536x768: what falls on each 4 x 4 pixels from the blobs or lines round it. Deck:
//! the most it can be, how many; then for each how many parts and the parts (none below 0). Punches their sum, kept
//! to the most as it is added up.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let most = inp();
    for _ in 0..inp() {
        let mut sum = 0;
        for _ in 0..inp() {
            sum += inp();
            if sum > most { sum = most; }
        }
        out(sum);
    }
}
