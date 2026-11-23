//! The polys: a nod to Complex's Dope (1995) and its "7800 PHONG polys". A morphing blob, Phong-shaded, tumbling
//! toward us over a dark nebula, with the brag box: 20 triangles for five and a half seconds, the mesh growing to
//! 7680, triangles split one at a time to 7800, a beat, and one more: 7801, and the box hops and turns gold. Then the
//! blob shrinks and moves aside, and a second blob, copper, comes in from the right, growing to 7800 triangles of its
//! own; the count races to 15601 and the box hops again, brighter. The two orbit each other, a whole turn, each
//! casting its shadow on the other. The count in the box is the one the mix machine counts as it takes each triangle
//! in, every frame, and every one of those triangles is filled.
//!
//! Everything is lists of cards through small machines; the host deals the lists, carries what one machine punches
//! to the next, and sorts pieces of triangles to their pixels by the row and column punched on them, as a card
//! sorter would. Per size of the mesh: the grid machine's angles, the sphere's points (angle, sine and dot) and the
//! waves' phases there (phase). Per frame: the turns' sines, the centre (dot); per corner the waves' sines, the
//! bulge, the corner pushed out, turned and placed (dot), its normal from the waves' slope (dot), made a unit
//! (square, root, unitize) and turned (dot); the splits' middles (dot); per triangle the normals it carries (dot,
//! mix); per edge its rows (rows) and the values down it (walk); per triangle row its pixels (cross) and the values
//! across (walk); per pixel the nearest piece (depth) and its light (face, dot, lift, spec, rim, dot); per row the
//! colours, each blob's from its level as it comes, no bands (py_tone, dot), and the brag box laid over (lay). With two blobs, the orbit
//! places them (sine, dot) and their shadows come from a shadow map: their corners seen from the light (dot), their
//! triangles filled again on the map with their depth (rows, walk, cross) and the nearest to the light kept on each
//! place with its blob (py_near); each pixel is placed on the map (dot, py_cell) and loses its diffuse light and
//! highlight if the other blob is there nearer the light (py_test). The nebula and its stars are worked out once
//! (nebula, angle, sine, dot, stars, star) and carried to every frame.
use crate::*;
use std::collections::HashMap;
use std::path::Path;

const WIDTH: i32 = W2 as i32;
const HEIGHT: i32 = H2 as i32;

// --- the timeline -----------------------------------------------------------------------------------------------
// 20 triangles (5 x 3) until frame 66; the grid grows (smoothstep) to 60 x 65 (7680) by 138; then splits, 110 of
// them by 168, then one more each tick, slower and slower, to 7800; a beat, and one more: 7801.
const GROW: (i32, i32) = (66, 138);
const FAST: i32 = 168;
const TICKS: [i32; 10] = [2, 2, 4, 4, 6, 6, 8, 10, 12, 16];
const BEAT: i32 = 18;
/// The box shows from half a second.
const BOX_FROM: usize = 6;
/// The box's hop at 7801 and at the final count, rows up (negative) or down, a frame at a time.
const HOP: [i32; 6] = [-4, -4, -4, -2, 0, 1];
/// The box (52 x 37 at x 4, y 4).
const BOX: (usize, usize, usize, i32) = (52, 37, 4, 4);

// The timeline (the eases and turns were the host's, the smoothsteps in floats). Every frame's numbers are
// worked out at once on the train's operations (fx::train::Cardiac: exact sums, products and divisions on CARDIAC),
// the host only picking each frame's out. The grid grows from 5 x 3 to 60 x 65 by a smoothstep from GROW.0 to
// GROW.1 (5 + 55 e and 3 + 62 e, rounded); then the splits: 110 over the next 30 frames, then one at each tick
// (TICKS apart) and one a beat after the last. From frame 270 the first blob shrinks to 20 pixels and moves aside
// (smoothstep, to 290). From 282 the second comes in from the right, its grid growing the same way to 60 x 65 by
// 306 as it closes in; then it splits, 20 a frame, until the two blobs together are the final count
// (polys/data/final.txt). They orbit each other, faster and faster to a turn in 80 frames, until they have gone a
// whole turn round.
const SHRINK: (i32, i32) = (270, 290);
const SECOND: (i32, i32) = (282, 306);
const RACE: i32 = 20;
const SPIN: i32 = 12;
pub const END: usize = SECOND.0 as usize + 92;

/// The orbit's plane: across it (to the right, a little up and toward us; its shear seen from the light is all
/// across) and toward the light, in hundredths. The blobs are on opposite sides of the centre, 56 pixels apart.
const ACROSS: [i32; 3] = [87, -35, -36];

/// An angle in 960ths of a turn, brought within half a turn either way (for the sky's constant bends).
fn half_turn(a: i32) -> i32 { let a = a.rem_euclid(960); if a >= 480 { a - 960 } else { a } }

/// A blob's numbers in a frame: the three waves' phases and a third of a turn on, the turns about x and y and a
/// third and two thirds of a turn on, the drift across, in 960ths of a turn; the radius in quarter pixels and a tenth
/// of it.
#[derive(Clone)]
struct Frame { wave: [i32; 3], wave240: [i32; 3], ax: i32, ax240: i32, ax480: i32, ay: i32, ay240: i32, ay480: i32, across: i32, radius: i32, tenth: i32 }

/// Everything the timeline says about a frame.
struct Tl {
    one: Frame, two: Frame,
    /// the first blob's grid and splits, the second's (when it has come), the triangles of both
    mesh: (i32, i32, i32), mesh2: Option<(i32, i32, i32)>, count: i32,
    /// still to go of the shrink and of the second's closing in (hundredths), 24 + f / 3, the orbit's angle within
    /// half a turn and a third of a turn on
    shrink: i32, second: i32, third24: i32, orbit: i32, orbit240: i32,
}

mod tl {
    pub use super::super::train::imp::{v, Ops, K, V};
}
use tl::{v, Ops, K, V};

/// Within half a turn either way, on CARDIAC: the rest over 960, less 960 from 480 on.
fn half<O: Ops>(o: &mut O, x: &V) -> V {
    let (_, r) = o.divmod(v(x), K(960));
    let under = o.lt(v(&r), K(480));
    let less = o.sum(&[(v(&r), 1)], -960);
    o.sel(&under, v(&r), v(&less))
}

/// C times a smoothstep from frame a over n frames, rounded: with k the frames into it (0..n), C k^2 (3n - 2k) / n^3,
/// rounded half up as fl((2 C k^2 (3n - 2k) + n^3) / (2 n^3)), the division as three exact floors.
fn smooth<O: Ops>(o: &mut O, f: &V, a: i32, n: i32, c: i32) -> V {
    let d = o.sum(&[(v(f), 1)], -a as i64);
    let k = o.clamp(v(&d), 0, n as i64);
    let k2 = o.mul(v(&k), v(&k), 0);
    let t = o.sum(&[(v(&k), -2)], 3 * n as i64);
    let p = o.lin(&[(v(&k2), Some(v(&t)))]);
    let x = o.lin(&[(v(&p), Some(K(2 * c as i64))), (K((n * n * n) as i64), None)]);
    let q = o.div(v(&x), K((n * n) as i64), 0);
    o.divmod(v(&q), K(2 * n as i64)).0
}

/// A turn rate (thousandths of a 960th a frame) times the frame, cut toward zero, within half a turn.
fn step<O: Ops>(o: &mut O, f: &V, k: i32) -> V {
    let p = o.lin(&[(v(f), Some(K(k.abs() as i64)))]);
    let q = o.div(v(&p), K(1000), 0);
    let q = if k < 0 { o.sum(&[(v(&q), -1)], 0) } else { q };
    half(o, &q)
}

/// The timeline for frames 0..n, on CARDIAC, for the final count given.
fn timeline<O: Ops>(o: &mut O, n: usize, last: i32) -> Vec<Tl> {
    let f: V = (0..n as i64).collect();
    // the first blob's grid and splits
    let lon = { let g = smooth(o, &f, GROW.0, GROW.1 - GROW.0, 55); o.sum(&[(v(&g), 1)], 5) };
    let lat = { let g = smooth(o, &f, GROW.0, GROW.1 - GROW.0, 62); o.sum(&[(v(&g), 1)], 3) };
    let fast = { let x = o.lin(&[(v(&f), Some(K(110))), (K((-110 * GROW.1 + (FAST - GROW.1) / 2) as i64), None)]); o.divmod(v(&x), K((FAST - GROW.1) as i64)).0 };
    let mut ticks: Vec<V> = vec![];
    let mut at = FAST;
    for t in TICKS { at += t; ticks.push(o.lt(v(&f), K(at as i64))); }
    ticks.push(o.lt(v(&f), K((at + BEAT) as i64)));
    let ticked = { let terms: Vec<_> = ticks.iter().map(|t| (v(t), -1)).collect(); o.sum(&terms, 110 + ticks.len() as i64) };
    let (p2, p3) = (o.lt(K(GROW.1 as i64), v(&f)), o.lt(K(FAST as i64), v(&f)));
    let extra = { let z = vec![0; n]; let a = o.sel(&p2, v(&fast), v(&z)); o.sel(&p3, v(&ticked), v(&a)) };
    // the second's
    let come = o.lt(v(&f), K(SECOND.0 as i64));
    let lon2 = { let g = smooth(o, &f, SECOND.0, SECOND.1 - SECOND.0, 55); o.sum(&[(v(&g), 1)], 5) };
    let lat2 = { let g = smooth(o, &f, SECOND.0, SECOND.1 - SECOND.0, 62); o.sum(&[(v(&g), 1)], 3) };
    let most = o.sum(&[(K(last as i64), 1)], -7801 - 2 * 60 * 64)[0];
    let extra2 = { let x = o.lin(&[(v(&f), Some(K(RACE as i64))), (K((-RACE * SECOND.1) as i64), None)]); o.clamp(v(&x), 0, most) };
    // the triangles: 2 lon (lat - 1) + extra, and the second's when it has come
    let count = {
        let (l1, l2) = (o.sum(&[(v(&lat), 1)], -1), o.sum(&[(v(&lat2), 1)], -1));
        let (d1, d2) = (o.sum(&[(v(&lon), 2)], 0), o.sum(&[(v(&lon2), 2)], 0));
        let two = o.lin(&[(v(&d2), Some(v(&l2))), (v(&extra2), None)]);
        let here = o.sum(&[(v(&come), -1)], 1);
        o.lin(&[(v(&d1), Some(v(&l1))), (v(&extra), None), (v(&here), Some(v(&two)))])
    };
    let togo = |o: &mut O, a: i32, b: i32| { let g = smooth(o, &f, a, b - a, 99); o.sum(&[(v(&g), -1)], 99) };
    let (shrink, second) = (togo(o, SHRINK.0, SHRINK.1), togo(o, SECOND.0, SECOND.1));
    let (third, _) = o.divmod(v(&f), K(3));
    let third24 = o.sum(&[(v(&third), 1)], 24);
    let radius = o.sum(&[(v(&third), 1)], 104);
    let (tenth, _) = o.divmod(v(&radius), K(10));
    // the orbit's angle: from 80 short of a turn, speeding up evenly for 24 frames from the second blob's coming
    // (SPIN t^2 / 48), then SPIN a frame
    let orbit = {
        let t = o.sum(&[(v(&f), 1)], -SECOND.0 as i64);
        let tt = o.clamp(v(&t), 0, 24);
        let sq = o.mul(v(&tt), v(&tt), 0);
        let (q, _) = o.divmod(v(&sq), K((48 / SPIN) as i64));
        let late = o.clamp(v(&t), 24, 999);
        o.lin(&[(v(&q), None), (v(&late), Some(K(SPIN as i64))), (K((-80 - SPIN * 24) as i64), None)])
    };
    let orbit_h = half(o, &orbit);
    let orbit240 = { let x = o.sum(&[(v(&orbit), 1)], 240); half(o, &x) };
    // the first blob's waves and turns (2.1, 1.3 and 3 radians a second; 0.7 and 1.1 about x and y; the drift 0.4)
    let rates = [26738, 16552, 38197, 8913, 14006, 5093];
    let one: Vec<V> = rates.iter().map(|&k| step(o, &f, k)).collect();
    let on = |o: &mut O, x: &V, by: i64| { let y = o.sum(&[(v(x), 1)], by); half(o, &y) };
    let w240: Vec<V> = (0..3).map(|i| on(o, &one[i], 240)).collect();
    let (ax240, ax480, ay240, ay480) = (on(o, &one[3], 240), on(o, &one[3], 480), on(o, &one[4], 240), on(o, &one[4], 480));
    // the second's: its waves a third of a turn and more on from the first's, its own turns (1.0 radians a second the
    // other way about x, 0.6 about y); 19 pixels in radius
    let w2: Vec<V> = [320, 170, 610].iter().enumerate().map(|(i, &by)| on(o, &one[i], by)).collect();
    let w2_240: Vec<V> = (0..3).map(|i| on(o, &w2[i], 240)).collect();
    let two: Vec<V> = [-12733, 7958, 7639].iter().map(|&k| step(o, &f, k)).collect();
    let (bx240, bx480, by240, by480) = (on(o, &two[0], 240), on(o, &two[0], 480), on(o, &two[1], 240), on(o, &two[1], 480));
    let i = |x: &V, k: usize| x[k] as i32;
    (0..n).map(|k| Tl {
        one: Frame { wave: [i(&one[0], k), i(&one[1], k), i(&one[2], k)], wave240: [i(&w240[0], k), i(&w240[1], k), i(&w240[2], k)],
            ax: i(&one[3], k), ax240: i(&ax240, k), ax480: i(&ax480, k), ay: i(&one[4], k), ay240: i(&ay240, k), ay480: i(&ay480, k),
            across: i(&one[5], k), radius: i(&radius, k), tenth: i(&tenth, k) },
        two: Frame { wave: [i(&w2[0], k), i(&w2[1], k), i(&w2[2], k)], wave240: [i(&w2_240[0], k), i(&w2_240[1], k), i(&w2_240[2], k)],
            ax: i(&two[0], k), ax240: i(&bx240, k), ax480: i(&bx480, k), ay: i(&two[1], k), ay240: i(&by240, k), ay480: i(&by480, k),
            across: i(&two[2], k), radius: 76, tenth: 7 },
        mesh: (i(&lon, k), i(&lat, k), i(&extra, k)),
        mesh2: if come[k] == 1 { None } else { Some((i(&lon2, k), i(&lat2, k), i(&extra2, k))) },
        count: i(&count, k), shrink: i(&shrink, k), second: i(&second, k), third24: i(&third24, k), orbit: i(&orbit_h, k), orbit240: i(&orbit240, k),
    }).collect()
}

/// The shadow map: a point seen from the light (toward it: 0.80 x and 0.97 y for each unit of depth toward us) lands
/// at x - 0.80 z, y - 0.97 z, moved so the orbit's centre is at the middle of a 192 x 96 map.
const SHEAR: [i32; 2] = [80, 97];

/// The light (toward it: up, left and toward us) and the half-way vector for the highlight, in hundredths.
const LIGHT: [i32; 3] = [-49, -59, -61];
const HALF: [i32; 3] = [-28, -33, -90];

/// A corner as it is drawn: on the screen in quarter pixels, its depth, its normal as we see it (hundredths).
#[derive(Clone, Copy, Default)]
struct Corner { x: i32, y: i32, z: i32, n: [i32; 3] }

struct Machines { angle: Machine, sine: Machine, grid: Machine, phase: Machine, dot: Machine, bulge: Machine, square: Machine, root: Machine,
    unitize: Machine, mix: Machine, rows: Machine, walk: Machine, cross: Machine, depth: Machine, face: Machine, lift: Machine, spec: Machine,
    rim: Machine, lay: Machine, nebula: Machine, stars: Machine, star: Machine, tone: Machine, near: Machine, cell: Machine,
    test: Machine }

/// A term of a dot: c v / 100.
type Pair = (i32, i32);

/// Run a machine that takes a count of items (at most 999, what a card holds) on a list of any length: so many
/// runs, each dealt `before`, its count, `after`, then its items (`size` cards each), punching `punched` cards an
/// item.
fn batched(host: &mut Host, m: &Machine, before: &[i32], after: &[i32], items: &[i32], size: usize, punched: usize) -> Vec<i32> {
    let mut out = vec![];
    for run in items.chunks(999 * size) {
        let count = (run.len() / size) as i32;
        out.extend(host.go1(m, &[before, &[count], after, run], punched * count as usize));
    }
    out
}

impl Machines {
    /// 99 sin of base + each angle (960ths of a turn; base and angle together -960..959).
    fn sines(&self, host: &mut Host, base: i32, angles: &[i32]) -> Vec<i32> {
        let reduced = batched(host, &self.angle, &[base], &[], angles, 1, 2);
        batched(host, &self.sine, &[], &[], &reduced, 2, 1)
    }
    /// For each item, bias + addend + each c v / 100.
    fn dot(&self, host: &mut Host, bias: i32, items: &[(i32, Vec<Pair>)]) -> Vec<i32> {
        if items.is_empty() { return vec![]; }
        let terms = items[0].1.len();
        let mut deck = vec![];
        for (addend, pairs) in items {
            assert_eq!(pairs.len(), terms);
            deck.push(*addend);
            for &(c, v) in pairs { deck.push(c); deck.push(v); }
        }
        batched(host, &self.dot, &[], &[terms as i32, bias], &deck, 1 + 2 * terms, 1)
    }
    /// The same products for every item: c (constant) times each item's values.
    fn dots(&self, host: &mut Host, bias: i32, consts: &[i32], values: &[&[i32]]) -> Vec<i32> {
        let n = values[0].len();
        let items: Vec<(i32, Vec<Pair>)> = (0..n).map(|i| (0, consts.iter().zip(values).map(|(&c, v)| (c, v[i])).collect())).collect();
        self.dot(host, bias, &items)
    }
    /// A value carried along an edge or across a row: start, end, the first step's distance, the whole, steps.
    fn walk(&self, host: &mut Host, items: &[[i32; 5]], total: usize) -> Vec<i32> {
        let mut deck: Vec<i32> = items.iter().flatten().copied().collect();
        deck.push(999);
        host.go1(&self.walk, &[&deck], total)
    }
}

/// The sphere's points at one size of the mesh, and the waves' phases there: worked out once a size.
struct Sphere { u: Vec<[i32; 3]>, w: Vec<[i32; 3]> }

fn sphere(m: &Machines, host: &mut Host, lon: i32, lat: i32) -> Sphere {
    let angles = host.go1(&m.grid, &[&[lat, lon]], (lat + lon) as usize);
    let (bands, around) = angles.split_at(lat as usize);
    let (st, ct) = (m.sines(host, 0, bands), m.sines(host, 240, bands));
    let (sp, cp) = (m.sines(host, 0, around), m.sines(host, 240, around));
    let n = (lat * lon) as usize;
    let lon = lon as usize;
    let ux = m.dot(host, 0, &(0..n).map(|v| (0, vec![(st[v / lon], cp[v % lon])])).collect::<Vec<_>>());
    let uz = m.dot(host, 0, &(0..n).map(|v| (0, vec![(st[v / lon], sp[v % lon])])).collect::<Vec<_>>());
    let u: Vec<[i32; 3]> = (0..n).map(|v| [ux[v], ct[v / lon], uz[v]]).collect();
    let w = batched(host, &m.phase, &[], &[], &u.concat(), 3, 3);
    Sphere { u, w: w.chunks(3).map(|c| [c[0], c[1], c[2]]).collect() }
}

/// The frame's corners: the sphere's points pushed in and out by the waves, turned and placed (about the first
/// blob's centre, or the one given), and their normals.
fn corners(m: &Machines, host: &mut Host, sp: &Sphere, fr: &Frame, centre: Option<[i32; 3]>) -> Vec<Corner> {
    let n = sp.u.len();
    // the turns' sines and cosines (and the sines' negatives), and the drift across
    let t = m.sines(host, 0, &[fr.ax, fr.ax240, fr.ax480, fr.ay, fr.ay240, fr.ay480, fr.across]);
    let (sa, ca, msa, sb, cb, msb) = (t[0], t[1], t[2], t[3], t[4], t[5]);
    let [cx, cy, cz] = match centre {
        Some(c) => c,
        None => [m.dots(host, 384, &[40], &[&[t[6]]])[0], m.dots(host, 192, &[12], &[&[sa]])[0], 0],
    };
    // the waves at each corner
    let col = |k: usize| -> Vec<i32> { sp.w.iter().map(|w| w[k]).collect() };
    let (wa, wb, wc) = (col(0), col(1), col(2));
    let sa_ = m.sines(host, fr.wave[0], &wa);
    let ca_ = m.sines(host, fr.wave240[0], &wa);
    let sb_ = m.sines(host, fr.wave[1], &wb);
    let cb_ = m.sines(host, fr.wave240[1], &wb);
    let sc_ = m.sines(host, fr.wave[2], &wc);
    let cc_ = m.sines(host, fr.wave240[2], &wc);
    let u = |k: usize| -> Vec<i32> { sp.u.iter().map(|u| u[k]).collect() };
    let (u0, u1, u2) = (u(0), u(1), u(2));
    // in and out: the bulge's tenths and units, then the radius pushed
    let p = m.dot(host, 0, &(0..n).map(|v| (0, vec![(sa_[v], cb_[v])])).collect::<Vec<_>>());
    let push = batched(host, &m.bulge, &[], &[], &(0..n).flat_map(|v| [p[v], sc_[v]]).collect::<Vec<_>>(), 2, 2);
    let (tenths, units): (Vec<i32>, Vec<i32>) = (push.iter().step_by(2).copied().collect(), push.iter().skip(1).step_by(2).copied().collect());
    let r = m.dot(host, fr.radius, &(0..n).map(|v| (0, vec![(tenths[v], fr.radius), (units[v], fr.tenth)])).collect::<Vec<_>>());
    let pos = |host: &mut Host, uk: &[i32]| m.dot(host, 0, &(0..n).map(|v| (0, vec![(uk[v], r[v])])).collect::<Vec<_>>());
    let (x, y, z) = (pos(host, &u0), pos(host, &u1), pos(host, &u2));
    // the turn: about x by ax, then about y by ay (offset to the centre, for the corners themselves)
    let turn = |host: &mut Host, x: &[i32], y: &[i32], z: &[i32], at: (i32, i32, i32)| -> [Vec<i32>; 3] {
        let ty = m.dot(host, at.1, &(0..n).map(|v| (0, vec![(ca, y[v]), (msa, z[v])])).collect::<Vec<_>>());
        let z1 = m.dot(host, 0, &(0..n).map(|v| (0, vec![(sa, y[v]), (ca, z[v])])).collect::<Vec<_>>());
        let tx = m.dot(host, at.0, &(0..n).map(|v| (0, vec![(cb, x[v]), (sb, z1[v])])).collect::<Vec<_>>());
        let tz = m.dot(host, at.2, &(0..n).map(|v| (0, vec![(cb, z1[v]), (msb, x[v])])).collect::<Vec<_>>());
        [tx, ty, tz]
    };
    let [sx, sy, sz] = turn(host, &x, &y, &z, (cx, cy, cz));
    // the normal: (k + g . u) u - g, g the waves' slope
    let q1 = m.dot(host, 0, &(0..n).map(|v| (0, vec![(ca_[v], cb_[v])])).collect::<Vec<_>>());
    let q2 = m.dot(host, 0, &(0..n).map(|v| (0, vec![(sa_[v], sb_[v])])).collect::<Vec<_>>());
    let g = [m.dots(host, 0, &[71], &[&q1]), m.dots(host, 0, &[43, 47], &[&q1, &cc_]), m.dots(host, 0, &[-86], &[&q2])];
    let mg = [m.dots(host, 0, &[-71], &[&q1]), m.dots(host, 0, &[-43, -47], &[&q1, &cc_]), m.dots(host, 0, &[86], &[&q2])];
    let s = m.dot(host, 100, &(0..n).map(|v| (tenths[v], vec![(g[0][v], u0[v]), (g[1][v], u1[v]), (g[2][v], u2[v])])).collect::<Vec<_>>());
    let uu = [&u0, &u1, &u2];
    let nn: Vec<Vec<i32>> = (0..3).map(|c| m.dot(host, 0, &(0..n).map(|v| (mg[c][v], vec![(uu[c][v], s[v])])).collect::<Vec<_>>())).collect();
    // made a unit: its size squared, the root, each part over the size
    let sq = batched(host, &m.square, &[], &[], &(0..n).flat_map(|v| [nn[0][v], nn[1][v], nn[2][v]]).collect::<Vec<_>>(), 3, 2);
    let len = batched(host, &m.root, &[], &[], &sq, 2, 1);
    let unit = batched(host, &m.unitize, &[], &[], &(0..n).flat_map(|v| [len[v], nn[0][v], len[v], nn[1][v], len[v], nn[2][v]]).collect::<Vec<_>>(), 2, 1);
    let part = |c: usize| -> Vec<i32> { (0..n).map(|v| unit[3 * v + c]).collect() };
    let [nx, ny, nz] = turn(host, &part(0), &part(1), &part(2), (0, 0, 0));
    (0..n).map(|v| Corner { x: sx[v], y: sy[v], z: sz[v], n: [nx[v], ny[v], nz[v]] }).collect()
}

/// The frame's triangles, in drawing order: the grid's two a square, then the splits, each at the middle of an
/// edge (its corner worked out by the dot machine): triangle 7919 k (mod 7680) for the k-th (on the train's
/// operations), its second half added at the end. With each, the line its second edge lies on if that edge is half
/// of a split one: the whole edge, the way the triangle beside it has it, so that the halves are filled to the same
/// line as their neighbour and no pixel falls between (the middle corner, cut to a whole quarter pixel, is off the
/// line by a little).
fn triangles<O: Ops>(m: &Machines, host: &mut Host, o: &mut O, c: &mut Vec<Corner>, lon: i32, lat: i32, extra: i32) -> (Vec<[usize; 3]>, Vec<Option<[usize; 2]>>) {
    let at = |i: i32, j: i32| (i * lon + j % lon) as usize;
    let mut tris = vec![];
    for i in 0..lat - 1 { for j in 0..lon {
        let (a, b, cc, d) = (at(i, j), at(i, j + 1), at(i + 1, j), at(i + 1, j + 1));
        tris.push([a, cc, b]); tris.push([b, cc, d]);
    } }
    let mut lines = vec![None; tris.len()];
    if extra == 0 { return (tris, lines); }
    let ks: V = (0..extra as i64).collect();
    let x = o.lin(&[(v(&ks), Some(K(7919)))]);
    let (_, cut) = o.divmod(v(&x), K(7680));
    let splits: Vec<(usize, [usize; 3])> = cut.iter().map(|&t| (t as usize, tris[t as usize])).collect();
    let values = |q: &Corner| [q.x, q.y, q.z, q.n[0], q.n[1], q.n[2]];
    let items: Vec<(i32, Vec<Pair>)> = splits.iter().flat_map(|&(_, [_, b, e])| {
        let (p, q) = (values(&c[b]), values(&c[e]));
        (0..6).map(move |k| (p[k], vec![(50, q[k]), (-50, p[k])]))
    }).collect();
    let mids = m.dot(host, 0, &items);
    for (k, &(t, [a, b, e])) in splits.iter().enumerate() {
        let v = &mids[6 * k..6 * k + 6];
        c.push(Corner { x: v[0], y: v[1], z: v[2], n: [v[3], v[4], v[5]] });
        let mid = c.len() - 1;
        tris[t] = [a, b, mid];
        tris.push([a, mid, e]);
        lines[t] = Some([b, e]);
        lines.push(Some([b, e]));
    }
    (tris, lines)
}

/// The normals each triangle carries: the triangle's mean normal, a part at a time (dot), then half the corner's own
/// and half that, in thousandths (mix, counting the triangles). Returns the count the mix machine punched, and for
/// each triangle the nine it punched (each part, each corner).
fn normals(m: &Machines, host: &mut Host, c: &[Corner], tris: &[[usize; 3]]) -> (i32, Vec<i32>) {
    let t = tris.len();
    let items: Vec<(i32, Vec<Pair>)> = tris.iter().flat_map(|tr| (0..3).map(move |k| (0, tr.iter().map(|&v| (33, c[v].n[k])).collect()))).collect();
    let mean = m.dot(host, 0, &items);
    let mut deck = vec![];
    for (i, tr) in tris.iter().enumerate() { for k in 0..3 { for &v in tr { deck.push(c[v].n[k]); deck.push(mean[3 * i + k]); } } }
    deck.push(999);
    let mixed = host.go1(&m.mix, &[&deck], 9 * t + 2);
    (1000 * mixed[9 * t] + mixed[9 * t + 1], mixed[..9 * t].to_vec())
}

/// Fill triangles (on the screen or the shadow map): each corner's place in quarter pixels and K values to carry.
/// Each edge's rows (rows) and the values down it (walk), each row's pixels (cross) and the values across (walk).
/// A half of a split edge (a triangle's second, where `line` gives the whole edge's ends) crosses its rows where the
/// whole edge does: the whole edge's rows and x down it too (rows, walk), and the host takes the cards of the half's
/// rows from those, where the whole edge meets them.
/// Returns the pieces sorted to their pixels, in drawing order: the triangle and its values.
fn raster<const K: usize>(m: &Machines, host: &mut Host, t: usize, corner: impl Fn(usize, usize) -> (i32, i32, [i32; K]),
    line: impl Fn(usize) -> Option<[(i32, i32); 2]>) -> Vec<Vec<(usize, [i32; K])>> {
    // the edges' rows, then the whole edges' the halves lie on
    let lined: Vec<(usize, [(i32, i32); 2])> = (0..t).filter_map(|i| line(i).map(|l| (i, l))).collect();
    let mut deck = vec![];
    for i in 0..t { for e in 0..3 { deck.push(corner(i, e).1); deck.push(corner(i, (e + 1) % 3).1); } }
    for (_, [p, q]) in &lined { deck.push(p.1); deck.push(q.1); }
    deck.push(999);
    let rows = host.go1(&m.rows, &[&deck], (3 * t + lined.len()) * 5);
    // the values down each edge: x and the K values, from its upper end
    let mut walks = vec![];
    let mut total = 0;
    for i in 0..t { for e in 0..3 {
        let r = &rows[5 * (3 * i + e)..5 * (3 * i + e) + 5];
        if r[2] == 0 { continue; }
        let (p, q) = (corner(i, e), corner(i, (e + 1) % 3));
        let (from, to) = if r[0] == 1 { (q, p) } else { (p, q) };
        walks.push([from.0, to.0, r[3], r[4], r[2]]);
        for k in 0..K { walks.push([from.2[k], to.2[k], r[3], r[4], r[2]]); }
        total += (1 + K) * r[2] as usize;
    } }
    for (l, (_, [p, q])) in lined.iter().enumerate() {
        let r = &rows[5 * (3 * t + l)..5 * (3 * t + l) + 5];
        if r[2] == 0 { continue; }
        let (from, to) = if r[0] == 1 { (q, p) } else { (p, q) };
        walks.push([from.0, to.0, r[3], r[4], r[2]]);
        total += r[2] as usize;
    }
    let down = m.walk(host, &walks, total);
    // x down each whole edge, by row
    let mut whole: HashMap<usize, Vec<(i32, i32)>> = HashMap::new();
    let mut on = total - lined.iter().enumerate().map(|(l, _)| rows[5 * (3 * t + l) + 2] as usize).sum::<usize>();
    for (l, &(i, _)) in lined.iter().enumerate() {
        let r = &rows[5 * (3 * t + l)..5 * (3 * t + l) + 5];
        whole.insert(i, (0..r[2]).map(|s| (r[1] + s, down[on + s as usize])).collect());
        on += r[2] as usize;
    }
    // each triangle's rows, from the two edges that meet each: x and the K values at each
    let mut met: Vec<(usize, i32, Vec<Vec<i32>>)> = vec![];   // triangle, row, the two crossings
    let mut at = 0;
    for i in 0..t {
        let mut by_row: Vec<(i32, Vec<Vec<i32>>)> = vec![];
        for e in 0..3 {
            let r = &rows[5 * (3 * i + e)..5 * (3 * i + e) + 5];
            let n = r[2] as usize;
            if n == 0 { continue; }
            for s in 0..n {
                let mut v: Vec<i32> = (0..1 + K).map(|k| down[at + k * n + s]).collect();
                let row = r[1] + s as i32;
                if e == 1 { if let Some(&(_, x)) = whole.get(&i).and_then(|w| w.iter().find(|(y, _)| *y == row)) { v[0] = x; } }
                match by_row.iter_mut().find(|(y, _)| *y == row) { Some((_, l)) => l.push(v), None => by_row.push((row, vec![v])) }
            }
            at += (1 + K) * n;
        }
        sorter::sort(&mut by_row, &[sorter::CARD], |(y, _)| [*y as i64]);
        for (y, l) in by_row { assert_eq!(l.len(), 2, "triangle {i} row {y}"); met.push((i, y, l)); }
    }
    let mut deck: Vec<i32> = met.iter().flat_map(|(_, _, l)| [l[0][0], l[1][0]]).collect();
    deck.push(999);
    let crossed = host.gon(&m.cross, &[&deck]);
    // the values across each row, from the left crossing
    let mut walks = vec![];
    let mut spans = vec![];
    let mut total = 0;
    let mut at = 0;
    for (i, y, l) in &met {
        let (flag, n, k0, width) = (crossed[at], crossed[at + 1] as usize, crossed[at + 2], crossed[at + 3]);
        let xs = crossed[at + 4..at + 4 + n].to_vec();
        at += 4 + n;
        if n == 0 { continue; }
        let (from, to) = if flag == 1 { (&l[1], &l[0]) } else { (&l[0], &l[1]) };
        for k in 1..1 + K { walks.push([from[k], to[k], k0, width, n as i32]); }
        spans.push((*i, *y, xs));
        total += K * n;
    }
    assert_eq!(at, crossed.len());
    let across = m.walk(host, &walks, total);
    // sorted to their pixels, in drawing order
    let mut pixels: Vec<Vec<(usize, [i32; K])>> = vec![vec![]; W2 * H2];
    let mut at = 0;
    for (i, y, xs) in &spans {
        let n = xs.len();
        for (s, &x) in xs.iter().enumerate() {
            assert!(x < WIDTH, "triangle {i} row {y}: pixel {x} past the right edge");
            pixels[(*y * WIDTH + x) as usize].push((*i, core::array::from_fn(|k| across[at + k * n + s])));
        }
        at += K * n;
    }
    pixels
}

/// The nebula and its stars, once: three waves crossing (nebula, angle, sine), their mix brought to 1..97 and
/// squared, a dark purple of that much (dot); then 140 stars from the random numbers (stars, dot), each laid on its
/// row in turn (star).
fn sky(m: &Machines, host: &mut Host) -> Vec<Vec<i32>> {
    let bends = m.sines(host, 0, &(0..HEIGHT).map(|y| half_turn(14 * y)).collect::<Vec<_>>());
    let mut rows = vec![];
    for y in 0..HEIGHT {
        let angles = host.go1(&m.nebula, &[&[y, bends[y as usize]]], 3 * W2);
        let c = m.sines(host, 0, &angles);
        let k: Vec<i32> = (0..3).map(|_| 17).collect();
        let parts: Vec<Vec<i32>> = (0..3).map(|j| (0..W2).map(|x| c[3 * x + j]).collect()).collect();
        let level = m.dots(host, 49, &k, &[&parts[0], &parts[1], &parts[2]]);
        let sq = m.dot(host, 0, &level.iter().map(|&v| (0, vec![(v, v)])).collect::<Vec<_>>());
        let (r, g, b) = (m.dots(host, 12, &[28], &[&sq]), m.dots(host, 3, &[8], &[&sq]), m.dots(host, 16, &[32], &[&sq]));
        rows.push((0..W2).flat_map(|x| [r[x], g[x], b[x]]).collect::<Vec<i32>>());
    }
    let rolls = host.go1(&m.stars, &[], 420);
    let roll = |k: usize| -> Vec<i32> { (0..140).map(|s| rolls[3 * s + k]).collect() };
    let scaled = |host: &mut Host, k: usize, bias: i32, by: i32| m.dot(host, bias, &roll(k).iter().map(|&r| (0, vec![(r, by)])).collect::<Vec<_>>());
    let (xs, ys, vs) = (scaled(host, 0, 0, 192), scaled(host, 1, 0, 96), scaled(host, 2, 140, 110));
    for s in 0..140 {
        let y = ys[s] as usize;
        rows[y] = host.go1(&m.star, &[&[xs[s], vs[s]], &rows[y]], 3 * W2);
    }
    rows
}

pub fn render(demo: &Path, refbin: &str) {
    let r = |n: &str| rust(demo, n);
    let trig = numbers(&demo.join("shared/data/trig.txt"));
    let quarter: Vec<i32> = (0..26).map(|k| trig[2 * k]).collect();
    let widen = |t: &[i16]| t.iter().map(|&v| v as i32).collect::<Vec<i32>>();
    let m = Machines { angle: r("polyangle"), sine: carrying(demo, "polysine", "quarter", &quarter), grid: r("polygrid"), phase: r("polyphase"),
        dot: r("polydot"), bulge: r("polybulge"), square: r("polysquare"), root: r("polyroot"), unitize: r("polyunitize"), mix: r("polymix"),
        rows: r("polyrows"), walk: r("polywalk"), cross: r("polycross"), depth: r("polydepth"), face: r("polyface"), lift: r("polylift"),
        spec: carrying(demo, "polyspec", "specular", &widen(&demokit::tables::SPECULAR)), rim: carrying(demo, "polyrim", "rim", &widen(&demokit::tables::RIM)),
        lay: r("polylay"), nebula: r("polynebula"), stars: r("polystars"), star: r("polystar"), tone: r("py_tone"),
        near: r("py_near"), cell: r("py_cell"), test: r("py_test") };
    // the brag boxes (52 x 37, raw RGB, one a count shown, the gold 7801 and the final count brighter) and the final
    // count
    let boxes = std::fs::read(demo.join("polys/data/boxes.rgb")).unwrap();
    let names: Vec<String> = read(demo, "polys/data/boxes.txt").split_whitespace().map(|s| s.to_string()).collect();
    let last: i32 = read(demo, "polys/data/final.txt").trim().parse().unwrap();
    let frames = END;
    // the timeline, every frame's numbers, on the train's operations; the frames the count first reaches 7801
    // and the final count, found among them
    let mut o = super::train::Cardiac::new(demo);
    let tl = timeline(&mut o, frames, last);
    let first_7801 = tl.iter().position(|t| t.count == 7801).expect("the count never reaches 7801");
    let landed = tl.iter().position(|t| t.count == last).expect("the count never reaches the final count");
    let mut host = Host::new();
    let sky = sky(&m, &mut host);
    let mut spheres: HashMap<(i32, i32), Sphere> = HashMap::new();
    let mut video = Video2::new(frames);
    let mut most = (0, 0);
    let mut second_steps = 0;
    for f in live::frames(frames) {
        let before = host.steps;
        let t_ = &tl[f];
        let mut fr = t_.one.clone();
        let (lon, lat, extra) = t_.mesh;
        // from frame 270, the orbit (sine, dot): its centre (moving 16 pixels right as the first blob shrinks), the
        // way from it to the second blob (across the orbit's plane and toward the light), the first blob on the near
        // side, shrinking; and the shadow map's move, the centre to the middle
        let mut orbit = None;
        if f as i32 >= SHRINK.0 {
            let t = m.sines(&mut host, 0, &[fr.across, fr.ax, t_.orbit, t_.orbit240]);
            let shrink = t_.shrink;
            let middle = [m.dot(&mut host, 448, &[(0, vec![(40, t[0]), (shrink, -64)])])[0], m.dot(&mut host, 192, &[(0, vec![(12, t[1])])])[0], 0];
            let way: Vec<i32> = (0..3).map(|k| m.dot(&mut host, 0, &[(0, vec![(ACROSS[k], t[3]), (LIGHT[k], t[2])])])[0]).collect();
            let move_by = [m.dot(&mut host, -64, &[(0, vec![(-40, t[0]), (shrink, 64)])])[0], m.dot(&mut host, 0, &[(0, vec![(-12, t[1])])])[0]];
            fr.radius = m.dot(&mut host, 80, &[(0, vec![(shrink, t_.third24)])])[0];
            fr.tenth = o.divmod(K(fr.radius as i64), K(10)).0[0] as i32;
            let apart = m.dot(&mut host, 112, &[(0, vec![(shrink, -112)])])[0];
            let centre: Vec<i32> = (0..3).map(|k| m.dot(&mut host, 0, &[(middle[k], vec![(way[k], -apart)])])[0]).collect();
            orbit = Some(([centre[0], centre[1], centre[2]], middle, way, move_by));
        }
        if !spheres.contains_key(&(lon, lat)) { let s = sphere(&m, &mut host, lon, lat); spheres.insert((lon, lat), s); }
        let mut c = corners(&m, &mut host, &spheres[&(lon, lat)], &fr, orbit.as_ref().map(|o| o.0));
        let (mut tris, mut lines) = triangles(&m, &mut host, &mut o, &mut c, lon, lat, extra);
        let first = tris.len();
        // the second blob, on the far side of the centre, closing in (dot): its corners and triangles, drawn after
        // the first's
        let mut move_by = None;
        if let (Some((lon, lat, extra)), Some((_, middle, way, moved))) = (t_.mesh2, &orbit) {
            let apart = m.dot(&mut host, 112, &[(0, vec![(t_.second, 130)])])[0];
            let centre: Vec<i32> = (0..3).map(|k| m.dot(&mut host, 0, &[(middle[k], vec![(way[k], apart)])])[0]).collect();
            if !spheres.contains_key(&(lon, lat)) { let s = sphere(&m, &mut host, lon, lat); spheres.insert((lon, lat), s); }
            let mut c2 = corners(&m, &mut host, &spheres[&(lon, lat)], &t_.two, Some([centre[0], centre[1], centre[2]]));
            let (tris2, lines2) = triangles(&m, &mut host, &mut o, &mut c2, lon, lat, extra);
            let base = c.len();
            tris.extend(tris2.iter().map(|t| t.map(|v| base + v)));
            lines.extend(lines2.iter().map(|l| l.map(|l| l.map(|v| base + v))));
            c.extend(&c2);
            move_by = Some(*moved);
        }
        // fill: the normals each triangle carries (counting them all), then each triangle's pieces on the screen
        let (counted, mixed) = normals(&m, &mut host, &c, &tris);
        assert_eq!(counted as usize, tris.len(), "frame {f}: the mix machine counted {counted}");
        let pixels = raster::<4>(&m, &mut host, tris.len(), |i, j| { let q = &c[tris[i][j]]; (q.x, q.y, [q.z, mixed[9 * i + j], mixed[9 * i + 3 + j], mixed[9 * i + 6 + j]]) },
            |i| lines[i].map(|l| l.map(|v| (c[v].x, c[v].y))));
        let blob = |i: usize| if i < first { 1 } else { 2 };
        // the nearest piece on each pixel: which blob's, its depth and normal
        let mut deck = vec![];
        for p in &pixels { deck.push(p.len() as i32); for (i, q) in p { deck.extend(q); deck.push(blob(*i)); } }
        deck.push(999);
        let near = host.go1(&m.depth, &[&deck], 5 * W2 * H2);
        let covered: Vec<usize> = (0..W2 * H2).filter(|&p| near[5 * p] > 0).collect();
        let k = covered.len();
        // each covered pixel's light
        let faced = batched(&mut host, &m.face, &[], &[], &covered.iter().flat_map(|&p| [near[5 * p + 2], near[5 * p + 3], near[5 * p + 4]]).collect::<Vec<_>>(), 3, 6);
        let part = |j: usize| -> Vec<i32> { (0..k).map(|i| faced[6 * i + j]).collect() };
        let (n0, n1, n2, t0, t1, t2) = (part(0), part(1), part(2), part(3), part(4), part(5));
        let size = m.dot(&mut host, 0, &(0..k).map(|i| (0, vec![(t0[i], n0[i]), (t1[i], n1[i]), (t2[i], n2[i])])).collect::<Vec<_>>());
        let lift = m.dots(&mut host, 499, &[-50], &[&size]);
        let d0 = m.dots(&mut host, 0, &LIGHT, &[&n0, &n1, &n2]);
        let h0 = m.dots(&mut host, 0, &HALF, &[&n0, &n1, &n2]);
        let lifted = batched(&mut host, &m.lift, &[], &[], &(0..k).flat_map(|i| [d0[i], lift[i], h0[i], lift[i], n2[i], lift[i]]).collect::<Vec<_>>(), 2, 1);
        let mut spec = batched(&mut host, &m.spec, &[], &[], &(0..k).map(|i| lifted[3 * i + 1]).collect::<Vec<_>>(), 1, 1);
        let mut rim = batched(&mut host, &m.rim, &[], &[], &(0..k).flat_map(|i| [lifted[3 * i], lifted[3 * i + 2]]).collect::<Vec<_>>(), 2, 2);
        // the shadows: both blobs' corners seen from the light (dot) and their triangles filled on the map with their
        // depth, the nearest to the light kept on each place with its blob (near); each pixel placed on the map (dot,
        // cell) and tested against the card there (test)
        if let Some(move_by) = move_by {
            let z: Vec<i32> = c.iter().map(|q| q.z).collect();
            let seen = |host: &mut Host, k: usize, along: Vec<i32>| m.dot(host, move_by[k], &along.iter().zip(&z).map(|(&a, &z)| (a, vec![(-SHEAR[k], z)])).collect::<Vec<_>>());
            let map_x = seen(&mut host, 0, c.iter().map(|q| q.x).collect());
            let map_y = seen(&mut host, 1, c.iter().map(|q| q.y).collect());
            let on_map = raster::<1>(&m, &mut host, tris.len(), |i, j| { let v = tris[i][j]; (map_x[v], map_y[v], [z[v]]) },
                |i| lines[i].map(|l| l.map(|v| (map_x[v], map_y[v]))));
            let mut deck = vec![];
            for p in &on_map { deck.push(p.len() as i32); for (i, q) in p { deck.push(q[0]); deck.push(blob(*i)); } }
            deck.push(999);
            let map = host.go1(&m.near, &[&deck], 2 * W2 * H2);
            let depth: Vec<i32> = covered.iter().map(|&p| near[5 * p + 1]).collect();
            let along: Vec<Vec<i32>> = (0..2).map(|j| m.dots(&mut host, move_by[j], &[-SHEAR[j]], &[&depth])).collect();
            let deck: Vec<i32> = covered.iter().enumerate().flat_map(|(i, &p)| [(p % W2) as i32, (p / W2) as i32, along[0][i], along[1][i]]).collect();
            let places = batched(&mut host, &m.cell, &[], &[], &deck, 4, 2);
            let deck: Vec<i32> = covered.iter().enumerate().flat_map(|(i, &p)| {
                let (x, y) = (places[2 * i], places[2 * i + 1]);
                let there = if x < 0 { [999, 0] } else { let at = (y * WIDTH + x) as usize; [map[2 * at], map[2 * at + 1]] };
                [depth[i], near[5 * p], there[0], there[1], rim[2 * i], spec[i]]
            }).collect();
            let tested = batched(&mut host, &m.test, &[], &[], &deck, 6, 2);
            for i in 0..k { rim[2 * i] = tested[2 * i]; spec[i] = tested[2 * i + 1]; }
        }
        let light = m.dot(&mut host, 0, &(0..k).map(|i| (rim[2 * i + 1], vec![(14, rim[2 * i]), (18, spec[i])])).collect::<Vec<_>>());
        // each pixel's level and highlight (tone), and its colour from them (dot): the first blob's grey-blue, the
        // second's copper, in hundredths of the level
        let tone = batched(&mut host, &m.tone, &[], &[], &(0..k).flat_map(|i| [light[i], spec[i]]).collect::<Vec<_>>(), 2, 3);
        let parts = |i: usize| if near[5 * covered[i]] == 2 { [86, 60, 45] } else { [75, 86, 86] };
        let channel: Vec<Vec<i32>> = (0..3).map(|c| {
            let add = if c == 1 { 2 } else { 1 };
            m.dot(&mut host, 0, &(0..k).map(|i| (tone[3 * i + add], vec![(parts(i)[c], tone[3 * i])])).collect::<Vec<_>>())
        }).collect();
        let mut lit: Vec<Option<[i32; 3]>> = vec![None; W2 * H2];
        for (i, &p) in covered.iter().enumerate() { lit[p] = Some([channel[0][i], channel[1][i], channel[2][i]]); }
        // the box: the count the mix machine punched (from 7801 gold, hopping; the final count brighter, hopping)
        let name = if counted == last { "bright".to_string() } else if counted == 7801 { "gold".to_string() } else { counted.to_string() };
        let which = names.iter().position(|n| *n == name).unwrap_or_else(|| panic!("no box for {name}"));
        let from = if f >= landed { landed } else { first_7801 };
        let top = BOX.3 + if f >= from && f - from < HOP.len() { HOP[f - from] } else { 0 };
        for y in 0..H2 {
            // the row: the blob's colours where it covers, else the sky's
            let mut rgb = vec![];
            for x in 0..W2 {
                match lit[y * W2 + x] {
                    Some(c) => rgb.extend(c),
                    None => rgb.extend(&sky[y][3 * x..3 * x + 3]),
                }
            }
            let by = y as i32 - top;
            if f >= BOX_FROM && (0..BOX.1 as i32).contains(&by) {
                let at = 3 * (which * BOX.0 * BOX.1 + by as usize * BOX.0);
                let picture: Vec<i32> = boxes[at..at + 3 * BOX.0].iter().map(|&v| v as i32).collect();
                rgb = lay(&m, &mut host, &rgb, BOX.2, &vec![true; BOX.0], &picture);
            }
            let at = (f * H2 + y) * W2 * 3;
            for (i, &v) in rgb.iter().enumerate() { video.buf[at + i] = v as u8; }
        }
        let steps = (host.steps - before) as usize;
        if f >= SECOND.0 as usize { second_steps += steps; }
        most = most.max((steps, f));
    }
    host.steps += o.host.steps; host.runs += o.host.runs;   // the timeline's, on the operations
    host.done();
    let refs: Vec<std::path::PathBuf> = ["shared/data/trig.txt", "polys/data/boxes.rgb", "polys/data/boxes.txt", "polys/data/final.txt"].iter().map(|p| demo.join(p)).collect();
    let refs: Vec<&Path> = refs.iter().map(|p| p.as_path()).collect();
    let twos = frames - SECOND.0 as usize;
    finish("polys", &host, &video, &reference(refbin, &refs), &demo.join("polys/video.rgb"),
        &format!(", {}M a frame on average, at most {}M (frame {}); with two blobs {}M a frame on average", host.steps / frames as u64 / 1_000_000, most.0 / 1_000_000, most.1,
            second_steps / twos / 1_000_000));
}

/// Lay a picture over part of a row (from x, as wide as the mask), where the mask shows.
fn lay(m: &Machines, host: &mut Host, row: &[i32], from: usize, mask: &[bool], picture: &[i32]) -> Vec<i32> {
    let n = mask.len();
    let mut deck = vec![n as i32];
    for i in 0..n { deck.push(mask[i] as i32); deck.extend(&picture[3 * i..3 * i + 3]); deck.extend(&row[3 * (from + i)..3 * (from + i) + 3]); }
    let laid = host.go1(&m.lay, &[&deck], 3 * n);
    let mut out = row.to_vec();
    out[3 * from..3 * (from + n)].copy_from_slice(&laid);
    out
}
