//! Spill, once per screen row per frame of the dive while the portal shows. Deck: how many pixels, then for each how
//! far it is from the middle (tenths of a unit). Outside the ring the portal's light spills, 0.35 exp(-(r - 6) / 4),
//! from a table a unit at a time and between its steps by a straight line (to nothing past the last, 11 units out).
//! Punches, per pixel, the spill in tenths of 32nds (0..110), for the spill mix machine.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let spill = tables::PORTAL_SPILL;
    let count = inp();
    for _ in 0..count {
        let far = inp() - 60;
        let mut tenths = 0;
        if far >= 0 && far <= 109 {
            let unit = far / 10;
            // SAFETY: far is 0..109
            unsafe { assume(unit >= 0 && unit < 11) }
            let at = spill[unit as usize];
            let mut step = -at;                 // to nothing past the last unit
            if unit < 10 {
                // SAFETY: tested just above
                unsafe { assume(unit < 10) }
                step += spill[unit as usize + 1];
            }
            tenths = 10 * at;
            for _ in 0..far - 10 * unit { tenths += step; }
        }
        out(tenths);
    }
}
