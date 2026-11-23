//! Independent reference for the die and the Rubik's cube (the prototype's dierubik.py): the plain cube coming toward
//! us tumbling, going to a die (its faces to ivory, the pips fizzing in), splitting into a scrambled Rubik's cube that
//! plays the rubik segment's 16-move solve while it tumbles, a pop, then roaming on tumbling, out to about twice the
//! distance and back. 192x96 on black, written from the formulas in ordinary Rust (a frame buffer of 384x192
//! subsamples, painter's order), every number as the machines keep it: whole numbers within a card, each product
//! rounded as the machines round it. Writes raw RGB frames to stdout. Args: the demo folder. With DR_STATS set,
//! prints the largest magnitude of each number to stderr.
//!
//! The view: the eye looks along z from 10 units in front of the cube's near place, 40 degrees high (264 subsamples
//! a unit at unit distance, the prototype's 263.76). The cube's place (x, y in 200ths of a unit, its distance in
//! 70ths) is scaled so it is always 700 away: x 245 / distance; the cube's sizes likewise (98 x 500ths / distance), so
//! a corner is the place plus the axes at those sizes, and seen at 2112 x / z eighths of a subsample. Where the
//! prototype used a depth buffer, the faces are drawn far to near: on a box, only the faces that face the eye (a box is
//! convex); on the die and each cubie, what sits on a hidden face first, then the body, then what sits on the faces
//! shown; the cubies by their distance (the turning layer's far side first, as the rubik segment). A plate, sticker or
//! pip is drawn only as far as it stands out of what it sits on (the rest can't be seen), so its sides cover the body's
//! edge exactly where the depth buffer's would.
use std::io::Write;
use std::path::Path;

#[path = "scene.rs"]
mod scene;
use scene::*;

const W: usize = 192;
const H: usize = 96;

fn numbers(p: &Path) -> Vec<i64> { std::fs::read_to_string(p).unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect() }

/// x y / 999, its size rounded half up (the dot machine).
fn m(x: i64, y: i64) -> i64 { let v = (x.abs() * y.abs() + 499) / 999; if (x < 0) != (y < 0) { -v } else { v } }
fn dot(p: &[(i64, i64)]) -> i64 { p.iter().map(|&(x, y)| m(x, y)).sum() }
/// a b / c, its size rounded half up (the muldiv machine).
fn muldiv(a: i64, b: i64, c: i64) -> i64 { let v = (a.abs() * b.abs() + c / 2) / c; if (a < 0) != (b < 0) { -v } else { v } }
/// The ease machine: from toward to, step j of n, straight or smooth, each product rounded down.
fn ease(smooth: bool, from: i64, to: i64, j: i64, n: i64) -> i64 {
    let mut part = (to - from).abs();
    for pass in 0..if smooth { 3 } else { 1 } { let times = if pass == 2 { 3 * n - 2 * j } else { j }; part = part * times / n; }
    if to < from { from - part } else { from + part }
}
fn hold(v: i64, lo: i64, hi: i64) -> i64 { v.clamp(lo, hi) }
/// The clock machine: the sum of counts times rates (quarter turns, tenths, thousandths of a tenth), rounded to the
/// tenth, as its quarter turn (0-3) and the rest (0-899).
fn clock(terms: &[(i64, &[i64])]) -> (i64, i64) {
    let total: i64 = terms.iter().map(|&(n, r)| n * (900_000 * r[0] + 1000 * r[1] + r[2])).sum::<i64>() + 500;
    let tenths = (total / 1000).rem_euclid(3600);
    (tenths / 900, tenths % 900)
}
/// The sine machine: 999 sin and cos of a quarter turn and rest, from the table between degrees by a straight line.
fn sincos(table: &[i64], (q, r): (i64, i64)) -> (i64, i64) {
    let read = |at: i64| { let (d, t) = ((at / 10) as usize, at % 10); table[d] + (table[d + 1] - table[d]) * t / 10 };
    let (mut s, mut c) = (read(r), read(900 - r));
    for _ in 0..q { let t = s; s = c; c = -t; }
    (s, c)
}
/// The perspective machine: 2112 v / z (264 subsamples a unit, in 8ths), cut toward zero.
fn persp(v: i64, z: i64) -> i64 { v.signum() * (2112 * v.abs() / z) }

struct Stats(std::collections::BTreeMap<&'static str, i64>);
impl Stats { fn see(&mut self, k: &'static str, v: i64) { let e = self.0.entry(k).or_insert(0); if v.abs() > *e { *e = v.abs(); } } }

struct Data { table: Vec<i64>, colours: Vec<[i64; 3]>, light: Vec<i64>, moves: Vec<[i64; 3]>, turns: Vec<i32>, pips: Vec<Vec<i32>>, sizes: Vec<i64>, rates: Vec<Vec<i64>>, cubies: Vec<[i32; 3]>, pockets: Vec<Vec<i64>> }

/// A step's number.
fn step(s: Step) -> i64 { match s { Step::Is(n) => n as i64, Step::Since { first, at, f } => (first + f - at) as i64 } }

/// A polygon on the screen (8ths of a subsample, y down plus 24) and its colour.
struct Poly { pts: Vec<(i64, i64)>, rgb: [i64; 3] }

/// The matrix: the tumble Ry Rx Rz, as the dot machine makes it, 999 scale, [row][column].
fn tumble(d: &Data, f: &Frame, st: &mut Stats) -> [[i64; 3]; 3] {
    let r = &d.rates;
    let angle = |k: usize| clock(&[(f.i as i64, &r[k]), (step(f.extra), &r[3 + k])]);
    let (sy, cy) = sincos(&d.table, angle(0));
    let (sx, cx) = sincos(&d.table, angle(1));
    let (sz, cz) = sincos(&d.table, angle(2));
    let (p1, p2, p3, p4) = (m(sy, sx), m(sy, cx), m(cy, sx), m(cy, cx));
    let g = [[m(cy, cz) + m(p1, sz), m(p1, cz) - m(cy, sz), p2], [m(cx, sz), m(cx, cz), -sx], [m(p3, sz) - m(sy, cz), m(sy, sz) + m(p3, cz), p4]];
    for row in &g { for &v in row { st.see("matrix entry", v); } }
    g
}

/// The place: x, y (200ths of a unit) and the distance (70ths).
fn place(d: &Data, p: Place, st: &mut Stats) -> (i64, i64, i64) {
    let pl = match p {
        Place::Approach(k) => (ease(true, -246, 0, k as i64, 18), ease(true, 89, 0, k as i64, 18), ease(true, 910, 395, k as i64, 18)),
        Place::Still => (0, 0, 395),
        Place::Roam(k) => {
            let s = |i: usize| sincos(&d.table, clock(&[(step(k), &d.rates[6 + i])]));
            let ((_, ca), (_, cb), (sc, _), (sd, _)) = (s(0), s(1), s(2), s(3));
            let z100 = dot(&[(260, 999), (-260, ca), (120, 999), (-120, cb)]);
            let z70 = dot(&[(395, 999), (182, 999), (-182, ca), (84, 999), (-84, cb)]);
            let w = dot(&[(100, 999), (z100, 999), (z100, 100)]);
            let v = dot(&[(60, 999), (z100, 599)]);
            st.see("roam: how far out (100ths)", z100); st.see("roam: across's size (200ths)", w);
            (m(w, sc), m(v, sd), z70)
        }
    };
    st.see("place across (200ths)", pl.0); st.see("place up (200ths)", pl.1); st.see("distance (70ths)", pl.2);
    pl
}

/// The sizes for a shape at distance z70: each 98 x its 500ths (times the scale) / z70.
fn sizes(d: &Data, z70: i64, scale: Scale, pips: Option<Step>, st: &mut Stats) -> Vec<i64> {
    let s500 = match scale {
        Scale::One => 500,
        Scale::Split(j) => ease(false, 540, 500, step(j), 7),
        Scale::Pop(k) => { let (s, _) = sincos(&d.table, clock(&[(hold(step(k), 0, 6), &d.rates[12])])); dot(&[(500, 999), (30, s)]) }
    };
    let mut out: Vec<i64> = d.sizes.iter().map(|&a| {
        let a = if let Scale::One = scale { a } else { muldiv(s500, a, 500) };
        muldiv(a, 98, z70)
    }).collect();
    for &v in &out { st.see("a size (scaled)", v); }
    out.push(pips.map_or(0, |j| ease(false, 0, out[S017], step(j), 4)));
    out
}

/// One shape at its place: its polygons in drawing order.
fn shape_polys(d: &Data, f: &Frame, which: Shape, st: &mut Stats) -> Vec<Poly> {
    let g0 = tumble(d, f, st);
    let (x200, y200, z70) = place(d, f.place, st);
    let c = [muldiv(x200, 245, z70), muldiv(y200, 245, z70), 700];
    st.see("place on the view (scaled)", c[0]); st.see("place on the view (scaled)", c[1]);
    // the corners' x are punched from a middle column near the cube's (8 subsamples a step), so they fit a card
    let shift = muldiv(c[0], 33, 700);
    let (die, rubik) = match which { Shape::Die(die) => (Some(die), None), Shape::Rubik(r) => (None, Some(r)), Shape::Split(..) => unreachable!() };
    let (scale, pips) = match (die, rubik) { (Some(dd), _) => (Scale::One, dd.pips), (_, Some(r)) => (r.scale, None), _ => unreachable!() };
    let sz = sizes(d, z70, scale, pips, st);
    // the turning layer's matrix: the tumble's times its turn about its axis
    let mut groups = vec![g0];
    let mut turning = None;
    if let Some(Rubik { turn: Some((mv, j, n)), .. }) = rubik {
        let [a, s, q] = d.moves[16 + mv];
        let e = ease(true, 0, 900, step(j), n as i64);
        let rate = [0, e, 0];
        let terms: Vec<(i64, &[i64])> = (0..q.abs()).map(|_| (1, &rate[..])).collect();
        let (sn, cs) = sincos(&d.table, clock(&terms));
        // about the axis by -side q quarter turns, eased
        let sn = if -s * q.signum() > 0 { sn } else { -sn };
        let (i, k) = AFTER[a as usize];
        let mut g1 = g0;
        for r in 0..3 { g1[r][i] = m(cs, g0[r][i]) + m(sn, g0[r][k]); g1[r][k] = m(cs, g0[r][k]) - m(sn, g0[r][i]); }
        groups.push(g1);
        turning = Some((a as usize, s as i32));
    }
    // each group's axes at each size; the place's depth along each axis
    let parts: Vec<Vec<Vec<[i64; 3]>>> = groups.iter().map(|g| (0..3).map(|b| sz.iter().map(|&s| [0, 1, 2].map(|k| m(g[k][b], s))).collect()).collect()).collect();
    let along: Vec<[i64; 3]> = groups.iter().map(|g| [0, 1, 2].map(|b| dot(&[(g[0][b], c[0]), (g[1][b], c[1]), (g[2][b], c[2])]))).collect();
    // a sum of terms: of the sizes themselves, or of the group's axis at those sizes (one part of it)
    let sum = |terms: &[Term], axis: usize, part: usize, sizes_only: Option<&[i64]>, g: usize| -> i64 {
        terms.iter().map(|&(sign, i)| sign as i64 * match sizes_only { Some(s) => s[i], None => parts[g][axis][i][part] }).sum()
    };
    // a face shows when the eye is beyond its plane
    let shows = |b: &Bx, a: usize, s: i32, st: &mut Stats| -> bool {
        let v = -(s as i64) * along[b.group][a] - sum(&b.plane(a, s), 0, 0, Some(&sz), 0);
        st.see("a face's test", v);
        hold(v, 0, 1) == 1
    };
    // the boxes in drawing order
    let mut boxes = vec![];
    if let Some(dd) = die {
        let (body, on) = die_boxes(&d.pips, dd.pips.is_some());
        let shown: Vec<bool> = (0..6).map(|fc| shows(&body, fc / 2, if fc % 2 == 1 { 1 } else { -1 }, st)).collect();
        boxes = in_order(body, on, &|fc| shown[fc]);
    }
    if let Some(r) = rubik {
        let pocket = &d.pockets[r.done];
        let mut list = vec![];
        // the eye is on the turning layer's side when beyond the plane half way out to it (the order machine's test,
        // 56 or more, on the test plus 55)
        let eye_side = turning.map(|(a, s)| -(s as i64) * along[0][a] - sz[S034] + 55 >= 56);
        for (n, &p) in d.cubies.iter().enumerate() {
            let group = turning.map_or(0, |(a, s)| (p[a] == s) as usize);
            let later = turning.map_or(1, |_| ((group == 1) == eye_side.unwrap()) as i32);
            let centre: Vec<i64> = (0..3).map(|k| (0..3).map(|b| p[b] as i64 * parts[group][b][S068][k]).sum()).collect();
            let key = dot(&[(c[0], centre[0]), (c[0], centre[0]), (c[1], centre[1]), (c[1], centre[1]), (c[2], centre[2]), (c[2], centre[2]),
                (centre[0], centre[0]), (centre[1], centre[1]), (centre[2], centre[2])]);
            st.see("a cubie's key", key);
            list.push((later, -key, n, p, group));
        }
        list.sort_by_key(|&(later, key, n, ..)| (later, key, n));
        for &(_, _, _, p, group) in &list {
            let (body, on) = cubie_boxes(p, group, &|fc| { let v = pocket[pocket_index(p, fc)]; (v >= 0).then_some(v as usize) });
            let shown: Vec<bool> = (0..6).map(|fc| shows(&body, fc / 2, if fc % 2 == 1 { 1 } else { -1 }, st)).collect();
            boxes.extend(in_order(body, on, &|fc| shown[fc]));
        }
    }
    // each face shown: its colour and its corners on the screen
    let colour = |g: usize, a: usize, s: i32, material: Material, st: &mut Stats| -> [i64; 3] {
        let gm = &groups[g];
        let lit = hold(s as i64 * dot(&[(gm[0][a], d.light[0]), (gm[1][a], d.light[1]), (gm[2][a], d.light[2])]), 0, 999);
        let toward = hold(-(s as i64) * gm[2][a], 0, 999);
        // the light at half its size (it reaches 1.08), so the colour is twice the base times it
        let k = dot(&[(310, 999), (190, lit), (40, toward)]);
        let base = match material {
            Material::Plastic => d.colours[PLASTIC],
            Material::Sticker(c) => d.colours[STICKERS + c],
            Material::Pip => d.colours[PIP],
            Material::DieFace(fc) => { let j = step(die.unwrap().colour); [0, 1, 2].map(|ch| ease(false, d.colours[PLAIN + fc][ch], d.colours[IVORY][ch], j, 8)) }
        };
        [0, 1, 2].map(|ch| { let v = 2 * m(base[ch], k); st.see("a colour before held", v); hold(v, 0, 255) })
    };
    let mut polys = vec![];
    for b in &boxes {
        for a in 0..3 { for s in [-1, 1] {
            if !shows(b, a, s, st) { continue }
            let rgb = colour(b.group, a, s, b.material, st);
            let pts = b.corners(a, s).iter().map(|corner| {
                let p: Vec<i64> = (0..3).map(|k| c[k] + (0..3).map(|ax| sum(&corner[ax], ax, k, None, b.group)).sum::<i64>()).collect();
                for &v in &p { st.see("a corner (scaled)", v); }
                let (x, y) = (persp(p[0], p[2]), 24 - persp(p[1], p[2]));
                st.see("a corner on the screen, x from the middle column", x - 64 * shift); st.see("a corner on the screen, y", y);
                (x, y)
            }).collect();
            polys.push(Poly { pts, rgb });
        } }
    }
    polys
}

fn pocket_index(p: [i32; 3], f: usize) -> usize { pocket(p, f) }

/// Painter's order on the subsamples, as the rubik segment's (its edge, start, column, walk and span machines), then
/// each pixel the mean of its four, rounded; black where nothing is.
fn raster(polys: &[Poly], st: &mut Stats) -> Vec<u8> {
    let mut sub = vec![None::<[i64; 3]>; 4 * W * H];
    for poly in polys {
        let n = poly.pts.len();
        for ys in 0..2 * H as i64 {
            let yc = 8 * (ys - 93) + 4;
            let (mut left, mut right) = (None::<i64>, None::<i64>);
            for k in 0..n {
                let ((x0, y0), (x1, y1)) = (poly.pts[k], poly.pts[(k + 1) % n]);
                if y0 == y1 { continue; }
                if ys == 0 { st.see("an edge across, down", x1 - x0); st.see("an edge across, down", y1 - y0); }
                let (xt, yt, xb, yb) = if y0 < y1 { (x0, y0, x1, y1) } else { (x1, y1, x0, y0) };
                if yc < yt || yc >= yb { continue; }
                let (num, den) = (xt * (yb - yt) + (xb - xt) * (yc - yt), yb - yt);
                if y1 > y0 {
                    let xs = -(-(num + 1532 * den)).div_euclid(8 * den);
                    left = Some(left.map_or(xs, |l: i64| l.max(xs)));
                } else {
                    let xs = (num + 1532 * den).div_euclid(8 * den);
                    right = Some(right.map_or(xs, |r: i64| r.min(xs)));
                }
            }
            if let (Some(l), Some(r)) = (left, right) {
                if l <= r { assert!(l >= 0 && r < 2 * W as i64, "off the screen"); }
                for xs in l..=r { sub[(ys * 2 * W as i64 + xs) as usize] = Some(poly.rgb); }
            }
        }
    }
    let mut frame = vec![0u8; W * H * 3];
    for y in 0..H { for x in 0..W { for c in 0..3 {
        let mut sum = 2;
        for (dy, dx) in [(0, 0), (0, 1), (1, 0), (1, 1)] { sum += sub[(2 * y + dy) * 2 * W + 2 * x + dx].map_or(0, |p| p[c]); }
        frame[(y * W + x) * 3 + c] = (sum / 4) as u8;
    } } }
    frame
}

/// Apply a move (axis, side, quarter turns) to the stickers, as the turn machine (the rubik segment's).
fn apply(stickers: &mut [[i64; 7]], mv: [i64; 3]) {
    let [a, s, q] = mv;
    let (i, j) = AFTER[a as usize];
    let quarters = (-s * q).rem_euclid(4);
    for st in stickers.iter_mut() {
        if st[a as usize] != s { continue; }
        for _ in 0..quarters { for base in [0, 3] { let (vi, vj) = (st[base + i], st[base + j]); st[base + i] = -vj; st[base + j] = vi; } }
    }
}

fn main() {
    let demo = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let r = |f: &str| numbers(&demo.join("dierubik/data").join(f));
    let rows = |v: Vec<i64>, n: usize| v.chunks(n).map(|c| c.to_vec()).collect::<Vec<_>>();
    let moves: Vec<[i64; 3]> = r("moves.txt").chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
    let solved: Vec<[i64; 7]> = r("solved.txt").chunks(7).map(|c| [c[0], c[1], c[2], c[3], c[4], c[5], c[6]]).collect();
    // the cube after the scramble and after each move of the solve, its stickers by pocket
    let mut pockets = vec![];
    let mut cube = solved.clone();
    for k in 0..=32 {
        if k >= 16 {
            let mut pocket = vec![-1i64; 162];
            for s in &cube {
                let a = (0..3).find(|&a| s[3 + a] != 0).unwrap();
                let p = [s[0] as i32, s[1] as i32, s[2] as i32];
                pocket[scene::pocket(p, face_of(a, s[3 + a] as i32))] = s[6];
            }
            pockets.push(pocket);
        }
        if k < 32 { apply(&mut cube, moves[k]); }
    }
    let d = Data {
        table: numbers(&demo.join("rubik/data/sine.txt")),
        colours: r("colours.txt").chunks(3).map(|c| [c[0], c[1], c[2]]).collect(),
        light: r("light.txt"), moves, turns: r("turns.txt").iter().map(|&v| v as i32).collect(),
        pips: std::fs::read_to_string(demo.join("dierubik/data/pips.txt")).unwrap().lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect(),
        sizes: r("sizes.txt"), rates: rows(r("rates.txt"), 3), cubies: cubies(&r("cubies.txt").iter().map(|&v| v as i32).collect::<Vec<_>>()), pockets,
    };
    let mut st = Stats(Default::default());
    let mut out = std::io::stdout().lock();
    for fi in 0..frames(&d.turns) {
        let f = frame(&d.turns, fi);
        let img = match f.shape {
            Shape::Split(die, rubik, j) => {
                let (a, b) = (raster(&shape_polys(&d, &f, Shape::Die(die), &mut st), &mut st), raster(&shape_polys(&d, &f, Shape::Rubik(rubik), &mut st), &mut st));
                let c = ease(false, 0, 32, step(j), 7);
                a.iter().zip(&b).map(|(&a, &b)| (a as i64 + (b as i64 - a as i64) * c / 32) as u8).collect()
            }
            s => raster(&shape_polys(&d, &f, s, &mut st), &mut st),
        };
        out.write_all(&img).unwrap();
    }
    if std::env::var("DR_STATS").is_ok() { for (k, v) in &st.0 { eprintln!("{k}: {v}"); } }
}
