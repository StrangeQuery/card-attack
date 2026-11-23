// Reference for the turn: the Mac Classic pulled back from, spun round and turned box by box into an IBM PC, the dive
// into its screen, then the CGA tunnel at 320x200 with its brag, dithered out to black. Every frame at 1536x768, in
// ordinary Rust, whole numbers throughout, every division rounded down (fl) as the machines round it.
//
// Units: lengths in tenths of a millimetre (the prototype's centimetres times 100), directions in ten-thousandths,
// the spin's sine and cosine in thousandths (999 for one, from the printed table; 1000 exactly when it hasn't turned),
// places on the screen in hundredths of a pixel from the middle, y down. The focal length is 2110 pixels (the
// prototype's 40 degrees across).
//
// The scene is boxes, all turned together, so in their own frame every box is lo..hi on each axis. Each face that
// faces the eye is cut where another box is inside it (a face's part in another box is never seen), into rectangles;
// those rectangles (pieces) never cross, so where two cover the same pixel the one on the eye's side of a plane that
// parts them is in front: no depth per pixel, only comparisons of the boxes' own numbers. A piece's edge takes every
// corner of another piece that lies on it, so pieces that meet share their edges exactly (no gaps between them on the
// screen). Each piece's span on each row is the swarm's (the edges walked; a pixel is in if its middle is), and each
// pixel shows the piece in front of all the others there. A pixel whose neighbour (left, right, up, down; across the
// frame's edges to the other side, as the prototype's roll does) shows another face direction is an outline (24), and
// the empty pixels beside an object its rim (24, 24, 28).
//
// The screen is a quad in front of the case, in the bezel: its picture, the Mac's last (the cold open's tunnel
// running on, with CARD ATTACK over it, as segments/mac draws it) or the CGA tunnel's, is placed exactly at a grid
// every 20 pixels where the eye's ray meets it, and between the grid's points by straight lines. After the dive the
// CGA frames are shown with the last 3D frame's texels: its screen is exactly the 1024x768 the CGA frames fill.
//
// Then the bezel's layer, for the cut's over line on the cold open: frame 0's pixels and a cover a pixel (in 32nds),
// 32 but where the cold open's screen goes (1024x684 at 256, 42), 0 there.
//
//   reference SEGMENTS_DIR > video_hd.rgb + bezel.rgba   (turn/data, mac/data, swarm/data/tables.txt, ui/data: the atlas)
// TURN_RANGES=1 also prints the largest size of each named number.
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

#[path = "../ui/pages.rs"]
#[allow(dead_code)]
mod pages;
#[path = "../ui/text.rs"]
mod text;
#[path = "../mac/desk.rs"]
#[allow(dead_code)]
mod desk;
#[path = "../mac/boom.rs"]
#[allow(dead_code)]
mod boom;
// desk.rs names its own At from the crate's root
#[allow(unused_imports)]
use desk::At;

const W: i64 = 1536;
const H: i64 = 768;
/// The focal length over ten: a place on the screen is 21100 times across (tenths) over depth (units), hundredths.
const FX10: i64 = 21_100;
/// The frames: pulling back, spinning, holding, diving; the CGA tunnel; its fade; black.
const N_OUT: i64 = 24;
const N_SPIN: i64 = 30;
const N_HOLD: i64 = 6;
const N_IN: i64 = 20;
pub const N3D: i64 = N_OUT + N_SPIN + N_HOLD + N_IN;
const N_CGA: i64 = 48;
const FADE: i64 = 12;
const BLACK: i64 = 3;
const FRAMES: i64 = N3D + N_CGA + FADE + BLACK;
/// The brag shows from this frame.
const BRAG_FROM: i64 = N3D + 6;
/// The frames the Mac's picture is on the screen (the morph not yet half way).
const MAC_FRAMES: i64 = 39;
/// The camera: how far it stands back at the Mac's screen, at the PC's (each screen 1024 pixels wide), and out.
const D0: i64 = 3709;
const D1: i64 = 5357;
const FAR: i64 = 12_500;
const CENTRE: [i64; 3] = [0, 1800, 0];
const MAC_AT: [i64; 3] = [0, 2260, -1490];
const PC_AT: [i64; 3] = [0, 3020, -1760];
/// Toward the camera from what it looks at, thousandths: from the front, and from the side (up and to the right).
const FRONT: [i64; 3] = [0, 0, -1000];
const SIDE: [i64; 3] = [380, 199, -904];
/// The light, thousandths.
const LIGHT: [i64; 3] = [-447, 744, -496];
/// The pictures are dealt with their edges repeated this far round (so a grid point off the screen's edge, up to
/// almost half the picture, keeps its true place).
const PAD: i64 = 240;
/// The outline's grey, the rim's.
const INK: [u8; 3] = [24, 24, 24];
const RIM: [u8; 3] = [24, 24, 28];
/// No piece: the empty black.
const NONE: u8 = 6;

type Box3 = ([i64; 3], [i64; 3], [i64; 3]);
const PLATINUM: [i64; 3] = [222, 216, 200];
const PLATINUM_D: [i64; 3] = [196, 190, 174];
const IBM: [i64; 3] = [214, 208, 190];
const IBM_D: [i64; 3] = [150, 146, 136];
const DARK: [i64; 3] = [52, 52, 56];
/// The Mac Classic: the case, the screen's bezel (four boxes), the floppy slot, the foot, the rainbow badge's six
/// stripes (their edges a twentieth of a millimetre up, to whole tenths). Then the IBM PC: the monitor's case, its
/// bezel, the top drive, the system unit, the second drive, five vents, the badge. Box k of the Mac becomes box k of
/// the PC; the PC's badge grows from nothing at its middle.
const MAC: [Box3; 13] = [
    ([-1230, 0, -1450], [1230, 3350, 1450], PLATINUM),
    ([-1080, 2860, -1520], [1080, 3040, -1430], PLATINUM_D),
    ([-1080, 1480, -1520], [1080, 1660, -1430], PLATINUM_D),
    ([900, 1660, -1520], [1080, 2860, -1430], PLATINUM_D),
    ([-1080, 1660, -1520], [-900, 2860, -1430], PLATINUM_D),
    ([-150, 825, -1490], [790, 895, -1430], DARK),
    ([-1100, 0, -1490], [1100, 120, -1190], PLATINUM_D),
    ([-920, 748, -1470], [-800, 773, -1440], [97, 187, 70]),
    ([-920, 723, -1470], [-800, 748, -1440], [253, 184, 39]),
    ([-920, 698, -1470], [-800, 723, -1440], [245, 130, 31]),
    ([-920, 673, -1470], [-800, 698, -1440], [224, 58, 62]),
    ([-920, 648, -1470], [-800, 673, -1440], [150, 61, 151]),
    ([-920, 623, -1470], [-800, 648, -1440], [0, 157, 220]),
];
const PC: [Box3; 14] = [
    ([-1900, 1520, -1700], [1900, 4520, 1900], IBM),
    ([-1600, 3995, -1790], [1600, 4270, -1700], IBM_D),
    ([-1600, 1770, -1790], [1600, 2045, -1700], IBM_D),
    ([1300, 2045, -1790], [1600, 3995, -1700], IBM_D),
    ([-1600, 2045, -1790], [-1300, 3995, -1700], IBM_D),
    ([360, 810, -2090], [1840, 1270, -2030], DARK),
    ([-2500, 0, -2050], [2500, 1440, 2050], IBM),
    ([360, 230, -2090], [1840, 690, -2030], DARK),
    ([-2100, 300, -2075], [-900, 340, -2045], IBM_D),
    ([-2100, 390, -2075], [-900, 430, -2045], IBM_D),
    ([-2100, 480, -2075], [-900, 520, -2045], IBM_D),
    ([-2100, 570, -2075], [-900, 610, -2045], IBM_D),
    ([-2100, 660, -2075], [-900, 700, -2045], IBM_D),
    ([-2000, 1090, -2075], [-1500, 1230, -2045], [40, 70, 140]),
];
/// The screens, quads facing the front: the Mac's 512x342 (its height to a whole tenth), the PC's 4:3.
const MAC_SCREEN: ([i64; 3], [i64; 3]) = ([-900, 1659, -1490], [900, 2861, -1490]);
const PC_SCREEN: ([i64; 3], [i64; 3]) = ([-1300, 2045, -1760], [1300, 3995, -1760]);
/// The screen is box 14 of the scene.
const SCREEN: usize = 14;

fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }
fn isqrt(n: i64) -> i64 { assert!((0..10_000_000).contains(&n), "a square root of {}", n); let mut r = (n as f64).sqrt() as i64; while r * r > n { r -= 1; } while (r + 1) * (r + 1) <= n { r += 1; } r }

static mut RANGES: Option<BTreeMap<&'static str, i64>> = None;
/// Keep the largest size of a named number.
fn seen(name: &'static str, v: i64) -> i64 {
    // SAFETY: one thread
    #[allow(static_mut_refs)]
    unsafe {
        let m = RANGES.get_or_insert_with(BTreeMap::new);
        let e = m.entry(name).or_insert(0);
        if v.abs() > *e { *e = v.abs(); }
    }
    v
}

/// The printed tables the swarm's sums use: 999 sin every thousandth of a turn (a quarter turn), and atan.
struct Tables { trig: Vec<i64>, atan: Vec<i64> }

impl Tables {
    fn read(dir: &str) -> Tables {
        let text = std::fs::read_to_string(format!("{dir}/swarm/data/tables.txt")).unwrap();
        let get = |name: &str| -> Vec<i64> {
            let l = text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap();
            l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect()
        };
        Tables { trig: get("trig"), atan: get("atan") }
    }
    /// 999 sin of an angle (millionths of a turn): the table every thousandth of a turn, between by tenths.
    fn sin(&self, a: i64) -> i64 {
        let t = fl(md(a, 1_000_000), 100);
        let q = t / 2500;
        let mut r = t % 2500;
        if q % 2 == 1 { r = 2500 - r; }
        let (k, fr) = ((r / 10) as usize, r % 10);
        let v = if fr == 0 { self.trig[k] } else { self.trig[k] + (self.trig[k + 1] - self.trig[k]) * fr / 10 };
        if q >= 2 { -v } else { v }
    }
    fn cos(&self, a: i64) -> i64 { self.sin(a + 250_000) }
    /// The angle of (x, y) from the x axis, millionths of a turn (0 for the origin).
    fn atan2(&self, y: i64, x: i64) -> i64 {
        let (ax, ay) = (x.abs(), y.abs());
        if ax == 0 && ay == 0 { return 0; }
        let base = if ay <= ax { self.atan_ratio(fl(ay * 100_000, ax)) } else { 250_000 - self.atan_ratio(fl(ax * 100_000, ay)) };
        match (x >= 0, y >= 0) {
            (true, true) => base,
            (false, true) => 500_000 - base,
            (false, false) => 500_000 + base,
            (true, false) => md(1_000_000 - base, 1_000_000),
        }
    }
    fn atan_ratio(&self, rho: i64) -> i64 {
        let (k, tenths) = ((rho / 50) as usize, rho % 50 / 5);
        if tenths == 0 { self.atan[k] } else { self.atan[k] + (self.atan[k + 1] - self.atan[k]) * tenths / 10 }
    }
}

/// The smoothstep of x (thousandths, held to 0..1000), in thousandths: 1000 at 1000.
fn smooth(x: i64) -> i64 {
    let x = x.clamp(0, 1000);
    fl(fl(x * x, 10) * (3000 - 2 * x), 100_000)
}
/// a + (b - a) u / 1000, rounded down.
fn lerp(a: i64, b: i64, u: i64) -> i64 { a + fl((b - a) * u, 1000) }
fn lerp3(a: [i64; 3], b: [i64; 3], u: i64) -> [i64; 3] { std::array::from_fn(|i| lerp(a[i], b[i], u)) }

/// The camera in the boxes' frame: the eye, the basis (across, up, forward; ten-thousandths), the morph (thousandths)
/// and the spin's cosine and sine (thousandths).
struct Cam { eye: [i64; 3], r: [i64; 3], u: [i64; 3], f: [i64; 3], m: i64, cs: [i64; 2] }

/// A vector ten thousand long the way of v (thousandths, about a thousand long): v's length's whole square root a,
/// and the rest; 10000 v / |v| is 20000 a v / (a^2 + |v|^2) to within a millionth (a Newton step), the bottom in
/// hundreds.
fn unit(v: [i64; 3]) -> [i64; 3] {
    let s = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let a = isqrt(s);
    let den = fl(a * a + s, 100);
    v.map(|c| fl(200 * c * a, den))
}

fn camera(t: &Tables, f: i64) -> Cam {
    let (mut m, mut spin) = (0, 0);
    let (tgt, dist, dirn);
    if f < N_OUT {
        // pull back from the Mac's screen
        let u = smooth(fl(1000 * f, N_OUT - 1));
        tgt = lerp3(MAC_AT, CENTRE, u);
        dist = lerp(D0, FAR, u);
        dirn = lerp3(FRONT, SIDE, u);
    } else if f < N_OUT + N_SPIN {
        // spin once round, eased, and turn into the PC from a fifth of the way to four fifths
        let k = f - N_OUT;
        spin = smooth(fl(1000 * k, N_SPIN - 1));
        m = smooth(fl(1000 * (10 * k - 58), 174));
        tgt = CENTRE;
        dist = FAR;
        dirn = SIDE;
    } else {
        // the PC; hold, then dive into its screen
        m = 1000;
        let g = f - N_OUT - N_SPIN;
        let u = if g >= N_HOLD { smooth(fl(1000 * (g - N_HOLD), N_IN - 1)) } else { 0 };
        tgt = lerp3(CENTRE, PC_AT, u);
        dist = lerp(FAR, D1, u);
        dirn = lerp3(SIDE, FRONT, u);
    }
    let n = unit(dirn);
    let eye: [i64; 3] = std::array::from_fn(|i| tgt[i] + fl(n[i] * dist, 10_000));
    let fw = n.map(|v| -v);
    // across: level, at right angles to forward (up x forward, the way of (-dz, 0, dx), made a unit)
    let r = unit([-dirn[2], 0, dirn[0]]);
    let up = [fl(fw[1] * r[2], 10_000), fl(fw[2] * r[0] - fw[0] * r[2], 10_000), fl(-fw[1] * r[0], 10_000)];
    // into the boxes' frame: turned back by the spin (none at no turn or a whole one)
    let (c, s) = if spin == 0 || spin == 1000 { (1000, 0) } else { (t.cos(1000 * spin), t.sin(1000 * spin)) };
    let back = |v: [i64; 3]| [fl(c * v[0] - s * v[2], 1000), v[1], fl(s * v[0] + c * v[2], 1000)];
    Cam { eye: back(eye), r: back(r), u: back(up), f: back(fw), m, cs: [c, s] }
}

/// The scene's boxes at morph m (thousandths): lo, hi, colour; then the screen (box 14).
fn boxes(m: i64) -> Vec<Box3> {
    let mut out = vec![];
    for k in 0..14 {
        let (blo, bhi, bc) = PC[k];
        let (alo, ahi, ac) = if k < 13 { MAC[k] } else {
            // from nothing at its middle
            let mid: [i64; 3] = std::array::from_fn(|i| fl(blo[i] + bhi[i], 2));
            (mid, mid, bc)
        };
        out.push((lerp3(alo, blo, m), lerp3(ahi, bhi, m), lerp3(ac, bc, m)));
    }
    out.push((lerp3(MAC_SCREEN.0, PC_SCREEN.0, m), lerp3(MAC_SCREEN.1, PC_SCREEN.1, m), [0, 0, 0]));
    out
}

/// A box with no size on some axis isn't drawn (and cuts nothing).
fn solid(b: &Box3) -> bool { (0..3).all(|k| b.1[k] - b.0[k] >= 1) }

/// A piece: a rectangle of a face, flat on its axis; which way it faces (side 0 toward lo), whose face it is.
#[derive(Clone, Debug)]
struct Piece { lo: [i64; 3], hi: [i64; 3], axis: usize, side: usize, owner: usize }

/// The two other axes of a face's rectangle.
fn others(axis: usize) -> (usize, usize) { ((axis + 1) % 3, (axis + 2) % 3) }

/// A rectangle (a1..a2 by b1..b2) less the open rectangle (j1..j2 by l1..l2): itself if they don't overlap, else its
/// parts round the overlap: the strip below it and the strip above (whole width), then the parts left and right.
fn cut(r: [i64; 4], h: [i64; 4]) -> Vec<[i64; 4]> {
    let [a1, a2, b1, b2] = r;
    let [j1, j2, l1, l2] = h;
    if j2 <= a1 || j1 >= a2 || l2 <= b1 || l1 >= b2 { return vec![r]; }
    let (h1, h2, g1, g2) = (a1.max(j1), a2.min(j2), b1.max(l1), b2.min(l2));
    let mut out = vec![];
    if g1 > b1 { out.push([a1, a2, b1, g1]); }
    if g2 < b2 { out.push([a1, a2, g2, b2]); }
    if h1 > a1 { out.push([a1, h1, g1, g2]); }
    if h2 < a2 { out.push([h2, a2, g1, g2]); }
    out
}

/// The pieces of every face that faces the eye: box by box, axis by axis, lo side then hi; the screen's front.
fn pieces(bx: &[Box3], eye: [i64; 3]) -> Vec<Piece> {
    let mut out = vec![];
    for (b, bb) in bx.iter().enumerate() {
        if b != SCREEN && !solid(bb) { continue; }
        for axis in 0..3 {
            for side in 0..2 {
                if b == SCREEN && (axis != 2 || side != 0) { continue; }
                let c = if side == 0 { bb.0[axis] } else { bb.1[axis] };
                if !(if side == 0 { eye[axis] < c } else { eye[axis] > c }) { continue; }
                let (i, j) = others(axis);
                let mut rects = vec![[bb.0[i], bb.1[i], bb.0[j], bb.1[j]]];
                for (o, ob) in bx.iter().enumerate().take(SCREEN) {
                    if o == b || !solid(ob) || !(ob.0[axis] < c && c < ob.1[axis]) { continue; }
                    rects = rects.into_iter().flat_map(|r| cut(r, [ob.0[i], ob.1[i], ob.0[j], ob.1[j]])).collect();
                }
                for [a1, a2, b1, b2] in rects {
                    let (mut lo, mut hi) = ([0; 3], [0; 3]);
                    lo[axis] = c; hi[axis] = c;
                    lo[i] = a1; hi[i] = a2; lo[j] = b1; hi[j] = b2;
                    out.push(Piece { lo, hi, axis, side, owner: b });
                }
            }
        }
    }
    out
}

/// A piece's corners round its rectangle, with every other piece's corner that lies on one of its edges put in.
fn outlines(ps: &[Piece]) -> Vec<Vec<[i64; 3]>> {
    let corners = |p: &Piece| -> [[i64; 3]; 4] {
        let (i, j) = others(p.axis);
        let at = |a: i64, b: i64| { let mut v = p.lo; v[i] = a; v[j] = b; v };
        [at(p.lo[i], p.lo[j]), at(p.hi[i], p.lo[j]), at(p.hi[i], p.hi[j]), at(p.lo[i], p.hi[j])]
    };
    let all: BTreeSet<[i64; 3]> = ps.iter().flat_map(|p| corners(p)).collect();
    ps.iter().map(|p| {
        let c = corners(p);
        let mut pts = vec![];
        for e in 0..4 {
            let (a, b) = (c[e], c[(e + 1) % 4]);
            pts.push(a);
            let t = (0..3).find(|&k| a[k] != b[k]).unwrap();
            let mut on: Vec<[i64; 3]> = all.iter().filter(|v| (0..3).all(|k| k == t || v[k] == a[k]) && v[t] > a[t].min(b[t]) && v[t] < a[t].max(b[t])).copied().collect();
            on.sort_by_key(|v| if b[t] > a[t] { v[t] } else { -v[t] });
            pts.extend(on);
        }
        pts
    }).collect()
}

/// Whether piece p is in front of piece q where both are seen: along the first axis on which they don't overlap, the
/// one on the eye's side of the plane between them. (None when they overlap on every axis: never, the reference
/// checks.)
fn front(p: &Piece, q: &Piece, eye: [i64; 3]) -> Option<bool> {
    for k in 0..3 {
        if p.hi[k] <= q.lo[k] { return Some(eye[k] <= p.hi[k]); }
        if q.hi[k] <= p.lo[k] { return Some(eye[k] >= p.lo[k]); }
    }
    None
}

/// A point of the boxes' frame on the screen (hundredths of a pixel from the middle, across and down): across and up
/// in tenths of a unit and the depth in units, from the eye along the basis, then 21100 times across over depth.
fn project(cam: &Cam, p: [i64; 3]) -> (i64, i64) {
    let d: [i64; 3] = std::array::from_fn(|k| p[k] - cam.eye[k]);
    let dot = |b: [i64; 3]| d[0] * b[0] + d[1] * b[1] + d[2] * b[2];
    let (x, y, z) = (fl(seen("a point along the basis", dot(cam.r)), 1000), fl(dot(cam.u), 1000), fl(seen("a point along the basis", dot(cam.f)), 10_000));
    assert!(z >= 1000, "a corner too near the eye: {}", z);
    seen("a point across or up (tenths)", x.max(y.abs()));
    seen("a depth", z);
    (seen("a place on the screen", fl(FX10 * x, z)), seen("a place on the screen", fl(-FX10 * y, z)))
}

/// A polygon's corners with its tall edges cut (1000 pixels or more: at the middle, rounded down, and again), as the
/// swarm's.
fn split_tall(pts: &[(i64, i64)]) -> Vec<(i64, i64)> {
    fn edge(out: &mut Vec<(i64, i64)>, a: (i64, i64), c: (i64, i64)) {
        if (c.1 - a.1).abs() >= 100_000 {
            let m = (fl(a.0 + c.0, 2), fl(a.1 + c.1, 2));
            edge(out, a, m);
            edge(out, m, c);
        } else {
            out.push(a);
        }
    }
    let mut out = vec![];
    for i in 0..pts.len() { edge(&mut out, pts[i], pts[(i + 1) % pts.len()]); }
    out
}

/// The pixels a polygon covers on row cy (from the middle): [left, right), as the swarm's.
fn span(pts: &[(i64, i64)], cy: i64) -> Option<(i64, i64)> {
    let yc = 100 * cy + 50;
    let mut b = vec![];
    let n = pts.len();
    for i in 0..n {
        let (a, c) = (pts[i], pts[(i + 1) % n]);
        let (top, bot) = if a.1 <= c.1 { (a, c) } else { (c, a) };
        if top.1 == bot.1 || yc < top.1 || yc >= bot.1 { continue; }
        let x = top.0 + fl((bot.0 - top.0) * (yc - top.1), bot.1 - top.1);
        b.push(fl(x + 49, 100).clamp(-768, 768));
    }
    if b.is_empty() { return None; }
    assert!(b.len() % 2 == 0, "a polygon's row met {} edges", b.len());
    let (l, r) = (*b.iter().min().unwrap(), *b.iter().max().unwrap());
    if l == r { None } else { Some((l, r)) }
}

/// The screen's picture on a grid of points 20 pixels apart (at the middles of pixels 20 i - 768 across and
/// 20 j - 384 down), over the screen's box: at each the place on the picture (thousandths of a texel, the padding
/// added) where the eye's ray meets the screen's plane. The ray's way in the boxes' frame is X r + Y u + 21100 f (X, Y
/// the point's place in tenths of a pixel, up for Y): across in thousands (dx), along z in ten-thousands (dz). It goes
/// the plane's depth from the eye along z, so it meets the plane 10000 dx / dz hundred-thousandths of that depth
/// across from the eye (in hundred-thousandths, or ten-thousandths past 9.9; held when dz is under 100 or the ratio past 99): from lo, H hundredths of a unit; the
/// picture's place is 10 (tw H / the screen's width), held to half a picture past either side and then to the
/// padding. Between the points, by straight lines: down each column, the top point's place plus the change times the
/// row's count over 20 (rounded down); then across, the same between the two columns' places on the row.
struct Map { i0: i64, j0: i64, u: Vec<Vec<i64>>, v: Vec<Vec<i64>> }

impl Map {
    fn column(g: &[Vec<i64>], j: usize, i: usize, fy: i64) -> i64 { g[j][i] + fl((g[j + 1][i] - g[j][i]) * fy, 20) }
    fn at(&self, cx: i64, cy: i64) -> (i64, i64) {
        let (i, fx, j, fy) = (fl(cx + 768, 20) - self.i0, md(cx + 768, 20), fl(cy + 384, 20) - self.j0, md(cy + 384, 20));
        let (i, j) = (i as usize, j as usize);
        let walk = |g: &[Vec<i64>]| { let (a, b) = (Map::column(g, j, i, fy), Map::column(g, j, i + 1, fy)); a + fl((b - a) * fx, 20) };
        (walk(&self.u), walk(&self.v))
    }
}

/// One way of a grid point: the ray's part across (dx) and along (dz) in ten-thousands; the eye's place less the
/// screen's lo; the plane's depth; the screen's size that way and the picture's: the place, thousandths of a texel.
fn grid_place(dx: i64, dz: i64, from_lo: i64, depth: i64, size: i64, tex: i64, down: bool) -> i64 {
    let held = dz < 100 || dx.abs() > 990 * dz;
    let h = if held { if dx >= 0 { 150 * size } else { -50 * size } } else if dx.abs() > 99 * dz {
        // a ray nearly along the plane: the ratio in ten-thousandths
        let ratio = seen("a grid point's ratio", fl(1000 * dx, dz));
        let (hi, lo) = (fl(ratio, 1000), md(ratio, 1000));
        seen("a hit's hundredths", 100 * from_lo + 10 * depth * hi + fl(depth * lo, 100))
    } else {
        let ratio = seen("a grid point's ratio", fl(10_000 * dx, dz));
        let (hi, lo) = (fl(ratio, 1000), md(ratio, 1000));
        seen("a hit's hundredths", 100 * from_lo + depth * hi + fl(depth * lo, 1000))
    };
    let h = h.clamp(-50 * size, 150 * size);
    let h = if down { 100 * size - h } else { h };
    let p = 1000 * PAD + 10 * fl(seen("a grid point's product", tex * h), size);
    p.clamp(0, 1000 * (tex + 2 * PAD) - 1)
}

fn screen_map(cam: &Cam, scr: &Box3, (tw, th): (i64, i64), bbox: (i64, i64, i64, i64)) -> Map {
    let (left, right, top, bottom) = bbox;
    let i0 = fl(left.clamp(-768, 767) + 768, 20);
    let i1 = fl(right.clamp(-768, 767) + 768, 20) + 1;
    let j0 = fl(top.clamp(-384, 383) + 384, 20);
    let j1 = fl(bottom.clamp(-384, 383) + 384, 20) + 1;
    let depth = scr.0[2] - cam.eye[2];
    let (wq, hq) = (scr.1[0] - scr.0[0], scr.1[1] - scr.0[1]);
    let (mut gu, mut gv) = (vec![], vec![]);
    for j in j0..=j1 {
        let (mut ru, mut rv) = (vec![], vec![]);
        for i in i0..=i1 {
            let (x, y) = (200 * i - 7675, 3835 - 200 * j);
            let d: [i64; 3] = std::array::from_fn(|k| fl(seen("a grid ray", x * cam.r[k] + y * cam.u[k] + FX10 * cam.f[k]), if k == 2 { 10_000 } else { 1000 }));
            ru.push(grid_place(d[0], d[2], cam.eye[0] - scr.0[0], depth, wq, tw, false));
            rv.push(grid_place(d[1], d[2], cam.eye[1] - scr.0[1], depth, hq, th, true));
        }
        gu.push(ru);
        gv.push(rv);
    }
    Map { i0, j0, u: gu, v: gv }
}

/// The Mac's last picture, still going, for the turn's first frames: the cold open's one-bit tunnel at its frames on
/// (108 + f: a picture every third frame), CARD ATTACK over it in full, the screen's corners; drawn as
/// segments/mac/reference.rs draws it (Atkinson's dither in eighths toward zero).
fn mac_pictures(demo: &std::path::Path) -> Vec<Vec<u8>> {
    use desk::{BLOCK, TH, TW};
    let (mw, mh) = (desk::W, desk::H);
    let (tabs, cards, turns, sprites) = (desk::tables(demo), desk::titles(demo), desk::dissolve(demo), desk::sprites(demo));
    // the sine table run on (turn/data/mac.txt)
    let sine: Vec<i64> = std::fs::read_to_string(demo.join("turn/data/mac.txt")).unwrap().lines().find(|l| l.starts_with("sine ")).unwrap()
        .split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
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
    let tunnel = |j: usize| -> Vec<bool> {
        let turn = 30 * sine[3 * j] / 100;
        let weight = 16 * sine[2 * j] / 100;
        let travel = (11 * j as i64) % 40;
        let mut value: Vec<i64> = blocks.iter().map(|&(a, [along, twist, light, dark])| {
            let mut u = a + turn + twist * weight / 8;
            if u < 0 { u += 800; }
            if u >= 800 { u -= 800; }
            if (u / 50) % 2 == ((along + travel) / 20) % 2 { light } else { dark }
        }).collect();
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
    };
    (0..MAC_FRAMES as usize).map(|f| {
        let pic = tunnel((108 + f) / 3);
        let mut px: Vec<bool> = (0..mw * mh).map(|i| {
            let (x, y) = (i % mw, i / mw);
            let mut v = pic[(y / BLOCK) * TW + x / BLOCK];
            // CARD ATTACK, dissolved in all the way: a pixel shows when its turn is under 85 times 3
            let c = cards[2][(y / BLOCK) * TW + x / BLOCK];
            if c > 0 && (turns[i] as i16) < 85 * 3 { v = c == 1; }
            v
        }).collect();
        for (name, x, y) in [("corner_tl", 0, 0), ("corner_tr", mw as i64 - 1, 0), ("corner_bl", 0, mh as i64 - 1), ("corner_br", mw as i64 - 1, mh as i64 - 1)] {
            let sp = &sprites[name];
            for j in 0..sp.h { for i in 0..sp.w {
                let (px_, py_) = (x + sp.ox as i64 + i as i64, y + sp.oy as i64 + j as i64);
                if px_ < 0 || py_ < 0 || px_ >= mw as i64 || py_ >= mh as i64 { continue; }
                match sp.px[j * sp.w + i] { 1 => px[py_ as usize * mw + px_ as usize] = false, 2 => px[py_ as usize * mw + px_ as usize] = true, _ => {} }
            } }
        }
        px.iter().flat_map(|&b| { let v = if b { 0 } else { 255 }; [v, v, v] }).collect()
    }).collect()
}

/// The CGA tunnel's tables (turn/data/cga.txt).
struct Cga { bayer: Vec<i64>, hi: Vec<i64>, lo: Vec<i64>, pals: Vec<Vec<u8>>, flicks: Vec<usize> }

fn read_cga(dir: &str) -> Cga {
    let text = std::fs::read_to_string(format!("{dir}/turn/data/cga.txt")).unwrap();
    let nums = |l: &str| -> Vec<i64> { l.split('#').next().unwrap().split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
    let get = |name: &str| nums(text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap());
    Cga { bayer: get("bayer"), hi: get("hi"), lo: get("lo"),
        pals: text.lines().filter(|l| l.starts_with("pal ")).map(|l| nums(l).iter().map(|&v| v as u8).collect()).collect(),
        flicks: get("flicks").iter().map(|&v| v as usize).collect() }
}

/// A pixel of the CGA tunnel, as it is every frame: what it is (0 black, 1 colour 1, 2 a check) by its distance from
/// the middle against the dither's thresholds; for a check, its angle round (a fifth-hundredth of a check, 0..999:
/// two checks), its depth's place along (the same for the rings), and its depth in hundredths (the twist's index).
#[derive(Clone, Copy)]
struct Px { class: i64, a5: i64, v5: i64, dp: i64, bayer: i64 }

fn cga_pixels(t: &Tables, cga: &Cga) -> Vec<Px> {
    let mut out = vec![];
    for y in 0..200 {
        for x in 0..320 {
            let (vx, vy) = (10 * x - 1595, 12 * y - 1194);
            let r2 = vx * vx + vy * vy;
            let b = cga.bayer[((y % 4) * 4 + x % 4) as usize];
            let class = if r2 > cga.hi[b as usize] { 2 } else if r2 > cga.lo[b as usize] { 1 } else { 0 };
            let mut p = Px { class, a5: 0, v5: 0, dp: 0, bayer: b };
            if class == 2 {
                // depth 1500 / r: thousandths, from r in tenths
                let d = fl(15_000_000, isqrt(r2));
                p.dp = fl(d, 10);
                p.v5 = md(fl(d, 7), 1000);
                p.a5 = md(fl(8 * t.atan2(vy, vx), 1000), 1000);
            }
            out.push(p);
        }
    }
    out
}

/// The CGA picture at time tau (frames), its palette's indices: the checks twisting down the tunnel, the brag over it
/// if asked.
fn cga_frame(t: &Tables, px: &[Px], tau: i64, brag: Option<&[u8]>) -> Vec<u8> {
    let p5 = md(fl(300 * t.sin(fl(tau * 119_366, 10)), 1000), 1000);
    let k = 6 * t.sin(fl(tau * 79_577, 10));
    let t5 = md(fl(1100 * tau, 12), 1000);
    let mut out = vec![0u8; 320 * 200];
    for (i, p) in px.iter().enumerate() {
        out[i] = match p.class {
            2 => {
                let tw = md(fl(seen("the twist's product", p.dp * k), 100_000), 1000);
                let a = md(p.a5 + p5 + tw, 1000);
                let b = md(p.v5 + t5, 1000);
                if (a >= 500) == (b >= 500) { 3 } else { 2 }
            }
            c => c as u8,
        };
    }
    if let Some(b) = brag { for i in 0..out.len() { if b[i] != 255 { out[i] = b[i]; } } }
    out
}

/// The brag on the CGA picture: 255 where it isn't, else the colour it puts there (3 the ink, 0 the box).
fn brag_layer(demo: &std::path::Path) -> Vec<u8> {
    let page = &pages::CGA_BRAG;
    let glyphs = text::Glyphs::read(demo);
    let px = text::draw(page, &glyphs, page.chars(), false);
    let mut out = vec![255u8; 320 * 200];
    for v in 0..page.h {
        for u in 0..page.w {
            let c = if px[(v * page.w + u) * 3] > 0 { 3 } else { 0 };
            for dy in 0..page.sy { for dx in 0..page.sx { out[(page.y + page.sy * v + dy) * 320 + page.x + page.sx * u + dx] = c; } }
        }
    }
    out
}

/// A picture as the screen takes it: RGB, w x h.
struct Picture { w: i64, h: i64, rgb: Vec<u8> }
impl Picture {
    fn texel(&self, u: i64, v: i64) -> [u8; 3] {
        let (u, v) = ((u - PAD).clamp(0, self.w - 1), (v - PAD).clamp(0, self.h - 1));
        let k = ((v * self.w + u) * 3) as usize;
        [self.rgb[k], self.rgb[k + 1], self.rgb[k + 2]]
    }
}

fn cga_rgb(cga: &Cga, idx: &[u8], tau: i64) -> Vec<u8> {
    let pal = &cga.pals[cga.flicks[(fl(tau, 12) % 6) as usize]];
    idx.iter().flat_map(|&i| pal[3 * i as usize..3 * i as usize + 3].to_vec()).collect()
}

/// A 3D frame: its pixels, and the screen's texel at each pixel it shows (for the last).
fn frame3d(t: &Tables, f: i64, pic: &Picture, stats: &mut BTreeMap<&'static str, i64>) -> (Vec<u8>, BTreeMap<(i64, i64), (i64, i64)>) {
    let cam = camera(t, f);
    let bx = boxes(cam.m);
    let ps = pieces(&bx, cam.eye);
    let shapes = outlines(&ps);
    let mut proj: BTreeMap<[i64; 3], (i64, i64)> = BTreeMap::new();
    let polys: Vec<Vec<(i64, i64)>> = shapes.iter().map(|s| split_tall(&s.iter().map(|&p| *proj.entry(p).or_insert_with(|| project(&cam, p))).collect::<Vec<_>>())).collect();
    // each piece's colour: its box's, lit as its face's way is
    let [c, s] = cam.cs;
    let colour = |p: &Piece| -> [u8; 3] {
        let sign = if p.side == 0 { -1 } else { 1 };
        let n = match p.axis { 0 => [c, 0, -s], 1 => [0, 1000, 0], _ => [s, 0, c] }.map(|v| sign * v);
        let dot = n[0] * LIGHT[0] + n[1] * LIGHT[1] + n[2] * LIGHT[2];
        let k = 6200 + fl(3800 * dot.max(0), 1_000_000) + fl(800 * (-n[2]).max(0), 1000);
        bx[p.owner].2.map(|v| fl(v * k, 10_000).min(255) as u8)
    };
    let colours: Vec<[u8; 3]> = ps.iter().map(colour).collect();
    let label = |p: &Piece| (2 * p.axis + p.side) as u8;
    // the screen's grid, when it's seen
    let map = if ps.iter().any(|p| p.owner == SCREEN) {
        let scr = &bx[SCREEN];
        let mut xs = vec![];
        let mut ys = vec![];
        for x in [scr.0[0], scr.1[0]] { for y in [scr.0[1], scr.1[1]] { let q = project(&cam, [x, y, scr.0[2]]); xs.push(fl(q.0, 100)); ys.push(fl(q.1, 100)); } }
        Some(screen_map(&cam, scr, (pic.w, pic.h), (*xs.iter().min().unwrap(), *xs.iter().max().unwrap(), *ys.iter().min().unwrap(), *ys.iter().max().unwrap())))
    } else { None };
    // each row: which piece each pixel shows
    let mut shown = vec![vec![usize::MAX; W as usize]; H as usize];
    let mut labels = vec![vec![NONE; W as usize]; H as usize];
    for py in 0..H {
        let cy = py - 384;
        let spans: Vec<(usize, (i64, i64))> = polys.iter().enumerate().filter_map(|(k, p)| span(p, cy).map(|s| (k, s))).collect();
        let mut cuts: Vec<i64> = spans.iter().flat_map(|(_, (l, r))| [*l, *r]).collect();
        cuts.sort();
        cuts.dedup();
        for w in cuts.windows(2) {
            let (x0, x1) = (w[0], w[1]);
            let mut best: Option<usize> = None;
            for &(k, (l, r)) in &spans {
                if l > x0 || r < x1 { continue; }
                best = Some(match best {
                    None => k,
                    Some(b) => match front(&ps[k], &ps[b], cam.eye) {
                        Some(true) => k,
                        Some(false) => b,
                        None => { *stats.entry("pieces that overlap on every axis").or_insert(0) += 1; b }
                    },
                });
            }
            if let Some(b) = best {
                for x in x0..x1 { shown[py as usize][(x + 768) as usize] = b; labels[py as usize][(x + 768) as usize] = label(&ps[b]); }
            }
        }
    }
    // the outlines, and the colours
    let mut img = vec![0u8; (W * H * 3) as usize];
    let mut texels = BTreeMap::new();
    for py in 0..H as usize {
        let (up, down) = ((py + H as usize - 1) % H as usize, (py + 1) % H as usize);
        for px in 0..W as usize {
            let l = labels[py][px];
            let (left, right) = ((px + W as usize - 1) % W as usize, (px + 1) % W as usize);
            let near = [labels[py][left], labels[py][right], labels[up][px], labels[down][px]];
            let c = if l == NONE {
                if near.iter().any(|&n| n != NONE) { RIM } else { [0, 0, 0] }
            } else if near.iter().any(|&n| n != l) {
                INK
            } else {
                let k = shown[py][px];
                if ps[k].owner == SCREEN {
                    let (cx, cy) = (px as i64 - 768, py as i64 - 384);
                    let (u, v) = map.as_ref().unwrap().at(cx, cy);
                    if std::env::var("TURN_RANGES").is_ok() {
                        // how far the grid's straight lines are from the true place (thousandths of a texel)
                        let scr = &bx[SCREEN];
                        let (x, y) = ((100 * cx + 50) as f64 / 10.0, -(100 * cy + 50) as f64 / 10.0);
                        let d: Vec<f64> = (0..3).map(|k| x * cam.r[k] as f64 + y * cam.u[k] as f64 + FX10 as f64 * cam.f[k] as f64).collect();
                        let t = (scr.0[2] - cam.eye[2]) as f64 / d[2];
                        let (hx, hy) = (cam.eye[0] as f64 + t * d[0], cam.eye[1] as f64 + t * d[1]);
                        let tu = 1000.0 * (PAD as f64 + pic.w as f64 * (hx - scr.0[0] as f64) / (scr.1[0] - scr.0[0]) as f64);
                        let tv = 1000.0 * (PAD as f64 + pic.h as f64 * (scr.1[1] as f64 - hy) / (scr.1[1] - scr.0[1]) as f64);
                        let off = ((u as f64 - tu).abs().max((v as f64 - tv).abs())) as i64;
                        seen("the picture's place off the true one (thousandths of a texel)", off);
                    }
                    let (iu, iv) = (fl(u, 1000), fl(v, 1000));
                    texels.insert((px as i64, py as i64), (iu, iv));
                    pic.texel(iu, iv)
                } else { colours[k] }
            };
            img[(py * W as usize + px) * 3..][..3].copy_from_slice(&c);
        }
    }
    // the last frame's screen texels, for the CGA frames: every screen pixel's, whatever it shows
    if f == N3D - 1 {
        texels.clear();
        for py in 0..H { for px in 0..W {
            let k = shown[py as usize][px as usize];
            if k != usize::MAX && ps[k].owner == SCREEN {
                let (u, v) = map.as_ref().unwrap().at(px - 768, py - 384);
                texels.insert((px, py), (fl(u, 1000), fl(v, 1000)));
            }
        } }
    }
    (img, texels)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let demo = std::path::PathBuf::from(dir);
    let t = Tables::read(dir);
    let cga = read_cga(dir);
    let mac = mac_pictures(&demo);
    let only: Option<Vec<i64>> = a.get(2).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let pixels = cga_pixels(&t, &cga);
    let brag = brag_layer(&demo);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut stats = BTreeMap::new();
    let mut last: BTreeMap<(i64, i64), (i64, i64)> = BTreeMap::new();
    let mut bezel = vec![];
    for f in 0..FRAMES {
        let wanted = only.as_ref().map_or(true, |o| o.contains(&f));
        if f < N3D {
            if !wanted && f != N3D - 1 { continue; }
            let cam_m = camera(&t, f).m;
            let pic = if cam_m < 500 {
                assert!(f < MAC_FRAMES, "the Mac's picture at frame {}", f);
                Picture { w: 512, h: 342, rgb: mac[f as usize].clone() }
            } else {
                Picture { w: 320, h: 200, rgb: cga_rgb(&cga, &cga_frame(&t, &pixels, f, None), f) }
            };
            let (img, texels) = frame3d(&t, f, &pic, &mut stats);
            if f == N3D - 1 { last = texels; }
            if f == 0 { bezel = img.clone(); }
            if wanted { out.write_all(&img).unwrap(); }
            continue;
        }
        if !wanted { continue; }
        let mut img = vec![0u8; (W * H * 3) as usize];
        if f < N3D + N_CGA + FADE {
            // the CGA picture where the last 3D frame's screen was; the fade a frame on in time
            let fade = f - (N3D + N_CGA) + 1;
            let tau = if fade >= 1 { f + 1 } else { f };
            let mut idx = cga_frame(&t, &pixels, tau, if f >= BRAG_FROM { Some(&brag) } else { None });
            if fade >= 1 { for (i, p) in pixels.iter().enumerate() { if 3 * (2 * p.bayer + 1) < 8 * fade { idx[i] = 0; } } }
            let pic = Picture { w: 320, h: 200, rgb: cga_rgb(&cga, &idx, tau) };
            for (&(px, py), &(u, v)) in &last { img[((py * W + px) * 3) as usize..][..3].copy_from_slice(&pic.texel(u, v)); }
        }
        out.write_all(&img).unwrap();
    }
    if only.is_none() {
        let mut layer = vec![];
        for y in 0..H { for x in 0..W {
            let i = ((y * W + x) * 3) as usize;
            let screen = (256..1280).contains(&x) && (42..726).contains(&y);
            layer.extend([bezel[i], bezel[i + 1], bezel[i + 2], if screen { 0 } else { 32 }]);
        } }
        out.write_all(&layer).unwrap();
    }
    out.flush().unwrap();
    for (k, v) in &stats { eprintln!("{k}: {v}"); }
    if std::env::var("TURN_RANGES").is_ok() {
        #[allow(static_mut_refs)]
        for (k, v) in unsafe { RANGES.as_ref().unwrap() } { eprintln!("{k}: {v}"); }
    }
}
