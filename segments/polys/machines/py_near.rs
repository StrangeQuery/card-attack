//! Near, once per frame of the polys: the shadow map, the nearest of the blobs' pieces to the light on each place of
//! the map. Deck: for each place, how many pieces cover it, then each one's depth and blob (1 or 2) (sorted to their
//! places by the host, as a card sorter would); then 999. Punches for each place the smallest depth and its blob (the
//! first of equals), or 999 and 0 if none.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let pieces = inp();
        if pieces == 999 { break; }
        let mut nearest = 999;
        let mut blob = 0;
        for _ in 0..pieces {
            let depth = inp();
            let which = inp();
            if depth < nearest { nearest = depth; blob = which; }
        }
        out(nearest);
        out(blob);
    }
}
