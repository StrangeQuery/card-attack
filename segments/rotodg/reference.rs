// The degauss at the rotozoom's end (moved from Mode X's end, where the dorm now shows the monitor
// small; the transition into the VGA plasma mid-degauss), in ordinary Rust: the reference for host/src/fx/rotodg.rs.
// 36 frames at 1536x768: the rotozoom's frames 36..63 (bbs/video.rgb) and then the plasma's 0..7 (vga/video.rgb), each
// 192x96 shown 8 x 8 a pixel, under Mode X's degauss (modex/degauss.rs) as if its frames 114..149: the blotch creeping
// in (its falloffs over 192x96: modex/data/gauss_roto.txt), the DEGAUSS sign blinking, the jolt from frame 20, the
// cut to the plasma at frame 28, half way through it.
// Args: the demo folder.
use std::io::Write;
use std::path::Path;

#[allow(unused_attributes, dead_code)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;
#[path = "../ui/pages.rs"]
#[allow(dead_code)]
mod pages;
#[path = "../ui/text.rs"]
#[allow(dead_code)]
mod text;
#[path = "../modex/degauss.rs"]
mod degauss;

const W: usize = 192;
const H: usize = 96;
const HW: usize = 1536;
const HH: usize = 768;
pub const FRAMES: usize = 36;
/// The rotozoom's frames from 36, then the plasma's from 0 at frame CUT.
pub const ROTO0: usize = 36;
pub const CUT: usize = 28;
/// The degauss's own frame for ours: from its 114.
pub const K0: usize = 114;

fn main() {
    let demo = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let roto = std::fs::read(demo.join("bbs/video.rgb")).unwrap();
    let vga = std::fs::read(demo.join("vga/video.rgb")).unwrap();
    let size = W * H * 3;
    let glyphs = text::Glyphs::read(&demo);
    let dg = degauss::read(&demo);
    let gauss = degauss::read_gauss(&demo, "gauss_roto.txt");
    let word_page = &pages::DEGAUSS;
    let word = text::draw(word_page, &glyphs, word_page.chars(), false);
    let sign = degauss::sign_pixels(&dg, &|x, y| text::on_page(word_page, x, y).map_or(false, |i| word[3 * i + 1] > 0));
    let turn: Vec<(i64, i64)> = (0..(W * H) as i64).map(|i| degauss::angle(&dg, 2 * (i % W as i64) - (W as i64 - 1), 2 * (i / W as i64) - (H as i64 - 1))).collect();
    let cell = |x: usize, y: usize| Some((y / 8) * W + x / 8);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut hd = vec![0u8; HW * HH * 3];
    let _ = Path::new("");
    for j in 0..FRAMES {
        let k = K0 + j;
        let pic = if j < CUT { &roto[(ROTO0 + j) * size..][..size] } else { &vga[(j - CUT) * size..][..size] };
        let s = degauss::strength(k);
        let blotched;
        let pic = if s > 0 { blotched = degauss::blotch_wh(&gauss, s, pic, W, H); &blotched[..] } else { pic };
        for y in 0..HH { for x in 0..HW {
            let i = ((y / 8) * W + x / 8) * 3;
            hd[(y * HW + x) * 3..][..3].copy_from_slice(&pic[i..i + 3]);
        } }
        if degauss::sign_shown(k) { degauss::lay_sign(&mut hd, &sign, &text::mix); }
        if (degauss::D0..degauss::D0 + degauss::DN).contains(&k) {
            out.write_all(&degauss::jolt(&dg, k - degauss::D0, &hd, &cell, &turn)).unwrap();
        } else {
            out.write_all(&hd).unwrap();
        }
    }
}
