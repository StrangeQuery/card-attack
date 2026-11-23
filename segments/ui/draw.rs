// The UI kit's layers in ordinary Rust, for the references (ui's own, the party chat's): groups of items (scenes.rs) drawn
// on a layer of W x H pixels, each red, green, blue and cover (0-32). Every pixel of every item is worked out by the
// formulas: the eases, the corners' cover counted 4 x 4 inside the circle, the shadows' levels, the text set by its
// advances, then each laid over the layer. Include with #[path] beside scenes.rs (as `scenes`).
#![allow(dead_code)]
use std::collections::HashMap;
use std::path::Path;

use super::scenes::*;

/// A number of the scene: from + (to - from) u (or u u (3 - 2 u)), each product rounded down; eased out,
/// to - (to - from) w w w, w = 1 - u, each product rounded down.
pub fn value(v: Val) -> i16 {
    match v {
        Val::K(k) => k,
        Val::Ease { smooth, from, to, j, n } => {
            let (j, n) = (j as i64, n as i64);
            let mut p = (to as i64 - from as i64).abs() * j / n;
            if smooth { p = p * j / n; p = p * (3 * n - 2 * j) / n; }
            (if to < from { from as i64 - p } else { from as i64 + p }) as i16
        }
        Val::Cube { from, to, j, n } => {
            let (j, n) = (j as i64, n as i64);
            let mut p = (to as i64 - from as i64).abs();
            for _ in 0..3 { p = p * (n - j) / n; }
            (if to < from { to as i64 + p } else { to as i64 - p }) as i16
        }
    }
}

pub struct Glyph { pub adv: i64, pub ox: i64, pub oy: i64, pub w: usize, pub h: usize, pub at: usize, pub rgb: Option<usize> }

/// The atlas (ui/data/atlas.txt and atlas.bin): each face's middle, each glyph, the coverages (and a colour glyph's
/// colours).
pub struct Atlas { pub mid: Vec<i64>, pub glyphs: HashMap<(usize, u32), Glyph>, pub bin: Vec<u8> }

impl Atlas {
    pub fn read(demo: &Path) -> Atlas {
        let bin = std::fs::read(demo.join("ui/data/atlas.bin")).unwrap();
        let (mut mid, mut glyphs): (Vec<i64>, HashMap<(usize, u32), Glyph>) = (vec![], HashMap::new());
        for l in std::fs::read_to_string(demo.join("ui/data/atlas.txt")).unwrap().lines().filter(|l| !l.starts_with('#')) {
            let p: Vec<i64> = l.split(' ').skip(1).map(|v| v.parse().unwrap()).collect();
            if l.starts_with("face") { mid.push(p[3]) }
            else if l.starts_with("rgb") { glyphs.get_mut(&(p[0] as usize, p[1] as u32)).unwrap().rgb = Some(p[2] as usize) }
            else { glyphs.insert((p[0] as usize, p[1] as u32), Glyph { adv: p[2], ox: p[3], oy: p[4], w: p[5] as usize, h: p[6] as usize, at: p[7] as usize, rgb: None }); }
        }
        Atlas { mid, glyphs, bin }
    }
}

/// Of a corner of radius r, the pixel px across and py down from the circle's middle: 2 for each of its 4 x 4
/// points (their middles) inside the circle of radius 4r.
fn corner(r: i64, px: i64, py: i64) -> u8 {
    let big = 4 * r;
    let mut n = 0;
    for sy in 4 * py..4 * py + 4 { for sx in 4 * px..4 * px + 4 {
        if (2 * sx + 1) * (2 * sx + 1) + (2 * sy + 1) * (2 * sy + 1) <= 4 * big * big { n += 2; }
    } }
    n
}

/// Pixel p laid over by colour rgb at cover c (then fade 32nds): its cover over the layer's, its colour by its share.
pub fn over(p: &mut [u8], rgb: [u8; 3], cover: u8, fade: i64) {
    let c = if fade < 32 { cover as i64 * fade / 32 } else { cover as i64 };
    if c == 0 { return }
    let a = p[3] as i64;
    if c == 32 || a == 0 { p[..3].copy_from_slice(&rgb); p[3] = c as u8; return }
    let both = a + (32 - a) * c / 32;
    let share = 32 * c / both;
    for k in 0..3 { let (x, y) = (p[k] as i64, rgb[k] as i64); p[k] = (x + (y - x) * share / 32) as u8; }
    p[3] = both as u8;
}

/// The groups drawn on a layer of w x h, its items' places counted from (-ox, -oy), the layer's top left.
pub fn layer(atlas: &Atlas, w: usize, h: usize, ox: i16, oy: i16, groups: &[Group]) -> Vec<u8> {
    let (glyphs, cover) = (&atlas.glyphs, &atlas.bin);
    let (wl, hl, ox, oy) = (w as i64, h as i64, ox as i64, oy as i64);
    let mut layer = vec![0u8; w * h * 4];
    for g in groups {
        let (dx, dy, fade) = (value(g.dx) as i64, value(g.dy) as i64, value(g.fade) as i64);
        let clip = g.clip.map(|c| c.map(|v| v as i64)).unwrap_or([-999, -999, 999, 999]);
        let at = |x: i64, y: i64| -> Option<usize> {
            let (bx, by) = (x + ox, y + oy);
            (bx >= 0 && bx < wl && by >= 0 && by < hl && x >= clip[0] && x < clip[2] && y >= clip[1] && y < clip[3]).then(|| ((by * wl + bx) * 4) as usize)
        };
        let mut own = vec![0u8; layer.len()];
        let target = if fade < 32 { &mut own } else { &mut layer };
        // a glyph or sprite: its coverages laid on at its place (in its own colours, if it has them)
        let sprite = |target: &mut Vec<u8>, gl: &Glyph, x: i64, y: i64, rgb: [u8; 3], fade: i64| {
            for row in 0..gl.h { for col in 0..gl.w {
                let k = row * gl.w + col;
                let c = cover[gl.at + k];
                if c == 0 { continue }
                let rgb = gl.rgb.map_or(rgb, |o| [cover[o + 3 * k], cover[o + 3 * k + 1], cover[o + 3 * k + 2]]);
                if let Some(i) = at(x + col as i64, y + row as i64) { over(&mut target[i..i + 4], rgb, c, fade); }
            } }
        };
        for it in &g.items {
            match *it {
                Item::Panel { x, y, w: bw, h: bh, r, top, bottom, rgb } => {
                    let (x0, y0, bw, bh, r) = (x as i64 + dx, y as i64 + dy, value(bw) as i64, bh as i64, r as i64);
                    let rgb = rgb.map(|v| value(v) as u8);
                    for i in 0..bh {
                        let py = if top && i < r { Some(r - 1 - i) } else if bottom && i >= bh - r { Some(i - (bh - r)) } else { None };
                        for k in 0..bw {
                            let c = match py {
                                Some(py) if k < r => corner(r, r - 1 - k, py),
                                Some(py) if k >= bw - r => corner(r, k - (bw - r), py),
                                _ => 32,
                            };
                            if c == 0 { continue }
                            if let Some(p) = at(x0 + k, y0 + i) { over(&mut target[p..p + 4], rgb, c, 32); }
                        }
                    }
                }
                Item::Shadow { x, y, w: sw, h: sh, blur, level } => {
                    let (x0, y0, b) = (x as i64 + dx, y as i64 + dy, blur as i64);
                    let n = 2 * b + 1;
                    let ramp = |s: i16, k: i64| value(ease(0, s, (k + 1) as usize, n as usize));
                    // how far into an edge a place is: (k, true) on its soft part, else the middle
                    let edge = |t: i64, size: i64| if t < 2 * b { Some(t) } else if t >= size { Some(size + 2 * b - 1 - t) } else { None };
                    for t in 0..sh as i64 + 2 * b {
                        let s = match edge(t, sh as i64) { Some(k) => ramp(level, k), None => level };
                        for u in 0..sw as i64 + 2 * b {
                            let c = match edge(u, sw as i64) { Some(k) => ramp(s, k), None => s };
                            if c == 0 { continue }
                            if let Some(p) = at(x0 - b + u, y0 - b + t) { over(&mut target[p..p + 4], [0, 0, 0], c as u8, 32); }
                        }
                    }
                }
                Item::Text { x, y, face, across, middle, rgb, says } => {
                    let codes: Vec<u32> = match says {
                        Says::Str(s) => s.chars().map(|c| c as u32).collect(),
                        Says::Num(v, suffix) => format!("{}{suffix}", value(v)).chars().map(|c| c as u32).collect(),
                        Says::Typed { text, typed, cursor } => text.chars().take(typed).map(|c| c as u32).chain(cursor.then_some(CURSOR_CODE)).collect(),
                    };
                    let gs: Vec<&Glyph> = codes.iter().map(|&c| &glyphs[&(face, c)]).collect();
                    let rgb = rgb.map(|v| value(v) as u8);
                    let width: i64 = gs.iter().map(|g| g.adv).sum();
                    let mut pen = x as i64 + dx - match across { Across::Left => 0, Across::Middle => width / 2, Across::Right => width };
                    let top = y as i64 + dy - if middle { atlas.mid[face] } else { 0 };
                    for gl in gs { sprite(target, gl, pen + gl.ox, top + gl.oy, rgb, 32); pen += gl.adv; }
                }
                Item::Sprite { x, y, code, rgb, fade } => {
                    let gl = &glyphs[&(9, code as u32)];
                    sprite(target, gl, x as i64 + dx + gl.ox, y as i64 + dy + gl.oy, rgb.map(|v| value(v) as u8), value(fade) as i64);
                }
                Item::Emoji { x, y, code, fade } => {
                    let gl = &glyphs[&(EMOJI, code)];
                    sprite(target, gl, x as i64 + dx + gl.ox, y as i64 + dy + gl.oy, [0; 3], value(fade) as i64);
                }
                Item::Spinner { x, y, phase, fade } => {
                    for k in 0..12i64 {
                        let grey = (70 + (k - phase as i64).rem_euclid(12) * 185 / 11) as u8;
                        let gl = &glyphs[&(9, (SPOKE as i64 + k) as u32)];
                        sprite(target, gl, x as i64 + dx + gl.ox, y as i64 + dy + gl.oy, [grey; 3], value(fade) as i64);
                    }
                }
            }
        }
        if fade < 32 {
            for k in 0..own.len() / 4 {
                if own[4 * k + 3] > 0 { let rgb = [own[4 * k], own[4 * k + 1], own[4 * k + 2]]; over(&mut layer[4 * k..4 * k + 4], rgb, own[4 * k + 3], fade); }
            }
        }
    }
    layer
}
