//! Ground rows, once per sample column per frame of the accelerated drive: which step of the march each screen row
//! sees the street before. Deck: how many steps, then each its row and 16ths (amrow's). Punches for each row from the
//! bottom (383) up the step (-1: none). A row sees the street at the first step at or above it that rises above all
//! those before (the nearest ground hides what is beyond).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// Whether a place d rows and f 16ths (-15 to 15) below another is at or below it: 16 d >= f.
#[inline(never)]
fn below(d: i16, f: i16) -> bool {
    d > 0 || (d == 0 && f <= 0)
}

pub fn main() {
    let steps = inp();
    let mut y = 383;
    let mut low_row = 999;
    let mut low_frac = 0;
    for k in 0..steps {
        let row = inp();
        let frac = inp();
        while y >= 0 && below(y - row, frac) && !below(y - low_row, low_frac) {
            out(k);
            y -= 1;
        }
        if !below(row - low_row, low_frac - frac) { low_row = row; low_frac = frac; }
    }
    while y >= 0 { out(-1); y -= 1; }
}
