//! Wall, once per frame of Boom: each column's wall, and the door over it, down the rows: the texel each row shows
//! (stepping 2 s 500ths a row, s the wall's scale) and its dot, black where the texel's grey is at most the row's
//! threshold (the 4x4 ordered dither's for the wall's light). Deck: the count; each: its first row, the texel there and
//! its 500ths, the step a row (texels, 500ths), the thresholds of four rows from the first, then the texture's column,
//! its 64 greys. Punches each row's dot (1 black, 0 white) down to the wall's foot (the texel passing 63) or the view's
//! (row 224); then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut y = inp();
        let mut texel = inp();
        let mut part = inp();
        let step = inp();
        let step_part = inp();
        let threshold = [inp(), inp(), inp(), inp()];
        let mut phase = 0;
        // the column's greys in order: each shown on the rows whose texel it is
        for read in 0..64 {
            let grey = inp();
            while texel == read && y < 224 {
                if grey <= threshold[phase] { out(1); } else { out(0); }
                phase += 1;
                if phase == 4 { phase = 0; }
                part += step_part;
                if part >= 500 { part -= 500; texel += 1; }
                texel += step;
                y += 1;
            }
        }
        out(-1);
    }
}
