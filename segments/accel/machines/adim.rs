//! Dim, once, the drive's last software frame frozen under the VIDEO OPTIONS menu. Deck: how many, then each a
//! channel's level (0-255). Punches each times 0.55, rounded down: 11 v / 20, as 11 a + 11 b / 20 for v = 20 a + b
//! (11 v would pass a card).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let level = inp();
        let twenties = level / 20;
        let rest = level - 20 * twenties;
        out(11 * twenties + 11 * rest / 20);
    }
}
