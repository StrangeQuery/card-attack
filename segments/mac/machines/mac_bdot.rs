//! Ball dots, once for the Wikipedia page's sphere: each row's pixels black where the light (mac_ball's) is under its
//! threshold, the 4x4 ordered dither at twice the size (each threshold two pixels wide, four round). Deck: the count;
//! each row: 1 if its first pixel is the second of its threshold's two, the four thresholds from the first pixel's on
//! (hundreds, rest; -99 hundreds for never), how many pixels, then each pixel's light (hundreds, rest). Punches each
//! pixel's dot (1 black, 0 white).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut second = inp();
        let hundreds = [inp(), inp(), inp(), inp()];
        let rests = [inp(), inp(), inp(), inp()];
        let n = inp();
        let mut column = 0;
        for _ in 0..n {
            let h = inp();
            let r = inp();
            let t = hundreds[column];
            if h < t || (h == t && r < rests[column]) { out(1); } else { out(0); }
            second += 1;
            if second == 2 {
                second = 0;
                column += 1;
                if column == 4 { column = 0; }
            }
        }
    }
}
