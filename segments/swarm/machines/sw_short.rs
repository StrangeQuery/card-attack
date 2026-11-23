//! Longs from three cards, for the swarm's arithmetic: a number given as millions, thousands and rest (the last two
//! 0..999) that fits a long, as its thousands (millions times 1000 plus thousands) and rest. Deck: the count, then the
//! numbers. Punches each as a long.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let millions = inp();
        let mut thousands = inp();
        // SAFETY: it fits a long, so its millions are -1 or 0
        unsafe { assume(millions >= -1 && millions <= 0); }
        if millions < 0 { thousands -= 999; thousands -= 1; }
        out(thousands);
        out(inp());
    }
}
