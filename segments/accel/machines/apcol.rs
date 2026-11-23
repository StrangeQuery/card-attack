//! Post columns, once per lamp post drawn per frame of the accelerated drive: how much of each column the post
//! covers. Deck: how many posts, then each: its middle across (pixel, 16ths), its half width hw (16ths), its first
//! column and the one past its last. Punches for each column the cover, 2 (hw + 8 - |d|) 32nds (0-32), d = 16 x - its
//! middle, and d.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let x = inp();
        let f = inp();
        let hw = inp();
        let first = inp();
        let last = inp();
        for column in first..last {
            let d = 16 * (column - x) - f;
            let size = abs(d);
            let mut cover = 2 * (hw + 8 - size);
            if cover < 0 { cover = 0; }
            if cover > 32 { cover = 32; }
            out(cover);
            out(d);
        }
    }
}
