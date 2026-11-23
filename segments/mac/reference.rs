// Mac reference: the cold open, 377 frames of a Mac Classic's 512x342 one-bit screen, drawn in ordinary Rust
// from the formulas, independently of the machines: the desk as desk.rs describes each frame (its places a window's
// corner and a distance, added), the text by ui/text.rs from the atlas's Mac faces, the sprites (data/sprites.txt),
// the zoom rects, the pointer's glides and straight moves; the game, the Wikipedia page's sphere and the Finnish flag
// (reference_boom.rs), the game gone haywire (data/haywire.txt's schedule, the pixels moved as the prototype's numpy
// moves them); then the one-bit tunnel at 128x86 blocks of 4x4 and the titles dissolving in over it. Writes the
// frames as RGB (black 0, white 255). Args: the demo folder.
//
// Whole numbers, rounded as the machines round:
// - A zoom rect's edge at step j of n: from + (to - from) j / n, the product's size rounded down (toward `from`).
// - The pointer's glide, step j of n from (x0, y0) by (dx, dy), m = n - j: c(v) = v m / n three times over, each
//   toward zero (v (1 - u), u = 1 - (m / n)^3, the ease out); the bow b(v) = (c(v) - c(c(v))) 32 / 100 toward zero
//   (0.32 u (1 - u) v); the point x0 + dx - c(dx) - b(dy), y0 + dy - c(dy) + b(dx). The prototype rounds the
//   floating point to the nearest pixel: within a pixel or two of it.
// - The pointer's straight move, step j of n from a to b: a + (b - a) j / n each way, the product's size rounded down
//   (toward a), as a zoom rect's edge. The prototype rounds to the nearest (halves to even).
// - The hall of mirrors at frame f (HAYWIRE to BOMB), from data/haywire.txt: a clean frame is the game's own; else
//   the game's own picture with its first BANDS bands spoiled in order, band j by j mod 3: 0 the picture before
//   (the one frame f - 1 showed) moved SHIFT columns right, wrapping; 1 the band's first column across it; 2 the
//   view from the same place half a turn round (no muzzle flash); then the first TEARS strips each moved its DX
//   across, wrapping; the status bar the junk one where the counter is junk (its digits, 100%, 50%, the face calm,
//   ahead). From BOMB to RESUME the picture of BOMB - 1 is held.
// - The tunnel's block (ix, iy): its offset from the middle in half pixels, dx = 8 ix - 507, dy = 8 iy - 337 (odd,
//   never 0); the slope step r = 128 shorter / longer rounded down; the angle A around (800 a turn) from the slope
//   table, by octant; the distance R2 = longer + c longer / 100 / 2 (c = 200 (sec - 1) from the table; each
//   rounded down); along, twist and the two shades from the ring table at R2. Picture j (t = j / 4 s): the turn
//   T = 30 sin(0.225 j) and the depth twist's weight s = 16 sin(0.15 j), each from the sine table (100 sin(0.075 i))
//   times 30 or 16 over 100 toward zero; the travel 11 j mod 40. Around: u = A + T + twist s / 8 (toward zero),
//   brought into 0..799; a check is 50 of u and 20 of along + travel; light where the two check numbers are both even
//   or both odd, else dark.
// - Atkinson's dither, row by row: each block's value (its shade, 256 white, plus what it was passed) is black under
//   128; its error (value, or value less 256) over 8, toward zero, is passed to the next two along, the three under
//   it and the one two under. The prototype's in floating point: the same picture, not the same dots.
// - The titles: a pixel of a card shows at j thirds when its turn (dissolve.bin) is under 85 j (exactly the
//   prototype's r < j / 3); its rim black, its letters white.
use std::collections::HashMap;
use std::io::Write;

#[path = "../ui/pages.rs"]
#[allow(dead_code)]
mod pages;
#[path = "../ui/text.rs"]
mod text;
#[path = "desk.rs"]
mod desk;
use desk::*;
#[path = "boom.rs"]
mod boom;
#[path = "reference_boom.rs"]
mod reference_boom;

fn base(rect: Rect, at: At) -> i64 {
    let [l, t, r, b] = rect.map(|v| v as i64);
    match at { At::L(d) => l + d as i64, At::T(d) => t + d as i64, At::R(d) => r + d as i64, At::B(d) => b + d as i64, At::Mid(d) => (l + r) / 2 + d as i64 }
}

/// A one-bit picture, w x h (the screen, or the game's frame), black true.
struct Screen { px: Vec<bool>, w: usize, h: usize }

impl Screen {
    fn set(&mut self, x: i64, y: i64, v: bool) {
        if x >= 0 && y >= 0 && x < self.w as i64 && y < self.h as i64 { self.px[y as usize * self.w + x as usize] = v; }
    }
    /// A filled rectangle, its edges' pixels included, kept to the screen.
    fn fill(&mut self, l: i64, t: i64, r: i64, b: i64, v: bool) {
        for y in t.max(0)..=b.min(self.h as i64 - 1) { for x in l.max(0)..=r.min(self.w as i64 - 1) { self.px[y as usize * self.w + x as usize] = v; } }
    }
    /// A picture worked out, its corner at (x, y): all its pixels, or (ink) only its black ones.
    fn picture(&mut self, pic: &[bool], w: usize, x: i64, y: i64, ink: bool) {
        for j in 0..pic.len() / w { for i in 0..w {
            if !ink || pic[j * w + i] { self.set(x + i as i64, y + j as i64, pic[j * w + i]); }
        } }
    }
    fn frame(&mut self, l: i64, t: i64, r: i64, b: i64) {
        self.fill(l, t, r, t, true); self.fill(l, b, r, b, true); self.fill(l, t, l, b, true); self.fill(r, t, r, b, true);
    }
    fn sprite(&mut self, s: &Sprite, x: i64, y: i64) {
        for j in 0..s.h { for i in 0..s.w {
            match s.px[j * s.w + i] { 1 => self.set(x + s.ox as i64 + i as i64, y + s.oy as i64 + j as i64, false), 2 => self.set(x + s.ox as i64 + i as i64, y + s.oy as i64 + j as i64, true), _ => {} }
        } }
    }
    /// The Finder's zoom rect: an XOR outline, kept to the screen.
    fn invert_frame(&mut self, l: i64, t: i64, r: i64, b: i64) {
        let (l, t, r, b) = (l.max(0), t.max(0), r.min(W as i64 - 1), b.min(H as i64 - 1));
        let mut flip = |x: i64, y: i64| { let i = y as usize * W + x as usize; self.px[i] = !self.px[i]; };
        for x in l..=r { flip(x, t); }
        if b != t { for x in l..=r { flip(x, b); } }
        for y in t + 1..b { flip(l, y); }
        if r != l { for y in t + 1..b { flip(r, y); } }
    }
}

/// The glide's point at step j of n.
fn glide(x0: i64, y0: i64, x1: i64, y1: i64, step: i64, steps: i64) -> (i64, i64) {
    let m = steps - step;
    let c = |v: i64| (0..3).fold(v, |v, _| v * m / steps);
    let (dx, dy) = (x1 - x0, y1 - y0);
    let bow = |v: i64| (c(v) - c(c(v))) * 32 / 100;
    (x0 + dx - c(dx) - bow(dy), y0 + dy - c(dy) + bow(dx))
}

fn main() {
    let demo = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let glyphs = text::Glyphs::read(&demo);
    let sprites = sprites(&demo);
    let tabs = tables(&demo);
    let cards = titles(&demo);
    let turn = dissolve(&demo);

    let shots = script();
    let bdata = boom::data(&demo);
    let trig = reference_boom::Trig::read(&demo);
    let sphere = reference_boom::sphere();
    let (flag_w, flag) = reference_boom::flag();
    // the game's frames as each shows them: the view, the gun over it, the status bar under it; gone haywire from
    // HAYWIRE, each from the one before
    let hw = boom::haywire(&demo);
    let mut shown: Vec<Vec<bool>> = vec![];
    for f in 0..boom::BOMB {
        use boom::*;
        let (view, (gx, gy)) = reference_boom::game_view(&bdata, &trig, f);
        let mut s = Screen { px: vec![false; VW * (VH + SBH)], w: VW, h: VH + SBH };
        s.px[..VW * VH].copy_from_slice(&view);
        let (kick, flash) = kick_flash(f);
        s.h = VH;
        s.sprite(&sprites[&format!("gun_{kick}_{flash}")], gx, gy);
        s.h = VH + SBH;
        let bar = VH as i64;
        let status = |s: &mut Screen, digits: &[(i64, i64)], face: &str| {
            s.sprite(&sprites["status_base"], 0, bar);
            for &(d, x) in digits { s.sprite(&sprites[&format!("big_{d}")], x, bar + 7); }
            for (name, x, y) in FIXED_DIGITS { s.sprite(&sprites[name], x, bar + y); }
            s.sprite(&sprites[face], FACE_AT.0, bar + FACE_AT.1);
        };
        let ammo = 50 - fired(f);
        status(&mut s, &[(ammo / 10, AMMO_AT[0].0), (ammo % 10, AMMO_AT[1].0)], &face(f));
        if f < HAYWIRE || hw.frames[f - HAYWIRE].clean { shown.push(s.px); continue }
        let spoil = &hw.frames[f - HAYWIRE];
        let behind = reference_boom::behind_view(&bdata, &trig, f);
        let last = &shown[f - 1];
        let mut out = s.px.clone();
        for (j, &(x, w)) in hw.bands.iter().take(spoil.bands).enumerate() {
            let cols = x..(x + w).min(VW);
            for y in 0..VH {
                let row = &mut out[y * VW..(y + 1) * VW];
                match j % 3 {
                    0 => { let mut moved = last[y * VW..(y + 1) * VW].to_vec(); moved.rotate_right(spoil.shift); row[cols.clone()].copy_from_slice(&moved[cols.clone()]); }
                    1 => { let v = row[x]; row[cols.clone()].fill(v); }
                    _ => row[cols.clone()].copy_from_slice(&behind[y * VW + cols.start..y * VW + cols.end]),
                }
            }
        }
        for &(y0, dx) in hw.tears.iter().take(spoil.tears) {
            for y in y0..y0 + TEAR { out[y * VW..(y + 1) * VW].rotate_right(dx.rem_euclid(VW as i64) as usize); }
        }
        if let Some(junk) = &spoil.junk {
            let mut b = Screen { px: vec![false; VW * (VH + SBH)], w: VW, h: VH + SBH };
            let at = if junk.len() == 3 { JUNK_AT[0] } else { JUNK_AT[1] };
            let digits: Vec<(i64, i64)> = junk.iter().zip(at).map(|(&d, x)| (d as i64, x)).collect();
            status(&mut b, &digits, JUNK_FACE);
            out[VW * VH..].copy_from_slice(&b.px[VW * VH..]);
        }
        shown.push(out);
    }

    // the tunnel's blocks: the angle around, and the ring's (along, twist, light, dark)
    let mut blocks = vec![];
    for iy in 0..TH as i64 { for ix in 0..TW as i64 {
        let (dx, dy) = (8 * ix - 507, 8 * iy - 337);
        let (longer, shorter, steep) = if dx.abs() < dy.abs() { (dy.abs(), dx.abs(), true) } else { (dx.abs(), dy.abs(), false) };
        let r = 128 * shorter / longer;
        let [angle, c] = tabs.slope[r as usize].map(|v| v as i64);
        let r2 = longer + c * longer / 100 / 2;
        let theta = if steep { 200 - angle } else { angle };
        let a = match (dx < 0, dy < 0) { (false, false) => theta, (true, false) => 400 - theta, (true, true) => 400 + theta, (false, true) => (800 - theta) % 800 };
        blocks.push((a, tabs.ring[r2 as usize].map(|v| v as i64)));
    } }
    let mut pictures: HashMap<i16, Vec<bool>> = HashMap::new();
    let mut tunnel = |j: i16| -> Vec<bool> {
        pictures.entry(j).or_insert_with(|| {
            let j = j as usize;
            let turn = 30 * tabs.sine[3 * j] as i64 / 100;
            let weight = 16 * tabs.sine[2 * j] as i64 / 100;
            let travel = (11 * j as i64) % 40;
            let mut value: Vec<i64> = blocks.iter().map(|&(a, [along, twist, light, dark])| {
                let mut u = a + turn + twist * weight / 8;
                if u < 0 { u += 800; }
                if u >= 800 { u -= 800; }
                if (u / 50) % 2 == ((along + travel) / 20) % 2 { light } else { dark }
            }).collect();
            // Atkinson's dither
            let mut black = vec![false; TW * TH];
            for y in 0..TH { for x in 0..TW {
                let old = value[y * TW + x];
                black[y * TW + x] = old < 128;
                let share = (if old < 128 { old } else { old - 256 }) / 8;
                for (ddx, ddy) in [(1i64, 0usize), (2, 0), (-1, 1), (0, 1), (1, 1), (0, 2)] {
                    let (nx, ny) = (x as i64 + ddx, y + ddy);
                    if nx >= 0 && nx < TW as i64 && ny < TH { value[ny * TW + nx as usize] += share; }
                }
            } }
            black
        }).clone()
    };

    let mut out = std::io::stdout().lock();
    for shot in &shots {
        let mut s = Screen { px: vec![false; W * H], w: W, h: H };
        if let Some(t) = shot.tunnel {
            let pic = tunnel(t.picture);
            for y in 0..H { for x in 0..W {
                let mut v = pic[(y / BLOCK) * TW + x / BLOCK];
                if let Some((card, k, n)) = t.title {
                    let thirds = desk::thirds(k, n);
                    let c = cards[card][(y / BLOCK) * TW + x / BLOCK];
                    if c > 0 && (turn[y * W + x] as i16) < 85 * thirds { v = c == 1; }
                }
                s.px[y * W + x] = v;
            } }
        } else {
            // the desktop's 50% grey
            for y in 0..H { for x in 0..W { s.px[y * W + x] = (x + y) % 2 == 0; } }
            for layer in &shot.layers {
                let at = |a: At| base(layer.rect, a);
                for item in &layer.items {
                    match item {
                        Item::Fill(l, t, r, b, v) => s.fill(at(*l), at(*t), at(*r), at(*b), *v),
                        Item::Frame(l, t, r, b) => s.frame(at(*l), at(*t), at(*r), at(*b)),
                        Item::Sprite(name, x, y) => s.sprite(&sprites[*name], at(*x), at(*y)),
                        Item::Clip(name, x, y, foot) => { let h = s.h; s.h = (at(*foot) + 1) as usize; s.sprite(&sprites[*name], at(*x), at(*y)); s.h = h; }
                        Item::Picture(pic, x, y) => match pic {
                            Pic::Game(f) => s.picture(&shown[*f], boom::VW, at(*x), at(*y), false),
                            Pic::Sphere => s.picture(&sphere, 2 * boom::BALL_R as usize + 1, at(*x), at(*y), true),
                            Pic::Flag => s.picture(&flag, flag_w, at(*x), at(*y), false),
                        },
                        Item::Text(tx) => {
                            // the glyphs: (face, code) or a sprite; and every piece's advance
                            let mut parts: Vec<(Option<(usize, u32)>, Option<&str>, i64)> = vec![];
                            for p in &tx.pieces {
                                match p {
                                    Piece::Str(face, st) => for ch in st.chars() { let c = code(ch); parts.push((Some((*face, c)), None, glyphs.advance(*face, c))); },
                                    Piece::Gap(g) => parts.push((None, None, *g as i64)),
                                    Piece::Sprite(name, adv) => parts.push((None, Some(*name), *adv as i64)),
                                }
                            }
                            let advances: Vec<i64> = parts.iter().map(|p| p.2).collect();
                            let start = text::line_start(at(tx.x), &advances, match tx.align { Align::Start => 0, Align::Centre => 1, Align::End => 2 });
                            let base = at(tx.y);
                            let end = start + advances.iter().sum::<i64>();
                            if let Some(([l, t, r, b], v)) = tx.back { s.fill(start - l as i64, base + t as i64, end + r as i64, base + b as i64, v); }
                            if !tx.glyphs { continue }
                            let mut pen = start;
                            for (g, sp, adv) in parts {
                                if let Some((face, c)) = g { glyphs.lay_bits(&mut s.px, W, face, c, pen, base, tx.black); }
                                if let Some(name) = sp { s.sprite(&sprites[name], pen, base); }
                                pen += adv;
                            }
                            if tx.caret { s.sprite(&sprites["caret"], pen, base); }
                        }
                    }
                }
            }
            for z in &shot.zoom {
                let e: Vec<i64> = (0..4).map(|k| {
                    let (a, b) = (base(z.from.0, z.from.1[k]), base(z.to.0, z.to.1[k]));
                    let part = (b - a).abs() * z.step as i64 / z.steps as i64;
                    if b < a { a - part } else { a + part }
                }).collect();
                s.invert_frame(e[0], e[1], e[2], e[3]);
            }
        }
        for c in ["corner_tl", "corner_tr", "corner_bl", "corner_br"] {
            let (x, y) = match c { "corner_tl" => (0, 0), "corner_tr" => (W as i64 - 1, 0), "corner_bl" => (0, H as i64 - 1), _ => (W as i64 - 1, H as i64 - 1) };
            s.sprite(&sprites[c], x, y);
        }
        if let Some((name, mouse)) = &shot.cursor {
            let p = |p: &Point| (base(p.rect, p.x), base(p.rect, p.y));
            let (x, y) = match mouse {
                Mouse::Still(at) => p(at),
                Mouse::Glide { from, to, step, steps } => { let (a, b) = (p(from), p(to)); glide(a.0, a.1, b.0, b.1, *step as i64, *steps as i64) }
                Mouse::Line { from, to, step, steps } => {
                    let (a, b) = (p(from), p(to));
                    let go = |a: i64, b: i64| { let part = (b - a).abs() * *step as i64 / *steps as i64; if b < a { a - part } else { a + part } };
                    (go(a.0, b.0), go(a.1, b.1))
                }
            };
            s.sprite(&sprites[name.as_str()], x, y);
        }
        let rgb: Vec<u8> = s.px.iter().flat_map(|&b| { let v = if b { 0 } else { 255 }; [v, v, v] }).collect();
        out.write_all(&rgb).unwrap();
    }
}
