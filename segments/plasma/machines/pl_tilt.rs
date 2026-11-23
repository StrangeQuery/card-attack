//! The plasma's tilt, once a frame: how far through the tilt back the frame is, in thousandths, for the
//! printed table of the eased angle's 600 cos and 100 sin (data/tilt.txt). Deck: the frame, the tilt's first frame,
//! its length less one. Punches the thousandths (0..999, by long division: k / n a digit at a time) and 1 once the
//! tilt is done (the table's last line), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let f = inp();
    let start = inp();
    let n = inp();
    let k = f - start;
    if k <= 0 {
        out(0);
        out(0);
    } else if k >= n {
        out(0);
        out(1);
    } else {
        let mut r = k;
        let mut u = 0;
        for _ in 0..3 {
            r *= 10;
            let mut d = 0;
            while r >= n {
                r -= n;
                d += 1;
            }
            u = u * 10 + d;
        }
        out(u);
        out(0);
    }
}
