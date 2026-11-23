//! Fine, once per sample column per frame of the accelerated drive (twice: the street's middle and its grade):
//! the part of the way from an entry of the plan to the next. Deck: how many, then each: the entry and the next
//! (eighths), how far between (16ths of a quarter, 0-127). Punches each (next - entry) part / 16, 64ths, rounded down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let entry = inp();
        let next = inp();
        let part = inp();
        // part added |next - entry| times (the product would pass a card past 7), halved four times
        let mut sum = 0;
        let mut i = abs(next - entry);
        while i > 0 { sum += part; i -= 1; }
        let mut lean = sum / 2 / 2 / 2 / 2;
        if next < entry { if 16 * lean < sum { lean += 1; } lean = -lean; }
        out(lean);
    }
}
