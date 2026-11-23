//! Longs as three cards, for the swarm: a long (thousands, rest 0..999) as millions, thousands and rest. Deck: the
//! count, then the longs. Punches each as three cards.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let h = inp();
        if h < 0 { out(-1); out(h + 999 + 1); } else { out(0); out(h); }
        out(inp());
    }
}
