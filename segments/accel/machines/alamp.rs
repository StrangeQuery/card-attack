//! Lamp lights, once per lamp post drawn per frame of the accelerated drive: how strong its light is. Deck: how many,
//! then each: the distance at its head's pixel (-1 off the screen), the post's (quarters), its haze (32nds), sc
//! (quarters of a pixel a unit). Punches each its strength, 500ths: (32 - haze) min(500, 125 + 3 sc + sc / 8) / 32,
//! rounded down; 0 where something nearer (by more than 2 quarters) hides the head or it is off the screen.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let there = inp();
        let near = inp();
        let clear = 32 - inp();
        let sc = inp();
        let mut s = 0;
        if there >= near - 2 {
            // 125 + 3 sc + sc / 8, at most 500, carried as its 32nds of clear
            let mut level = 125 + sc / 2 / 2 / 2;
            for _ in 0..3 { if level < 500 { level += sc; } }
            if level > 500 { level = 500; }
            let mut sum = 0;
            for _ in 0..clear { sum += level; while sum >= 32 { sum -= 32; s += 1; } }
        }
        out(s);
    }
}
