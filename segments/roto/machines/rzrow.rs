//! Rotozoomer row, once per screen row per frame. Its program deck carries the texture, the 9x9 heart, in `tex`:
//! three texels (digits) of a row to a cell, column by column (the 9 rows' first three texels, then the next three),
//! punched in by the host at the label resource_tex. Deck: this row's start (u, v) and step (du, dv). For each of the 96
//! pixels the texel is (u / 100, v / 100): its cell is 9 (u / 100 / 3) + v / 100 and its digit (u / 100) mod 3.
//! Punches the digit (a colour code), then steps (u, v) by (du, dv), modulo 900.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The texture's size in hundredths of a texel: (u, v) are kept in 0..TILE.
const TILE: i16 = 900;

pub fn main() {
    let tex: [i16; 27] = load_resource("tex");  // the texture: the host punches it in here
    let mut u = inp();                  // where on the texture, in hundredths of a texel
    let mut v = inp();
    let du = inp();                     // the step per pixel
    let dv = inp();
    for _pixel in 0..96 {
        let mut cell = v / 100;         // the texture row ...
        let mut texel = u / 100;        // ... and column
        while texel >= 3 { texel -= 3; cell += 9; }    // three texels to a cell, a column of 9
        // SAFETY: u and v are kept in 0..TILE, so the cell is 0..26
        unsafe {
            assume(cell >= 0);
            assume(cell < 27);
        }
        out(digit(tex[cell as usize], texel));
        u = wrap(u + du, TILE);         // from one tile either side
        v = wrap(v + dv, TILE);
    }
}

