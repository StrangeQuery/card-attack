//! The UI kit's text pages: screens of text in a PC's bitmap font (segments/ui/pages.rs), on CARDIAC, at the page's
//! own pixels; and pictures crossfaded at 1536x768. Shared by the effects that show text (vga, usenet; the BBS, the
//! Mac, the terminal can take it up).
//!
//! A page, a frame: the page machine (ui_page) places every character that has arrived (the advances summed along a
//! line, centred on its x if the line says so, the line step down the page) and the cursor just after the last; the
//! host fetches each glyph's covers from the atlas (its bitmap faces: one bit, each glyph its whole cell) by the
//! glyph's code, and deep's mix machine (umix) lays each pixel on: from the paper (or the page under it, where the
//! run has no paper) toward the ink by its cover, a + (b - a) c / 32. How many characters have arrived (step j of n
//! of the page's) and a crossfade's cover (32 j / n) are the ease machine's (ui_ease).
//!
//! A crossfade at 1536x768 mixes two pictures, each shown there as a blit of its own pixels (a 192x96 frame 8 x 8
//! times over, a page sx x sy): every pair of their pixels that meets on the frame is mixed once on the mix machine,
//! and the host files the result wherever that pair meets (a blit of computed pixels).
//!
//! Lines in a proportional bitmap face at one bit (the Mac's Chicago and Geneva, the atlas's faces 17-19): the text
//! machine (ui_text) places every glyph, the advances summed from the line's anchor (the width halved and taken off
//! for a centred line, all of it for one set right), each glyph's bitmap at its offsets from the pen; the host lays
//! its set pixels on in the line's ink (one bit: a copy, nothing to mix).
use crate::*;
use crate::fx::ui::Atlas;
use std::collections::HashMap;
use std::path::Path;

#[allow(dead_code)]
#[path = "../../../segments/ui/pages.rs"]
pub mod pages;
use pages::*;

pub const HW: usize = 1536;
pub const HH: usize = 768;
/// A pixel of the frame no picture covers: black.
pub const NONE: u32 = u32::MAX;

/// A picture on the 1536x768 frame: its pixels (RGB), and for each pixel of the frame the index of the pixel of
/// the picture shown there, or NONE.
pub struct Shown<'a> { pub px: &'a [u8], pub at: &'a [u32] }

/// Where a 192x96 frame's pixels go on the 1536x768 frame: each 8 x 8 times over.
pub fn big8() -> Vec<u32> { (0..HW * HH).map(|i| ((i / HW / 8) * 192 + i % HW / 8) as u32).collect() }

/// Where a page's pixels go on the 1536x768 frame: each sx x sy times over, from its place.
pub fn on_frame(page: &Page) -> Vec<u32> {
    (0..HW * HH).map(|i| {
        let (x, y) = (i % HW, i / HW);
        if x < page.x || y < page.y { return NONE }
        let (u, v) = ((x - page.x) / page.sx, (y - page.y) / page.sy);
        if u < page.w && v < page.h { (v * page.w + u) as u32 } else { NONE }
    }).collect()
}

pub struct Text { page: Machine, ease: Machine, mix: Machine, line: Machine, pub atlas: Atlas }

/// A line for the text machine: its anchor (x, y), how it sits on it (0 starting there, 1 centred, 2 ending there),
/// and each glyph's advance and its bitmap's place from the pen (across, down).
pub struct Line { pub x: i32, pub y: i32, pub across: i32, pub glyphs: Vec<[i32; 3]> }

impl Text {
    pub fn new(demo: &Path) -> Text { Text { page: rust(demo, "ui_page"), ease: rust(demo, "ui_ease"), mix: rust(demo, "umix"), line: rust(demo, "ui_text"), atlas: Atlas::read(demo) } }

    /// Every glyph's place on its line, on the text machine (ui_text): x, y of each, line after line.
    pub fn set(&self, host: &mut Host, lines: &[Line]) -> Vec<i32> {
        let mut deck = vec![lines.len() as i32];
        for l in lines {
            deck.extend([0, l.x, 0, l.y, 0, l.across, l.glyphs.len() as i32]);
            if l.across > 0 { deck.extend(l.glyphs.iter().map(|g| g[0])); }
            for g in &l.glyphs { deck.extend(g); }
        }
        host.go1(&self.line, &[&deck], 2 * lines.iter().map(|l| l.glyphs.len()).sum::<usize>())
    }

    /// A glyph of a face as the text machine sets it: its advance and its bitmap's place from the pen.
    pub fn glyph(&self, face: usize, code: u32) -> [i32; 3] { let g = self.atlas.glyph(face, code); [g.adv as i32, g.ox as i32, g.oy as i32] }

    /// Lay a glyph of a bitmap face on a one-bit screen (w wide, true black) with its bitmap's top left at x, y (as
    /// the text machine placed it): its set pixels made the ink, kept to the screen.
    pub fn lay_bits(&self, screen: &mut [bool], w: usize, face: usize, code: u32, x: i32, y: i32, ink: bool) {
        let g = self.atlas.glyph(face, code);
        let h = screen.len() / w;
        for row in 0..g.h { for col in 0..g.w {
            let (sx, sy) = (x + col as i32, y + row as i32);
            if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 || self.atlas.cover[g.at + row * g.w + col] == 0 { continue }
            screen[sy as usize * w + sx as usize] = ink;
        } }
    }

    /// Steps along the way from 0 to `to`: each (j, n) gives to j / n, rounded down, on the ease machine.
    pub fn steps(&self, host: &mut Host, to: i32, steps: &[(i32, i32)]) -> Vec<i32> {
        let deck: Vec<i32> = steps.iter().flat_map(|&(j, n)| [1, 0, to, j, n]).collect();
        host.go1(&self.ease, &[&[steps.len() as i32], &deck], steps.len())
    }

    /// Mix, on the mix machine: for each (a, b, c), a + (b - a) c / 32 toward zero. At most 333 a run (the deck's
    /// count, three cards a pixel, is a card).
    pub fn mix(&self, host: &mut Host, abc: &[[i32; 3]]) -> Vec<i32> {
        let mut out = Vec::with_capacity(abc.len());
        for part in abc.chunks(999) {
            let deck: Vec<i32> = part.iter().flatten().copied().collect();
            out.extend(host.go1(&self.mix, &[&[part.len() as i32], &deck], part.len()));
        }
        out
    }

    /// The page with its first `shown` characters, and the cursor after the last if `cursor`: its own w x h pixels,
    /// RGB, black where nothing is.
    pub fn draw(&self, host: &mut Host, page: &Page, shown: usize, cursor: bool) -> Vec<u8> {
        // every character's place, on the page machine
        let mut deck = vec![page.advance as i32, page.step as i32, page.top as i32, shown as i32, cursor as i32, page.lines.len() as i32];
        let mut chars: Vec<(u32, Rgb, Option<Rgb>)> = vec![];
        for line in page.lines {
            let n = line.runs.iter().map(|r| r.text.chars().count()).sum::<usize>();
            deck.extend([line.centred as i32, line.x as i32, n as i32]);
            for run in line.runs { chars.extend(run.text.chars().map(|c| (c as u32, run.ink, run.paper))); }
        }
        chars.truncate(shown);
        if cursor && !chars.is_empty() { chars.push((CURSOR, page.cursor, None)); }
        let places = host.go1(&self.page, &[&deck], 2 * chars.len());
        // each glyph's pixels laid on in order, on the mix machine; the page kept to, as a blit does
        let mut px = vec![0u8; page.w * page.h * 3];
        let mut lay: Vec<(usize, Option<Rgb>, Rgb, i32)> = vec![];
        let mut seen = std::collections::HashSet::new();
        for (k, &(code, ink, paper)) in chars.iter().enumerate() {
            let gl = self.atlas.glyph(page.face, code);
            assert!(gl.ox == 0 && gl.oy == 0, "a bitmap face's glyph is its whole cell");
            for row in 0..gl.h { for col in 0..gl.w {
                let (x, y) = (places[2 * k] + col as i32, places[2 * k + 1] + row as i32);
                if x < 0 || y < 0 || x >= page.w as i32 || y >= page.h as i32 { continue }
                let c = self.atlas.cover[gl.at + row * gl.w + col] as i32;
                let i = (y as usize * page.w + x as usize) * 3;
                if paper.is_none() && c == 0 { continue }
                // a pixel laid on twice: what's under it the second time is the first's
                if !seen.insert(i) { self.lay(host, &mut px, &mut lay); seen.clear(); seen.insert(i); }
                lay.push((i, paper, ink, c));
            } }
        }
        self.lay(host, &mut px, &mut lay);
        px
    }

    /// Lay pixels on: where, the paper (None: the page under it), the ink, the cover.
    fn lay(&self, host: &mut Host, px: &mut [u8], lay: &mut Vec<(usize, Option<Rgb>, Rgb, i32)>) {
        let abc: Vec<[i32; 3]> = lay.iter().flat_map(|&(i, paper, ink, c)| (0..3).map(move |k| (i, paper, ink, c, k)))
            .map(|(i, paper, ink, c, k)| [paper.map_or(px[i + k], |p| p[k]) as i32, ink[k] as i32, c]).collect();
        let mixed = self.mix(host, &abc);
        for (n, &(i, ..)) in lay.iter().enumerate() { for k in 0..3 { px[i + k] = mixed[3 * n + k] as u8; } }
        lay.clear();
    }

    /// Picture a crossfading into picture b at `cover` 32nds of the way, on the 1536x768 frame: each pair of their
    /// pixels that meets, mixed once.
    pub fn crossfade(&self, host: &mut Host, a: &Shown, b: &Shown, cover: i32) -> Vec<u8> {
        let mut pairs: HashMap<(u32, u32), u32> = HashMap::new();
        let mut order: Vec<(u32, u32)> = vec![];
        let slot: Vec<u32> = (0..HW * HH).map(|i| *pairs.entry((a.at[i], b.at[i])).or_insert_with(|| { order.push((a.at[i], b.at[i])); order.len() as u32 - 1 })).collect();
        let rgb = |s: &Shown, i: u32, k: usize| if i == NONE { 0 } else { s.px[3 * i as usize + k] as i32 };
        let abc: Vec<[i32; 3]> = order.iter().flat_map(|&(ia, ib)| (0..3).map(move |k| [rgb(a, ia, k), rgb(b, ib, k), cover])).collect();
        let mixed = self.mix(host, &abc);
        let mut frame = vec![0u8; HW * HH * 3];
        for (i, &s) in slot.iter().enumerate() { for k in 0..3 { frame[3 * i + k] = mixed[3 * s as usize + k] as u8; } }
        frame
    }

    /// A picture as it is on the 1536x768 frame (a blit: no mixing).
    pub fn blit(s: &Shown) -> Vec<u8> {
        let mut frame = vec![0u8; HW * HH * 3];
        for (i, &at) in s.at.iter().enumerate() { if at != NONE { frame[3 * i..3 * i + 3].copy_from_slice(&s.px[3 * at as usize..3 * at as usize + 3]); } }
        frame
    }
}
