//! Rings, once per tile row of the plasma at 192x96, the first of its four waves. Deck: where the rings and the
//! colours start this frame (from the clock), then for each of the 96 pixels its distance from the rings' centre.
//! Punches for each the rings' wave at the distance plus its start (32 + 32 sin of its step out of 64, 0..64, from
//! a quarter of a turn in a table) plus the colours' start: the sum the other waves add to.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let rings = inp();
    let colours = inp();
    for _ in 0..96 {
        let mut k = inp() + rings;     // 3..555
        while k >= 64 { k -= 64; }
        let mut half = 32;
        if k >= 32 {                   // the second half turn is the first upside down
            k -= 32;
            half = -32;
        }
        if k > 16 { k = 32 - k; }      // and each half is a quarter there and back
        // SAFETY: k is brought into 0..63, then 0..31, then 0..16 above
        unsafe { assume(k >= 0 && k <= 16); }
        let quarter = tables::PL_WAVE64[k as usize];
        if half > 0 { out(colours + 32 + quarter); } else { out(colours + 32 - quarter); }
    }
}
