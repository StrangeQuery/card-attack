// UI kit reference: a scene's layer (scenes.rs) drawn in ordinary Rust, frame by frame: W x H pixels, each red, green,
// blue and cover (0-32). Every pixel of every item is worked out by the formulas (draw.rs): the glides, the text set
// by its advances, then each laid over the layer. Args: the scene's name, the demo folder.
use std::io::Write;

#[allow(dead_code)]
#[path = "scenes.rs"]
mod scenes;
#[path = "draw.rs"]
mod draw;
use scenes::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let sc = scene(&a[1]);
    let atlas = draw::Atlas::read(std::path::Path::new(&a[2]));
    let mut out = std::io::stdout().lock();
    for f in 0..sc.frames {
        out.write_all(&draw::layer(&atlas, sc.w, sc.h, sc.ox, sc.oy, &(sc.frame)(f))).unwrap();
    }
}
