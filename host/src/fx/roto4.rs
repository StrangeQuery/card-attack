//! Rotozoomer at twice the resolution: a 192x96 frame as four independent 96x48 tiles. Per tile per frame a rotor
//! machine (rotor4, told the tile's place) finds its rows' starts at a half-pixel step; then the usual row and paint
//! machines per row. The host stitches the tiles into the frame. The tumble lays its cube over these same tiles.
use crate::*;
use std::path::Path;

const HEART: [&str; 9] = ["000000000", "033000330", "311303113", "321111113", "311111113", "031111130", "003111300", "000313000", "000030000"];
/// Frames rendered: 144 is the view's period, more so the cut can take ten seconds of it after the tumble.
pub const FRAMES: usize = 192;

/// The rotozoomer's machines and cards.
pub struct Roto4 { rot: Machine, clock: Machine, rz: Machine, paint: Machine, palette: Vec<i32>, sin48: Vec<i32> }

impl Roto4 {
    pub fn new(demo: &Path) -> Self {
        let cells: Vec<i32> = HEART.iter().flat_map(|r| [0, 3, 6].map(|i| r[i..i + 3].parse::<i32>().unwrap())).collect();
        let cols: Vec<i32> = (0..27).map(|i| cells[3 * (i % 9) + i / 9]).collect();
        let rz = carrying(demo, "rzrow", "tex", &cols);
        Roto4 { rot: rust(demo, "rotor4"), clock: rust(demo, "r4_clock"), rz, paint: one(demo, "shared/original/paint.asm"),
            palette: tables::flat(&[[150, 104, 58], [214, 28, 52], [255, 170, 180], [96, 12, 24], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0], [0, 0, 0]]),
            sin48: tables::cube::sin48() }
    }

    /// Frame f's tile at (tx, ty): its 48 rows of red, green and blue.
    pub fn tile(&self, host: &mut Host, f: i32, tx: usize, ty: usize) -> Vec<Vec<i32>> {
        // the frame's two angles on the clock machine
        let t = host.go1(&self.clock, &[&[f]], 2);
        let (a, b, cu, cv) = (t[0], t[1], 450, 450);
        let o = host.go(&self.rot, &[&[&[a, b, cu, cv, tx as i32, ty as i32], &self.sin48]], Some(2 + 2 * H));
        let (du, dv, rows) = (o[0], o[1], &o[2..]);
        (0..H).map(|y| {
            let codes = host.go1(&self.rz, &[&[rows[2 * y], rows[2 * y + 1], du, dv]], W);
            host.go1(&self.paint, &[&self.palette, &codes], W * 3)
        }).collect()
    }
}

pub fn render(demo: &Path, refbin: &str) {
    let roto = Roto4::new(demo);
    let mut host = Host::new();
    let mut video = Video2::new(FRAMES);
    for f in 0..FRAMES {
        for (tx, ty) in [(0, 0), (W, 0), (0, H), (W, H)] {
            for (y, rgb) in roto.tile(&mut host, f as i32, tx, ty).iter().enumerate() {
                let at = ((f * H2 + ty + y) * W2 + tx) * 3;
                for (i, &v) in rgb.iter().enumerate() { video.buf[at + i] = v as u8; }
            }
        }
    }
    host.done();
    finish("roto4", &host, &video, &reference(refbin, &[]), &demo.join("roto/video4.rgb"), "");
}
