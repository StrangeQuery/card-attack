//! Walk, once per frame of the polys: a value carried along an edge or across a row. Deck: for each, the value at
//! its start and at its end, the first step's distance from the start, the whole distance, and how many steps
//! (four apart, at least one); then 999 (no value reaches it). Punches start + (end - start) k / distance at each
//! step k (cut toward zero), kept as a whole and a remainder: the change divided once, then added a unit of distance
//! at a time, straight to the value, each part with the change's sign.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let start = inp();
        if start >= 999 { break; }
        let mut size = inp() - start;
        let mut k = inp();
        let distance = inp();
        let mut steps = inp();
        // the change's size divided by the distance, by taking it away, counting toward the change's sign
        let mut one = 1;
        if size < 0 { size = -size; one = -1; }
        let mut whole = 0;
        while size >= distance { size -= distance; whole += one; }
        // what's left over, less a distance: added to the remainder at each unit, and a distance taken back unless
        // it carries one
        let short = size - distance;
        let mut value = start;
        let mut over = 0;
        loop {
            while k > 0 {
                k -= 1;
                value += whole;
                over += short;
                if over < 0 { over += distance; } else { value += one; }
            }
            out(value);
            k = 4;
            steps -= 1;
            if steps <= 0 { break; }
        }
    }
}
