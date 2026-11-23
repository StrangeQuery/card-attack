//! Slant, once per tile row of the plasma at 192x96. Deck: where the slanting wave starts this frame (from the
//! clock), then for each of the 96 pixels its u, v and the sum so far. Punches each sum with the slanting wave at
//! u - v plus its start added: 32 + 32 sin of its step out of 96, 0..64, from a quarter of a turn in a table.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let slanting = inp();
    for _ in 0..96 {
        let mut k = inp() + slanting;
        k -= inp();
        let sum = inp();
        while k >= 96 { k -= 96; }     // k = u - v + start, -750..630: into 0..95
        while k < 0 { k += 96; }
        let mut half = 32;
        if k >= 48 {                   // the second half turn is the first upside down
            k -= 48;
            half = -32;
        }
        if k > 24 { k = 48 - k; }      // and each half is a quarter there and back
        // SAFETY: k is brought into 0..95, then 0..47, then 0..24 above
        unsafe { assume(k >= 0 && k <= 24); }
        let quarter = tables::PL_WAVE96[k as usize];
        if half > 0 { out(sum + 32 + quarter); } else { out(sum + 32 - quarter); }
    }
}
