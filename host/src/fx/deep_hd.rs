//! The dive's creatures at 1536x768: the dive's second video. Its water is the dive's own picture before the fish,
//! the marine snow and the jellyfish (192x96), each pixel made 8 x 8; over it, drawn sharp through the same camera:
//! the fish from the atlas, the marine snow as tiny rising bubbles, the jellyfish (bells from the bell atlas,
//! tentacles and oral arms as soft strands, their halo). segments/deep/reference_hd.rs says exactly what is drawn.
//!
//! Every number is worked out by the machines. A creature's numbers (where it is on the screen, which way it faces,
//! how big, its strands' points and pieces) are the swarm's sums (fx/swarm_calc.rs: products from quarter squares,
//! long division, square roots and angles from the swarm's printed tables), worked out a level at a time, the host
//! choosing by the machines' flags what to work out next. The pixels: walk machines step a sprite's texel (or a
//! strand's place across and along it) a pixel at a time across each row, the host fetches the texel by the index
//! they punch, and colour machines light, fog and mix it in. The host keeps the picture as 4 x 4 tiles, each one
//! colour until something is drawn in it (a copy of the dive's pixel), so the jellyfish's halos are mixed once a
//! tile where nothing finer is there.
use crate::*;
use super::swarm_calc::{Arg, Calc, Items, N, Tables};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub const HW: usize = 1536;
pub const HH: usize = 768;
pub type VideoHd = cardiac_host::Video<HW, HH>;
const TILE: usize = 4;
const TW: usize = HW / TILE;
const TH: usize = HH / TILE;
const BUBBLE: [i32; 3] = [225, 242, 250];

/// The atlases and tables (deep/data/hd.txt and the .bin files make_hd.py writes beside it).
pub struct Data {
    fish: Vec<u8>, fish_w: Vec<i32>, fish_h: Vec<i32>, fish_at: Vec<usize>,
    bell: Vec<u8>, bell_w: Vec<i32>, bell_at: Vec<usize>,
    bub: Vec<u8>, bub_at: Vec<usize>, bub_side: Vec<i32>, bub_phases: Vec<i32>, bub_size: Vec<i32>,
    cover: Vec<i32>, pale_glow: Vec<i32>, pale_rim: Vec<i32>, strand: Vec<i32>, halo: Vec<i32>, fine: Vec<i32>, taper: Vec<i32>, arm_taper: Vec<i32>,
    /// the orca at its 12 swim phases (the mesh kit's cards, data/make_orca.py), its haze by how far ahead (32nds)
    orcas: Vec<super::mesh::Model>, orca_haze: Vec<i32>,
    /// the ending's knob: its models, atlas and tables (data/make_knob.py)
    knob: KnobData,
    /// the swarm's printed tables, for the sums
    tables: Tables,
}

/// The atlases and tables, read once for every block of the dive the player splits it into.
pub fn data(demo: &Path) -> std::sync::Arc<Data> {
    static DATA: std::sync::OnceLock<std::sync::Arc<Data>> = std::sync::OnceLock::new();
    DATA.get_or_init(|| std::sync::Arc::new(Data::load(demo))).clone()
}

impl Data {
    pub fn load(demo: &Path) -> Data {
        let text = read(demo, "deep/data/hd.txt");
        let get = |name: &str| -> Vec<i32> { text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap_or_else(|| panic!("hd.txt: no {name}")).split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
        let bin = |n: &str| std::fs::read(demo.join("deep/data").join(n)).unwrap_or_else(|e| panic!("deep/data/{n}: {e} (make it with deep/data/make_hd.py)"));
        let us = |v: Vec<i32>| v.into_iter().map(|x| x as usize).collect::<Vec<usize>>();
        Data { fish: bin("hd_fish.bin"), fish_w: get("fish_w"), fish_h: get("fish_h"), fish_at: us(get("fish_at")),
            bell: bin("hd_bell.bin"), bell_w: get("bell_w"), bell_at: us(get("bell_at")),
            bub: bin("hd_bubble.bin"), bub_at: us(get("bubble_at")), bub_side: get("bubble_side"), bub_phases: get("bubble_phases"), bub_size: get("bubble_size"),
            orcas: (0..12).map(|k| super::mesh::Model::read(demo, &format!("deep/data/orca_{k}.txt"))).collect(), orca_haze: get("orca_haze"),
            knob: KnobData::load(demo),
            cover: get("cover"), pale_glow: get("pale_glow"), pale_rim: get("pale_rim"), strand: get("strand"), halo: get("halo"), fine: get("fine"),
            taper: get("taper"), arm_taper: get("arm_taper"),
            tables: super::swarm_calc::load_tables(&read(demo, "swarm/data/tables.txt")) }
    }
    fn fish_texel(&self, k: usize, row: i32, phase: i32, u: i32, v: i32) -> Option<[i32; 5]> {
        let (w, h) = (self.fish_w[k], self.fish_h[k]);
        if u < 0 || v < 0 || u >= w || v >= h { return None; }
        let at = self.fish_at[k] + (((row * 8 + phase) * w * h + v * w + u) * 5) as usize;
        let t = &self.fish[at..at + 5];
        Some([t[0] as i32, t[1] as i32, t[2] as i32, t[3] as i32, t[4] as i32])
    }
    fn bell_texel(&self, k: usize, cell: i32, u: i32, v: i32) -> Option<[i32; 2]> {
        let w = self.bell_w[k];
        if u < 0 || v < 0 || u >= w || v >= w { return None; }
        let at = self.bell_at[k] + ((cell * w * w + v * w + u) * 2) as usize;
        Some([self.bell[at] as i32, self.bell[at + 1] as i32])
    }
    fn bub_texel(&self, i: usize, sx: i32, sy: i32, u: i32, v: i32) -> i32 {
        let (side, ph) = (self.bub_side[i], self.bub_phases[i]);
        self.bub[self.bub_at[i] + ((sy * ph + sx) * side * side + v * side + u) as usize] as i32
    }
}

/// The swarm's machines the sums use.
const SUMS: &[&str] = &["sw_term", "sw_mag", "sw_qis", "sw_qp", "sw_lsum", "sw_shift", "sw_divd", "sw_join", "sw_floor3", "sw_sign",
    "sw_ang", "sw_sin", "sw_order", "sw_atan1", "sw_atan2", "sw_quad", "sw_hund", "sw_isqrt", "sw_isqrt2", "sw_short"];

pub struct Machines { walk: Machine, rows: Machine, cover: Machine, fcol: Machine, bub: Machine, jelly: Machine, eadd: Machine,
    gauss: Machine, hamt: Machine, mix256: Machine, mix: Machine, spill: Machine, sums: BTreeMap<&'static str, Machine>,
    ahead: Machine, keep: Machine, kit: super::mesh::Kit,
    klift: Machine, kindex: Machine, ksum: Machine, kahead: Machine, klit: Machine, kink: Machine, kseg: Machine, kmix256: Machine }

pub fn machines(demo: &Path) -> Machines {
    let r = |n: &str| rust(demo, n);
    Machines { walk: r("hd_walk"), rows: r("hd_rows"), cover: r("hd_cover"), fcol: r("hd_fcol"), bub: r("hd_bub"), jelly: r("hd_jelly"),
        eadd: r("hd_eadd"), gauss: r("hd_gauss"), hamt: r("hd_hamt"), mix256: r("hd_mix256"), mix: r("umix"), spill: r("dspillmix"), ahead: r("orca_ahead"), keep: r("orca_keep"), kit: super::mesh::Kit::new(demo),
        klift: r("k_lift"), kindex: r("k_index"), ksum: r("k_sum"), kahead: r("k_ahead"), klit: r("k_lit"), kink: r("k_ink"), kseg: r("k_seg"), kmix256: r("k_mix256"),
        sums: SUMS.iter().map(|&n| (n, r(n))).collect() }
}

/// What the dive's frame gives the creatures.
pub struct Scene<'a> {
    pub n: i32,
    /// the camera's place (hundredths of a unit) and its right, up and forward in 300ths (the ending's)
    pub cam: [i32; 3],
    pub r3: [i32; 3], pub u3: [i32; 3], pub f3: [i32; 3],
    pub f: [i32; 3], pub r: [i32; 3], pub u: [i32; 3],
    /// the fish: head, tail (tenths), flash (32nds), number
    pub fish: Vec<([i32; 3], [i32; 3], i32, i32)>,
    pub fish_light: i32,
    pub far: [i32; 3],
    pub show: i32, pub snow: [i32; 2],
    /// the marine snow's specks (tenths), as the dive's seed machine scattered them
    pub specks: &'a [i32],
    /// the jellyfish the dive drew, far ones first: how far ahead (tenths), its fade (64ths), place, turn, kind,
    /// radius, height, lengths, and the first frame the dive showed it
    pub jellies: Vec<(i32, i32, [i32; 3], i32, i32, i32, i32, [i32; 8], i32)>,
    pub wave: i32,
    pub sin60: &'a [i32],
    /// the dive's picture before the creatures, its depths, and that picture finished (caption, portal, whiteout)
    pub under: &'a [[i32; 3]], pub depth: &'a [i32], pub base: &'a [[i32; 3]],
    /// what is laid over every pixel, per pixel of the dive: the caption's fade (-1: none) and dots, the portal's
    /// mixes in order (a colour and 32nds, or the spill's colour and its tenths of 32nds to mix in 256ths with the
    /// portal's on), and the whiteout (32nds)
    pub caption: i32, pub dots: &'a [Vec<i32>],
    pub portal: &'a std::collections::HashMap<usize, Vec<(bool, [i32; 3], i32)>>, pub on: i32,
    pub white32: i32,
}

/// A number's cards as the number.
fn val(c: &Calc, n: N) -> i64 {
    let k = c.cards(n);
    match k.len() { 1 => k[0] as i64, 2 => k[0] as i64 * 1000 + k[1] as i64, 3 => k[0] as i64 * 1_000_000 + k[1] as i64 * 1000 + k[2] as i64, l => panic!("a number of {l} cards") }
}
fn flag(c: &Calc, a: Arg) -> bool { match a { Arg::Some(n, f, 1) => { let v = c.cards(n)[f as usize]; assert!(v == 0 || v == 1); v == 1 } _ => panic!("not a flag") } }
/// A number that fits a card, as the card.
fn cardv(c: &Calc, n: N) -> i32 { let v = val(c, n); assert!(v.abs() <= 999, "a card of {v}"); v as i32 }
/// A place (thousandths) as its whole and thousandths.
fn long(c: &Calc, n: N) -> [i32; 2] { let v = val(c, n); [v.div_euclid(1000) as i32, v.rem_euclid(1000) as i32] }

/// The sums worked out on the machines.
fn eval(host: &mut Host, m: &Machines, d: &Data, c: &mut Calc) { eval_at(host, m, d, c, 0) }
fn eval_at(host: &mut Host, m: &Machines, d: &Data, c: &mut Calc, at: u32) {
    let check = std::env::var("DEEP_CARDS").is_ok();
    let mut go = |name: &'static str, items: Items, out: &mut Vec<i32>| {
        let machine = m.sums.get(name).unwrap_or_else(|| panic!("no machine {name}"));
        let from = out.len();
        for (k, cards) in items.chunks(999) {
            let count = [k as i32];
            host.gon_into(machine, &[&count, cards], out);
        }
        if check { if let Some(v) = out[from..].iter().find(|v| v.abs() > 999) { panic!("{name} punched {v} at {at}"); } }
    };
    c.eval(&mut go, &d.tables);
}

/// A machine over items of `width` cards each, after its header cards and the count, punching `each` cards an item
/// (0: as many as it punches): at most 999 items a deck.
fn run(host: &mut Host, machine: &Machine, head: &[i32], items: &[i32], width: usize, each: usize) -> Vec<i32> {
    let mut out = vec![];
    for chunk in items.chunks(999 * width) {
        let mut deck = head.to_vec();
        deck.push((chunk.len() / width) as i32);
        deck.extend_from_slice(chunk);
        if let Some(i) = deck.iter().position(|v| v.abs() > 999) { panic!("{}: card {i} of {} is {}", machine.name, deck.len(), deck[i]); }
        // each 0: as many as it punches (a walk's pixels)
        out.extend(if each == 0 { host.gon(machine, &[&deck]) } else { host.go1(machine, &[&deck], chunk.len() / width * each) });
    }
    out
}

/// The picture: 4 x 4 tiles, each one colour (a copy of the dive's) until something finer is drawn in it.
struct Canvas { flat: Vec<[i32; 3]>, fine: Vec<Option<Box<[[i32; 3]; 16]>>>, touched: Vec<bool> }
impl Canvas {
    fn new(under: &[[i32; 3]]) -> Canvas {
        Canvas { flat: (0..TW * TH).map(|t| under[(t / TW) / 2 * 192 + (t % TW) / 2]).collect(), fine: vec![None; TW * TH], touched: vec![false; TW * TH] }
    }
    fn tile(x: usize, y: usize) -> (usize, usize) { ((y / TILE) * TW + x / TILE, (y % TILE) * TILE + x % TILE) }
    fn get(&self, x: usize, y: usize) -> [i32; 3] { let (t, k) = Self::tile(x, y); match &self.fine[t] { Some(f) => f[k], None => self.flat[t] } }
    fn set(&mut self, x: usize, y: usize, c: [i32; 3]) {
        let (t, k) = Self::tile(x, y);
        self.touched[t] = true;
        if self.fine[t].is_none() { self.fine[t] = Some(Box::new([self.flat[t]; 16])); }
        self.fine[t].as_mut().unwrap()[k] = c;
    }
}

/// Mix colours into pixels by amounts (32nds) on the mix machine, a channel at a time: (pixel x, y, colour, k), each
/// pixel once a batch, in order.
fn mix_in(host: &mut Host, m: &Machines, cv: &mut Canvas, items: &[(usize, usize, [i32; 3], i32)]) {
    let mut start = 0;
    while start < items.len() {
        let mut seen = HashSet::new();
        let mut end = start;
        while end < items.len() && seen.insert((items[end].0, items[end].1)) { end += 1; }
        let batch = &items[start..end];
        let mut out = vec![[0; 3]; batch.len()];
        for ch in 0..3 {
            let flat: Vec<i32> = batch.iter().flat_map(|&(x, y, c, k)| [cv.get(x, y)[ch], c[ch], k]).collect();
            for (e, v) in run(host, &m.mix, &[], &flat, 3, 1).into_iter().enumerate() { out[e][ch] = v; }
        }
        for (e, &(x, y, _, _)) in batch.iter().enumerate() { cv.set(x, y, out[e]); }
        start = end;
    }
}

/// The places a projection gives: how far ahead (fiftieths of a tenth), the flags that leave it out, and once seen
/// its place on the screen (thousandths of a pixel).
struct Seen { a: N, bx: N, cy: N, off: [Arg; 3], x: Option<N>, y: Option<N> }

/// A point (thousandths of a tenth, three numbers) seen from the camera, first stage: as the reference's project.
fn see(c: &mut Calc, p: [N; 3], s: &Scene) -> Seen {
    let dot = |c: &mut Calc, v: [i32; 3]| { let k: Vec<N> = v.iter().map(|&x| c.num(x as i64)).collect(); c.lin(&[(p[0], Some(k[0]), false), (p[1], Some(k[1]), false), (p[2], Some(k[2]), false)]) };
    let (df, dr, du) = (dot(c, s.f), dot(c, s.r), dot(c, s.u));
    let a = c.divk(df, 2000);
    let a = c.within(a, 99_999);
    let (bx, cy) = (c.divk(dr, 2000), c.divk(du, 2000));
    let near = c.num(150);
    let behind = c.lt(a, near);
    let six = c.mulk(a, 6);
    let (ax, ay) = (c.abs(bx), c.abs(cy));
    let (fx, fy) = (c.mulk(ax, 5), c.mulk(ay, 5));
    let wide = c.lt(six, fx);
    let tall = c.lt(six, fy);
    Seen { a, bx, cy, off: [behind, wide, tall], x: None, y: None }
}
fn seen(c: &Calc, s: &Seen) -> bool { !s.off.iter().any(|&f| flag(c, f)) }
/// The second stage, for a point seen: its place on the screen.
fn place(c: &mut Calc, s: &mut Seen) {
    // seen: no further across or up than 1.2 times ahead
    s.bx = c.within(s.bx, 120_000);
    s.cy = c.within(s.cy, 120_000);
    let nx = c.mulk(s.bx, 800);
    let qx = c.divrem(nx, s.a, 1).0;
    s.x = Some(c.addk(qx, 768_000));
    let ny = c.mulk(s.cy, 800);
    let qy = c.divrem(ny, s.a, 1).0;
    let mid = c.num(384_000);
    s.y = Some(c.sub(mid, qy));
}

/// A sprite's walk, as the reference's Walk: the box round (cx, cy) reaching `reach`, the place at its first pixel
/// (thousandths of a texel) from (w0, h0) at the middle, the steps across and down; and the place a further 999
/// pixels across.
struct WalkN { x0: N, x1: N, y0: N, y1: N, u0: N, v0: N, u9: N, v9: N, st: [N; 4] }
fn walk_sums(c: &mut Calc, cx: N, cy: N, reach: N, w0: i64, h0: i64, st: [N; 4]) -> WalkN {
    let (px, py) = (c.shift(cx), c.shift(cy));
    let (px, py) = (c.within(px, 1600), c.within(py, 900));
    let lo = c.sub(px, reach);
    let x0 = c.clamp(lo, 0, 2000);
    let hi = c.add(px, reach);
    let x1 = c.clamp(hi, -2000, HW as i64 - 1);
    let lo = c.sub(py, reach);
    let y0 = c.clamp(lo, 0, 2000);
    let hi = c.add(py, reach);
    let y1 = c.clamp(hi, -2000, HH as i64 - 1);
    let (half, one) = (c.num(500), c.num(1000));
    let dx = c.lin(&[(x0, Some(one), false), (half, None, false), (cx, None, true)]);
    let dy = c.lin(&[(y0, Some(one), false), (half, None, false), (cy, None, true)]);
    let (dx, dy) = (c.within(dx, 2_100_000), c.within(dy, 2_100_000));
    // dx st: the box reaches half the sprite's diagonal, so these are within the sprite's own texels (thousandths)
    let part = |c: &mut Calc, a: N, b: N| -> N { let p = c.lin_within(&[(a, Some(b), false)], Some(998_000_000)); c.shift(p) };
    let (a0, a1, a2, a3) = (part(c, dx, st[0]), part(c, dy, st[1]), part(c, dx, st[2]), part(c, dy, st[3]));
    let (w0, h0) = (c.num(w0), c.num(h0));
    let u0 = c.lin(&[(w0, None, false), (a0, None, false), (a1, None, false)]);
    let v0 = c.lin(&[(h0, None, false), (a2, None, false), (a3, None, false)]);
    let k999 = c.num(999);
    let u9 = c.lin(&[(u0, None, false), (st[0], Some(k999), false)]);
    let v9 = c.lin(&[(v0, None, false), (st[2], Some(k999), false)]);
    WalkN { x0, x1, y0, y1, u0, v0, u9, v9, st }
}

/// Walk a box: each row's start (the rows machine down its first column, and down column 999 for a wide box), then
/// each pixel's two wholes (the walk machine across, at most 999 pixels a walk). Calls f(x, y, u, v) a row at a time.
fn walk_box(host: &mut Host, m: &Machines, c: &Calc, w: &WalkN, mut f: impl FnMut(&mut Host, i32, Vec<(i32, i32, i32)>)) {
    // the box's pixels (places on the screen: the host's addresses, not cards)
    let (x0, x1, y0, y1) = (val(c, w.x0) as i32, val(c, w.x1) as i32, val(c, w.y0) as i32, val(c, w.y1) as i32);
    if x0 > x1 || y0 > y1 { return; }
    let st: Vec<[i32; 2]> = w.st.iter().map(|&s| long(c, s)).collect();
    let rows_from = |host: &mut Host, u: N, v: N| -> Vec<[i32; 4]> {
        let (u, v) = (long(c, u), long(c, v));
        let deck = [y1 - y0, u[0], u[1], st[1][0], st[1][1], v[0], v[1], st[3][0], st[3][1]];
        run(host, &m.rows, &[], &deck, 9, 4 * (y1 - y0 + 1) as usize).chunks(4).map(|o| [o[0], o[1], o[2], o[3]]).collect()
    };
    let first = rows_from(host, w.u0, w.v0);
    let second = if x1 - x0 + 1 > 999 { Some(rows_from(host, w.u9, w.v9)) } else { None };
    let mut items = vec![];
    for (r, p) in first.iter().enumerate() {
        let n = (x1 - x0 + 1).min(999);
        items.extend([n - 1, p[0], p[1], st[0][0], st[0][1], p[2], p[3], st[2][0], st[2][1]]);
        if let Some(sec) = &second { let p = sec[r]; items.extend([x1 - x0 - 999, p[0], p[1], st[0][0], st[0][1], p[2], p[3], st[2][0], st[2][1]]); }
    }
    let per = (x1 - x0 + 1) as usize;
    let out = run(host, &m.walk, &[], &items, 9, 0);
    // the walk machine punches as many pairs as each walk has pixels; deal them out row by row
    let _ = per;
    let mut at = 0;
    for (r, _) in first.iter().enumerate() {
        let y = y0 + r as i32;
        let mut px = Vec::with_capacity(per);
        for x in x0..=x1 { px.push((x, out[at], out[at + 1])); at += 2; }
        f(host, y, px);
    }
    assert_eq!(at, out.len());
}

pub fn frame(host: &mut Host, m: &Machines, d: &Data, s: &Scene) -> Vec<u8> {
    let mut cv = Canvas::new(s.under);
    let depth = |x: usize, y: usize| s.depth[(y / 8) * 192 + x / 8];
    // the orcas, far out in the water (before the fish, snow and jellyfish, which are nearer)
    orcas(host, m, d, s, &mut cv, &depth);
    fish(host, m, d, s, &mut cv, &depth);
    bubbles(host, m, d, s, &mut cv, &depth);
    for j in &s.jellies { jelly(host, m, d, s, j, &mut cv, &depth); }
    finish(host, m, s, &mut cv);
    ending(host, m, d, s, &mut cv);
    let mut out = vec![0u8; HW * HH * 3];
    for y in 0..HH { for x in 0..HW {
        let (t, _) = Canvas::tile(x, y);
        let c = if cv.touched[t] { cv.get(x, y) } else { s.base[(y / 8) * 192 + x / 8] };
        for ch in 0..3 { assert!((0..=255).contains(&c[ch]), "deep (1536x768): a channel out of 0..255"); out[(y * HW + x) * 3 + ch] = c[ch] as u8; }
    } }
    out
}

fn fish(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas, depth: &dyn Fn(usize, usize) -> i32) {
    if s.fish.is_empty() { return; }
    let mut c = Calc::new();
    // where each is, how far ahead
    let mut all = vec![];
    for &(h, t, flash, id) in &s.fish {
        let hn: Vec<N> = h.iter().map(|&v| c.num(v as i64)).collect();
        let tn: Vec<N> = t.iter().map(|&v| c.num(v as i64)).collect();
        let mid: Vec<N> = (0..3).map(|k| { let sum = c.add(hn[k], tn[k]); c.mulk(sum, 500) }).collect();
        let sn = see(&mut c, [mid[0], mid[1], mid[2]], s);
        let fk: Vec<N> = s.f.iter().map(|&v| c.num(v as i64)).collect();
        let hf = c.lin(&[(hn[0], Some(fk[0]), false), (hn[1], Some(fk[1]), false), (hn[2], Some(fk[2]), false)]);
        let tf = c.lin(&[(tn[0], Some(fk[0]), false), (tn[1], Some(fk[1]), false), (tn[2], Some(fk[2]), false)]);
        let (k300, k500) = (c.num(300), c.num(500));
        let skip = [c.lt(sn.a, k500), c.lt(hf, k300), c.lt(tf, k300)];
        all.push((sn, skip, hn, tn, flash, id));
    }
    // the light: 0.62 + 0.5 light (hundredths, at most 111)
    let light = c.num(s.fish_light as i64);
    let l50 = c.mulk(light, 50);
    let l32 = c.divk(l50, 32);
    let l62 = c.addk(l32, 62);
    let k111 = c.num(111);
    let lit = c.min(l62, k111);
    eval_at(host, m, d, &mut c, 1);
    let lit_v = val(&c, lit);
    let mut shown: Vec<usize> = (0..all.len()).filter(|&e| seen(&c, &all[e].0) && !all[e].1.iter().any(|&f| flag(&c, f))).collect();
    shown.sort_by(|&p, &q| val(&c, all[q].0.a).cmp(&val(&c, all[p].0.a)).then(p.cmp(&q)));
    // which way each faces, its size, its walk
    struct Pose { row: N, phase: N, levels: Vec<Arg>, mirror: Arg, sn: N, cs: N, fogi: N, near: N, kf: N }
    let mut poses = vec![];
    for &e in &shown {
        let (sn_, _, hn, tn, flash, id) = &mut all[e];
        place(&mut c, sn_);
        let m2: Vec<N> = (0..3).map(|k| c.add(hn[k], tn[k])).collect();
        let gx = c.sub(hn[0], tn[0]);
        let gz = c.sub(hn[2], tn[2]);
        // head to tail is 1.1 units (the script's): each part at most 20 tenths
        let (gx, gz) = (c.within(gx, 20), c.within(gz, 20));
        let rk: Vec<N> = s.r.iter().map(|&v| c.num(v as i64)).collect();
        let uk: Vec<N> = s.u.iter().map(|&v| c.num(v as i64)).collect();
        let side_r = c.lin(&[(m2[2], Some(rk[0]), false), (m2[0], Some(rk[2]), true)]);
        let flip = c.below0(side_r);
        let across0 = c.lin(&[(gx, Some(m2[2]), false), (gz, Some(m2[0]), true)]);
        let nacross = c.neg(across0);
        let across = c.sel(flip, nacross, across0);
        let away = c.lin(&[(gx, Some(m2[0]), false), (gz, Some(m2[2]), false)]);
        let aacross = c.abs(across);
        let yaw = c.atan2(away, aacross);
        let k24 = c.num(24);
        let big = c.lin(&[(yaw, Some(k24), false)]);
        let big = c.addk(big, 6_500_000);
        let s1 = c.shift(big);
        let s2 = c.shift(s1);
        let yi = c.modk(s2, 24);
        let mirror = c.below0(across);
        let sq = c.lin(&[(m2[0], Some(m2[0]), false), (m2[2], Some(m2[2]), false)]);
        let level = c.sqrt(sq);
        let lv = c.mulk(level, 268);
        let up = c.mulk(m2[1], -1000);
        let above = c.lt(lv, up);
        let su0 = c.lin(&[(m2[2], Some(uk[0]), false), (m2[0], Some(uk[2]), true)]);
        let nsu = c.neg(su0);
        let su = c.sel(flip, nsu, su0);
        let ar = c.abs(side_r);
        let roll = c.atan2(su, ar);
        let (sn, cs) = c.sin(roll);
        let idn = c.num(*id as i64);
        let i27 = c.mulk(idn, 27);
        let i10 = c.divk(i27, 10);
        let ph = c.addk(i10, s.n as i64);
        let phase = c.modk(ph, 8);
        let (k0, k13) = (c.num(0), c.num(13));
        let ab = c.sel(above, k13, k0);
        let row = c.add(ab, yi);
        let k5280 = c.num(5_280_000);
        let levels: Vec<Arg> = (0..9).map(|k| { let p = c.mulk(sn_.a, 7 * d.fish_w[k] as i64); c.lt(p, k5280) }).collect();
        let a500 = c.divk(sn_.a, 500);
        let k49 = c.num(49);
        let fogi = c.min(a500, k49);
        let a49 = c.addk(sn_.a, 49);
        let near = c.divk(a49, 50);
        let fln = c.num(*flash as i64);
        let fl55 = c.mulk(fln, 55);
        let kf = c.divk(fl55, 32);
        poses.push(Pose { row, phase, levels, mirror, sn, cs, fogi, near, kf });
    }
    eval_at(host, m, d, &mut c, 2);
    // the walks and colours
    struct Drawn { k: usize, walk: WalkN, g: N, fogc: [N; 3] }
    let mut drawn = vec![];
    for (p, &e) in poses.iter().zip(&shown) {
        let k = (0..9).find(|&k| flag(&c, p.levels[k])).unwrap();
        let a = all[e].0.a;
        let s1000 = { let x = c.mulk(a, 7 * d.fish_w[k] as i64); let q = c.divk(x, 5280); c.within(q, 999) };
        let t1000 = { let x = c.mulk(a, 14 * d.fish_h[k] as i64); let q = c.divk(x, 5280); c.within(q, 999) };
        let mir = flag(&c, p.mirror);
        let step = |c: &mut Calc, x: N, y: N, neg: bool| -> N { let l = c.lin(&[(x, Some(y), neg)]); let q = c.divk(l, 999); c.within(q, 999) };
        let st = [step(&mut c, p.cs, s1000, mir), step(&mut c, p.sn, s1000, !mir), step(&mut c, p.sn, t1000, false), step(&mut c, p.cs, t1000, false)];
        let k422 = c.num(422_000);
        let r0 = c.div(k422, a);
        let reach = c.addk(r0, 2);
        let reach = c.within(reach, 1000);
        let walk = walk_sums(&mut c, all[e].0.x.unwrap(), all[e].0.y.unwrap(), reach, 500 * d.fish_w[k] as i64, 500 * d.fish_h[k] as i64, st);
        let fog = c.num(demokit::tables::FISH_FOG[val(&c, p.fogi) as usize] as i64);
        let lit = c.num(lit_v);
        let left = { let k32 = c.num(32); c.sub(k32, fog) };
        let g = { let x = c.mul(lit, left); c.divk(x, 32) };
        let fogc = s.far.map(|v| { let x = c.num(v as i64); let y = c.mul(x, fog); c.divk(y, 32) });
        drawn.push(Drawn { k, walk, g, fogc });
    }
    eval_at(host, m, d, &mut c, 3);
    for ((p, dr), &_e) in poses.iter().zip(&drawn).zip(&shown) {
        let (row, phase, k) = (cardv(&c, p.row), cardv(&c, p.phase), dr.k);
        let (g, kf, near) = (cardv(&c, dr.g), cardv(&c, p.kf), cardv(&c, p.near));
        let fogc = dr.fogc.map(|n| cardv(&c, n));
        let mut px = vec![];
        walk_box(host, m, &c, &dr.walk, |_, y, row_px| {
            for (x, u, v) in row_px {
                let Some(t) = d.fish_texel(k, row, phase, u, v) else { continue };
                if t[3] == 0 { continue; }
                px.push((x as usize, y as usize, t));
            }
        });
        if px.is_empty() { continue; }
        let covers = run(host, &m.cover, &[near], &px.iter().flat_map(|&(x, y, t)| [depth(x, y), t[3]]).collect::<Vec<_>>(), 2, 1);
        let shine = run(host, &m.fcol, &[kf, 0], &px.iter().flat_map(|&(_, _, t)| [t[4], 0]).collect::<Vec<_>>(), 2, 1);
        let cols: Vec<Vec<i32>> = (0..3).map(|ch| run(host, &m.fcol, &[g, fogc[ch]], &px.iter().zip(&shine).flat_map(|(&(_, _, t), &sh)| [t[ch], sh]).collect::<Vec<_>>(), 2, 1)).collect();
        let items: Vec<(usize, usize, [i32; 3], i32)> = px.iter().enumerate().filter(|(e, _)| covers[*e] > 0).map(|(e, &(x, y, _))| (x, y, [cols[0][e], cols[1][e], cols[2][e]], covers[e])).collect();
        mix_in(host, m, cv, &items);
    }
}

fn bubbles(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas, depth: &dyn Fn(usize, usize) -> i32) {
    let mut c = Calc::new();
    let mut all = vec![];
    for e in (0..s.specks.len() / 3).filter(|e| e % 4 == 0) {
        let sp = &s.specks[3 * e..3 * e + 3];
        let (spy, up, nn, k70) = (c.num(sp[1] as i64), c.num(s.snow[0] as i64), c.num(s.n as i64), c.num(70));
        let k3 = c.num(3);
        let y0 = c.lin(&[(spy, None, false), (up, None, true), (nn, Some(k3), false), (k70, None, false)]);
        let ym = c.modk(y0, 140);
        let y = c.addk(ym, -70);
        for copy in 0..3i64 {
            let (spz, along, k20) = (c.num(sp[2] as i64), c.num(s.snow[1] as i64), c.num(20));
            let z0 = c.lin(&[(spz, None, false), (along, None, true), (k20, None, false)]);
            let zm = c.modk(z0, 140);
            let z = c.addk(zm, -20 + 140 * copy);
            let x = c.num(sp[0] as i64);
            let (k1, k2, k0) = (c.num(58_887), c.num(17_507), c.num(111_408));
            let ang = c.lin(&[(nn, Some(k0), false), (x, Some(k1), false), (z, Some(k2), false)]);
            let (sn, _) = c.sin(ang);
            let w4 = c.mulk(sn, 400);
            let wob = c.divk(w4, 999);
            let k1000 = c.num(1000);
            let px = c.lin(&[(x, Some(k1000), false), (wob, None, false)]);
            let py = c.mulk(y, 1000);
            let pz = c.mulk(z, 1000);
            all.push(see(&mut c, [px, py, pz], s));
        }
    }
    eval_at(host, m, d, &mut c, 4);
    let mut shown = vec![];
    for (e, sn) in all.iter_mut().enumerate() {
        if !seen(&c, sn) { continue; }
        place(&mut c, sn);
        let a10 = c.divk(sn.a, 50);
        let k400 = c.num(400);
        let far = c.lt(a10, k400);
        let left = c.sub(k400, a10);
        let show = c.num(s.show as i64);
        let ks = c.mul(left, show);
        let k32 = c.divk(ks, 400);
        let a49 = c.addk(sn.a, 49);
        let near = c.divk(a49, 50);
        shown.push((e, a10, far, k32, near));
    }
    eval_at(host, m, d, &mut c, 5);
    let mut subs = vec![];
    for &(e, a10, far, k32, near) in &shown {
        if !flag(&c, far) { continue; }
        let sn = &all[e];
        let (xx, yy) = (sn.x.unwrap(), sn.y.unwrap());
        let (px, py) = (c.shift(xx), c.shift(yy));
        let (rx, ry) = (c.modk(xx, 1000), c.modk(yy, 1000));
        let (sx, sy) = (c.divk(rx, 250), c.divk(ry, 250));
        subs.push((a10, k32, near, px, py, sx, sy));
    }
    eval_at(host, m, d, &mut c, 6);
    for &(a10, k32, near, px, py, sx, sy) in &subs {
        let i = d.bub_size[cardv(&c, a10) as usize] as usize;
        let (side, ph) = (d.bub_side[i], d.bub_phases[i]);
        let (px, py) = (val(&c, px) as i32, val(&c, py) as i32);
        let (sx, sy) = if ph == 4 { (cardv(&c, sx), cardv(&c, sy)) } else { (0, 0) };
        let h = side / 2;
        let mut pix = vec![];
        for v in 0..side { for u in 0..side {
            let (x, y) = (px - h + u, py - h + v);
            if x < 0 || y < 0 || x >= HW as i32 || y >= HH as i32 { continue; }
            let a8 = d.bub_texel(i, sx, sy, u, v);
            if a8 == 0 { continue; }
            pix.push((x as usize, y as usize, a8));
        } }
        if pix.is_empty() { continue; }
        let covers = run(host, &m.bub, &[cardv(&c, k32), cardv(&c, near)], &pix.iter().flat_map(|&(x, y, a8)| [depth(x, y), a8]).collect::<Vec<_>>(), 2, 1);
        let items: Vec<(usize, usize, [i32; 3], i32)> = pix.iter().zip(&covers).filter(|(_, &k)| k > 0).map(|(&(x, y, _), &k)| (x, y, BUBBLE, k)).collect();
        mix_in(host, m, cv, &items);
    }
}

/// A strand's piece's numbers: its glow f, its walk (across, in eighths of its width; along, in hundredths of it).
struct Piece { skip: Vec<Arg>, f: Option<N>, walk: Option<WalkN> }

/// A strand's points on the screen (thousandths of a pixel) and its width (thousandths of a pixel): its pieces'
/// sums, the first stage (what leaves a piece out: too long a step to work out, or shorter than its width).
fn pieces_first(c: &mut Calc, pts: &[Option<(N, N)>], sigma: N) -> Vec<Option<(Piece, N, N, N, N)>> {
    (0..pts.len() - 1).map(|h| {
        let (Some(p), Some(q)) = (pts[h], pts[h + 1]) else { return None };
        let ddx = c.sub(q.0, p.0);
        let ddy = c.sub(q.1, p.1);
        let dx = c.divk(ddx, 100);
        let dy = c.divk(ddy, 100);
        let (dx, dy) = (c.within(dx, 99_999), c.within(dy, 99_999));
        let (adx, ady) = (c.abs(dx), c.abs(dy));
        let k999 = c.num(999);
        let skip = vec![c.lt(k999, adx), c.lt(k999, ady)];
        let _ = sigma;
        Some((Piece { skip, f: None, walk: None }, dx, dy, dx, p.0))
    }).collect()
}

/// The second stage: for a piece not left out by its steps, its length (tenths of a pixel), and whether it is
/// shorter than its width.
fn pieces_len(c: &mut Calc, pieces: &mut [Option<(Piece, N, N, N, N)>], sigma: N) {
    for pc in pieces.iter_mut() {
        let Some((piece, dx, dy, len, _)) = pc else { continue };
        if piece.skip.iter().any(|&f| flag(c, f)) { *pc = None; continue; }
        // each step at most 999 tenths of a pixel either way
        let (x, y) = (c.within(*dx, 999), c.within(*dy, 999));
        let sq = c.lin(&[(x, Some(x), false), (y, Some(y), false)]);
        *len = c.sqrt(sq);
        let l1000 = c.mulk(*len, 1000);
        let s10 = c.mulk(sigma, 10);
        piece.skip = vec![c.lt(l1000, s10)];
    }
}

fn jelly(host: &mut Host, m: &Machines, d: &Data, s: &Scene, j: &(i32, i32, [i32; 3], i32, i32, i32, i32, [i32; 8], i32), cv: &mut Canvas, depth: &dyn Fn(usize, usize) -> i32) {
    let &(ahead, fade1, cpos, turn, kind, rad, ht, lengths, first) = j;
    let mut c = Calc::new();
    // the fade: eased out of the dark, and in over 3 s from the first frame it showed
    let f1 = c.num(fade1 as i64);
    let f5 = c.mulk(f1, 5);
    let f2 = c.divk(f5, 2);
    let k64 = c.num(64);
    let fm = c.min(f2, k64);
    let sq = c.mul(fm, fm);
    let eased = c.divk(sq, 64);
    let (nn, fst, one) = (c.num(s.n as i64), c.num(first as i64), c.num(1));
    let since = c.lin(&[(nn, None, false), (fst, None, true), (one, None, false)]);
    let s64 = c.mulk(since, 64);
    let r36 = c.divk(s64, 36);
    let rise = c.min(r36, k64);
    let fade = c.min(eased, rise);
    let cn: Vec<N> = cpos.iter().map(|&v| c.num(v as i64)).collect();
    let cm: Vec<N> = cn.iter().map(|&v| c.mulk(v, 1000)).collect();
    let mut centre = see(&mut c, [cm[0], cm[1], cm[2]], s);
    eval_at(host, m, d, &mut c, 7);
    let fade_v = cardv(&c, fade);
    if fade_v == 0 || !seen(&c, &centre) { return; }
    place(&mut c, &mut centre);
    let a = centre.a;
    let (cx, cy) = (centre.x.unwrap(), centre.y.unwrap());
    let k4e7 = c.num(40_000_000);
    let ppt = c.div(k4e7, a);
    let ppt = c.within(ppt, 266_667);
    let glow = if kind == 1 { [220, 150, 255] } else { [140, 200, 255] };
    let pale = [230, 245, 255];
    // the bell's cells: its pulse, the views either side of its height, turned as the screen's up is
    let radn = c.num(rad as i64);
    let r7 = c.mulk(radn, 7);
    let r7 = c.addk(r7, -580);
    let pr = c.divk(r7, 30);
    let pulse = c.clamp(pr, 0, 7);
    let sq = c.lin(&[(cn[0], Some(cn[0]), false), (cn[2], Some(cn[2]), false)]);
    let level = c.sqrt(sq);
    let ny = c.neg(cn[1]);
    let em0 = c.atan2(ny, level);
    let half = c.num(500_000);
    let hi = c.lt(em0, half);
    let em1 = c.addk(em0, -1_000_000);
    let em = c.sel(hi, em0, em1);
    let from0 = c.addk(em, 83_333);
    let from = c.clamp(from0, 0, 6 * 41_667 - 1);
    let va = c.divk(from, 41_667);
    let va = c.within(va, 5);
    let va41 = c.mulk(va, 41_667);
    let rest = c.sub(from, va41);
    let r32 = c.mulk(rest, 32);
    let w = c.divk(r32, 41_667);
    let w = c.within(w, 32);
    let nr1 = c.num(-s.r[1] as i64);
    let u1 = c.num(s.u[1] as i64);
    let up = c.atan2(nr1, u1);
    let (sn, cs) = c.sin(up);
    let k1e6 = c.num(1_000_000);
    let levels: Vec<Arg> = (0..13).map(|k| { let p = c.mulk(a, d.bell_w[k] as i64); c.lt(p, k1e6) }).collect();
    let (sx_, sy_) = (c.shift(cx), c.shift(cy));
    let ox = c.addk(sx_, -768);
    let oy = c.addk(sy_, -384);
    let osq = c.lin(&[(ox, Some(ox), false), (oy, Some(oy), false)]);
    let off = c.sqrt(osq);
    let off2 = c.lin(&[(off, Some(off), false)]);
    let off2 = c.addk(off2, 640_000);
    let root = c.sqrt(off2);
    let root = c.within(root, 99_999);
    let k8e5 = c.num(800_000);
    let inv = c.div(k8e5, root);
    let inv = c.within(inv, 1000);
    let pp150 = c.mulk(ppt, 150);
    let s150 = c.divk(pp150, 1000);
    let k420 = c.num(420);
    let sigma = c.max(s150, k420);
    let sigma = c.within(sigma, 40_100);
    let pp350 = c.mulk(ppt, 350);
    let s350 = c.divk(pp350, 1000);
    let k800 = c.num(800);
    let arm_sigma = c.max(s350, k800);
    let arm_sigma = c.within(arm_sigma, 93_400);
    let rp = c.mul(radn, ppt);
    let a1 = c.divk(rp, 1000);
    let a1 = c.within(a1, 31_000);
    // the halo's box and walk: hundredths of its width across and down, a 4 x 4's middle at a time
    let aheadn = c.num(ahead as i64);
    let k336 = c.num(33_600);
    let rch = c.div(k336, aheadn);
    let reach_h = c.addk(rch, 1);
    let reach_h = c.within(reach_h, 3_400);
    let lo = c.sub(sx_, reach_h);
    let tx0 = { let q = c.divk(lo, TILE as i64); c.clamp(q, 0, 2000) };
    let hi = c.add(sx_, reach_h);
    let tx1 = { let q = c.divk(hi, TILE as i64); c.clamp(q, -2000, TW as i64 - 1) };
    let lo = c.sub(sy_, reach_h);
    let ty0 = { let q = c.divk(lo, TILE as i64); c.clamp(q, 0, 2000) };
    let hi = c.add(sy_, reach_h);
    let ty1 = { let q = c.divk(hi, TILE as i64); c.clamp(q, -2000, TH as i64 - 1) };
    let a4 = c.mulk(aheadn, 1000 * TILE as i64);
    let hstep = c.divk(a4, 112);
    let hstep = c.within(hstep, 21_500);
    let (k4000, k2000) = (c.num(1000 * TILE as i64), c.num(2000));
    let du0 = c.lin(&[(tx0, Some(k4000), false), (k2000, None, false), (cx, None, true)]);
    let du0 = c.within(du0, 2_000_000);
    let dv0 = c.lin(&[(ty0, Some(k4000), false), (k2000, None, false), (cy, None, true)]);
    let dv0 = c.within(dv0, 2_000_000);
    // the halo's box reaches 3 widths, 33600 / ahead pixels: these are within 3.4 widths (hundredths, thousandths)
    let hu0 = { let x = c.lin_within(&[(du0, Some(aheadn), false)], Some(40_000_000)); let q = c.divk(x, 112); c.within(q, 400_000) };
    let hv0 = { let x = c.lin_within(&[(dv0, Some(aheadn), false)], Some(40_000_000)); let q = c.divk(x, 112); c.within(q, 400_000) };
    eval_at(host, m, d, &mut c, 8);
    let k = (0..13).find(|&k| flag(&c, levels[k])).unwrap();
    let s1000 = { let x = c.mulk(a, d.bell_w[k] as i64); let q = c.divk(x, 1000); c.within(q, 999) };
    let offv = val(&c, off);
    let (ex, ey) = if offv == 0 { (c.num(0), c.num(0)) } else { let (x, y) = (c.mulk(ox, 1000), c.mulk(oy, 1000)); let off = c.within(off, 99_999); let (qx, qy) = (c.div(x, off), c.div(y, off)); (c.within(qx, 1000), c.within(qy, 1000)) };
    let short = { let k = c.num(1000); c.sub(k, inv) };
    let short = c.within(short, 1000);
    let sm = |c: &mut Calc, p: N, q: N| -> N { let a = c.mul(short, p); let b = c.divk(a, 1000); let b = c.within(b, 1000); let b2 = c.mul(b, q); let r = c.divk(b2, 1000); c.within(r, 1000) };
    let xx = sm(&mut c, ex, ex);
    let xy = sm(&mut c, ex, ey);
    let yy = sm(&mut c, ey, ey);
    let k1000 = c.num(1000);
    let sxx = c.sub(k1000, xx);
    let sxy = c.neg(xy);
    let syy = c.sub(k1000, yy);
    let (sxx, sxy, syy) = (c.within(sxx, 1000), c.within(sxy, 1000), c.within(syy, 1000));
    let step = |c: &mut Calc, x: N, y: N, neg: bool| -> N { let l = c.lin(&[(x, Some(y), neg)]); let q = c.divk(l, 999); c.within(q, 999) };
    let r0 = [step(&mut c, cs, s1000, false), step(&mut c, sn, s1000, true), step(&mut c, sn, s1000, false), step(&mut c, cs, s1000, false)];
    let comb = |c: &mut Calc, a: N, p: N, b: N, q: N| -> N { let l = c.lin(&[(a, Some(p), false), (b, Some(q), false)]); let q = c.divk(l, 1000); c.within(q, 2_000) };
    let st = [comb(&mut c, r0[0], sxx, r0[1], sxy), comb(&mut c, r0[0], sxy, r0[1], syy), comb(&mut c, r0[2], sxx, r0[3], sxy), comb(&mut c, r0[2], sxy, r0[3], syy)];
    let k706 = c.num(706_667);
    let q0 = c.div(k706, a);
    let q1 = c.mulk(q0, 1000);
    let inv9 = c.within(inv, 99_999);
    let q2 = c.div(q1, inv9);
    let reach = c.addk(q2, 2);
    let reach = c.within(reach, 2000);
    let bell_walk = walk_sums(&mut c, cx, cy, reach, 500 * d.bell_w[k] as i64, 500 * d.bell_w[k] as i64, st);
    // the tentacles' points: the angle each hangs from, then its 25 points
    let wave = s.wave as i64;
    let mut tents = vec![];
    for tn in 0..24i64 {
        let main = tn % 3 == 0;
        let li = (if main { tn / 3 } else { tn } % 8) as usize;
        let ln0 = c.num(lengths[li] as i64);
        let lnn = c.mulk(ln0, d.fine[tn as usize] as i64);
        // its angle, (8 tn + 3 turn) / 192 of a turn: the whole turns taken off first (a quotient is under a million)
        let (tr, k3, tnn, k8) = (c.num(turn as i64), c.num(3), c.num(tn), c.num(8));
        let a0 = c.lin(&[(tr, Some(k3), false), (tnn, Some(k8), false)]);
        let a0 = c.modk(a0, 192);
        let a0 = c.mulk(a0, 1_000_000);
        let ang = c.divk(a0, 192);
        let (sa, ca) = c.sin(ang);
        // the tentacle's phase down it, 19 tn / 6 steps of 60
        let t19 = c.mulk(tnn, 19);
        let ph = c.divk(t19, 6);
        let wv_ = c.num(wave);
        let idx: Vec<N> = (0..=24i64).map(|h| { let hh = c.num(2 * h); let x = c.lin(&[(hh, None, false), (ph, None, false), (wv_, None, false)]); c.modk(x, 60) }).collect();
        let rr: Vec<N> = (0..=24i64).map(|h| { let x = c.mulk(radn, 18400 - 115 * h); let q = c.divk(x, 200); c.within(q, 11_000) }).collect();
        tents.push((main, lnn, sa, ca, idx, rr));
    }
    // the arms' points
    let mut arms = vec![];
    for an in 0..4i64 {
        let (tr, base) = (c.num(turn as i64), c.num(16 * an + 8));
        let a0 = c.add(tr, base);
        let a0 = c.modk(a0, 64);
        let a0 = c.mulk(a0, 1_000_000);
        let ang = c.divk(a0, 64);
        let (sa, ca) = c.sin(ang);
        for off in [-2i64, 0, 2] {
            let mut pts = vec![];
            for i in 0..=20i64 {
                let (wn, ii, ann) = (c.num(wave), c.num(i), c.num(an));
                let (k125, k334, k159) = (c.num(125_000), c.num(33_423), c.num(159_155));
                let wa = c.lin(&[(ii, Some(k125), false), (wn, Some(k334), false), (ann, Some(k159), false)]);
                let (ws, _) = c.sin(wa);
                let w15 = c.mulk(ws, 1500);
                let wv = c.divk(w15, 999);
                let wv = c.within(wv, 1500);
                let r12 = c.mulk(radn, 12 * (10 + i));
                let ra = c.divk(r12, 10);
                let ra = c.within(ra, 41_400);
                let (k318, offn) = (c.num(318_310), c.num(off));
                let fa = c.lin(&[(ii, Some(k318), false), (offn, Some(k159), false)]);
                let (fs, _) = c.sin(fa);
                let f4 = c.mulk(fs, 400);
                let f9 = c.divk(f4, 999);
                let f9 = c.within(f9, 400);
                let frill = c.addk(f9, 1000);
                let af = c.mul(a1, frill);
                let af = c.divk(af, 1000);
                let af = c.within(af, 44_000);
                let i12 = c.num(12 + i);
                let sh = c.lin(&[(af, Some(i12), false)]);
                let sh = c.lin(&[(sh, Some(offn), false)]);
                let shift = c.divk(sh, 6);
                pts.push((wv, ra, shift));
            }
            arms.push((ca, sa, off, pts));
        }
    }
    eval_at(host, m, d, &mut c, 9);
    // the bell
    let (pulse_v, va_v, w_v) = (cardv(&c, pulse), cardv(&c, va), cardv(&c, w));
    let mut emit: BTreeMap<usize, (i32, i32)> = BTreeMap::new();
    let mut bell_px = vec![];
    walk_box(host, m, &c, &bell_walk, |_, y, row| {
        for (x, u, v) in row {
            let (Some(ta), Some(tb)) = (d.bell_texel(k, 7 * pulse_v + va_v, u, v), d.bell_texel(k, 7 * pulse_v + va_v + 1, u, v)) else { continue };
            if ta == [0, 0] && tb == [0, 0] { continue; }
            bell_px.push((y as usize * HW + x as usize, ta, tb));
        }
    });
    if !bell_px.is_empty() {
        let g = run(host, &m.mix, &[], &bell_px.iter().flat_map(|&(_, ta, tb)| [ta[0], tb[0], w_v]).collect::<Vec<_>>(), 3, 1);
        let e = run(host, &m.mix, &[], &bell_px.iter().flat_map(|&(_, ta, tb)| [ta[1], tb[1], w_v]).collect::<Vec<_>>(), 3, 1);
        for (n, &(i, _, _)) in bell_px.iter().enumerate() { emit.insert(i, (g[n], e[n])); }
    }
    // the strands' points in the water, then on the screen
    let c1000: Vec<N> = cn.iter().map(|&v| c.mulk(v, 1000)).collect();
    let htn = c.num(ht as i64);
    let ht2 = c.mulk(htn, 2);
    let ytop = c.add(c1000[1], ht2);
    let mut tent_pts = vec![];
    for (_, lnn, sa, ca, idx, rr) in &tents {
        let mut pts = vec![];
        for h in 0..=24usize {
            let sw = c.num(s.sin60[val(&c, idx[h]) as usize] as i64);
            let sv = c.mulk(sw, 5 * h as i64);
            let p = |c: &mut Calc, a: N, b: N, neg: bool| -> N { let l = c.lin(&[(a, Some(b), neg)]); c.divk(l, 999) };
            let x1 = p(&mut c, rr[h], *ca, false);
            let x2 = p(&mut c, *sa, sv, false);
            let z1 = p(&mut c, rr[h], *sa, false);
            let z2 = p(&mut c, *ca, sv, false);
            let x = c.lin(&[(c1000[0], None, false), (x1, None, false), (x2, None, false)]);
            let z = c.lin(&[(c1000[2], None, false), (z1, None, false), (z2, None, true)]);
            let hl = c.mulk(*lnn, 10 * h as i64);
            let drop = c.divk(hl, 24);
            let y = c.sub(ytop, drop);
            pts.push(see(&mut c, [x, y, z], s));
        }
        tent_pts.push(pts);
    }
    let longest = *lengths.iter().max().unwrap() as i64;
    let mut arm_pts = vec![];
    for (ca, sa, _, pts) in &arms {
        let mut out = vec![];
        for (i, &(wv, ra, _)) in pts.iter().enumerate() {
            let p = |c: &mut Calc, a: N, b: N| -> N { let l = c.lin(&[(a, Some(b), false)]); c.divk(l, 999) };
            let x1 = p(&mut c, ra, *ca);
            let x2 = p(&mut c, *sa, wv);
            let z1 = p(&mut c, ra, *sa);
            let z2 = p(&mut c, *ca, wv);
            let x = c.lin(&[(c1000[0], None, false), (x1, None, false), (x2, None, false)]);
            let z = c.lin(&[(c1000[2], None, false), (z1, None, false), (z2, None, true)]);
            let lg = c.num(longest);
            let l55 = c.mulk(lg, 55 * i as i64);
            let dropn = c.divk(l55, 2);
            let y = c.sub(c1000[1], dropn);
            out.push(see(&mut c, [x, y, z], s));
        }
        arm_pts.push(out);
    }
    eval_at(host, m, d, &mut c, 10);
    let screen = |c: &mut Calc, pts: &mut Vec<Seen>, shifts: Option<&Vec<(N, N, N)>>| -> Vec<Option<(N, N)>> {
        (0..pts.len()).map(|i| {
            if !seen(c, &pts[i]) { return None; }
            place(c, &mut pts[i]);
            let x = match shifts { Some(sh) => c.add(pts[i].x.unwrap(), sh[i].2), None => pts[i].x.unwrap() };
            Some((x, pts[i].y.unwrap()))
        }).collect()
    };
    let tent_xy: Vec<Vec<Option<(N, N)>>> = tent_pts.iter_mut().map(|p| screen(&mut c, p, None)).collect();
    let arm_xy: Vec<Vec<Option<(N, N)>>> = arm_pts.iter_mut().zip(&arms).map(|(p, a)| screen(&mut c, p, Some(&a.3))).collect();
    // each strand's pieces: first what leaves one out
    let mut strands: Vec<(Vec<Option<(Piece, N, N, N, N)>>, i64, Vec<i64>, i64, N, &Vec<Option<(N, N)>>)> = vec![];
    for (t, xy) in tents.iter().zip(&tent_xy) {
        let w = if t.0 { 75 } else { 32 };
        strands.push((pieces_first(&mut c, xy, sigma), w, d.taper[..24].iter().map(|&v| v as i64).collect(), 432, sigma, xy));
    }
    for (a, xy) in arms.iter().zip(&arm_xy) {
        let w = if a.2 == 0 { 23 } else { 16 };
        strands.push((pieces_first(&mut c, xy, arm_sigma), w, d.arm_taper[..20].iter().map(|&v| v as i64).collect(), 399, arm_sigma, xy));
    }
    eval_at(host, m, d, &mut c, 11);
    for (pieces, _, _, _, sig, _) in strands.iter_mut() { pieces_len(&mut c, pieces, *sig); }
    eval_at(host, m, d, &mut c, 12);
    // then each piece's glow and walk
    for (pieces, w, taper, kk, sig, xy) in strands.iter_mut() {
        for (h, pc) in pieces.iter_mut().enumerate() {
            let Some((piece, dx, dy, len, _)) = pc else { continue };
            if piece.skip.iter().any(|&f| flag(&c, f)) { *pc = None; continue; }
            let (p, q) = (xy[h].unwrap(), xy[h + 1].unwrap());
            let (dx, dy, len) = (*dx, *dy, *len);
            // not left out: each step at most 999 tenths of a pixel either way
            let (dx, dy) = (c.within(dx, 999), c.within(dy, 999));
            let len = c.within(len, 1_500);
            // the piece's strength: the strand's, by the taper along it, by the constant
            let wn = c.num(*w);
            let wt = c.mulk(wn, taper[h]);
            let nk = c.mulk(wt, *kk);
            let n100 = c.divk(nk, 100);
            let ls = c.mul(len, *sig);
            let ls = c.divk(ls, 1000);
            let ls = c.within(ls, 140_000);
            let one = c.num(1);
            let ls = c.max(ls, one);
            let ls = c.within(ls, 99_999);
            let f0 = c.div(n100, ls);
            let f1 = c.divk(f0, 10);
            let k299 = c.num(299);
            let f = c.min(f1, k299);
            piece.f = Some(f);
            *pc = Some((Piece { skip: vec![], f: Some(f), walk: None }, dx, dy, len, pc.as_ref().unwrap().4));
        }
    }
    eval_at(host, m, d, &mut c, 14);
    // and for each piece that glows, its walk
    let mut boxes: Vec<(usize, usize, [N; 8])> = vec![];
    for (si, (pieces, _, _, _, sig, xy)) in strands.iter_mut().enumerate() {
        for (h, pc) in pieces.iter_mut().enumerate() {
            let Some((piece, dx, dy, len, _)) = pc else { continue };
            if cardv(&c, piece.f.unwrap()) == 0 { *pc = None; continue; }
            let (p, q) = (xy[h].unwrap(), xy[h + 1].unwrap());
            let (dx, dy, len) = (*dx, *dy, *len);
            let tx = { let x = c.mulk(dx, 100_000); c.div(x, len) };
            let ty = { let x = c.mulk(dy, 100_000); c.div(x, len) };
            let (nx, ny) = (c.neg(ty), tx);
            let s3 = c.mulk(*sig, 3);
            let s3 = c.divk(s3, 1000);
            let reach = c.addk(s3, 1);
            let (hx, hy) = (c.mulk(dx, 50), c.mulk(dy, 50));
            let pa = c.sub(p.0, hx);
            let qb = c.add(q.0, hx);
            let (xa, xb) = (c.min(pa, qb), c.max(pa, qb));
            let pa = c.sub(p.1, hy);
            let qb = c.add(q.1, hy);
            let (ya, yb) = (c.min(pa, qb), c.max(pa, qb));
            let (xa, xb, ya, yb) = (c.shift(xa), c.shift(xb), c.shift(ya), c.shift(yb));
            let lo = c.sub(xa, reach);
            let x0 = c.clamp(lo, 0, 2000);
            let hi = c.add(xb, reach);
            let x1 = c.clamp(hi, -2000, HW as i64 - 1);
            let lo = c.sub(ya, reach);
            let y0 = c.clamp(lo, 0, 2000);
            let hi = c.add(yb, reach);
            let y1 = c.clamp(hi, -2000, HH as i64 - 1);
            boxes.push((si, h, [tx, ty, nx, ny, x0, x1, y0, y1]));
        }
    }
    eval_at(host, m, d, &mut c, 15);
    // the walks of those on the screen
    for &(si, h, [tx, ty, nx, ny, x0, x1, y0, y1]) in &boxes {
        if val(&c, x0) > val(&c, x1) || val(&c, y0) > val(&c, y1) { strands[si].0[h] = None; continue; }
        let (sig, xy) = (strands[si].4, strands[si].5);
        let p = xy[h].unwrap();
        let len = strands[si].0[h].as_ref().unwrap().3;
        let (half, k1000) = (c.num(500), c.num(1000));
        let ddx = c.lin(&[(x0, Some(k1000), false), (half, None, false), (p.0, None, true)]);
        let ddy = c.lin(&[(y0, Some(k1000), false), (half, None, false), (p.1, None, true)]);
        let (dx0, dy0) = (c.divk(ddx, 100), c.divk(ddy, 100));
        let (dx0, dy0) = (c.within(dx0, 99_999), c.within(dy0, 99_999));
        let sg = c.within(sig, 99_999);
        let (nx, ny, tx, ty) = (c.within(nx, 100_000), c.within(ny, 100_000), c.within(tx, 100_000), c.within(ty, 100_000));
        // the box is within a piece and a half and three widths of it: these are within 999 widths (thousandths)
        let across = c.lin_within(&[(dx0, Some(nx), false), (dy0, Some(ny), false)], Some(998_000_000));
        let a0 = c.div(across, sg);
        let a0 = c.within(a0, 200_000);
        let a0 = c.mulk(a0, 8);
        let ax = { let t = c.mulk(nx, 10); let q = c.div(t, sg); let q = c.within(q, 2_400); c.mulk(q, 8) };
        let ay = { let t = c.mulk(ny, 10); let q = c.div(t, sg); let q = c.within(q, 2_400); c.mulk(q, 8) };
        let along = c.lin_within(&[(dx0, Some(tx), false), (dy0, Some(ty), false)], Some(998_000_000));
        let l0 = c.div(along, len);
        // a piece at least as long as its width (at least 0.42 pixels): at most 238096 thousandths a pixel
        let lx = { let t = c.mulk(tx, 10); let q = c.div(t, len); c.within(q, 240_000) };
        let ly = { let t = c.mulk(ty, 10); let q = c.div(t, len); c.within(q, 240_000) };
        let k999 = c.num(999);
        let u9 = c.lin(&[(a0, None, false), (ax, Some(k999), false)]);
        let v9 = c.lin(&[(l0, None, false), (lx, Some(k999), false)]);
        strands[si].0[h].as_mut().unwrap().0.walk = Some(WalkN { x0, x1, y0, y1, u0: a0, v0: l0, u9, v9, st: [ax, ay, lx, ly] });
    }
    eval_at(host, m, d, &mut c, 13);
    // the strands' glow, piece after piece
    for (pieces, ..) in &strands {
        for pc in pieces.iter().flatten() {
            let (piece, ..) = pc;
            let (Some(f), Some(walk)) = (piece.f, &piece.walk) else { continue };
            let f = cardv(&c, f);
            if f == 0 { continue; }
            let mut px = vec![];
            walk_box(host, m, &c, walk, |_, y, row| {
                for (x, i, l) in row {
                    if !(-24..24).contains(&i) || !(-50..150).contains(&l) { continue; }
                    let g = d.strand[((if i >= 0 { i } else { -1 - i }) * 200 + l + 50) as usize];
                    if g > 0 { px.push((y as usize * HW + x as usize, g)); }
                }
            });
            if px.is_empty() { continue; }
            let deck: Vec<i32> = px.iter().flat_map(|&(i, g)| [emit.get(&i).map_or(0, |e| e.0), g]).collect();
            let out = run(host, &m.eadd, &[f], &deck, 2, 1);
            for (n, &(i, _)) in px.iter().enumerate() { let e = emit.entry(i).or_insert((0, 0)); e.0 = out[n]; }
        }
    }
    // the glow made a colour, mixed in by its cover
    let fade = fade_v;
    let triples: Vec<i32> = (0..=32).flat_map(|k| (0..3).flat_map(move |ch| [glow[ch], pale[ch], k])).collect();
    let cols = run(host, &m.mix, &[], &triples, 3, 1);
    let px: Vec<(usize, i32, i32)> = emit.iter().map(|(&i, &(e, edge))| (i, e, edge)).collect();
    if !px.is_empty() {
        let deck: Vec<i32> = px.iter().flat_map(|&(i, e, edge)| [depth(i % HW, i / HW), d.cover[e as usize], d.pale_glow[e as usize], d.pale_rim[edge as usize]]).collect();
        let out = run(host, &m.jelly, &[fade, ahead], &deck, 4, 2);
        let items: Vec<(usize, usize, [i32; 3], i32)> = px.iter().enumerate().filter(|(n, _)| out[2 * n] > 0)
            .map(|(n, &(i, _, _))| { let kp = out[2 * n + 1] as usize; (i % HW, i / HW, [cols[3 * kp], cols[3 * kp + 1], cols[3 * kp + 2]], out[2 * n]) }).collect();
        mix_in(host, m, cv, &items);
    }
    // the halo, a 4 x 4 at a time
    let (tx0, tx1, ty0, ty1) = (cardv(&c, tx0), cardv(&c, tx1), cardv(&c, ty0), cardv(&c, ty1));
    if tx0 > tx1 || ty0 > ty1 { return; }
    let (u0, v0, hs) = (long(&c, hu0), long(&c, hv0), long(&c, hstep));
    let rows = run(host, &m.rows, &[], &[ty1 - ty0, u0[0], u0[1], 0, 0, v0[0], v0[1], hs[0], hs[1]], 9, 4 * (ty1 - ty0 + 1) as usize);
    let items: Vec<i32> = rows.chunks(4).flat_map(|p| [tx1 - tx0, p[0], p[1], hs[0], hs[1], p[2], p[3], 0, 0]).collect();
    let uv = run(host, &m.walk, &[], &items, 9, 0);
    let tiles: Vec<(usize, usize)> = (ty0..=ty1).flat_map(|ty| (tx0..=tx1).map(move |tx| (tx as usize, ty as usize))).collect();
    assert_eq!(uv.len(), 2 * tiles.len());
    let ks = run(host, &m.gauss, &[], &uv, 2, 1);
    let lit: Vec<(usize, i32)> = tiles.iter().enumerate().filter(|(n, _)| ks[*n] >= 0 && d.halo[ks[*n] as usize] > 0).map(|(n, _)| (n, d.halo[ks[n] as usize])).collect();
    if lit.is_empty() { return; }
    let deck: Vec<i32> = lit.iter().flat_map(|&(n, h)| { let (tx, ty) = tiles[n]; [depth(TILE * tx, TILE * ty), h] }).collect();
    let amounts = run(host, &m.hamt, &[fade, ahead], &deck, 2, 1);
    // each tile's pixels (or its one colour), mixed in 256ths
    let mut items = vec![];
    for (e, &(n, _)) in lit.iter().enumerate() {
        if amounts[e] == 0 { continue; }
        let (tx, ty) = tiles[n];
        let t = ty * TW + tx;
        if cv.fine[t].is_some() { for k in 0..16 { items.push((t, Some(k), amounts[e])); } } else { items.push((t, None, amounts[e])); }
    }
    let get = |cv: &Canvas, t: usize, k: Option<usize>| -> [i32; 3] { match k { Some(k) => cv.fine[t].as_ref().unwrap()[k], None => cv.flat[t] } };
    let mut out = vec![[0; 3]; items.len()];
    for ch in 0..3 {
        let deck: Vec<i32> = items.iter().flat_map(|&(t, k, a)| [get(cv, t, k)[ch], glow[ch], a]).collect();
        for (e, v) in run(host, &m.mix256, &[], &deck, 3, 1).into_iter().enumerate() { out[e][ch] = v; }
    }
    for (e, &(t, k, _)) in items.iter().enumerate() {
        cv.touched[t] = true;
        match k { Some(k) => cv.fine[t].as_mut().unwrap()[k] = out[e], None => cv.flat[t] = out[e] }
    }
}

/// The caption, the portal and the whiteout over every tile something was drawn in (the rest are the dive's own
/// finished picture): each pixel (or the tile's one colour) as its pixel of the dive is.
fn finish(host: &mut Host, m: &Machines, s: &Scene, cv: &mut Canvas) {
    // what to finish: (tile, its pixel or the whole tile), and the dive's pixel under it
    let mut items: Vec<(usize, Option<usize>, usize)> = vec![];
    for t in 0..TW * TH {
        if !cv.touched[t] { continue; }
        let (tx, ty) = (t % TW, t / TW);
        let low = (ty * TILE / 8) * 192 + tx * TILE / 8;
        if cv.fine[t].is_some() { for k in 0..16 { items.push((t, Some(k), low)); } } else { items.push((t, None, low)); }
    }
    let get = |cv: &Canvas, t: usize, k: Option<usize>| -> [i32; 3] { match k { Some(k) => cv.fine[t].as_ref().unwrap()[k], None => cv.flat[t] } };
    let set = |cv: &mut Canvas, t: usize, k: Option<usize>, c: [i32; 3]| { match k { Some(k) => cv.fine[t].as_mut().unwrap()[k] = c, None => cv.flat[t] = c } };
    // a stage: each item's mix (umix, or the spill's in 256ths), if it has one
    let stage = |host: &mut Host, cv: &mut Canvas, op: &dyn Fn(usize) -> Option<(bool, [i32; 3], i32)>| {
        let todo: Vec<(usize, (bool, [i32; 3], i32))> = items.iter().enumerate().filter_map(|(e, it)| op(it.2).map(|o| (e, o))).collect();
        for fine in [false, true] {
            let part: Vec<&(usize, (bool, [i32; 3], i32))> = todo.iter().filter(|(_, o)| o.0 == fine).collect();
            if part.is_empty() { continue; }
            let mut out = vec![[0; 3]; part.len()];
            for ch in 0..3 {
                let deck: Vec<i32> = part.iter().flat_map(|&&(e, (_, to, k))| [get(cv, items[e].0, items[e].1)[ch], to[ch], k]).collect();
                let v = if fine { run(host, &m.spill, &[s.on], &deck, 3, 1) } else { run(host, &m.mix, &[], &deck, 3, 1) };
                for (n, x) in v.into_iter().enumerate() { out[n][ch] = x; }
            }
            for (n, &&(e, _)) in part.iter().enumerate() { set(cv, items[e].0, items[e].1, out[n]); }
        }
    };
    if s.caption >= 0 {
        stage(host, cv, &|low: usize| { let (x, y) = (low % 192, low / 192); (y < 7 && x < 69).then(|| (false, if s.dots[y][x] == 1 { [255; 3] } else { [0; 3] }, 2 * s.caption)) });
    }
    for j in 0..3 {
        stage(host, cv, &|low: usize| s.portal.get(&low).and_then(|ops| ops.get(j).copied()));
    }
    if s.white32 > 0 { stage(host, cv, &|_| Some((false, [255; 3], s.white32))); }
}

// --- the orcas ---------------------------------------------------------------------------------------------------

/// The orcas' frames, their length (model units, nose to flukes), the body's colour, their heading and side
/// (thousandths) and their lengths in units (the leader's, the second's), as reference_orca.rs.
const ORCA_FIRST: i32 = 84;
const ORCA_LAST: i32 = 156;
const ORCA_LENGTH: i64 = 860;
const ORCA_DARK: [i32; 3] = [2, 4, 7];
const ORCA_HEAD: [i64; 3] = [662, 0, 749];
const ORCA_SIDE: [i64; 3] = [749, 0, -662];

/// The pair (frames 84..156): far out and low in the canyon's haze, gliding left to right, each drawn by the mesh
/// kit flat and unlit (each pixel's depth), hidden where the dive's seabed or kelp is nearer, its colour the dark body
/// mixed toward the water's far colour by the haze at its distance; the further first. Every number of where they are
/// is the swarm's sums from the frame's number and the camera's.
fn orcas(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas, depth: &dyn Fn(usize, usize) -> i32) {
    if !(ORCA_FIRST..=ORCA_LAST).contains(&s.n) { return; }
    let mut c = Calc::new();
    let n = c.num(s.n as i64);
    let k = c.addk(n, -(ORCA_FIRST as i64));
    let k = c.within(k, 72);
    // the leader's place from the camera (hundredths): across from 38 left over the 72 frames, 19 below, 66 ahead
    let across = { let x = c.mulk(k, 7000); let q = c.divk(x, 72); let q = c.within(q, 7000); let camx = c.num(s.cam[0] as i64); let q = c.addk(q, -3800); c.sub(q, camx) };
    let rel1 = [across, c.num(-1900), c.num(6600)];
    // the second: behind along the heading, to the far side, a little lower, each by a slow sine
    let sine = |c: &mut Calc, per: i64, phase: i64, amp: i64| -> N {
        let x = c.mulk(k, per);
        let a = c.divk(x, 72);
        let a = c.addk(a, phase);
        let (sn, _) = c.sin(a);
        let y = c.mulk(sn, amp);
        let q = c.divk(y, 999);
        c.within(q, amp)
    };
    let g1 = { let v = sine(&mut c, 800_000, 0, 500); c.addk(v, 1400) };
    let g2 = { let v = sine(&mut c, 600_000, 159_155, 400); c.addk(v, 700) };
    let g3 = { let v = sine(&mut c, 500_000, 0, 150); c.addk(v, -300) };
    let rel2: [N; 3] = std::array::from_fn(|i| {
        let h = { let x = c.mulk(g1, ORCA_HEAD[i]); let q = c.divk(x, 1000); c.within(q, 2000) };
        let sd = { let x = c.mulk(g2, ORCA_SIDE[i]); let q = c.divk(x, 1000); c.within(q, 2000) };
        let mut terms = vec![(rel1[i], None, false), (h, None, true), (sd, None, true)];
        if i == 1 { terms.push((g3, None, false)); }
        c.lin(&terms)
    });
    // the camera's right, up and forward (hundredths); the model's axes in the world (its x the side, y up, z back
    // along the heading)
    let cmat: [[N; 3]; 3] = [s.r, s.u, s.f].map(|v| v.map(|x| c.num(x as i64)));
    let axes = [ORCA_SIDE, [0, 1000, 0], ORCA_HEAD.map(|v| -v)];
    let rot: [[N; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| {
        let terms: Vec<(N, Option<N>, bool)> = (0..3).filter(|&q| axes[j][q] != 0).map(|q| { let a = c.num(axes[j][q]); (cmat[i][q], Some(a), false) }).collect();
        let l = c.lin(&terms);
        let r = c.divk(l, 100);
        c.within(r, 1000)
    }));
    let place = |c: &mut Calc, rel: [N; 3], l: i64| -> [N; 3] {
        std::array::from_fn(|i| {
            let q = c.lin(&[(cmat[i][0], Some(rel[0]), false), (cmat[i][1], Some(rel[1]), false), (cmat[i][2], Some(rel[2]), false)]);
            let q = c.divk(q, 10);
            let q = c.mulk(q, ORCA_LENGTH);
            let p = c.divk(q, 1000 * l);
            c.within(p, 5000)
        })
    };
    let p1 = place(&mut c, rel1, 46);
    let p2 = place(&mut c, rel2, 37);
    let n64 = c.mulk(n, 64);
    let ph = c.divk(n64, 100);
    let mut both = vec![];
    for (pos, l, off) in [(p1, 46i64, 0i64), (p2, 37, 7)] {
        let zref = c.addk(pos[2], -500);
        let o = c.addk(ph, off);
        let phase = c.modk(o, 12);
        let zl = c.mulk(zref, l);
        let a0 = c.divk(zl, ORCA_LENGTH);
        let r0 = c.modk(zl, ORCA_LENGTH);
        let far = c.mulk(pos[2], l);
        both.push((pos, l, zref, phase, a0, r0, far));
    }
    let further = c.lt(both[0].6, both[1].6);
    eval(host, m, d, &mut c);
    if flag(&c, further) { both.swap(0, 1); }
    let (zero, k80) = (c.num(0), 80);
    for (pos, l, zref, phase, a0, r0, _) in both {
        let model = &d.orcas[val(&c, phase) as usize];
        let view = super::mesh::View { focal10: k80, zref, light: [zero; 3], half: [zero; 3] };
        let mat = super::mesh::Material { ambient: [0; 3], diffuse: [0; 3], highlight: [0; 3] };
        let (_, px) = m.kit.pixels(host, &mut c, model, &super::mesh::Place { rot, pos }, &view, &mat, super::mesh::Shading::Depth, HW, HH);
        if px.is_empty() { continue; }
        // how far ahead each depth is (whole units), worked out down the depths; each pixel's from it by its depth
        let table = run(host, &m.ahead, &[cardv(&c, a0), cardv(&c, r0), l as i32], &[0; 1000], 1, 1);
        let items: Vec<(usize, usize, i32)> = px.iter().map(|&(at, v)| ((at as usize) % HW, (at as usize) / HW, table[v[0] as usize])).collect();
        let keep = run(host, &m.keep, &[], &items.iter().flat_map(|&(x, y, a)| [a, depth(x, y)]).collect::<Vec<_>>(), 2, 1);
        // the colour at each distance shown: the body mixed toward the far colour by the haze there
        let shown: Vec<&(usize, usize, i32)> = items.iter().zip(&keep).filter(|(_, &k)| k == 1).map(|(i, _)| i).collect();
        let mut dists: Vec<i32> = shown.iter().map(|i| i.2).collect();
        dists.sort();
        dists.dedup();
        let deck: Vec<i32> = dists.iter().flat_map(|&a| (0..3).flat_map(move |ch| [ORCA_DARK[ch], s.far[ch], d.orca_haze[a as usize]])).collect();
        let cols = run(host, &m.mix, &[], &deck, 3, 1);
        for &&(x, y, a) in &shown {
            let e = dists.binary_search(&a).unwrap();
            cv.set(x, y, [cols[3 * e], cols[3 * e + 1], cols[3 * e + 2]]);
        }
    }
}

// --- the ending: the knob ------------------------------------------------------------------------------------------

/// The ending's frames, the click, and the dial's middle (hundredths) and frame (thousandths), as reference_knob.rs.
const END_FIRST: i32 = 228;
const END_LAST: i32 = 352;
const CLICK: i32 = 331;
const KNOB: [i64; 3] = [-166, -10922, 32190];
const K_SIDE: [i64; 3] = [992, 0, 128];
const K_UP: [i64; 3] = [0, 1000, 0];
const K_FWD: [i64; 3] = [-128, 0, 992];
const INK: [i64; 3] = [255, 225, 160];
const PAINT: [i64; 3] = [58, 55, 45];
const WARM: [i32; 3] = [255, 210, 140];
const SILT: [i32; 3] = [92, 86, 72];
const PALE: [i32; 3] = [110, 104, 92];
const CRUST: [i32; 3] = [58, 52, 40];
const BLUE: [i32; 3] = [170, 210, 255];
const FLASH: [i32; 3] = [225, 240, 255];
const ORCA2_AXES: [[i64; 3]; 3] = [[-9, 0, 1000], [0, 1000, 0], [1000, 0, 9]];
const ORCA2_FIRST: i32 = 282;
const ORCA2_LAST: i32 = 328;

/// The knob's models (make_knob.py: each corner's place in the atlas where its normal would be), the atlas and the
/// tables (knob.txt).
pub struct KnobData {
    rock: super::mesh::Model, plate: super::mesh::Model, knob: super::mesh::Model,
    atlas: Vec<u8>, pages: Vec<(i32, i32, usize, i32, i32)>, turn: BTreeMap<i32, (i32, i32)>,
    haze120: Vec<i32>, lift: [Vec<i32>; 3], halo: Vec<i32>, puff: Vec<i32>, glow: [Vec<i32>; 3],
    silt: Vec<i32>, debris: Vec<i32>, shake: Vec<i32>, crackle: Vec<i32>,
}

impl KnobData {
    fn load(demo: &Path) -> KnobData {
        let text = read(demo, "deep/data/knob.txt");
        let get = |name: &str| -> Vec<i32> { text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap_or_else(|| panic!("knob.txt: no {name}")).split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() };
        let p = get("pages");
        let t = get("turn");
        KnobData { rock: super::mesh::Model::read(demo, "deep/data/knob_rock.txt"), plate: super::mesh::Model::read(demo, "deep/data/knob_plate.txt"),
            knob: super::mesh::Model::read(demo, "deep/data/knob_knob.txt"),
            atlas: std::fs::read(demo.join("deep/data/knob.bin")).unwrap_or_else(|e| panic!("deep/data/knob.bin: {e} (make it with deep/data/make_knob.py)")),
            pages: p.chunks(5).map(|c| (c[0], c[1], c[2] as usize, c[3], c[4])).collect(), turn: t.chunks(3).map(|c| (c[0], (c[1], c[2]))).collect(),
            haze120: get("haze120"), lift: [get("lift0"), get("lift1"), get("lift2")], halo: get("halo"), puff: get("puff"),
            glow: [get("glow2"), get("glow4"), get("glow5")], silt: get("silt"), debris: get("debris"), shake: get("shake"), crackle: get("crackle") }
    }
    /// The texel the kit's walks punched (page, across and down from the page's origin): S, C, B, ink, whose.
    fn texel(&self, page: i32, u: i32, v: i32) -> [i32; 11] {
        let (w, h, at, ou, ov) = self.pages[page as usize];
        let (u, v) = (u + ou, v + ov);
        assert!((0..w).contains(&u) && (0..h).contains(&v), "a texel off its page");
        let o = at + ((v * w + u) * 11) as usize;
        std::array::from_fn(|k| self.atlas[o + k] as i32)
    }
}

/// A place seen from the camera, as the reference's proj: how far ahead (hundredths), across and up, the flags that
/// leave it out, and once seen its place on the screen (thousandths of a pixel).
struct Pn { a: N, bx: N, cy: N, off: [Arg; 3], x: Option<N>, y: Option<N> }
fn pn(c: &mut Calc, rel: [N; 3], cm: &[[N; 3]; 3]) -> Pn {
    let dot = |c: &mut Calc, v: [N; 3]| { let l = c.lin(&[(rel[0], Some(v[0]), false), (rel[1], Some(v[1]), false), (rel[2], Some(v[2]), false)]); c.divk(l, 300) };
    let (bx, cy, a) = (dot(c, cm[0]), dot(c, cm[1]), dot(c, cm[2]));
    let hundred = c.num(100);
    let behind = c.lt(a, hundred);
    let six = c.mulk(a, 6);
    let (ax, ay) = (c.abs(bx), c.abs(cy));
    let (fx, fy) = (c.mulk(ax, 5), c.mulk(ay, 5));
    let wide = c.lt(six, fx);
    let tall = c.lt(six, fy);
    Pn { a, bx, cy, off: [behind, wide, tall], x: None, y: None }
}
fn pn_seen(c: &Calc, p: &Pn) -> bool { !p.off.iter().any(|&f| flag(c, f)) }
fn pn_place(c: &mut Calc, p: &mut Pn) {
    let a = c.within(p.a, 99_999);
    let bx = c.within(p.bx, 120_000);
    let cy = c.within(p.cy, 120_000);
    let nx = c.mulk(bx, 800);
    let qx = c.divrem(nx, a, 1).0;
    p.x = Some(c.addk(qx, 768_000));
    let ny = c.mulk(cy, 800);
    let qy = c.divrem(ny, a, 1).0;
    let mid = c.num(384_000);
    p.y = Some(c.sub(mid, qy));
}

/// A place from the dial's middle (hundredths, numbers) less the camera's: from the camera.
fn from_cam(c: &mut Calc, q: [N; 3], cam: [N; 3]) -> [N; 3] {
    std::array::from_fn(|i| { let k = c.num(KNOB[i]); c.lin(&[(k, None, false), (q[i], None, false), (cam[i], None, true)]) })
}

/// Mix every pixel of these 4 x 4 pixels toward a colour (umix in 32nds, or in 256ths): each tile once, its one colour
/// or its 16.
fn mix_tiles(host: &mut Host, m: &Machines, cv: &mut Canvas, items: &[(usize, [i32; 3], i32)], fine256: bool) {
    let mut deck: Vec<(usize, Option<usize>, [i32; 3], i32)> = vec![];
    for &(t, to, k) in items {
        if cv.fine[t].is_some() { for e in 0..16 { deck.push((t, Some(e), to, k)); } } else { deck.push((t, None, to, k)); }
    }
    let get = |cv: &Canvas, t: usize, e: Option<usize>| -> [i32; 3] { match e { Some(e) => cv.fine[t].as_ref().unwrap()[e], None => cv.flat[t] } };
    let mut out = vec![[0; 3]; deck.len()];
    for ch in 0..3 {
        let flat: Vec<i32> = deck.iter().flat_map(|&(t, e, to, k)| [get(cv, t, e)[ch], to[ch], k]).collect();
        let v = run(host, if fine256 { &m.kmix256 } else { &m.mix }, &[], &flat, 3, 1);
        for (i, x) in v.into_iter().enumerate() { out[i][ch] = x; }
    }
    for (i, &(t, e, _, _)) in deck.iter().enumerate() {
        cv.touched[t] = true;
        match e { Some(e) => cv.fine[t].as_mut().unwrap()[e] = out[i], None => cv.flat[t] = out[i] }
    }
}

/// The frame's numbers (the reference's Ctl): the value, the shudder, the power (19ths), the glow (hundredths), how
/// far the scale has lit (76ths).
struct Ctl { v100: N, w: i32, p19: N, g100: N, mm: N }
fn ctl(c: &mut Calc, n: i32, kd: &KnobData) -> Ctl {
    let (v100, w) = if (308..=330).contains(&n) { kd.turn[&n] } else if n < 308 { (900, 0) } else { (1000, 0) };
    let v100 = c.num(v100 as i64);
    let nn = c.num(n as i64);
    let k = c.addk(nn, -(CLICK as i64));
    let p19 = c.clamp(k, 0, 19);
    let t = c.mulk(v100, 45);
    let t = c.divk(t, 100);
    let glow1000 = c.addk(t, 550);
    let f = c.mulk(p19, 16);
    let f = c.addk(f, 190);
    let g = c.mul(glow1000, f);
    let g100 = c.divk(g, 1900);
    let g100 = c.within(g100, 300);
    let five = c.mulk(p19, 5);
    let s76 = c.num(76);
    let mm = c.min(five, s76);
    Ctl { v100, w, p19, g100, mm }
}

/// The ending over the dive's finished picture (frames 228..352), as reference_knob.rs.
fn ending(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas) {
    let n = s.n;
    if !(END_FIRST..=END_LAST).contains(&n) { return; }
    let kd = &d.knob;
    let raw = |x: usize, y: usize| s.depth[(y / 8) * 192 + x / 8];
    // the water's light beyond the dial, every 4 x 4 pixels
    if n >= 263 {
        let mut c = Calc::new();
        // (the frames since 262, at most 30, on the calculator)
        let tt = { let nn = c.num(n as i64); let t = c.addk(nn, -262); let thirty = c.num(30); let t = c.min(t, thirty); c.within(t, 30) };
        let sq = c.mul(tt, tt);
        let two = c.mulk(tt, -2);
        let w = c.addk(two, 90);
        let p = c.mul(sq, w);
        let p = c.mulk(p, 32);
        let k32 = c.divk(p, 27000);
        eval(host, m, d, &mut c);
        let k32 = cardv(&c, k32);
        let mut items: Vec<(usize, Option<usize>, usize)> = vec![];
        for t in 0..TW * TH {
            let (tx, ty) = (t % TW, t / TW);
            if cv.fine[t].is_some() { for e in 0..16 { items.push((t, Some(e), ty)); } } else { items.push((t, None, ty)); }
            let _ = tx;
        }
        let get = |cv: &Canvas, t: usize, e: Option<usize>| -> [i32; 3] { match e { Some(e) => cv.fine[t].as_ref().unwrap()[e], None => cv.flat[t] } };
        let mut out = vec![[0; 3]; items.len()];
        for ch in 0..3 {
            let deck: Vec<i32> = items.iter().flat_map(|&(t, e, ty)| [get(cv, t, e)[ch], kd.lift[ch][ty], raw((t % TW) * TILE, (t / TW) * TILE)]).collect();
            for (i, v) in run(host, &m.klift, &[k32], &deck, 3, 1).into_iter().enumerate() { out[i][ch] = v; }
        }
        for (i, &(t, e, _)) in items.iter().enumerate() {
            cv.touched[t] = true;
            match e { Some(e) => cv.fine[t].as_mut().unwrap()[e] = out[i], None => cv.flat[t] = out[i] }
        }
    }
    let cost = std::env::var("KNOB_COST").is_ok();
    let mut at = host.steps;
    let mut tell = |host: &Host, what: &str| { if cost { eprintln!("{n} {what}: {}M", (host.steps - at) / 1_000_000); } at = host.steps; };
    tell(host, "lift");
    if (ORCA2_FIRST..=ORCA2_LAST).contains(&n) { orca2(host, m, d, s, cv); }
    tell(host, "orca");
    // the knob's pixels first (their colours don't depend on what is under them), so the dial's glow on the water
    // is mixed only into the 4 x 4 pixels the knob doesn't wholly cover; then the knob over it
    let kp = knob(host, m, d, s);
    tell(host, "knob");
    let mut covered = vec![0u8; TW * TH];
    let mut seen = vec![false; HW * HH];
    for &(x, y, _) in &kp { if !seen[y * HW + x] { seen[y * HW + x] = true; covered[(y / TILE) * TW + x / TILE] += 1; } }
    halo(host, m, d, s, cv, &covered);
    tell(host, "halo");
    for (x, y, c3) in kp { cv.set(x, y, c3); }
    silt(host, m, d, s, cv);
    tell(host, "silt");
    if n > CLICK { debris(host, m, d, s, cv); }
    tell(host, "debris");
    if n >= CLICK { crackle(host, m, d, s, cv); }
    tell(host, "crackle");
}

/// The camera's vectors (300ths) and place (hundredths) as numbers.
fn cam_nums(c: &mut Calc, s: &Scene) -> ([[N; 3]; 3], [N; 3]) {
    let cm = [s.r3, s.u3, s.f3].map(|v| v.map(|x| c.num(x as i64)));
    let cam = s.cam.map(|x| c.num(x as i64));
    (cm, cam)
}

/// A model's turn into the camera's space: fl(camera's vectors times its axes, 300), thousandths.
fn k_turn(c: &mut Calc, cm: &[[N; 3]; 3], axes: [[i64; 3]; 3]) -> [[N; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| {
        let terms: Vec<(N, Option<N>, bool)> = (0..3).filter(|&q| axes[j][q] != 0).map(|q| { let a = c.num(axes[j][q]); (cm[i][q], Some(a), false) }).collect();
        let l = c.lin(&terms);
        let r = c.divk(l, 300);
        c.within(r, 1000)
    }))
}

fn orca2(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas) {
    let n = s.n;
    let mut c = Calc::new();
    let (cm, cam) = cam_nums(&mut c, s);
    let nn = c.num(n as i64);
    let k = c.addk(nn, -(ORCA2_FIRST as i64));
    let k = c.within(k, 46);
    let ang = { let x = c.mulk(k, 500_000); c.divk(x, 46) };
    let (sn, _) = sin_cos(&mut c, ang);
    let across = { let x = c.mulk(k, 14000); let q = c.divk(x, 46); let q = c.within(q, 14000); let s7 = c.num(7000); c.sub(s7, q) };
    let rel: [N; 3] = std::array::from_fn(|i| {
        let base = { let f = c.num(K_FWD[i] * 5500); let f = c.divk(f, 1000); c.addk(f, KNOB[i]) };
        let sd = { let x = c.mulk(across, K_SIDE[i]); let q = c.divk(x, 1000); c.within(q, 7000) };
        let mut terms = vec![(base, None, false), (sd, None, false), (cam[i], None, true)];
        if i == 1 { let x = c.mulk(sn, 200); let q = c.divk(x, 999); let q = c.within(q, 200); let dn = c.addk(q, -2100); terms.push((dn, None, false)); }
        c.lin(&terms)
    });
    let rot = k_turn(&mut c, &cm, ORCA2_AXES);
    let pos: [N; 3] = std::array::from_fn(|i| {
        let q = c.lin(&[(cm[i][0], Some(rel[0]), false), (cm[i][1], Some(rel[1]), false), (cm[i][2], Some(rel[2]), false)]);
        let q = c.divk(q, 300);
        let q = c.mulk(q, 860);
        let p = c.divk(q, 4600);
        c.within(p, 5000)
    });
    let zref = c.addk(pos[2], -500);
    let n64 = c.mulk(nn, 64);
    let ph = c.divk(n64, 100);
    let phase = c.modk(ph, 12);
    let zl = c.mulk(zref, 46);
    let a0 = c.divk(zl, 860);
    let r0 = c.modk(zl, 860);
    eval(host, m, d, &mut c);
    let model = &d.orcas[val(&c, phase) as usize];
    let zero = c.num(0);
    let view = super::mesh::View { focal10: 80, zref, light: [zero; 3], half: [zero; 3] };
    let mat = super::mesh::Material { ambient: [0; 3], diffuse: [0; 3], highlight: [0; 3] };
    let (_, px) = m.kit.pixels(host, &mut c, model, &super::mesh::Place { rot, pos }, &view, &mat, super::mesh::Shading::Depth, HW, HH);
    if px.is_empty() { return; }
    let table = run(host, &m.ahead, &[cardv(&c, a0), cardv(&c, r0), 46], &[0; 1000], 1, 1);
    let items: Vec<(usize, usize, i32)> = px.iter().map(|&(at, v)| ((at as usize) % HW, (at as usize) / HW, table[v[0] as usize])).collect();
    let keep = run(host, &m.keep, &[], &items.iter().flat_map(|&(x, y, a)| [a, s.depth[(y / 8) * 192 + x / 8]]).collect::<Vec<_>>(), 2, 1);
    let shown: Vec<&(usize, usize, i32)> = items.iter().zip(&keep).filter(|(_, &k)| k == 1).map(|(i, _)| i).collect();
    let mut out = vec![[0; 3]; shown.len()];
    for ch in 0..3 {
        let deck: Vec<i32> = shown.iter().flat_map(|&&(x, y, a)| [ORCA_DARK[ch], cv.get(x, y)[ch], d.knob.haze120[a.min(199) as usize]]).collect();
        for (i, v) in run(host, &m.mix, &[], &deck, 3, 1).into_iter().enumerate() { out[i][ch] = v; }
    }
    for (i, &&(x, y, _)) in shown.iter().enumerate() { cv.set(x, y, out[i]); }
}

fn halo(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas, covered: &[u8]) {
    let kd = &d.knob;
    let mut c = Calc::new();
    let (cm, cam) = cam_nums(&mut c, s);
    let ct = ctl(&mut c, s.n, kd);
    let zero = c.num(0);
    let rel = from_cam(&mut c, [zero; 3], cam);
    let mut p = pn(&mut c, rel, &cm);
    eval(host, m, d, &mut c);
    if !pn_seen(&c, &p) { return; }
    pn_place(&mut c, &mut p);
    let a = c.within(p.a, 99_999);
    let big = c.num(588_800);
    let rad = c.div(big, a);
    let rad = c.within(rad, 1000);
    let r2 = c.mul(rad, rad);
    let r2 = c.mulk(r2, 2);
    let f = c.mulk(ct.p19, 16);
    let f = c.addk(f, 190);
    let r2 = c.mul(r2, f);
    let r2 = c.divk(r2, 1000);
    let r2 = c.divk(r2, 190);
    let one = c.num(1);
    let r2 = c.max(r2, one);
    let r2 = c.within(r2, 99_999);
    let ccx = { let x = p.x.unwrap(); let q = c.divk(x, 1000); c.within(q, 2000) };
    let ccy = { let y = p.y.unwrap(); let q = c.divk(y, 1000); c.within(q, 2000) };
    let hundred = c.num(100);
    let ex: Vec<N> = (0..TW as i64).map(|tx| {
        let m4 = c.num(4 * tx + 2);
        let dx = c.sub(m4, ccx);
        let sq = c.mul(dx, dx);
        let sq = c.divk(sq, 100);
        let e = c.div(sq, r2);
        c.min(e, hundred)
    }).collect();
    let ey: Vec<N> = (0..TH as i64).map(|ty| {
        let m4 = c.num(4 * ty + 2);
        let dy = c.sub(m4, ccy);
        let sq = c.mul(dy, dy);
        let sq = c.mulk(sq, 22);
        let sq = c.divk(sq, 1000);
        let e = c.div(sq, r2);
        c.min(e, hundred)
    }).collect();
    let amt: Vec<N> = kd.halo.iter().map(|&h| { let x = c.mulk(ct.g100, h as i64); c.divk(x, 100) }).collect();
    eval(host, m, d, &mut c);
    let ex: Vec<i32> = ex.iter().map(|&e| cardv(&c, e)).collect();
    let ey: Vec<i32> = ey.iter().map(|&e| cardv(&c, e)).collect();
    let amt: Vec<i32> = amt.iter().map(|&a| cardv(&c, a)).collect();
    let deck: Vec<i32> = (0..TW * TH).flat_map(|t| [ex[t % TW], ey[t / TW]]).collect();
    let idx = run(host, &m.kindex, &[99], &deck, 2, 1);
    let items: Vec<(usize, [i32; 3], i32)> = idx.iter().enumerate().filter(|&(t, &e)| e >= 0 && amt[e as usize] > 0 && covered[t] < 16).map(|(t, &e)| (t, WARM, amt[e as usize])).collect();
    mix_tiles(host, m, cv, &items, true);
}

/// The knob's pixels in the order they are laid (each model over the last): place and colour.
fn knob(host: &mut Host, m: &Machines, d: &Data, s: &Scene) -> Vec<(usize, usize, [i32; 3])> {
    let mut laid = vec![];
    let n = s.n;
    let kd = &d.knob;
    let mut c = Calc::new();
    let (cm, cam) = cam_nums(&mut c, s);
    let ct = ctl(&mut c, n, kd);
    let zero = c.num(0);
    let rel = from_cam(&mut c, [zero; 3], cam);
    let pc: [N; 3] = std::array::from_fn(|i| { let q = c.lin(&[(cm[i][0], Some(rel[0]), false), (cm[i][1], Some(rel[1]), false), (cm[i][2], Some(rel[2]), false)]); let q = c.divk(q, 300); c.within(q, 9_800) });
    let rot = k_turn(&mut c, &cm, [K_SIDE, K_UP, K_FWD]);
    // the tremble and the shudder (units across and along, the dial's frame)
    let (sx, sz) = if n > CLICK {
        let a10 = { let x = c.mulk(ct.p19, 50); let q = c.divk(x, 19); c.addk(q, 12) };
        let (r1, r2) = (kd.shake[(2 * (n - 332)) as usize] as i64, kd.shake[(2 * (n - 332) + 1) as usize] as i64);
        let x = c.mulk(a10, 2 * r1 - 999);
        let sx = c.divk(x, 9990);
        let z = c.mulk(a10, 6 * (2 * r2 - 999));
        let sz = c.divk(z, 99_900);
        let (sx, sz) = (c.within(sx, 10), c.within(sz, 10));
        (c.neg(sx), c.neg(sz))
    } else { (zero, zero) };
    let w = c.num(ct.w as i64);
    let kx = c.add(w, sx);
    let off = |c: &mut Calc, o: [N; 3]| -> [N; 3] { std::array::from_fn(|i| { let q = c.lin(&[(rot[i][0], Some(o[0]), false), (rot[i][2], Some(o[2]), false)]); let q = c.divk(q, 1000); c.within(q, 100) }) };
    let po = off(&mut c, [sx, zero, sz]);
    let ko = off(&mut c, [kx, zero, sz]);
    let rock_pos: [N; 3] = std::array::from_fn(|i| { let x = c.mulk(pc[i], 30); let q = c.divk(x, 100); c.within(q, 6000) });
    let plate_pos: [N; 3] = std::array::from_fn(|i| c.add(pc[i], po[i]));
    let knob_pos: [N; 3] = std::array::from_fn(|i| c.add(pc[i], ko[i]));
    let delta = { let x = c.addk(ct.v100, -900); let x = c.mulk(x, 250_000); c.divk(x, 300) };
    let (sn, cs) = sin_cos(&mut c, delta);
    let nsn = c.neg(sn);
    let k999 = c.num(999);
    let ry = [[Some(cs), None, Some(sn)], [None, Some(k999), None], [Some(nsn), None, Some(cs)]];
    let rot_k: [[N; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| {
        let terms: Vec<(N, Option<N>, bool)> = (0..3).filter_map(|q| ry[q][j].map(|e| (rot[i][q], Some(e), false))).collect();
        let l = c.lin(&terms);
        let r = c.divk(l, 999);
        c.within(r, 1000)
    }));
    // from the click: the glow's gain over 9, each ink's colour and how its light multiplies it
    let post = n > CLICK;
    let (mut dg, mut xs, mut times) = (None, vec![], vec![]);
    if post {
        let two = c.num(200);
        let g = c.min(ct.g100, two);
        let g = c.addk(g, -95);
        let g = c.mulk(g, 32);
        dg = Some(c.divk(g, 100));
        let fade = { let x = c.mulk(ct.mm, 65); let q = c.divk(x, 76); let q = c.within(q, 65); c.addk(q, 35) };
        let f11 = { let x = c.mulk(ct.mm, 60); let q = c.divk(x, 76); let q = c.within(q, 60); c.addk(q, 40) };
        let k255 = c.num(255);
        let (k0, k10, k16) = (c.num(0), c.num(10), c.num(16));
        let k100 = c.num(100);
        for cls in 0..11i64 {
            let x = c.mulk(ct.mm, 11);
            let x = c.addk(x, -38 - 76 * cls);
            let x = c.mulk(x, 10);
            let l10 = c.divk(x, 76);
            let l10 = c.max(l10, k0);
            let l10 = c.min(l10, k10);
            let t14 = { let x = c.mulk(l10, 14); let q = c.divk(x, 10); c.within(q, 14) };
            let tm = c.addk(t14, 10);
            times.push(tm);
            let gg10 = c.min(tm, k16);
            let l14 = { let x = c.mulk(l10, 14); c.min(x, k100) };
            let x: [N; 3] = std::array::from_fn(|ch| {
                let pb = { let x = c.mulk(l14, INK[ch]); let q = c.divk(x, 200); let q = c.within(q, 128); c.addk(q, PAINT[ch]) };
                let nf = { let x = c.neg(fade); c.addk(x, 100) };
                let a1 = c.mul(pb, nf);
                let a2 = { let x = c.mul(ct.g100, fade); let x = c.mulk(x, 8 * INK[ch]); let q = c.divk(x, 1000); c.within(q, 61_200) };
                let q100 = c.add(a1, a2);
                let tq = { let x = c.mul(q100, gg10); let q = c.divk(x, 10); c.within(q, 90_000) };
                let mlt = { let x = c.mulk(fade, 55); c.addk(x, 4500) };
                let y = c.mul(tq, mlt);
                let y = c.divk(y, 1000);
                let y = c.divk(y, 700);
                c.min(y, k255)
            });
            xs.push(x);
        }
        let x11: [N; 3] = std::array::from_fn(|ch| { let x = c.mul(ct.g100, f11); let x = c.mulk(x, INK[ch]); let q = c.divk(x, 10000); c.min(q, k255) });
        xs.push(x11);
    }
    let models: [(&super::mesh::Model, [[N; 3]; 3], [N; 3], i64, i64); 3] = [(&kd.rock, rot, rock_pos, -300, 30), (&kd.plate, rot, plate_pos, -470, 100), (&kd.knob, rot_k, knob_pos, -300, 100)];
    let mut places = vec![];
    for (_, _, pos, back, scale) in &models {
        let zref = c.addk(pos[2], *back);
        let a0 = c.divk(zref, *scale);
        let r0 = c.modk(zref, *scale);
        places.push((zref, a0, r0));
    }
    eval(host, m, d, &mut c);
    let dg = dg.map(|g| cardv(&c, g));
    let xs: Vec<[i32; 3]> = xs.iter().map(|x| x.map(|v| cardv(&c, v))).collect();
    let times: Vec<i32> = times.iter().map(|&t| cardv(&c, t)).chain([-1]).collect();
    for (e, (model, r, pos, _, scale)) in models.iter().enumerate() {
        let (zref, a0, r0) = places[e];
        let view = super::mesh::View { focal10: 80, zref, light: [zero; 3], half: [zero; 3] };
        let (_, px) = m.kit.carry(host, &mut c, model, &super::mesh::Place { rot: *r, pos: *pos }, &view, HW, HH);
        if px.is_empty() { continue; }
        let table = run(host, &m.kahead, &[cardv(&c, a0), cardv(&c, r0), *scale as i32], &[0; 1000], 1, 1);
        let items: Vec<(usize, usize, i32, [i32; 3])> = px.iter().map(|&(at, v)| ((at as usize) % HW, (at as usize) / HW, table[v[0] as usize], [v[1], v[2], v[3]])).collect();
        let keep = run(host, &m.keep, &[], &items.iter().flat_map(|&(x, y, a, _)| [a, s.depth[(y / 8) * 192 + x / 8]]).collect::<Vec<_>>(), 2, 1);
        let shown: Vec<&(usize, usize, i32, [i32; 3])> = items.iter().zip(&keep).filter(|(_, &k)| k == 1).map(|(i, _)| i).collect();
        let tex: Vec<[i32; 11]> = shown.iter().map(|it| kd.texel(it.3[2], it.3[0], it.3[1])).collect();
        let mut col: Vec<[i32; 3]> = tex.iter().map(|t| [t[0], t[1], t[2]]).collect();
        if let Some(dg) = dg {
            let deck: Vec<i32> = tex.iter().flat_map(|t| (0..3).flat_map(move |ch| [t[3 + ch], t[6 + ch]])).collect();
            let lit = run(host, &m.klit, &[dg], &deck, 2, 1);
            for (i, cc) in col.iter_mut().enumerate() { *cc = [lit[3 * i], lit[3 * i + 1], lit[3 * i + 2]]; }
            let inked: Vec<usize> = (0..tex.len()).filter(|&i| tex[i][9] > 0).collect();
            let ks = run(host, &m.kink, &[], &inked.iter().flat_map(|&i| [tex[i][9], times[tex[i][10] as usize]]).collect::<Vec<_>>(), 2, 1);
            for ch in 0..3 {
                let deck: Vec<i32> = inked.iter().zip(&ks).flat_map(|(&i, &k)| [col[i][ch], xs[tex[i][10] as usize][ch], k]).collect();
                for (j, v) in run(host, &m.mix, &[], &deck, 3, 1).into_iter().enumerate() { col[inked[j]][ch] = v; }
            }
        }
        // hazed by the dive's fog at its distance
        for ch in 0..3 {
            let deck: Vec<i32> = shown.iter().zip(&col).flat_map(|(it, cc)| [cc[ch], s.far[ch], d.orca_haze[it.2.min(199) as usize]]).collect();
            for (j, v) in run(host, &m.mix, &[], &deck, 3, 1).into_iter().enumerate() { col[j][ch] = v; }
        }
        for (it, cc) in shown.iter().zip(&col) { laid.push((it.0, it.1, *cc)); }
    }
    laid
}

/// The 4 x 4 pixels a blob's box covers about (x, y) (whole pixels, numbers), reaching r: the first's place and e
/// (tenths) across and down, fl(10 d^2, r2), each at most 100.
fn falloff(c: &mut Calc, x: N, y: N, reach: N, r2: N) -> (N, N, Vec<N>, Vec<N>, Vec<N>, Vec<N>) {
    let lo = c.sub(x, reach);
    let lo = c.clamp(lo, 0, 2000);
    let tx0 = c.divk(lo, 4);
    let tx0 = c.within(tx0, 500);
    let hi = c.add(x, reach);
    let hi = c.clamp(hi, -2000, 1535);
    let tx1 = c.divk(hi, 4);
    let tx1 = c.within(tx1, 500);
    let lo = c.sub(y, reach);
    let lo = c.clamp(lo, 0, 2000);
    let ty0 = c.divk(lo, 4);
    let ty0 = c.within(ty0, 500);
    let hi = c.add(y, reach);
    let hi = c.clamp(hi, -2000, 767);
    let ty1 = c.divk(hi, 4);
    let ty1 = c.within(ty1, 500);
    (tx0, ty0, vec![tx1], vec![ty1], vec![x, y], vec![r2])
}

fn silt(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas) {
    let kd = &d.knob;
    let puffs: Vec<(i64, i64)> = [(0i64, 314i32), (1, 323)].iter().filter(|&&(_, st)| (0..14).contains(&(s.n - st))).map(|&(p, st)| (p, (s.n - st) as i64)).collect();
    if puffs.is_empty() { return; }
    let mut c = Calc::new();
    let (cm, cam) = cam_nums(&mut c, s);
    let mut blobs = vec![];
    // each puff's age (frames since it started) on the calculator, and what's worked out from it
    let start = |p: i64| if p == 0 { 314 } else { 323 };
    let ages: BTreeMap<i64, N> = puffs.iter().map(|&(p, age)| { let n = c.num(s.n as i64); let a = c.addk(n, -start(p)); (age, c.within(a, 13)) }).collect();
    for &(puff, age) in &puffs {
        let ag = ages[&age];
        for i in 0..18i64 {
            let r = &kd.silt[((puff * 18 + i) * 3) as usize..];
            let a = c.num(1000 * r[0] as i64);
            let (sn, cs) = sin_cos(&mut c, a);
            let grown = { let g = c.mulk(ag, 12); c.addk(g, 490) };
            let rad = { let x = c.num(120 * r[1] as i64); let q = c.divk(x, 1000); let q = c.within(q, 120); c.add(q, grown) };
            let y = { let r2 = c.num(r[2] as i64); let up = c.addk(r2, 500); let a8 = c.mulk(ag, 8); let x = c.mul(a8, up); let q = c.divk(x, 1000); c.addk(q, -35) };
            let qx = { let x = c.mul(cs, rad); let q = c.divk(x, 999); c.within(q, 1000) };
            let qz = { let x = c.mul(sn, rad); let q = c.divk(x, 999); c.within(q, 1000) };
            let rel = from_cam(&mut c, [qx, y, qz], cam);
            blobs.push((age, pn(&mut c, rel, &cm)));
        }
    }
    eval(host, m, d, &mut c);
    let mut seen = vec![];
    for (age, mut p) in blobs {
        if !pn_seen(&c, &p) { continue; }
        pn_place(&mut c, &mut p);
        seen.push((age, p));
    }
    // each blob's size, its box and its fall-off across and down
    let mut boxes = vec![];
    for (age, p) in &seen {
        let a = c.within(p.a, 99_999);
        let k = { let a5 = c.mulk(ages[age], 5); let a = c.addk(a5, 35); c.mulk(a, 80_000) };
        let sz = c.div(k, a);
        let sz = c.divk(sz, 100);
        let one = c.num(1);
        let sz = c.max(sz, one);
        let sz = c.within(sz, 316);
        // e = fl(10 d^2, 2 sz^2), as fl(fl(10 d^2, 2), sz^2): a divisor under 100000
        let r2 = c.mul(sz, sz);
        let reach = c.mulk(sz, 3);
        let x = { let v = p.x.unwrap(); let q = c.divk(v, 1000); c.within(q, 2000) };
        let y = { let v = p.y.unwrap(); let q = c.divk(v, 1000); c.within(q, 2000) };
        let (tx0, ty0, tx1, ty1, xy, _) = falloff(&mut c, x, y, reach, r2);
        boxes.push((*age, tx0, ty0, tx1[0], ty1[0], xy, r2));
    }
    eval(host, m, d, &mut c);
    let mut sums: BTreeMap<usize, Vec<i32>> = BTreeMap::new();
    let mut per = vec![];
    for (age, tx0, ty0, tx1, ty1, xy, r2) in &boxes {
        let (tx0, ty0, tx1, ty1) = (val(&c, *tx0), val(&c, *ty0), val(&c, *tx1), val(&c, *ty1));
        if tx0 > tx1 || ty0 > ty1 { continue; }
        let hundred = c.num(100);
        let ex: Vec<N> = (tx0..=tx1).map(|tx| { let m4 = c.num(4 * tx + 2); let dx = c.sub(m4, xy[0]); let sq = c.mul(dx, dx); let sq = c.mulk(sq, 5); let e = c.div(sq, *r2); c.min(e, hundred) }).collect();
        let ey: Vec<N> = (ty0..=ty1).map(|ty| { let m4 = c.num(4 * ty + 2); let dy = c.sub(m4, xy[1]); let sq = c.mul(dy, dy); let sq = c.mulk(sq, 5); let e = c.div(sq, *r2); c.min(e, hundred) }).collect();
        per.push((*age, tx0, ty0, ex, ey));
    }
    // the fall-off's 256ths at each age
    let mut amt: BTreeMap<i64, Vec<N>> = BTreeMap::new();
    for &(_, age) in &puffs {
        let left = { let na = c.neg(ages[&age]); c.addk(na, 14) };
        amt.insert(age, kd.puff.iter().map(|&p| { let pn = c.num(p as i64); let x = c.mul(pn, left); c.divk(x, 28) }).collect());
    }
    eval(host, m, d, &mut c);
    for (age, tx0, ty0, ex, ey) in per {
        let ex: Vec<i32> = ex.iter().map(|&e| cardv(&c, e)).collect();
        let ey: Vec<i32> = ey.iter().map(|&e| cardv(&c, e)).collect();
        let deck: Vec<i32> = ey.iter().flat_map(|&b| ex.iter().flat_map(move |&a| [a, b])).collect();
        let idx = run(host, &m.kindex, &[99], &deck, 2, 1);
        let w = ex.len();
        for (k, &e) in idx.iter().enumerate() {
            if e < 0 { continue; }
            let t = ((ty0 + (k / w) as i64) as usize) * TW + (tx0 + (k % w) as i64) as usize;
            sums.entry(t).or_default().push(cardv(&c, amt[&age][e as usize]));
        }
    }
    let tiles: Vec<usize> = sums.keys().copied().collect();
    let total = sum_up(host, m, 141, &tiles.iter().map(|t| sums[t].clone()).collect::<Vec<_>>());
    let items: Vec<(usize, [i32; 3], i32)> = tiles.iter().zip(&total).filter(|(_, &v)| v > 0).map(|(&t, &v)| (t, SILT, v)).collect();
    mix_tiles(host, m, cv, &items, true);
}

fn debris(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas) {
    let kd = &d.knob;
    let mut c = Calc::new();
    let (cm, cam) = cam_nums(&mut c, s);
    let nn = c.num(s.n as i64);
    let mut flakes = vec![];
    for i in 0..140usize {
        let r: Vec<i64> = kd.debris[7 * i..7 * i + 7].iter().map(|&v| v as i64).collect();
        let born = { let x = c.num(18 * r[0]); let q = c.divk(x, 1000); c.addk(q, 332) };
        let age = c.sub(nn, born);
        let neg = c.below0(age);
        let k16 = c.num(16);
        let old = c.lt(k16, age);
        let a = c.num(1000 * r[1]);
        let (sn, cs) = sin_cos(&mut c, a);
        let sp = { let x = c.num(17 * r[3]); let q = c.divk(x, 1000); c.addk(q, 5) };
        let sp = c.within(sp, 22);
        let age16 = c.within(age, 2000);
        let rad = { let x = c.num(300 * r[2]); let q = c.divk(x, 1000); let q = c.within(q, 300); let q = c.addk(q, 180); let g = c.mul(sp, age16); c.add(q, g) };
        let rad = c.within(rad, 50_000);
        let ri = { let x = c.num(10 * r[4]); let q = c.divk(x, 1000); c.addk(q, 2) };
        let ri = c.within(ri, 12);
        let y = { let g = c.mul(ri, age16); let g = c.addk(g, 10); c.within(g, 1000) };
        let qx = { let x = c.mul(cs, rad); let q = c.divk(x, 999); c.within(q, 1000) };
        let qz = { let x = c.mul(sn, rad); let q = c.divk(x, 999); c.within(q, 1000) };
        let rel = from_cam(&mut c, [qx, y, qz], cam);
        let p = pn(&mut c, rel, &cm);
        let size = { let x = c.num(45 * r[6]); let q = c.divk(x, 1000); c.addk(q, 25) };
        let k = { let x = c.mulk(age, -272); let x = c.addk(x, 272 * 17); c.divk(x, 170) };
        flakes.push((i, neg, old, p, size, k, r[5] == 1));
    }
    eval(host, m, d, &mut c);
    let mut boxes = vec![];
    for (_, neg, old, mut p, size, k, pale) in flakes {
        if flag(&c, neg) || flag(&c, old) || !pn_seen(&c, &p) { continue; }
        pn_place(&mut c, &mut p);
        let a = c.within(p.a, 99_999);
        let sz = c.mulk(size, 8000);
        let sz = c.div(sz, a);
        let h = c.num(100);
        let p100 = c.max(sz, h);
        let r = c.mulk(p100, 10);
        let (x, y) = (p.x.unwrap(), p.y.unwrap());
        let x0 = { let v = c.sub(x, r); let v = c.divk(v, 1000); c.clamp(v, 0, 2000) };
        let x1 = { let v = c.add(x, r); let v = c.divk(v, 1000); c.clamp(v, -2000, 1535) };
        let y0 = { let v = c.sub(y, r); let v = c.divk(v, 1000); c.clamp(v, 0, 2000) };
        let y1 = { let v = c.add(y, r); let v = c.divk(v, 1000); c.clamp(v, -2000, 767) };
        boxes.push((x0, x1, y0, y1, k, pale));
    }
    eval(host, m, d, &mut c);
    for (x0, x1, y0, y1, k, pale) in boxes {
        let (x0, x1, y0, y1, k) = (val(&c, x0), val(&c, x1), val(&c, y0), val(&c, y1), cardv(&c, k));
        let col = if pale { PALE } else { CRUST };
        let px: Vec<(usize, usize)> = (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x as usize, y as usize))).collect();
        if px.is_empty() { continue; }
        let mut out = vec![[0; 3]; px.len()];
        for ch in 0..3 {
            let deck: Vec<i32> = px.iter().flat_map(|&(x, y)| [cv.get(x, y)[ch], col[ch], k]).collect();
            for (i, v) in run(host, &m.mix, &[], &deck, 3, 1).into_iter().enumerate() { out[i][ch] = v; }
        }
        for (i, &(x, y)) in px.iter().enumerate() { cv.set(x, y, out[i]); }
    }
}

/// A line of the lightning: its ends (thousandths of a pixel, numbers), whether it starts and ends its stroke, a
/// fork's.
struct SegN { a: [N; 2], b: [N; 2], first: bool, last: bool, fork: bool }

fn crackle(host: &mut Host, m: &Machines, d: &Data, s: &Scene, cv: &mut Canvas) {
    let n = s.n;
    let kd = &d.knob;
    let mut c = Calc::new();
    let (cm, cam) = cam_nums(&mut c, s);
    let ct = ctl(&mut c, n, kd);
    // this frame's arcs: how many, how far they reach (pixels), how wide, the flash (32nds)
    let (count, reach, w, flash) = match n {
        331 => (c.num(3), c.num(90), 2, c.num(4)),
        350 => (c.num(16), c.num(700), 4, c.num(11)),
        351 => (c.num(16), c.num(700), 4, c.num(19)),
        352 => (c.num(18), c.num(900), 5, c.num(30)),
        _ => {
            let a = { let x = c.mulk(ct.p19, 5); let q = c.divk(x, 19); c.addk(q, 1) };
            let r = { let x = c.mulk(ct.p19, 120); let q = c.divk(x, 19); c.addk(q, 60) };
            let f = { let x = c.mulk(ct.p19, 128); c.divk(x, 1900) };
            (a, r, 2, f)
        }
    };
    let zero = c.num(0);
    let rel = from_cam(&mut c, [zero; 3], cam);
    let mut centre = pn(&mut c, rel, &cm);
    eval(host, m, d, &mut c);
    let (count, flash) = (val(&c, count) as usize, cardv(&c, flash));
    let mut segs: Vec<SegN> = vec![];
    if pn_seen(&c, &centre) {
        pn_place(&mut c, &mut centre);
        let mut starts = vec![];
        for i in 0..count {
            let r: Vec<i64> = kd.crackle[(((n - 331) as usize * 20 + i) * 80)..][..80].iter().map(|&v| v as i64).collect();
            let a = c.num(1000 * r[0]);
            let (sn, cs) = sin_cos(&mut c, a);
            let rad = { let x = c.num(400 * r[1]); let q = c.divk(x, 1000); let q = c.within(q, 400); let q = c.addk(q, 600); let q = c.mulk(q, 460); let q = c.divk(q, 1000); c.within(q, 460) };
            let qx = { let x = c.mul(cs, rad); let q = c.divk(x, 999); c.within(q, 500) };
            let qz = { let x = c.mul(sn, rad); let q = c.divk(x, 999); c.within(q, 500) };
            let ten = c.num(10);
            let rel = from_cam(&mut c, [qx, ten, qz], cam);
            starts.push((r, pn(&mut c, rel, &cm)));
        }
        eval(host, m, d, &mut c);
        let (cx, cy) = (centre.x.unwrap(), centre.y.unwrap());
        for (r, mut p) in starts {
            if !pn_seen(&c, &p) { continue; }
            pn_place(&mut c, &mut p);
            let (mut x, mut y) = (p.x.unwrap(), p.y.unwrap());
            let dy = c.sub(y, cy);
            let dx = c.sub(x, cx);
            let (dx, dy) = (c.divk(dx, 10), c.divk(dy, 10));
            let (dx, dy) = (c.within(dx, 99_999), c.within(dy, 99_999));
            let at = c.atan2(dy, dx);
            let turn = c.num((r[2] - 500) * 159_155);
            let turn = c.divk(turn, 1000);
            let mut ang = c.add(at, turn);
            let len = { let h = c.num(r[3]); let h = c.divk(h, 2); let h = c.addk(h, 500); let h = c.mul(h, reach); let h = c.mulk(h, 100); c.divk(h, 1000) };
            let len = c.within(len, 90_000);
            for j in 0..10usize {
                let st = &r[4 + 7 * j..];
                let t = { let x = c.num((st[0] - 500) * 222_817); c.divk(x, 1000) };
                ang = c.add(ang, t);
                let (sn, cs) = sin_cos(&mut c, ang);
                let nx = { let v = c.mul(cs, len); let v = c.divk(v, 999); c.add(x, v) };
                let ny = { let v = c.mul(sn, len); let v = c.divk(v, 999); c.add(y, v) };
                segs.push(SegN { a: [x, y], b: [nx, ny], first: j == 0, last: j == 9, fork: false });
                (x, y) = (nx, ny);
                if st[1] == 1 {
                    let t = { let v = c.num((st[2] - 500) * 413_803); c.divk(v, 1000) };
                    let mut fa = c.add(ang, t);
                    let (mut fx, mut fy) = (x, y);
                    for mm in 0..4usize {
                        let t = { let v = c.num((st[3 + mm] - 500) * 190_986); c.divk(v, 1000) };
                        fa = c.add(fa, t);
                        let (sn, cs) = sin_cos(&mut c, fa);
                        let len10 = c.mulk(len, 10);
                        let gx = { let v = c.mul(cs, len10); let v = c.divk(v, 999 * 14); c.add(fx, v) };
                        let gy = { let v = c.mul(sn, len10); let v = c.divk(v, 999 * 14); c.add(fy, v) };
                        segs.push(SegN { a: [fx, fy], b: [gx, gy], first: mm == 0, last: mm == 3, fork: true });
                        (fx, fy) = (gx, gy);
                    }
                }
            }
        }
    }
    // which lines come within 64 pixels of the screen; each one's longer way, and its ends in that order
    let mut flags = vec![];
    for sg in &segs {
        let (minx, maxx) = (c.min(sg.a[0], sg.b[0]), c.max(sg.a[0], sg.b[0]));
        let (miny, maxy) = (c.min(sg.a[1], sg.b[1]), c.max(sg.a[1], sg.b[1]));
        let (lo, hx, hy) = (c.num(-64_000), c.num(1_599_000), c.num(831_000));
        let off = [c.lt(maxx, lo), c.lt(hx, minx), c.lt(maxy, lo), c.lt(hy, miny)];
        let dx = c.sub(sg.b[0], sg.a[0]);
        let dy = c.sub(sg.b[1], sg.a[1]);
        let (ax, ay) = (c.abs(dx), c.abs(dy));
        let steep = c.lt(ax, ay);
        flags.push((off, steep, dx, dy));
    }
    eval(host, m, d, &mut c);
    let near: Vec<bool> = flags.iter().map(|f| !f.0.iter().any(|&o| flag(&c, o))).collect();
    // the lines' pixels: along the longer way from the lesser end
    let mut lines = vec![];
    for (e, sg) in segs.iter().enumerate() {
        if !near[e] { continue; }
        let steep = flag(&c, flags[e].1);
        let (p, q) = if steep { ([sg.a[1], sg.a[0]], [sg.b[1], sg.b[0]]) } else { (sg.a, sg.b) };
        let back = c.lt(q[0], p[0]);
        lines.push((e, steep, p, q, back));
    }
    eval(host, m, d, &mut c);
    let mut runs = vec![];
    for (e, steep, p, q, back) in lines {
        let (a, b) = if flag(&c, back) { (q, p) } else { (p, q) };
        // the line's width, a fork one less but at least 1, on the calculator
        let wd = { let wn = c.num(w as i64); if segs[e].fork { let less = c.addk(wn, -1); let one = c.num(1); let m = c.max(less, one); c.within(m, w as i64) } else { wn } };
        let dx = c.sub(b[0], a[0]);
        let dx = c.within(dx, 99_999);
        let flat = c.is0(dx);
        let c0 = { let v = c.addk(a[0], 499); let q = c.divk(v, 1000); c.within(q, 3000) };
        let c1 = { let v = c.addk(b[0], -500); let q = c.divk(v, 1000); c.within(q, 3000) };
        let none = c.lt(c1, c0);
        runs.push((e, steep, a, b, wd, dx, flat, c0, c1, none));
    }
    eval(host, m, d, &mut c);
    let mut walks = vec![];
    for (e, steep, a, b, wd, dx, flat, c0, c1, none) in runs {
        if flag(&c, flat) || flag(&c, none) { continue; }
        let dy = c.sub(b[1], a[1]);
        let dy = c.within(dy, 99_999);
        let slope = { let v = c.mulk(dy, 1000); c.div(v, dx) };
        let y0 = { let v = c.mulk(c0, 1000); let v = c.addk(v, 500); let v = c.sub(v, a[0]); let v = c.within(v, 1000); let v = c.mul(v, dy); let v = c.div(v, dx); c.add(a[1], v) };
        let yb = c.divk(a[1], 1000);
        let yb = c.within(yb, 3000);
        let base = c.mulk(yb, 1000);
        let u0 = { let v = c.sub(y0, base); let h = c.mulk(wd, -500); let h = c.addk(h, 499); c.add(v, h) };
        let v0 = { let v = c.sub(y0, base); let h = c.mulk(wd, 500); let h = c.addk(h, -500); c.add(v, h) };
        let (slope, u0, v0) = (c.long(slope), c.long(u0), c.long(v0));
        walks.push((e, steep, c0, c1, yb, slope, u0, v0));
    }
    // the lines' glow: each line's box of 4 x 4 pixels, how far along and across it each is
    let reach_px = { let wn = c.num(w as i64); let x = c.mulk(wn, 2); let x = c.addk(x, 4); c.mulk(x, 3) };
    let mut glows = vec![];
    for (e, sg) in segs.iter().enumerate() {
        if !near[e] { continue; }
        let dxt = { let v = c.sub(sg.b[0], sg.a[0]); let q = c.divk(v, 100); c.within(q, 1000) };
        let dyt = { let v = c.sub(sg.b[1], sg.a[1]); let q = c.divk(v, 100); c.within(q, 1000) };
        let sq = c.lin(&[(dxt, Some(dxt), false), (dyt, Some(dyt), false)]);
        let lent = c.sqrt(sq);
        let lent = c.within(lent, 99_999);
        let zero_len = c.is0(lent);
        glows.push((e, dxt, dyt, lent, zero_len));
    }
    eval(host, m, d, &mut c);
    let mut boxes = vec![];
    for (e, dxt, dyt, lent, zero_len) in glows {
        if flag(&c, zero_len) { continue; }
        let sg = &segs[e];
        let ux = { let v = c.mulk(dxt, 1000); c.div(v, lent) };
        let uy = { let v = c.mulk(dyt, 1000); c.div(v, lent) };
        let (ux, uy) = (c.within(ux, 1000), c.within(uy, 1000));
        let len_px = c.divk(lent, 10);
        let len_px = c.within(len_px, 200);
        let box1 = |c: &mut Calc, a: N, b: N, top: i64| -> (N, N) {
            let lo = { let v = c.min(a, b); let v = c.divk(v, 1000); let v = c.sub(v, reach_px); c.clamp(v, 0, 9999) };
            let hi = { let v = c.max(a, b); let v = c.divk(v, 1000); let v = c.add(v, reach_px); c.clamp(v, -9999, top) };
            let (a4, b4) = (c.divk(lo, 4), c.divk(hi, 4));
            (c.within(a4, 500), c.within(b4, 500))
        };
        let (tx0, tx1) = box1(&mut c, sg.a[0], sg.b[0], 1535);
        let (ty0, ty1) = box1(&mut c, sg.a[1], sg.b[1], 767);
        let px0 = { let v = c.mulk(tx0, 4000); let v = c.addk(v, 2000); let v = c.sub(v, sg.a[0]); c.within(v, 200_000) };
        let py0 = { let v = c.mulk(ty0, 4000); let v = c.addk(v, 2000); let v = c.sub(v, sg.a[1]); c.within(v, 200_000) };
        let s0 = { let v = c.lin(&[(px0, Some(ux), false), (py0, Some(uy), false)]); c.divk(v, 1000) };
        let t0 = { let v = c.lin(&[(px0, Some(uy), false), (py0, Some(ux), true)]); c.divk(v, 1000) };
        let (s4x, s4y) = (c.mulk(ux, 4), c.mulk(uy, 4));
        let m4x = c.mulk(ux, -4);
        let lg = |c: &mut Calc, v: N| c.long(v);
        let (s0, t0, s4x, s4y, m4x) = (lg(&mut c, s0), lg(&mut c, t0), lg(&mut c, s4x), lg(&mut c, s4y), lg(&mut c, m4x));
        boxes.push((e, tx0, tx1, ty0, ty1, s0, t0, s4x, s4y, m4x, len_px));
    }
    eval(host, m, d, &mut c);
    // the lines' pixels: each column's rows (or each row's columns) from the walks, forks first, then the mains
    let mut core: BTreeMap<usize, i32> = BTreeMap::new();
    let deck: Vec<i32> = walks.iter().flat_map(|(_, _, c0, c1, _, sl, u0, v0)| {
        let (sl, u0, v0) = (long(&c, *sl), long(&c, *u0), long(&c, *v0));
        [(val(&c, *c1) - val(&c, *c0)) as i32, u0[0], u0[1], sl[0], sl[1], v0[0], v0[1], sl[0], sl[1]]
    }).collect();
    let rows = if walks.is_empty() { vec![] } else { run(host, &m.walk, &[], &deck, 9, 0) };
    let mut at = 0;
    let mut fills: Vec<(bool, Vec<(i64, i64)>)> = vec![];
    for (e, steep, c0, c1, yb, _, _, _) in &walks {
        let (c0, c1, yb) = (val(&c, *c0), val(&c, *c1), val(&c, *yb));
        let mut px = vec![];
        for k in 0..=(c1 - c0) {
            let (r0, r1) = (yb + rows[at] as i64, yb + rows[at + 1] as i64);
            at += 2;
            for r in r0..=r1 { px.push(if *steep { (r, c0 + k) } else { (c0 + k, r) }); }
        }
        fills.push((segs[*e].fork, px));
    }
    assert_eq!(at, rows.len());
    for fork in [true, false] {
        for (f, px) in &fills {
            if *f != fork { continue; }
            for &(x, y) in px { if (0..HW as i64).contains(&x) && (0..HH as i64).contains(&y) { core.insert(y as usize * HW + x as usize, if fork { 24 } else { 32 }); } }
        }
    }
    // the glow: each line's 4 x 4 pixels, how far past its ends and across (lines' starts down the box's first column,
    // then across), the table's glow where it reaches, summed per 4 x 4 pixels
    let mut parts: BTreeMap<usize, Vec<i32>> = BTreeMap::new();
    let table = &kd.glow[match w { 2 => 0, 4 => 1, _ => 2 }];
    for (e, tx0, tx1, ty0, ty1, s0, t0, s4x, s4y, m4x, len_px) in &boxes {
        let (tx0, tx1, ty0, ty1) = (val(&c, *tx0), val(&c, *tx1), val(&c, *ty0), val(&c, *ty1));
        if tx0 > tx1 || ty0 > ty1 { continue; }
        let (s0, t0, s4x, s4y, m4x) = (long(&c, *s0), long(&c, *t0), long(&c, *s4x), long(&c, *s4y), long(&c, *m4x));
        let deck = [(ty1 - ty0) as i32, s0[0], s0[1], s4y[0], s4y[1], t0[0], t0[1], m4x[0], m4x[1]];
        let starts = run(host, &m.rows, &[], &deck, 9, 4 * (ty1 - ty0 + 1) as usize);
        let wdeck: Vec<i32> = starts.chunks(4).flat_map(|p| [(tx1 - tx0) as i32, p[0], p[1], s4x[0], s4x[1], p[2], p[3], s4y[0], s4y[1]]).collect();
        let st = run(host, &m.walk, &[], &wdeck, 9, 0);
        let (first, last) = (segs[*e].first as i32, segs[*e].last as i32);
        let lp = cardv(&c, *len_px);
        let sdeck: Vec<i32> = st.chunks(2).flat_map(|p| [p[0], p[1], lp, first, last]).collect();
        let at = run(host, &m.kseg, &[], &sdeck, 5, 2);
        let wdt = (tx1 - tx0 + 1) as usize;
        for (k, p) in at.chunks(2).enumerate() {
            if p[0] < 0 { continue; }
            let t = (ty0 as usize + k / wdt) * TW + tx0 as usize + k % wdt;
            parts.entry(t).or_default().push(table[(64 * p[0] + p[1]) as usize]);
        }
    }
    // the pixels: the lines in white, the glow toward blue, the flash
    let core: Vec<(usize, i32)> = core.into_iter().collect();
    if !core.is_empty() {
        let mut out = vec![[0; 3]; core.len()];
        for ch in 0..3 {
            let deck: Vec<i32> = core.iter().flat_map(|&(p, k)| [cv.get(p % HW, p / HW)[ch], 255, k]).collect();
            for (i, v) in run(host, &m.mix, &[], &deck, 3, 1).into_iter().enumerate() { out[i][ch] = v; }
        }
        for (i, &(p, _)) in core.iter().enumerate() { cv.set(p % HW, p / HW, out[i]); }
    }
    let tiles: Vec<usize> = parts.keys().copied().collect();
    if !tiles.is_empty() {
        let total = sum_up(host, m, 32, &tiles.iter().map(|t| parts[t].clone()).collect::<Vec<_>>());
        let items: Vec<(usize, [i32; 3], i32)> = tiles.iter().zip(&total).filter(|(_, &g)| g > 0).map(|(&t, &g)| (t, BLUE, g)).collect();
        mix_tiles(host, m, cv, &items, false);
    }
    if flash > 0 {
        let items: Vec<(usize, [i32; 3], i32)> = (0..TW * TH).map(|t| (t, FLASH, flash)).collect();
        mix_tiles(host, m, cv, &items, false);
    }
}

/// Each list's sum, kept to the most (k_sum, at most 999 lists a deck).
fn sum_up(host: &mut Host, m: &Machines, most: i32, lists: &[Vec<i32>]) -> Vec<i32> {
    let mut out = vec![];
    for chunk in lists.chunks(999) {
        let mut deck = vec![most, chunk.len() as i32];
        for l in chunk { assert!(l.len() <= 999); deck.push(l.len() as i32); deck.extend(l); }
        out.extend(host.go1(&m.ksum, &[&deck], chunk.len()));
    }
    out
}

/// 999 sin and cos of an angle (millionths of a turn): the cos as the sin a quarter turn on (the reference's).
/// The angle taken whole turns on first, so it is never below 0 (the sines' table goes by its last two cards).
fn sin_cos(c: &mut Calc, a: N) -> (N, N) {
    let a = c.addk(a, 5_000_000);
    let (sn, _) = c.sin(a);
    let q = c.addk(a, 250_000);
    let (cs, _) = c.sin(q);
    (sn, cs)
}
