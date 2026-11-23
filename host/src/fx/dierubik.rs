//! The die and the Rubik's cube, 192x96 on black, for the BBS to lay over its page and
//! the rotozoomer: the plain cube coming toward us tumbling, going to a die (its faces to ivory, the pips fizzing in),
//! splitting into a scrambled Rubik's cube that plays the rubik segment's 16-move solve while it tumbles, a pop, then
//! roaming on, tumbling, out to about twice the distance and back.
//!
//! Everything is lists of cards through small machines, the rubik segment's where they fit (sine, dot, sum, hold,
//! order, turn, pocket, and its polygon machines edge, start, walk, span, piece, pixel) and this segment's own:
//! the clock (each angle from the frame's number and its rates), muldiv (the place and the sizes scaled to the
//! cube's distance), persp (a corner seen from the eye) and column (the polygon machines' column about a middle the
//! frame chooses). The host deals the lists, carries what one machine punches to the next, and sorts cards by numbers
//! machines punched, as a card sorter would: the cubies into drawing order, an edge's bounds into rows, the pieces of
//! polygons onto their pixels. What it decides itself is the timeline (dierubik/scene.rs): which shape a frame shows,
//! which move is turning, where each part starts (the sum machine works out a step from the frame and that); and
//! which boxes make the shapes (the same file).
//!
//! Per frame: the tumble's three angles (clock), their sines (sine) and the matrix Ry Rx Rz (dot); the turning
//! layer's turn (ease, clock, sine) and its matrix (dot); the place: eased toward us (ease) or roaming (clock, sine,
//! dot); scaled to the distance (muldiv), the sizes too (ease, clock, sine, dot, muldiv); the axes at each size and
//! the place's depth along each axis (dot); which faces show (sum, hold); the cubies' keys (sum, dot) and which half
//! is drawn first (sum, order); each face's colour (dot, hold, ease); each corner (sum), seen (persp, sum); per
//! polygon its edges, spans and pieces; per pixel row its pixels (pixel). The split's crossfade is the mix machine's.
use crate::*;
use std::path::Path;

#[allow(dead_code)]
#[path = "../../../segments/dierubik/scene.rs"]
pub mod scene;
use scene::*;

/// A value with its negative, as the dot machine punches them.
type Signed = (i32, i32);

/// Sort records of `width` cards by the first `keys` cards of each, keeping their order otherwise, on the card
/// sorter (the last key card's columns first).
fn sort(cards: &[i32], width: usize, keys: usize) -> Vec<i32> {
    let mut records: Vec<&[i32]> = cards.chunks(width).collect();
    sorter::sort(&mut records, &vec![sorter::CARD; keys], |r| r[..keys].iter().map(|&c| c as i64).collect::<Vec<_>>());
    records.concat()
}

struct M { clock: Machine, sine: Machine, dot: Machine, sum: Machine, hold: Machine, ease: Machine, muldiv: Machine, order: Machine,
    turn: Machine, pocket: Machine, persp: Machine, edge: Machine, start: Machine, column: Machine, walk: Machine, span: Machine,
    piece: Machine, pixel: Machine, mix: Machine }

pub struct Data { table: Vec<i32>, colours: Vec<[i32; 3]>, light: Vec<i32>, moves: Vec<[i32; 3]>, pub turns: Vec<i32>, pips: Vec<Vec<i32>>,
    sizes: Vec<i32>, rates: Vec<Vec<i32>>, cubies: Vec<[i32; 3]>, list: Vec<i32> }

pub fn data(demo: &Path) -> Data {
    let r = |f: &str| numbers(&demo.join("dierubik/data").join(f));
    Data {
        table: numbers(&demo.join("rubik/data/sine.txt")),
        colours: r("colours.txt").chunks(3).map(|c| [c[0], c[1], c[2]]).collect(),
        light: r("light.txt"), moves: r("moves.txt").chunks(3).map(|c| [c[0], c[1], c[2]]).collect(), turns: r("turns.txt"),
        pips: read(demo, "dierubik/data/pips.txt").lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect(),
        sizes: r("sizes.txt"), rates: r("rates.txt").chunks(3).map(|c| c.to_vec()).collect(), cubies: cubies(&r("cubies.txt")), list: r("cubies.txt"),
    }
}

/// Lists of groups for the dot and sum machines, ended by 0.
struct Groups(Vec<i32>);
impl Groups {
    fn new() -> Self { Groups(vec![]) }
    fn add(&mut self, cards: &[i32]) { self.0.extend_from_slice(cards); }
    fn end(mut self) -> Vec<i32> { self.0.push(0); self.0 }
}

struct Ctx<'a> { m: &'a M, d: &'a Data, host: &'a mut Host }

impl Ctx<'_> {
    fn dot(&mut self, g: Groups) -> Vec<Signed> { let out = self.host.gon(&self.m.dot, &[&g.end()]); out.chunks(2).map(|c| (c[0], c[1])).collect() }
    fn dot1(&mut self, pairs: &[(i32, i32)]) -> Signed { let mut g = Groups::new(); g.add(&[pairs.len() as i32]); for &(x, y) in pairs { g.add(&[x, y]); } self.dot(g)[0] }
    fn sum(&mut self, g: Groups) -> Vec<i32> { self.host.gon(&self.m.sum, &[&g.end()]) }
    /// A step of the timeline: as it is, or counted from its part's start (first + f - at, on the sum machine).
    fn step(&mut self, s: Step) -> i32 {
        match s {
            Step::Is(n) => n,
            Step::Since { first, at, f } => { let mut g = Groups::new(); g.add(&[3, 1, first, 1, f, -1, at]); self.sum(g)[0] }
        }
    }
    /// Each value held between least and most (groups of at most 999).
    fn hold(&mut self, values: &[i32], least: i32, most: i32) -> Vec<i32> {
        let mut deck = vec![];
        for chunk in values.chunks(999) { deck.extend([chunk.len() as i32, least, most]); deck.extend(chunk); }
        deck.push(0);
        self.host.go1(&self.m.hold, &[&deck], values.len())
    }
    /// The ease machine: each (smooth, from, to, j, n).
    fn ease(&mut self, items: &[(bool, i32, i32, i32, i32)]) -> Vec<i32> {
        let mut deck = vec![items.len() as i32];
        for &(smooth, from, to, j, n) in items { deck.extend([if smooth { 3 } else { 1 }, from, to, j, n]); }
        self.host.go1(&self.m.ease, &[&deck], items.len())
    }
    fn muldiv(&mut self, items: &[(i32, i32, i32)]) -> Vec<i32> {
        let mut deck = vec![items.len() as i32];
        for &(a, b, c) in items { deck.extend([a, b, c]); }
        self.host.go1(&self.m.muldiv, &[&deck], items.len())
    }
    /// The clock machine: each angle's terms (count, rate), as its quarter turn and rest.
    fn clock(&mut self, angles: &[Vec<(i32, Vec<i32>)>]) -> Vec<(i32, i32)> {
        let mut deck = vec![angles.len() as i32];
        for terms in angles { deck.push(terms.len() as i32); for (n, r) in terms { deck.push(*n); deck.extend(r); } }
        self.host.go1(&self.m.clock, &[&deck], 2 * angles.len()).chunks(2).map(|c| (c[0], c[1])).collect()
    }
    fn sine(&mut self, (q, r): (i32, i32)) -> (i32, i32) { let s = self.host.go1(&self.m.sine, &[&[q, r], &self.d.table, &self.d.table], 2); (s[0], s[1]) }

    /// The tumble's matrix, Ry Rx Rz, [row][column].
    fn tumble(&mut self, f: &Frame) -> [[Signed; 3]; 3] {
        let extra = self.step(f.extra);
        let r = &self.d.rates;
        let angles: Vec<Vec<(i32, Vec<i32>)>> = (0..3).map(|k| vec![(f.i, r[k].clone()), (extra, r[3 + k].clone())]).collect();
        let a = self.clock(&angles);
        let [(sy, cy), (sx, cx), (sz, cz)] = [0, 1, 2].map(|k| self.sine(a[k]));
        let mut g = Groups::new();
        for (x, y) in [(sy, sx), (sy, cx), (cy, sx), (cy, cx), (sy, 999), (sz, 999), (sx, 999)] { g.add(&[1, x, y]); }
        let p = self.dot(g);
        let (p1, p2, p3, p4, nsy, nsz, nsx) = (p[0].0, p[1], p[2].0, p[3], p[4].1, p[5].1, p[6]);
        let mut g = Groups::new();
        g.add(&[2, cy, cz, p1, sz]);
        g.add(&[2, p1, cz, cy, nsz]);
        g.add(&[1, cx, sz]);
        g.add(&[1, cx, cz]);
        g.add(&[2, p3, sz, nsy, cz]);
        g.add(&[2, sy, sz, p3, cz]);
        let e = self.dot(g);
        [[e[0], e[1], p2], [e[2], e[3], (nsx.1, nsx.0)], [e[4], e[5], p4]]
    }

    /// The place: across, up (200ths) and the distance (70ths).
    fn place(&mut self, p: Place) -> (i32, i32, i32) {
        match p {
            Place::Approach(k) => { let v = self.ease(&[(true, -246, 0, k, 18), (true, 89, 0, k, 18), (true, 910, 395, k, 18)]); (v[0], v[1], v[2]) }
            Place::Still => (0, 0, 395),
            Place::Roam(k) => {
                let k = self.step(k);
                let angles: Vec<Vec<(i32, Vec<i32>)>> = (0..4).map(|i| vec![(k, self.d.rates[6 + i].clone())]).collect();
                let a = self.clock(&angles);
                let [(_, ca), (_, cb), (sc, _), (sd, _)] = [0, 1, 2, 3].map(|i| self.sine(a[i]));
                let mut g = Groups::new();
                g.add(&[4, 260, 999, -260, ca, 120, 999, -120, cb]);
                g.add(&[5, 395, 999, 182, 999, -182, ca, 84, 999, -84, cb]);
                let z = self.dot(g);
                let (z100, z70) = (z[0].0, z[1].0);
                let mut g = Groups::new();
                g.add(&[3, 100, 999, z100, 999, z100, 100]);
                g.add(&[2, 60, 999, z100, 599]);
                let wv = self.dot(g);
                let mut g = Groups::new();
                g.add(&[1, wv[0].0, sc]);
                g.add(&[1, wv[1].0, sd]);
                let xy = self.dot(g);
                (xy[0].0, xy[1].0, z70)
            }
        }
    }
}

/// One shape at its place: its pixels (192x96 RGB), black where nothing is.
fn shape(cx: &mut Ctx, f: &Frame, which: Shape, pockets: &[Vec<i32>]) -> Vec<i32> {
    let d = cx.d;
    let g0 = cx.tumble(f);
    let (die, rubik) = match which { Shape::Die(dd) => (Some(dd), None), Shape::Rubik(r) => (None, Some(r)), Shape::Split(..) => unreachable!() };
    // the turning layer's matrix: the tumble's times its turn about its axis, -side q quarter turns, eased
    let mut groups = vec![g0];
    let mut turning = None;
    if let Some(Rubik { turn: Some((mv, j, n)), .. }) = rubik {
        let [a, s, q] = d.moves[16 + mv];
        let j = cx.step(j);
        let e = cx.ease(&[(true, 0, 900, j, n)])[0];
        let angle = cx.clock(&[(0..q.abs()).map(|_| (1, vec![0, e, 0])).collect()])[0];
        let (sn, cs) = cx.sine(angle);
        let sn = if -s * q.signum() > 0 { sn } else { cx.dot1(&[(sn, 999)]).1 };
        let (i, k) = AFTER[a as usize];
        let mut g = Groups::new();
        for r in 0..3 {
            g.add(&[2, cs, g0[r][i].0, sn, g0[r][k].0]);
            g.add(&[2, cs, g0[r][k].0, sn, g0[r][i].1]);
        }
        let mixed = cx.dot(g);
        let mut g1 = g0;
        for r in 0..3 { g1[r][i] = mixed[2 * r]; g1[r][k] = mixed[2 * r + 1]; }
        groups.push(g1);
        turning = Some((a as usize, s));
    }
    // the place, on the view: 700 away, across and up 245 x / distance; the middle column for the corners' x
    let (x200, y200, z70) = cx.place(f.place);
    let c = cx.muldiv(&[(x200, 245, z70), (y200, 245, z70)]);
    let c = [c[0], c[1], 700];
    let shift = cx.muldiv(&[(c[0], 33, 700)])[0];
    let mut g = Groups::new();
    g.add(&[9, 1, 192]);
    for _ in 0..8 { g.add(&[1, shift]); }
    let middle = cx.sum(g)[0];
    let mut g = Groups::new();
    g.add(&[3, 1, middle, 1, middle, -1, 1]);
    let mirror = cx.sum(g)[0];
    // the sizes: each 98 times its 500ths (scaled) over the distance; the pips' as they fizz in
    let base: Vec<i32> = match rubik.map(|r| r.scale) {
        None | Some(Scale::One) => d.sizes.clone(),
        Some(scale) => {
            let s500 = match scale {
                Scale::Split(j) => { let j = cx.step(j); cx.ease(&[(false, 540, 500, j, 7)])[0] }
                Scale::Pop(k) => {
                    let k = cx.step(k);
                    let k = cx.hold(&[k], 0, 6)[0];
                    let angle = cx.clock(&[vec![(k, d.rates[12].clone())]])[0];
                    let (s, _) = cx.sine(angle);
                    cx.dot1(&[(500, 999), (30, s)]).0
                }
                Scale::One => unreachable!(),
            };
            cx.muldiv(&d.sizes.iter().map(|&a| (s500, a, 500)).collect::<Vec<_>>())
        }
    };
    let mut sz = cx.muldiv(&base.iter().map(|&a| (a, 98, z70)).collect::<Vec<_>>());
    let pips = die.and_then(|dd| dd.pips);
    sz.push(match pips { Some(j) => { let j = cx.step(j); cx.ease(&[(false, 0, sz[S017], j, 4)])[0] } None => 0 });
    // each group's axes at each size, [group][axis][size][part]; the place's depth along each axis
    let mut g = Groups::new();
    for gm in &groups { for b in 0..3 { for &s in &sz { for k in 0..3 { g.add(&[1, gm[k][b].0, s]); } } } }
    for gm in &groups { for b in 0..3 { g.add(&[3, gm[0][b].0, c[0], gm[1][b].0, c[1], gm[2][b].0, c[2]]); } }
    let all = cx.dot(g);
    let per = 3 * SIZES * 3;
    let part = |g: usize, b: usize, i: usize, k: usize| all[g * per + (b * SIZES + i) * 3 + k].0;
    let along = |g: usize, b: usize| all[groups.len() * per + 3 * g + b];

    // a face's test, s x (the place's depth along its axis) + its plane, negated: 1 or more where the face shows
    let test = |b: &Bx, a: usize, s: i32, g: &mut Groups| {
        let plane = b.plane(a, s);
        let dep = along(b.group, a);
        g.add(&[1 + plane.len() as i32, 1, if s > 0 { dep.1 } else { dep.0 }]);
        for (sign, i) in plane { g.add(&[-sign, sz[i]]); }
    };
    // a box's six faces, by face (2 axis + 1 if it faces up the axis)
    let six: Vec<(usize, i32)> = (0..6).map(|fc| (fc / 2, if fc % 2 == 1 { 1 } else { -1 })).collect();
    let shown_of = |cx: &mut Ctx, bodies: &[&Bx]| -> Vec<bool> {
        let mut g = Groups::new();
        for b in bodies { for &(a, s) in &six { test(b, a, s, &mut g); } }
        let v = cx.sum(g);
        cx.hold(&v, 0, 1).iter().map(|&x| x == 1).collect()
    };
    let mut boxes = vec![];
    if let Some(dd) = die {
        let (body, on) = die_boxes(&d.pips, dd.pips.is_some());
        let shown = shown_of(cx, &[&body]);
        boxes = in_order(body, on, &|fc| shown[fc]);
    }
    if let Some(r) = rubik {
        let pocket = &pockets[r.done];
        let cubies: Vec<([i32; 3], usize)> = match turning {
            None => d.cubies.iter().map(|&p| (p, 0)).collect(),
            Some((a, s)) => d.cubies.iter().map(|&p| (p, (p[a] == s) as usize)).collect(),
        };
        // which half is drawn first: the order machine, told whether the eye is on the turning layer's side (beyond
        // the plane half way out to it)
        let later: Vec<i32> = match turning {
            None => vec![1; cubies.len()],
            Some((a, s)) => {
                let dep = along(0, a);
                let mut g = Groups::new();
                g.add(&[3, 1, if s > 0 { dep.1 } else { dep.0 }, -1, sz[S034], 1, 55]);
                let eye = cx.sum(g)[0];
                let mut deck = vec![eye, s];
                for c in d.list.chunks(3) { deck.push(c[a]); }
                let order = cx.host.go1(&cx.m.order, &[&deck], 54);
                // the middle cubie (13) isn't drawn
                (0..27).filter(|&n| n != 13).map(|n| order[2 * n + 1]).collect()
            }
        };
        // each cubie's middle (sum), and its key: twice the place times it plus its own square (dot)
        let mut g = Groups::new();
        for &(p, grp) in &cubies { for k in 0..3 { g.add(&[3, p[0], part(grp, 0, S068, k), p[1], part(grp, 1, S068, k), p[2], part(grp, 2, S068, k)]); } }
        let middles = cx.sum(g);
        let mut g = Groups::new();
        for m in middles.chunks(3) { g.add(&[9, c[0], m[0], c[0], m[0], c[1], m[1], c[1], m[1], c[2], m[2], c[2], m[2], m[0], m[0], m[1], m[1], m[2], m[2]]); }
        let keys = cx.dot(g);
        let mut drawing: Vec<usize> = (0..cubies.len()).collect();
        sorter::sort(&mut drawing, &[sorter::CARD, sorter::CARD], |&n| [later[n] as i64, keys[n].1 as i64]);
        let made: Vec<(Bx, Vec<(usize, Vec<Bx>)>)> = drawing.iter().map(|&n| {
            let (p, grp) = cubies[n];
            cubie_boxes(p, grp, &|fc| { let v = pocket[pocket_index(p, fc)]; (v >= 0).then_some(v as usize) })
        }).collect();
        let bodies: Vec<&Bx> = made.iter().map(|(b, _)| b).collect();
        let shown = shown_of(cx, &bodies);
        for (n, (body, on)) in made.into_iter().enumerate() { boxes.extend(in_order(body, on, &|fc| shown[6 * n + fc])); }
    }

    // which faces of the boxes show
    let mut g = Groups::new();
    for b in &boxes { for &(a, s) in &six { test(b, a, s, &mut g); } }
    let v = cx.sum(g);
    let shows: Vec<bool> = cx.hold(&v, 0, 1).iter().map(|&x| x == 1).collect();
    let mut faces = vec![];
    for (n, b) in boxes.iter().enumerate() { for (k, &(a, s)) in six.iter().enumerate() { if shows[6 * n + k] { faces.push((b, a, s)); } } }

    // the colours: per face direction its light, then per material
    let mut normals: Vec<(usize, usize, i32)> = faces.iter().map(|&(b, a, s)| (b.group, a, s)).collect();
    normals.sort();
    normals.dedup();
    let mut g = Groups::new();
    for &(grp, a, _) in &normals { let gm = &groups[grp]; g.add(&[3, gm[0][a].0, d.light[0], gm[1][a].0, d.light[1], gm[2][a].0, d.light[2]]); }
    let dots = cx.dot(g);
    let pick = |v: Signed, s: i32| if s > 0 { v.0 } else { v.1 };
    let mut values = vec![];
    for (i, &(grp, a, s)) in normals.iter().enumerate() { values.push(pick(dots[i], s)); values.push(pick(groups[grp][2][a], -s)); }
    let held = cx.hold(&values, 0, 999);
    let mut g = Groups::new();
    for i in 0..normals.len() { g.add(&[3, 310, 999, 190, held[2 * i], 40, held[2 * i + 1]]); }
    let light: Vec<i32> = cx.dot(g).iter().map(|v| v.0).collect();
    let mut wanted: Vec<(usize, Material)> = faces.iter().map(|&(b, a, s)| (normals.iter().position(|&n| n == (b.group, a, s)).unwrap(), b.material)).collect();
    wanted.sort();
    wanted.dedup();
    let die_faces: Vec<[i32; 3]> = match die {
        Some(dd) => {
            let j = cx.step(dd.colour);
            let items: Vec<(bool, i32, i32, i32, i32)> = (0..6).flat_map(|fc| (0..3).map(move |ch| (fc, ch))).map(|(fc, ch)| (false, d.colours[PLAIN + fc][ch], d.colours[IVORY][ch], j, 8)).collect();
            cx.ease(&items).chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
        }
        None => vec![],
    };
    let mut g = Groups::new();
    for &(i, mat) in &wanted {
        let base = match mat { Material::Plastic => d.colours[PLASTIC], Material::Sticker(c) => d.colours[STICKERS + c], Material::Pip => d.colours[PIP], Material::DieFace(fc) => die_faces[fc] };
        for ch in 0..3 { g.add(&[2, base[ch], light[i], base[ch], light[i]]); }
    }
    let raw: Vec<i32> = cx.dot(g).iter().map(|v| v.0).collect();
    let colours = cx.hold(&raw, 0, 255);
    let colour = |grp: usize, a: usize, s: i32, mat: Material| -> Vec<i32> {
        let i = normals.iter().position(|&n| n == (grp, a, s)).unwrap();
        let at = wanted.iter().position(|&w| w == (i, mat)).unwrap();
        colours[3 * at..3 * at + 3].to_vec()
    };

    // every corner (sum), seen (persp), y as the polygon machines take it (sum)
    let mut g = Groups::new();
    for &(b, a, s) in &faces {
        for corner in b.corners(a, s) {
            for k in 0..3 {
                let terms: Vec<(i32, i32)> = corner.iter().enumerate().flat_map(|(ax, t)| t.iter().map(move |&(sign, i)| (sign, ax, i)).collect::<Vec<_>>()).map(|(sign, ax, i)| (sign, part(b.group, ax, i, k))).collect();
                g.add(&[1 + terms.len() as i32, 1, c[k]]);
                for (sign, v) in terms { g.add(&[sign, v]); }
            }
        }
    }
    let points = cx.sum(g);
    let mut seen = vec![];
    for part in points.chunks(3 * 400) {
        let mut deck = vec![(2 * part.len() / 3) as i32];
        for p in part.chunks(3) { deck.extend([p[0], p[2], shift, p[1], p[2], 0]); }
        seen.extend(cx.host.go1(&cx.m.persp, &[&deck], 4 * part.len() / 3));
    }
    let mut g = Groups::new();
    for s in seen.chunks(4) { g.add(&[2, 1, 24, 1, s[3]]); }
    let down = cx.sum(g);
    let screen: Vec<(i32, i32)> = seen.chunks(4).zip(&down).map(|(s, &y)| (s[0], y)).collect();

    // per polygon: its edges, spans and pieces of pixels
    let mut pieces = vec![];
    for (n, &(b, a, s)) in faces.iter().enumerate() {
        let mut deck = vec![4];
        for &(x, y) in &screen[4 * n..4 * n + 4] { deck.extend([x, y]); }
        let edges = cx.host.gon(&cx.m.edge, &[&deck]);
        let edges = cx.host.gon(&cx.m.start, &[&edges]);
        let edges = cx.host.gon(&cx.m.column, &[&[middle, mirror], &edges]);
        let bounds = cx.host.gon(&cx.m.walk, &[&edges]);
        let mut deck = sort(&bounds, 3, 2);
        deck.push(-1);
        let spans = cx.host.gon(&cx.m.span, &[&deck]);
        pieces.extend(cx.host.gon(&cx.m.piece, &[&colour(b.group, a, s, b.material), &spans]));
    }
    // the pieces onto their pixels: by pixel row, then column, in drawing order
    let pieces = sort(&pieces, 7, 2);
    let mut frame = vec![0; W2 * H2 * 3];
    let mut next = 0;
    for y in 0..H2 {
        let mut mine: Vec<Vec<&[i32]>> = vec![vec![]; W2];
        while next < pieces.len() / 7 && pieces[7 * next] == y as i32 { let p = &pieces[7 * next..7 * next + 7]; mine[p[1] as usize].push(p); next += 1; }
        let covered: Vec<usize> = (0..W2).filter(|&x| !mine[x].is_empty()).collect();
        if covered.is_empty() { continue }
        for ch in 0..3 {
            let mut deck = vec![covered.len() as i32];
            for &x in &covered {
                deck.push(0);
                for p in &mine[x] { deck.extend([p[2], p[3], p[4 + ch]]); }
                deck.push(-1);
            }
            let got = cx.host.go1(&cx.m.pixel, &[&deck], covered.len());
            for (&x, &v) in covered.iter().zip(&got) { frame[(y * W2 + x) * 3 + ch] = v; }
        }
    }
    assert_eq!(next * 7, pieces.len());
    frame
}

fn pocket_index(p: [i32; 3], f: usize) -> usize { pocket(p, f) }

/// The cube after the scramble and after each move of the solve, its stickers filed in their pockets by the pocket
/// machine's numbers (the turn machine turning them, dealt each sticker with the move's axis first).
fn pockets(m: &M, d: &Data, host: &mut Host, demo: &Path) -> Vec<Vec<i32>> {
    let solved: Vec<[i32; 7]> = numbers(&demo.join("dierubik/data/solved.txt")).chunks(7).map(|c| [c[0], c[1], c[2], c[3], c[4], c[5], c[6]]).collect();
    let mut out = vec![];
    let mut cube = solved;
    for k in 0..=32 {
        if k >= 16 {
            let deck: Vec<i32> = cube.iter().flat_map(|s| s[..6].to_vec()).collect();
            let keys = host.go1(&m.pocket, &[&deck], 54);
            let mut pocket = vec![-1; 162];
            for (s, &key) in cube.iter().zip(&keys) { pocket[key as usize] = s[6]; }
            out.push(pocket);
        }
        if k < 32 {
            let mv = d.moves[k];
            let a = mv[0] as usize;
            let order = [a, (a + 1) % 3, (a + 2) % 3];
            let mut deck = vec![mv[1], mv[2]];
            for s in &cube { for base in [0, 3] { for &i in &order { deck.push(s[base + i]); } } deck.push(s[6]); }
            let got = host.go1(&m.turn, &[&deck], 54 * 7);
            cube = got.chunks(7).map(|c| { let mut s = [0; 7]; for base in [0, 3] { for (i, &k) in order.iter().enumerate() { s[base + k] = c[base + i]; } } s[6] = c[6]; s }).collect();
        }
    }
    out
}

/// Frames: 18 coming toward us, 8 going to a die, 12 of the die, 6 splitting, 72 of the solve (turns.txt), 14 of the
/// pop, 80 roaming.
pub const FRAMES: usize = 210;

pub fn render(demo: &Path, refbin: &str) {
    let r = |n: &str| rust(demo, n);
    let m = M { clock: r("dr_clock"), sine: r("rb_sine"), dot: r("rb_dot"), sum: r("rb_sum"), hold: r("rb_hold"), ease: r("ui_ease"), muldiv: r("dr_muldiv"),
        order: r("rb_order"), turn: r("rb_turn"), pocket: r("rb_pocket"), persp: r("dr_persp"), edge: r("rb_edge"), start: r("rb_start"), column: r("dr_column"),
        walk: r("rb_walk"), span: r("rb_span"), piece: r("rb_piece"), pixel: r("rb_pixel"), mix: r("umix") };
    let d = data(demo);
    let n = scene::frames(&d.turns);
    assert_eq!(n, FRAMES, "turns.txt and FRAMES differ");
    let mut host = Host::new();
    let pockets = pockets(&m, &d, &mut host, demo);
    let mut video = Video2::new(n);
    for fi in live::frames(n) {
        let f = frame(&d.turns, fi);
        let mut cx = Ctx { m: &m, d: &d, host: &mut host };
        let px = match f.shape {
            Shape::Split(die, rubik, j) => {
                let a = shape(&mut cx, &f, Shape::Die(die), &pockets);
                let b = shape(&mut cx, &f, Shape::Rubik(rubik), &pockets);
                let j = cx.step(j);
                let cover = cx.ease(&[(false, 0, 32, j, 7)])[0];
                let mut out = Vec::with_capacity(a.len());
                for (pa, pb) in a.chunks(333).zip(b.chunks(333)) {
                    let mut deck = vec![pa.len() as i32];
                    for (&x, &y) in pa.iter().zip(pb) { deck.extend([x, y, cover]); }
                    out.extend(cx.host.go1(&cx.m.mix, &[&deck], pa.len()));
                }
                out
            }
            s => shape(&mut cx, &f, s, &pockets),
        };
        for y in 0..H2 { video.row(fi, y, &px[y * W2 * 3..(y + 1) * W2 * 3]); }
    }
    host.done();
    finish("dierubik", &host, &video, &reference(refbin, &[demo]), &demo.join("dierubik/video.rgb"), "");
}
