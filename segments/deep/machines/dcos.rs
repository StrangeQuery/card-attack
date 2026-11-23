//! Cosine, per jellyfish per frame of the dive (round its bell and its tentacles). Deck: how far the jellyfish is
//! turned (steps of a 64-step turn, 0..63), how many, then steps s (0..63). Punches, per step, the cosine and the
//! sine of s plus the turn, 100 cos and 100 sin, from a quarter-turn table.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let quarter = tables::QUARTER_COS64;
    let turn = inp();
    let count = inp();
    for _ in 0..count {
        let step = inp() + turn + 64;
        for k in 0..2 {
            // the cosine of step - 16 k, round the turn
            let mut i = step - 16 * k;
            while i >= 64 { i -= 64; }
            let mut below = false;
            if i > 32 { i = 64 - i; }
            if i > 16 { i = 32 - i; below = true; }
            // SAFETY: 0..16 now
            unsafe { assume(i >= 0 && i <= 16) };
            if below { out(-quarter[i as usize]); } else { out(quarter[i as usize]); }
        }
    }
}
