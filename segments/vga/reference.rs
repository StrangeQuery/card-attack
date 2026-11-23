// VGA reference: the climb from the EGA to the VGA, drawn in ordinary Rust from the plasma's formula and its own
// frames (plasma/formula.rs, plasma/video.rgb), the EGA's sixteen colours (vga/data/ega.txt), the 16-colour plasma's
// table and the wipe (vga/data/ansi.txt, wipe.txt), the CompuServe page and the brag (ui/pages.rs) and the atlas.
// Writes, one after another:
// - video.rgb, 192x96: the plasma in sixteen colours (each pixel's place on the colour ring, from the formula, its
//   code the printed table's, its colour the EGA's), of the plasma's frames 48..77, then 90..97; then 98..103 with
//   the wipe: the rows above each frame's edge the plasma's own frame, the edge's row white;
// - video_hd.rgb, 1536x768: six frames of the 16-colour plasma (its frames 24..29) crossfading into CompuServe's
//   page arriving (its first 252 j / 6 characters at step j, the cursor after them), 24 of the page with the
//   cursor blinking (six frames on, six off);
// - brag.rgba, 704x192 RGB and cover: 320 x 200 / 256 COLORS on a black box, for the cut's over line.
// The crossfade at step j of 7 takes each pixel from a toward b by 32 j / 7 32nds (rounded down), a + (b - a) c / 32
// toward zero: j / 7 to within a 64th. Args: the demo folder.
use std::io::Write;

#[path = "../ui/pages.rs"]
#[allow(dead_code)]
mod pages;
#[path = "../ui/text.rs"]
mod text;
#[allow(unused_attributes, dead_code)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;
#[allow(dead_code)]
#[path = "../plasma/formula.rs"]
mod formula;
use pages::*;

const W: usize = 192;
const H: usize = 96;
const HW: usize = 1536;
const HH: usize = 768;
/// The 16-colour plasma's frames: the plasma's frame each shows (the intro, back after the page, the wipe).
const INTRO: usize = 30;
const BACK: usize = 8;
const WIPE: usize = 6;
fn plasma_frame(f: usize) -> usize { if f < INTRO { 48 + f } else { 90 + f - INTRO } }
/// Frames of a crossfade (its steps 1..=6 of 7); frames of the page held.
const X: usize = 6;
const HOLD: usize = 24;

fn main() {
    let demo = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let plasma = std::fs::read(demo.join("plasma/video.rgb")).unwrap();
    let frame = |f: usize| &plasma[f * W * H * 3..(f + 1) * W * H * 3];
    let printed = |f: &str| -> Vec<i32> { std::fs::read_to_string(demo.join(f)).unwrap().lines().filter(|l| !l.starts_with('#')).flat_map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect::<Vec<i32>>()).collect() };
    let ega: Vec<[i32; 3]> = printed("vga/data/ega.txt").chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
    let (ansi, wipe) = (printed("vga/data/ansi.txt"), printed("vga/data/wipe.txt"));
    let keys = formula::keys(&std::fs::read_to_string(demo.join("plasma/data/ring.txt")).unwrap());
    let script = formula::script(demo.join("plasma/data/script.txt").to_str().unwrap());
    let glyphs = text::Glyphs::read(&demo);
    let mut out = std::io::stdout().lock();

    // the 16-colour plasma's frames
    let mut egas = vec![];
    for f in 0..INTRO + BACK + WIPE {
        let pf = plasma_frame(f);
        let s = &script[pf];
        let mut pic = vec![0u8; W * H * 3];
        for y in 0..H { for x in 0..W {
            let c = match formula::plane(x as i32, y as i32, s[0], s[1], None) {
                formula::Look::Plasma { u, v, .. } => ega[ansi[formula::plasma(u, v, pf as i32) as usize] as usize],
                look => formula::colour(&keys, look, pf as i32),
            };
            pic[(y * W + x) * 3..][..3].copy_from_slice(&c.map(|v| v as u8));
        } }
        if f >= INTRO + BACK {
            let edge = wipe[f - INTRO - BACK] as usize;
            pic[..edge.min(H) * W * 3].copy_from_slice(&frame(pf)[..edge.min(H) * W * 3]);
            if edge < H { pic[edge * W * 3..(edge + 1) * W * 3].fill(255); }
        }
        out.write_all(&pic).unwrap();
        egas.push(pic);
    }

    // the 1536x768 frames: a 192x96 picture's pixel under each, the page's
    let page = &COMPUSERVE;
    let total = page.chars();
    let big = |pic: &[u8], x: usize, y: usize| { let i = ((y / 8) * W + x / 8) * 3; [pic[i], pic[i + 1], pic[i + 2]] };
    let shown = |px: &[u8], x: usize, y: usize| text::on_page(page, x, y).map_or([0; 3], |i| [px[3 * i], px[3 * i + 1], px[3 * i + 2]]);
    let mut hd = vec![0u8; HW * HH * 3];
    for f in 0..X + HOLD {
        // the page as this frame shows it, and the step of the crossfade (0: none)
        let (chars, cursor, j) = if f < X { (total * (f + 1) / 6, true, f + 1) } else { (total, (f - X) / 6 % 2 == 0, 0) };
        let px = text::draw(page, &glyphs, chars, cursor);
        let c = (32 * j / 7) as u8;
        for y in 0..HH { for x in 0..HW {
            let rgb = if f < X {
                let (a, b) = (big(&egas[INTRO - X + f], x, y), shown(&px, x, y));
                [0, 1, 2].map(|k| text::mix(a[k], b[k], c))
            } else {
                shown(&px, x, y)
            };
            hd[(y * HW + x) * 3..][..3].copy_from_slice(&rgb);
        } }
        out.write_all(&hd).unwrap();
    }

    // the brag: the page's pixels four times over, all of it covering (the box is black)
    let brag = &VGA_BRAG;
    let px = text::draw(brag, &glyphs, brag.chars(), false);
    let (bw, bh) = (brag.w * brag.sx, brag.h * brag.sy);
    let mut layer = vec![0u8; bw * bh * 4];
    for y in 0..bh { for x in 0..bw {
        let i = text::on_page(brag, brag.x + x, brag.y + y).unwrap();
        layer[(y * bw + x) * 4..][..4].copy_from_slice(&[px[3 * i], px[3 * i + 1], px[3 * i + 2], 32]);
    } }
    out.write_all(&layer).unwrap();
}
