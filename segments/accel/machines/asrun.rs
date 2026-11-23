//! Street row runs, the software half's, once per fourth row of the street per frame of the drive, for each of
//! asrow's numbers: the blocks' middles along each span. Deck: asrow4's 24 spans. Punches for each span's 8 blocks p
//! plus or less the start and a step more each block (the whole; the 64ths carried).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..24 {
        let p = inp();
        let back = inp() < 0;
        let mut whole = inp();
        let mut part = inp();
        let step = inp();
        let half = inp();
        for _ in 0..8 {
            out(if back { p - whole } else { p + whole });
            whole += step;
            part += half;
            if part >= 64 { part -= 64; whole += 1; }
        }
    }
}
