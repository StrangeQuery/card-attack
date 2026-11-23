//! First, once per frame of Boom: each column's ray's crossings one way (mac_across's, the map's cells looked up), in
//! order, up to the first wall: the door (one cell: a ray enters it at most once a way) while it is part way up, and
//! the wall (shut, the door is a wall; all the way up, it isn't there). Deck: the count of lists; each: the door (0
//! shut, 1 part way, 2 up), how far to the first line, how many crossings, and each one's cell, how far it had gone
//! across, the texel crossed at. Punches for the door, if met first: 1, how far across, how far along, the texel; for
//! the wall: 2, its texture, how far across, how far along, the texel; 0 if there is none.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let door = inp();
        // how far along each crossing is, counted on before it (so it never passes the last one's)
        let mut along = inp() - 64;
        let n = inp();
        let mut found = 0;
        for _ in 0..n {
            let cell = inp();
            let across = inp();
            let tex = inp();
            along += 64;
            // the door all the way up is an empty cell
            if found == 0 && cell != 0 && (cell != 5 || door != 2) {
                if cell != 5 || door == 0 {
                    out(2);
                    out(cell);
                    found = 1;
                } else {
                    out(1);
                }
                out(across);
                out(along);
                out(tex);
            }
        }
        if found == 0 { out(0); }
    }
}
