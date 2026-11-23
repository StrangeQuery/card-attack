//! Street covers, once per row of the street per frame of the accelerated drive, for the pixels near a line's edge:
//! how much of the pixel's width lies past an edge. Deck: how many, then each: how far past the edge it is (640ths,
//! asedge's) and the pixel's width (640ths). Punches each c = d 32 / w + 16, the quotient toward zero, kept to 0-32
//! (straight away 0 or 32 for |d| at least half the width and 2 more).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let d = inp();
        let w = inp();
        let reach = w / 2 + 2;
        let mut q = 32;
        if d <= -reach { q = 0; } else if d < reach {
            // |d| 32 / w by adding: |d| added 32 times, w taken away whenever the sum reaches it
            let size = abs(d);
            let mut part = 0;
            let mut sum = -w;
            for _ in 0..32 {
                sum += size;
                while sum >= 0 { sum -= w; part += 1; }
            }
            q = if d < 0 { 16 - part } else { 16 + part };
        }
        // (kept to 0-32: a narrow pixel, the software half's 1, goes past it)
        out(if q > 32 { 32 } else if q < 0 { 0 } else { q });
    }
}
