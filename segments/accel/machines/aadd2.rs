//! Places, as many as the host has, per frame of the accelerated drive: a place (a pixel and 16ths) moved by another
//! and some 16ths more, or back by them. Deck: how many, then each: the place, the move (pixel and 16ths), 16ths more,
//! 1 to add or -1 to take away. Punches each the place, a pixel and 16ths (0-15); 999 and 0 when an added move
//! takes it past 990 pixels (a card holds the place).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let mut p = inp();
        let mut f = inp();
        let mut q = inp();
        let mut g = inp() + inp();
        if inp() < 0 { q = -q; g = -g; }
        // (a card holds the place: past 990 pixels, 999 and 0)
        if q >= 0 && p > 990 - q { out(999); out(0); continue; }
        p += q;
        f += g;
        while f >= 16 { f -= 16; p += 1; }
        while f < 0 { f += 16; p -= 1; }
        out(p);
        out(f);
    }
}
