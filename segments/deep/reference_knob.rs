// The dive's ending at 1536x768 (frames 228..352): a big, chunky old amp's volume knob on a
// craggy rock pillar, rough, worn and fouled, faintly glowing in the dark water, set to 9. The diver comes to a stop
// over it; it sticks twice, then one click turns it to 10; it powers up for a second and a half, crackling; lightning;
// a flash; and the dive cuts to the train. The design in whole numbers: every
// sum, product and division below is one the machines do (the swarm's sums for the few numbers a frame, small
// machines for the many pixels).
//
// Each frame of the ending, over the dive's finished picture:
// - the water's light: from frame 263 the open water (where the dive's depth is open) lit a little from above, by
//   rows of 4 (a table: exp(-y / 0.35 768) of 12, 22, 26), coming up over 30 frames (smoothstep, 32nds);
// - frames 282..328, the larger orca back across, right to left, 55 units beyond the dial and 21 below it (and up to 2
//   above that, by a sine), 46 units long, as the pair earlier (the kit's flat draw, hidden where the dive's seabed is
//   nearer), a dark shape: its body (2, 4, 7) mixed toward the picture by the haze looking into the lit water
//   (32 min(1, 0.85 (a / 120)^0.8));
// - the dial's glow on the water: round the dial's middle on the screen, every 4 x 4 pixels, e = (10 dx^2) / R + (22
//   dy^2) / R tenths (R twice the glow's radius squared, wider as it powers up), 256ths from the table (25.6
//   exp(-e)) times the glow, mixed toward (255, 210, 140);
// - the knob: three models drawn by the mesh kit, carrying each corner's place in the atlas across its triangles: the
//   rock (30 units to the dive's unit), the plate (100), the knob and its skirt (100, turned to its value); each in
//   turn over the last. Each pixel: how far ahead (whole units), hidden where the dive's seabed is nearer; its texel
//   (the atlas, by the page and place the kit carried); before the click, its colour as it is; from the click, the
//   dial's glow added (B times how much stronger it is than at 9, 32nds) and the ink lit (mixed toward the ink's
//   colour of its number this frame); then hazed by the dive's fog (32 min(1, 0.85 (a / 70)^0.8));
// - silt: two puffs of 18 blobs off the rock round the plate as the knob sticks (frames 314 and 323, 14 frames each),
//   every 4 x 4 pixels, each blob 256 exp(-e) (14 - age) / 28 256ths, summed (at most 141), mixed toward (92, 86, 72);
// - from the click, flakes of crust shaken loose: 140, each a square drifting out and up, mixed in by 27 (17 - age) / 17
//   32nds;
// - from the click, arcs off the rim: each a jagged line of 10 steps, turning at random, forking; drawn (forks 24
//   32nds of white, mains white), their glow every 4 x 4 pixels (by whole pixels along past a line's end and across
//   it, from the table: the design's blur), summed (at most 32), mixed toward (170, 210, 255); and the frame
//   flashed toward (225, 240, 255) (32nds).
// The random choices (the blobs', flakes' and arcs') are from knob.txt, random cards, the blobs' and arcs' printed
// already scaled to the numbers they stand for.

use super::hd::{mix32, Trig};
use std::collections::HashMap;

fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }
fn isqrt(n: i64) -> i64 { let mut r = (n as f64).sqrt() as i64; while r * r > n { r -= 1; } while (r + 1) * (r + 1) <= n { r += 1; } r }
/// The sums' bound: every number worked out is under a billion in size (three cards), a divisor under 100000.
fn b(v: i64) -> i64 { assert!(v.abs() < 999_000_000, "past three cards: {}", v); v }
fn q3(n: i64, d: i64) -> i64 { assert!(d > 0 && d <= 99_999, "a divisor {}", d); let v = fl(1000 * b(n), d); assert!(v.abs() <= 999_999, "a quotient {}", v); v }
/// a + (b - a) k / 256, toward zero.
fn mix256(a: i64, bb: i64, k: i64) -> i64 { a + (bb - a) * k / 256 }

pub const FIRST: i64 = 228;
pub const LAST: i64 = 352;
const CLICK: i64 = 331;
/// The dial's middle (hundredths of a unit), its frame (thousandths): across, up, along.
pub const KNOB: [i64; 3] = [-166, -10922, 32190];
pub const SIDE: [i64; 3] = [992, 0, 128];
pub const UP: [i64; 3] = [0, 1000, 0];
pub const FWD: [i64; 3] = [-128, 0, 992];
const INK: [i64; 3] = [255, 225, 160];
const PAINT: [i64; 3] = [58, 55, 45];
const WARM: [i64; 3] = [255, 210, 140];
const SILT: [i64; 3] = [92, 86, 72];
const PALE: [i64; 3] = [110, 104, 92];
const CRUST: [i64; 3] = [58, 52, 40];
const BLUE: [i64; 3] = [170, 210, 255];
const FLASH: [i64; 3] = [225, 240, 255];
const DARK: [i64; 3] = [2, 4, 7];
/// The orca's model axes in the world (thousandths): its x across its heading, y up, z back along it (it heads right
/// to left across the dial's frame, a little away).
const ORCA_AXES: [[i64; 3]; 3] = [[-9, 0, 1000], [0, 1000, 0], [1000, 0, 9]];
pub const ORCA_FIRST: i64 = 282;
pub const ORCA_LAST: i64 = 328;

/// A model with a place in the atlas at each corner (make_knob.py).
pub struct Mesh4 { pub v: Vec<[i64; 3]>, pub val: Vec<[i64; 3]>, pub t: Vec<[usize; 3]>, pub fnorm: Vec<[i64; 3]>, pub fd: Vec<i64> }

pub fn read_mesh4(path: &str) -> Mesh4 {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path, e));
    let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    let nv: usize = lines[1].split_whitespace().nth(1).unwrap().parse().unwrap();
    let nums = |l: &str| -> Vec<i64> { l.split_whitespace().take(7).map(|v| v.parse().unwrap()).collect() };
    let mut m = Mesh4 { v: vec![], val: vec![], t: vec![], fnorm: vec![], fd: vec![] };
    for l in &lines[2..2 + nv] { let x = nums(l); m.v.push([x[0], x[1], x[2]]); m.val.push([x[3], x[4], x[5]]); }
    let nt: usize = lines[2 + nv].split_whitespace().nth(1).unwrap().parse().unwrap();
    for l in &lines[3 + nv..3 + nv + nt] {
        let x = nums(l);
        m.t.push([x[0] as usize, x[1] as usize, x[2] as usize]);
        m.fnorm.push([x[3], x[4], x[5]]);
        m.fd.push(x[6]);
    }
    m
}

pub struct Data {
    pub rock: Mesh4, pub plate: Mesh4, pub knob: Mesh4,
    atlas: Vec<u8>, pages: Vec<(i64, i64, usize, i64, i64)>,
    turn: HashMap<i64, (i64, i64)>,
    haze120: Vec<i64>, lift: [Vec<i64>; 3], halo: Vec<i64>, puff: Vec<i64>, glow: [Vec<i64>; 3],
    silt: Vec<i64>, debris: Vec<i64>, shake: Vec<i64>, crackle: Vec<i64>,
}

impl Data {
    pub fn load(dir: &std::path::Path) -> Data {
        let text = std::fs::read_to_string(dir.join("knob.txt")).unwrap();
        let get = |name: &str| -> Vec<i64> { text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap_or_else(|| panic!("knob.txt: no {}", name)).split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
        let p = get("pages");
        let t = get("turn");
        let mesh = |n: &str| read_mesh4(dir.join(n).to_str().unwrap());
        Data { rock: mesh("knob_rock.txt"), plate: mesh("knob_plate.txt"), knob: mesh("knob_knob.txt"),
            atlas: std::fs::read(dir.join("knob.bin")).expect("deep/data/knob.bin (make it with make_knob.py)"),
            pages: p.chunks(5).map(|c| (c[0], c[1], c[2] as usize, c[3], c[4])).collect(),
            turn: t.chunks(3).map(|c| (c[0], (c[1], c[2]))).collect(),
            haze120: get("haze120"), lift: [get("lift0"), get("lift1"), get("lift2")], halo: get("halo"), puff: get("puff"),
            glow: [get("glow2"), get("glow4"), get("glow5")], silt: get("silt"), debris: get("debris"), shake: get("shake"), crackle: get("crackle") }
    }
    /// A texel (its place from its page's origin): S, C, B (rgb), ink (32nds), whose.
    fn texel(&self, page: i64, u: i64, v: i64) -> [i64; 11] {
        let (w, h, at, ou, ov) = self.pages[page as usize];
        let (u, v) = (u + ou, v + ov);
        assert!((0..w).contains(&u) && (0..h).contains(&v), "a texel off its page");
        let o = at + ((v * w + u) * 11) as usize;
        std::array::from_fn(|k| self.atlas[o + k] as i64)
    }
}

/// What the dive gives the ending: the frame's number, the camera (hundredths of a unit; its forward, right and up in
/// 300ths), the water's far colour, the dive's depths (192x96, 999 open) and the fog's table.
pub struct Scene<'a> {
    pub n: i64, pub cam: [i64; 3], pub f: [i64; 3], pub r: [i64; 3], pub u: [i64; 3], pub far: [i64; 3],
    pub depth: &'a [i32], pub haze70: &'a [i64],
}

/// A walk from a to b (either way round): the k-th of n places on from the lesser, the value there plus the change
/// to the other's times (k + 1) / (n + 1), cut toward zero (the kit's).
fn walk(a: i64, bb: i64, va: i64, vb: i64, k: i64) -> i64 {
    let (n, vl, vr) = if bb < a { (a - bb, vb, va) } else { (bb - a, va, vb) };
    vl + (vr - vl) * (k + 1) / (n + 1)
}

/// The kit's draw (focal length 800 pixels; each corner's three values carried across): each pixel the model covers
/// (its index in the 1536x768 frame), its depth less zref and the nearest piece's three values.
pub fn draw4(mesh: &Mesh4, r: [[i64; 3]; 3], p: [i64; 3], zref: i64) -> Vec<(usize, i64, [i64; 3])> {
    let (w, h) = (1536i64, 768i64);
    let corners: Vec<(i64, i64, i64)> = mesh.v.iter().map(|v| {
        let t: Vec<i64> = (0..3).map(|i| r[i][0] * v[0] + r[i][1] * v[1] + r[i][2] * v[2] + 1000 * p[i]).collect();
        let z = fl(t[2], 1000);
        (fl(80 * t[0], z), fl(-80 * t[1], z), z - zref)
    }).collect();
    let cam: [i64; 3] = [0, 1, 2].map(|j| fl(fl(r[0][j] * -p[0] + r[1][j] * -p[1] + r[2][j] * -p[2], 1000), 10));
    let front: Vec<usize> = (0..mesh.t.len()).filter(|&k| {
        let n = mesh.fnorm[k];
        (0..3).map(|j| n[j] * cam[j] / 100).sum::<i64>() > mesh.fd[k] - 6
    }).collect();
    for &k in &front { for &i in &mesh.t[k] { assert!((0..=999).contains(&corners[i].2), "a knob's depth of {}", corners[i].2); } }
    let vals = |i: usize| -> [i64; 4] { [corners[i].2, mesh.val[i][0], mesh.val[i][1], mesh.val[i][2]] };
    let mut edges: std::collections::BTreeMap<(usize, usize), Vec<(i64, i64, [i64; 4])>> = Default::default();
    for &k in &front {
        let t = mesh.t[k];
        for e in 0..3 {
            let (i, j) = (t[e], t[(e + 1) % 3]);
            let key = (i.min(j), i.max(j));
            if edges.contains_key(&key) { continue; }
            let (top, bot) = if corners[j].1 < corners[i].1 { (j, i) } else { (i, j) };
            let (a, bb) = (corners[top], corners[bot]);
            let (va, vb) = (vals(top), vals(bot));
            let first = fl(a.1 + 49, 100);
            let rows = fl(bb.1 + 49, 100) - first;
            let mut list = vec![];
            for jr in 0..rows.max(0) {
                let row = first + jr;
                if !(-384..384).contains(&row) { continue; }
                let x = a.0 + fl((bb.0 - a.0) * (100 * row + 50 - a.1), bb.1 - a.1);
                let px = fl(x + 49, 100).clamp(-768, 768);
                list.push((row, px, std::array::from_fn(|c| walk(0, rows, va[c], vb[c], jr))));
            }
            edges.insert(key, list);
        }
    }
    let mut best: std::collections::BTreeMap<usize, [i64; 4]> = Default::default();
    for &k in &front {
        let t = mesh.t[k];
        let mut by_row: std::collections::BTreeMap<i64, Vec<(i64, [i64; 4])>> = Default::default();
        for e in 0..3 {
            let (i, j) = (t[e], t[(e + 1) % 3]);
            for &(row, px, v) in &edges[&(i.min(j), i.max(j))] { by_row.entry(row).or_default().push((px, v)); }
        }
        for (row, c) in by_row {
            assert_eq!(c.len(), 2, "a triangle's row met {} edges", c.len());
            let ((pa, va), (pb, vb)) = (c[0], c[1]);
            let (lo, n) = (pa.min(pb), (pb - pa).abs());
            for kx in 0..n {
                let v: [i64; 4] = std::array::from_fn(|ch| walk(pa, pb, va[ch], vb[ch], kx));
                let x = lo + kx;
                if !(-w / 2..w / 2).contains(&x) { continue; }
                let at = ((row + h / 2) * w + x + w / 2) as usize;
                if best.get(&at).is_none_or(|bv| v[0] < bv[0]) { best.insert(at, v); }
            }
        }
    }
    best.into_iter().map(|(at, v)| (at, v[0], [v[1], v[2], v[3]])).collect()
}

/// A place from the camera (hundredths of a unit) on the screen: across and down (thousandths of a pixel) and how far
/// ahead (hundredths), if at least a unit ahead and no further across or up than 1.2 times that.
fn proj(rel: [i64; 3], s: &Scene) -> Option<(i64, i64, i64)> {
    let dot = |v: [i64; 3]| fl(b(rel[0] * v[0] + rel[1] * v[1] + rel[2] * v[2]), 300);
    let a = dot(s.f);
    if a < 100 { return None; }
    let (bx, cy) = (dot(s.r), dot(s.u));
    if 5 * bx.abs() > 6 * a || 5 * cy.abs() > 6 * a { return None; }
    Some((768_000 + q3(800 * bx, a), 384_000 - q3(800 * cy, a), a))
}

/// The low-res pixel under a pixel.
fn low(x: i64, y: i64) -> usize { (y / 8) as usize * 192 + (x / 8) as usize }

/// The frame's numbers.
struct Ctl { v100: i64, w: i64, p19: i64, g100: i64, mm: i64 }
fn ctl(n: i64, d: &Data) -> Ctl {
    let (v100, w) = if (308..=330).contains(&n) { d.turn[&n] } else if n < 308 { (900, 0) } else { (1000, 0) };
    let p19 = (n - CLICK).clamp(0, 19);
    let glow1000 = 550 + fl(45 * v100, 100);
    let g100 = fl(glow1000 * (190 + 16 * p19), 1900);
    Ctl { v100, w, p19, g100, mm: (5 * p19).min(76) }
}

pub fn frame(s: &Scene, d: &Data, t: &Trig, orcas: &[super::orca::Mesh], rgb: &mut [[i64; 3]]) {
    let n = s.n;
    if !(FIRST..=LAST).contains(&n) { return; }
    let raw = |x: i64, y: i64| s.depth[low(x, y)] as i64;
    let dep = |x: i64, y: i64| raw(x, y).min(99);
    let c = ctl(n, d);
    // the water's light beyond the dial
    if n >= 263 {
        let tt = (n - 262).min(30);
        let k32 = fl(32 * tt * tt * (90 - 2 * tt), 27000);
        for y in 0..768i64 { for x in 0..1536i64 {
            if raw(x, y) < 999 { continue; }
            let px = &mut rgb[(y * 1536 + x) as usize];
            for ch in 0..3 { px[ch] += fl(d.lift[ch][(y / 4) as usize] * k32, 32); }
        } }
    }
    // the orca back across
    if (ORCA_FIRST..=ORCA_LAST).contains(&n) { orca(s, d, t, orcas, rgb, &dep); }
    // the dial's glow on the water
    let rel_k = [0, 1, 2].map(|i| KNOB[i] - s.cam[i]);
    if let Some((cx, cy, a)) = proj(rel_k, s) {
        let rad = fl(588_800, a);
        // e = (10 dx^2 + 22 dy^2) / (2 rad^2 (1 + p)), in tenths: the squares in hundreds and thousands over that in
        // thousands
        let r2 = fl(b(2 * rad * rad * (190 + 16 * c.p19)), 190_000).max(1);
        let (ccx, ccy) = (fl(cx, 1000), fl(cy, 1000));
        let ex: Vec<i64> = (0..384).map(|tx| fl(fl((4 * tx + 2 - ccx) * (4 * tx + 2 - ccx), 100), r2).min(100)).collect();
        let ey: Vec<i64> = (0..192).map(|ty| fl(fl(22 * (4 * ty + 2 - ccy) * (4 * ty + 2 - ccy), 1000), r2).min(100)).collect();
        let amt: Vec<i64> = d.halo.iter().map(|&h| fl(h * c.g100, 100)).collect();
        for ty in 0..192usize { for tx in 0..384usize {
            let e = ex[tx] + ey[ty];
            if e > 99 || amt[e as usize] == 0 { continue; }
            for y in 4 * ty..4 * ty + 4 { for x in 4 * tx..4 * tx + 4 {
                let px = &mut rgb[y * 1536 + x];
                for ch in 0..3 { px[ch] = mix256(px[ch], WARM[ch], amt[e as usize]); }
            } }
        } }
    }
    knob(s, d, t, &c, rgb, &dep);
    silt(s, d, t, rgb);
    if n > CLICK { debris(s, d, t, rgb); }
    if n >= CLICK { crackle(s, d, t, &c, rgb); }
}

/// The model's turn into the camera's space (thousandths): the camera's vectors (300ths) times the model's axes.
fn turn(cm: [[i64; 3]; 3], axes: [[i64; 3]; 3]) -> [[i64; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| fl((0..3).map(|q| cm[i][q] * axes[j][q]).sum::<i64>(), 300)))
}

fn orca(s: &Scene, d: &Data, t: &Trig, orcas: &[super::orca::Mesh], rgb: &mut [[i64; 3]], dep: &dyn Fn(i64, i64) -> i64) {
    let n = s.n;
    let k = n - ORCA_FIRST;
    let sn = t.sin(fl(500_000 * k, 46));
    let across = 7000 - fl(14000 * k, 46);
    let rel: [i64; 3] = std::array::from_fn(|i| KNOB[i] + fl(FWD[i] * 5500, 1000) + fl(SIDE[i] * across, 1000) + if i == 1 { -2100 + fl(200 * sn, 999) } else { 0 } - s.cam[i]);
    let cm = [s.r, s.u, s.f];
    let rot = turn(cm, ORCA_AXES);
    let pos: [i64; 3] = std::array::from_fn(|i| fl(fl((0..3).map(|q| cm[i][q] * rel[q]).sum::<i64>(), 300) * 860, 4600));
    let zref = pos[2] - 500;
    let phase = md(fl(64 * n, 100), 12) as usize;
    for (at, dd) in super::orca::draw(&orcas[phase], rot, pos, zref) {
        let (x, y) = ((at % 1536) as i64, (at / 1536) as i64);
        let ahead = fl((dd + zref) * 46, 860);
        if ahead > dep(x, y) + 1 { continue; }
        let k = d.haze120[ahead.min(199) as usize];
        rgb[at] = [0, 1, 2].map(|ch| mix32(DARK[ch], rgb[at][ch], k));
    }
}

fn knob(s: &Scene, d: &Data, t: &Trig, c: &Ctl, rgb: &mut [[i64; 3]], dep: &dyn Fn(i64, i64) -> i64) {
    let n = s.n;
    let cm = [s.r, s.u, s.f];
    let rel = [0, 1, 2].map(|i| KNOB[i] - s.cam[i]);
    let pc: [i64; 3] = std::array::from_fn(|i| fl((0..3).map(|q| cm[i][q] * rel[q]).sum::<i64>(), 300));
    let rot = turn(cm, [SIDE, UP, FWD]);
    // the dial's tremble as it powers up (units across and along), and the knob's shudder as it sticks
    let (sx, sz) = if n > CLICK {
        let a10 = 12 + fl(50 * c.p19, 19);
        let (r1, r2) = (d.shake[(2 * (n - 332)) as usize], d.shake[(2 * (n - 332) + 1) as usize]);
        (fl(r1 * a10, 9990), fl(r2 * a10, 99900))
    } else { (0, 0) };
    let off = |o: [i64; 3]| -> [i64; 3] { std::array::from_fn(|i| fl(rot[i][0] * o[0] + rot[i][1] * o[1] + rot[i][2] * o[2], 1000)) };
    let rock_pos: [i64; 3] = std::array::from_fn(|i| fl(30 * pc[i], 100));
    let po = off([-sx, 0, -sz]);
    let plate_pos: [i64; 3] = std::array::from_fn(|i| pc[i] + po[i]);
    let ko = off([c.w - sx, 0, -sz]);
    let knob_pos: [i64; 3] = std::array::from_fn(|i| pc[i] + ko[i]);
    let delta = fl(250_000 * (c.v100 - 900), 300);
    let (sn, cs) = (t.sin(delta), t.sin(delta + 250_000));
    let ry = [[cs, 0, sn], [0, 999, 0], [-sn, 0, cs]];
    let rot_k: [[i64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| fl(rot[i][0] * ry[0][j] + rot[i][1] * ry[1][j] + rot[i][2] * ry[2][j], 999)));
    // from the click: the glow's gain over 9, and each ink's colour and strength this frame
    let post = n > CLICK;
    let (dg, x_ink, l10) = if post {
        let dg = fl(32 * (c.g100.min(200) - 95), 100);
        let fade = 35 + fl(65 * c.mm, 76);
        let f11 = 40 + fl(60 * c.mm, 76);
        let mut x = vec![[0i64; 3]; 12];
        let mut l = vec![0i64; 12];
        for cls in 0..11i64 {
            let l10 = fl(10 * (11 * c.mm - 38 - 76 * cls), 76).clamp(0, 10);
            let gg10 = (10 + fl(14 * l10, 10)).min(16);
            for ch in 0..3 {
                let pbar = PAINT[ch] + fl(INK[ch] * (14 * l10).min(100), 200);
                let q100 = pbar * (100 - fade) + fl(INK[ch] * c.g100 * fade * 8, 1000);
                let tq = fl(q100 * gg10, 10);
                x[cls as usize][ch] = fl(b(tq * (4500 + 55 * fade)), 700_000).min(255);
            }
            l[cls as usize] = l10;
        }
        for ch in 0..3 { x[11][ch] = fl(INK[ch] * c.g100 * f11, 10000).min(255); }
        l[11] = -1;
        (dg, x, l)
    } else { (0, vec![], vec![]) };
    for (mesh, r, pos, zref, scale) in [(&d.rock, rot, rock_pos, rock_pos[2] - 300, 30), (&d.plate, rot, plate_pos, plate_pos[2] - 470, 100), (&d.knob, rot_k, knob_pos, knob_pos[2] - 300, 100)] {
        for (at, dd, v) in draw4(mesh, r, pos, zref) {
            let (x, y) = ((at % 1536) as i64, (at / 1536) as i64);
            let ahead = fl(dd + zref, scale);
            if ahead > dep(x, y) + 1 { continue; }
            let tx = d.texel(v[2], v[0], v[1]);
            let mut col = [tx[0], tx[1], tx[2]];
            if post {
                for ch in 0..3 { col[ch] = tx[3 + ch] + fl(tx[6 + ch] * dg, 32); }
                let (ink, who) = (tx[9], tx[10] as usize);
                if ink > 0 {
                    let k = if l10[who] < 0 { ink } else { fl(22 * fl(ink * (10 + fl(14 * l10[who], 10)), 10), 32).min(32) };
                    for ch in 0..3 { col[ch] = mix32(col[ch], x_ink[who][ch], k); }
                }
            }
            let hz = s.haze70[ahead.min(199) as usize];
            rgb[at] = [0, 1, 2].map(|ch| { let v = mix32(col[ch], s.far[ch], hz); assert!((0..=255).contains(&v)); v });
        }
    }
}

/// The places a blob's box covers, by 4 x 4 pixels, and e (tenths) at each: across and down, separately.
fn falloff(x: i64, y: i64, reach: i64, r2: i64) -> (i64, i64, Vec<i64>, Vec<i64>) {
    let (tx0, tx1) = (fl((x - reach).max(0), 4), fl((x + reach).min(1535), 4));
    let (ty0, ty1) = (fl((y - reach).max(0), 4), fl((y + reach).min(767), 4));
    let ex = (tx0..=tx1).map(|tx| fl(10 * (4 * tx + 2 - x) * (4 * tx + 2 - x), r2)).collect();
    let ey = (ty0..=ty1).map(|ty| fl(10 * (4 * ty + 2 - y) * (4 * ty + 2 - y), r2)).collect();
    (tx0, ty0, ex, ey)
}

fn silt(s: &Scene, d: &Data, t: &Trig, rgb: &mut [[i64; 3]]) {
    let mut sum: HashMap<(i64, i64), i64> = HashMap::new();
    for (puff, start) in [(0i64, 314i64), (1, 323)] {
        let age = s.n - start;
        if !(0..14).contains(&age) { continue; }
        let amt: Vec<i64> = d.puff.iter().map(|&p| fl(p * (14 - age), 28)).collect();
        for i in 0..18 {
            let r = &d.silt[((puff * 18 + i) * 3) as usize..];
            let a = r[0];
            let rad = 490 + fl(r[1], 1000) + 12 * age;
            let q = [fl(t.sin(a + 250_000) * rad, 999), -35 + fl(8 * age * (500 + r[2]), 1000), fl(t.sin(a) * rad, 999)];
            let rel = [0, 1, 2].map(|c| KNOB[c] + q[c] - s.cam[c]);
            let Some((x, y, ahead)) = proj(rel, s) else { continue };
            let sz = fl(fl((35 + 5 * age) * 80_000, ahead), 100).max(1);
            assert!(sz <= 316, "a blob of silt {} pixels across", sz);
            let (tx0, ty0, ex, ey) = falloff(fl(x, 1000), fl(y, 1000), 3 * sz, 2 * sz * sz);
            for (j, &e2) in ey.iter().enumerate() { for (i2, &e1) in ex.iter().enumerate() {
                let e = e1 + e2;
                if e > 99 { continue; }
                *sum.entry((tx0 + i2 as i64, ty0 + j as i64)).or_insert(0) += amt[e as usize];
            } }
        }
    }
    for ((tx, ty), v) in sum {
        let k = v.min(141);
        for y in 4 * ty..4 * ty + 4 { for x in 4 * tx..4 * tx + 4 {
            let px = &mut rgb[(y * 1536 + x) as usize];
            for ch in 0..3 { px[ch] = mix256(px[ch], SILT[ch], k); }
        } }
    }
}

fn debris(s: &Scene, d: &Data, t: &Trig, rgb: &mut [[i64; 3]]) {
    for i in 0..140usize {
        let r = &d.debris[7 * i..7 * i + 7];
        let age = s.n - (332 + fl(r[0], 1000));
        if !(0..=16).contains(&age) { continue; }
        let a = r[1];
        let rad = 180 + fl(r[2], 1000) + (5 + fl(r[3], 1000)) * age;
        let q = [fl(t.sin(a + 250_000) * rad, 999), 10 + (2 + fl(r[4], 1000)) * age, fl(t.sin(a) * rad, 999)];
        let rel = [0, 1, 2].map(|c| KNOB[c] + q[c] - s.cam[c]);
        let Some((x, y, ahead)) = proj(rel, s) else { continue };
        let p100 = fl((25 + fl(r[6], 1000)) * 8000, ahead).max(100);
        let (x0, x1) = (fl(x - 10 * p100, 1000).max(0), fl(x + 10 * p100, 1000).min(1535));
        let (y0, y1) = (fl(y - 10 * p100, 1000).max(0), fl(y + 10 * p100, 1000).min(767));
        let k = fl(272 * (17 - age), 170);
        let col = if r[5] == 1 { PALE } else { CRUST };
        for yy in y0..=y1 { for xx in x0..=x1 {
            let px = &mut rgb[(yy * 1536 + xx) as usize];
            for ch in 0..3 { px[ch] = mix32(px[ch], col[ch], k); }
        } }
    }
}

/// A line of the lightning: its ends (thousandths of a pixel), whether it starts and ends its stroke, a fork's.
struct Seg { a: (i64, i64), b: (i64, i64), first: bool, last: bool, fork: bool }

fn crackle(s: &Scene, d: &Data, t: &Trig, c: &Ctl, rgb: &mut [[i64; 3]]) {
    let n = s.n;
    let k = c.p19;
    let (count, reach, w, flash) = match n {
        331 => (3, 90, 2, 4),
        350 => (16, 700, 4, 11),
        351 => (16, 700, 4, 19),
        352 => (18, 900, 5, 30),
        _ => (1 + fl(5 * k, 19), 60 + fl(120 * k, 19), 2, fl(128 * k, 1900)),
    };
    let rel_k = [0, 1, 2].map(|i| KNOB[i] - s.cam[i]);
    let mut segs: Vec<Seg> = vec![];
    // a line is drawn only if it comes within 64 pixels of the screen
    let near = |sg: &Seg| !(sg.a.0.max(sg.b.0) < -64_000 || sg.a.0.min(sg.b.0) > 1_599_000 || sg.a.1.max(sg.b.1) < -64_000 || sg.a.1.min(sg.b.1) > 831_000);
    if let Some((cx, cy, _)) = proj(rel_k, s) {
        for i in 0..count {
            let r = &d.crackle[(((n - 331) * 20 + i) * 80) as usize..];
            let a = r[0];
            let rad = fl(460 * (600 + fl(r[1], 1000)), 1000);
            let q = [fl(t.sin(a + 250_000) * rad, 999), 10, fl(t.sin(a) * rad, 999)];
            let rel = [0, 1, 2].map(|c2| KNOB[c2] + q[c2] - s.cam[c2]);
            let Some((mut x, mut y, _)) = proj(rel, s) else { continue };
            // its way out from the dial's middle (in hundredths of a pixel: the sums' angle takes sizes under 100000)
            let mut ang = md(t.atan2(fl(y - cy, 10), fl(x - cx, 10)) + fl(r[2], 1000), 1_000_000);
            let len = fl((500 + fl(r[3], 2)) * reach * 100, 1000);
            for j in 0..10usize {
                let st = &r[4 + 7 * j..];
                ang = md(ang + fl(st[0], 1000), 1_000_000);
                let (nx, ny) = (x + fl(t.sin(ang + 250_000) * len, 999), y + fl(t.sin(ang) * len, 999));
                segs.push(Seg { a: (x, y), b: (nx, ny), first: j == 0, last: j == 9, fork: false });
                (x, y) = (nx, ny);
                if st[1] == 1 {
                    let mut fa = md(ang + fl(st[2], 1000), 1_000_000);
                    let (mut fx, mut fy) = (x, y);
                    for m in 0..4usize {
                        fa = md(fa + fl(st[3 + m], 1000), 1_000_000);
                        let (gx, gy) = (fx + fl(t.sin(fa + 250_000) * len * 10, 999 * 14), fy + fl(t.sin(fa) * len * 10, 999 * 14));
                        segs.push(Seg { a: (fx, fy), b: (gx, gy), first: m == 0, last: m == 3, fork: true });
                        (fx, fy) = (gx, gy);
                    }
                }
            }
        }
    }
    // the lines: forks first, then the mains over them
    let mut core: HashMap<usize, i64> = HashMap::new();
    for fork in [true, false] {
        for sg in segs.iter().filter(|sg| sg.fork == fork && near(sg)) {
            let wd = if fork { (w - 1).max(1) } else { w };
            for (px, py) in line(sg.a, sg.b, wd) { if (0..1536).contains(&px) && (0..768).contains(&py) { core.insert((py * 1536 + px) as usize, if fork { 24 } else { 32 }); } }
        }
    }
    // their glow, every 4 x 4 pixels
    let table = &d.glow[match w { 2 => 0, 4 => 1, _ => 2 }];
    let reach_px = 3 * (4 + 2 * w);
    let mut glow: HashMap<(i64, i64), i64> = HashMap::new();
    for sg in segs.iter().filter(|sg| near(sg)) {
        let (dxt, dyt) = (fl(sg.b.0 - sg.a.0, 100), fl(sg.b.1 - sg.a.1, 100));
        let lent = isqrt(b(dxt * dxt + dyt * dyt));
        if lent == 0 { continue; }
        let (ux, uy) = (fl(1000 * dxt, lent), fl(1000 * dyt, lent));
        let len_px = fl(lent, 10);
        let (lo_x, hi_x) = (fl(sg.a.0.min(sg.b.0), 1000) - reach_px, fl(sg.a.0.max(sg.b.0), 1000) + reach_px);
        let (lo_y, hi_y) = (fl(sg.a.1.min(sg.b.1), 1000) - reach_px, fl(sg.a.1.max(sg.b.1), 1000) + reach_px);
        let (tx0, tx1) = (fl(lo_x.max(0), 4), fl(hi_x.min(1535), 4));
        let (ty0, ty1) = (fl(lo_y.max(0), 4), fl(hi_y.min(767), 4));
        if tx0 > tx1 || ty0 > ty1 { continue; }
        for ty in ty0..=ty1 { for tx in tx0..=tx1 {
            let (px, py) = (1000 * (4 * tx + 2) - sg.a.0, 1000 * (4 * ty + 2) - sg.a.1);
            let sv = fl(fl(b(px * ux + py * uy), 1000), 1000);
            let tv = fl(fl(b(px * uy - py * ux), 1000), 1000).abs();
            let along = if sv < 0 { if sg.first { -sv } else { continue } } else if sv >= len_px { if sg.last { sv - len_px } else { continue } } else { 0 };
            if along > 63 || tv > 63 { continue; }
            let g = table[(64 * along + tv) as usize];
            if g > 0 { *glow.entry((tx, ty)).or_insert(0) += g; }
        } }
    }
    for y in 0..768i64 { for x in 0..1536i64 {
        let at = (y * 1536 + x) as usize;
        let mut px = rgb[at];
        if let Some(&k) = core.get(&at) { for ch in 0..3 { px[ch] = mix32(px[ch], 255, k); } }
        if let Some(&g) = glow.get(&(x / 4, y / 4)) { for ch in 0..3 { px[ch] = mix32(px[ch], BLUE[ch], g.min(32)); } }
        if flash > 0 { for ch in 0..3 { px[ch] = mix32(px[ch], FLASH[ch], flash); } }
        rgb[at] = px;
    } }
}

/// A line's pixels, wd wide: along its longer way, each column (or row) whose middle is between its ends, the rows
/// (or columns) whose middles are within wd / 2 of where it crosses, that place stepped a pixel at a time.
fn line(a: (i64, i64), bb: (i64, i64), wd: i64) -> Vec<(i64, i64)> {
    let steep = (bb.1 - a.1).abs() > (bb.0 - a.0).abs();
    let (a, bb) = if steep { ((a.1, a.0), (bb.1, bb.0)) } else { (a, bb) };
    let (a, bb) = if bb.0 < a.0 { (bb, a) } else { (a, bb) };
    let dx = bb.0 - a.0;
    let mut out = vec![];
    if dx == 0 { return out; }
    let (c0, c1) = (fl(a.0 - 500 + 999, 1000), fl(bb.0 - 500, 1000));
    if c1 < c0 { return out; }
    let slope = fl(1000 * (bb.1 - a.1), dx);
    let y0 = a.1 + fl((1000 * c0 + 500 - a.0) * (bb.1 - a.1), dx);
    for k in 0..=(c1 - c0) {
        let yc = y0 + k * slope;
        let (r0, r1) = (fl(yc - 500 * wd - 500 + 999, 1000), fl(yc + 500 * wd - 500, 1000));
        for r in r0..=r1 { out.push(if steep { (r, c0 + k) } else { (c0 + k, r) }); }
    }
    out
}
