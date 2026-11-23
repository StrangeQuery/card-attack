//! Rotor for one tile of the rotozoomer at twice the resolution (192x96, four 96x48 tiles), once per tile per
//! frame. Two phases. First: the angle a and the zoom phase b (0-47), the view centre (cu, cv), the tile's place in
//! the frame (tx 0 or 96 across, ty 0 or 48 down), then the 48-entry sine table (24 sin); keeps S = sin a,
//! C = cos a = sin(a + 12) and the zoom Z = 21 + 8 sin b / 10 (2-40). Second: one half-pixel step right moves the texture
//! point by (du, dv) = (Z C, Z S) / 20, one down by (-dv, du). Punches du, dv, then for each of the tile's
//! 48 rows the texture point of its left pixel, (u, v), kept modulo 900 (the 9x9 tile in hundredths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The 9x9 tile in hundredths of a texel: (u, v) are kept in 0..TILE.
const TILE: i16 = 900;

pub fn main() {
    let angle = inp();
    let zoom_phase = inp();
    let mut u = inp();                  // the view centre on the texture
    let mut v = inp();
    let tile_x = inp();                 // the tile's place in the frame: 0 or 96 across ...
    let tile_y = inp();                 // ... 0 or 48 down
    let mut cos_step = angle + 12;      // cos a = sin(a + 12), round the 48 steps
    if cos_step >= 48 { cos_step -= 48; }
    // read the 48-card sine table, keeping the three entries this frame needs
    let mut sin_a = 0;
    let mut cos_a = 0;
    let mut sin_b = 0;
    for i in 0..48 {
        let entry = inp();
        if i == angle { sin_a = entry; }
        if i == cos_step { cos_a = entry; }
        if i == zoom_phase { sin_b = entry; }
    }
    let zoom = sin_b * 8 / 10 + 21;
    // (zoom C, zoom S): C and S added zoom times (no multiply; zoom is 2..40)
    let mut zoom_cos = 0;
    let mut zoom_sin = 0;
    for _ in 0..zoom { zoom_cos += cos_a; zoom_sin += sin_a; }
    overlay();
    let du = zoom_cos / 20;             // toward zero
    let dv = zoom_sin / 20;
    out(du);
    out(dv);
    // the tile's top-left pixel: centre + (tx - 96) (du, dv) + (ty - 48) (-dv, du), walked in 48 steps of
    // -2 (du, dv) on the left and -(-dv, du) at the top; then every row's, one step (-dv, du) at a time
    let mut step_u = 0;
    let mut step_v = 0;
    if tile_y < 48 { step_u = dv; step_v = -du; }
    if tile_x < 96 { step_u -= du + du; step_v -= dv + dv; }
    for row in -48..48 {
        if row >= 0 {
            out(u);
            out(v);
            step_u = -dv;
            step_v = du;
        }
        u = wrap(u + step_u, TILE);     // from one tile either side
        v = wrap(v + step_v, TILE);
    }
}
