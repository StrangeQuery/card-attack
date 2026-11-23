//! Bloom's blur, per line of the 192 x 96 blocks per pass per channel per frame of the accelerated drive: a box of n
//! along it (its ends repeated). Deck: n, how many lines, then each: the n cards of the first box, how many places,
//! then for each place the card the box takes in after it and the one it lets go. Punches each place's box's sum /
//! n, rounded down (the sum carried as that and what it leaves, so it never passes a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    for _ in 0..inp() {
        let mut q = 0;
        let mut rest = 0;
        for _ in 0..n {
            rest += inp();
            while rest >= n { rest -= n; q += 1; }
        }
        for _ in 0..inp() {
            out(q);
            rest += inp();
            while rest >= n { rest -= n; q += 1; }
            rest -= inp();
            while rest < 0 { rest += n; q -= 1; }
        }
    }
}
