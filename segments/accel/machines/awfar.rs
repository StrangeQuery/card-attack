//! Wall distance, once per wall a column's ray meets per frame of the accelerated drive. Deck: how many, then each Q's
//! whole and 44 times its part (amul's). Punches each the wall's distance there, 44 Q (quarters, at most 999), and its
//! haze ((a - 224) 4 / 47, 0-32).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let q = inp();
        let part = inp();
        let mut far = 999;
        if q <= 22 { far = 44 * q + part; if far > 999 { far = 999; } }
        out(far);
        // (a - 224) 4 / 47: four additions, 47 taken away while it fits
        let mut haze = 0;
        let mut sum = 0;
        if far > 224 {
            for _ in 0..4 {
                sum += far - 224;
                while sum >= 47 { sum -= 47; haze += 1; }
            }
        }
        out(if haze > 32 { 32 } else { haze });
    }
}
