//! Post boxes, once per lamp post drawn per frame of the accelerated drive. Deck: how many, then each: its middle
//! across (the pixel and 16ths), quarters of a pixel a unit there (sc), its top's row and its foot's row. Punches each
//! its half width hw (16ths: sc / 2, at least 10); the columns from x0 = x - 1 + (f - hw) / 16 to x1 = x + 2 + (f +
//! hw) / 16 (rounded down; kept to 0-768); the rows from the top's (at least 0) to the foot's and one (at most 384).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let f = inp();
        let mut hw = inp() / 2;
        if hw < 10 { hw = 10; }
        out(hw);
        let mut left = x - 1;
        let mut part = f - hw;
        while part < 0 { part += 16; left -= 1; }
        out(if left < 0 { 0 } else { left });
        let mut right = x + 2;
        part = f + hw;
        while part >= 16 { part -= 16; right += 1; }
        out(if right > 768 { 768 } else { right });
        let top = inp();
        out(if top < 0 { 0 } else { top });
        let foot = inp() + 1;
        out(if foot > 384 { 384 } else { foot });
    }
}
