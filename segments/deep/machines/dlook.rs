//! Look up, per frame of the dive (the portal's radius and angle, the jellyfish's halo). Deck: a table's size
//! (up to 50) and its numbers, how many, then indices. Punches, per index, its number in the table.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let size = inp();
    let mut table = [0i16; 50];
    for i in 0..size { table[i as usize] = inp(); }
    let count = inp();
    for _ in 0..count {
        let i = inp();
        // SAFETY: the host deals indices within the table
        unsafe { assume(i >= 0 && i < 50) };
        out(table[i as usize]);
    }
}
