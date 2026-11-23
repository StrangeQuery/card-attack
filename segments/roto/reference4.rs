// Rotozoomer reference at twice the resolution, 192x96 over the same view: each pixel's texture point straight
// from the formula, with the step per pixel halved, modulo the 9x9 tile.
mod roto_data;
use roto_data::*;
use std::io::Write;
fn main() {
    let mut out = std::io::stdout().lock();
    for f in 0..192i32 {
        let (a, b, cu, cv) = ((f % 48) as usize, ((f * 2 / 3) % 48) as usize, 450, 450);
        let (s, c, z) = (SIN48[a], SIN48[(a + 12) % 48], 21 + (8 * SIN48[b]) / 10);
        let (du, dv) = ((z * c) / 20, (z * s) / 20);
        for y in 0..96 {
            for x in 0..192 {
                let u = (cu + (x - 96) * du - (y - 48) * dv).rem_euclid(900);
                let v = (cv + (x - 96) * dv + (y - 48) * du).rem_euclid(900);
                let code = (TEX[(v / 100) as usize].as_bytes()[(u / 100) as usize] - b'0') as usize;
                out.write_all(&PALETTE[3 * code..3 * code + 3]).unwrap();
            }
        }
    }
}
