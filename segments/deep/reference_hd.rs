// The dive's creatures at 1536x768: the reference for the dive's second video, a frame for each of
// the dive's. Its water is the dive's own picture before the fish, the marine snow and the jellyfish (the seabed, the
// kelp and the seagrass, 192x96), each pixel made 8 x 8; over it, drawn sharp through the same camera and the same
// creatures' places: the fish from the atlas, the marine snow as tiny rising bubbles, the jellyfish (their bells from
// the bell atlas, seen from three heights, their tentacles and oral arms as soft strands, their halo). Each
// creature is hidden where the dive's seabed, kelp or seagrass is nearer (its depth, 8 x 8 pixels a sample).
//
// The look is the HD preview's (prototypes): its floating point made
// whole numbers. Every sum, product and division below is one the machines do; a creature's numbers (where it is on
// the screen, which way it faces, how big, its strands' points) are worked out exactly (the swarm's sums: products
// of cards, divisions rounded down, square roots and angles from its printed tables), and its pixels a row at a time
// by walks that add a step per pixel (thousandths). Where it differs from the preview:
// - The fish: the atlas cell nearest the fish's yaw (13), elevation (2) and swim phase (8), at the atlas's size
//   nearest above the size it is drawn (nine sizes, each smaller by the square root of 2), each pixel its nearest
//   texel; the swim phase goes by the fish's number in its school (the script's), not by tracking it.
// - The jellyfish: the bell from the bell atlas (made by data/make_hd.py from the preview's own ray cast, seen from far
//   off: from 30 degrees below to 60 above every 15, the two nearest mixed, at eight sizes of its pulse); its canals
//   don't turn with the jellyfish; off the middle of the screen it is drawn longer away from the middle, as the
//   preview's perspective camera saw it. The tentacles and oral arms are each drawn as 24 (20) straight pieces with
//   the glow across them falling off as the preview's blur does; the arms as three strands each where the preview
//   drew seven; a piece shorter than its own width is left out. Each piece's glow reaches half a piece past either
//   end, fading, so that the pieces' add up along the strand. The halo is worked out every 4 x 4 pixels, in 256ths.
// - The fade-in: 3 s from the first frame the dive shows a jellyfish, by its number in the script.
// - The bubbles: a sprite at the nearest of 60 sizes, placed to a quarter of a pixel (up to 8 pixels across).
use std::collections::HashMap;

pub const HW: usize = 1536;
pub const HH: usize = 768;
const TILE: i64 = 4;

fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }
fn isqrt(n: i64) -> i64 { let mut r = (n as f64).sqrt() as i64; while r * r > n { r -= 1; } while (r + 1) * (r + 1) <= n { r += 1; } r }
/// The sums' bound: every number worked out is under a billion in size (three cards), a divisor under 100000.
fn b(v: i64) -> i64 { assert!(v.abs() < 999_000_000, "past three cards: {}", v); v }
fn q(n: i64, d: i64) -> i64 { assert!(d > 0 && d <= 99_999, "a divisor {}", d); let v = fl(b(n), d); assert!(v.abs() <= 999_999, "a quotient {}", v); v }
/// fl(1000 n / d): the long division with a card of zeros more.
fn q3(n: i64, d: i64) -> i64 { assert!(d > 0 && d <= 99_999, "a divisor {}", d); let v = fl(1000 * b(n), d); assert!(v.abs() <= 999_999, "a quotient {}", v); v }
fn card(v: i64) -> i64 { assert!(v.abs() <= 999, "past a card: {}", v); v }

/// a + (b - a) k / 32, toward zero (the dive's mix).
pub fn mix32(a: i64, b: i64, k: i64) -> i64 { a + (b - a) * k / 32 }
/// v t / 100 by v's three digits, each part rounded down by itself: 100 a t / 100 + 10 b t / 100 + c t / 100.
fn pct(v: i64, t: i64) -> i64 { let v = card(v); (v / 100) * t + (v / 10 % 10) * t / 10 + (v % 10) * t / 100 }
/// The dive's v t / 100 by v's digits, ((c t / 10 + b t) / 10 + a t).
fn hundredth(v: i64, t: i64) -> i64 { let s = v.abs(); let h = ((s % 10) * t / 10 + (s / 10 % 10) * t) / 10 + (s / 100) * t; if v < 0 { -h } else { h } }

/// The swarm's printed tables (swarm/data/tables.txt): 999 sin every thousandth of a turn, atan every 2000th.
pub struct Trig { trig: Vec<i64>, atan: Vec<i64> }
impl Trig {
    pub fn load(path: &str) -> Trig {
        let text = std::fs::read_to_string(path).unwrap();
        let get = |name: &str| -> Vec<i64> { text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap().split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
        Trig { trig: get("trig"), atan: get("atan") }
    }
    /// 999 sin of an angle (millionths of a turn).
    pub fn sin(&self, a: i64) -> i64 {
        let t = fl(md(a, 1_000_000), 100);
        let qd = t / 2500;
        let mut r = t % 2500;
        if qd % 2 == 1 { r = 2500 - r; }
        let (k, f) = ((r / 10) as usize, r % 10);
        let v = if f == 0 { self.trig[k] } else { self.trig[k] + (self.trig[k + 1] - self.trig[k]) * f / 10 };
        if qd >= 2 { -v } else { v }
    }
    fn cos(&self, a: i64) -> i64 { self.sin(a + 250_000) }
    /// The angle of (x, y) from the x axis, millionths of a turn, 0..999999.
    pub fn atan2(&self, y: i64, x: i64) -> i64 {
        assert!(x.abs() < 1_000_000 && y.abs() < 1_000_000);
        let (ax, ay) = (x.abs(), y.abs());
        if ax == 0 && ay == 0 { return 0; }
        let base = if ay <= ax { self.ratio(fl(ay * 100_000, ax)) } else { 250_000 - self.ratio(fl(ax * 100_000, ay)) };
        match (x >= 0, y >= 0) { (true, true) => base, (false, true) => 500_000 - base, (false, false) => 500_000 + base, (true, false) => md(1_000_000 - base, 1_000_000) }
    }
    fn ratio(&self, rho: i64) -> i64 { let (k, t) = ((rho / 50) as usize, rho % 50 / 5); if t == 0 { self.atan[k] } else { self.atan[k] + (self.atan[k + 1] - self.atan[k]) * t / 10 } }
}

/// The atlases and tables (data/hd.txt, the .bin files beside it).
pub struct Data {
    pub fish: Vec<u8>, pub fish_w: Vec<i64>, pub fish_h: Vec<i64>, pub fish_at: Vec<usize>,
    pub bell: Vec<u8>, pub bell_w: Vec<i64>, pub bell_at: Vec<usize>,
    pub bub: Vec<u8>, pub bub_at: Vec<usize>, pub bub_side: Vec<i64>, pub bub_phases: Vec<i64>, pub bub_size: Vec<i64>,
    pub orcas: Vec<super::orca::Mesh>, pub orca_haze: Vec<i64>,
    pub cover: Vec<i64>, pub pale_glow: Vec<i64>, pub pale_rim: Vec<i64>, pub strand: Vec<i64>, pub halo: Vec<i64>, pub fine: Vec<i64>, pub taper: Vec<i64>, pub arm_taper: Vec<i64>,
}
impl Data {
    pub fn load(hd_txt: &str) -> Data {
        let text = std::fs::read_to_string(hd_txt).unwrap();
        let get = |name: &str| -> Vec<i64> { text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap_or_else(|| panic!("no {}", name)).split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
        let dir = std::path::Path::new(hd_txt).parent().unwrap();
        let bin = |n: &str| std::fs::read(dir.join(n)).unwrap_or_else(|e| panic!("{}: {} (make it with segments/deep/data/make_hd.py)", n, e));
        let us = |v: Vec<i64>| v.into_iter().map(|x| x as usize).collect::<Vec<usize>>();
        Data { fish: bin("hd_fish.bin"), fish_w: get("fish_w"), fish_h: get("fish_h"), fish_at: us(get("fish_at")),
            bell: bin("hd_bell.bin"), bell_w: get("bell_w"), bell_at: us(get("bell_at")),
            bub: bin("hd_bubble.bin"), bub_at: us(get("bubble_at")), bub_side: get("bubble_side"), bub_phases: get("bubble_phases"), bub_size: get("bubble_size"),
            orcas: (0..12).map(|k| super::orca::read_mesh(dir.join(format!("orca_{}.txt", k)).to_str().unwrap())).collect(), orca_haze: get("orca_haze"),
            cover: get("cover"), pale_glow: get("pale_glow"), pale_rim: get("pale_rim"), strand: get("strand"), halo: get("halo"), fine: get("fine"), taper: get("taper"), arm_taper: get("arm_taper") }
    }
    /// A fish texel (red, green, blue, cover, flash) of level k, cell (row, phase), at (u, v); None off the cell.
    pub fn fish_texel(&self, k: usize, row: i64, phase: i64, u: i64, v: i64) -> Option<[i64; 5]> {
        let (w, h) = (self.fish_w[k], self.fish_h[k]);
        if u < 0 || v < 0 || u >= w || v >= h { return None; }
        let at = self.fish_at[k] + (((row * 8 + phase) * w * h + v * w + u) * 5) as usize;
        let t = &self.fish[at..at + 5];
        Some([t[0] as i64, t[1] as i64, t[2] as i64, t[3] as i64, t[4] as i64])
    }
    /// A bell texel (glow, rim; hundredths) of level k, cell, at (u, v); None off the cell.
    pub fn bell_texel(&self, k: usize, cell: i64, u: i64, v: i64) -> Option<[i64; 2]> {
        let w = self.bell_w[k];
        if u < 0 || v < 0 || u >= w || v >= w { return None; }
        let at = self.bell_at[k] + ((cell * w * w + v * w + u) * 2) as usize;
        Some([self.bell[at] as i64, self.bell[at + 1] as i64])
    }
    /// A bubble's texel (0..255) of size i, offset (sx, sy), at (u, v) in its box.
    pub fn bub_texel(&self, i: usize, sx: i64, sy: i64, u: i64, v: i64) -> i64 {
        let (side, ph) = (self.bub_side[i], self.bub_phases[i]);
        self.bub[self.bub_at[i] + ((sy * ph + sx) * side * side + v * side + u) as usize] as i64
    }
}

/// What the dive's frame gives the creatures.
pub struct Scene<'a> {
    pub n: i64,
    /// the camera's place (hundredths of a unit)
    pub cam: [i64; 3],
    pub f: [i64; 3], pub r: [i64; 3], pub u: [i64; 3],
    pub fish: Vec<([i64; 3], [i64; 3], i64, i64)>,
    pub fish_light: i64,
    pub far: [i64; 3],
    pub show: i64, pub snow: [i64; 2],
    pub specks: &'a [[i32; 3]],
    /// the jellyfish the dive drew, far ones first: how far ahead (the dive's, tenths), its fade (64ths), its numbers
    /// (place, turn, kind, radius, height, eight lengths) and the first frame the dive showed it
    pub jellies: Vec<(i64, i64, Vec<i64>, i64)>,
    pub wave: i64,
    pub sin60: &'a [i32],
    /// the dive's picture before the fish, snow and jellyfish (192x96) and its depths there (whole units)
    pub under: &'a [[i32; 3]], pub depth: &'a [i32],
}

/// A walk's place: thousandths, its whole part the texel (or the index).
fn whole(p: i64) -> i64 { fl(p, 1000) }

/// A point (thousandths of a tenth of a unit, from the camera) on the screen: the place in thousandths of a pixel
/// (across, down) and how far ahead in fiftieths of a tenth (A'), if it is at least 3 tenths ahead and no further
/// than 1.2 times that across or up.
fn project(p: [i64; 3], s: &Scene) -> Option<(i64, i64, i64)> {
    let dot = |v: [i64; 3]| b(p[0] * v[0] + p[1] * v[1] + p[2] * v[2]);
    let a = fl(dot(s.f), 2000);
    if a < 150 { return None; }
    let (bx, cy) = (fl(dot(s.r), 2000), fl(dot(s.u), 2000));
    if 5 * bx.abs() > 6 * a || 5 * cy.abs() > 6 * a { return None; }
    Some((768_000 + q3(800 * bx, a), 384_000 - q3(800 * cy, a), a))
}

/// A rotation for drawing a sprite: its cosine and sine (999ths) and the steps of the walk across the sprite's
/// texels (thousandths of a texel a pixel) at level texels s1000 across and t1000 down, mirrored (m = -1) or not.
fn steps(c: i64, sn: i64, s1000: i64, t1000: i64, m: i64) -> [i64; 4] {
    [fl(m * c * s1000, 999), fl(-m * sn * s1000, 999), fl(sn * t1000, 999), fl(c * t1000, 999)]
}

/// A sprite drawn through a walk: for each row from y0 to y1 and column from x0 to x1, the texel (u, v) under the
/// pixel's middle: the place at (x0, y0) and the steps across and down.
struct Walk { x0: i64, x1: i64, y0: i64, y1: i64, u0: i64, v0: i64, st: [i64; 4] }
impl Walk {
    /// The box round a middle (cx, cy, thousandths of a pixel) reaching R pixels, on the screen; the place at its
    /// first pixel's middle: (w0, h0) (thousandths) at the middle.
    fn new(cx: i64, cy: i64, reach: i64, w0: i64, h0: i64, st: [i64; 4]) -> Option<Walk> {
        let (px, py) = (fl(cx, 1000), fl(cy, 1000));
        let (x0, x1, y0, y1) = ((px - reach).max(0), (px + reach).min(HW as i64 - 1), (py - reach).max(0), (py + reach).min(HH as i64 - 1));
        if x0 > x1 || y0 > y1 { return None; }
        let (dx, dy) = (1000 * x0 + 500 - cx, 1000 * y0 + 500 - cy);
        let u0 = w0 + fl(b(dx * st[0]), 1000) + fl(b(dy * st[1]), 1000);
        let v0 = h0 + fl(b(dx * st[2]), 1000) + fl(b(dy * st[3]), 1000);
        Some(Walk { x0, x1, y0, y1, u0, v0, st })
    }
    /// Each pixel (x, y) and its texel.
    fn each(&self, mut f: impl FnMut(i64, i64, i64, i64)) {
        for y in self.y0..=self.y1 {
            let (mut u, mut v) = (self.u0 + (y - self.y0) * self.st[1], self.v0 + (y - self.y0) * self.st[3]);
            for x in self.x0..=self.x1 {
                f(x, y, whole(u), whole(v));
                u += self.st[0];
                v += self.st[2];
            }
        }
    }
}

/// The low-res pixel under a pixel.
fn low(x: i64, y: i64) -> usize { (y / 8) as usize * 192 + (x / 8) as usize }

pub fn frame(s: &Scene, d: &Data, t: &Trig, rgb: &mut Vec<[i64; 3]>) {
    // the water: the dive's picture before the creatures, each pixel 8 x 8
    for y in 0..HH { for x in 0..HW { let c = s.under[low(x as i64, y as i64)]; rgb[y * HW + x] = [c[0] as i64, c[1] as i64, c[2] as i64]; } }
    let depth = |x: i64, y: i64| (s.depth[low(x, y)] as i64).min(99);
    // the orcas, far out in the water (before the fish, snow and jellyfish, which are nearer)
    super::orca::frame(s.n, s.cam, s.r, s.u, s.f, s.far, &d.orcas, &d.orca_haze, t, rgb, &depth);
    fish(s, d, t, rgb, &depth);
    bubbles(s, d, t, rgb, &depth);
    for j in &s.jellies { jelly(s, d, t, j, rgb, &depth); }
}

fn fish(s: &Scene, d: &Data, t: &Trig, rgb: &mut [[i64; 3]], depth: &dyn Fn(i64, i64) -> i64) {
    const FISH_FOG: [i64; 50] = [0, 1, 2, 2, 3, 4, 5, 5, 6, 7, 7, 8, 9, 9, 10, 10, 11, 12, 12, 13, 13, 14, 14, 15, 15,
        15, 16, 16, 17, 17, 17, 18, 18, 19, 19, 19, 20, 20, 20, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23];
    // each fish's middle on the screen and how far ahead; far ones first
    let mut seen = vec![];
    for (e, &(h, tl, flash, id)) in s.fish.iter().enumerate() {
        let m: [i64; 3] = [0, 1, 2].map(|c| 500 * (h[c] + tl[c]));
        let hf = h[0] * s.f[0] + h[1] * s.f[1] + h[2] * s.f[2];
        let tf = tl[0] * s.f[0] + tl[1] * s.f[1] + tl[2] * s.f[2];
        let Some((cx, cy, a)) = project(m, s) else { continue };
        if a < 500 || hf < 300 || tf < 300 { continue; }
        seen.push((a, e, cx, cy, h, tl, flash, id));
    }
    seen.sort_by(|p, q| q.0.cmp(&p.0).then(p.1.cmp(&q.1)));
    // the light: 0.62 + 0.5 light (hundredths, at most 1.11)
    let lit = (62 + fl(50 * s.fish_light, 32)).min(111);
    for (a, _, cx, cy, h, tl, flash, id) in seen {
        let m2: [i64; 3] = [0, 1, 2].map(|c| h[c] + tl[c]);
        let (gx, gz) = (h[0] - tl[0], h[2] - tl[2]);
        assert!(gx.abs() <= 20 && gz.abs() <= 20, "a fish longer than 2 units");
        // which way it heads from the line of sight: across it (signed, to the screen's right) and away
        let side_r = m2[2] * s.r[0] - m2[0] * s.r[2];
        let flip = side_r < 0;
        let across0 = gx * m2[2] - gz * m2[0];
        let across = if flip { -across0 } else { across0 };
        let away = gx * m2[0] + gz * m2[2];
        let yaw = t.atan2(away, across.abs());
        let yi = md(fl(fl(24 * yaw + 6_500_000, 1000), 1000), 24);
        let mirror = across < 0;
        let level = isqrt(m2[0] * m2[0] + m2[2] * m2[2]);
        let above = 268 * level < -1000 * m2[1];
        // its roll: the screen's way of the sideways line
        let su0 = m2[2] * s.u[0] - m2[0] * s.u[2];
        let su = if flip { -su0 } else { su0 };
        let roll = t.atan2(su, side_r.abs());
        let (sn, c) = (t.sin(roll), t.cos(roll));
        let phase = md(s.n + fl(27 * id, 10), 8);
        let row = if above { 13 } else { 0 } + yi;
        // the atlas's size: the first at which a pixel takes less than a texel
        let k = (0..9).find(|&k| a * d.fish_w[k] * 7 < 5_280_000).unwrap();
        let s1000 = fl(a * d.fish_w[k] * 7, 5280);
        let t1000 = fl(a * d.fish_h[k] * 14, 5280);
        let st = steps(c, sn, s1000, t1000, if mirror { -1 } else { 1 });
        let reach = q(422_000, a) + 2;
        let Some(walk) = Walk::new(cx, cy, reach, 500 * d.fish_w[k], 500 * d.fish_h[k], st) else { continue };
        // the colour: lit, the flash, the fog
        let fog = FISH_FOG[fl(a, 500).min(49) as usize];
        let g = fl(lit * (32 - fog), 32);
        let kf = fl(55 * flash, 32);
        let fogc = s.far.map(|v| fl(v * fog, 32));
        let near = fl(a + 49, 50);
        walk.each(|x, y, u, v| {
            let Some(tx) = d.fish_texel(k, row, phase, u, v) else { return };
            if tx[3] == 0 { return; }
            let cover = if depth(x, y) * 10 >= near { tx[3] / 8 } else { 0 };
            let i = y as usize * HW + x as usize;
            for ch in 0..3 {
                let val = (pct(tx[ch] + pct(tx[4], kf), g) + fogc[ch]).min(255);
                rgb[i][ch] = mix32(rgb[i][ch], val, cover);
            }
        });
    }
}

fn bubbles(s: &Scene, d: &Data, t: &Trig, rgb: &mut [[i64; 3]], depth: &dyn Fn(i64, i64) -> i64) {
    const BUBBLE: [i64; 3] = [225, 242, 250];
    for (e, sp) in s.specks.iter().enumerate() {
        if e % 4 != 0 { continue; }
        let y = md(sp[1] as i64 - s.snow[0] + 3 * s.n + 70, 140) - 70;
        for copy in 0..3 {
            let z = md(sp[2] as i64 - s.snow[1] + 20, 140) - 20 + 140 * copy;
            let x = sp[0] as i64;
            // a little wobble as it rises: 0.4 sin(0.7 n + 0.37 x + 0.11 z) tenths
            let wob = fl(400 * t.sin(111_408 * s.n + 58_887 * x + 17_507 * z), 999);
            let Some((cx, cy, a)) = project([1000 * x + wob, 1000 * y, 1000 * z], s) else { continue };
            let a10 = fl(a, 50);
            if a10 >= 400 { continue; }
            let k32 = fl((400 - a10) * s.show, 400);
            let i = d.bub_size[a10 as usize] as usize;
            let (side, ph) = (d.bub_side[i], d.bub_phases[i]);
            let (px, py) = (fl(cx, 1000), fl(cy, 1000));
            let (sx, sy) = if ph == 4 { (fl(md(cx, 1000), 250), fl(md(cy, 1000), 250)) } else { (0, 0) };
            let near = fl(a + 49, 50);
            let h = side / 2;
            for v in 0..side { for u in 0..side {
                let (x, y) = (px - h + u, py - h + v);
                if x < 0 || y < 0 || x >= HW as i64 || y >= HH as i64 { continue; }
                let a8 = d.bub_texel(i, sx, sy, u, v);
                if a8 == 0 { continue; }
                let cover = if depth(x, y) * 10 >= near { fl(a8 * k32, 255).min(19) } else { 0 };
                let j = y as usize * HW + x as usize;
                for ch in 0..3 { rgb[j][ch] = mix32(rgb[j][ch], BUBBLE[ch], cover); }
            } }
        }
    }
}

/// A strand's piece, from p to q (thousandths of a pixel), its width (thousandths of a pixel) and strength: the
/// glow it adds (hundredths) to the pixels within three widths of it and half a piece past either end.
fn piece(p: (i64, i64), qq: (i64, i64), sigma: i64, strength: i64, kk: i64, d: &Data, glow: &mut HashMap<usize, (i64, i64)>) {
    let (dx, dy) = (fl(qq.0 - p.0, 100), fl(qq.1 - p.1, 100));
    if dx.abs() > 999 || dy.abs() > 999 { return; }
    let len = isqrt(dx * dx + dy * dy);
    // shorter than its width: left out (its neighbours' glow covers it)
    if 1000 * len < 10 * sigma { return; }
    let f = fl(fl(fl(strength * kk, 100), q(len * sigma, 1000).max(1)), 10).min(299);
    if f == 0 { return; }
    let (tx, ty) = (q(dx * 100_000, len), q(dy * 100_000, len));
    let (nx, ny) = (-ty, tx);
    // the box: the piece and three widths round it
    let reach = fl(3 * sigma, 1000) + 1;
    let (hx, hy) = (50 * dx, 50 * dy);
    let (xa, xb) = ((p.0 - hx).min(qq.0 + hx), (p.0 - hx).max(qq.0 + hx));
    let (ya, yb) = ((p.1 - hy).min(qq.1 + hy), (p.1 - hy).max(qq.1 + hy));
    let (x0, x1) = ((fl(xa, 1000) - reach).max(0), (fl(xb, 1000) + reach).min(HW as i64 - 1));
    let (y0, y1) = ((fl(ya, 1000) - reach).max(0), (fl(yb, 1000) + reach).min(HH as i64 - 1));
    if x0 > x1 || y0 > y1 { return; }
    let (dx0, dy0) = (fl(1000 * x0 + 500 - p.0, 100), fl(1000 * y0 + 500 - p.1, 100));
    // across: eighths of the width (thousandths); along: hundredths of the piece (thousandths)
    let a0 = 8 * q(b(dx0 * nx + dy0 * ny), sigma);
    let (ax, ay) = (8 * q(10 * nx, sigma), 8 * q(10 * ny, sigma));
    let l0 = q(b(dx0 * tx + dy0 * ty), len);
    let (lx, ly) = (q(10 * tx, len), q(10 * ty, len));
    for y in y0..=y1 {
        let (mut ac, mut al) = (a0 + (y - y0) * ay, l0 + (y - y0) * ly);
        for x in x0..=x1 {
            let (i, l) = (whole(ac), whole(al));
            let g = if (-24..24).contains(&i) && (-50..150).contains(&l) { d.strand[(if i >= 0 { i } else { -1 - i } * 200 + l + 50) as usize] } else { 0 };
            if g > 0 {
                let e = glow.entry(y as usize * HW + x as usize).or_insert((0, 0));
                e.0 = (e.0 + hundredth(f, g)).min(299);
            }
            ac += ax;
            al += lx;
        }
    }
}

fn jelly(s: &Scene, d: &Data, t: &Trig, j: &(i64, i64, Vec<i64>, i64), rgb: &mut [[i64; 3]], depth: &dyn Fn(i64, i64) -> i64) {
    let (ahead, fade1, jl, first) = (j.0, j.1, &j.2, j.3);
    // the fade: eased out of the dark, and in over 3 s from the first frame it showed
    let eased = fl(fl(5 * fade1, 2).min(64).pow(2), 64);
    let rise = fl(64 * (s.n - first + 1), 36).min(64);
    let fade = eased.min(rise);
    if fade == 0 { return; }
    let c = [jl[0], jl[1], jl[2]];
    let (turn, kind, rad, ht) = (jl[3], jl[4], jl[5], jl[6]);
    let lengths: Vec<i64> = jl[7..15].to_vec();
    let Some((cx, cy, a)) = project(c.map(|v| 1000 * v), s) else { return };
    let ppt = q(40_000_000, a);
    let glow = if kind == 1 { [220, 150, 255] } else { [140, 200, 255] };
    let pale = [230, 245, 255];
    let mut emit: HashMap<usize, (i64, i64)> = HashMap::new();
    // the bell: the atlas cells for its pulse and the two views nearest its height, turned as the screen's up is
    let pulse = (fl(7 * (rad - 85) + 15, 30)).clamp(0, 7);
    let level = isqrt(c[0] * c[0] + c[2] * c[2]);
    let em0 = t.atan2(-c[1], level);
    let em = if em0 >= 500_000 { em0 - 1_000_000 } else { em0 };
    // the views every 15 degrees (41667 millionths of a turn) from 30 below: the one below and the next, 32nds on
    let from = (em + 83_333).clamp(0, 6 * 41_667 - 1);
    let va = fl(from, 41_667);
    let (vb, w) = (va + 1, fl(32 * (from - 41_667 * va), 41_667));
    let up = t.atan2(-s.r[1], s.u[1]);
    let (sn, cs) = (t.sin(up), t.cos(up));
    let k = (0..13).find(|&k| a * d.bell_w[k] < 1_000_000).unwrap();
    let s1000 = fl(a * d.bell_w[k], 1000);
    // seen off the middle of the screen it is drawn longer away from the middle, as a perspective camera sees it:
    // by 1 / cos of how far off it is (the sprite's places taken back by inv, thousandths, along that way)
    let (ox, oy) = (fl(cx, 1000) - 768, fl(cy, 1000) - 384);
    let off = isqrt(ox * ox + oy * oy);
    let inv = q(800_000, isqrt(640_000 + off * off));
    let (ex, ey) = if off == 0 { (0, 0) } else { (q(1000 * ox, off), q(1000 * oy, off)) };
    let short = 1000 - inv;
    let (sxx, sxy, syy) = (1000 - fl(fl(short * ex, 1000) * ex, 1000), -fl(fl(short * ex, 1000) * ey, 1000), 1000 - fl(fl(short * ey, 1000) * ey, 1000));
    let r = steps(cs, sn, s1000, s1000, 1);
    let st = [fl(r[0] * sxx + r[1] * sxy, 1000), fl(r[0] * sxy + r[1] * syy, 1000), fl(r[2] * sxx + r[3] * sxy, 1000), fl(r[2] * sxy + r[3] * syy, 1000)];
    let reach = q(1000 * q(706_667, a), inv) + 2;
    if let Some(walk) = Walk::new(cx, cy, reach, 500 * d.bell_w[k], 500 * d.bell_w[k], st) {
        walk.each(|x, y, u, v| {
            let (Some(ta), Some(tb)) = (d.bell_texel(k, 7 * pulse + va, u, v), d.bell_texel(k, 7 * pulse + vb, u, v)) else { return };
            if ta == [0, 0] && tb == [0, 0] { return; }
            emit.insert(y as usize * HW + x as usize, (mix32(ta[0], tb[0], w), mix32(ta[1], tb[1], w)));
        });
    }
    // the tentacles: eight long ones as the dive waves them and sixteen fine ones between, 24 pieces each
    let sigma = (fl(150 * ppt, 1000)).max(420);
    for tn in 0..24i64 {
        let main = tn % 3 == 0;
        let li = (if main { tn / 3 } else { tn } % 8) as usize;
        let ln = lengths[li] * d.fine[tn as usize];
        let ang = fl(1_000_000 * (8 * tn + 3 * turn), 192);
        let (sa, ca) = (t.sin(ang), t.cos(ang));
        let ph = fl(19 * tn, 6);
        let pts: Vec<Option<(i64, i64)>> = (0..=24i64).map(|h| {
            let wv = 5 * s.sin60[md(2 * h + ph + s.wave, 60) as usize] as i64 * h;
            let rr = fl(rad * (18400 - 115 * h), 200);
            let p = [1000 * c[0] + fl(rr * ca, 999) + fl(sa * wv, 999), 1000 * c[1] + 2 * ht - fl(h * ln * 10, 24),
                1000 * c[2] + fl(rr * sa, 999) - fl(ca * wv, 999)];
            project(p, s).map(|(x, y, _)| (x, y))
        }).collect();
        let strength = if main { 75 } else { 32 };
        for h in 0..24 {
            if let (Some(p), Some(qq)) = (pts[h], pts[h + 1]) { piece(p, qq, sigma, strength * d.taper[h], 432, d, &mut emit); }
        }
    }
    // the oral arms: four frilled ribbons from under the middle, three strands each
    let arm_sigma = (fl(350 * ppt, 1000)).max(800);
    let longest = *lengths.iter().max().unwrap();
    let a1 = fl(rad * ppt, 1000);
    for an in 0..4i64 {
        let ang = fl(1_000_000 * (16 * an + 8 + turn), 64);
        let (sa, ca) = (t.sin(ang), t.cos(ang));
        for off in [-2i64, 0, 2] {
            let pts: Vec<Option<(i64, i64)>> = (0..=20i64).map(|i| {
                let wv = fl(1500 * t.sin(125_000 * i + 33_423 * s.wave + 159_155 * an), 999);
                let ra = fl(12 * rad * (10 + i), 10);
                let p = [1000 * c[0] + fl(ra * ca, 999) + fl(sa * wv, 999), 1000 * c[1] - fl(i * longest * 55, 2),
                    1000 * c[2] + fl(ra * sa, 999) - fl(ca * wv, 999)];
                let frill = 1000 + fl(400 * t.sin(318_310 * i + 159_155 * off), 999);
                let shift = fl(fl(a1 * frill, 1000) * (12 + i) * off, 6);
                project(p, s).map(|(x, y, _)| (x + shift, y))
            }).collect();
            let strength = if off == 0 { 23 } else { 16 };
            for i in 0..20 {
                if let (Some(p), Some(qq)) = (pts[i], pts[i + 1]) { piece(p, qq, arm_sigma, strength * d.arm_taper[i], 399, d, &mut emit); }
            }
        }
    }
    // the glow made a colour: toward the pale edge colour by how bright, covering by how bright, by the fade; hidden
    // where the seabed is more than 2 units nearer
    let col: Vec<[i64; 3]> = (0..=32).map(|k| [0, 1, 2].map(|ch| mix32(glow[ch], pale[ch], k))).collect();
    let mut px: Vec<(&usize, &(i64, i64))> = emit.iter().collect();
    px.sort();
    for (&i, &(e, edge)) in px {
        let (x, y) = ((i % HW) as i64, (i / HW) as i64);
        if depth(x, y) * 10 < ahead - 20 { continue; }
        let cover = fl(d.cover[e as usize] * fade, 128).min(31);
        let kp = (d.pale_glow[e as usize] + d.pale_rim[edge as usize]).min(32);
        for ch in 0..3 { rgb[i][ch] = mix32(rgb[i][ch], col[kp as usize][ch], cover); }
    }
    // the halo, every 4 x 4 pixels: 0.35 exp(-r^2 / 2 sigma^2), sigma 11200 / ahead pixels, out to 3 sigma
    let reach = fl(33_600, ahead) + 1;
    let (pxc, pyc) = (fl(cx, 1000), fl(cy, 1000));
    let (tx0, tx1) = ((fl(pxc - reach, TILE)).max(0), (fl(pxc + reach, TILE)).min(HW as i64 / TILE - 1));
    let (ty0, ty1) = ((fl(pyc - reach, TILE)).max(0), (fl(pyc + reach, TILE)).min(HH as i64 / TILE - 1));
    // hundredths of sigma (thousandths) across and down from the middle, at each 4 x 4's middle
    let step = q(1000 * TILE * ahead, 112);
    let u0 = q(b((1000 * TILE * tx0 + 2000 - cx) * ahead), 112);
    let v0 = q(b((1000 * TILE * ty0 + 2000 - cy) * ahead), 112);
    for ty in ty0..=ty1 {
        let vv = whole(v0 + (ty - ty0) * step);
        for tx in tx0..=tx1 {
            let uu = whole(u0 + (tx - tx0) * step);
            let Some(k) = gauss(uu, vv) else { continue };
            let strength = d.halo[k as usize];
            if strength == 0 { continue; }
            let (x, y) = (TILE * tx, TILE * ty);
            if depth(x, y) * 10 < ahead - 20 { continue; }
            let amount = fl(strength * fade, 40);
            if amount == 0 { continue; }
            for yy in y..y + TILE { for xx in x..x + TILE {
                let i = yy as usize * HW + xx as usize;
                for ch in 0..3 { rgb[i][ch] = mix256(rgb[i][ch], glow[ch], amount); }
            } }
        }
    }
}

/// The halo's place in its table: half the squared distance, u and v in tenths of its width (from hundredths, toward
/// zero), each square halved by itself; none past 31 tenths either way.
fn gauss(across: i64, up: i64) -> Option<i64> {
    let (u, v) = ((across / 10).abs(), (up / 10).abs());
    if u > 31 || v > 31 { return None; }
    Some(u * u / 2 + v * v / 2)
}

/// a + (b - a) k / 256, toward zero.
fn mix256(a: i64, b: i64, k: i64) -> i64 { a + (b - a) * k / 256 }
