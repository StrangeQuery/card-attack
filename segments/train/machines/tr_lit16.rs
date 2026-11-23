//! Shadow taps, for the train: of a place's 16 depths in the sun's map (longs), how many are nearer than the place's
//! (a long); the sun's share, 100 less 100/16 for each, rounded down. Deck: the count, then items: the need, the 16
//! depths. Punches each share.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let n_hi = inp();
        let n_lo = inp();
        let mut s = 3;
        for _ in 0..16 {
            let d_hi = inp();
            let d_lo = inp();
            if d_hi < n_hi || (d_hi == n_hi && d_lo < n_lo) { s += 25; }
        }
        out(100 - s / 2 / 2);
    }
}
