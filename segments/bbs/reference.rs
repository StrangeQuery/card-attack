// BBS reference: the cube over the BBS's message reader, and over the rotozoomer, drawn in ordinary Rust from the
// cube's own frames (the tumble's, tumble/video.rgb; the die's and the Rubik's cube's, dierubik/video.rgb), the
// rotozoomer's (roto/video4.rgb), the BBS page (ui/pages.rs) and the atlas. Wherever a cube's pixel isn't black the
// cube is in front; elsewhere the picture behind. Args: the demo folder, then `hd` or `roto`:
// - hd, video_hd.rgb, 1536x768, 272 frames: the BBS page (each pixel 2 x 4, 1280x768 centred) arriving 20 characters
//   a frame from its seventh frame (the first 20 (k - 6) at frame k), the cursor after them blinking (six frames on,
//   six off); over it the tumble's frames 14..147, then the die's and the Rubik's cube's frames 0..129, each pixel 8 x 8;
//   then its frames 130..137 over the page crossfading into the rotozoomer's frames 42..49 (step j of 9: 32 j / 9
//   32nds, rounded down, a + (b - a) c / 32 toward zero).
// - roto, video.rgb, 192x96, 72 frames: the rotozoomer's frames 50..121 with the cube's frames 138..209 over them; from
//   the 37th, each row of the rotozoomer pushed sideways (wrapping round) by 5 g / 15 sin(9 y - 28.648 k degrees)
//   pixels, rounded half away from zero, g = min(k, 15), k frames into the wobble: the sine from the printed table
//   (rubik/data/sine.txt) between degrees by a straight line, the angle in tenths (90 a row; 3313.521 a frame, -0.5
//   radians, rounded to the tenth).
use std::io::Write;

#[path = "../ui/pages.rs"]
#[allow(dead_code)]
mod pages;
#[path = "../ui/text.rs"]
mod text;

const W: usize = 192;
const H: usize = 96;
const HW: usize = 1536;
const HH: usize = 768;
const F: usize = W * H * 3;

fn main() {
    let demo = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let mode = std::env::args().nth(2).unwrap();
    let read = |p: &str| std::fs::read(demo.join(p)).unwrap();
    let (tumble, cube, roto) = (read("tumble/video.rgb"), read("dierubik/video.rgb"), read("roto/video4.rgb"));
    let frame = |v: &Vec<u8>, f: usize| v[f * F..(f + 1) * F].to_vec();
    let mut out = std::io::stdout().lock();
    if mode == "hd" {
        let glyphs = text::Glyphs::read(&demo);
        let page = &pages::BBS;
        let total = page.chars();
        let mut hd = vec![0u8; HW * HH * 3];
        for k in 0..272 {
            let shown = (20 * k.max(6).min(51) - 120).min(total);
            let px = text::draw(page, &glyphs, shown, k / 6 % 2 == 0);
            let (front, behind) = if k < 134 { (frame(&tumble, 14 + k), None) } else if k < 264 { (frame(&cube, k - 134), None) } else { (frame(&cube, k - 134), Some(frame(&roto, 42 + k - 264))) };
            let c = (32 * (k as i64 - 263) / 9) as u8;
            for y in 0..HH { for x in 0..HW {
                let i = ((y / 8) * W + x / 8) * 3;
                let rgb = if front[i..i + 3].iter().any(|&v| v > 0) {
                    [front[i], front[i + 1], front[i + 2]]
                } else {
                    let p = text::on_page(page, x, y).map_or([0; 3], |j| [px[3 * j], px[3 * j + 1], px[3 * j + 2]]);
                    match &behind { None => p, Some(b) => [0, 1, 2].map(|ch| text::mix(p[ch], b[i + ch], c)) }
                };
                hd[(y * HW + x) * 3..][..3].copy_from_slice(&rgb);
            } }
            out.write_all(&hd).unwrap();
        }
    } else {
        let table: Vec<i64> = std::fs::read_to_string(demo.join("rubik/data/sine.txt")).unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect();
        let sine = |tenths: i64| {
            let read = |at: i64| { let (d, t) = ((at / 10) as usize, at % 10); table[d] + (table[d + 1] - table[d]) * t / 10 };
            let (q, r) = (tenths / 900, tenths % 900);
            let (mut s, mut c) = (read(r), read(900 - r));
            for _ in 0..q { let t = s; s = c; c = -t; }
            s
        };
        for k in 0..72 {
            let (back, front) = (frame(&roto, 50 + k), frame(&cube, 138 + k));
            let mut img = vec![0u8; F];
            for y in 0..H {
                let push = if k < 36 { 0 } else {
                    let w = (k - 36) as i64;
                    let grown = w.min(15);
                    let tenths = ((y as i64 * 90_000 + w * 3_313_521 + 500) / 1000).rem_euclid(3600);
                    let s = sine(tenths);
                    s.signum() * ((grown * s.abs() + 1498) / 999 / 3)
                };
                for x in 0..W {
                    let i = (y * W + x) * 3;
                    let from = (y * W + (x as i64 - push).rem_euclid(W as i64) as usize) * 3;
                    let src = if front[i..i + 3].iter().any(|&v| v > 0) { &front[i..i + 3] } else { &back[from..from + 3] };
                    img[i..i + 3].copy_from_slice(src);
                }
            }
            out.write_all(&img).unwrap();
        }
    }
}
