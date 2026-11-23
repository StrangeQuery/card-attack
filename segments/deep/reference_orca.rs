// The orcas (the dive at 1536x768, frames 84..156): a pair of killer whales gliding left to right far out and low in
// the canyon's haze, seen only as silhouettes, as in the design (prototypes/orca.py, variant 6). Drawn into the
// water before the fish, snow and jellyfish; hidden where the dive's seabed or kelp is nearer.
//
// The model is the Killer Whale by Poly by Google (CC BY 3.0) as the mesh kit's cards (data/make_orca.py: 860 units
// nose to flukes, the head at -z) at 12 phases of a gentle swim. Each frame, in whole numbers as the machines work it
// out (the swarm's sums, the mesh kit):
// - where each is from the camera (hundredths of a unit): the leader 66 ahead in the world's z, 19 below the diver,
//   across from 38 left to 32 right of the world's middle over the 72 frames, less the diver's x; the second behind it
//   along its heading (14 and up to 5 more, slowly), to its far side (7 and up to 4) and a little lower (3 and up to
//   1.5), by the swarm's sines;
// - the heading fixed (thousandths: 662 across, 749 along; the side 749, -662); the model's turn R = C B in
//   thousandths, C the camera's right, up and forward (hundredths), B the model's axes in the world (its x the side,
//   y up, z back along the heading), each entry rounded down; its place in the camera's space in model units
//   (fl(fl(q, 10) 860, 1000 L), q the place's dot with the camera's vector, L its length in units: 46, the second 37);
// - its swim phase: fl(64 n, 100) mod 12, the second's 7 phases on;
// - drawn by the mesh kit, flat, unlit (the kit's corners, back faces, edges, spans and depth test, focal length
//   800 pixels, the depth test's zero its middle's depth less 500): each pixel's depth;
// - each pixel's distance ahead in units, fl((depth + zero) L, 860); hidden where that is more than the dive's depth
//   there (at most 99) plus 1; its colour a dark body (2, 4, 7) mixed toward the water's far colour by the dive's
//   fog at that distance (32nds: 32 min(1, 0.85 (a / 70)^0.8)): the haze;
// - the further of the two drawn first.

use super::hd::{mix32, Trig};

fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }

/// A mesh as make_mesh.py prints it (as the kit's reference reads it).
pub struct Mesh { pub v: Vec<[i64; 3]>, pub t: Vec<[usize; 3]>, pub fnorm: Vec<[i64; 3]>, pub fd: Vec<i64> }

pub fn read_mesh(path: &str) -> Mesh {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {}", path, e));
    let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    let nv: usize = lines[1].split_whitespace().nth(1).unwrap().parse().unwrap();
    let nums = |l: &str| -> Vec<i64> { l.split_whitespace().take(7).map(|v| v.parse().unwrap()).collect() };
    let mut m = Mesh { v: vec![], t: vec![], fnorm: vec![], fd: vec![] };
    for l in &lines[2..2 + nv] { let x = nums(l); m.v.push([x[0], x[1], x[2]]); }
    let nt: usize = lines[2 + nv].split_whitespace().nth(1).unwrap().parse().unwrap();
    for l in &lines[3 + nv..3 + nv + nt] {
        let x = nums(l);
        m.t.push([x[0] as usize, x[1] as usize, x[2] as usize]);
        m.fnorm.push([x[3], x[4], x[5]]);
        m.fd.push(x[6]);
    }
    m
}

pub const FIRST: i64 = 84;
pub const LAST: i64 = 156;
const F10: i64 = 80;
const MARGIN: i64 = 6;
const LENGTH: i64 = 860;
const DARK: [i64; 3] = [2, 4, 7];
/// the heading and the side (thousandths)
const HEAD: [i64; 3] = [662, 0, 749];
const SIDE: [i64; 3] = [749, 0, -662];

fn by100(c: i64, v: i64) -> i64 { c * v / 100 }

/// A walk from a to b (either way round): the k-th of n places on from the lesser, the value there plus the change
/// to the other's times (k + 1) / (n + 1), cut toward zero (the kit's).
fn walk(a: i64, b: i64, va: i64, vb: i64, k: i64) -> i64 {
    let (n, vl, vr) = if b < a { (a - b, vb, va) } else { (b - a, va, vb) };
    vl + (vr - vl) * (k + 1) / (n + 1)
}

/// The kit's flat draw, unlit: each pixel the model covers (its index in the 1536x768 frame) and its depth less zref.
pub fn draw(mesh: &Mesh, r: [[i64; 3]; 3], p: [i64; 3], zref: i64) -> Vec<(usize, i64)> {
    let (w, h) = (1536i64, 768i64);
    let corners: Vec<(i64, i64, i64)> = mesh.v.iter().map(|v| {
        let t: Vec<i64> = (0..3).map(|i| r[i][0] * v[0] + r[i][1] * v[1] + r[i][2] * v[2] + 1000 * p[i]).collect();
        let z = fl(t[2], 1000);
        let (x, y) = (fl(F10 * t[0], z), fl(-F10 * t[1], z));
        let depth = z - zref;
        assert!((0..=999).contains(&depth), "an orca's depth of {}", depth);
        (x, y, depth)
    }).collect();
    let cam: [i64; 3] = [0, 1, 2].map(|j| fl(fl(r[0][j] * -p[0] + r[1][j] * -p[1] + r[2][j] * -p[2], 1000), 10));
    let front: Vec<usize> = (0..mesh.t.len()).filter(|&k| {
        let n = mesh.fnorm[k];
        (0..3).map(|j| by100(n[j], cam[j])).sum::<i64>() > mesh.fd[k] - MARGIN
    }).collect();
    let mut edges: std::collections::BTreeMap<(usize, usize), Vec<(i64, i64, i64)>> = Default::default();
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
            let mut list = vec![];
            for jr in 0..rows.max(0) {
                let row = first + jr;
                if !(-384..384).contains(&row) { continue; }
                let x = a.0 + fl((b.0 - a.0) * (100 * row + 50 - a.1), b.1 - a.1);
                let px = fl(x + 49, 100).clamp(-768, 768);
                list.push((row, px, walk(0, rows, a.2, b.2, jr)));
            }
            edges.insert(key, list);
        }
    }
    let mut best: std::collections::BTreeMap<usize, i64> = Default::default();
    for &k in &front {
        let t = mesh.t[k];
        let mut by_row: std::collections::BTreeMap<i64, Vec<(i64, i64)>> = Default::default();
        for e in 0..3 {
            let (i, j) = (t[e], t[(e + 1) % 3]);
            for &(row, px, v) in &edges[&(i.min(j), i.max(j))] { by_row.entry(row).or_default().push((px, v)); }
        }
        for (row, c) in by_row {
            assert_eq!(c.len(), 2, "a triangle's row met {} edges", c.len());
            let ((pa, va), (pb, vb)) = (c[0], c[1]);
            let (lo, n) = (pa.min(pb), (pb - pa).abs());
            for kx in 0..n {
                let d = walk(pa, pb, va, vb, kx);
                let x = lo + kx;
                if !(-w / 2..w / 2).contains(&x) { continue; }
                let at = ((row + h / 2) * w + x + w / 2) as usize;
                if best.get(&at).is_none_or(|&b| d < b) { best.insert(at, d); }
            }
        }
    }
    best.into_iter().collect()
}

/// The pair this frame (camera at `cam`, hundredths of a unit; its right, up and forward in hundredths): for each,
/// far one first, its place, turn, zero, length and phase.
pub fn orcas(n: i64, cam: [i64; 3], r: [i64; 3], u: [i64; 3], f: [i64; 3], t: &Trig) -> Vec<([i64; 3], [[i64; 3]; 3], i64, i64, usize)> {
    let k = n - FIRST;
    let rel1 = [-3800 + fl(7000 * k, 72) - cam[0], -1900, 6600];
    let g1 = 1400 + fl(500 * t.sin(fl(800_000 * k, 72)), 999);
    let g2 = 700 + fl(400 * t.sin(fl(600_000 * k, 72) + 159_155), 999);
    let g3 = -300 + fl(150 * t.sin(fl(500_000 * k, 72)), 999);
    let rel2 = [0, 1, 2].map(|c| rel1[c] - fl(HEAD[c] * g1, 1000) - fl(SIDE[c] * g2, 1000) + if c == 1 { g3 } else { 0 });
    let cmat = [r, u, f];
    // the model's axes in the world: x the side, y up, z back along the heading
    let axes = [SIDE, [0, 1000, 0], HEAD.map(|v| -v)];
    let rot: [[i64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| fl((0..3).map(|c| cmat[i][c] * axes[j][c]).sum::<i64>(), 100)));
    let phase = fl(64 * n, 100);
    let mut both: Vec<([i64; 3], [[i64; 3]; 3], i64, i64, usize)> = [(rel1, 46, 0), (rel2, 37, 7)].iter().map(|&(rel, l, off)| {
        let pos: [i64; 3] = std::array::from_fn(|i| fl(fl((0..3).map(|c| cmat[i][c] * rel[c]).sum::<i64>(), 10) * LENGTH, 1000 * l));
        (pos, rot, pos[2] - 500, l, md(phase + off, 12) as usize)
    }).collect();
    // the further first (ahead in units: the place's depth by its length)
    if both[1].0[2] * both[1].3 > both[0].0[2] * both[0].3 { both.swap(0, 1); }
    both
}

/// Draw the pair into the picture.
pub fn frame(n: i64, cam: [i64; 3], r: [i64; 3], u: [i64; 3], f: [i64; 3], far: [i64; 3], meshes: &[Mesh], haze: &[i64], t: &Trig,
             rgb: &mut [[i64; 3]], depth: &dyn Fn(i64, i64) -> i64) {
    if !(FIRST..=LAST).contains(&n) { return; }
    for (pos, rot, zref, l, ph) in orcas(n, cam, r, u, f, t) {
        for (at, d) in draw(&meshes[ph], rot, pos, zref) {
            let (x, y) = ((at % 1536) as i64, (at / 1536) as i64);
            let ahead = fl((d + zref) * l, LENGTH);
            if ahead > depth(x, y) + 1 { continue; }
            let k = haze[ahead.min(199) as usize];
            rgb[at] = [0, 1, 2].map(|c| mix32(DARK[c], far[c], k));
        }
    }
}
