//! The mesh kit's test, drawn independently in ordinary Rust: Blender's Suzanne at her second subdivision (7,872
//! faces, 15,744 triangles) turning under a light at 1536x768, smooth-shaded, 48 frames. Every number is a whole
//! number, worked out the way the machines work it out (the swarm's arithmetic, the swarm's edges, the kit's own
//! machines), so the frames are the spec to the pixel.
//!
//! The model (data/suzanne2.txt, from Blender by data/make_mesh.py): corners in whole units within 499 of its middle,
//! normals in hundredths (99 long), each triangle's face normal and its plane's distance.
//!
//! Each frame:
//! - the turn: a yaw (a twentieth of a turn and more each frame) and a fixed pitch, their sines from the swarm's
//!   table (999 sin, by tenths); the matrix R in thousandths, its products rounded down;
//! - each corner in the camera's space, in thousandths of a unit (exact sums of products: R v + 1000 P); its depth
//!   rounded down to a unit; its place on the screen, hundredths of a pixel from the middle: 150 times across (or
//!   less up) over the depth, rounded down (a focal length of 1500 pixels); its depth for the depth test, less 1321;
//! - the light and the half-way vector turned into the model's space (R's transpose, thousandths, then 98 hundredths of
//!   that), and the camera,
//!   in tens of units;
//! - each triangle faces the camera when its normal's dot with the camera (hundredths times tens, each product cut
//!   toward zero to a hundredth) passes its plane's distance less 6 (the margin keeps a face seen nearly edge on);
//! - each corner's light: its normal's dots with the light and the half-way vector (none below 0), the first in tenths; the
//!   highlight from the table (data/spec.txt) at the half-way dot; the colour amb + d diff / 100 + s spec / 100, at most 255;
//! - each edge of a triangle that faces the camera, once: the rows it meets (their middles from its upper end to
//!   short of its lower) and where it crosses each, its upper end's x plus its run times how far the row is below over
//!   its height, rounded down; the first pixel whose middle is at or right of it; and its ends' values (depth, red,
//!   green, blue) down it: at its j-th of n rows, the upper's plus the change times (j + 1) / (n + 1), cut toward 0;
//! - each triangle's span on each row: from its two edges' pixels, the left one's values to the right one's the same
//!   way across (the k-th of n pixels at (k + 1) / (n + 1));
//! - each pixel: the nearest piece (the least depth; the first of equals, in the triangles' order), its colour; the
//!   background where there is none.
//!
//! MESH_SHADING=flat draws each triangle in its face's light (only the depth carried); MESH_SHADING=pixel carries the
//! corners' normals instead of colours and lights each pixel that shows, its normal first lifted a step toward a
//! unit (half of what its size squared falls short, in hundredths). The host takes the same.
use std::io::Write;

const W: i64 = 1536;
const H: i64 = 768;
const FRAMES: i64 = 48;
/// The focal length over 10: 1500 pixels.
const F10: i64 = 150;
/// The model's place in the camera's space (units): straight ahead.
const P: [i64; 3] = [0, 0, 1820];
/// The depth test's zero: the nearest a corner can be.
const ZREF: i64 = 1321;
/// The turn, millionths of a turn: the yaw at frame 0 and its step a frame; the pitch.
const YAW0: i64 = -96_000;
const YAW_STEP: i64 = 4000;
const PITCH: i64 = -30_000;
/// The light: the way toward it, and the half-way vector between it and the way toward the camera (thousandths,
/// the camera's space: x right, y up, z away).
const LIGHT: [i64; 3] = [-451, 621, -641];
const HALF: [i64; 3] = [-249, 343, -906];
/// The back-face test's margin (tens of units).
const MARGIN: i64 = 6;
/// Suzanne's clay: ambient, diffuse and highlight colours; the background.
const AMB: [i64; 3] = [34, 32, 36];
const DIFF: [i64; 3] = [205, 196, 186];
const SPEC: [i64; 3] = [110, 110, 110];
const BACK: [u8; 3] = [40, 42, 48];

fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }
/// c v / 100, cut toward zero.
fn by100(c: i64, v: i64) -> i64 { c * v / 100 }

struct Trig(Vec<i64>);
impl Trig {
    fn read(dir: &str) -> Trig {
        let text = std::fs::read_to_string(format!("{dir}/swarm/data/tables.txt")).unwrap();
        let l = text.lines().find(|l| l.split_whitespace().next() == Some("trig")).unwrap();
        Trig(l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect())
    }
    /// 999 sin of an angle (millionths of a turn): the table every thousandth of a turn, between by tenths.
    fn sin(&self, a: i64) -> i64 {
        let t = fl(md(a, 1_000_000), 100);
        let q = t / 2500;
        let mut r = t % 2500;
        if q % 2 == 1 { r = 2500 - r; }
        let (k, fr) = ((r / 10) as usize, r % 10);
        let v = if fr == 0 { self.0[k] } else { self.0[k] + (self.0[k + 1] - self.0[k]) * fr / 10 };
        if q >= 2 { -v } else { v }
    }
    fn cos(&self, a: i64) -> i64 { self.sin(a + 250_000) }
}

pub struct Mesh { pub v: Vec<[i64; 3]>, pub n: Vec<[i64; 3]>, pub t: Vec<[usize; 3]>, pub fnorm: Vec<[i64; 3]>, pub fd: Vec<i64> }

pub fn read_mesh(path: &str) -> Mesh {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path, e));
    let mut lines = text.lines().filter(|l| !l.starts_with('#'));
    let mut m = Mesh { v: vec![], n: vec![], t: vec![], fnorm: vec![], fd: vec![] };
    let count = |l: Option<&str>, what: &str| -> usize {
        let w: Vec<&str> = l.unwrap().split_whitespace().collect();
        assert_eq!(w[0], what);
        w[1].parse().unwrap()
    };
    count(lines.next(), "faces");
    let nv = count(lines.next(), "vertices");
    for _ in 0..nv {
        let x: Vec<i64> = lines.next().unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect();
        m.v.push([x[0], x[1], x[2]]);
        m.n.push([x[3], x[4], x[5]]);
    }
    let nt = count(lines.next(), "triangles");
    for _ in 0..nt {
        let x: Vec<i64> = lines.next().unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect();
        m.t.push([x[0] as usize, x[1] as usize, x[2] as usize]);
        m.fnorm.push([x[3], x[4], x[5]]);
        m.fd.push(x[6]);
    }
    m
}

fn read_spec(dir: &str) -> Vec<i64> {
    let text = std::fs::read_to_string(format!("{dir}/mesh/data/spec.txt")).unwrap();
    text.lines().filter(|l| !l.starts_with('#')).flat_map(|l| l.split_whitespace().map(|v| v.parse::<i64>().unwrap()).collect::<Vec<_>>()).collect()
}

/// The values of a walk from a to b (either way round): the k-th of n places on from the lesser, the value at its
/// side plus the change to the other's times (k + 1) / (n + 1), cut toward zero.
fn walk(a: i64, b: i64, va: i64, vb: i64, k: i64) -> i64 {
    let (n, vl, vr) = if b < a { (a - b, vb, va) } else { (b - a, va, vb) };
    vl + (vr - vl) * (k + 1) / (n + 1)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let only: Option<Vec<i64>> = a.get(2).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let trig = Trig::read(dir);
    let mesh = read_mesh(&format!("{dir}/mesh/data/suzanne2.txt"));
    let spec = read_spec(dir);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut stats = (0usize, 0usize, 0usize, 0usize);
    for f in 0..FRAMES {
        if let Some(o) = &only { if !o.contains(&f) { continue; } }
        let yaw = YAW0 + YAW_STEP * f;
        let (sy, cy, sp, cp) = (trig.sin(yaw), trig.cos(yaw), trig.sin(PITCH), trig.cos(PITCH));
        // R: the pitch (about x) after the yaw (about y)
        let r = [[cy, 0, sy], [fl(sp * sy, 1000), cp, fl(-sp * cy, 1000)], [fl(-cp * sy, 1000), sp, fl(cp * cy, 1000)]];
        // the corners on the screen
        let corners: Vec<(i64, i64, i64)> = mesh.v.iter().map(|v| {
            let t: Vec<i64> = (0..3).map(|i| r[i][0] * v[0] + r[i][1] * v[1] + r[i][2] * v[2] + 1000 * P[i]).collect();
            let z = fl(t[2], 1000);
            let (x, y) = (fl(F10 * t[0], z), fl(-F10 * t[1], z));
            let depth = z - ZREF;
            assert!((0..=999).contains(&depth), "a depth of {}", depth);
            assert!(x.abs() < 76_800 && y.abs() < 38_400, "a corner off the screen");
            (x, y, depth)
        }).collect();
        // the light, the half-way vector and the camera in the model's space
        let tr = |w: [i64; 3]| -> [i64; 3] { std::array::from_fn(|j| fl(r[0][j] * w[0] + r[1][j] * w[1] + r[2][j] * w[2], 1000)) };
        // at 98 hundredths, so a dot with a normal carried across a triangle (a little over 99 long) fits a card
        let (lm, hm) = (tr(LIGHT).map(|v| fl(98 * v, 100)), tr(HALF).map(|v| fl(98 * v, 100)));
        let cam: [i64; 3] = tr([-P[0], -P[1], -P[2]]).map(|c| fl(c, 10));
        let shading = std::env::var("MESH_SHADING").unwrap_or_default();
        let light = |n: [i64; 3], lift: bool| -> [i64; 3] {
            let d: i64 = (0..3).map(|j| by100(n[j], lm[j])).sum();
            let h: i64 = (0..3).map(|j| by100(n[j], hm[j])).sum();
            // lifted: one step toward a unit normal's, half of what its size squared (hundredths) falls short of 98
            let short = if lift { (98 - (0..3).map(|j| by100(n[j], n[j])).sum::<i64>()) / 2 } else { 0 };
            let d = if d <= 0 { 0 } else { (d + by100(short, d)).min(989) } / 10;
            let h = if h <= 0 { 0 } else { (h + by100(short, h)).min(999) };
            let s = spec[h as usize];
            std::array::from_fn(|c| (AMB[c] + by100(d, DIFF[c]) + by100(s, SPEC[c])).clamp(0, 255))
        };
        // the corners' colours (smooth), or their normals carried (per pixel)
        let colour: Vec<[i64; 3]> = mesh.n.iter().map(|&n| if shading == "pixel" { n } else { light(n, false) }).collect();
        // the triangles that face the camera
        let front: Vec<usize> = (0..mesh.t.len()).filter(|&k| {
            let n = mesh.fnorm[k];
            (0..3).map(|j| by100(n[j], cam[j])).sum::<i64>() > mesh.fd[k] - MARGIN
        }).collect();
        // each edge once: its rows, crossings' pixels and values
        let mut edges: std::collections::BTreeMap<(usize, usize), Vec<(i64, i64, [i64; 4])>> = Default::default();
        let flat = shading == "flat";
        let value = |i: usize| -> [i64; 4] { if flat { [corners[i].2, 0, 0, 0] } else { [corners[i].2, colour[i][0], colour[i][1], colour[i][2]] } };
        for &k in &front {
            let t = mesh.t[k];
            for e in 0..3 {
                let (i, j) = (t[e], t[(e + 1) % 3]);
                let key = (i.min(j), i.max(j));
                if edges.contains_key(&key) { continue; }
                let (top, bot) = if corners[j].1 < corners[i].1 { (j, i) } else { (i, j) };
                let (a, b) = (corners[top], corners[bot]);
                let first = fl(a.1 + 49, 100);
                let rows = fl(b.1 + 49, 100) - first;
                let (va, vb) = (value(top), value(bot));
                let mut list = vec![];
                for jr in 0..rows.max(0) {
                    let row = first + jr;
                    if !(-384..384).contains(&row) { continue; }
                    let x = a.0 + fl((b.0 - a.0) * (100 * row + 50 - a.1), b.1 - a.1);
                    let px = fl(x + 49, 100).clamp(-768, 768);
                    list.push((row, px, std::array::from_fn(|c| walk(0, rows, va[c], vb[c], jr))));
                }
                edges.insert(key, list);
            }
        }
        // the pieces on each pixel, nearest first of equals
        let mut best: Vec<Option<(i64, [i64; 3])>> = vec![None; (W * H) as usize];
        let mut pieces = 0;
        for &k in &front {
            let t = mesh.t[k];
            let face = if flat { light(mesh.fnorm[k], false) } else { [0; 3] };
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
                    let mut v: [i64; 4] = std::array::from_fn(|ch| walk(pa, pb, va[ch], vb[ch], kx));
                    if flat { v[1..].copy_from_slice(&face); }
                    let at = ((row + 384) * W + lo + kx + 768) as usize;
                    pieces += 1;
                    if best[at].is_none_or(|(d, _)| v[0] < d) { best[at] = Some((v[0], [v[1], v[2], v[3]])); }
                }
            }
        }
        let covered = best.iter().filter(|b| b.is_some()).count();
        stats = (stats.0 + front.len(), stats.1 + edges.len(), stats.2 + pieces, stats.3 + covered);
        let mut img = vec![0u8; (W * H * 3) as usize];
        for (p, b) in best.iter().enumerate() {
            let c = match b {
                Some((_, c)) => { let c = if shading == "pixel" { light(*c, true) } else { *c }; [c[0] as u8, c[1] as u8, c[2] as u8] }
                None => BACK,
            };
            img[3 * p..3 * p + 3].copy_from_slice(&c);
        }
        out.write_all(&img).unwrap();
    }
    if std::env::var("MESH_STATS").is_ok() {
        let n = FRAMES as usize;
        eprintln!("a frame on average: {} triangles facing, {} edges, {} pieces, {} pixels covered", stats.0 / n, stats.1 / n, stats.2 / n, stats.3 / n);
    }
}
