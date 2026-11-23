// Reference for the polys: every frame at 192x96 in ordinary Rust, straight from the formulas: the timeline (the
// meshes' sizes and their splits), each blob's corners morphed, turned and placed, their normals, each triangle
// filled row by row with a depth test, the second blob's shadow (its depth seen from the light, then each pixel of
// the first tested against it), each pixel shaded, the nebula and its stars behind, then the brag box. Whole numbers
// throughout, truncated toward zero as Rust's `/` does; mul(c, v) is c v / 100. Reads the trig table, the boxes,
// their names and the final count from the paths given; writes raw RGB for all frames to stdout.
use std::io::Write;
#[allow(unused_attributes)]
#[path = "../../demokit/src/lib.rs"]
#[allow(dead_code)]
mod demokit;
use demokit::tables::{RIM, SPECULAR};
const W: i32 = 192;
const H: i32 = 96;

// --- the timeline -----------------------------------------------------------------------------------------------
// 20 triangles (5 x 3) until frame 66; the grid grows (smoothstep) to 60 x 65 (7680) by 138; then splits, 110 of
// them by 168, then one more each tick, slower and slower, to 7800; a beat, and one more: 7801.
const GROW: (i32, i32) = (66, 138);
const FAST: i32 = 168;
const TICKS: [i32; 10] = [2, 2, 4, 4, 6, 6, 8, 10, 12, 16];
const BEAT: i32 = 18;
/// How far (0..1) a smoothstep from frame a to b has gone in frame f.
fn smooth(f: i32, (a, b): (i32, i32)) -> f64 {
    let u = ((f as f64 - a as f64) / (b - a) as f64).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}
/// A grid growing from 5 x 3 to 60 x 65: corners around, bands of corners.
fn grid_at(e: f64) -> (i32, i32) { ((5.0 + 55.0 * e).round() as i32, (3.0 + 62.0 * e).round() as i32) }
/// The grid (corners around, bands of corners) and the splits so far, in frame f.
fn mesh_size(f: i32) -> (i32, i32, i32) {
    let grid = grid_at(smooth(f, GROW));
    if f <= GROW.1 { return (grid.0, grid.1, 0); }
    if f <= FAST { return (grid.0, grid.1, (110 * (f - GROW.1) + (FAST - GROW.1) / 2) / (FAST - GROW.1)); }
    let (mut at, mut extra) = (FAST, 110);
    for t in TICKS { at += t; if f >= at { extra += 1; } }
    if f >= at + BEAT { extra += 1; }
    (grid.0, grid.1, extra)
}
// Then the two blobs. From frame 270 the first shrinks to 20 pixels and moves aside (smoothstep, to 290). From 282
// the second comes in from the right, its grid growing the same way to 60 x 65 by 306 as it closes in; then it
// splits, 20 a frame, until the two blobs together are the final count. They orbit each other, faster and faster
// to a turn in 80 frames, until they have gone a whole turn round.
const SHRINK: (i32, i32) = (270, 290);
const SECOND: (i32, i32) = (282, 306);
const RACE: i32 = 20;
const SPIN: i32 = 12;
const END: i32 = SECOND.0 + 92;
/// The second blob's grid and splits in frame f (none before it comes), for the final count given.
fn mesh_size2(f: i32, last: i32) -> Option<(i32, i32, i32)> {
    if f < SECOND.0 { return None; }
    let grid = grid_at(smooth(f, SECOND));
    Some((grid.0, grid.1, (RACE * (f - SECOND.1)).clamp(0, last - 7801 - 2 * 60 * 64)))
}

/// Angles are in 960ths of a turn. The frame's: the three waves (2.1, 1.3 and 3 radians a second), the turns about
/// x and y (0.7 and 1.1), the drift across (0.4); and the blob's radius in quarter pixels (26 pixels, growing by
/// one a second).
struct Frame { wave: [i32; 3], ax: i32, ay: i32, across: i32, radius: i32 }
fn frame(f: i32) -> Frame {
    let step = |k: i32| f * k / 1000 % 960;
    Frame { wave: [step(26738), step(16552), step(38197)], ax: step(8913), ay: step(14006), across: step(5093), radius: 104 + f / 3 }
}

/// The second blob's frame: its waves a third of a turn and more on from the first's, its own turns (1.0 radians a
/// second the other way about x, 0.6 about y); 19 pixels in radius.
fn frame2(f: i32) -> Frame {
    let step = |k: i32| (f * k / 1000).rem_euclid(960);
    let one = frame(f);
    let wave = [(one.wave[0] + 320) % 960, (one.wave[1] + 170) % 960, (one.wave[2] + 610) % 960];
    Frame { wave, ax: step(-12733), ay: step(7958), across: step(7639), radius: 76 }
}

/// How much of a smoothstep is still to go, in hundredths (99 to 0).
fn togo(f: i32, span: (i32, i32)) -> i32 { 99 - (99.0 * smooth(f, span)).round() as i32 }

/// The orbit's angle (960ths of a turn): from 80 short of a turn (the second blob to the right), speeding up evenly
/// for 24 frames from the second blob's coming, then 12 a frame.
fn orbit_angle(f: i32) -> i32 {
    let t = f - SECOND.0;
    -80 + if t <= 0 { 0 } else if t <= 24 { SPIN * t * t / 48 } else { SPIN * 12 + SPIN * (t - 24) }
}

/// The orbit's plane: across it (to the right, a little up and toward us; its shear seen from the light is all
/// across) and toward the light, in hundredths. The blobs are on opposite sides of the centre, 56 pixels apart.
const ACROSS: [i32; 3] = [87, -35, -36];

/// 99 sin of an angle: brought into the first quarter turn (the second half turn is the first's negative, the
/// second quarter the first's mirror), then read from every other card of the trig table, ten to a step, between
/// its steps by a straight line.
struct Trig(Vec<i32>);
impl Trig {
    fn sin(&self, a: i32) -> i32 {
        let mut a = a.rem_euclid(960);
        let below = a >= 480;
        if below { a -= 480; }
        if a > 240 { a = 480 - a; }
        let (at, next) = (self.0[(a / 10 * 2) as usize], self.0[(a / 10 * 2 + 2) as usize]);
        let s = at + (next - at) * (a % 10) / 10;
        if below { -s } else { s }
    }
    fn cos(&self, a: i32) -> i32 { self.sin(a + 240) }
}

fn mul(c: i32, v: i32) -> i32 { c * v / 100 }

/// The light (toward it: up, left and toward us) and the half-way vector for the highlight, in hundredths.
const LIGHT: [i32; 3] = [-49, -59, -61];
const HALF: [i32; 3] = [-28, -33, -90];

/// A corner as it is drawn: on the screen in quarter pixels (x, y), its depth, and its normal as we see it, in
/// hundredths.
#[derive(Clone, Copy)]
struct Corner { x: i32, y: i32, z: i32, n: [i32; 3] }

/// A wave's phase at a point on the sphere (in 960ths of a turn), within half a turn either way.
fn phase(w: i32) -> i32 {
    if w >= 480 { w - 960 } else if w < -480 { w + 960 } else { w }
}

/// The frame's corners: the sphere's points, pushed in and out by the waves, turned and placed.
fn corners(tr: &Trig, fr: &Frame, lon: i32, lat: i32, centre: [i32; 3]) -> Vec<Corner> {
    let (sa, ca, sb, cb) = (tr.sin(fr.ax), tr.cos(fr.ax), tr.sin(fr.ay), tr.cos(fr.ay));
    // the turn: about x by ax, then about y by ay
    let turn = |[x, y, z]: [i32; 3]| -> [i32; 3] {
        let y1 = mul(ca, y) - mul(sa, z);
        let z1 = mul(sa, y) + mul(ca, z);
        [mul(cb, x) + mul(sb, z1), y1, mul(cb, z1) - mul(sb, x)]
    };
    let [cx, cy, cz] = centre;
    let r0 = fr.radius;
    let mut out = vec![];
    for i in 0..lat {
        let theta = 480 * (2 * i + 1) / (2 * lat);
        for j in 0..lon {
            let phi = 960 * j / lon;
            let (st, ct, sp, cp) = (tr.sin(theta), tr.cos(theta), tr.sin(phi), tr.cos(phi));
            let u = [mul(st, cp), ct, mul(st, sp)];
            // the waves' phases here: 5x + 3y, 6z and 9y 960ths (about 3.2x + 1.9y, 3.9z and 5.8y radians)
            let w = [phase(5 * u[0] + 3 * u[1]), phase(6 * u[2]), phase(9 * u[1])];
            let (sa_, ca_) = (tr.sin(fr.wave[0] + w[0]), tr.cos(fr.wave[0] + w[0]));
            let (sb_, cb_) = (tr.sin(fr.wave[1] + w[1]), tr.cos(fr.wave[1] + w[1]));
            let (sc_, cc_) = (tr.sin(fr.wave[2] + w[2]), tr.cos(fr.wave[2] + w[2]));
            // in and out by 0.22 sin A cos B + 0.08 sin C, in thousandths
            let p = mul(sa_, cb_);
            let dk = 2 * p + 2 * p / 10 + 8 * sc_ / 10;
            let r = r0 + mul(dk / 10, r0) + mul(dk % 10, r0 / 10);
            let [x, y, z] = turn([mul(u[0], r), mul(u[1], r), mul(u[2], r)]);
            // the normal: k u less the waves' slope g along the surface, (k + g . u) u - g, in hundredths
            let q1 = mul(ca_, cb_);
            let q2 = mul(sa_, sb_);
            let g = [mul(71, q1), mul(43, q1) + mul(47, cc_), -mul(86, q2)];
            let gu = mul(g[0], u[0]) + mul(g[1], u[1]) + mul(g[2], u[2]);
            let s = 100 + dk / 10 + gu;
            let n: [i32; 3] = core::array::from_fn(|c| mul(u[c], s) - g[c]);
            let len = isqrt(n.iter().map(|v| v * v).sum());
            let n = turn(n.map(|v| v * 100 / len));
            out.push(Corner { x: cx + x, y: cy + y, z: cz + z, n });
        }
    }
    out
}

fn isqrt(n: i32) -> i32 { let mut r = 0; while (r + 1) * (r + 1) <= n { r += 1; } r }

/// The frame's triangles, in drawing order: the grid's two a square, then the splits, each at the middle of an
/// edge: triangle 7919 k (mod 7680) for the k-th, its second half added at the end. With each, the line its second
/// edge lies on, if it's half of a split edge: the whole edge, which the triangle beside it has unsplit, so the
/// halves are filled to the same line as their neighbour and no pixel falls between (the middle corner, cut to a
/// whole quarter pixel, is off the line by a little).
fn triangles(c: &mut Vec<Corner>, lon: i32, lat: i32, extra: i32) -> (Vec<[usize; 3]>, Vec<Option<[usize; 2]>>) {
    let at = |i: i32, j: i32| (i * lon + j % lon) as usize;
    let mut tris = vec![];
    for i in 0..lat - 1 { for j in 0..lon {
        let (a, b, cc, d) = (at(i, j), at(i, j + 1), at(i + 1, j), at(i + 1, j + 1));
        tris.push([a, cc, b]); tris.push([b, cc, d]);
    } }
    let mid = |p: i32, q: i32| p + mul(50, q) - mul(50, p);
    let mut halves = vec![];
    for k in 0..extra {
        let t = (k * 7919 % 7680) as usize;
        let [a, b, e] = tris[t];
        halves.push([a, b, e]);
        let (p, q) = (c[b], c[e]);
        c.push(Corner { x: mid(p.x, q.x), y: mid(p.y, q.y), z: mid(p.z, q.z), n: core::array::from_fn(|l| mid(p.n[l], q.n[l])) });
        let m = c.len() - 1;
        tris[t] = [a, b, m];
        tris.push([a, m, e]);
    }
    let mut lines = vec![None; tris.len() - extra as usize];
    for k in 0..extra { let [_, b, e] = halves[k as usize]; lines[(k * 7919 % 7680) as usize] = Some([b, e]); }
    lines.extend((0..extra).map(|k| { let [_, b, e] = halves[k as usize]; Some([b, e]) }));
    (tris, lines)
}

/// What a pixel holds once a triangle has covered it: its depth, its normal (in thousandths) and which blob's.
#[derive(Clone, Copy)]
struct Frag { z: i32, n: [i32; 3], blob: i32 }

/// A corner of a triangle being filled: on screen (or on the shadow map), and K values to carry across it: depth,
/// and on the screen the normal (half the corner's own, half the triangle's: the mean of its corners', 0.33 of
/// each), in thousandths.
#[derive(Clone, Copy)]
struct Vtx<const K: usize> { x: i32, y: i32, a: [i32; K] }

/// a + (b - a) k / d, toward zero: a value carried along an edge or across a row.
fn lerp(a: i32, b: i32, k: i32, d: i32) -> i32 { a + (b - a) * k / d }

/// Fill a triangle: each row whose middle (4 y + 2) is from an edge's upper end's height to short of its lower's
/// meets two of the edges; each pixel whose middle (4 x + 2) is from the left one to short of the right is the
/// triangle's, with its values carried down the edges and across the row, handed to `put` with the pixel. A half of
/// a split edge (the second, if `line` is given) crosses its rows where the whole edge, the line, does (on the rows the
/// line meets: on the shadow map, the middle corner, placed from its own place on the screen, can be a row past it).
fn fill<const K: usize>(v: [Vtx<K>; 3], line: Option<[(i32, i32); 2]>, mut put: impl FnMut(usize, [i32; K])) {
    let edges = [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])].map(|(p, q)| if p.y <= q.y { (p, q) } else { (q, p) });
    let line = line.map(|[p, q]| if p.1 <= q.1 { (p, q) } else { (q, p) });
    for y in 0..H {
        let yc = 4 * y + 2;
        let met: Vec<(i32, [i32; K])> = edges.iter().enumerate().filter(|(_, (p, q))| p.y <= yc && yc < q.y).map(|(e, (p, q))| {
            let (k, d) = (yc - p.y, q.y - p.y);
            let x = match line { Some((lp, lq)) if e == 1 && lp.1 <= yc && yc < lq.1 => lerp(lp.0, lq.0, yc - lp.1, lq.1 - lp.1), _ => lerp(p.x, q.x, k, d) };
            (x, core::array::from_fn(|i| lerp(p.a[i], q.a[i], k, d)))
        }).collect();
        if met.is_empty() { continue; }
        assert_eq!(met.len(), 2);
        let (l, r) = if met[0].0 <= met[1].0 { (met[0], met[1]) } else { (met[1], met[0]) };
        for x in ((l.0 + 1).div_euclid(4)).max(0)..W {
            let xc = 4 * x + 2;
            if xc >= r.0 { break; }
            let a: [i32; K] = core::array::from_fn(|i| lerp(l.1[i], r.1[i], xc - l.0, r.0 - l.0));
            put((y * W + x) as usize, a);
        }
    }
}

/// A pixel's colour from its normal (in thousandths). Facing away, its depth part is turned round (the rest kept, so a
/// normal tipped past the blob's edge keeps the light of its side). Its size squared (each part a
/// tenth at a time times itself) says how far short of a unit it is: the lift is half the shortfall, and the
/// diffuse light (the normal's dot with the light), the highlight's dot (with the half-way vector) and its depth are
/// each brought up by it, one step toward a unit. The highlight is that dot to the 24th power, from a table; the
/// rim, light catching the edge, 0.35 (1 - |depth|)^3, from another, in tenths of a step. They make a level in
/// two-hundredths, 0.2 + 0.7 diffuse + 0.9 highlight + the rim, at most 250 (1.25), smooth (no bands); the colour
/// is a grey-blue of that level (0.75, 0.86, 0.86 of it), with the highlight added on its own; the second blob's
/// copper (0.86, 0.6, 0.45). In the second blob's shadow, the first blob has no diffuse light and no highlight: only
/// the ambient 0.2 and the rim.
fn shade(n: [i32; 3], blob: i32, lit: bool) -> [i32; 3] {
    let n = if n[2] > 0 { [n[0], n[1], -n[2]] } else { n };
    let size = mul(n[0] / 10, n[0]) + mul(n[1] / 10, n[1]) + mul(n[2] / 10, n[2]);
    let lift = 499 - mul(50, size);
    let up = |v: i32| (v + mul(v / 10, lift)).clamp(-999, 999);
    let dot = |l: [i32; 3]| mul(l[0], n[0]) + mul(l[1], n[1]) + mul(l[2], n[2]);
    let (d, h, e) = (up(dot(LIGHT)).max(0), up(dot(HALF)), up(n[2]));
    let d = if lit { d } else { 0 };
    let s = if h < 800 || !lit { 0 } else {
        let (i, t) = (((h - 800) / 10) as usize, (h - 800) % 10);
        SPECULAR[i] as i32 + (SPECULAR[i + 1] - SPECULAR[i]) as i32 / 10 * t
    };
    let rim = RIM[(2 * (-e / 10) / 10) as usize] as i32;
    let v = (40 + mul(14, d) + mul(18, s) + rim).min(250);
    let spark = 4 * (s / 10) / 10;
    if blob == 2 { return [mul(86, v) + spark, mul(60, v) + 3 * (s / 10) / 10, mul(45, v) + spark]; }
    [mul(75, v) + spark, mul(86, v) + 3 * (s / 10) / 10, mul(86, v) + spark]
}

/// The nebula and its stars, once. Three waves crossing: angles 9 x + 1.5 (99 sin 14 y), 17 y + 7 x and 12 (x + y)
/// (about x / 17 + sin y / 11, y / 9 + x / 23 and (x + y) / 13 radians); 0.17 of each added to 49 (1..97) and
/// squared, a dark purple of that much. Then 140 stars, placed and lit by the random numbers from seed 801.
fn sky(tr: &Trig) -> Vec<[i32; 3]> {
    let mut sky = vec![[0; 3]; (W * H) as usize];
    for y in 0..H { for x in 0..W {
        let bend = tr.sin(14 * y);
        let (c1, c2, c3) = (tr.sin(9 * x + bend + bend / 2), tr.sin(17 * y + 7 * x), tr.sin(12 * (x + y)));
        let c = 49 + mul(17, c1) + mul(17, c2) + mul(17, c3);
        let c = mul(c, c);
        sky[(y * W + x) as usize] = [12 + mul(28, c), 3 + mul(8, c), 16 + mul(32, c)];
    } }
    let mut seed = 801i16;
    let mut roll = |n: i32| { seed = demokit::lcg::next(seed); mul(seed as i32 / 10, n) };
    for _ in 0..140 {
        let x = roll(192); let y = roll(96); let v = 140 + roll(110);
        sky[(y * W + x) as usize] = [v, v, v];
    }
    sky
}

/// The shadow map: each point seen from the light (toward it: 0.80 x and 0.97 y for each unit of depth toward us)
/// lands at x - 0.80 z, y - 0.97 z, moved so the orbit's centre is at the middle of a 192 x 96 map; the light's ray
/// through a point keeps those, and its depth falls toward the light.
const SHEAR: [i32; 2] = [80, 97];
fn on_map(p: [i32; 3], move_by: [i32; 2]) -> [i32; 2] { [p[0] - mul(SHEAR[0], p[2]) + move_by[0], p[1] - mul(SHEAR[1], p[2]) + move_by[1]] }

/// A triangle's corners as they are filled on the screen: depth, and the normal carried (5 (corner + mean)).
fn screen_vtx(c: &[Corner], t: [usize; 3]) -> [Vtx<4>; 3] {
    let p = [c[t[0]], c[t[1]], c[t[2]]];
    let mean: [i32; 3] = core::array::from_fn(|l| mul(33, p[0].n[l]) + mul(33, p[1].n[l]) + mul(33, p[2].n[l]));
    p.map(|q| Vtx { x: q.x, y: q.y, a: [q.z, 5 * (q.n[0] + mean[0]), 5 * (q.n[1] + mean[1]), 5 * (q.n[2] + mean[2])] })
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let tr = Trig(std::fs::read_to_string(&a[1]).unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect());
    let boxes = std::fs::read(&a[2]).unwrap();
    let names: Vec<String> = std::fs::read_to_string(&a[3]).unwrap().split_whitespace().map(|s| s.to_string()).collect();
    let last: i32 = std::fs::read_to_string(&a[4]).unwrap().trim().parse().unwrap();
    const BOX: (usize, usize) = (52, 37);
    let sky = sky(&tr);
    let count = |f: i32| {
        let (lon, lat, e) = mesh_size(f);
        2 * lon * (lat - 1) + e + mesh_size2(f, last).map_or(0, |(lon, lat, e)| 2 * lon * (lat - 1) + e)
    };
    let first_7801 = (0..).find(|&f| count(f) == 7801).unwrap();
    let landed = (0..).find(|&f| count(f) == last).unwrap();
    let mut out = std::io::stdout().lock();
    for f in 0..END {
        let mut fr = frame(f);
        let (lon, lat, extra) = mesh_size(f);
        // the centre: the first blob's, drifting; from frame 270 the orbit's, moving 16 pixels right as the first
        // blob shrinks
        let mut middle = [384 + mul(40, tr.sin(fr.across)), 192 + mul(12, tr.sin(fr.ax)), 0];
        let mut centre = middle;
        let mut way = [0; 3];
        let mut move_by = [mul(-40, tr.sin(fr.across)), mul(-12, tr.sin(fr.ax))];
        if f >= SHRINK.0 {
            // the way from the centre to the second blob: across the orbit's plane and toward the light
            let a = orbit_angle(f);
            way = core::array::from_fn(|k| mul(ACROSS[k], tr.cos(a)) + mul(LIGHT[k], tr.sin(a)));
            let (shrink, apart) = (togo(f, SHRINK), 112 + mul(togo(f, SHRINK), -112));
            fr.radius = 80 + mul(shrink, 24 + f / 3);
            middle[0] += 64 + mul(shrink, -64);
            move_by[0] += -64 + mul(shrink, 64);
            centre = core::array::from_fn(|k| middle[k] + mul(way[k], -apart));
        }
        let mut c = corners(&tr, &fr, lon, lat, centre);
        let (mut tris, lines) = triangles(&mut c, lon, lat, extra);
        let ends = |c: &[Corner], l: Option<[usize; 2]>| l.map(|[p, q]| [(c[p].x, c[p].y), (c[q].x, c[q].y)]);
        let mut buf: Vec<Option<Frag>> = vec![None; (W * H) as usize];
        let keep = |buf: &mut Vec<Option<Frag>>, blob: i32, at: usize, a: [i32; 4]| {
            if buf[at].map_or(true, |f| a[0] < f.z) { buf[at] = Some(Frag { z: a[0], n: [a[1], a[2], a[3]], blob }); }
        };
        for (&t, &l) in tris.iter().zip(&lines) { fill(screen_vtx(&c, t), ends(&c, l), |at, a| keep(&mut buf, 1, at, a)); }
        // the second blob, drawn after the first, on the far side of the centre; then both blobs' depths on the
        // shadow map, the nearest to the light kept with which blob it is
        let mut map: Vec<Option<(i32, i32)>> = vec![None; (W * H) as usize];
        let mut shadows = false;
        if let Some((lon, lat, extra)) = mesh_size2(f, last) {
            let fr2 = frame2(f);
            let apart = 112 + mul(togo(f, SECOND), 130);
            let mut c2 = corners(&tr, &fr2, lon, lat, core::array::from_fn(|k| middle[k] + mul(way[k], apart)));
            let (tris2, lines2) = triangles(&mut c2, lon, lat, extra);
            for (&t, &l) in tris2.iter().zip(&lines2) { fill(screen_vtx(&c2, t), ends(&c2, l), |at, a| keep(&mut buf, 2, at, a)); }
            for (blob, c, tris, lines) in [(1, &c, &tris, &lines), (2, &c2, &tris2, &lines2)] {
                let seen = |k: usize| { let q = c[k]; on_map([q.x, q.y, q.z], move_by) };
                for (&t, &l) in tris.iter().zip(lines) {
                    let v = t.map(|k| { let [x, y] = seen(k); Vtx { x, y, a: [c[k].z] } });
                    fill(v, l.map(|l| l.map(|k| { let [x, y] = seen(k); (x, y) })), |at, [z]| if map[at].map_or(true, |(m, _)| z < m) { map[at] = Some((z, blob)); });
                }
            }
            tris.extend(tris2);
            shadows = true;
        }
        // each pixel looked up on the map (from its middle and depth): in shadow if the other blob is there nearer
        // the light
        let lit = |at: usize, z: i32, blob: i32| -> bool {
            if !shadows { return true; }
            let [s, r] = on_map([4 * (at as i32 % W) + 2, 4 * (at as i32 / W) + 2, z], move_by);
            if !(0..4 * W).contains(&s) || !(0..4 * H).contains(&r) { return true; }
            map[(r / 4 * W + s / 4) as usize].map_or(true, |(m, b)| b == blob || m >= z)
        };
        let mut img: Vec<[i32; 3]> = buf.iter().zip(&sky).enumerate()
            .map(|(at, (p, s))| p.map_or(*s, |fr| shade(fr.n, fr.blob, lit(at, fr.z, fr.blob)))).collect();
        // the brag box: the count drawn, from half a second; at 7801 it hops and turns gold, and the race on is
        // gold; at the final count it hops again, brighter
        let count = tris.len() as i32;
        if f >= 6 {
            let name = if count == last { "bright".to_string() } else if count == 7801 { "gold".to_string() } else { count.to_string() };
            let k = names.iter().position(|n| *n == name).unwrap();
            let hop = [-4, -4, -4, -2, 0, 1];
            let from = if f >= landed { landed } else { first_7801 };
            let top = 4 + if f >= from && f - from < 6 { hop[(f - from) as usize] } else { 0 };
            for y in 0..BOX.1 { for x in 0..BOX.0 {
                let at = 3 * (k * BOX.0 * BOX.1 + y * BOX.0 + x);
                img[((top + y as i32) * W + 4 + x as i32) as usize] = [boxes[at] as i32, boxes[at + 1] as i32, boxes[at + 2] as i32];
            } }
        }
        let bytes: Vec<u8> = img.iter().flat_map(|p| p.map(|v| { assert!((0..=255).contains(&v)); v as u8 })).collect();
        out.write_all(&bytes).unwrap();
    }
}
