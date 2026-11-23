//! The mesh kit: a model from Blender (or any .glb/.obj) drawn on CARDIAC at up to 1536x768, placed by a turn and a
//! place the machines work out, with a depth test: flat lit (a light per face), or not lit, each pixel's depth or the
//! numbers its corners carry given back.
//!
//! The model is an asset, prepared once (`segments/mesh/data/make_mesh.py`): corners in whole units, normals in
//! hundredths, each triangle's face normal and its plane's distance. Everything worked out from it each frame is
//! CARDIAC's:
//!
//! - **the scene's numbers** (the swarm's arithmetic, `swarm_calc`): the turn's sines, the matrix, the light and the
//!   half-way vector turned into the model's space, the camera there in tens of units;
//! - **which triangles face the camera** (`polydot`, `mesh_front`): the face normal's dot with the camera against the
//!   plane's distance, with a margin;
//! - **the corners on the screen** (`swarm_calc`): each turned and placed exactly (thousandths of a unit), its depth,
//!   its place in hundredths of a pixel from the middle (a long division by the depth);
//! - **the light** at each face (flat): `polydot` the dots with the light and the half-way vector, `mesh_tone`, the
//!   highlight from its table (the host fetches it by the index the machine punched), `mesh_mix` the colour from the
//!   material's tables (`polydot`);
//! - **the edges** (the swarm's edge machines, `sw_order`, then `sw_esub` .. `sw_epix`): each edge of a triangle that
//!   faces the camera walked once from the end `sw_order` says is the top, where it crosses each row and the first
//!   pixel right of it, so triangles that share it meet exactly; its ends' values (depth, or what the corners carry)
//!   carried down its rows (`mesh_walk`);
//! - **the spans** (`mesh_ends`, `mesh_walk`): each triangle's row from its two edges' pixels, the left one and the
//!   count of pixels as `mesh_ends` punches them, every value carried across it, each pixel's place punched;
//! - **the depth test** (`mesh_depth`): the host sorts the pieces to their pixels, in the triangles' order, as a card
//!   sorter would; where more than one covers a pixel the machine keeps the nearest.
//!
//! The host deals, carries, sorts and fetches by index, and copies the computed pixels into the frame.
use crate::*;
use super::swarm_calc::{Arg, Calc, Items, N, Tables};
use std::collections::BTreeMap;
use std::path::Path;

/// A long: two cards, thousands and the rest (0..999).
type L = [i32; 2];

const MACHINES: &[&str] = &["sw_qi", "sw_qp", "sw_lsum", "sw_abs", "sw_divd", "sw_join", "sw_floor", "sw_add", "sw_short",
    "sw_esub", "sw_erows", "sw_eclip", "sw_eend", "sw_ewalk", "sw_ecross", "sw_epix",
    "sw_mag", "sw_qis", "sw_term", "sw_shift", "sw_floor3", "sw_sign", "sw_ang", "sw_sin", "sw_card", "sw_wide2",
    "sw_order", "polydot", "mesh_front", "mesh_tone", "mesh_half", "mesh_mix", "mesh_walk", "mesh_ends", "mesh_depth", "mesh_near"];

/// A model as the data prints it (make_mesh.py).
pub struct Model {
    pub faces: usize,
    /// corners: whole units, within the model's radius of its middle; their normals in hundredths
    pub v: Vec<[i32; 3]>,
    pub n: Vec<[i32; 3]>,
    /// triangles: corners, face normal (hundredths), plane's distance
    pub t: Vec<[usize; 3]>,
    pub fnorm: Vec<[i32; 3]>,
    pub fd: Vec<i32>,
    /// the farthest corner from the middle
    pub radius: i64,
}

impl Model {
    pub fn read(demo: &Path, file: &str) -> Model {
        let text = read(demo, file);
        let mut lines = text.lines().filter(|l| !l.starts_with('#'));
        let mut header = |what: &str| -> usize {
            let w: Vec<&str> = lines.next().unwrap().split_whitespace().collect();
            assert_eq!(w[0], what, "{file}");
            w[1].parse().unwrap()
        };
        let faces = header("faces");
        let nv = header("vertices");
        let mut m = Model { faces, v: vec![], n: vec![], t: vec![], fnorm: vec![], fd: vec![], radius: 0 };
        let mut nums = || -> Vec<i32> { lines.next().unwrap().split_whitespace().take(7).map(|v| v.parse().unwrap()).collect() };
        for _ in 0..nv {
            let x = nums();
            m.v.push([x[0], x[1], x[2]]);
            m.n.push([x[3], x[4], x[5]]);
        }
        let mut lines = text.lines().filter(|l| !l.starts_with('#')).skip(2 + nv);
        let nt: usize = lines.next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        for _ in 0..nt {
            let x: Vec<i32> = lines.next().unwrap().split_whitespace().take(7).map(|v| v.parse().unwrap()).collect();
            m.t.push([x[0] as usize, x[1] as usize, x[2] as usize]);
            m.fnorm.push([x[3], x[4], x[5]]);
            m.fd.push(x[6]);
        }
        m.radius = m.v.iter().map(|p| p.iter().map(|&c| (c as i64).pow(2)).sum::<i64>()).max().map(|r2| (r2 as f64).sqrt().ceil() as i64).unwrap_or(0);
        m
    }
}

/// How a model is lit: one colour a face; or not lit at all, each pixel's depth given back for the caller to colour
/// (a silhouette: `Kit::pixels`); or not lit, each corner's three numbers where its normal would be (a place in a
/// texture, say) carried across the triangles and given back with each pixel's depth (`Kit::carry`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shading { Flat, Smooth, Pixel, Depth, Carry }

/// A material: its ambient, diffuse and highlight colours (0..255).
#[derive(Clone, Copy)]
pub struct Material { pub ambient: [i32; 3], pub diffuse: [i32; 3], pub highlight: [i32; 3] }

/// Where a model is this frame, as the sums' numbers (worked out on CARDIAC): its turn R (thousandths, rows across,
/// up and away; R v turns a corner into the camera's space) and its middle's place in the camera's space (units).
pub struct Place { pub rot: [[N; 3]; 3], pub pos: [N; 3] }

/// The camera and the light. The camera is at the origin looking along z, x right and y up; `focal10` is its focal
/// length in pixels over 10. `zref` is the depth test's zero (units): every corner's depth less it must be 0..999.
/// `light` is the way toward the light, `half` the half-way vector between it and the way toward the camera
/// (thousandths, the camera's space): numbers of the sums, so they too can move.
pub struct View { pub focal10: i64, pub zref: N, pub light: [N; 3], pub half: [N; 3] }

/// What a draw did, for the reports.
#[derive(Default, Clone, Copy, Debug)]
pub struct Drawn { pub facing: usize, pub corners: usize, pub edges: usize, pub pieces: usize, pub pixels: usize, pub steps: [u64; 6] }

/// The stages' names, for MESH_STAGES.
pub const STAGES: [&str; 6] = ["scene", "faces", "corners", "light", "edges", "pixels"];

/// A material's tables (see Kit::tables).
struct Shade { ambient: [i32; 3], diffuse: Vec<[i32; 3]>, highlight: Vec<[i32; 3]>, squares: Vec<i32> }

pub struct Kit {
    m: BTreeMap<&'static str, Machine>,
    pub tables: Tables,
    /// the highlight, 99 (h / 969)^32, by the half-way dot (0..999)
    spec: Vec<i32>,
}

/// The margin of the back-face test, tens of units: a face seen nearly edge on is kept (drawn and hidden by the
/// depth test), never one that faces the camera dropped.
const MARGIN: i32 = 6;

impl Kit {
    pub fn new(demo: &Path) -> Kit {
        let mut m = BTreeMap::new();
        for &n in MACHINES { m.insert(n, rust(demo, n)); }
        let tables = super::swarm_calc::load_tables(&read(demo, "swarm/data/tables.txt"));
        let spec = read(demo, "mesh/data/spec.txt").lines().filter(|l| !l.starts_with('#')).flat_map(|l| l.split_whitespace().map(|v| v.parse::<i32>().unwrap()).collect::<Vec<_>>()).collect();
        Kit { m, tables, spec }
    }

    /// Run a machine over items: its deck is the count, the header, then the items, at most 999 a run.
    fn go(&self, host: &mut Host, name: &str, header: &[i32], items: &[Vec<i32>]) -> Vec<i32> {
        let mut out = vec![];
        for chunk in items.chunks(999) {
            let mut deck = Vec::with_capacity(1 + header.len() + chunk.iter().map(|c| c.len()).sum::<usize>());
            deck.push(chunk.len() as i32);
            deck.extend(header);
            for it in chunk { deck.extend(it); }
            if let Some(i) = deck.iter().position(|v| v.abs() > 999) { panic!("{name}: card {i} of {} is {}", deck.len(), deck[i]); }
            out.extend(host.gon(&self.m[name], &[&deck]));
        }
        out
    }

    /// Work out the sums so far.
    pub fn eval(&self, host: &mut Host, c: &mut Calc) {
        let mut go = |m: &'static str, items: Items, out: &mut Vec<i32>| {
            for (k, cards) in items.chunks(999) {
                let count = [k as i32];
                if let Some(i) = cards.iter().position(|v| v.abs() > 999) { panic!("{m}: card {} of {} is {}", i + 1, cards.len() + 1, cards[i]); }
                host.gon_into(&self.m[m], &[&count, cards], out);
            }
        };
        c.eval(&mut go, &self.tables);
    }

    /// A turn for `Place`: a yaw (about the up axis) then a pitch (about the across axis), angles in millionths of a
    /// turn (numbers of the sums). R = pitch after yaw, in thousandths: the sines from the swarm's table, products
    /// rounded down.
    pub fn turn(c: &mut Calc, yaw: N, pitch: N) -> [[N; 3]; 3] {
        let (sy, cy) = c.sin(yaw);
        let (sp, cp) = c.sin(pitch);
        let zero = c.num(0);
        let r10 = c.lin_k(&[(sp, Some(sy), false)]);
        let r12 = c.lin_k(&[(sp, Some(cy), true)]);
        let r20 = c.lin_k(&[(cp, Some(sy), true)]);
        let r22 = c.lin_k(&[(cp, Some(cy), false)]);
        [[cy, zero, sy], [r10, cp, r12], [r20, sp, r22]]
    }

    /// Products of polydot's: each item an addend and pairs (c, v), c -99..99; punches the bias plus the addend plus
    /// each c v / 100 cut toward zero.
    fn dots(&self, host: &mut Host, terms: usize, bias: i32, items: &[Vec<i32>]) -> Vec<i32> {
        for it in items { debug_assert_eq!(it.len(), 1 + 2 * terms); }
        self.go(host, "polydot", &[terms as i32, bias], items)
    }

    /// The material's tables, worked out by polydot: its diffuse colour times each light 0..99 and its highlight's
    /// colour times each highlight 0..99 (hundredths, cut to a whole), and each part of a normal squared (-102..102,
    /// hundredths) for the lift.
    fn tables(&self, host: &mut Host, mat: &Material, lift: bool) -> Shade {
        let part = |host: &mut Host, colour: [i32; 3]| -> Vec<[i32; 3]> {
            let items: Vec<Vec<i32>> = (0..100).flat_map(|k| (0..3).map(move |c| vec![0, k, colour[c]])).collect();
            self.dots(host, 1, 0, &items).chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
        };
        let diffuse = part(host, mat.diffuse);
        let highlight = part(host, mat.highlight);
        let squares = if lift { self.dots(host, 1, 0, &(-102..=102).map(|n| vec![0, n, n]).collect::<Vec<_>>()) } else { vec![] };
        Shade { ambient: mat.ambient, diffuse, highlight, squares }
    }

    /// The light of normals (hundredths; lifted toward a unit first when carried across a triangle): each one's
    /// colour, worked out by polydot, mesh_half, mesh_tone and mesh_mix, the host fetching from the tables by the
    /// indices the machines punched.
    fn light(&self, host: &mut Host, normals: &[[i32; 3]], lm: [i32; 3], hm: [i32; 3], sh: &Shade, lift: bool) -> Vec<[i32; 3]> {
        if normals.is_empty() { return vec![]; }
        let with = |w: [i32; 3]| -> Vec<Vec<i32>> { normals.iter().map(|n| vec![0, n[0], w[0], n[1], w[1], n[2], w[2]]).collect() };
        let d = self.dots(host, 3, 0, &with(lm));
        let h = self.dots(host, 3, 0, &with(hm));
        // lifted: each dot's change, itself times half of what the normal's size squared falls short of a unit's
        let (dc, hc) = if lift {
            let sq = |v: i32| sh.squares[(v + 102) as usize];
            let short = self.go(host, "mesh_half", &[], &normals.iter().map(|n| vec![sq(n[0]), sq(n[1]), sq(n[2])]).collect::<Vec<_>>());
            (self.dots(host, 1, 0, &d.iter().zip(&short).map(|(&v, &k)| vec![0, k, v]).collect::<Vec<_>>()),
             self.dots(host, 1, 0, &h.iter().zip(&short).map(|(&v, &k)| vec![0, k, v]).collect::<Vec<_>>()))
        } else { (vec![0; d.len()], vec![0; h.len()]) };
        let tone = self.go(host, "mesh_tone", &[], &(0..d.len()).map(|k| vec![d[k], dc[k], h[k], hc[k]]).collect::<Vec<_>>());
        // the colour's parts, fetched from the tables by the indices the machines punched
        let items: Vec<Vec<i32>> = tone.chunks(2).map(|t| {
            let s = self.spec[t[1] as usize];
            [sh.diffuse[t[0] as usize], sh.highlight[s as usize]].concat()
        }).collect();
        let rgb = self.go(host, "mesh_mix", &sh.ambient, &items);
        rgb.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
    }

    // --- the swarm's arithmetic for the edges ---------------------------------------------------------------------

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

    /// fl(N 1000^extra / D) and the remainder: N three cards, D a long (1..99999). The size's leading cards that the
    /// machine punched 0 are left out of the deck (fewer digits to divide).
    fn divide(&self, host: &mut Host, items: &[([i32; 3], usize, L)]) -> Vec<(L, L)> {
        let sizes = self.go(host, "sw_abs", &[], &items.iter().map(|(n, _, _)| n.to_vec()).collect::<Vec<_>>());
        let lead = |k: usize| if sizes[4 * k + 1] != 0 { 0 } else if sizes[4 * k + 2] != 0 { 1 } else { 2 };
        let deck: Vec<Vec<i32>> = items.iter().enumerate().map(|(k, (_, extra, d))| {
            let mut v = d.to_vec();
            v.push((3 - lead(k) + *extra) as i32);
            v.extend(&sizes[4 * k + 1 + lead(k)..4 * k + 4]);
            v.extend(std::iter::repeat_n(0, *extra));
            v
        }).collect();
        let digits = self.go(host, "sw_divd", &[], &deck);
        let (mut jdeck, mut rems, mut at) = (vec![], vec![], 0);
        for (k, (_, extra, _)) in items.iter().enumerate() {
            let n = 3 * (3 - lead(k) + extra);
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

    /// Each edge's crossings of the rows on the screen: (row, first pixel at or right of it) for each, from its top
    /// down, with the row's index along the edge (from its first row) and how many rows it meets in all. Edges: (top,
    /// bottom) places, longs in hundredths of a pixel from the middle, the top's y not below the bottom's.
    fn crossings(&self, host: &mut Host, edges: &[((L, L), (L, L))]) -> Vec<(i32, Vec<(i32, i32, i32)>)> {
        let mut found: Vec<(i32, Vec<(i32, i32, i32)>)> = vec![(0, vec![]); edges.len()];
        if edges.is_empty() { return found; }
        let sub = self.go(host, "sw_esub", &[], &edges.iter().map(|e| [e.0 .0, e.0 .1, e.1 .0, e.1 .1].concat()).collect::<Vec<_>>());
        let rows = self.go(host, "sw_erows", &[], &(0..edges.len()).map(|k| sub[5 * k + 2..5 * k + 5].to_vec()).collect::<Vec<_>>());
        // the edges that meet a row (by the count the machine punched)
        let live: Vec<usize> = (0..edges.len()).filter(|&k| rows[k] != 0).collect();
        if live.is_empty() { return found; }
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
        let mut div = vec![];
        for (n, &k) in live.iter().enumerate() { div.push((p[2 * n], 0, height(k))); div.push((p[2 * n + 1], 0, height(k))); }
        let q = self.divide(host, &div);
        let x0 = self.go(host, "sw_add", &[2], &live.iter().enumerate().map(|(n, &k)| [edges[k].0 .0, q[2 * n].0].concat()).collect::<Vec<_>>());
        let clip = self.go(host, "sw_eclip", &[], &live.iter().map(|&k| vec![edges[k].0 .1[0], edges[k].0 .1[1], rows[k]]).collect::<Vec<_>>());
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
        let mut at = 0;
        for (n, &k) in live.iter().enumerate() {
            found[k].0 = rows[k];
            for j in clip[2 * n]..end[n] {
                found[k].1.push((cross[3 * at], pixels[at], j));
                at += 1;
            }
        }
        assert_eq!(at, pixels.len());
        found
    }

    /// Draw a model on a frame (w x h, at most 1536x768, its middle the camera's axis): the pixels it covers given
    /// back (the frame's index, its colour), nearest first; with `Shading::Depth` each pixel's depth (less the view's
    /// zref, 0..999) in place of its colour's red, no light. The place and the view are numbers of the sums `c` holds;
    /// they are worked out here with the rest.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, host: &mut Host, c: &mut Calc, model: &Model, place: &Place, view: &View, mat: &Material, shading: Shading,
                frame: &mut [u8], w: usize, h: usize) -> Drawn {
        let (drawn, winners) = self.pixels(host, c, model, place, view, mat, shading, w, h);
        for (p, c3) in &winners {
            let o = 3 * *p as usize;
            frame[o] = c3[0] as u8; frame[o + 1] = c3[1] as u8; frame[o + 2] = c3[2] as u8;
        }
        drawn
    }

    /// The same, the pixels the model covers given back (the frame's index, its colour) rather than drawn; with
    /// `Shading::Depth` each pixel's depth (less the view's zref, 0..999) in place of its colour's red, no light.
    #[allow(clippy::too_many_arguments)]
    pub fn pixels(&self, host: &mut Host, c: &mut Calc, model: &Model, place: &Place, view: &View, mat: &Material, shading: Shading,
                  w: usize, h: usize) -> (Drawn, Vec<(u32, [i32; 3])>) {
        self.draw_pixels(host, c, model, place, view, std::slice::from_ref(mat), &vec![0; model.t.len()], shading, w, h)
    }

    /// As `pixels`, each triangle in its own material's light (`face_mat`: flat shading only, for more than one).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_pixels(&self, host: &mut Host, c: &mut Calc, model: &Model, place: &Place, view: &View, mats: &[Material], face_mat: &[usize],
                shading: Shading, w: usize, h: usize) -> (Drawn, Vec<(u32, [i32; 3])>) {
        let (drawn, px) = self.raster(host, c, model, place, view, mats, face_mat, shading, w, h);
        (drawn, px.into_iter().map(|(p, v)| (p, [v[1], v[2], v[3]])).collect())
    }

    /// `Shading::Carry`: each pixel the model covers (the frame's index), its depth (less the view's zref, 0..999) and
    /// the three numbers the model's corners carry where their normals would be, carried across its triangles.
    #[allow(clippy::too_many_arguments)]
    pub fn carry(&self, host: &mut Host, c: &mut Calc, model: &Model, place: &Place, view: &View, w: usize, h: usize) -> (Drawn, Vec<(u32, [i32; 4])>) {
        let mat = Material { ambient: [0; 3], diffuse: [0; 3], highlight: [0; 3] };
        self.raster(host, c, model, place, view, std::slice::from_ref(&mat), &vec![0; model.t.len()], Shading::Carry, w, h)
    }

    #[allow(clippy::too_many_arguments)]
    fn raster(&self, host: &mut Host, c: &mut Calc, model: &Model, place: &Place, view: &View, mats: &[Material], face_mat: &[usize],
                shading: Shading, w: usize, h: usize) -> (Drawn, Vec<(u32, [i32; 4])>) {
        assert!(w <= 1536 && h <= 768 && w % 2 == 0 && h % 2 == 0);
        assert!(mats.len() == 1 || shading == Shading::Flat, "materials by face are for flat shading");
        let mut drawn = Drawn::default();
        let mut from = host.steps;
        let stage = |host: &Host, steps: &mut [u64; 6], s: usize, from: &mut u64| { steps[s] += host.steps - *from; *from = host.steps; };
        // the scene: the light, the half-way vector and the camera in the model's space (R's transpose)
        let r = place.rot;
        let back = |c: &mut Calc, way: [N; 3], neg: bool| -> [N; 3] {
            std::array::from_fn(|j| c.lin_k(&[(r[0][j], Some(way[0]), neg), (r[1][j], Some(way[1]), neg), (r[2][j], Some(way[2]), neg)]))
        };
        // the light's two ways at 98 hundredths, so that a normal's dots with them stay within a card
        let (k98, k100) = (c.num(98), c.num(100));
        let mut short = |c: &mut Calc, way: [N; 3]| -> [N; 3] { let b = back(c, way, false); b.map(|n| { let m = c.mul(n, k98); c.div(m, k100) }) };
        let lm = short(c, view.light);
        let hm = short(c, view.half);
        let cam = back(c, place.pos, true);
        let ten = c.num(10);
        // the camera in tens of units, taken away: the back-face sums start from the plane's distance
        let cam10: [N; 3] = std::array::from_fn(|j| { let q = c.div(cam[j], ten); c.neg(q) });
        let card = |c: &mut Calc, n: N| c.run("sw_card", vec![Arg::All(n)], 1, 1, 999);
        let lm: [N; 3] = std::array::from_fn(|j| card(c, lm[j]));
        let hm: [N; 3] = std::array::from_fn(|j| card(c, hm[j]));
        let cam10: [N; 3] = std::array::from_fn(|j| card(c, cam10[j]));
        // the corners' sums need the turn and the place as numbers the products can take
        let p1000: [N; 3] = std::array::from_fn(|i| c.mulk(place.pos[i], 1000));
        self.eval(host, c);
        let shades: Vec<Shade> = mats.iter().map(|m| self.tables(host, m, shading == Shading::Pixel)).collect();
        let shade = &shades[0];
        let lm = lm.map(|n| c.cards(n)[0]);
        let hm = hm.map(|n| c.cards(n)[0]);
        let cam10 = cam10.map(|n| c.cards(n)[0]);
        stage(host, &mut drawn.steps, 0, &mut from);

        // which triangles face the camera: the plane's distance less the margin, less the normal's dot with the camera
        let items: Vec<Vec<i32>> = (0..model.t.len()).map(|k| {
            let n = model.fnorm[k];
            vec![model.fd[k], n[0], cam10[0], n[1], cam10[1], n[2], cam10[2]]
        }).collect();
        let sums = self.dots(host, 3, -MARGIN, &items);
        let flags = self.go(host, "mesh_front", &[], &sums.iter().map(|&v| vec![v]).collect::<Vec<_>>());
        let front: Vec<usize> = (0..model.t.len()).filter(|&k| flags[k] == 1).collect();
        drawn.facing = front.len();
        stage(host, &mut drawn.steps, 1, &mut from);

        // the corners of those triangles on the screen: turned and placed exactly (thousandths of a unit), the depth
        // rounded down to a unit, the place 10 f / 10 times across (or less up) over the depth, hundredths of a pixel
        let mut used: Vec<usize> = front.iter().flat_map(|&k| model.t[k]).collect();
        used.sort();
        used.dedup();
        drawn.corners = used.len();
        let (f10, minus_f10) = (c.num(view.focal10), c.num(view.focal10));
        let reach = 1000 * model.radius;
        let mut nodes = Vec::with_capacity(used.len());
        for &i in &used {
            let v = model.v[i].map(|x| c.num(x as i64));
            let t: [N; 3] = std::array::from_fn(|row| {
                let known = reach + 1000 * c.bound(place.pos[row]);
                c.lin_within(&[(r[row][0], Some(v[0]), false), (r[row][1], Some(v[1]), false), (r[row][2], Some(v[2]), false), (p1000[row], None, false)], Some(known))
            });
            let z = c.shift(t[2]);
            let nx = c.lin(&[(t[0], Some(f10), false)]);
            let ny = c.lin(&[(t[1], Some(minus_f10), true)]);
            let (sx, sy) = (c.div(nx, z), c.div(ny, z));
            let (sx, sy) = (c.long(sx), c.long(sy));
            let depth = c.sub(z, view.zref);
            let depth = card(c, depth);
            nodes.push((sx, sy, depth));
        }
        self.eval(host, c);
        let mut at: Vec<(L, L, i32)> = vec![([0, 0], [0, 0], 0); model.v.len()];
        for (&i, &(sx, sy, d)) in used.iter().zip(&nodes) {
            let (x, y) = (c.cards(sx), c.cards(sy));
            at[i] = ([x[0], x[1]], [y[0], y[1]], c.cards(d)[0]);
        }
        stage(host, &mut drawn.steps, 2, &mut from);

        // the light, at the faces; or what the corners carry
        let mut values: Vec<[i32; 4]> = vec![[0; 4]; model.v.len()];
        let mut face_rgb: BTreeMap<usize, [i32; 3]> = BTreeMap::new();
        match shading {
            Shading::Smooth => {
                let rgb = self.light(host, &used.iter().map(|&i| model.n[i]).collect::<Vec<_>>(), lm, hm, shade, false);
                for (&i, c) in used.iter().zip(rgb) { values[i] = [at[i].2, c[0], c[1], c[2]]; }
            }
            Shading::Pixel | Shading::Carry => { for &i in &used { values[i] = [at[i].2, model.n[i][0], model.n[i][1], model.n[i][2]]; } }
            Shading::Flat => {
                for (m, sh) in shades.iter().enumerate() {
                    let faces: Vec<usize> = front.iter().copied().filter(|&k| face_mat[k] == m).collect();
                    let rgb = self.light(host, &faces.iter().map(|&k| model.fnorm[k]).collect::<Vec<_>>(), lm, hm, sh, false);
                    for (&k, c) in faces.iter().zip(rgb) { face_rgb.insert(k, c); }
                }
                for &i in &used { values[i] = [at[i].2, 0, 0, 0]; }
            }
            Shading::Depth => { for &i in &used { values[i] = [at[i].2, 0, 0, 0]; } }
        }
        let channels = if matches!(shading, Shading::Flat | Shading::Depth) { 1 } else { 4 };
        stage(host, &mut drawn.steps, 3, &mut from);

        // each edge once, its top end first (1 from sw_order where the first end is below the second; a level edge
        // meets no row)
        let mut index: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        let mut ends: Vec<(usize, usize)> = vec![];
        for &k in &front {
            let t = model.t[k];
            for e in 0..3 {
                let (i, j) = (t[e], t[(e + 1) % 3]);
                index.entry((i.min(j), i.max(j))).or_insert_with(|| { ends.push((i, j)); ends.len() - 1 });
            }
        }
        let below = self.go(host, "sw_order", &[], &ends.iter().map(|&(i, j)| [at[i].1, at[j].1].concat()).collect::<Vec<_>>());
        for (k, e) in ends.iter_mut().enumerate() { if below[6 * k + 4] == 1 { *e = (e.1, e.0); } }
        drawn.edges = ends.len();
        let places: Vec<((L, L), (L, L))> = ends.iter().map(|&(a, b)| ((at[a].0, at[a].1), (at[b].0, at[b].1))).collect();
        let cross = self.crossings(host, &places);
        // the ends' values down each edge's rows
        let mut items = vec![];
        for (k, &(a, b)) in ends.iter().enumerate() {
            let rows = cross[k].0;
            if rows == 0 || cross[k].1.is_empty() { continue; }
            for ch in 0..channels { items.push(vec![0, rows, values[a][ch], values[b][ch]]); }
        }
        let walked = self.go(host, "mesh_walk", &[0], &items);
        // per edge, per row on the screen: (row, pixel, values)
        let mut on_rows: Vec<Vec<(i32, i32, [i32; 4])>> = vec![vec![]; ends.len()];
        let mut at_w = 0;
        for k in 0..ends.len() {
            let rows = cross[k].0 as usize;
            if rows == 0 || cross[k].1.is_empty() { continue; }
            for &(row, px, j) in &cross[k].1 {
                let mut v = [0; 4];
                for (ch, slot) in v.iter_mut().enumerate().take(channels) { *slot = walked[at_w + ch * rows + j as usize]; }
                on_rows[k].push((row, px, v));
            }
            at_w += channels * rows;
        }
        assert_eq!(at_w, walked.len());
        stage(host, &mut drawn.steps, 4, &mut from);

        // each triangle's spans: from its two edges' pixels on each row, the left first and the count of pixels as
        // mesh_ends punches them (none where the two meet), the values carried across
        struct Span { tri: usize, row: i32, n: usize }
        let mut rows_of = vec![];
        for &k in &front {
            let t = model.t[k];
            let mut by_row: BTreeMap<i32, Vec<(i32, [i32; 4])>> = BTreeMap::new();
            for e in 0..3 {
                let (i, j) = (t[e], t[(e + 1) % 3]);
                for &(row, px, v) in &on_rows[index[&(i.min(j), i.max(j))]] { by_row.entry(row).or_default().push((px, v)); }
            }
            for (row, cr) in by_row {
                assert_eq!(cr.len(), 2, "a triangle's row met {} edges", cr.len());
                rows_of.push((k, row, [cr[0], cr[1]]));
            }
        }
        let order = self.go(host, "mesh_ends", &[], &rows_of.iter().map(|(_, _, cr)| vec![cr[0].0, cr[1].0]).collect::<Vec<_>>());
        let mut spans = vec![];
        let (mut depth_items, mut chan_items) = (vec![], vec![]);
        for (q, &(k, row, mut cr)) in rows_of.iter().enumerate() {
            if order[2 * q] == 1 { cr.swap(0, 1); }
            let n = order[2 * q + 1];
            if n == 0 { continue; }
            assert!((1..=998).contains(&n), "a span of {n} pixels");
            let ((pa, va), (pb, vb)) = (cr[0], cr[1]);
            spans.push(Span { tri: k, row, n: n as usize });
            depth_items.push(vec![pa, pb, va[0], vb[0]]);
            for ch in 1..channels { chan_items.push(vec![pa, pb, va[ch], vb[ch]]); }
        }
        let depths = self.go(host, "mesh_walk", &[1], &depth_items);
        let chans = self.go(host, "mesh_walk", &[0], &chan_items);
        // the pieces, each at its pixel (place punched) in the triangles' order
        let (hw, hh) = ((w / 2) as i32, (h / 2) as i32);
        let mut pieces: Vec<(u32, [i32; 4])> = Vec::with_capacity(depths.len() / 2);
        let mut places: Vec<[i64; 2]> = Vec::with_capacity(depths.len() / 2);
        let (mut dat, mut cat) = (0, 0);
        for s in &spans {
            for kx in 0..s.n {
                let (x, d) = (depths[dat + 2 * kx], depths[dat + 2 * kx + 1]);
                let mut v = [d, 0, 0, 0];
                if channels == 4 { for ch in 0..3 { v[ch + 1] = chans[cat + ch * s.n + kx]; } } else if shading == Shading::Depth { v[1] = d; } else { let c3 = face_rgb[&s.tri]; v[1..].copy_from_slice(&c3); }
                if (-hw..hw).contains(&x) && (-hh..hh).contains(&s.row) {
                    pieces.push((((s.row + hh) as u32) * w as u32 + (x + hw) as u32, v));
                    places.push([s.row as i64, x as i64]);
                }
            }
            dat += 2 * s.n;
            if channels == 4 { cat += 3 * s.n; }
        }
        assert_eq!(dat, depths.len());
        drawn.pieces = pieces.len();
        // sorted to their pixels by row, then column (the card sorter: the triangles' order kept), the nearest kept
        // where more than one is
        let mut winners: Vec<(u32, [i32; 4])> = vec![];
        let mut contested: Vec<(u32, Vec<i32>)> = vec![];
        for g in sorter::groups(&places, &[sorter::CARD, sorter::CARD]) {
            if g.len() == 1 {
                winners.push(pieces[g[0]]);
            } else {
                let mut deck = vec![g.len() as i32];
                for &i in &g { deck.extend(pieces[i].1); }
                contested.push((pieces[g[0]].0, deck));
            }
        }
        if shading == Shading::Carry {
            let near = self.go(host, "mesh_near", &[], &contested.iter().map(|c| c.1.clone()).collect::<Vec<_>>());
            for (k, cp) in contested.iter().enumerate() { winners.push((cp.0, [near[4 * k], near[4 * k + 1], near[4 * k + 2], near[4 * k + 3]])); }
        } else {
            let near = self.go(host, "mesh_depth", &[], &contested.iter().map(|c| c.1.clone()).collect::<Vec<_>>());
            for (k, cp) in contested.iter().enumerate() { winners.push((cp.0, [0, near[3 * k], near[3 * k + 1], near[3 * k + 2]])); }
        }
        drawn.pixels = winners.len();
        // per pixel: the normal's light (after the depth test, only where it shows)
        if shading == Shading::Pixel {
            let rgb = self.light(host, &winners.iter().map(|w| [w.1[1], w.1[2], w.1[3]]).collect::<Vec<_>>(), lm, hm, shade, true);
            for (wi, c3) in winners.iter_mut().zip(rgb) { wi.1 = [wi.1[0], c3[0], c3[1], c3[2]]; }
        }
        stage(host, &mut drawn.steps, 5, &mut from);
        (drawn, winners)
    }
}

// --- the test: Suzanne turning under a light ----------------------------------------------------------------------

pub const FRAMES: usize = 48;
const W: usize = 1536;
const H: usize = 768;
pub type VideoHd = cardiac_host::Video<1536, 768>;
/// As the reference's (segments/mesh/reference.rs).
const F10: i64 = 150;
const POS: [i64; 3] = [0, 0, 1820];
const ZREF: i64 = 1321;
const YAW0: i64 = -96_000;
const YAW_STEP: i64 = 4000;
const PITCH: i64 = -30_000;
const LIGHT: [i64; 3] = [-451, 621, -641];
const HALF: [i64; 3] = [-249, 343, -906];
const CLAY: Material = Material { ambient: [34, 32, 36], diffuse: [205, 196, 186], highlight: [110, 110, 110] };
const BACK: [u8; 3] = [40, 42, 48];

pub fn render(demo: &Path, refbin: &str) {
    let kit = Kit::new(demo);
    let model = Model::read(demo, "mesh/data/suzanne2.txt");
    let shading = match std::env::var("MESH_SHADING").as_deref() { Ok("flat") => Shading::Flat, Ok("pixel") => Shading::Pixel, _ => Shading::Smooth };
    let mut host = Host::new();
    let mut video = VideoHd::new(FRAMES);
    let only: Option<Vec<usize>> = std::env::var("MESH_FRAMES").ok().map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let mut total = Drawn::default();
    for f in live::range(FRAMES) {
        if only.as_ref().is_some_and(|o| !o.contains(&f)) { continue; }
        live::time_frames(f..f + 1, || {
            let mut c = Calc::new();
            // the yaw: its start plus the frame's number of steps
            let (y0, step, fr) = (c.num(YAW0), c.num(YAW_STEP), c.num(f as i64));
            let yaw = c.lin(&[(y0, None, false), (fr, Some(step), false)]);
            let pitch = c.num(PITCH);
            let rot = Kit::turn(&mut c, yaw, pitch);
            let pos = POS.map(|v| c.num(v));
            let view = View { focal10: F10, zref: c.num(ZREF), light: LIGHT.map(|v| c.num(v)), half: HALF.map(|v| c.num(v)) };
            let frame = &mut video.buf[f * W * H * 3..(f + 1) * W * H * 3];
            // the background: a run of its colour
            for p in frame.chunks_mut(3) { p.copy_from_slice(&BACK); }
            let d = kit.draw(&mut host, &mut c, &model, &Place { rot, pos }, &view, &CLAY, shading, frame, W, H);
            total.facing += d.facing; total.corners += d.corners; total.edges += d.edges; total.pieces += d.pieces; total.pixels += d.pixels;
            for s in 0..6 { total.steps[s] += d.steps[s]; }
        });
    }
    host.done();
    if std::env::var("MESH_STAGES").is_ok() {
        let n = only.as_ref().map_or(FRAMES, |o| o.len()) as u64;
        eprintln!("a frame on average: {} triangles facing, {} corners, {} edges, {} pieces, {} pixels", total.facing as u64 / n, total.corners as u64 / n, total.edges as u64 / n, total.pieces as u64 / n, total.pixels as u64 / n);
        for (s, name) in STAGES.iter().enumerate() { eprintln!("  {name:8} {:>7.2}M a frame", total.steps[s] as f64 / n as f64 / 1e6); }
    }
    let reference = reference(refbin, &[demo]);
    finish("mesh", &host, &video, &reference, &demo.join("mesh/video_hd.rgb"), " (1536x768)");
}
