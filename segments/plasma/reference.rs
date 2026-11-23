// Plasma reference at 192x96: four waves and the turning colours at every pixel, on a plane that stands facing the
// camera and, at the end, tilts back to become the ray's floor. Each pixel straight from the formula
// (formula.rs), in ordinary Rust. Args: the colour ring, the script (per frame the plane's 600 cos and 100 sin).
use std::io::Write;
#[allow(unused_attributes, dead_code)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;
#[allow(dead_code)]
#[path = "formula.rs"]
mod formula;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let keys = formula::keys(&std::fs::read_to_string(&a[1]).unwrap());
    let script = formula::script(&a[2]);
    let mut out = std::io::stdout().lock();
    for (f, s) in script.iter().enumerate() {
        for y in 0..96 { for x in 0..192 {
            let c = formula::colour(&keys, formula::plane(x, y, s[0], s[1], None), f as i32);
            out.write_all(&c.map(|v| v as u8)).unwrap();
        } }
    }
}
