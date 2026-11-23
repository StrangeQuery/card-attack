//! Place, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for each:
//! x as a whole unit (near, a slice before the hit; far, at it; within 300 either way), which column from it (-1..1;
//! 0 far) and the map row. Each column of rock is shifted a tone in the strata now and then and is a little lighter
//! or darker, by two tables of 16: its place in the second is the first's at its column (x + 100, mod 16) and the row
//! (mod 16) added (mod 16). Punches, per pixel: the place (0..15) and the second table's number there.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut column = inp() + inp() + 420;
        let mut row = inp();
        while column >= 160 { column -= 160; }
        while column >= 16 { column -= 16; }
        while row >= 16 { row -= 16; }
        // SAFETY: 0..15 now
        unsafe { assume(column >= 0 && column < 16) };
        let shuffled = tables::U_ROCK_PLACE[column as usize];
        let mut place = shuffled + row;
        if place >= 16 { place -= 16; }
        // SAFETY: 0..15
        unsafe { assume(place >= 0 && place < 16) };
        out(place);
        out(tables::U_ROCK_VARY[place as usize]);
    }
}
