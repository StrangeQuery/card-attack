//! Edge rows, for the swarm's polygons: how many rows an edge meets, its height less the first row's distance below
//! its top, in hundreds, rounded up. Deck: the count, then each edge's height (a long, hundredths of a pixel, under
//! 100000) and the first row's middle below its top (0..99). Punches each count.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut hh = inp();
        let mut hl = inp();
        hl -= inp();
        if hl < 0 { hl += 999; hl += 1; hh -= 1; }
        hl -= 999 - 99;
        if hl > 0 { hl -= 1; hh += 1; } else { hl += 999; }
        // SAFETY: a long's rest is 0..999
        unsafe { assume(hl >= 0 && hl <= 999); }
        out(hh * 10 + digit(hl, 0));
    }
}
