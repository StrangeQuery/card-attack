//! Post heads, once per lamp post drawn per frame of the accelerated drive: its head 0.8 units out over the street.
//! Deck: how many, then each: its middle across (pixel, 16ths), sc (quarters of a pixel a unit), the side (-1 left,
//! 1 right). Punches each the head across, x - side (3 sc + sc / 5) 16ths (pixel, 16ths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let mut f = inp();
        let sc = inp();
        let side = inp();
        // 3 sc + sc / 5 16ths, carried a pixel at a time, out toward the street (against the side)
        let mut reach = 0;
        let mut part = 0;
        for k in 0..4 {
            part += if k < 3 { sc } else { sc / 5 };
            while part >= 16 { part -= 16; reach += 1; }
        }
        let mut head = x;
        if side < 0 { head += reach; f += part; } else { head -= reach; f -= part; }
        while f >= 16 { f -= 16; head += 1; }
        while f < 0 { f += 16; head -= 1; }
        out(head);
        out(f);
    }
}
