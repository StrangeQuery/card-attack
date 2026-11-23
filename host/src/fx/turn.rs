//! The turn, at 1536x768: the camera pulls back from the Mac's screen, the Mac Classic spins round and
//! turns box by box into the IBM PC and its monitor, the camera dives into its screen; then the CGA tunnel at 320x200
//! (the checkerboard in colours 2 and 3, its far end through colour 1 to black by a 4x4 ordered dither, the palette
//! flicking each second), the brag 320 x 200 / 4 COLORS in the lower left, dithered out to black.
//!
//! The 3D frames, a group of 8 at a time: every number of the scene is a sum worked out by the swarm's arithmetic
//! machines (swarm_calc): the camera's path (the smoothsteps, the straight lines, the unit vectors), the spin's sine,
//! the morph (every box's corners and colours a straight line from the Mac's to the PC's), every comparison of the
//! boxes' own numbers and the eye's on each axis (the sorter's order of them), the light on each face, every corner's
//! place on the screen, the screen's grid onto its picture. From the comparisons alone the host cuts each face that
//! faces the eye where other boxes are inside it, joins the pieces' edges where one's corner lies on another's, and
//! picks at each pixel the piece on the eye's side of the plane that parts it from the others: bookkeeping on the
//! machines' order. The swarm's machines walk each piece's edges down the rows (spans), its grid machines walk the
//! screen's picture across its pixels (the host fetches the texels by index), and tn_edge marks each pixel whose
//! neighbour shows another face's way (the outline) or an object beside nothing (the rim).
//!
//! The Mac's picture on the screen is the cold open's running on: its one-bit tunnel's next pictures with CARD ATTACK
//! over it, on its own machines (segments/mac) and its sine table run on (tn_macturn).
//!
//! The CGA tunnel: once, each pixel's distance from the middle against the dither's thresholds, and for a check its
//! angle (atan), its depth (1500 over the square root) and its place along, on the swarm's arithmetic (a quarter of
//! the picture: the rest by the angle's symmetry); each frame, the turn at each depth (tn_twist) and each row's colours
//! (tn_cga); the host fetches each pixel's palette colour by its index. The brag is the text kit's CGA_BRAG page (face
//! 10), laid on at 2x2. After the dive the CGA frames are shown with the last 3D frame's texels (its screen fills the
//! same 1024x768).
use crate::*;
use super::swarm_calc::{Arg, Calc, Items, N, Tables};
use super::text::{pages, Text};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

pub const N_OUT: usize = 24;
pub const N_SPIN: usize = 30;
pub const N_HOLD: usize = 6;
pub const N_IN: usize = 20;
pub const N3D: usize = N_OUT + N_SPIN + N_HOLD + N_IN;
pub const N_CGA: usize = 48;
pub const FADE: usize = 12;
pub const BLACK: usize = 3;
pub const FRAMES: usize = N3D + N_CGA + FADE + BLACK;
const BRAG_FROM: usize = N3D + 6;
const MAC_FRAMES: usize = 39;
const W: usize = 1536;
const H: usize = 768;
pub type VideoHd = cardiac_host::Video<1536, 768>;
/// The pictures are dealt with their edges repeated this far round (see the reference).
const PAD: i64 = 240;
const FX10: i64 = 21_100;
const D0: i64 = 3709;
const D1: i64 = 5357;
const FAR: i64 = 12_500;
const CENTRE: [i64; 3] = [0, 1800, 0];
const MAC_AT: [i64; 3] = [0, 2260, -1490];
const PC_AT: [i64; 3] = [0, 3020, -1760];
const FRONT: [i64; 3] = [0, 0, -1000];
const SIDE: [i64; 3] = [380, 199, -904];
const LIGHT: [i64; 3] = [-447, 744, -496];
const INK: [u8; 3] = [24, 24, 24];
const RIM: [u8; 3] = [24, 24, 28];
const NONE: i32 = 6;
const SCREEN: usize = 14;
/// The twist's table: from this depth (hundredths), so many, in decks of so many.
const DP0: i64 = 700;
const DPS: i64 = 1500;
const DP_DECK: i64 = 750;

type Box3 = ([i64; 3], [i64; 3], [i64; 3]);
const PLATINUM: [i64; 3] = [222, 216, 200];
const PLATINUM_D: [i64; 3] = [196, 190, 174];
const IBM: [i64; 3] = [214, 208, 190];
const IBM_D: [i64; 3] = [150, 146, 136];
const DARK: [i64; 3] = [52, 52, 56];
/// The boxes, as the reference's (segments/turn/reference.rs).
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
const MAC_SCREEN: ([i64; 3], [i64; 3]) = ([-900, 1659, -1490], [900, 2861, -1490]);
const PC_SCREEN: ([i64; 3], [i64; 3]) = ([-1300, 2045, -1760], [1300, 3995, -1760]);

/// A long: thousands and the rest.
type L = [i32; 2];

const MACHINES: &[&str] = &["sw_qi", "sw_qp", "sw_lsum", "sw_abs", "sw_divd", "sw_join", "sw_floor", "sw_add",
    "sw_esub", "sw_erows", "sw_eclip", "sw_eend", "sw_ewalk", "sw_ecross", "sw_epix",
    "sw_mag", "sw_qis", "sw_term", "sw_shift", "sw_floor3", "sw_sign", "sw_hund", "sw_isqrt", "sw_isqrt2", "sw_ang", "sw_sin",
    "sw_order", "sw_atan1", "sw_atan2", "sw_quad", "sw_short", "sw_tall", "sw_wide2",
    "tn_edge", "tn_twist", "tn_cga", "tn_step", "tn_walk", "tn_macturn",
    // the cold open's (segments/mac), for its tunnel running on
    "mac_offset", "mac_slope", "mac_angle", "mac_reach", "mac_twist", "mac_trow", "mac_dither", "mac_dissolve"];

/// The CGA tunnel's tables (turn/data/cga.txt).
struct Cga { bayer: Vec<i64>, hi: Vec<i64>, lo: Vec<i64>, pals: Vec<Vec<u8>>, flicks: Vec<usize> }

fn read_cga(demo: &Path) -> Cga {
    let text = read(demo, "turn/data/cga.txt");
    let nums = |l: &str| -> Vec<i64> { l.split('#').next().unwrap().split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
    let get = |name: &str| nums(text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap());
    Cga { bayer: get("bayer"), hi: get("hi"), lo: get("lo"),
        pals: text.lines().filter(|l| l.starts_with("pal ")).map(|l| nums(l).iter().map(|&v| v as u8).collect()).collect(),
        flicks: get("flicks").iter().map(|&v| v as usize).collect() }
}

/// A picture the screen shows, its edges repeated PAD round.
struct Picture { w: usize, texels: Vec<[u8; 3]> }
impl Picture {
    fn padded(rgb: &[u8], w: usize, h: usize) -> Picture {
        let (pw, ph) = (w + 2 * PAD as usize, h + 2 * PAD as usize);
        let mut texels = Vec::with_capacity(pw * ph);
        for v in 0..ph { for u in 0..pw {
            let (x, y) = (u.saturating_sub(PAD as usize).min(w - 1), v.saturating_sub(PAD as usize).min(h - 1));
            let k = (y * w + x) * 3;
            texels.push([rgb[k], rgb[k + 1], rgb[k + 2]]);
        } }
        Picture { w: pw, texels }
    }
    fn at(&self, u: i32, v: i32) -> [u8; 3] { self.texels[v as usize * self.w + u as usize] }
}

pub struct Turn { m: BTreeMap<&'static str, Machine>, tables: Tables, cga: Cga }

/// A frame's scene after the sums: its pieces' polygons on the screen (longs), each one's label (its face's way) and
/// colour, which is the screen's, and the screen's grid.
struct Scene { polys: Vec<Vec<(L, L)>>, labels: Vec<i32>, colours: Vec<[u8; 3]>, screen: Vec<bool>, front: Vec<Vec<bool>>, grid: Option<Grid> }
/// The screen's grid: its first column and row, and the places on the picture at its points (longs).
struct Grid { i0: i32, j0: i32, u: Vec<Vec<L>>, v: Vec<Vec<L>> }

impl Turn {
    pub fn new(demo: &Path) -> Turn {
        let mut m = BTreeMap::new();
        for &n in MACHINES { m.insert(n, rust(demo, n)); }
        let tables = super::swarm_calc::load_tables(&read(demo, "swarm/data/tables.txt"));
        Turn { m, tables, cga: read_cga(demo) }
    }

    /// Run a machine over items: its deck is the count, the header, then the items, at most 999 a run.
    fn go(&self, host: &mut Host, name: &str, header: &[i32], items: &[Vec<i32>]) -> Vec<i32> {
        let mut out = vec![];
        for chunk in items.chunks(999) {
            let mut deck = vec![chunk.len() as i32];
            deck.extend(header);
            for it in chunk { deck.extend(it); }
            if let Some(i) = deck.iter().position(|v| v.abs() > 999) { panic!("{name}: card {i} of {} is {}", deck.len(), deck[i]); }
            out.extend(host.gon(&self.m[name], &[&deck]));
        }
        out
    }

    fn eval(&self, host: &mut Host, c: &mut Calc) {
        let mut go = |m: &'static str, items: Items, out: &mut Vec<i32>| {
            for (k, cards) in items.chunks(999) {
                let count = [k as i32];
                if let Some(i) = cards.iter().position(|v| v.abs() > 999) { panic!("{m}: card {} of {} is {}", i + 1, cards.len() + 1, cards[i]); }
                host.gon_into(&self.m[m], &[&count, cards], out);
            }
        };
        c.eval(&mut go, &self.tables);
    }

    // --- the swarm's arithmetic for the walks --------------------------------------------------------------------

    fn sums(&self, host: &mut Host, sums: &[Vec<(i32, i32, i32)>]) -> Vec<[i32; 3]> {
        let terms: Vec<(i32, i32, i32)> = sums.iter().flatten().copied().collect();
        let idx = self.go(host, "sw_qi", &[], &terms.iter().map(|&(a, b, _)| vec![a, b]).collect::<Vec<_>>());
        let items: Vec<Vec<i32>> = terms.iter().enumerate().map(|(k, &(_, _, place))| {
            let mut v = vec![place];
            v.extend(self.tables.quarter[idx[4 * k] as usize][idx[4 * k + 1] as usize]);
            v.extend(self.tables.quarter[idx[4 * k + 2] as usize][idx[4 * k + 3] as usize]);
            v
        }).collect();
        let products = self.go(host, "sw_qp", &[], &items);
        let mut k = 0;
        let items: Vec<Vec<i32>> = sums.iter().map(|s| {
            let mut v = vec![s.len() as i32];
            for _ in s { v.extend(&products[3 * k..3 * k + 3]); k += 1; }
            v
        }).collect();
        self.go(host, "sw_lsum", &[], &items).chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
    }

    fn divide(&self, host: &mut Host, items: &[([i32; 3], usize, L)]) -> Vec<(L, L)> {
        let sizes = self.go(host, "sw_abs", &[], &items.iter().map(|(n, _, _)| n.to_vec()).collect::<Vec<_>>());
        let deck: Vec<Vec<i32>> = items.iter().enumerate().map(|(k, (_, extra, d))| {
            let mut v = d.to_vec();
            v.push(3 + *extra as i32);
            v.extend(&sizes[4 * k + 1..4 * k + 4]);
            v.extend(std::iter::repeat_n(0, *extra));
            v
        }).collect();
        let digits = self.go(host, "sw_divd", &[], &deck);
        let (mut jdeck, mut rems, mut at) = (vec![], vec![], 0);
        for (_, extra, _) in items {
            let n = 3 * (3 + extra);
            let mut v = vec![n as i32];
            v.extend(&digits[at..at + n]);
            jdeck.push(v);
            rems.push([digits[at + n], digits[at + n + 1]]);
            at += n + 2;
        }
        let q = self.go(host, "sw_join", &[], &jdeck);
        let deck: Vec<Vec<i32>> = items.iter().enumerate().map(|(k, (_, _, d))| {
            let mut v = vec![sizes[4 * k]];
            v.extend(&q[2 * k..2 * k + 2]);
            v.extend(rems[k]);
            v.extend(d);
            v
        }).collect();
        self.go(host, "sw_floor", &[], &deck).chunks(4).map(|c| ([c[0], c[1]], [c[2], c[3]])).collect()
    }
}

// --- the scene's sums ---------------------------------------------------------------------------------------------

/// A number as the cards say (one, a long, or three).
fn value(c: &Calc, n: N) -> i64 {
    let k = c.cards(n);
    k.iter().fold(0i64, |a, &v| a * 1000 + v as i64)
}
fn flag(c: &Calc, a: Arg) -> bool {
    match a { Arg::Some(n, f, 1) => { let v = c.cards(n)[f as usize]; assert!(v == 0 || v == 1); v == 1 } _ => panic!("not a flag") }
}

/// The smoothstep of x (thousandths, held to 0..1000), in thousandths: fl(fl(x^2, 10) (3000 - 2 x), 100000).
fn smooth(c: &mut Calc, x: N) -> N {
    let x = c.clamp(x, 0, 1000);
    let xx = c.mul(x, x);
    let q = c.divk(xx, 10);
    let q = c.within(q, 100_000);
    let k = c.num(3000);
    let two = c.num(2);
    let w = c.lin(&[(k, None, false), (x, Some(two), true)]);
    let w = c.within(w, 3000);
    let p = c.mul(q, w);
    let p = c.shift(p);
    let s = c.divk(p, 100);
    c.within(s, 1000)
}
/// a + fl((b - a) u, 1000) for numbers a, b of the data; a itself when they're the same.
fn lerp(c: &mut Calc, a: i64, b: i64, u: N) -> N {
    if a == b { return c.num(a); }
    let (a1000, d) = (c.num(1000 * a), c.num(b - a));
    let s = c.lin(&[(a1000, None, false), (d, Some(u), false)]);
    let s = c.shift(s);
    c.within(s, a.abs().max(b.abs()))
}
/// Ten thousand long the way of v (about a thousand long): 20000 a v / (a^2 + |v|^2), a the whole root, the bottom in
/// hundreds.
fn unit(c: &mut Calc, v: [N; 3]) -> [N; 3] {
    let s = c.lin(&[(v[0], Some(v[0]), false), (v[1], Some(v[1]), false), (v[2], Some(v[2]), false)]);
    let s = c.within(s, 1_100_000);
    let a = c.sqrt(s);
    let a = c.within(a, 1049);
    let den = c.lin(&[(a, Some(a), false), (s, None, false)]);
    let den = c.divk(den, 100);
    let den = c.within(den, 22_000);
    v.map(|x| {
        let x200 = c.mulk(x, 200);
        let p = c.mul(x200, a);
        let q = c.div(p, den);
        c.within(q, 10_000)
    })
}

/// The camera at frame f: the eye in the boxes' frame and the basis (ten-thousandths), the morph, the spin's
/// cosine and sine (thousandths), after the first sums (spin then known).
struct CamA { spin: Option<(N, Arg, Arg)>, m: N, eye: [N; 3], basis: [[N; 3]; 3] }
struct Cam { m: N, eye: [N; 3], r: [N; 3], u: [N; 3], f: [N; 3], cs: [N; 2] }

fn camera_a(c: &mut Calc, f: usize) -> CamA {
    let f = f as i64;
    let (mut spin, mut m) = (None, c.num(0));
    let (tgt, dist, dirn): ([N; 3], N, [N; 3]);
    if f < N_OUT as i64 {
        // (the frame's thousandths of the leg, on the calculator)
        let fr = c.num(f);
        let x = c.mulk(fr, 1000);
        let x = c.divk(x, N_OUT as i64 - 1);
        let u = smooth(c, x);
        tgt = std::array::from_fn(|i| lerp(c, MAC_AT[i], CENTRE[i], u));
        dist = lerp(c, D0, FAR, u);
        dirn = std::array::from_fn(|i| lerp(c, FRONT[i], SIDE[i], u));
    } else if f < (N_OUT + N_SPIN) as i64 {
        let fr = c.num(f);
        let k = c.addk(fr, -(N_OUT as i64));
        let x = c.mulk(k, 1000);
        let x = c.divk(x, N_SPIN as i64 - 1);
        let s = smooth(c, x);
        let z = c.is0(s);
        let less = c.addk(s, -1000);
        let whole = c.is0(less);
        spin = Some((s, z, whole));
        let x = c.mulk(k, 10);
        let x = c.addk(x, -58);
        let x = c.mulk(x, 1000);
        let x = c.divk(x, 174);
        m = smooth(c, x);
        tgt = CENTRE.map(|v| c.num(v));
        dist = c.num(FAR);
        dirn = SIDE.map(|v| c.num(v));
    } else {
        m = c.num(1000);
        let g = f - (N_OUT + N_SPIN) as i64;
        let u = if g >= N_HOLD as i64 {
            let fr = c.num(f);
            let x = c.addk(fr, -((N_OUT + N_SPIN + N_HOLD) as i64));
            let x = c.mulk(x, 1000);
            let x = c.divk(x, N_IN as i64 - 1);
            smooth(c, x)
        } else { c.num(0) };
        tgt = std::array::from_fn(|i| lerp(c, CENTRE[i], PC_AT[i], u));
        dist = lerp(c, FAR, D1, u);
        dirn = std::array::from_fn(|i| lerp(c, SIDE[i], FRONT[i], u));
    }
    let n = unit(c, dirn);
    let eye: [N; 3] = std::array::from_fn(|i| {
        let p = c.mul(n[i], dist);
        let q = c.divk(p, 10_000);
        let e = c.add(tgt[i], q);
        c.within(e, 20_000)
    });
    let fw = n.map(|v| { let x = c.neg(v); c.within(x, 10_000) });
    // across: level, at right angles to forward: the way of (-dz, 0, dx), made a unit
    let mdz = c.neg(dirn[2]);
    let z = c.num(0);
    let r = unit(c, [mdz, z, dirn[0]]);
    let k = |c: &mut Calc, t: &[(N, Option<N>, bool)]| { let s = c.lin(t); let q = c.divk(s, 10_000); c.within(q, 10_000) };
    let up = [k(c, &[(fw[1], Some(r[2]), false)]), k(c, &[(fw[2], Some(r[0]), false), (fw[0], Some(r[2]), true)]), k(c, &[(fw[1], Some(r[0]), true)])];
    CamA { spin, m, eye, basis: [r, up, fw] }
}

/// The camera in the boxes' frame: turned back by the spin (none when it hasn't turned, or has turned once round, as
/// the machines' flags say).
fn camera_b(c: &mut Calc, a: &CamA) -> Cam {
    let turned = a.spin.filter(|&(_, z, w)| !flag(c, z) && !flag(c, w));
    let (cos, sin) = match turned {
        None => (c.num(1000), c.num(0)),
        Some((s, _, _)) => { let a = c.mulk(s, 1000); let (sn, cs) = c.sin(a); (cs, sn) }
    };
    let back = |c: &mut Calc, v: [N; 3], bound: i64| -> [N; 3] {
        if turned.is_none() { return v; }
        let x = c.lin(&[(cos, Some(v[0]), false), (sin, Some(v[2]), true)]);
        let x = c.shift(x);
        let z = c.lin(&[(sin, Some(v[0]), false), (cos, Some(v[2]), false)]);
        let z = c.shift(z);
        [c.within(x, bound), v[1], c.within(z, bound)]
    };
    let eye = back(c, a.eye, 20_000);
    let [r, u, f] = a.basis.map(|v| back(c, v, 10_000));
    Cam { m: a.m, eye, r, u, f, cs: [cos, sin] }
}

/// The scene's boxes at the morph: lo, hi and colour as numbers; the screen last.
fn boxes(c: &mut Calc, m: N) -> Vec<([N; 3], [N; 3], [N; 3])> {
    let mut out = vec![];
    for k in 0..14 {
        let (blo, bhi, bc) = PC[k];
        let (alo, ahi, ac) = if k < 13 { MAC[k] } else { let mid: [i64; 3] = std::array::from_fn(|i| (blo[i] + bhi[i]).div_euclid(2)); (mid, mid, bc) };
        out.push((std::array::from_fn(|i| lerp(c, alo[i], blo[i], m)), std::array::from_fn(|i| lerp(c, ahi[i], bhi[i], m)), std::array::from_fn(|i| lerp(c, ac[i], bc[i], m))));
    }
    let z = c.num(0);
    out.push((std::array::from_fn(|i| lerp(c, MAC_SCREEN.0[i], PC_SCREEN.0[i], m)), std::array::from_fn(|i| lerp(c, MAC_SCREEN.1[i], PC_SCREEN.1[i], m)), [z, z, z]));
    out
}

/// The order of the numbers on one axis, as the machines compared them: each number's rank (equal numbers the same).
struct Order { rank: BTreeMap<N, usize>, rep: Vec<N> }
impl Order {
    fn lt(&self, a: N, b: N) -> bool { self.rank[&a] < self.rank[&b] }
    fn le(&self, a: N, b: N) -> bool { self.rank[&a] <= self.rank[&b] }
    fn max(&self, a: N, b: N) -> N { if self.lt(a, b) { b } else { a } }
    fn min(&self, a: N, b: N) -> N { if self.lt(b, a) { b } else { a } }
    /// One number for all those equal to it.
    fn canon(&self, a: N) -> N { self.rep[self.rank[&a]] }
}

/// Each axis's numbers compared pairwise: (a, b, the sign of a - b: below 0 and 0).
fn comparisons(c: &mut Calc, values: &[Vec<N>; 3]) -> [Vec<(N, N, N)>; 3] {
    std::array::from_fn(|k| {
        let mut out = vec![];
        let vs: Vec<N> = values[k].iter().copied().collect::<BTreeSet<N>>().into_iter().collect();
        for i in 0..vs.len() { for j in i + 1..vs.len() {
            let d = c.sub(vs[i], vs[j]);
            out.push((vs[i], vs[j], c.sign(d)));
        } }
        out
    })
}

fn order(c: &Calc, values: &[N], cmp: &[(N, N, N)]) -> Order {
    let mut less: BTreeMap<(N, N), std::cmp::Ordering> = BTreeMap::new();
    for &(a, b, s) in cmp {
        let k = c.cards(s);
        let o = if k[1] == 1 { std::cmp::Ordering::Equal } else if k[0] == 1 { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        less.insert((a, b), o);
        less.insert((b, a), o.reverse());
    }
    let mut vs: Vec<N> = values.iter().copied().collect::<BTreeSet<N>>().into_iter().collect();
    vs.sort_by(|&a, &b| if a == b { std::cmp::Ordering::Equal } else { less[&(a, b)] });
    let (mut rank, mut rep) = (BTreeMap::new(), vec![]);
    for (i, &v) in vs.iter().enumerate() {
        if i == 0 || less[&(vs[i - 1], v)] != std::cmp::Ordering::Equal { rep.push(v); }
        rank.insert(v, rep.len() - 1);
    }
    Order { rank, rep }
}

/// A piece of a face (see the reference): lo and hi on each axis (numbers), its axis and side, its box.
#[derive(Clone)]
struct Piece { lo: [N; 3], hi: [N; 3], axis: usize, side: usize, owner: usize }

fn others(axis: usize) -> (usize, usize) { ((axis + 1) % 3, (axis + 2) % 3) }

/// The pieces of the faces that face the eye, as the reference cuts them, from the order alone.
fn pieces(bx: &[([N; 3], [N; 3], [N; 3])], eye: [N; 3], o: &[Order; 3]) -> Vec<Piece> {
    let solid = |b: &([N; 3], [N; 3], [N; 3])| (0..3).all(|k| o[k].lt(b.0[k], b.1[k]));
    let mut out = vec![];
    for (b, bb) in bx.iter().enumerate() {
        if b != SCREEN && !solid(bb) { continue; }
        for axis in 0..3 {
            let oa = &o[axis];
            for side in 0..2 {
                if b == SCREEN && (axis != 2 || side != 0) { continue; }
                let cc = if side == 0 { bb.0[axis] } else { bb.1[axis] };
                if !(if side == 0 { oa.lt(eye[axis], cc) } else { oa.lt(cc, eye[axis]) }) { continue; }
                let (i, j) = others(axis);
                let (oi, oj) = (&o[i], &o[j]);
                let mut rects = vec![[bb.0[i], bb.1[i], bb.0[j], bb.1[j]]];
                for (ob_i, ob) in bx.iter().enumerate().take(SCREEN) {
                    if ob_i == b || !solid(ob) || !(oa.lt(ob.0[axis], cc) && oa.lt(cc, ob.1[axis])) { continue; }
                    let h = [ob.0[i], ob.1[i], ob.0[j], ob.1[j]];
                    rects = rects.into_iter().flat_map(|r| {
                        let [a1, a2, b1, b2] = r;
                        let [j1, j2, l1, l2] = h;
                        if oi.le(j2, a1) || oi.le(a2, j1) || oj.le(l2, b1) || oj.le(b2, l1) { return vec![r]; }
                        let (h1, h2, g1, g2) = (oi.max(a1, j1), oi.min(a2, j2), oj.max(b1, l1), oj.min(b2, l2));
                        let mut v = vec![];
                        if oj.lt(b1, g1) { v.push([a1, a2, b1, g1]); }
                        if oj.lt(g2, b2) { v.push([a1, a2, g2, b2]); }
                        if oi.lt(a1, h1) { v.push([a1, h1, g1, g2]); }
                        if oi.lt(h2, a2) { v.push([h2, a2, g1, g2]); }
                        v
                    }).collect();
                }
                for [a1, a2, b1, b2] in rects {
                    let (mut lo, mut hi) = ([cc; 3], [cc; 3]);
                    lo[i] = a1; hi[i] = a2; lo[j] = b1; hi[j] = b2;
                    let canon = |v: [N; 3]| -> [N; 3] { std::array::from_fn(|k| o[k].canon(v[k])) };
                    out.push(Piece { lo: canon(lo), hi: canon(hi), axis, side, owner: b });
                }
            }
        }
    }
    out
}

/// Each piece's corners round it, with every other piece's corner on one of its edges put in (by the order).
fn outlines(ps: &[Piece], o: &[Order; 3]) -> Vec<Vec<[N; 3]>> {
    let corners = |p: &Piece| -> [[N; 3]; 4] {
        let (i, j) = others(p.axis);
        let at = |a: N, b: N| { let mut v = p.lo; v[i] = a; v[j] = b; v };
        [at(p.lo[i], p.lo[j]), at(p.hi[i], p.lo[j]), at(p.hi[i], p.hi[j]), at(p.lo[i], p.hi[j])]
    };
    let all: BTreeSet<[N; 3]> = ps.iter().flat_map(|p| corners(p)).collect();
    ps.iter().map(|p| {
        let c = corners(p);
        let mut pts = vec![];
        for e in 0..4 {
            let (a, b) = (c[e], c[(e + 1) % 4]);
            pts.push(a);
            let t = (0..3).find(|&k| a[k] != b[k]).unwrap();
            let (lo, hi) = if o[t].lt(a[t], b[t]) { (a[t], b[t]) } else { (b[t], a[t]) };
            let mut on: Vec<[N; 3]> = all.iter().filter(|v| (0..3).all(|k| k == t || v[k] == a[k]) && o[t].lt(lo, v[t]) && o[t].lt(v[t], hi)).copied().collect();
            let up = o[t].lt(a[t], b[t]);
            on.sort_by(|x, y| { let r = o[t].rank[&x[t]].cmp(&o[t].rank[&y[t]]); if up { r } else { r.reverse() } });
            pts.extend(on);
        }
        pts
    }).collect()
}

/// Whether p is in front of q where both are seen (see the reference), by the order.
fn front(p: &Piece, q: &Piece, eye: [N; 3], o: &[Order; 3]) -> Option<bool> {
    for k in 0..3 {
        if o[k].le(p.hi[k], q.lo[k]) { return Some(o[k].le(eye[k], p.hi[k])); }
        if o[k].le(q.hi[k], p.lo[k]) { return Some(o[k].le(p.lo[k], eye[k])); }
    }
    None
}

/// A point on the screen: hundredths of a pixel from the middle, across and down, as longs.
struct Projector { cam_products: BTreeMap<(usize, N), [N; 3]> }
impl Projector {
    fn new() -> Self { Projector { cam_products: BTreeMap::new() } }
    /// (p_k - eye_k) times the basis's k parts.
    fn part(&mut self, c: &mut Calc, cam: &Cam, k: usize, v: N) -> [N; 3] {
        *self.cam_products.entry((k, v)).or_insert_with(|| {
            let d = c.sub(v, cam.eye[k]);
            let d = c.within(d, 25_000);
            [cam.r[k], cam.u[k], cam.f[k]].map(|b| c.mul(d, b))
        })
    }
    fn project(&mut self, c: &mut Calc, cam: &Cam, p: [N; 3]) -> (N, N) {
        let parts: Vec<[N; 3]> = (0..3).map(|k| self.part(c, cam, k, p[k])).collect();
        let sum = |c: &mut Calc, j: usize| c.lin(&[(parts[0][j], None, false), (parts[1][j], None, false), (parts[2][j], None, false)]);
        let x = sum(c, 0);
        let x = c.shift(x);
        let x = c.within(x, 47_000);
        let y = sum(c, 1);
        let y = c.shift(y);
        let y = c.within(y, 47_000);
        let z = sum(c, 2);
        let z = c.divk(z, 10_000);
        let z = c.within(z, 30_000);
        let fx = c.num(FX10);
        let sx = c.mul(x, fx);
        let sx = c.div(sx, z);
        let sx = c.within(sx, 400_000);
        let sy = c.lin(&[(y, Some(fx), true)]);
        let sy = c.div(sy, z);
        let sy = c.within(sy, 400_000);
        (c.long(sx), c.long(sy))
    }
}

/// The light on a face's way, ten-thousandths: 6200 + 3800 (n . light) + 800 (toward us), each held to 0 and up.
fn light(c: &mut Calc, cam: &Cam, axis: usize, side: usize) -> N {
    let [cs, sn] = cam.cs;
    let z = c.num(0);
    let k1000 = c.num(1000);
    let n: [N; 3] = match axis { 0 => [cs, z, c.neg(sn)], 1 => [z, k1000, z], _ => [sn, z, cs] };
    let n = if side == 0 { n.map(|v| c.neg(v)) } else { n };
    let l: Vec<N> = LIGHT.iter().map(|&v| c.num(v)).collect();
    let dot = c.lin(&[(n[0], Some(l[0]), false), (n[1], Some(l[1]), false), (n[2], Some(l[2]), false)]);
    let dot = c.within(dot, 1_000_000);
    // 3800 (n . light) / 1000000 is 38 (n . light) / 10000
    let lit = c.max(dot, z);
    let lit = c.mulk(lit, 38);
    let lit = c.divk(lit, 10_000);
    let nz = c.neg(n[2]);
    let toward = c.max(nz, z);
    let toward = c.mulk(toward, 800);
    let toward = c.shift(toward);
    let base = c.num(6200);
    let k = c.lin(&[(base, None, false), (lit, None, false), (toward, None, false)]);
    c.within(k, 11_000)
}

/// One way of the screen's grid point (see the reference's grid_place): the ray's parts across and along, the eye's
/// place less the screen's lo, the plane's depth, the screen's size that way, the picture's.
struct Place { across: N, along: N, from_lo: N, depth: N, size: N, tex: i64, down: bool }
/// How a grid point is placed, as the machines' flags say: held to an edge (past it the high one), or by the ratio
/// in hundred-thousandths, or in ten-thousandths.
#[derive(Clone, Copy)]
enum Way { Held(bool), Fine, Coarse }

impl Turn {
    /// The 3D frames' scenes, a group at a time: the sums, then the pieces, their polygons and colours, the screen's grid.
    fn scenes(&self, host: &mut Host, frames: &[usize]) -> Vec<Scene> {
        let mut c = Calc::new();
        let cams_a: Vec<CamA> = frames.iter().map(|&f| camera_a(&mut c, f)).collect();
        self.eval(host, &mut c);
        let cams: Vec<Cam> = cams_a.iter().map(|a| camera_b(&mut c, a)).collect();
        // the boxes, the comparisons on each axis of their numbers and the eye's
        let mut all = vec![];
        for cam in &cams {
            let bx = boxes(&mut c, cam.m);
            let values: [Vec<N>; 3] = std::array::from_fn(|k| { let mut v: Vec<N> = bx.iter().flat_map(|b| [b.0[k], b.1[k]]).collect(); v.push(cam.eye[k]); v });
            let cmp = comparisons(&mut c, &values);
            all.push((bx, values, cmp));
        }
        self.eval(host, &mut c);
        // the pieces, by the order; their corners on the screen; the light on them; the screen's corners
        struct Stage { ps: Vec<Piece>, polys: Vec<Vec<(N, N)>>, colours: Vec<[N; 3]>, scr: Option<([(N, N); 4], Place, Place)>, o: [Order; 3] }
        let mut stages = vec![];
        for (cam, (bx, values, cmp)) in cams.iter().zip(&all) {
            let o: [Order; 3] = std::array::from_fn(|k| order(&c, &values[k], &cmp[k]));
            let eye: [N; 3] = std::array::from_fn(|k| o[k].canon(cam.eye[k]));
            let ps = pieces(bx, eye, &o);
            let shapes = outlines(&ps, &o);
            let mut pr = Projector::new();
            let mut seen_pts: BTreeMap<[N; 3], (N, N)> = BTreeMap::new();
            let polys: Vec<Vec<(N, N)>> = shapes.iter().map(|s| s.iter().map(|&p| *seen_pts.entry(p).or_insert_with(|| pr.project(&mut c, cam, p))).collect()).collect();
            let mut lights: BTreeMap<(usize, usize), N> = BTreeMap::new();
            let mut colours = vec![];
            for p in &ps {
                let k = *lights.entry((p.axis, p.side)).or_insert_with(|| light(&mut c, cam, p.axis, p.side));
                colours.push(bx[p.owner].2.map(|ch| {
                    let v = c.mul(ch, k);
                    let v = c.divk(v, 10_000);
                    let top = c.num(255);
                    c.min(v, top)
                }));
            }
            let scr = if ps.iter().any(|p| p.owner == SCREEN) {
                let s = &bx[SCREEN];
                let corners = [(s.0[0], s.0[1]), (s.0[0], s.1[1]), (s.1[0], s.0[1]), (s.1[0], s.1[1])].map(|(x, y)| {
                    let (px, py) = pr.project(&mut c, cam, [x, y, s.0[2]]);
                    (px, py)
                });
                let depth = c.sub(s.0[2], cam.eye[2]);
                let depth = c.within(depth, 20_000);
                let wq = c.sub(s.1[0], s.0[0]);
                let wq = c.within(wq, 2600);
                let hq = c.sub(s.1[1], s.0[1]);
                let hq = c.within(hq, 1950);
                let fx = c.sub(cam.eye[0], s.0[0]);
                let fx = c.within(fx, 25_000);
                let fy = c.sub(cam.eye[1], s.0[1]);
                let fy = c.within(fy, 25_000);
                let z = c.num(0);
                Some((corners, Place { across: z, along: z, from_lo: fx, depth, size: wq, tex: 0, down: false }, Place { across: z, along: z, from_lo: fy, depth, size: hq, tex: 0, down: true }))
            } else { None };
            stages.push(Stage { ps, polys, colours, scr, o });
        }
        self.eval(host, &mut c);
        // the screen's box on the screen, its grid's columns and rows
        let mut boxes_ = vec![];
        for st in &stages {
            boxes_.push(st.scr.as_ref().map(|(corners, _, _)| {
                let pixel = |c: &mut Calc, l: N| { let v = c.run("sw_wide2", vec![Arg::All(l)], 3, 3, 400_000); c.divk(v, 100) };
                let px: Vec<N> = corners.iter().map(|&(x, _)| pixel(&mut c, x)).collect();
                let py: Vec<N> = corners.iter().map(|&(_, y)| pixel(&mut c, y)).collect();
                let most = |c: &mut Calc, v: &[N], hi: bool| v[1..].iter().fold(v[0], |a, &b| if hi { c.max(a, b) } else { c.min(a, b) });
                let (l, r, t, b) = (most(&mut c, &px, false), most(&mut c, &px, true), most(&mut c, &py, false), most(&mut c, &py, true));
                let cell = |c: &mut Calc, v: N, lo: i64, hi: i64, plus: i64| { let v = c.clamp(v, lo, hi); let v = c.addk(v, -lo); let v = c.divk(v, 20); let v = c.addk(v, plus); c.within(v, 77) };
                [cell(&mut c, l, -768, 767, 0), cell(&mut c, r, -768, 767, 1), cell(&mut c, t, -384, 383, 0), cell(&mut c, b, -384, 383, 1)]
            }));
        }
        self.eval(host, &mut c);
        // the grid's points: the ray's parts, and whether each is held
        let mut grids = vec![];
        for (fi, st) in stages.iter().enumerate() {
            let cam = &cams[fi];
            grids.push(boxes_[fi].map(|bxn| {
                let [i0, i1, j0, j1] = bxn.map(|n| value(&c, n));
                let mut pts = vec![];
                for j in j0..=j1 {
                    let mut row = vec![];
                    for i in i0..=i1 {
                        let (x, y) = (c.num(200 * i - 7675), c.num(3835 - 200 * j));
                        let fxn = c.num(FX10);
                        // the ray's way: across and up in thousands, along in ten-thousands
                        let d: [N; 3] = std::array::from_fn(|k| {
                            let s = c.lin(&[(x, Some(cam.r[k]), false), (y, Some(cam.u[k]), false), (fxn, Some(cam.f[k]), false)]);
                            let s = c.divk(s, if k == 2 { 10_000 } else { 1000 });
                            c.within(s, if k == 2 { 40_000 } else { 400_000 })
                        });
                        // held: the ray hardly along the screen's way, or meeting it too far off; the ratio past 9.9
                        let k100 = c.num(100);
                        let flat = c.lt(d[2], k100);
                        let ways: [(Arg, Arg); 2] = [0, 1].map(|w| {
                            let a = c.abs(d[w]);
                            let far = c.mulk(d[2], 990);
                            let big = c.mulk(d[2], 99);
                            (c.lt(far, a), c.lt(big, a))
                        });
                        let neg = [c.below0(d[0]), c.below0(d[1])];
                        row.push((d, flat, ways, neg));
                    }
                    pts.push(row);
                }
                let _ = st;
                (i0, j0, pts)
            }));
        }
        self.eval(host, &mut c);
        // each grid point's places on the picture
        let mut placed = vec![];
        for (fi, st) in stages.iter().enumerate() {
            let tex = if fi < frames.len() && frames[fi] < MAC_FRAMES { (512, 342) } else { (320, 200) };
            placed.push(grids[fi].as_ref().map(|(i0, j0, pts)| {
                let (_, pu, pv) = st.scr.as_ref().unwrap();
                let mut gu = vec![];
                let mut gv = vec![];
                for row in pts {
                    let (mut ru, mut rv) = (vec![], vec![]);
                    for (d, flat, ways, neg) in row {
                        for (w, p) in [pu, pv].into_iter().enumerate() {
                            let p = Place { across: d[w], along: d[2], tex: if w == 0 { tex.0 } else { tex.1 }, ..*p };
                            let how = if flag(&c, *flat) || flag(&c, ways[w].0) { Way::Held(!flag(&c, neg[w])) } else if flag(&c, ways[w].1) { Way::Coarse } else { Way::Fine };
                            let v = self.grid_place(&mut c, &p, how);
                            if w == 0 { ru.push(v) } else { rv.push(v) }
                        }
                    }
                    gu.push(ru);
                    gv.push(rv);
                }
                (*i0, *j0, gu, gv)
            }));
        }
        // the tall edges cut, as the swarm's
        let mut polys: Vec<Vec<Vec<(N, N)>>> = stages.iter().map(|s| s.polys.clone()).collect();
        loop {
            let mut checks = vec![];
            for (fi, ps) in polys.iter().enumerate() {
                for (pi, p) in ps.iter().enumerate() {
                    for e in 0..p.len() {
                        let (a, b) = (p[e], p[(e + 1) % p.len()]);
                        let t = c.run("sw_tall", vec![Arg::All(a.1), Arg::All(b.1)], 1, 0, 0);
                        checks.push((fi, pi, e, t));
                    }
                }
            }
            self.eval(host, &mut c);
            let tall: Vec<_> = checks.into_iter().filter(|ch| c.cards(ch.3)[0] == 1).collect();
            if tall.is_empty() { break; }
            for &(fi, pi, e, _) in tall.iter().rev() {
                let p = &mut polys[fi][pi];
                let (a, b) = (p[e], p[(e + 1) % p.len()]);
                let mid = |c: &mut Calc, p: N, q: N| -> N {
                    let (p, q) = (c.run("sw_wide2", vec![Arg::All(p)], 3, 3, 400_000), c.run("sw_wide2", vec![Arg::All(q)], 3, 3, 400_000));
                    let m = c.add(p, q);
                    let m = c.divk(m, 2);
                    c.long(m)
                };
                let (x, y) = (mid(&mut c, a.0, b.0), mid(&mut c, a.1, b.1));
                p.insert(e + 1, (x, y));
            }
            self.eval(host, &mut c);
        }
        self.eval(host, &mut c);
        // the scenes, as cards
        let l = |c: &Calc, n: N| -> L { let k = c.cards(n); assert_eq!(k.len(), 2); [k[0], k[1]] };
        stages.iter().enumerate().map(|(fi, st)| {
            let eye: [N; 3] = std::array::from_fn(|k| st.o[k].canon(cams[fi].eye[k]));
            let n = st.ps.len();
            let front_m: Vec<Vec<bool>> = (0..n).map(|a| (0..n).map(|b| a != b && front(&st.ps[a], &st.ps[b], eye, &st.o).unwrap_or(false)).collect()).collect();
            Scene {
                polys: polys[fi].iter().map(|p| p.iter().map(|&(x, y)| (l(&c, x), l(&c, y))).collect()).collect(),
                labels: st.ps.iter().map(|p| (2 * p.axis + p.side) as i32).collect(),
                colours: st.colours.iter().map(|cl| cl.map(|v| value(&c, v) as u8)).collect(),
                screen: st.ps.iter().map(|p| p.owner == SCREEN).collect(),
                front: front_m,
                grid: placed[fi].as_ref().map(|(i0, j0, gu, gv)| Grid { i0: *i0 as i32, j0: *j0 as i32,
                    u: gu.iter().map(|r| r.iter().map(|&v| l(&c, v)).collect()).collect(),
                    v: gv.iter().map(|r| r.iter().map(|&v| l(&c, v)).collect()).collect() }),
            }
        }).collect()
    }

    /// A grid point's place on the picture one way (see the reference's grid_place), as a long: held to an edge
    /// (the flag's way) or where the ray meets the plane.
    fn grid_place(&self, c: &mut Calc, p: &Place, how: Way) -> N {
        let size = p.size;
        let (lo, hi) = (c.mulk(size, -50), c.mulk(size, 150));
        let h = match how {
            Way::Held(true) => hi,
            Way::Held(false) => lo,
            Way::Fine | Way::Coarse => {
                // the ratio in hundred-thousandths (10000 dx / dz), or past 9.9 in ten-thousandths (1000 dx / dz), as
                // thousands and the rest; the hit from lo, hundredths
                let coarse = matches!(how, Way::Coarse);
                let x = if coarse { p.across } else { c.mulk(p.across, 10) };
                let ratio = c.divrem(x, p.along, 1).0;
                let ratio = c.within(ratio, 999_999);
                let rl = c.long(ratio);
                let (rhi, rlo) = (c.part(rl, 0, 1, 1, 999), c.part(rl, 1, 1, 1, 999));
                let small = c.mul(p.depth, rlo);
                let small = if coarse { c.divk(small, 100) } else { c.shift(small) };
                let k100 = c.num(100);
                let big = if coarse { c.mulk(rhi, 10) } else { rhi };
                let h = c.lin(&[(p.from_lo, Some(k100), false), (p.depth, Some(big), false), (small, None, false)]);
                c.within(h, 160_000_000)
            }
        };
        let h = c.max(h, lo);
        let h = c.min(h, hi);
        let h = c.within(h, 390_000);
        let h = if p.down { let k100 = c.num(100); let v = c.lin(&[(size, Some(k100), false), (h, None, true)]); c.within(v, 390_000) } else { h };
        let t = c.num(p.tex);
        let q = c.mul(h, t);
        let q = c.div(q, size);
        let q = c.within(q, 200_000);
        let base = c.num(1000 * PAD);
        let ten = c.num(10);
        let v = c.lin(&[(base, None, false), (q, Some(ten), false)]);
        let v = c.clamp(v, 0, 1000 * (p.tex + 2 * PAD) - 1);
        c.long(v)
    }
}

// --- the pixels ---------------------------------------------------------------------------------------------------

impl Turn {
    /// Each polygon's span on each row it covers (the swarm's edge walks): per polygon, row (-384..383) -> [l, r).
    fn spans(&self, host: &mut Host, polys: &[Vec<(L, L)>]) -> Vec<BTreeMap<i32, (i32, i32)>> {
        struct Edge { top: (L, L), bottom: (L, L) }
        let mut edges = vec![];
        let mut uses: Vec<(usize, usize)> = vec![];
        let mut seen: BTreeMap<((L, L), (L, L)), usize> = BTreeMap::new();
        for (p, poly) in polys.iter().enumerate() {
            let n = poly.len();
            for i in 0..n {
                let (a, b) = (poly[i], poly[(i + 1) % n]);
                let (top, bottom) = if b.1 < a.1 { (b, a) } else { (a, b) };
                let k = *seen.entry((top, bottom)).or_insert_with(|| { edges.push(Edge { top, bottom }); edges.len() - 1 });
                uses.push((p, k));
            }
        }
        let mut out: Vec<BTreeMap<i32, (i32, i32)>> = vec![BTreeMap::new(); polys.len()];
        if edges.is_empty() { return out; }
        let sub = self.go(host, "sw_esub", &[], &edges.iter().map(|e| [e.top.0, e.top.1, e.bottom.0, e.bottom.1].concat()).collect::<Vec<_>>());
        let rows = self.go(host, "sw_erows", &[], &(0..edges.len()).map(|k| sub[5 * k + 2..5 * k + 5].to_vec()).collect::<Vec<_>>());
        let live: Vec<usize> = (0..edges.len()).filter(|&k| rows[k] != 0).collect();
        let run = |k: usize| -> L { [sub[5 * k], sub[5 * k + 1]] };
        let height = |k: usize| -> L { [sub[5 * k + 2], sub[5 * k + 3]] };
        let first = |k: usize| sub[5 * k + 4];
        let mut prods = vec![];
        for &k in &live {
            let r = run(k);
            prods.push(vec![(first(k), r[0], 1), (first(k), r[1], 0)]);
            prods.push(vec![(100, r[0], 1), (100, r[1], 0)]);
        }
        let p = self.sums(host, &prods);
        // the crossing at the first row, and the step a row on: none for an edge that meets one row (a flat one's
        // would pass a long, and it takes no step)
        let mut div = vec![];
        for (n, &k) in live.iter().enumerate() {
            div.push((p[2 * n], 0, height(k)));
            if rows[k] > 1 { div.push((p[2 * n + 1], 0, height(k))); }
        }
        let quot = self.divide(host, &div);
        let mut q = vec![];
        let mut at = 0;
        for &k in &live {
            q.push(quot[at]);
            at += 1;
            if rows[k] > 1 { q.push(quot[at]); at += 1; } else { q.push(([0, 0], [0, 0])); }
        }
        let x0 = self.go(host, "sw_add", &[2], &live.iter().enumerate().map(|(n, &k)| [edges[k].top.0, q[2 * n].0].concat()).collect::<Vec<_>>());
        let clip = self.go(host, "sw_eclip", &[], &live.iter().map(|&k| vec![edges[k].top.1[0], edges[k].top.1[1], rows[k]]).collect::<Vec<_>>());
        let end = self.go(host, "sw_eend", &[], &live.iter().enumerate().map(|(n, &k)| vec![rows[k], clip[2 * n], clip[2 * n + 1]]).collect::<Vec<_>>());
        let carries = self.go(host, "sw_ewalk", &[], &live.iter().enumerate().map(|(n, &k)| {
            let mut v = height(k).to_vec(); v.extend(q[2 * n].1); v.extend(q[2 * n + 1].1); v.push(end[n]); v
        }).collect::<Vec<_>>());
        let mut items = vec![];
        let mut at = 0;
        for n in 0..live.len() {
            let mut v = x0[2 * n..2 * n + 2].to_vec();
            v.extend(q[2 * n + 1].0);
            v.extend([clip[2 * n], end[n], clip[2 * n + 1]]);
            v.extend(&carries[at..at + end[n] as usize]);
            at += end[n] as usize;
            items.push(v);
        }
        let cross = self.go(host, "sw_ecross", &[], &items);
        let pixels = self.go(host, "sw_epix", &[], &cross.chunks(3).map(|c| c[1..3].to_vec()).collect::<Vec<_>>());
        let mut by_edge: Vec<Vec<(i32, i32)>> = vec![vec![]; edges.len()];
        let mut at = 0;
        for (n, &k) in live.iter().enumerate() {
            for _ in 0..(end[n] - clip[2 * n]) { by_edge[k].push((cross[3 * at], pixels[at])); at += 1; }
        }
        let mut found: BTreeMap<(usize, i32), Vec<i32>> = BTreeMap::new();
        for &(p, k) in &uses { for &(row, px) in &by_edge[k] { found.entry((p, row)).or_default().push(px); } }
        for ((p, row), mut px) in found {
            px.sort();
            let (a, b) = (px[0], px[px.len() - 1]);
            if a != b { out[p].insert(row, (a, b)); }
        }
        out
    }

    /// The screen's runs walked across its grid: each pixel's texel (across, down). Down each column of the grid by
    /// straight lines to the rows the runs are on, then across each run a cell at a time (tn_step, tn_walk).
    fn texels(&self, host: &mut Host, g: &Grid, runs: &[(usize, i32, i32)]) -> Vec<Vec<(i32, i32)>> {
        if runs.is_empty() { return vec![]; }
        let (ni, nj) = (g.u[0].len(), g.u.len());
        let rows: BTreeSet<usize> = runs.iter().map(|r| r.0).collect();
        // the bands (20 rows between two of the grid's rows) the runs are on, and each one's rows on the frame
        let bands: Vec<usize> = (0..nj - 1).filter(|&jj| { let top = 20 * (g.j0 as usize + jj); rows.range(top..top + 20).next().is_some() }).collect();
        let on = |jj: usize| -> (usize, usize) { let top = 20 * (g.j0 as usize + jj); (top, 20.min(H - top)) };
        let pairs: Vec<Vec<i32>> = bands.iter().flat_map(|&jj| (0..ni).flat_map(move |ii| [[g.u[jj][ii], g.u[jj + 1][ii]].concat(), [g.v[jj][ii], g.v[jj + 1][ii]].concat()])).collect();
        let steps = self.go(host, "tn_step", &[], &pairs);
        let mut items = vec![];
        let mut k = 0;
        for &jj in &bands {
            for ii in 0..ni {
                for (w, gw) in [&g.u, &g.v].into_iter().enumerate() {
                    let mut v = vec![1];
                    v.extend(gw[jj][ii]);
                    v.extend(&steps[3 * (2 * k + w)..3 * (2 * k + w) + 3]);
                    v.extend([0, on(jj).1 as i32]);
                    items.push(v);
                }
                k += 1;
            }
        }
        let walked = self.go(host, "tn_walk", &[], &items);
        // each column's places on each row of the bands
        let mut column: BTreeMap<(usize, usize), (L, L)> = BTreeMap::new();
        let mut at = 0;
        for &jj in &bands {
            let (top, n) = on(jj);
            for ii in 0..ni {
                let u = &walked[at..at + 2 * n];
                let v = &walked[at + 2 * n..at + 4 * n];
                for r in 0..n { column.insert((ii, top + r), ([u[2 * r], u[2 * r + 1]], [v[2 * r], v[2 * r + 1]])); }
                at += 4 * n;
            }
        }
        // each run's pieces, a cell of the grid each: from the cell's left column toward its right
        let mut pieces = vec![];
        for (n, &(py, a, b)) in runs.iter().enumerate() {
            let mut x = a;
            while x < b {
                let cell = (x + 768).div_euclid(20);
                let left = 20 * cell - 768;
                let end = b.min(left + 20);
                let ii = (cell - g.i0) as usize;
                pieces.push((n, ii, py, x - left, end - x));
                x = end;
            }
        }
        let pairs: Vec<Vec<i32>> = pieces.iter().flat_map(|&(_, ii, py, _, _)| {
            let (l, r) = (column[&(ii, py)], column[&(ii + 1, py)]);
            [[l.0, r.0].concat(), [l.1, r.1].concat()]
        }).collect();
        let steps = self.go(host, "tn_step", &[], &pairs);
        let items: Vec<Vec<i32>> = pieces.iter().enumerate().flat_map(|(k, &(_, ii, py, pass, count))| {
            let l = column[&(ii, py)];
            [0, 1].map(|w| {
                let mut v = vec![0];
                v.extend(if w == 0 { l.0 } else { l.1 });
                v.extend(&steps[3 * (2 * k + w)..3 * (2 * k + w) + 3]);
                v.extend([pass, count]);
                v
            })
        }).collect();
        let tex = self.go(host, "tn_walk", &[], &items);
        let mut out: Vec<Vec<(i32, i32)>> = runs.iter().map(|_| vec![]).collect();
        let mut at = 0;
        for &(n, _, _, _, count) in &pieces {
            let count = count as usize;
            for q in 0..count { out[n].push((tex[at + q], tex[at + count + q])); }
            at += 2 * count;
        }
        out
    }

    /// A 3D frame from its scene: which piece each pixel shows, the outlines, the colours and the screen's texels.
    /// Returns the frame and, for the screen's pixels, their texels.
    fn frame(&self, host: &mut Host, s: &Scene, pic: &Picture) -> (Vec<u8>, Vec<((usize, usize), (i32, i32))>) {
        let spans = self.spans(host, &s.polys);
        // each row: the pieces' spans, cut at every end, each cut the piece in front of all the others there
        let mut shown: Vec<Vec<(i32, i32, usize)>> = vec![vec![]; H];
        for (py, row) in shown.iter_mut().enumerate() {
            let cy = py as i32 - 384;
            let here: Vec<(usize, (i32, i32))> = spans.iter().enumerate().filter_map(|(k, m)| m.get(&cy).map(|&sp| (k, sp))).collect();
            let mut cuts: Vec<i32> = here.iter().flat_map(|(_, (l, r))| [*l, *r]).collect();
            cuts.sort();
            cuts.dedup();
            for w in cuts.windows(2) {
                let (x0, x1) = (w[0], w[1]);
                let mut best: Option<usize> = None;
                for &(k, (l, r)) in &here {
                    if l > x0 || r < x1 { continue; }
                    best = Some(match best { None => k, Some(b) => if s.front[k][b] { k } else { b } });
                }
                if let Some(b) = best {
                    match row.last_mut() { Some(last) if last.1 == x0 && last.2 == b => last.1 = x1, _ => row.push((x0, x1, b)) }
                }
            }
        }
        // what each pixel shows (a face's way, or nothing), as runs; the outline machine's pieces of each row
        let label_runs: Vec<Vec<(i32, i32, i32)>> = shown.iter().map(|row| {
            let mut v: Vec<(i32, i32, i32)> = vec![];
            let mut x = -768;
            for &(a, b, k) in row {
                if a > x { v.push((x, a, NONE)); }
                let l = s.labels[k];
                match v.last_mut() { Some(last) if last.2 == l && last.1 == a => last.1 = b, _ => v.push((a, b, l)) }
                x = b;
            }
            if x < 768 { v.push((x, 768, NONE)); }
            v
        }).collect();
        let at = |runs: &[(i32, i32, i32)], x: i32| -> i32 { let x = if x < -768 { x + 1536 } else if x >= 768 { x - 1536 } else { x }; runs.iter().find(|r| r.0 <= x && x < r.1).unwrap().2 };
        let mut items = vec![];
        let mut pieces_of: Vec<Vec<(i32, i32, i32)>> = vec![];
        for py in 0..H {
            let (up, mid, down) = (&label_runs[(py + H - 1) % H], &label_runs[py], &label_runs[(py + 1) % H]);
            let mut cuts: Vec<i32> = up.iter().chain(mid).chain(down).flat_map(|r| [r.0, r.1]).collect();
            cuts.sort();
            cuts.dedup();
            let mut ps = vec![];
            for w in cuts.windows(2) {
                let (a, b) = (w[0], w[1]);
                let shows = at(mid, a);
                items.push(vec![a, b, at(up, a), shows, at(down, a), at(mid, a - 1), at(mid, b)]);
                ps.push((a, b, shows));
            }
            pieces_of.push(ps);
        }
        let marks = self.go(host, "tn_edge", &[], &items);
        // the screen's pixels' texels
        let screen_runs: Vec<(usize, i32, i32)> = shown.iter().enumerate().flat_map(|(py, row)| row.iter().filter(|r| s.screen[r.2]).map(move |&(a, b, _)| (py, a, b))).collect();
        let tex = match &s.grid { Some(g) => self.texels(host, g, &screen_runs), None => vec![] };
        let mut texel_at: Vec<((usize, usize), (i32, i32))> = vec![];
        for (k, &(py, a, _)) in screen_runs.iter().enumerate() {
            for (n, &t) in tex[k].iter().enumerate() { texel_at.push(((py, (a + 768) as usize + n), t)); }
        }
        let texel_map: std::collections::HashMap<(usize, usize), (i32, i32)> = texel_at.iter().copied().collect();
        // the colours: the outline's grey, the rim's, nothing's black, a piece's own or its texel
        let mut img = vec![0u8; W * H * 3];
        let mut m = 0;
        for py in 0..H {
            for &(a, b, shows) in &pieces_of[py] {
                let n = b - a;
                let kinds: Vec<i32> = if n == 1 { vec![marks[m]] } else if n == 2 { vec![marks[m], marks[m + 1]] } else { vec![marks[m], marks[m + 1], marks[m + 2]] };
                m += kinds.len();
                for x in a..b {
                    let mark = if x == a { kinds[0] } else if x == b - 1 { *kinds.last().unwrap() } else { kinds[1] };
                    let px = (x + 768) as usize;
                    let c = if mark == 1 { if shows == NONE { RIM } else { INK } } else if shows == NONE { [0, 0, 0] } else {
                        let k = shown[py].iter().find(|r| r.0 <= x && x < r.1).unwrap().2;
                        if s.screen[k] { let (u, v) = texel_map[&(py, px)]; pic.at(u, v) } else { s.colours[k] }
                    };
                    img[(py * W + px) * 3..][..3].copy_from_slice(&c);
                }
            }
        }
        (img, texel_at)
    }
}

// --- the CGA tunnel ------------------------------------------------------------------------------------------------

/// A pixel of the CGA tunnel as it is every frame (see the reference): what it is (0, 1, 2 a check), and for a check
/// its place round, along, its depth (hundredths); its dither's place.
#[derive(Clone, Copy, Default)]
struct Px { class: i32, a5: i32, v5: i32, dp: i32, bayer: i32 }

impl Turn {
    /// The CGA tunnel's pixels: a quarter of the picture worked out (its distance, depth, place along and angle), the
    /// rest by the angle's symmetry; each pixel's class from its own dither's thresholds.
    fn cga_pixels(&self, host: &mut Host) -> Vec<Px> {
        let mut c = Calc::new();
        let cga = &self.cga;
        let mut quarter = vec![];
        for y in 0..100i64 {
            for x in 0..160i64 {
                let (vx, vy) = (10 * x - 1595, 12 * y - 1194);
                let (nx, ny) = (c.num(vx), c.num(vy));
                let r2 = c.lin(&[(nx, Some(nx), false), (ny, Some(ny), false)]);
                let r2 = c.within(r2, 4_000_000);
                // the four pixels with this distance: (x, y), (319 - x, y), (x, 199 - y), (319 - x, 199 - y)
                let places = [(x, y), (319 - x, y), (x, 199 - y), (319 - x, 199 - y)];
                let flags: Vec<(Arg, Arg)> = places.iter().map(|&(px, py)| {
                    let b = cga.bayer[((py % 4) * 4 + px % 4) as usize] as usize;
                    let (hi, lo) = (c.num(cga.hi[b]), c.num(cga.lo[b]));
                    (c.lt(hi, r2), c.lt(lo, r2))
                }).collect();
                quarter.push((x, y, vx, vy, r2, flags));
            }
        }
        self.eval(host, &mut c);
        let mut worked = vec![];
        for &(x, y, vx, vy, r2, ref flags) in &quarter {
            if !flags.iter().any(|f| flag(&c, f.0)) { worked.push(None); continue; }
            let root = c.sqrt(r2);
            let root = c.within(root, 2000);
            let k = c.num(15_000_000);
            let d = c.div(k, root);
            let d = c.within(d, 30_000);
            let dp = c.divk(d, 10);
            let v7 = c.divk(d, 7);
            let v5 = c.modk(v7, 1000);
            let (nx, ny) = (c.num(vx), c.num(vy));
            let ang = c.atan2(ny, nx);
            // the angle in the other quarters: the third quarter's 500000 + b becomes 1500000 - a, 1000000 - a, a - 500000
            let a5s: Vec<N> = [(0, 1), (1_500_000, -1), (1_000_000, -1), (-500_000, 1)].iter().map(|&(base, sign)| {
                let b = c.num(base);
                let one = c.num(sign);
                let a = c.lin(&[(b, None, false), (ang, Some(one), false)]);
                let a = c.within(a, 1_000_000);
                let a8 = c.mulk(a, 8);
                let a8 = c.shift(a8);
                c.modk(a8, 1000)
            }).collect();
            let _ = (x, y);
            worked.push(Some((dp, v5, a5s)));
        }
        self.eval(host, &mut c);
        let mut out = vec![Px::default(); 320 * 200];
        for (q, &(x, y, _, _, _, ref flags)) in quarter.iter().enumerate() {
            let places = [(x, y), (319 - x, y), (x, 199 - y), (319 - x, 199 - y)];
            for (k, &(px, py)) in places.iter().enumerate() {
                let b = cga.bayer[((py % 4) * 4 + px % 4) as usize] as i32;
                let class = if flag(&c, flags[k].0) { 2 } else if flag(&c, flags[k].1) { 1 } else { 0 };
                let mut p = Px { class, bayer: b, ..Default::default() };
                if class == 2 {
                    let (dp, v5, a5s) = worked[q].as_ref().unwrap();
                    p.dp = value(&c, *dp) as i32;
                    p.v5 = value(&c, *v5) as i32;
                    p.a5 = value(&c, a5s[k]) as i32;
                }
                out[(py * 320 + px) as usize] = p;
            }
        }
        out
    }

    /// The CGA picture's indices at time tau: the frame's numbers (the turn, the twist's K, the travel), the turn at
    /// each depth (tn_twist), each row's colours (tn_cga); the brag laid on (its page's ink and box), the fade.
    fn cga_frames(&self, host: &mut Host, px: &[Px], frames: &[(usize, bool, i32)], brag: &[u8]) -> Vec<Vec<u8>> {
        let mut c = Calc::new();
        let mut nums = vec![];
        for &(tau, _, _) in frames {
            let tau = tau as i64;
            // fl(119366 tau, 10), a whole turn and more past frame 83: as 11936 tau + fl(6 tau, 10)
            // (the frame's time, scaled on the calculator)
            let tn = c.num(tau);
            let (whole, part) = (c.mulk(tn, 11_936), c.mulk(tn, 6));
            let part = c.divk(part, 10);
            let a9 = c.add(whole, part);
            let s9 = c.sin(a9).0;
            let p5 = c.mulk(s9, 300);
            let p5 = c.shift(p5);
            let p5 = c.modk(p5, 1000);
            let (whole, part) = (c.mulk(tn, 7957), c.mulk(tn, 7));
            let part = c.divk(part, 10);
            let a6 = c.add(whole, part);
            let s6 = c.sin(a6).0;
            let k = c.mulk(s6, 6);
            let k = c.within(k, 6000);
            let kl = c.long(k);
            let t = c.mulk(tn, 1100);
            let t = c.divk(t, 12);
            let t5 = c.modk(t, 1000);
            // each deck of the twist's table: its first depth's d K, its turn, its remainder
            let decks: Vec<(N, N)> = (0..DPS / DP_DECK).map(|d| {
                let start = c.num(DP0 + d * DP_DECK);
                let prod = c.mul(start, k);
                let q = c.shift(prod);
                let q = c.divk(q, 100);
                let q = c.within(q, 200);
                let k100000 = c.num(100_000);
                let rem = c.lin(&[(prod, None, false), (q, Some(k100000), true)]);
                let rem = c.within(rem, 99_999);
                let turn = c.add(q, p5);
                let turn = c.modk(turn, 1000);
                (turn, c.long(rem))
            }).collect();
            // the fade's level, 8 a frame of it (the frame given, from N3D + N_CGA - 1 on), or none
            let fd = frames[nums.len()].2;
            let level = if fd >= 0 { let x = c.num(fd as i64); let x = c.addk(x, -((N3D + N_CGA) as i64 - 1)); Some(c.mulk(x, 8)) } else { None };
            nums.push((kl, t5, decks, level));
        }
        self.eval(host, &mut c);
        let mut out = vec![];
        for (fi, &(_, with_brag, _)) in frames.iter().enumerate() {
            let (kl, t5, decks, level) = &nums[fi];
            let fade = level.map_or(0, |l| value(&c, l) as i32);
            let kc = c.cards(*kl).to_vec();
            let items: Vec<Vec<i32>> = decks.iter().map(|&(turn, rem)| { let r = c.cards(rem); vec![value(&c, turn) as i32, r[0], r[1], kc[0], kc[1], DP_DECK as i32] }).collect();
            let mut twist = vec![];
            for it in &items { twist.extend(host.gon(&self.m["tn_twist"], &[it])); }
            let travel = value(&c, *t5) as i32;
            let mut idx = vec![0u8; 320 * 200];
            for y in 0..200 {
                let mut deck = vec![travel, fade];
                for x in 0..320 {
                    let i = y * 320 + x;
                    let p = px[i];
                    if with_brag && brag[i] != 255 {
                        deck.push(brag[i] as i32);
                    } else if p.class == 2 {
                        let d = p.dp as i64 - DP0;
                        assert!((0..DPS).contains(&d), "a depth off the twist's table: {}", p.dp);
                        deck.extend([2, p.a5, twist[d as usize], p.v5]);
                    } else {
                        deck.push(p.class);
                    }
                    if fade > 0 { deck.push(6 * p.bayer + 3); }
                }
                let colours = host.go1(&self.m["tn_cga"], &[&deck], 320);
                for x in 0..320 { idx[y * 320 + x] = colours[x] as u8; }
            }
            out.push(idx);
        }
        out
    }

    fn cga_rgb(&self, idx: &[u8], tau: usize) -> Vec<u8> {
        let pal = &self.cga.pals[self.cga.flicks[(tau / 12) % 6]];
        idx.iter().flat_map(|&i| pal[3 * i as usize..3 * i as usize + 3].to_vec()).collect()
    }
}

impl Turn {
    /// The Mac's last picture, still going, at the turn's frames: the cold open's one-bit tunnel at its frames on (108
    /// + f, a picture every third frame), CARD ATTACK over it in full, the screen's corners; on the cold open's
    /// machines (segments/mac, fx/mac.rs) as it makes them, its turn from its sine table run on (tn_macturn).
    fn mac_pictures(&self, host: &mut Host, demo: &Path, frames: &[usize]) -> BTreeMap<usize, Vec<u8>> {
        use super::mac::desk::{self, BLOCK, TH, TW};
        let (mw, mh) = (desk::W, desk::H);
        let m = &self.m;
        let tables = desk::tables(demo);
        let sine: Vec<i32> = read(demo, "turn/data/mac.txt").lines().find(|l| l.starts_with("sine ")).unwrap().split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
        let (cards, order, sprites) = (desk::titles(demo), desk::dissolve(demo), desk::sprites(demo));
        // the tunnel's blocks, row by row: the angle round and the ring table's cards at the distance
        let mut blocks: Vec<[i32; 5]> = vec![];
        for iy in 0..TH as i32 {
            let off = host.go1(&m["mac_offset"], &[&[iy]], 3 * TW);
            let pairs: Vec<i32> = (0..TW).flat_map(|x| [off[3 * x], off[3 * x + 1]]).collect();
            let steps = host.go1(&m["mac_slope"], &[&pairs], TW);
            let entry = |x: usize| tables.slope[steps[x] as usize].map(|v| v as i32);
            let deck: Vec<i32> = (0..TW).flat_map(|x| [entry(x)[0], off[3 * x + 2]]).collect();
            let around = host.go1(&m["mac_angle"], &[&deck], TW);
            let deck: Vec<i32> = (0..TW).flat_map(|x| [entry(x)[1], off[3 * x]]).collect();
            let far = host.go1(&m["mac_reach"], &[&deck], TW);
            for x in 0..TW {
                let [along, twist, light, dark] = tables.ring[far[x] as usize].map(|v| v as i32);
                blocks.push([around[x], along, twist, light, dark]);
            }
        }
        // picture j: each block black (true) or white, Atkinson's dither carried down the rows
        let mut pictures: BTreeMap<usize, Vec<bool>> = BTreeMap::new();
        let mut tunnel = |host: &mut Host, j: usize| -> Vec<bool> {
            pictures.entry(j).or_insert_with(|| {
                let tw = host.go1(&m["tn_macturn"], &[&[j as i32], &sine], 3);
                let (turn, weight, travel) = (tw[0], tw[1], tw[2]);
                let twisted = host.go1(&m["mac_twist"], &[&[weight]], 61);
                let mut black = vec![false; TW * TH];
                let (mut passed, mut under) = (vec![0; TW], vec![0; TW]);
                for y in 0..TH {
                    let row = &blocks[y * TW..(y + 1) * TW];
                    let deck: Vec<i32> = row.iter().flat_map(|&[around, along, twist, light, dark]| [around, twisted[twist as usize], along, light, dark]).collect();
                    let shade = host.go1(&m["mac_trow"], &[&[turn, travel], &deck], TW);
                    let deck: Vec<i32> = (0..TW).flat_map(|x| [shade[x], passed[x], under[x]]).collect();
                    let out = host.go1(&m["mac_dither"], &[&[TW as i32], &deck], 3 * TW);
                    let mut at = 0;
                    for x in 0..TW {
                        black[y * TW + x] = out[at] == 1;
                        under[x] = out[at + 1];
                        at += 2;
                        if x > 0 { passed[x - 1] = out[at]; at += 1; }
                    }
                    passed[TW - 1] = out[at];
                }
                black
            }).clone()
        };
        // CARD ATTACK all the way in: the dissolve at three thirds (the kit's ease machine)
        let come = Text::new(demo).steps(host, 255, &[(3, 3)])[0];
        let card = &cards[2];
        let mut out = BTreeMap::new();
        for &f in frames {
            let pic = tunnel(host, (108 + f) / 3);
            let mut s: Vec<bool> = (0..mw * mh).map(|i| pic[(i / mw / BLOCK) * TW + i % mw / BLOCK]).collect();
            for y in 0..mh {
                let xs: Vec<usize> = (0..mw).filter(|&x| card[(y / BLOCK) * TW + x / BLOCK] > 0).collect();
                if xs.is_empty() { continue }
                let turns: Vec<i32> = xs.iter().map(|&x| order[y * mw + x] as i32).collect();
                let shown = host.go1(&m["mac_dissolve"], &[&[come, xs.len() as i32], &turns], xs.len());
                for (&x, &on) in xs.iter().zip(&shown) { if on == 1 { s[y * mw + x] = card[(y / BLOCK) * TW + x / BLOCK] == 1; } }
            }
            let (r, b) = (mw as i32 - 1, mh as i32 - 1);
            for (name, x, y) in [("corner_tl", 0, 0), ("corner_tr", r, 0), ("corner_bl", 0, b), ("corner_br", r, b)] {
                let sp = &sprites[name];
                for j in 0..sp.h { for i in 0..sp.w {
                    let (px, py) = (x + sp.ox as i32 + i as i32, y + sp.oy as i32 + j as i32);
                    if px < 0 || py < 0 || px >= mw as i32 || py >= mh as i32 { continue }
                    match sp.px[j * sp.w + i] { 1 => s[py as usize * mw + px as usize] = false, 2 => s[py as usize * mw + px as usize] = true, _ => {} }
                } }
            }
            out.insert(f, s.iter().flat_map(|&b| { let v = if b { 0 } else { 255 }; [v, v, v] }).collect());
        }
        out
    }
}

/// The brag on the CGA picture (the text kit's page, drawn on CARDIAC): 255 where it isn't, else its colour (3 the
/// ink, 0 the box).
fn brag_layer(host: &mut Host, demo: &Path) -> Vec<u8> {
    let page = &pages::CGA_BRAG;
    let px = Text::new(demo).draw(host, page, page.chars(), false);
    let mut out = vec![255u8; 320 * 200];
    for v in 0..page.h { for u in 0..page.w {
        let c = if px[(v * page.w + u) * 3] > 0 { 3 } else { 0 };
        for dy in 0..page.sy { for dx in 0..page.sx { out[(page.y + page.sy * v + dy) * 320 + page.x + page.sx * u + dx] = c; } }
    } }
    out
}

/// What every block of the turn needs, made once (by the player, once for all the blocks it splits the turn into: see
/// setup): the brag's layer, the CGA tunnel's pixels, and the last 3D frame's screen texels (where the CGA frames show
/// their pictures).
pub struct Once { brag: Vec<u8>, px: Vec<Px>, last: Vec<((usize, usize), (i32, i32))> }

impl Turn {
    fn once(&self, host: &mut Host, demo: &Path) -> Once {
        let brag = brag_layer(host, demo);
        let px = self.cga_pixels(host);
        let f = N3D - 1;
        let pic = self.cga_frames(host, &px, &[(f, false, -1)], &brag).remove(0);
        let pic = Picture::padded(&self.cga_rgb(&pic, f), 320, 200);
        let scene = self.scenes(host, &[f]).remove(0);
        let (_, last) = self.frame(host, &scene, &pic);
        Once { brag, px, last }
    }
}

/// The player's job before the turn's blocks: `once`, kept for them (live::keep).
pub fn setup(demo: &Path) {
    let mut host = Host::new();
    let once = Turn::new(demo).once(&mut host, demo);
    live::count(host.steps);
    live::keep("turn", once);
}

pub fn render(demo: &Path, refbin: &str) {
    let turn = Turn::new(demo);
    let mut host = Host::new();
    let kept = live::kept::<Once>("turn");
    let once = match kept { Some(k) => k, None => Arc::new(turn.once(&mut host, demo)) };
    let mut video = VideoHd::new(FRAMES);
    let only: Option<Vec<usize>> = std::env::var("TURN_FRAMES").ok().map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let block = live::range(FRAMES);
    let wanted = |f: usize| only.as_ref().is_none_or(|o| o.contains(&f)) && block.contains(&f);
    // the CGA tunnel's pictures: each frame the screen or the frame shows one (the fade a frame on in time)
    let mut cga_of: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let taus: Vec<(usize, usize, bool, i32)> = (MAC_FRAMES..N3D + N_CGA + FADE).filter(|&f| wanted(f)).map(|f| {
        // fading from N3D + N_CGA on: the frame, for its level on the calculator (cga_frames), or -1
        let fading = f >= N3D + N_CGA;
        (f, if fading { f + 1 } else { f }, f >= BRAG_FROM, if fading { f as i32 } else { -1 })
    }).collect();
    // the Mac's pictures, the 3D frames 8 at a time
    let three: Vec<usize> = (0..N3D).filter(|&f| wanted(f)).collect();
    let mac = turn.mac_pictures(&mut host, demo, &three.iter().copied().filter(|&f| f < MAC_FRAMES).collect::<Vec<_>>());
    for group in three.chunks(8) {
        live::time_frames(group[0]..group[group.len() - 1] + 1, || {
            let ts: Vec<&(usize, usize, bool, i32)> = taus.iter().filter(|t| group.contains(&t.0)).collect();
            if !ts.is_empty() {
                let pics = turn.cga_frames(&mut host, &once.px, &ts.iter().map(|&&(_, t, b, fd)| (t, b, fd)).collect::<Vec<_>>(), &once.brag);
                for (&&(f, tau, _, _), idx) in ts.iter().zip(pics) { cga_of.insert(f, turn.cga_rgb(&idx, tau)); }
            }
            let scenes = turn.scenes(&mut host, group);
            for (&f, s) in group.iter().zip(&scenes) {
                let pic = if f < MAC_FRAMES { Picture::padded(&mac[&f], 512, 342) } else { Picture::padded(&cga_of[&f], 320, 200) };
                let (img, _) = turn.frame(&mut host, s, &pic);
                video.buf[f * W * H * 3..(f + 1) * W * H * 3].copy_from_slice(&img);
            }
        });
    }
    // the CGA frames, where the last 3D frame's screen was, 16 at a time
    let flat: Vec<&(usize, usize, bool, i32)> = taus.iter().filter(|t| t.0 >= N3D).collect();
    for chunk in flat.chunks(16) {
        live::time_frames(chunk[0].0..chunk[chunk.len() - 1].0 + 1, || {
            let pics = turn.cga_frames(&mut host, &once.px, &chunk.iter().map(|&&(_, t, b, fd)| (t, b, fd)).collect::<Vec<_>>(), &once.brag);
            for (&&(f, tau, _, _), idx) in chunk.iter().zip(pics) {
                let pic = Picture::padded(&turn.cga_rgb(&idx, tau), 320, 200);
                let frame = &mut video.buf[f * W * H * 3..(f + 1) * W * H * 3];
                for &((py, px), (u, v)) in &once.last { frame[(py * W + px) * 3..][..3].copy_from_slice(&pic.at(u, v)); }
            }
        });
    }
    // the bezel's layer, for the cut's over line on the cold open: frame 0, covering (32) all but the cold open's screen
    // (made by the block that makes frame 0; a copy of computed pixels)
    let mut layer = cardiac_host::Video::<1536, 1024>::new(1);
    if wanted(0) {
        for y in 0..H { for x in 0..W {
            let i = (y * W + x) * 3;
            let screen = (256..1280).contains(&x) && (42..726).contains(&y);
            layer.buf[(y * W + x) * 4..][..4].copy_from_slice(&[video.buf[i], video.buf[i + 1], video.buf[i + 2], if screen { 0 } else { 32 }]);
        } }
    }
    host.done();
    let both = reference(refbin, &[demo]);
    let end = FRAMES * W * H * 3;
    let part = |a: usize, b: usize| &both[a.min(both.len())..b.min(both.len())];
    finish("turn", &host, &video, part(0, end), &demo.join("turn/video_hd.rgb"), "");
    finish("turn (the bezel's layer)", &Host::new(), &layer, part(end, both.len()), &demo.join("turn/bezel.rgba"), &format!(" (1536x768 RGB and cover, for the cut's over on the cold open: over turn/bezel.rgba 1536x768 0 0 0 {})", super::mac::FRAMES));
}
