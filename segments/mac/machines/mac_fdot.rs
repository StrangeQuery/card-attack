//! Dots, once per frame of Boom: each row of the floor and the ceiling, each of its 448 texels' greys black or white,
//! black where the grey is at most its column's threshold (the 4x4 ordered dither's for the row's light, four columns
//! round). Deck: the count; each row: the thresholds of columns 0-3, then the 448 greys. Punches each pixel's dot (1
//! black, 0 white).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let threshold = [inp(), inp(), inp(), inp()];
        let mut phase = 0;
        for _ in 0..448 {
            if inp() <= threshold[phase] { out(1); } else { out(0); }
            phase += 1;
            if phase == 4 { phase = 0; }
        }
    }
}
