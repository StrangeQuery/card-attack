// The UI kit's text pages in ordinary Rust, for the references (vga, usenet, ...): a page (pages.rs) drawn at its own
// pixels with its first `shown` characters and the cursor after them, from the atlas's bitmap faces; and its pixels
// shown on the 1536x768 frame. Include with #[path] beside pages.rs. Every glyph pixel is laid on as the kit lays
// any: ink over paper (or over the page, where there's no paper) by its cover, a + (b - a) c / 32 toward zero.
// And lines in a proportional bitmap face at one bit (the Mac's, faces 17-19): each glyph placed from the pen by its
// offsets, the advances summed (the width halved for a centred line, taken off for one set right), its set pixels
// laid on in the line's ink (a pixel of a one-bit screen: true black).
#![allow(dead_code)]
use std::collections::HashMap;
use std::path::Path;

use super::pages::*;

/// The atlas's glyphs: (face, code) -> (its covers' offset in atlas.bin, width, height), and the covers; and each
/// glyph's advance and its bitmap's place from the pen.
pub struct Glyphs { at: HashMap<(usize, u32), (usize, usize, usize)>, cover: Vec<u8>, pen: HashMap<(usize, u32), (i64, i64, i64)> }

impl Glyphs {
    pub fn read(demo: &Path) -> Glyphs {
        let cover = std::fs::read(demo.join("ui/data/atlas.bin")).unwrap();
        let (mut at, mut pen) = (HashMap::new(), HashMap::new());
        for l in std::fs::read_to_string(demo.join("ui/data/atlas.txt")).unwrap().lines().filter(|l| l.starts_with("g ")) {
            let p: Vec<i64> = l.split(' ').skip(1).map(|v| v.parse::<i64>().unwrap()).collect();
            at.insert((p[0] as usize, p[1] as u32), (p[7] as usize, p[5] as usize, p[6] as usize));
            pen.insert((p[0] as usize, p[1] as u32), (p[2], p[3], p[4]));
        }
        Glyphs { at, cover, pen }
    }

    /// A glyph's advance.
    pub fn advance(&self, face: usize, code: u32) -> i64 { self.pen.get(&(face, code)).unwrap_or_else(|| panic!("no glyph {} in face {}", code, face)).0 }

    /// Lay a glyph of a bitmap face on a one-bit screen (w wide, true black) with its pen at x, y: its set pixels
    /// made the ink, the rest left; kept to the screen.
    pub fn lay_bits(&self, screen: &mut [bool], w: usize, face: usize, code: u32, x: i64, y: i64, ink: bool) {
        let (_, ox, oy) = self.pen[&(face, code)];
        let (at, gw, gh) = self.at[&(face, code)];
        let h = screen.len() / w;
        for row in 0..gh { for col in 0..gw {
            let (sx, sy) = (x + ox + col as i64, y + oy + row as i64);
            if sx < 0 || sy < 0 || sx >= w as i64 || sy >= h as i64 || self.cover[at + row * gw + col] == 0 { continue }
            screen[sy as usize * w + sx as usize] = ink;
        } }
    }
}

/// Where a line's pen starts: at x (Start), the width (the advances summed) halved and taken off (Centre), or all of
/// it taken off (End); toward zero, as the text machine (ui_text) halves.
pub fn line_start(x: i64, advances: &[i64], across: u8) -> i64 {
    let width: i64 = advances.iter().sum();
    match across { 0 => x, 1 => x - width / 2, _ => x - width }
}

/// a + (b - a) c / 32, toward zero.
pub fn mix(a: u8, b: u8, c: u8) -> u8 { (a as i32 + (b as i32 - a as i32) * c as i32 / 32) as u8 }

/// The page with its first `shown` characters (and the cursor just after the last if `cursor`): w x h RGB, black
/// where nothing is.
pub fn draw(page: &Page, glyphs: &Glyphs, shown: usize, cursor: bool) -> Vec<u8> {
    let mut px = vec![0u8; page.w * page.h * 3];
    let lay = |px: &mut Vec<u8>, code: u32, x: i64, y: i64, ink: Rgb, paper: Option<Rgb>| {
        let &(at, w, h) = glyphs.at.get(&(page.face, code)).unwrap();
        for row in 0..h { for col in 0..w {
            let (gx, gy) = (x + col as i64, y + row as i64);
            if gx < 0 || gy < 0 || gx >= page.w as i64 || gy >= page.h as i64 { continue }
            let c = glyphs.cover[at + row * w + col];
            let i = (gy as usize * page.w + gx as usize) * 3;
            let under = match paper { Some(p) => p, None if c == 0 => continue, None => [px[i], px[i + 1], px[i + 2]] };
            for k in 0..3 { px[i + k] = mix(under[k], ink[k], c); }
        } }
    };
    let (mut left, mut last) = (shown, None);
    for (n, line) in page.lines.iter().enumerate() {
        let y = page.top as i64 + n as i64 * page.step as i64;
        let count = line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>() as i64;
        let mut x = line.x as i64 - if line.centred { count * page.advance as i64 / 2 } else { 0 };
        for run in line.runs { for ch in run.text.chars() {
            if left == 0 { continue }
            left -= 1;
            lay(&mut px, ch as u32, x, y, run.ink, run.paper);
            x += page.advance as i64;
            last = Some((x, y));
        } }
    }
    if let (true, Some((x, y))) = (cursor, last) { lay(&mut px, CURSOR, x, y, page.cursor, None); }
    px
}

/// Pixel (x, y) of the 1536x768 frame on the page shown there: its index in the page's pixels, if it's on it.
pub fn on_page(page: &Page, x: usize, y: usize) -> Option<usize> {
    if x < page.x || y < page.y { return None }
    let (u, v) = ((x - page.x) / page.sx, (y - page.y) / page.sy);
    (u < page.w && v < page.h).then_some(v * page.w + u)
}
