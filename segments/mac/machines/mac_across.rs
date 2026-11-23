//! Across, once per frame of Boom: each crossing of a column's ray (mac_cross's) as the map's cell the other way and the
//! texel it crosses at: the camera's place that way (cell, texel) and how far the ray has gone, forward or back (going
//! back, a texel less unless exactly on one). Deck: the count of lists; each: the camera's cell and texel the other
//! way, 1 if the ray goes back that way, how many cells the map has that way; then mac_cross's punches for it (each
//! crossing's cell, how far across, the remainder's flag; -1). Punches each crossing inside the map: the cell, the cell
//! the other way, the texel, how far across; then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let pcell = inp();
        let ptex = inp();
        let back = inp();
        let cells = inp();
        let mut inside = 1;
        loop {
            let cell = inp();
            if cell < 0 { break }
            let across = inp();
            let short = inp();
            // how far across in cells and texels, by taking 64 away while it fits
            let mut tcell = 0;
            let mut ttex = across;
            while ttex >= 64 { ttex -= 64; tcell += 1; }
            let mut ocell;
            let mut tex;
            if back == 0 {
                tex = ptex + ttex;
                ocell = pcell + tcell;
                if tex >= 64 { tex -= 64; ocell += 1; }
            } else {
                tex = ptex - ttex - short;
                ocell = pcell - tcell;
                if tex < 0 { tex += 64; ocell -= 1; }
            }
            if ocell < 0 || ocell >= cells { inside = 0; }
            if inside == 1 {
                out(cell);
                out(ocell);
                out(tex);
                out(across);
            }
        }
        out(-1);
    }
}
