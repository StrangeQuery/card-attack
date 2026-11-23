//! Where the camera is, once per frame of the drive. Deck: the frame f. The camera goes 9 quarters of a unit a
//! frame, so it is 9 f quarters along the street. Punches the road table's entry under it (9 f / 8, a card every
//! 2 units), how far into that entry (9 f mod 8, quarters) and 8 times that; then how far into its block (9 f mod
//! 120; a block every 30 units), less that, and 104 less that (where the block's near and far ends are); then 15
//! and 8 times the block's number (where it starts in the road table and in the blocks' table).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let frame = inp();
    // 9 f passes a card, so it is counted up a frame at a time, in entries and blocks with their remainders
    let mut entry = 0;
    let mut into_entry = 0;
    let mut road_at = 0;
    let mut blocks_at = 0;
    let mut into_block = 0;
    for _ in 0..frame {
        into_entry += 1;                // 9 = 8 + 1: one entry and a quarter more
        entry += 1;
        if into_entry == 8 { into_entry = 0; entry += 1; }
        into_block += 9;
        if into_block >= 120 { into_block -= 120; road_at += 15; blocks_at += 8; }
    }
    out(entry);
    out(into_entry);
    out(8 * into_entry);
    out(into_block);
    out(-into_block);
    out(104 - into_block);
    out(road_at);
    out(blocks_at);
}
