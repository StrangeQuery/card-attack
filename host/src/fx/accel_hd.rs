//! The accelerated drive's half at 768x384 (segments/accel/hd.rs is its reference): every number a machine's, the
//! host dealing decks, fetching table entries by the indexes the machines punch, filing pixels.
//!
//! Per frame: the drive's dz, dcam and dturn place the camera and turn it; acam its height; aray the 769 columns'
//! rays. The street: at 25 sample columns (every 32 pixels) the ray is marched out (amz along, twice: the plan's
//! place and the dash's; amx across), afine the plan's middle and grade between entries, amoff how far from the
//! street's middle, amul and amrow the row where each step is seen, amlamp and ampool the lamps' light and the haze,
//! amrows which step each row sees, amnum and amlerp each row's distance, place across, light, haze and dash between
//! two steps.
use crate::*;
use std::collections::HashMap;
use std::path::Path;

pub const RW: usize = 768;
pub const RH: usize = 384;

const NAMES: &[&str] = &["aphase", "aswerve", "alane", "anear", "athree", "dz", "dcam", "dturn", "acam", "aray", "amz", "amx", "afine", "amoff", "amul", "amrow", "amlamp", "ampool", "amrows",
    "amnum", "amlerp", "around", "asrow", "asgap", "asdash", "asmat", "aswide", "aslong", "ascov", "ascut", "akover", "amix", "adither", "asedge", "dpick", "dface", "dlat", "dhit", "awprep", "awq", "awfar", "awx", "awtf", "awte",
    "awfoot", "awalk", "awup", "awbay", "awtex", "awblend", "awcol", "awclip",
    "dlamp", "acarz", "dcarx", "dmul", "dalong", "athvis", "athx", "afog", "aweight", "ay8", "apost1", "apost2", "apost3", "apost4", "apcol", "apshade", "ashade",
    "apaint", "aarm", "alamp", "a810", "asamp", "atwth", "acarxy", "alin", "apair", "aadd2", "asum", "ahalf",
    "agparam", "aamt", "agtab", "agaxis", "agpt", "agsum", "ajk", "ajflags", "abthr", "abox", "abmix", "abadd",
    "aflp", "apart", "aflv", "aflax", "afway", "afsq", "aflrow", "aghc", "aghg", "aghu", "aghs", "aavg4", "asrow4", "asrun", "asg4", "asql", "asqp", "ashu", "ashk", "asaxis", "asfade", "ageq", "aflabs"];

/// The machines that work each item of their deck alone (no card carried from one item to the next): the same item
/// always gets the same cards, so each distinct one is run once.
const ONE_AT_A_TIME: &[&str] = &["asum", "amix", "amul", "apaint", "agsum", "awtex", "awblend", "ascov", "ascut", "asedge", "akover", "apart", "aamt", "amlerp", "amnum", "abmix", "abadd", "aavg4", "ageq", "athx", "afog", "aweight", "ay8"];

/// The accelerated drive's data (segments/accel/data and the drive's).
pub struct Data { pub road: Vec<i32>, pub grade: Vec<i32>, pub turn: Vec<i32>,
    /// The camera's place across the street each frame (eighths; then two unused columns, as the drive's script):
    /// worked out on CARDIAC when the drive is set up (`Hd::lanes`).
    pub script: Vec<i32>,
    /// The tables the camera's path and our car's lane are worked out from: the wobble (data/lanesine.txt) and the
    /// swerve, whole eighths and 216ths over a pair (data/laneswerve.txt).
    pub lanesine: Vec<i32>, pub laneswerve: Vec<i32>, pub steps: Vec<i32>, pub along: Vec<i32>, pub across: Vec<i32>,
    pub palette: Vec<[i32; 3]>, pub street: HashMap<String, Vec<usize>>, pub cross: Vec<Vec<i32>>, pub dither: Vec<i32>,
    /// The card's DAC: each channel's level as 8-bit (data/dac.txt).
    pub dac: Vec<Vec<u8>>, pub blocks: Vec<i32>, pub cars: Vec<i32>, pub traffic: Vec<[i32; 5]>, pub pictures: Vec<u8>, pub kinds: Vec<i32>, pub steps_soft: Vec<i32>, pub softq: Vec<Vec<i32>>, pub jet: Vec<(i32, i32, Vec<[i32; 4]>)>, pub glowt: Vec<i32>, pub exp: Vec<i32>, pub ln: Vec<i32>, pub ghosts: Vec<Vec<i32>>,
    /// The ghosts' profiles with each card repeated 16 times: an entry for every value aghs punches (0-575), so the
    /// host fetches a profile's card by it.
    pub ghost16: Vec<Vec<i32>>, pub bays: Vec<Vec<usize>>, pub roofs: Vec<Vec<usize>> }

impl Data {
    pub fn load(demo: &Path) -> Data {
        let a = |f: &str| numbers(&demo.join("accel/data").join(f));
        let pools: Vec<Vec<i32>> = read(demo, "accel/data/pools.txt").lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect();
        let lines = |f: &str| -> Vec<Vec<i32>> { read(demo, &format!("accel/data/{f}")).lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect() };
        let street = read(demo, "accel/data/street.txt").lines().map(|l| { let (n, v) = l.split_once(' ').unwrap(); (n.to_string(), v.split_whitespace().map(|k| k.parse().unwrap()).collect()) }).collect();
        Data { road: a("road.txt"), grade: a("grade.txt"), turn: numbers(&demo.join("drive/data/turn.txt")), script: vec![], lanesine: a("lanesine.txt"), laneswerve: a("laneswerve.txt"), steps: a("steps.txt"),
            along: pools[0].clone(), across: pools[1].clone(), palette: lines("palette.txt").iter().map(|c| [c[0], c[1], c[2]]).collect(), street,
            cross: lines("cross.txt"), dither: a("dither.txt"), dac: lines("dac.txt").iter().map(|l| l.iter().map(|&v| v as u8).collect()).collect(),
            blocks: a("blocks.txt"), cars: a("cars.txt"), traffic: lines("traffic.txt").iter().map(|l| [l[0], l[1], l[2], l[3], l[4]]).collect(),
            pictures: std::fs::read(demo.join("accel/data/traffic.bin")).expect("accel/data/traffic.bin: run make_data.py"), kinds: a("kinds.txt"), steps_soft: a("steps_soft.txt"), softq: lines("softq.txt"), jet: {
                let l = lines("jet.txt");
                let mut out = vec![];
                let mut i = 0;
                while i < l.len() {
                    let (w, h) = (l[i][1], l[i][2]);
                    let px = l[i + 1..i + 1 + h as usize].iter().flat_map(|r| r.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect::<Vec<_>>()).collect();
                    out.push((w, h, px));
                    i += 1 + h as usize;
                }
                out
            }, glowt: a("glowt.txt"), exp: lines("exp.txt")[0].clone(), ln: lines("exp.txt")[1].clone(), ghosts: lines("ghosts.txt"),
            ghost16: lines("ghosts.txt").iter().map(|r| r.iter().flat_map(|&g| std::iter::repeat_n(g, 16)).collect()).collect(),
            bays: lines("bays.txt").iter().map(|l| l.iter().map(|&v| v as usize).collect()).collect(),
            roofs: lines("roofs.txt").iter().map(|l| l.iter().map(|&v| v as usize).collect()).collect() }
    }
}

/// Where a row meets the street at a sample column (hd.rs's Ground).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ground { pub a4: i32, pub off: i32, pub pool: i32, pub fk: i32, pub zd: i32 }

/// The camera's numbers for a frame.
pub struct Cam { pub f: i32, pub slope: i32, pub entry: i32, pub into: i32, pub o: i32, pub minus: i32, pub far_end: i32, pub road_at: i32, pub blocks_at: i32, pub x8: i32, pub c: i32, pub s: i32, pub h8: i32, pub hf: i32, pub h0: i32,
    pub across: Vec<i32>, pub along: Vec<i32> }

pub struct Hd { m: HashMap<&'static str, Machine>, pub d: Data, pub jump: super::accel_jump::Jump,
    /// Drawing the software half: the scene at each 4 x 4 block's middle only (hd.rs's frame_soft).
    pub soft: std::cell::Cell<bool>, fogged: std::cell::RefCell<HashMap<(usize, i32), [i32; 3]>>,
    /// The glow's profile times an amount, made on agtab as each amount is first wanted.
    tables: std::cell::RefCell<HashMap<i32, Vec<i32>>> }

/// A pixel of the scene: its colour (tenths of a step) and distance (quarters; 999 the sky).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Px { pub c: [i32; 3], pub a4: i32 }

impl Hd {
    pub fn new(demo: &Path) -> Hd {
        let mut hd = Hd { m: NAMES.iter().map(|&n| (n, rust(demo, n))).collect(), d: Data::load(demo), jump: super::accel_jump::Jump::new(demo), soft: Default::default(), fogged: Default::default(), tables: Default::default() };
        // the camera's path and our car's lane: one run from the camera's first frame (k = -3), which follows the
        // car 3 frames behind, to the car's last (k = 239)
        let lane = hd.lanes(&mut Host::new(), 243);
        hd.d.script = lane[..236].iter().flat_map(|&l| [l, 0, 0]).collect();
        hd.jump.lane = lane[3..].to_vec();
        hd
    }

    /// The camera's path across the street and our car's lane, a frame each from k = 12 t = -3 on, in eighths from
    /// the street's middle. aphase punches where in the wobble's table each frame is (and the sign), aswerve which
    /// entry of the swerve's table applies; the host fetches the entries, and alane adds them up and rounds.
    fn lanes(&self, host: &mut Host, frames: usize) -> Vec<i32> {
        let n = frames as i32;
        let phase = self.go(host, "aphase", &[&[n]]);
        let swerve = self.go(host, "aswerve", &[&[n]]);
        let mut deck = vec![n];
        for (p, &e) in phase.chunks(2).zip(&swerve) {
            let e = 2 * e as usize;
            deck.extend([self.d.lanesine[p[0] as usize], p[1], self.d.laneswerve[e], self.d.laneswerve[e + 1]]);
        }
        self.go(host, "alane", &[&deck])
    }

    fn go(&self, host: &mut Host, name: &str, deck: &[&[i32]]) -> Vec<i32> { if std::env::var("ACCEL_TRACE").is_ok() { eprintln!("{name} {}", deck.iter().map(|d| d.len()).sum::<usize>()); } host.go(&self.m[name], &[deck], None) }

    /// A machine that takes a count then that many items (`each` cards an item): run over the items in pieces of
    /// at most 400 (a run's deck holds 1999 cards), its cards punched one after another.
    fn each(&self, host: &mut Host, name: &str, head: &[i32], items: &[i32], each: usize) -> Vec<i32> {
        // a machine that works each item alone gives the same item the same cards: each distinct item run once
        if ONE_AT_A_TIME.contains(&name) && items.len() > each {
            let list: Vec<&[i32]> = items.chunks(each).collect();
            let mut distinct: Vec<&[i32]> = list.clone();
            sorter::distinct(&mut distinct, &vec![sorter::CARD; each], |d| d.iter().map(|&c| c as i64).collect::<Vec<_>>());
            if distinct.len() < list.len() {
                let flat: Vec<i32> = distinct.iter().flat_map(|d| d.iter().copied()).collect();
                let got = self.each_all(host, name, head, &flat, each);
                let outs = got.len() / distinct.len();
                let at: HashMap<&[i32], usize> = distinct.iter().enumerate().map(|(k, &d)| (d, k)).collect();
                return list.iter().flat_map(|it| got[outs * at[it]..outs * at[it] + outs].iter().copied()).collect();
            }
        }
        self.each_all(host, name, head, items, each)
    }

    fn each_all(&self, host: &mut Host, name: &str, head: &[i32], items: &[i32], each: usize) -> Vec<i32> {
        let mut out = vec![];
        for piece in items.chunks((1990 / each).min(400) * each) {
            let mut deck = head.to_vec();
            deck.push((piece.len() / each) as i32);
            out.extend(self.go(host, name, &[&deck, piece]));
        }
        out
    }

    pub fn camera(&self, host: &mut Host, f: i32) -> Cam {
        let d = &self.d;
        let z = self.go(host, "dz", &[&[f]]);
        let (entry, into, o) = (z[0], z[1], z[3]);
        let lane = d.script[3 * f as usize];
        let under = &d.road[entry as usize..entry as usize + 9];
        let cam = self.go(host, "dcam", &[&[into, lane], under]);
        let turn = self.go(host, "dturn", &[&[cam[1]], &d.turn]);
        let (c, s) = (turn[0], turn[1]);
        let h = self.go(host, "acam", &[&[d.grade[entry as usize], d.grade[entry as usize + 1], into]]);
        let across = self.go(host, "aray", &[&[c, s, 0]]);
        let along = self.go(host, "aray", &[&[c, s, 1]]);
        Cam { f, slope: cam[1], entry, into, o, minus: z[4], far_end: z[5], road_at: z[6], blocks_at: z[7], x8: cam[0], c, s, h8: h[0], hf: h[1], h0: 152, across, along }
    }

    /// Sample column x's ground, for each row (hd.rs's march).
    pub fn march(&self, host: &mut Host, cam: &Cam, x: usize) -> Vec<Option<Ground>> {
        let d = &self.d;
        let (rx, rz) = (cam.across[x], cam.along[x]);
        let steps = if self.soft.get() { &d.steps_soft } else { &d.steps };   // the places, then 0
        let n = steps.len() - 1;
        let plan = self.go(host, "amz", &[&[rz, cam.into, 128], steps]);
        let dash = self.go(host, "amz", &[&[rz, cam.o, 480], steps]);
        let side = self.go(host, "amx", &[&[rx], steps]);
        let at = |k: usize| (cam.entry + plan[2 * k]) as usize;
        let deck: Vec<i32> = (0..n).flat_map(|k| [d.road[at(k)], d.road[at(k) + 1], plan[2 * k + 1]]).collect();
        let road_lean = self.go(host, "afine", &[&[n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [d.grade[at(k)], d.grade[at(k) + 1], plan[2 * k + 1]]).collect();
        let grade_lean = self.go(host, "afine", &[&[n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [d.road[at(k)], road_lean[k], side[2 * k], side[2 * k + 1]]).collect();
        let off = self.go(host, "amoff", &[&[cam.x8, if rx < 0 { -1 } else { 1 }, n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [cam.h8, d.grade[at(k)], 220, steps[k]]).collect();
        let q1 = self.go(host, "amul", &[&[n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [q1[2 * k + 1], 0, 16, steps[k]]).collect();
        let q2 = self.go(host, "amul", &[&[n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [cam.hf, grade_lean[k], 110, steps[k]]).collect();
        let q3 = self.go(host, "amul", &[&[n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [q3[2 * k + 1], 0, 4, steps[k]]).collect();
        let q4 = self.go(host, "amul", &[&[n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [q1[2 * k], q2[2 * k], q3[2 * k], q4[2 * k]]).collect();
        let rows = self.go(host, "amrow", &[&[cam.h0, -2, 400, n as i32], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [dash[2 * k], dash[2 * k + 1], plan[2 * k], plan[2 * k + 1]]).collect();
        let lamps = self.go(host, "amlamp", &[&[n as i32, plan[0], plan[1], dash[1]], &deck]);
        let deck: Vec<i32> = (0..n).flat_map(|k| [d.along[lamps[2 * k] as usize], d.across[off[2 * k + 1] as usize], steps[k]]).collect();
        let pools = self.go(host, "ampool", &[&[n as i32], &deck]);
        if std::env::var("ACCEL_STEPS").map(|v| v == x.to_string()).unwrap_or(false) {
            for k in 0..n { eprintln!("step {} a {} off {} lamp {} q {} pool {} fk {} row {} {} q1 {} {} q2 {} q3 {} D {} {} F {} {}", k, steps[k], off[2 * k], lamps[2 * k], off[2 * k + 1], pools[2 * k], pools[2 * k + 1], rows[2 * k], rows[2 * k + 1], q1[2*k], q1[2*k+1], q2[2*k], q3[2*k], cam.h8, d.grade[at(k)], cam.hf, grade_lean[k]); }
        }
        let which = self.go(host, "amrows", &[&[n as i32], &rows]);
        // each row's step, its share of the way from the one before
        // (the software half: its blocks' middle rows only)
        let ys: Vec<(usize, usize)> = which.iter().enumerate().filter(|&(_, &k)| k > 0).map(|(i, &k)| (RH - 1 - i, k as usize))
            .filter(|&(y, _)| !self.soft.get() || y % 4 == 2).collect();
        let deck: Vec<i32> = ys.iter().flat_map(|&(y, k)| [rows[2 * k - 2], rows[2 * k - 1], rows[2 * k], rows[2 * k + 1], y as i32]).collect();
        let share = self.each(host, "amnum", &[], &deck, 5);
        let mut deck: Vec<i32> = vec![];
        for (j, &(_, k)) in ys.iter().enumerate() {
            let (num, den) = (share[2 * j], share[2 * j + 1]);
            for (p, q) in [(steps[k - 1], steps[k]), (off[2 * k - 2], off[2 * k]), (pools[2 * k - 2], pools[2 * k]), (pools[2 * k - 1], pools[2 * k + 1]),
                           (dash[2 * k - 1], lamps[2 * k + 1])] {
                deck.extend([p, q, num, den]);
            }
        }
        let between = self.each(host, "amlerp", &[], &deck, 4);
        let mut out = vec![None; RH];
        for (i, &k) in which.iter().enumerate() {
            let y = RH - 1 - i;
            if k == 0 { out[y] = Some(Ground { a4: steps[0], off: off[0], pool: pools[0], fk: pools[1], zd: dash[1] }); }
        }
        for (j, &(y, _)) in ys.iter().enumerate() {
            let v = &between[5 * j..5 * j + 5];
            out[y] = Some(Ground { a4: v[0], off: v[1], pool: v[2], fk: v[3], zd: v[4] });
        }
        out
    }
}

/// How much of the frame is drawn (ACCEL_STAGES, as the reference's).
pub fn stages() -> i32 { std::env::var("ACCEL_STAGES").ok().and_then(|v| v.parse().ok()).unwrap_or(6) }

/// The march's rows at each sample column for frame f, as the reference's dump prints them.
pub fn dump(demo: &Path, f: i32) {
    let hd = Hd::new(demo);
    let mut host = Host::new();
    if std::env::var("ACCEL_SOFT").is_ok() {
        use std::io::Write;
        std::io::stdout().write_all(&hd.frame_soft(&mut host, f)).unwrap();
        return;
    }
    if std::env::var("ACCEL_RGB").is_ok() {
        // the frame's pixels (768x384, 8-bit), for checking against the reference's `hd F F+1`
        use std::io::Write;
        let px = hd.frame(&mut host, f);
        std::io::stdout().write_all(&hd.rgb565(&mut host, &px)).unwrap();
        return;
    }
    hd.soft.set(std::env::var("ACCEL_SOFT_DUMP").is_ok());
    let mut cam = hd.camera(&mut host, f);
    let path = hd.jump.path(&hd, &mut host, f);
    (cam.h8, cam.hf, cam.h0) = (path.h8, path.hf, path.h0);
    for s in 0..=24 {
        for (y, g) in hd.march(&mut host, &cam, 32 * s).iter().enumerate() {
            if let Some(g) = g { println!("{} {} {} {} {} {} {}", s, y, g.a4, g.off, g.pool, g.fk, g.zd); }
        }
    }
}

/// The cover edges a partly covered street pixel is measured against: 64ths and 640ths over (the stripe either
/// side of the middle, the edge line, the walk's and kerb's start, the kerb's end, beyond).
const EDGES: [(i32, i32); 7] = [(-8, 0), (8, 0), (364, 0), (380, 0), (384, 0), (393, 6), (576, 0)];

impl Hd {
    /// A one-card-in, one-card-out machine over many cards, each distinct card worked out once.
    fn memo(&self, host: &mut Host, name: &str, cards: &[i32]) -> Vec<i32> {
        let mut distinct: Vec<i32> = cards.to_vec();
        sorter::distinct(&mut distinct, &[sorter::CARD], |&c| [c as i64]);
        let out = self.each(host, name, &[], &distinct, 1);
        let map: HashMap<i32, i32> = distinct.into_iter().zip(out).collect();
        cards.iter().map(|c| map[c]).collect()
    }

    /// Palette colours fogged toward the sky by fk 32nds, made on the mix machine as they are first wanted.
    pub fn fog_all(&self, host: &mut Host, wanted: &[(usize, i32)]) {
        let mut todo: Vec<(usize, i32)> = wanted.iter().copied().filter(|k| !self.fogged.borrow().contains_key(k)).collect();
        sorter::distinct(&mut todo, &[sorter::up(6), sorter::CARD], |&(id, fk)| [id as i64, fk as i64]);
        if todo.is_empty() { return; }
        let sky = self.d.palette[self.d.street["sky"][0]];
        let mut chans = vec![];
        for c in 0..3 {
            let deck: Vec<i32> = todo.iter().flat_map(|&(id, fk)| [1, self.d.palette[id][c], sky[c], fk]).collect();
            chans.push(self.each(host, "amix", &[], &deck, 4));
        }
        let mut f = self.fogged.borrow_mut();
        for (i, &k) in todo.iter().enumerate() { f.insert(k, [chans[0][i], chans[1][i], chans[2][i]]); }
    }

    /// The street on every row (sky where neither sample column sees it): each pixel's colour and distance.
    pub fn street(&self, host: &mut Host, samples: &[Vec<Option<Ground>>], px: &mut [Px]) {
        let d = &self.d;
        let pick = |n: &str, k: usize| d.street[n][k];
        // the dash's places round 480, per sample column
        let wrapped: Vec<Vec<i32>> = samples.iter().map(|col| {
            let v: Vec<i32> = col.iter().map(|g| g.map_or(0, |g| g.zd)).collect();
            self.each(host, "around", &[480], &v, 1)
        }).collect();
        // (pixel index, distance, |across|, light, haze, dash place, what) for the stripe and the partly covered
        struct Todo { i: usize, a4: i32, off: i32, pool: i32, fk: i32, zd: i32, what: i32 }
        let mut todo: Vec<Todo> = vec![];
        let mut interior: Vec<(usize, usize, i32, i32)> = vec![];      // pixel, id, haze, distance
        let soft = self.soft.get();
        // the pixels of a row drawn: every one, or the software half's blocks' middles
        let xs: Vec<usize> = if soft { (0..RW / 4).map(|m| 4 * m + 2).collect() } else { (0..RW).collect() };
        for y in 0..RH {
            if soft && y % 4 != 2 { continue; }
            if samples.iter().all(|c| c[y].is_none()) { continue; }
            let spans = |get: &dyn Fn(&Ground) -> i32, none: i32| -> Vec<i32> {
                (0..24).flat_map(|s| match (samples[s][y], samples[s + 1][y]) {
                    (Some(p), Some(q)) => [get(&p), get(&q)],
                    (Some(p), None) => [get(&p), get(&p)],
                    (None, Some(q)) => [get(&q), get(&q)],
                    (None, None) => [none, none],
                }).collect()
            };
            let row = |host: &mut Host, v: Vec<i32>| -> Vec<i32> {
                if soft { let sp = self.go(host, "asrow4", &[&v]); self.go(host, "asrun", &[&sp]) } else { self.go(host, "asrow", &[&v]) }
            };
            let a4 = row(host, spans(&|g| g.a4, -1));
            let off = row(host, spans(&|g| g.off, 0));
            let pool = row(host, spans(&|g| g.pool, 0));
            let fk = row(host, spans(&|g| g.fk, 0));
            let zs: Vec<i32> = (0..=24).map(|s| match samples[s][y] { Some(_) => wrapped[s][y], None => {
                // a sample without street takes its neighbour's place (its span is the neighbour's alone)
                if s > 0 && samples[s - 1][y].is_some() { wrapped[s - 1][y] } else if s < 24 && samples[s + 1][y].is_some() { wrapped[s + 1][y] } else { 0 }
            } }).collect();
            let gaps = self.go(host, "asgap", &[&zs]);
            if std::env::var("ACCEL_ROW").map(|v| v == y.to_string()).unwrap_or(false) { eprintln!("zs {:?}", &zs[6..10]); eprintln!("gaps7 {:?}", &gaps[42..48]); if soft { let g4 = self.go(host, "asg4", &[&gaps]); eprintln!("g4 {:?}", &g4[42..48]); } }
            let zd = if soft { let g4 = self.go(host, "asg4", &[&gaps]); self.go(host, "asdash", &[&[8], &g4]) } else { self.go(host, "asdash", &[&[32], &gaps]) };
            let deck: Vec<i32> = (0..xs.len()).flat_map(|x| [a4[x], off[x]]).collect();
            let places = self.go(host, "asmat", &[&[xs.len() as i32], &deck]);
            let mut k = 0;
            for (x, &px_x) in xs.iter().enumerate() {
                if places[k] < 0 { k += 1; continue; }
                let (reach, across) = (places[k], places[k + 1]);
                k += 2;
                let i = y * RW + px_x;
                let what = d.cross[reach as usize][across as usize];
                if std::env::var("ACCEL_PIX").map(|v| v == i.to_string()).unwrap_or(false) { eprintln!("ground a4 {} off {} pool {} fk {} zd {} what {} reach {}", a4[x], off[x], pool[x], fk[x], zd[x], what, reach); }
                let p = pool[x] as usize;
                match what {
                    0 => interior.push((i, pick("asphalt", p), fk[x], a4[x])),
                    2 => interior.push((i, pick("edge", 0), fk[x], a4[x])),
                    3 => interior.push((i, pick("kerb", 0), fk[x], a4[x])),
                    4 => interior.push((i, pick("walk", p), fk[x], a4[x])),
                    5 => interior.push((i, pick("beyond", 0), fk[x], a4[x])),
                    _ => todo.push(Todo { i, a4: a4[x], off: across, pool: pool[x], fk: fk[x], zd: zd[x], what }),
                }
            }
        }
        // the dashes: each pixel's length, its cover past the dash's two ends, the difference
        // (only on and by the stripe: elsewhere its cover is 0 whatever the dash)
        let dashed: Vec<usize> = (0..todo.len()).filter(|&j| todo[j].what == 1 || todo[j].what == -2).collect();
        let deck: Vec<i32> = dashed.iter().map(|&j| todo[j].a4).collect();
        // (the software half's edges are hard: a footprint of 1)
        let long = if soft { vec![1; deck.len()] } else { self.memo(host, "aslong", &deck) };
        let deck: Vec<i32> = dashed.iter().enumerate().flat_map(|(n, &j)| [todo[j].zd, 0, long[n], todo[j].zd, 240, long[n]]).collect();
        let ends = self.each(host, "ascut", &[], &deck, 3);
        let deck: Vec<i32> = (0..dashed.len()).flat_map(|n| [ends[2 * n], ends[2 * n + 1], 32]).collect();
        let cuts = self.each(host, "akover", &[], &deck, 3);
        let mut dash = vec![0; todo.len()];
        for (n, &j) in dashed.iter().enumerate() { dash[j] = cuts[n]; }
        // the partly covered: each pixel's width, its covers past the edges, the bands' covers
        let part: Vec<usize> = (0..todo.len()).filter(|&j| todo[j].what < 0).collect();
        let deck: Vec<i32> = part.iter().map(|&j| todo[j].a4).collect();
        let wide = if soft { vec![1; deck.len()] } else { self.memo(host, "aswide", &deck) };
        let mut deck: Vec<i32> = vec![];
        for &j in &part { for &(l, r) in &EDGES { deck.extend([todo[j].off, l, r]); } }
        let past = self.each(host, "asedge", &[], &deck, 3);
        let deck: Vec<i32> = (0..past.len()).flat_map(|k| [past[k], wide[k / 7]]).collect();
        let covers = self.each(host, "ascov", &[], &deck, 2);
        let deck: Vec<i32> = part.iter().enumerate().flat_map(|(n, &j)| {
            let c = &covers[7 * n..7 * n + 7];
            [c[0], c[1], dash[j], c[2], c[3], 32, c[4], c[6], 32, c[4], c[5], 32, c[6], 0, 32]
        }).collect();
        let ks = self.each(host, "akover", &[], &deck, 3);
        // stripe pixels: the stripe's colour at its dash; partly covered: laid on in order, then the haze
        for j in 0..todo.len() { if todo[j].what == 1 { interior.push((todo[j].i, pick("stripe", dash[j] as usize), todo[j].fk, todo[j].a4)); } }
        let sky = d.palette[pick("sky", 0)];
        let mut chans = vec![];
        for c in 0..3 {
            let deck: Vec<i32> = part.iter().enumerate().flat_map(|(n, &j)| {
                let t = &todo[j];
                let p = t.pool as usize;
                let k = &ks[5 * n..5 * n + 5];
                let col = |id: usize| d.palette[id][c];
                [6, col(pick("asphalt", p)), col(pick("stripe", 32)), k[0], col(pick("edge", 0)), k[1], col(pick("walk", p)), k[2],
                 col(pick("kerb", 0)), k[3], col(pick("beyond", 0)), k[4], sky[c], t.fk]
            }).collect();
            chans.push(self.each(host, "amix", &[], &deck, 14));
        }
        for (n, &j) in part.iter().enumerate() { px[todo[j].i] = Px { c: [chans[0][n], chans[1][n], chans[2][n]], a4: todo[j].a4 }; }
        let wanted: Vec<(usize, i32)> = interior.iter().map(|&(_, id, fk, _)| (id, fk)).collect();
        self.fog_all(host, &wanted);
        let f = self.fogged.borrow();
        for &(i, id, fk, a4) in &interior { px[i] = Px { c: f[&(id, fk)], a4 }; }
    }

    /// Frame f of the software half (hd.rs's frame_soft), 192x96 8-bit: the scene at each 4 x 4 block's middle (the
    /// street marched coarser, its edges hard, the walls' nearest texels), what stands in it, the jet, our car drawn
    /// at 192x96, the glows; each channel quantised to the drive's levels through the ordered dither.
    pub fn frame_soft(&self, host: &mut Host, f: i32) -> Vec<u8> {
        self.soft.set(true);
        let mut cam = self.camera(host, f);
        let mut path = self.jump.path(self, host, f);
        (cam.h8, cam.hf, cam.h0) = (path.h8, path.hf, path.h0);
        let sky = Px { c: self.d.palette[self.d.street["sky"][0]], a4: 999 };
        let mut px = vec![sky; RW * RH];
        let samples: Vec<Vec<Option<Ground>>> = (0..=24).map(|s| self.march(host, &cam, 32 * s)).collect();
        self.street(host, &samples, &mut px);
        self.walls(host, &cam, &mut px);
        if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix scene {:?} {}", px[i].c, px[i].a4); }
        let mut lights = self.stand(host, &cam, &mut px);
        if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix stand {:?} {}", px[i].c, px[i].a4); }
        lights.extend(self.jet_lights(host, f));
        self.jet_picture(host, f, &mut px);
        lights.extend(self.jump.car(self, host, &cam, f, &mut path, &mut px));
        if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix before glows {:?} {}", px[i].c, px[i].a4); for l in &lights { eprintln!("light {} {} {} {} {}", 16 * l.x[0] + l.x[1], 16 * l.y[0] + l.y[1], l.a4, l.s, l.kind); } }
        self.glows(host, &lights, &mut px);
        if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix {:?} {}", px[i].c, px[i].a4); }
        // the levels
        const BAYER: [usize; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
        let (w, h) = (RW / 4, RH / 4);
        let mut out = vec![0u8; w * h * 3];
        for c in 0..3 {
            let q = &self.d.softq[11 * c..11 * c + 11];
            let ts: Vec<i32> = (0..w * h).map(|i| px[(4 * (i / w) + 2) * RW + 4 * (i % w) + 2].c[c]).collect();
            let lv = self.each(host, "asql", &q[1], &ts, 1);
            let deck: Vec<i32> = (0..w * h).flat_map(|i| { let b = BAYER[(i / w % 4) * 4 + i % w % 4]; [ts[i], q[2 + lv[i] as usize][b], lv[i]] }).collect();
            let pick = self.each(host, "asqp", &[], &deck, 3);
            for i in 0..w * h { out[3 * i + c] = q[0][pick[i] as usize] as u8; }
        }
        self.soft.set(false);
        out
    }

    /// Frame f of the accelerated half (as far as ACCEL_STAGES says), before the 16-bit dither.
    pub fn frame(&self, host: &mut Host, f: i32) -> Vec<Px> {
        let mut cam = self.camera(host, f);
        // the camera over the jump (accel_jump.rs)
        let mut path = self.jump.path(self, host, f);
        (cam.h8, cam.hf, cam.h0) = (path.h8, path.hf, path.h0);
        let sky = Px { c: self.d.palette[self.d.street["sky"][0]], a4: 999 };
        let mut px = vec![sky; RW * RH];
        let samples: Vec<Vec<Option<Ground>>> = (0..=24).map(|s| self.march(host, &cam, 32 * s)).collect();
        self.street(host, &samples, &mut px);
        if stages() >= 2 { self.walls(host, &cam, &mut px); }
        if stages() < 3 { return px; }
        let mut lights = self.stand(host, &cam, &mut px);
        let mut jet = self.jet_lights(host, f);
        self.jet_picture(host, f, &mut px);
        lights.extend(self.jump.car(self, host, &cam, f, &mut path, &mut px));
        lights.extend(self.jump.sparks(self, host, &cam, f, &mut path, &mut px));
        lights.append(&mut jet);
        if stages() >= 4 { self.glows(host, &lights, &mut px); }
        if stages() >= 5 { self.bloom(host, &mut px); }
        if stages() >= 6 { self.flares(host, &lights, &mut px); }
        px
    }

    /// The 16-bit frame: each channel through the dither machine, a row at a time; the levels as the card's DAC
    /// shows them (8-bit, fetched by level).
    pub fn rgb565(&self, host: &mut Host, px: &[Px]) -> Vec<u8> {
        let mut out = vec![0u8; RW * RH * 3];
        let t = |i: usize| self.d.dither[(i / RW & 3) * 4 + (i % RW & 3)];
        for (c, most) in [31, 63, 31].iter().enumerate() {
            // each distinct channel and threshold worked out once (a machine given the same cards punches the same)
            let mut pairs: Vec<(i32, i32)> = (0..RW * RH).map(|i| (px[i].c[c], t(i))).collect();
            sorter::distinct(&mut pairs, &[sorter::CARD, sorter::CARD], |&(v, t)| [v as i64, t as i64]);
            let mut level: HashMap<(i32, i32), i32> = HashMap::new();
            for piece in pairs.chunks(384) {
                let mut deck: Vec<i32> = piece.iter().flat_map(|&(v, t)| [v, t]).collect();
                deck.resize(768, 0);
                let l = self.go(host, "adither", &[&[*most], &deck]);
                for (k, &p) in piece.iter().enumerate() { level.insert(p, l[k]); }
            }
            for i in 0..RW * RH { out[i * 3 + c] = self.d.dac[c][level[&(px[i].c[c], t(i))] as usize]; }
        }
        out
    }
}

/// A wall a column's ray meets, and what its machines have said of it.
#[derive(Clone, Default)]
struct Wall { x: usize, face: usize, start: i32, met: i32, end: bool, k: usize, qw: i32, ql: i32, den: i32, a4: i32, fk: i32, texel: i32, tenths: i32,
    f: [i32; 2], rows: Vec<[i32; 3]>, bottom: i32 }

impl Hd {
    /// Each column's walls, nearest first, each from the lowest row still open up, laid over the street and sky where
    /// nearer.
    pub fn walls(&self, host: &mut Host, cam: &Cam, px: &mut [Px]) {
        let d = &self.d;
        let pick = |host: &mut Host, start: i32, extra: i32, step: i32, count: i32, table: &[i32]| self.go(host, "dpick", &[&[start, extra, step, count], table]);
        let shifts = pick(host, cam.road_at, 6, 15, 9, &d.road);
        let rows = pick(host, cam.blocks_at, 0, 1, 72, &d.blocks);
        let bases = pick(host, cam.road_at, 6, 1, 135, &d.grade);
        let faces = self.go(host, "dface", &[&[cam.x8], &shifts, &rows]);
        let mut walls: Vec<Wall> = vec![];
        let soft = self.soft.get();
        for x in 0..RW {
            if soft && x % 4 != 2 { continue; }
            let (rx, rz) = (cam.across[x], cam.along[x]);
            let near = self.go(host, "dlat", &[&[rx, rz, cam.minus]]);
            let far = self.go(host, "dlat", &[&[rx, rz, cam.far_end]]);
            let mut deck = vec![cam.o];
            for k in 0..9 { deck.extend([near[k], far[k], faces[10 * k + 1], faces[10 * k + 6]]); }
            let hits = self.go(host, "dhit", &[&deck]);
            for h in hits[..hits.len() - 1].chunks(3) {
                walls.push(Wall { x, face: h[0] as usize, start: h[1], met: h[2], ..Default::default() });
            }
        }
        if walls.is_empty() { return; }
        // each wall's block, its face's number over 10 (10 faces to a block)
        let blocks = self.each(host, "apart", &[], &walls.iter().flat_map(|w| [0, w.face as i32, 1, 10]).collect::<Vec<_>>(), 4);
        for (w, &k) in walls.iter_mut().zip(&blocks) { w.k = k as usize; }
        let ray = |w: &Wall| (cam.across[w.x], cam.along[w.x]);
        let deck: Vec<i32> = walls.iter().flat_map(|w| [ray(w).0, faces[w.face], w.start, w.met]).collect();
        let prep = self.each(host, "awprep", &[], &deck, 4);
        for (n, w) in walls.iter_mut().enumerate() { w.end = prep[4 * n + 3] > 0; }
        let deck: Vec<i32> = walls.iter().flat_map(|w| [ray(w).0, ray(w).1, faces[w.face], w.start, w.end as i32]).collect();
        let q = self.each(host, "awq", &[], &deck, 5);
        for (n, w) in walls.iter_mut().enumerate() { w.qw = q[3 * n]; w.ql = q[3 * n + 1]; w.den = q[3 * n + 2]; }
        let deck: Vec<i32> = walls.iter().flat_map(|w| [w.ql, 0, 44, w.den]).collect();
        let more = self.each(host, "amul", &[], &deck, 4);
        let deck: Vec<i32> = walls.iter().enumerate().flat_map(|(n, w)| [w.qw, more[2 * n]]).collect();
        let far = self.each(host, "awfar", &[], &deck, 2);
        for (n, w) in walls.iter_mut().enumerate() { w.a4 = far[2 * n]; w.fk = far[2 * n + 1]; }
        // where along: faces, then end walls
        let faces_at: Vec<usize> = (0..walls.len()).filter(|&n| !walls[n].end).collect();
        let deck: Vec<i32> = faces_at.iter().flat_map(|&n| [ray(&walls[n]).1, prep[4 * n], prep[4 * n + 1]]).collect();
        let xs = self.each(host, "awx", &[], &deck, 3);
        let deck: Vec<i32> = faces_at.iter().enumerate().flat_map(|(j, &n)| [xs[3 * j + 2], 0, 5, prep[4 * n]]).collect();
        let fifths = self.each(host, "amul", &[], &deck, 4);
        let deck: Vec<i32> = faces_at.iter().enumerate().flat_map(|(j, &n)| [xs[3 * j], xs[3 * j + 1], fifths[2 * j], walls[n].start]).collect();
        let place = self.each(host, "awtf", &[], &deck, 4);
        for (j, &n) in faces_at.iter().enumerate() { walls[n].texel = place[2 * j]; walls[n].tenths = place[2 * j + 1]; }
        let ends_at: Vec<usize> = (0..walls.len()).filter(|&n| walls[n].end).collect();
        let deck: Vec<i32> = ends_at.iter().flat_map(|&n| [walls[n].start, 0, prep[4 * n], ray(&walls[n]).1]).collect();
        let lat = self.each(host, "amul", &[], &deck, 4);
        let deck: Vec<i32> = ends_at.iter().enumerate().flat_map(|(j, &n)| [lat[2 * j + 1], 0, 10, ray(&walls[n]).1]).collect();
        let tens = self.each(host, "amul", &[], &deck, 4);
        let deck: Vec<i32> = ends_at.iter().enumerate().flat_map(|(j, &n)| [lat[2 * j], tens[2 * j], prep[4 * n + 1]]).collect();
        let place = self.each(host, "awte", &[], &deck, 3);
        for (j, &n) in ends_at.iter().enumerate() { walls[n].texel = place[2 * j]; walls[n].tenths = place[2 * j + 1]; }
        // F0: where up each wall the horizon is
        let deck: Vec<i32> = walls.iter().flat_map(|w| [bases[15 * w.k], bases[15 * w.k + 1], cam.h8, cam.hf]).collect();
        let foot = self.each(host, "awfoot", &[], &deck, 4);
        for (n, w) in walls.iter_mut().enumerate() { w.f = [foot[3 * n + 1], foot[3 * n + 2]]; }
        if let Ok(v) = std::env::var("ACCEL_WALL") { for (n, w) in walls.iter().enumerate().filter(|(_, w)| w.x.to_string() == v) { eprintln!("wallq {} f0 {} q {} {} den {} base {}", cam.f, 20 * w.f[0] + w.f[1], w.qw, w.ql, w.den, foot[3 * n]); } }
        // the rows, nearest wall first in each column: the n-th walls of every column together
        let mut open = vec![RH as i32 - 1; RW];
        let mut order: Vec<Vec<usize>> = vec![];
        let mut seen = vec![0usize; RW];
        for (n, w) in walls.iter().enumerate() {
            if order.len() <= seen[w.x] { order.push(vec![]); }
            order[seen[w.x]].push(n);
            seen[w.x] += 1;
        }
        for batch in &order {
            // each wall from the lowest open row where the street is no nearer
            for &n in batch {
                let w = &walls[n];
                if open[w.x] < 0 { continue; }
                let mut deck = vec![w.a4, open[w.x]];
                deck.extend((0..=open[w.x]).rev().map(|y| px[y as usize * RW + w.x].a4));
                let clip = self.go(host, "awclip", &[&deck])[0];
                if clip < 0 { walls[n].bottom = -1; } else { walls[n].bottom = clip; }
            }
            let live: Vec<usize> = batch.iter().copied().filter(|&n| open[walls[n].x] >= 0 && walls[n].bottom >= 0).collect();
            let deck: Vec<i32> = live.iter().flat_map(|&n| { let w = &walls[n]; [w.qw, w.ql, w.den, w.bottom, cam.h0, w.f[0], w.f[1]] }).collect();
            let at = self.each(host, "awalk", &[], &deck, 7);
            let deck: Vec<i32> = live.iter().enumerate().flat_map(|(j, &n)| { let w = &walls[n]; [w.qw, w.ql, w.den, w.bottom, at[3 * j], at[3 * j + 1], at[3 * j + 2], faces[w.face + 2]] }).collect();
            let up = self.each(host, "awup", &[], &deck, 8);
            let mut k = 0;
            for &n in &live {
                let w = &mut walls[n];
                while up[k] > 0 { w.rows.push([up[k + 1], up[k + 2], up[k + 3]]); k += 4; }
                if !w.rows.is_empty() { open[w.x] = up[k + 1]; }
                k += 2;
            }
        }
        if let Ok(v) = std::env::var("ACCEL_WALL") { for w in walls.iter().filter(|w| w.x.to_string() == v) {
            let f = &faces[w.face..w.face + 5];
            eprintln!("wall f {} k {} end {} e {} start {} i0 {} ws {} a4 {} fk {} top {} tex {} tone {}", cam.f, w.k, w.end, f[0], w.start, w.texel, w.tenths, w.a4, w.fk, f[2], f[4], f[3]);
        } }
        // the texels: each wall's two texel columns, its texel rows
        let next = self.each(host, "asum", &[], &walls.iter().flat_map(|w| [w.texel, 1, 1]).collect::<Vec<_>>(), 3);
        let deck: Vec<i32> = walls.iter().zip(&next).flat_map(|(w, &t1)| { let f = &faces[w.face..w.face + 5]; [w.texel, f[4], w.end as i32, t1, f[4], w.end as i32] }).collect();
        let bay = self.each(host, "awbay", &[], &deck, 3);
        let mut items: Vec<(usize, i32)> = vec![];       // (wall, texel row)
        // the software half: at the blocks' middles, the nearest texel (the second column past 5 tenths, the row above
        // past 10 twentieths)
        let mut near: Vec<(usize, usize, i32, usize)> = vec![];    // pixel, wall, texel row, column (0 or 1)
        if soft {
            let rows: Vec<(usize, usize, [i32; 3])> = walls.iter().enumerate().flat_map(|(n, w)| w.rows.iter().enumerate()
                .filter(move |(r, _)| (w.bottom as usize - r) % 4 == 2).map(move |(r, row)| (n, w.bottom as usize - r, *row))).collect();
            let up = self.each(host, "ageq", &[], &rows.iter().flat_map(|r| [r.2[1], 10]).collect::<Vec<_>>(), 2);
            let across = self.each(host, "ageq", &[], &walls.iter().flat_map(|w| [w.tenths, 5]).collect::<Vec<_>>(), 2);
            for (k, &(n, y, row)) in rows.iter().enumerate() { near.push((y * RW + walls[n].x, n, if up[k] == 1 { row[2] } else { row[0] }, across[n] as usize)); }
        }
        for (n, w) in walls.iter().enumerate() {
            let mut js: Vec<i32> = if soft { near.iter().filter(|t| t.1 == n).map(|t| t.2).collect() } else { w.rows.iter().flat_map(|r| [r[0], r[2]]).collect() };
            sorter::distinct(&mut js, &[sorter::CARD], |&j| [j as i64]);
            items.extend(js.into_iter().map(|j| (n, j)));
        }
        let deck: Vec<i32> = items.iter().flat_map(|&(n, j)| { let top = faces[walls[n].face + 2]; [j, top, bay[4 * n], j, top, bay[4 * n + 2]] }).collect();
        let tex = self.each(host, "awtex", &[], &deck, 3);
        // a roof's texel (awtex's place below 0, -1 - its roof): the roof, -1 - place
        let roofs: Vec<(usize, i32)> = (0..tex.len() / 2).filter(|&e| tex[2 * e] < 0).map(|e| (e, tex[2 * e])).collect();
        let roof_no = self.each(host, "asum", &[], &roofs.iter().flat_map(|&(_, p)| [-1, p, -1]).collect::<Vec<_>>(), 3);
        let mut roof = vec![0; tex.len() / 2];
        for (&(e, _), &r) in roofs.iter().zip(&roof_no) { roof[e] = r; }
        // each texel's palette colour, hazed
        let id = |n: usize, side: usize, j: usize| -> usize {
            let w = &walls[n];
            let f = &faces[w.face..w.face + 5];
            let (tone, end) = (f[3] as usize, w.end as usize);
            let (place, up) = (tex[4 * j + 2 * side], tex[4 * j + 2 * side + 1] as usize);
            let across = bay[4 * n + 2 * side + 1] as usize;
            if place < 0 { d.roofs[((tone * 2 + end) * 2 + roof[2 * j + side] as usize) as usize][across * 12 + up] }
            else { d.bays[(tone * 2 + end) * 9 + d.kinds[place as usize] as usize][across * 12 + up] }
        };
        let wanted: Vec<(usize, i32)> = items.iter().enumerate().flat_map(|(j, &(n, _))| [(id(n, 0, j), walls[n].fk), (id(n, 1, j), walls[n].fk)]).collect();
        self.fog_all(host, &wanted);
        let fogged = self.fogged.borrow().clone();
        if soft {
            let at: HashMap<(usize, i32), usize> = items.iter().enumerate().map(|(j, &k)| (k, j)).collect();
            let things: Vec<(usize, i32, [i32; 3])> = near.iter().map(|&(i, n, j, side)| (i, walls[n].a4, fogged[&(id(n, side, at[&(n, j)]), walls[n].fk)])).collect();
            self.lay_nearer(host, px, &things);
            return;
        }
        let mut rowcol: Vec<[i32; 3]> = vec![[0; 3]; items.len()];
        for c in 0..3 {
            // two texels the same are their own blend (a run of one colour, copied); the rest blended
            let ab: Vec<(i32, i32)> = items.iter().enumerate().map(|(j, &(n, _))| (fogged[&(id(n, 0, j), walls[n].fk)][c], fogged[&(id(n, 1, j), walls[n].fk)][c])).collect();
            let differ = self.differ(host, &ab);
            let mixed: Vec<usize> = (0..items.len()).filter(|&j| differ[j]).collect();
            let deck: Vec<i32> = mixed.iter().flat_map(|&j| [ab[j].0, ab[j].1, walls[items[j].0].tenths]).collect();
            let h = self.each(host, "awblend", &[], &deck, 3);
            for j in 0..items.len() { rowcol[j][c] = ab[j].0; }
            for (k, &j) in mixed.iter().enumerate() { rowcol[j][c] = h[k]; }
        }
        let at: HashMap<(usize, i32), usize> = items.iter().enumerate().map(|(j, &k)| (k, j)).collect();
        if let Ok(v) = std::env::var("ACCEL_WALL") { for (n, w) in walls.iter().enumerate().filter(|(_, w)| w.x.to_string() == v) {
            for (r, row) in w.rows.iter().enumerate() { let j = at[&(n, row[0])]; eprintln!("row {} {} {} {} {} {}", cam.f, w.bottom - r as i32, row[0], row[1], id(n, 0, j), id(n, 1, j)); }
        } }
        // each wall's rows: the texel rows under and above, the twentieths up
        let mut seq: Vec<(usize, usize, usize, [i32; 3], [i32; 3], i32)> = vec![];      // pixel, wall, texel rows' run, under, above, w
        // (a new run where a wall starts or where either texel row differs from the row's below)
        let pairs: Vec<(i32, i32)> = walls.iter().flat_map(|w| w.rows.windows(2).flat_map(|p| [(p[1][0], p[0][0]), (p[1][2], p[0][2])])).collect();
        let changed = self.differ(host, &pairs);
        let mut runs = 0;
        let mut at_pair = 0;
        for (n, w) in walls.iter().enumerate() {
            for (r, row) in w.rows.iter().enumerate() {
                let y = w.bottom as usize - r;
                if r == 0 || changed[at_pair] || changed[at_pair + 1] { runs += 1; }
                if r > 0 { at_pair += 2; }
                seq.push((y * RW + w.x, n, runs, rowcol[at[&(n, row[0])]], rowcol[at[&(n, row[2])]], row[1]));
            }
        }
        let mut out = vec![[0i32; 3]; seq.len()];
        for c in 0..3 {
            // rows whose texel rows are the same colour are that colour (copied); the rest by awcol, a run's first
            // row given (new) wherever the deck takes it up
            let differ = self.differ(host, &seq.iter().map(|s| (s.3[c], s.4[c])).collect::<Vec<_>>());
            let kept: Vec<usize> = (0..seq.len()).filter(|&j| differ[j]).collect();
            for j in 0..seq.len() { out[j][c] = seq[j].3[c]; }
            for part in kept.chunks(600) {
                let mut deck = vec![part.len() as i32];
                let mut last = usize::MAX;
                for &j in part {
                    let s = &seq[j];
                    if s.2 != last { deck.extend([1, s.3[c], s.4[c]]); } else { deck.push(0); }
                    last = s.2;
                    deck.push(s.5);
                }
                let v = self.go(host, "awcol", &[&deck]);
                for (k, &val) in v.iter().enumerate() { out[part[k]][c] = val; }
            }
        }
        let things: Vec<(usize, i32, [i32; 3])> = seq.iter().enumerate().map(|(j, s)| (s.0, walls[s.1].a4, out[j])).collect();
        self.lay_nearer(host, px, &things);
    }
}

/// A light (hd.rs's): where (pixel and 16ths across, row and 16ths), distance, its colour's parts of a glow's amount
/// (K, below), strength (500ths), kind (0 a lamp, 1 headlights, 2 tail lamps, 3 the jet's).
#[derive(Clone, Copy, Debug)]
pub struct Light { pub x: [i32; 2], pub y: [i32; 2], pub a4: i32, pub rgb: [i32; 3], pub k: [i32; 3], pub s: i32, pub kind: i32 }

/// 8-bit colours of the lights.
pub const LAMP: [i32; 3] = [255, 230, 170];
pub const GLARE: [i32; 3] = [255, 250, 220];
pub const TAIL: [i32; 3] = [255, 50, 50];
pub const GREEN: [i32; 3] = [60, 255, 90];
pub const WHITE: [i32; 3] = [255, 255, 255];

/// Each colour's part of a glow's amount, K (hd.rs's light_k: 8-bit v 200 / 255 of the channel's tenths a step):
/// the lamps' (255, 230, 170), the headlights' glare (255, 250, 220), the tail lamps' (255, 50, 50), the jet's green
/// (60, 255, 90) and its white.
pub const LAMP_K: [i32; 3] = [243, 445, 162];
pub const GLARE_K: [i32; 3] = [243, 484, 209];
pub const TAIL_K: [i32; 3] = [243, 96, 47];
pub const GREEN_K: [i32; 3] = [57, 494, 85];
pub const WHITE_K: [i32; 3] = [243, 494, 243];
/// Per kind of light (lamp, headlights, tail lamp, the jet's): its glow's radius sc m / d at least least, its
/// strength n / 5, its aspect c (hd.rs's glows).
const GLOW: [[i32; 6]; 5] = [[9, 10, 12, 9, 32, 999], [6, 10, 12, 7, 23, 999], [7, 20, 8, 6, 20, 999], [0, 1, 12, 10, 32, 999], [9, 55, 8, 10, 18, 144]];

impl Hd {
    /// A one-card-a-time machine over many items (each `each` cards in, punching `outs` each).
    fn many(&self, host: &mut Host, name: &str, head: &[i32], items: &[Vec<i32>]) -> Vec<Vec<i32>> {
        if items.is_empty() { return vec![]; }
        let each = items[0].len();
        let flat: Vec<i32> = items.iter().flatten().copied().collect();
        let out = self.each(host, name, head, &flat, each);
        let outs = out.len() / items.len();
        out.chunks(outs).map(|c| c.to_vec()).collect()
    }

    /// Whether a pixel is drawn: every one, or in the software half each 4 x 4 block's middle.
    pub fn shown(&self, i: usize) -> bool { !self.soft.get() || (i % RW % 4 == 2 && i / RW % 4 == 2) }

    /// `go` and `var_each` for the jump's module.
    pub fn go_pub(&self, host: &mut Host, name: &str, deck: &[&[i32]]) -> Vec<i32> { self.go(host, name, deck) }
    pub fn var_each_pub(&self, host: &mut Host, name: &str, items: &[Vec<i32>]) -> Vec<i32> { self.var_each(host, name, &[], items, 1) }

    /// `many` for the jump's module.
    pub fn many_pub(&self, host: &mut Host, name: &str, items: &[Vec<i32>]) -> Vec<Vec<i32>> { self.many(host, name, &[], items) }

    /// Whether each pair's two cards differ: their difference on asum, against 0.
    pub fn differ(&self, host: &mut Host, pairs: &[(i32, i32)]) -> Vec<bool> {
        if pairs.is_empty() { return vec![]; }
        let deck: Vec<i32> = pairs.iter().flat_map(|&(a, b)| [a, b, -1]).collect();
        self.each(host, "asum", &[], &deck, 3).into_iter().map(|d| d != 0).collect()
    }

    /// Lay things over pixels where they are nearer, in the order given: each thing (its pixel, its distance, its
    /// colour) replaces what the pixel shows if it is nearer, so a later thing goes over an earlier one only where
    /// nearer still. anear picks each pixel's winner; the host groups the things by pixel, keeping their order, and
    /// copies the winners in.
    fn lay_nearer(&self, host: &mut Host, px: &mut [Px], things: &[(usize, i32, [i32; 3])]) {
        let mut groups: Vec<(usize, Vec<usize>)> = vec![];
        let mut at: HashMap<usize, usize> = HashMap::new();
        for (t, &(i, _, _)) in things.iter().enumerate() {
            let g = *at.entry(i).or_insert_with(|| { groups.push((i, vec![])); groups.len() - 1 });
            groups[g].1.push(t);
        }
        // whole pixels a run, its deck under 1999 cards
        let mut start = 0;
        while start < groups.len() {
            let (mut deck, mut end) = (vec![0], start);
            while end < groups.len() && deck.len() + 2 + groups[end].1.len() <= 1990 {
                let (i, ts) = &groups[end];
                deck.extend([px[*i].a4, ts.len() as i32]);
                deck.extend(ts.iter().map(|&t| things[t].1));
                end += 1;
            }
            assert!(end > start, "a pixel with more things than a deck holds");
            deck[0] = (end - start) as i32;
            let won = self.go(host, "anear", &[&deck]);
            for (g, &w) in (start..end).zip(&won) {
                if w > 0 { let t = groups[g].1[w as usize - 1]; px[groups[g].0] = Px { c: things[t].2, a4: things[t].1 }; }
            }
            start = end;
        }
    }

    /// The colour channels of many pixels moved toward the sky by k 32nds `layers` times (each done once).
    fn hazed(&self, host: &mut Host, wanted: &[([i32; 3], i32, i32)]) -> HashMap<([i32; 3], i32, i32), [i32; 3]> {
        let mut todo: Vec<([i32; 3], i32, i32)> = wanted.to_vec();
        sorter::distinct(&mut todo, &[sorter::CARD; 5], |&(c, a, b)| [c[0] as i64, c[1] as i64, c[2] as i64, a as i64, b as i64]);
        let sky = self.d.palette[self.d.street["sky"][0]];
        let mut out: HashMap<([i32; 3], i32, i32), [i32; 3]> = HashMap::new();
        let mut chans = vec![];
        for c in 0..3 {
            let deck: Vec<i32> = todo.iter().flat_map(|&(v, k, n)| { let mut d = vec![n, v[c]]; for _ in 0..n { d.extend([sky[c], k]); } d }).collect();
            // items vary in length (the layers): run them in pieces by hand
            let mut res = vec![];
            let mut start = 0;
            while start < todo.len() {
                let mut end = start;
                let mut cards = 0;
                while end < todo.len() && cards + 2 + 2 * todo[end].2 as usize <= 1900 { cards += 2 + 2 * todo[end].2 as usize; end += 1; }
                let part: Vec<i32> = todo[start..end].iter().flat_map(|&(v, k, n)| { let mut d = vec![n, v[c]]; for _ in 0..n { d.extend([sky[c], k]); } d }).collect();
                res.extend(self.go(host, "amix", &[&[(end - start) as i32], &part]));
                start = end;
            }
            let _ = deck;
            chans.push(res);
        }
        for (i, &k) in todo.iter().enumerate() { out.insert(k, [chans[0][i], chans[1][i], chans[2][i]]); }
        out
    }

    /// Paint pixels where the thing is nearer: (pixel index, the thing's distance, cover, colour).
    fn paint(&self, host: &mut Host, px: &mut [Px], items: &[(usize, i32, i32, [i32; 3])]) {
        // (the software half: the blocks' middles only)
        let kept: Vec<(usize, i32, i32, [i32; 3])>;
        let items = if self.soft.get() { kept = items.iter().copied().filter(|t| self.shown(t.0)).collect(); &kept[..] } else { items };
        if items.is_empty() { return; }
        let mut a4 = vec![0; items.len()];
        let mut cols = vec![[0; 3]; items.len()];
        for c in 0..3 {
            let deck: Vec<Vec<i32>> = items.iter().map(|&(i, a, k, col)| vec![px[i].a4, a, k, px[i].c[c], col[c]]).collect();
            let out = self.many(host, "apaint", &[], &deck);
            for (j, o) in out.iter().enumerate() { cols[j][c] = o[0]; a4[j] = o[1]; }
        }
        for (j, &(i, _, _, _)) in items.iter().enumerate() { px[i] = Px { c: cols[j], a4: a4[j] }; }
    }

    /// The lamp posts and the traffic, far to near, each over what is further; the lights they carry.
    pub fn stand(&self, host: &mut Host, cam: &Cam, px: &mut [Px]) -> Vec<Light> {
        let d = &self.d;
        let block = cam.road_at / 15;
        let _ = block;
        let pick = |host: &mut Host, start: i32, extra: i32, step: i32, count: i32, table: &[i32]| self.go(host, "dpick", &[&[start, extra, step, count], table]);
        let sevens = pick(host, cam.road_at, 7, 15, 7, &d.road);
        let eights = pick(host, cam.road_at, 8, 15, 7, &d.road);
        let pairs: Vec<i32> = (0..7).flat_map(|k| [sevens[k], eights[k]]).collect();
        let lamps = self.go(host, "dlamp", &[&[cam.o, cam.x8], &pairs]);
        let ncars = d.cars.len() / 6;
        let deck: Vec<i32> = d.cars.chunks(6).flat_map(|c| [c[0], c[1]]).collect();
        let traffic = self.go(host, "acarz", &[&[cam.f, cam.into, cam.entry, ncars as i32], &deck]);
        // (across eighths, ahead quarters, what: -1 left lamp, -2 right, else the car)
        let mut things: Vec<(i32, i32, i32)> = (0..14).map(|k| (lamps[2 * k], lamps[2 * k + 1], if k % 2 == 0 { -1 } else { -2 })).collect();
        for c in 0..ncars {
            let t = &traffic[3 * c..3 * c + 3];
            let dx = self.go(host, "dcarx", &[&[t[2], d.road[t[1] as usize], d.road[t[1] as usize + 1], d.cars[6 * c + 2], cam.x8]])[0];
            things.push((dx, t[0], c as i32));
        }
        // turned to the view
        let mut seen: Vec<(i32, i32, i32, i32, i32)> = vec![];    // (a4, |across|, side, dz, what)
        for &(dx, dz, what) in &things {
            let p1 = self.go(host, "dmul", &[&[dx, cam.c, 400]])[0];
            let p2 = self.go(host, "dmul", &[&[dz, cam.s, 200]])[0];
            let q1 = self.go(host, "dmul", &[&[dx, cam.s, 400]])[0];
            let q2 = self.go(host, "dmul", &[&[dz, cam.c, 400]])[0];
            let a4 = self.go(host, "dalong", &[&[q1, q2]])[0];
            let v = self.go(host, "athvis", &[&[1, p1, p2, a4]]);
            if v[2] > 0 { seen.push((a4, v[0], v[1], dz, what)); }
        }
        sorter::sort(&mut seen, &[sorter::down(6)], |t| [t.0 as i64]);
        let n = seen.len();
        let mut lights = vec![];
        if n == 0 { return lights; }
        let q = self.many(host, "amul", &[], &seen.iter().map(|t| vec![t.1, 0, 220, t.0]).collect::<Vec<_>>());
        let q2 = self.many(host, "amul", &[], &seen.iter().enumerate().map(|(j, t)| vec![q[j][1], 0, 16, t.0]).collect::<Vec<_>>());
        let xs = self.many(host, "athx", &[], &seen.iter().enumerate().map(|(j, t)| vec![q[j][0], q2[j][0], t.2]).collect::<Vec<_>>());
        let sc = self.many(host, "amul", &[], &seen.iter().map(|t| vec![22, 0, 320, t.0]).collect::<Vec<_>>());
        let fk = self.many(host, "afog", &[], &seen.iter().map(|t| vec![t.0]).collect::<Vec<_>>());
        let at = self.many(host, "aweight", &[], &seen.iter().map(|t| vec![cam.into, t.3]).collect::<Vec<_>>());
        let g = |j: usize| (d.grade[(cam.entry + at[j][0]) as usize], d.grade[(cam.entry + at[j][0]) as usize + 1]);
        let feet = self.many(host, "ay8", &[], &(0..n).map(|j| vec![g(j).0, g(j).1, at[j][1], 0]).collect::<Vec<_>>());
        let tops = self.many(host, "ay8", &[], &(0..n).map(|j| vec![g(j).0, g(j).1, at[j][1], 40]).collect::<Vec<_>>());
        // the rows of the feet and the tops, as the march's
        let rows = |host: &mut Host, ys: &[Vec<i32>]| -> Vec<Vec<i32>> {
            let q1 = self.many(host, "amul", &[], &(0..n).map(|j| vec![cam.h8, ys[j][0], 220, seen[j].0]).collect::<Vec<_>>());
            let q2 = self.many(host, "amul", &[], &(0..n).map(|j| vec![q1[j][1], 0, 16, seen[j].0]).collect::<Vec<_>>());
            let q3 = self.many(host, "amul", &[], &(0..n).map(|j| vec![cam.hf, ys[j][1], 110, seen[j].0]).collect::<Vec<_>>());
            let q4 = self.many(host, "amul", &[], &(0..n).map(|j| vec![q3[j][1], 0, 4, seen[j].0]).collect::<Vec<_>>());
            let deck: Vec<i32> = (0..n).flat_map(|j| [q1[j][0], q2[j][0], q3[j][0], q4[j][0]]).collect();
            self.go(host, "amrow", &[&[cam.h0, -200, 900, n as i32], &deck]).chunks(2).map(|c| c.to_vec()).collect()
        };
        let foot = rows(host, &feet);
        let top = rows(host, &tops);
        let sky = d.palette[d.street["sky"][0]];
        let post = d.palette[d.street["post"][0]];
        for j in 0..n {
            let (a4, _, _, _, what) = seen[j];
            let (x, xf, sc4, k) = (xs[j][0], xs[j][1], sc[j][0], fk[j][0]);
            if std::env::var("ACCEL_THINGS").is_ok() { eprintln!("thing {what} a4 {a4} x16 {} sc4 {sc4} fk {k} foot {:?} top {:?}", 16 * x + xf, foot[j], top[j]); }
            if what < 0 {
                let side = if what == -1 { -1 } else { 1 };
                let b = self.go(host, "apost1", &[&[1, x, xf, sc4, top[j][0], foot[j][0]]]);
                let (hw, x0, x1, y0, y1) = (b[0], b[1], b[2], b[3], b[4]);
                // the body's columns and rows, drawn when there are some of each
                let size = self.go(host, "asum", &[&[2, x1, x0, -1, y1, y0, -1]]);
                let ncols = size[0];
                if ncols > 0 && size[1] > 0 {
                    let cols = self.go(host, "apcol", &[&[1, x, xf, hw, x0, x1]]);
                    let shades = self.many(host, "apshade", &[], &(0..ncols as usize).map(|c| vec![cols[2 * c + 1], hw, side]).collect::<Vec<_>>());
                    let deck: Vec<i32> = shades.iter().map(|s| s[0]).collect();
                    let shaded = self.go(host, "ashade", &[&[post[0], post[1], post[2], deck.len() as i32], &deck]);
                    let colours: Vec<[i32; 3]> = shaded.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
                    let haze = self.hazed(host, &colours.iter().map(|&c| (c, k, 2)).collect::<Vec<_>>());
                    let mut items = vec![];
                    for c in 0..ncols as usize {
                        let cov = cols[2 * c];
                        if cov == 0 { continue; }
                        for yy in y0..y1 { items.push(((yy as usize) * RW + x0 as usize + c, a4, cov, haze[&(colours[c], k, 2)])); }
                    }
                    self.paint(host, px, &items);
                }
                let head = self.go(host, "apost2", &[&[1, x, xf, sc4, side]]);
                let arm_cols = self.go(host, "apost3", &[&[1, x, head[0]]]);
                let arm_rows = self.go(host, "apost4", &[&[1, sc4, top[j][0], top[j][1]]]);
                let hazed = self.hazed(host, &[(post, k, 1)])[&(post, k, 1)];
                let arm = self.go(host, "a810", &[&[3], &hazed]);
                let mut cells = vec![];
                for yy in arm_rows[0]..=arm_rows[1] { for xx in arm_cols[0]..arm_cols[1] { cells.push((yy as usize) * RW + xx as usize); } }
                cells.retain(|&i| self.shown(i));
                if !cells.is_empty() {
                    let hz = self.many(host, "afog", &[], &cells.iter().map(|&i| vec![px[i].a4]).collect::<Vec<_>>());
                    let mut out = vec![[0; 3]; cells.len()];
                    for c in 0..3 {
                        let deck: Vec<Vec<i32>> = cells.iter().enumerate().map(|(m, &i)| vec![px[i].a4, a4, hz[m][0], px[i].c[c], arm[c]]).collect();
                        let o = self.many(host, "aarm", &[sky[c]], &deck);
                        for m in 0..cells.len() { out[m][c] = o[m][0]; }
                    }
                    for (m, &i) in cells.iter().enumerate() { px[i].c = out[m]; }
                }
                // the light, if the head's pixel shows the post's distance (or further)
                let (hx, hy) = (head[0], top[j][0]);
                // (the software half looks at the pixel it shows there: its block's middle)
                let (sx, sy) = if self.soft.get() { (4 * (hx / 4) + 2, 4 * (hy / 4) + 2) } else { (hx, hy) };
                let there = if (0..RW as i32).contains(&hx) && (0..RH as i32).contains(&hy) { px[sy as usize * RW + sx as usize].a4 } else { -1 };
                let s = self.go(host, "alamp", &[&[1, there, a4, k, sc4]])[0];
                if s > 0 { lights.push(Light { x: [head[0], head[1]], y: [arm_rows[2], arm_rows[3]], a4, rgb: LAMP, k: LAMP_K, s, kind: 0 }); }
            } else {
                let car = &d.cars[6 * what as usize..6 * what as usize + 6];
                let (art, body) = (car[3], car[4]);
                let half = self.go(host, "amul", &[&[1, sc4, 0, 1, 2]])[0];
                let w = self.go(host, "apair", &[&[1, sc4, 9, 0]]);
                let h = self.go(host, "apair", &[&[1, sc4, 3, half]]);
                let tw = self.go(host, "atwth", &[&[2, w[0], w[1], h[0], h[1]]]);
                if tw[1] == 0 { continue; }
                let (tw, th) = (tw[0], tw[2]);
                let corner = self.go(host, "acarxy", &[&[1, x, xf, foot[j][0], foot[j][1], w[0], w[1], h[0], h[1]]]);
                // the smallest picture made at least tw wide (a picture is chosen by the width the machines punched)
                let mut best = None;
                for t in &d.traffic { if t[0] == art && t[1] == body { best = Some(*t); if t[2] >= tw { break; } } }
                let [_, _, pw, ph, start] = best.unwrap();
                let us = self.go(host, "asamp", &[&[1, tw, pw]]);
                let vs = self.go(host, "asamp", &[&[1, th, ph]]);
                let mut items = vec![];
                let mut want = vec![];
                for v in 0..th { for u in 0..tw {
                    let (xx, yy) = (corner[0] + u, corner[2] + v);
                    if !(0..RW as i32).contains(&xx) || !(0..RH as i32).contains(&yy) { continue; }
                    let k8 = start as usize + 8 * (vs[v as usize] * pw + us[u as usize]) as usize;
                    let get = |i: usize| u16::from_le_bytes([d.pictures[k8 + 2 * i], d.pictures[k8 + 2 * i + 1]]) as i32;
                    if get(3) == 0 { continue; }
                    let c = [get(0), get(1), get(2)];
                    want.push((c, k, 2));
                    items.push((yy as usize * RW + xx as usize, a4, get(3), c));
                }}
                let haze = self.hazed(host, &want);
                let items: Vec<(usize, i32, i32, [i32; 3])> = items.into_iter().map(|(i, a, cov, c)| (i, a, cov, haze[&(c, k, 2)])).collect();
                self.paint(host, px, &items);
                // its lamps: half way down, 3 and 13 16ths across
                let hh = self.go(host, "ahalf", &[&[1, h[0], h[1]]]);
                let ly = self.go(host, "aadd2", &[&[1, corner[2], corner[3], hh[0], hh[1], 0, 1]]);
                let level = if art == 1 { self.go(host, "alin", &[&[1, 175, sc4, 5, 650]])[0] } else {
                    let t12 = self.go(host, "amul", &[&[1, sc4, 0, 1, 12]])[0];
                    let base = self.go(host, "asum", &[&[1, 75, t12, 1]])[0];
                    self.go(host, "alin", &[&[1, base, sc4, 2, 300]])[0]
                };
                let clear = self.go(host, "asum", &[&[1, 32, k, -1]])[0];
                let s = self.go(host, "akover", &[&[1, level, 0, clear]])[0];
                for nth in [3, 13] {
                    let extra = self.go(host, "amul", &[&[1, w[1], 0, nth, 16]])[0];
                    let off = self.go(host, "apair", &[&[1, w[0], nth, extra]]);
                    let lx = self.go(host, "aadd2", &[&[1, corner[0], corner[1], off[0], off[1], 0, 1]]);
                    if lx[0] == 999 { continue; }
                    lights.push(Light { x: [lx[0], lx[1]], y: [ly[0], ly[1]], a4, rgb: if art == 1 { GLARE } else { TAIL }, k: if art == 1 { GLARE_K } else { TAIL_K }, s, kind: if art == 1 { 1 } else { 2 } });
                }
            }
        }
        lights
    }
}

impl Hd {
    /// A machine over items of any number of cards (a count, then the items), in runs of at most 1900 cards,
    /// punching `outs` cards an item.
    fn var_each(&self, host: &mut Host, name: &str, head: &[i32], items: &[Vec<i32>], outs: usize) -> Vec<i32> {
        if ONE_AT_A_TIME.contains(&name) && items.len() > 1 {
            let mut distinct: Vec<&Vec<i32>> = items.iter().collect();
            let longest = items.iter().map(|it| it.len()).max().unwrap_or(0);
            sorter::distinct(&mut distinct, &vec![sorter::CARD; longest], |d| d.iter().map(|&c| c as i64).collect::<Vec<_>>());
            if distinct.len() < items.len() {
                let owned: Vec<Vec<i32>> = distinct.iter().map(|d| (*d).clone()).collect();
                let got = self.var_each_all(host, name, head, &owned, outs);
                let at: HashMap<&Vec<i32>, usize> = distinct.iter().enumerate().map(|(k, &d)| (d, k)).collect();
                return items.iter().flat_map(|it| got[outs * at[it]..outs * at[it] + outs].iter().copied()).collect();
            }
        }
        self.var_each_all(host, name, head, items, outs)
    }

    fn var_each_all(&self, host: &mut Host, name: &str, head: &[i32], items: &[Vec<i32>], outs: usize) -> Vec<i32> {
        let mut out = vec![];
        let mut start = 0;
        while start < items.len() {
            let mut end = start;
            let mut cards = 0;
            while end < items.len() && (end == start || cards + items[end].len() <= 1900) && (end - start) * outs < 1900 { cards += items[end].len(); end += 1; }
            let mut deck = head.to_vec();
            deck.push((end - start) as i32);
            for it in &items[start..end] { deck.extend(it); }
            out.extend(self.go(host, name, &[&deck]));
            start = end;
        }
        out
    }

    /// The jet's lamps (hd.rs's), while it is in the strip of sky.
    pub fn jet_lights(&self, host: &mut Host, f: i32) -> Vec<Light> {
        let flags = self.go(host, "ajflags", &[&[f]]);
        if flags[0] == 0 { return vec![]; }
        let g = self.jet(host, f);
        let (ix, top, w, h) = (g[0], [g[1], g[2]], g[3], g[4]);
        let mut lights = vec![];
        if flags[1] == 1 {
            let red = self.go(host, "amul", &[&[1, w, 0, 32, 100]])[0];
            let green = self.go(host, "amul", &[&[1, w, 0, 68, 100]])[0];
            let green = self.go(host, "apair", &[&[1, w, 15, green]]);
            let down = self.go(host, "amul", &[&[1, h, 0, 96, 10]])[0];
            let at = self.go(host, "aadd2", &[&[3, ix, 0, 0, 0, red, 1, ix, 0, green[0], green[1], 0, 1, top[0], top[1], 0, 0, down, 1]]);
            lights.push(Light { x: [at[0], at[1]], y: [at[4], at[5]], a4: 999, rgb: TAIL, k: TAIL_K, s: 250, kind: 3 });
            lights.push(Light { x: [at[2], at[3]], y: [at[4], at[5]], a4: 999, rgb: GREEN, k: GREEN_K, s: 250, kind: 3 });
        }
        if flags[2] == 1 {
            let across = self.go(host, "apair", &[&[2, w, 8, 0, h, 8, 0]]);
            let at = self.go(host, "aadd2", &[&[2, ix, 0, across[0], across[1], 0, 1, top[0], top[1], across[2], across[3], 0, 1]]);
            lights.push(Light { x: [at[0], at[1]], y: [at[2], at[3]], a4: 999, rgb: WHITE, k: WHITE_K, s: 350, kind: 3 });
        }
        lights
    }

    /// The jet's picture over the sky (hd.rs's jet): its pixels laid on as a thing at 998 quarters.
    pub fn jet_picture(&self, host: &mut Host, f: i32, px: &mut [Px]) {
        if self.go(host, "ajflags", &[&[f]])[0] == 0 { return; }
        let g = self.jet(host, f);
        let k = self.go(host, "asum", &[&[1, f, 103, -1]])[0];
        let (w, h, pic) = &self.d.jet[k as usize - 1];
        let (ix, iy) = (g[0], g[1]);
        let mut items = vec![];
        for yy in 0..*h { for xx in 0..*w {
            let (x, y) = (ix + xx, iy + yy);
            if !(0..RW as i32).contains(&x) || !(0..RH as i32).contains(&y) { continue; }
            let c = pic[(yy * w + xx) as usize];
            if c[3] == 0 { continue; }
            items.push((y as usize * RW + x as usize, 998, c[3], [c[0], c[1], c[2]]));
        } }
        self.paint(host, px, &items);
    }

    /// The jet's picture's place (hd.rs's): its left pixel, its top (pixel, 16ths), its width and height (pixels).
    pub fn jet(&self, host: &mut Host, f: i32) -> [i32; 5] {
        let k = self.go(host, "asum", &[&[1, f, 103, -1]])[0];
        let kk = self.go(host, "ajk", &[&[k]]);
        let size = self.go(host, "asum", &[&[1, 64, kk[0], 1]])[0];
        let t = self.go(host, "amul", &[&[1, size, 0, 9, 16]])[0];
        let x = self.go(host, "amul", &[&[1, k, 0, 160, 9]])[0];
        let top = self.go(host, "aadd2", &[&[1, 120, 0, kk[1], kk[2], t, -1]]);
        let ix = self.go(host, "aadd2", &[&[1, 368, 0, 0, x, 0, 1]]);
        let ix = self.go(host, "aadd2", &[&[1, ix[0], ix[1], 0, size, 0, -1]]);
        let w = self.go(host, "amul", &[&[1, size, 0, 1, 8]])[0];
        let h = self.go(host, "amul", &[&[1, size, 0, 7, 128]])[0];
        [ix[0], top[0], top[1], w, h]
    }

    /// Each light's glow added (hd.rs's glows): its profile's table times its amount, laid on the pixels round it by
    /// how far out each is (an octagon), every pixel's glows summed.
    pub fn glows(&self, host: &mut Host, lights: &[Light], px: &mut [Px]) {
        if std::env::var("ACCEL_LIGHTS").is_ok() { for l in lights { eprintln!("light {} {} a4 {} s {} {}", 16 * l.x[0] + l.x[1], 16 * l.y[0] + l.y[1], l.a4, l.s, l.kind); } }
        let lit: Vec<&Light> = lights.iter().filter(|l| l.s > 10).collect();
        if lit.is_empty() { return; }
        let sc = self.many(host, "amul", &[], &lit.iter().map(|l| vec![22, 0, 320, l.a4]).collect::<Vec<_>>());
        let par = self.many(host, "agparam", &[], &lit.iter().enumerate().map(|(j, l)| {
            let g = GLOW[l.kind as usize];
            vec![sc[j][0], g[0], g[1], g[2], g[5], l.s, g[3]]
        }).collect::<Vec<_>>());
        let amt = self.many(host, "aamt", &[], &lit.iter().enumerate().map(|(j, l)| vec![par[j][1], l.k[0], l.k[1], l.k[2]]).collect::<Vec<_>>());
        // the tables, each amount once
        {
            let mut want: Vec<i32> = amt.iter().flatten().copied().filter(|a| !self.tables.borrow().contains_key(a)).collect();
            sorter::distinct(&mut want, &[sorter::CARD], |&a| [a as i64]);
            for a in want {
                let t = self.go(host, "agtab", &[&[a, self.d.glowt.len() as i32], &self.d.glowt]);
                self.tables.borrow_mut().insert(a, t);
            }
        }
        let tables = self.tables.borrow();
        let mut adds: Vec<[Vec<i32>; 3]> = vec![Default::default(); px.len()];
        let axis = |host: &mut Host, k: i32, r: i32, c: i32, size: i32| -> (i32, Vec<(i32, i32)>) {
            // the pixels out either way that can land on the screen
            let from = if c < 0 { -c } else if c >= size { c - size + 1 } else { 0 };
            // (a glow reaches at most 6 radii, under 800 pixels: none further is wanted, and a card holds the count)
            let to = (c + 1).max(size - c).min(999);
            let mut v = vec![];
            let mut a = from;
            while a < to {
                let b = (a + 900).min(to);
                let o = self.go(host, "agaxis", &[&[k, r, a, b]]);
                let got = o.len() / 2;
                v.extend(o.chunks(2).map(|p| (p[0], p[1])));
                if got < (b - a) as usize { break; }
                a = b;
            }
            (from, v)
        };
        for (j, l) in lit.iter().enumerate() {
            let (r4, c) = (par[j][0], GLOW[l.kind as usize][4]);
            let tab = [&tables[&amt[j][0]], &tables[&amt[j][1]], &tables[&amt[j][2]]];
            let (cx, cy) = (l.x[0], l.y[0]);
            let (x0, xs) = axis(host, 4 * c, r4, cx, RW as i32);
            let (y0, ys) = axis(host, 128, r4, cy, RH as i32);
            if xs.is_empty() { continue; }
            for (n, &(ty, ey)) in ys.iter().enumerate() {
                let dy = y0 + n as i32;
                let rows: Vec<i32> = [cy + dy, cy - dy].into_iter().enumerate().filter(|&(m, y)| !(m == 1 && dy == 0) && (0..RH as i32).contains(&y))
                    .map(|(_, y)| y).filter(|&y| !self.soft.get() || y % 4 == 2).collect();
                if rows.is_empty() { continue; }
                let mut ts = vec![];
                for piece in xs.chunks(900) {
                    let deck: Vec<i32> = piece.iter().flat_map(|&(tx, ex)| [tx, ex]).collect();
                    ts.extend(self.go(host, "agpt", &[&[ty, ey, piece.len() as i32], &deck]));
                }
                for (m, &t) in ts.iter().enumerate() {
                    if t > 192 { continue; }
                    let dx = x0 + m as i32;
                    let v = [tab[0][t as usize], tab[1][t as usize], tab[2][t as usize]];
                    if v == [0, 0, 0] { continue; }
                    for &y in &rows {
                        for (side, x) in [(0, cx + dx), (1, cx - dx)] {
                            if side == 1 && dx == 0 { continue; }
                            if !(0..RW as i32).contains(&x) { continue; }
                            if !self.shown(y as usize * RW + x as usize) { continue; }
                            let a = &mut adds[y as usize * RW + x as usize];
                            for ch in 0..3 { if v[ch] > 0 { a[ch].push(v[ch]); } }
                        }
                    }
                }
            }
        }
        for ch in 0..3 {
            let at: Vec<usize> = (0..px.len()).filter(|&i| !adds[i][ch].is_empty()).collect();
            let items: Vec<Vec<i32>> = at.iter().map(|&i| { let mut it = vec![px[i].c[ch], adds[i][ch].len() as i32]; it.extend(&adds[i][ch]); it }).collect();
            let out = self.var_each(host, "agsum", &[], &items, 1);
            for (n, &i) in at.iter().enumerate() { px[i].c[ch] = out[n]; }
        }
    }
}

impl Hd {
    /// Three boxes of 2 r + 1 down the columns, then three along the rows, of the 192 x 96 blocks (hd.rs's blur): a
    /// line of nothing but 0 stays so.
    fn blur(&self, host: &mut Host, small: &[i32], r: i64) -> Vec<i32> {
        let (w, h) = (192usize, 96usize);
        let n = 2 * r + 1;
        let mut b = small.to_vec();
        for axis in 0..2 { for _ in 0..3 {
            let (lines, len) = if axis == 0 { (w, h) } else { (h, w) };
            let at = |l: usize, k: i64| -> usize { let k = k.clamp(0, len as i64 - 1) as usize; if axis == 0 { k * w + l } else { l * w + k } };
            let mut items = vec![];
            let mut which = vec![];
            for l in 0..lines {
                if (0..len as i64).all(|k| b[at(l, k)] == 0) { continue; }
                let mut it: Vec<i32> = (-r..=r).map(|k| b[at(l, k)]).collect();
                it.push(len as i32);
                for k in 0..len as i64 { it.push(b[at(l, k + r + 1)]); it.push(b[at(l, k - r)]); }
                items.push(it);
                which.push(l);
            }
            let out = self.var_each(host, "abox", &[n as i32], &items, len);
            let mut next = vec![0; w * h];
            for (j, &l) in which.iter().enumerate() { for k in 0..len { next[at(l, k as i64)] = out[j * len + k]; } }
            b = next;
        } }
        b
    }

    /// The bloom (hd.rs's): each 4 x 4 block's top-left pixel past bright, blurred by boxes of 7 and of 19, laid on
    /// its block.
    pub fn bloom(&self, host: &mut Host, px: &mut [Px]) {
        const T: [i32; 3] = [182, 371, 182];
        let (w, h) = (192usize, 96usize);
        for c in 0..3 {
            let tl: Vec<i32> = (0..w * h).map(|i| px[(i / w) * 4 * RW + (i % w) * 4].c[c]).collect();
            let small = self.each(host, "abthr", &[T[c]], &tl, 1);
            let (b3, b9) = (self.blur(host, &small, 3), self.blur(host, &small, 9));
            let some: Vec<usize> = (0..w * h).filter(|&i| b3[i] > 0 || b9[i] > 0).collect();
            let amt = self.each(host, "abmix", &[], &some.iter().flat_map(|&i| [b3[i], b9[i]]).collect::<Vec<_>>(), 2);
            let blocks: Vec<(usize, i32)> = some.iter().zip(&amt).filter(|(_, &a)| a > 0).map(|(&i, &a)| (i, a)).collect();
            let pixels = |i: usize| -> Vec<usize> { let (bx, by) = (i % w, i / w); (0..16).map(|k| (4 * by + k / 4) * RW + 4 * bx + k % 4).collect() };
            let items: Vec<Vec<i32>> = blocks.iter().map(|&(i, a)| { let mut it = vec![a, 16]; it.extend(pixels(i).iter().map(|&p| px[p].c[c])); it }).collect();
            let out = self.var_each(host, "abadd", &[], &items, 16);
            for (j, &(i, _)) in blocks.iter().enumerate() { for (k, p) in pixels(i).into_iter().enumerate() { px[p].c[c] = out[16 * j + k]; } }
        }
    }
}

/// The streak's colour (thousandths of phi), the star's (hd.rs's star_k of a lamp's and of headlights'), the
/// channels' steps; the ghosts (hundredths of the way, 4 times the radius, ghost_k of the colour, disc or ring).
const STREAK: [i32; 3] = [76, 97, 138];
const STAR_LAMP: [i32; 3] = [120, 108, 80];
const STAR_HEAD: [i32; 3] = [120, 117, 103];
const SCALE: [i32; 3] = [310, 630, 310];
const GHOSTS: [(i32, i32, [i32; 3], usize); 5] = [(45, 40, [14, 58, 17], 0), (80, 20, [28, 41, 8], 0), (125, 88, [11, 35, 28], 1),
    (160, 32, [28, 29, 23], 0), (210, 136, [14, 47, 28], 1)];

impl Hd {
    /// A falloff's term along one way from a light (aflax's), as many pixels as asked, 192 past where it stops.
    fn flare_axis(&self, host: &mut Host, a0: i32, n: i32, m: i32, d: i32) -> Vec<i32> {
        if n <= 0 { return vec![]; }
        let mut v = self.go(host, "aflax", &[&[a0, n, m, d]]);
        v.resize(n as usize, 192);
        v
    }

    /// The flares (hd.rs's): each lamp's and headlight's streak and star, by the falloff's table at the indexes the
    /// machines punch; the ghosts of the strongest headlight; all of it summed onto the pixels.
    pub fn flares(&self, host: &mut Host, lights: &[Light], px: &mut [Px]) {
        let mut fl: Vec<&Light> = lights.iter().filter(|l| (l.kind == 0 || l.kind == 1) && l.s > 60).collect();
        sorter::sort(&mut fl, &[sorter::down(3)], |l| [l.s as i64]);
        if fl.is_empty() { return; }
        let d = &self.d;
        let e = |t: i32| -> i32 { if (0..192).contains(&t) { d.exp[t as usize] } else { 0 } };
        let size = self.many(host, "aflp", &[], &fl.iter().map(|l| vec![l.s, if l.kind == 1 { 5 } else { 3 }]).collect::<Vec<_>>());
        let parts = self.many(host, "apart", &[], &size.iter().flat_map(|p| [vec![20, p[0], 54, 50], vec![60, p[0], 3, 100], vec![10, p[0], 14, 100]]).collect::<Vec<_>>());
        let streak = self.many(host, "aamt", &[], &size.iter().map(|p| vec![p[0], STREAK[0], STREAK[1], STREAK[2]]).collect::<Vec<_>>());
        let star = self.many(host, "aamt", &[], &fl.iter().zip(&size).map(|(l, p)| { let k = if l.kind == 1 { STAR_HEAD } else { STAR_LAMP }; vec![p[0], k[0], k[1], k[2]] }).collect::<Vec<_>>());
        let at = self.many(host, "aflv", &[], &(0..fl.len()).flat_map(|j| (0..3).flat_map(move |c| [(j, c, 0), (j, c, 1)])).map(|(j, c, w)| vec![if w == 0 { streak[j][c] } else { star[j][c] }, SCALE[c]]).collect::<Vec<_>>());
        let index = |v: i32| -> i32 { if v < 0 { 192 } else { d.ln[v as usize] } };
        let mut adds: Vec<[Vec<i32>; 3]> = vec![Default::default(); px.len()];
        for (j, l) in fl.iter().enumerate() {
            let (phi, r, dg) = (size[j][0], size[j][1], size[j][2]);
            let (ds, d1, d2) = (parts[3 * j][0], parts[3 * j + 1][0], parts[3 * j + 2][0]);
            let sv: [i32; 3] = std::array::from_fn(|c| index(at[6 * j + 2 * c][0]));
            let cv: [i32; 3] = std::array::from_fn(|c| index(at[6 * j + 2 * c + 1][0]));
            let (cx, fx, cy, fy) = (l.x[0], l.x[1], l.y[0], l.y[1]);
            // the ways out from the light along each axis, and the streak's reach along x (8 r), on afway
            let wy = self.go(host, "afway", &[&[fy, r]]);
            let wx = self.go(host, "afway", &[&[fx, r]]);
            let r8 = wx[2];
            // (the ways reach r8 across and r down: the places they give are the flare's box, clipped to the screen)
            {
                // the ways out from the light: its own pixel, then on (16 - f, 32 - f, ...) and back (16 + f, ...)
                let ways = |host: &mut Host, f: i32, w: &[i32], n: i32, m: i32, dd: i32| -> [Vec<i32>; 3] {
                    [self.flare_axis(host, f, 1, m, dd), self.flare_axis(host, w[0], n, m, dd), self.flare_axis(host, w[1], n, m, dd)]
                };
                let (g, y1, y2) = (ways(host, fy, &wy, r, 100, dg), ways(host, fy, &wy, r, 100, d1), ways(host, fy, &wy, r, 10, d2));
                let gy: Vec<Vec<i32>> = g.iter().map(|v| if v.is_empty() { vec![] } else { self.each(host, "afsq", &[], v, 1) }).collect();
                let (gx, x2, x1) = (ways(host, fx, &wx, r8, 20, ds), ways(host, fx, &wx, r8, 10, d2), ways(host, fx, &wx, r8, 100, d1));
                let rows: Vec<(i32, usize, usize)> = (0..3).flat_map(|w| (0..g[w].len()).map(move |k| (w, k))).map(|(w, k)| {
                    let yy = match w { 0 => cy, 1 => cy + 1 + k as i32, _ => cy - 1 - k as i32 };
                    (yy, w, k)
                }).filter(|&(yy, _, _)| (0..RH as i32).contains(&yy)).collect();
                for &(yy, wy, ky) in &rows {
                    for wx in 0..3 {
                        if gx[wx].is_empty() { continue; }
                        let deck: Vec<i32> = (0..gx[wx].len()).flat_map(|k| [gx[wx][k], x2[wx][k], x1[wx][k]]).collect();
                        for c in 0..3 {
                            let t = self.go(host, "aflrow", &[&[gy[wy][ky], y1[wy][ky], y2[wy][ky], sv[c], cv[c], gx[wx].len() as i32], &deck]);
                            for (k, ts) in t.chunks(3).enumerate() {
                                let xx = match wx { 0 => cx, 1 => cx + 1 + k as i32, _ => cx - 1 - k as i32 };
                                if !(0..RW as i32).contains(&xx) { continue; }
                                let a = &mut adds[yy as usize * RW + xx as usize][c];
                                for &tt in ts { let v = e(tt); if v > 0 { a.push(v); } }
                            }
                        }
                    }
                }
            }
            if j == 0 && l.kind == 1 && l.s > 225 {
                for (t, rad4, col, ring) in GHOSTS {
                    let q = self.go(host, "aghc", &[&[2, cx, 384, t, cy, 192, t]]);
                    let g = self.go(host, "aghg", &[&[2, cx, q[2], q[0], q[1], cy, q[5], q[3], q[4]]]);
                    if g[0] == 999 || g[1] == 999 { continue; }
                    let (gx, gy) = (g[0], g[1]);
                    let w = self.go(host, "apart", &[&[1, 300, phi, 3, 10]])[0];
                    let rr16 = self.go(host, "apart", &[&[1, 0, w, rad4, 125]])[0];
                    // its radius rr in whole pixels and the count out from its middle, rr + 1
                    let rr = self.go(host, "apart", &[&[2, 3, rr16, 1, 16, 4, rr16, 1, 16]]);
                    let amt = self.go(host, "aamt", &[&[1, phi, col[0], col[1], col[2]]]);
                    let u2 = self.go(host, "aghu", &[&[rr16, rr[1]]]);
                    // its box, gx - rr to gx + rr + 1 across and the same about gy down (then clipped); drawn unless
                    // it is wholly off the screen (rr is at most 65: wholly off past 900 or before -500 across, and
                    // with no rows past 600 or before -500 down)
                    if !(-500..=900).contains(&gx) { continue; }
                    let bx = self.go(host, "asum", &[&[2, gx, rr[0], -1, gx, rr[1], 1]]);
                    if bx[1] <= 0 || bx[0] >= RW as i32 { continue; }
                    let by = if (-500..=600).contains(&gy) { self.go(host, "asum", &[&[2, gy, rr[0], -1, gy, rr[1], 1]]) } else { vec![0, 0] };
                    let (xlo, xhi) = (bx[0].max(0), bx[1].min(RW as i32));
                    // u u at each place of the box from its first: aghu's, out from the middle both ways
                    let both: Vec<i32> = u2[1..].iter().rev().chain(u2.iter()).copied().collect();
                    let cols: Vec<i32> = (xlo..xhi).collect();
                    let ux: Vec<i32> = cols.iter().map(|&xx| both[(xx - bx[0]) as usize]).collect();
                    let mut found: Vec<(usize, i32)> = vec![];
                    for yy in by[0].max(0)..by[1].min(RH as i32) {
                        let s = self.go(host, "aghs", &[&[both[(yy - by[0]) as usize], ux.len() as i32], &ux]);
                        for (k, &v) in s.iter().enumerate() { if v < 576 { found.push((yy as usize * RW + cols[k] as usize, v)); } }
                    }
                    // the profile at each place (the table's card for every 16 of it), times the amount
                    let prof = |v: i32| d.ghost16[ring][v as usize];
                    let mut ks: Vec<i32> = found.iter().map(|&(_, v)| prof(v)).collect();
                    sorter::distinct(&mut ks, &[sorter::CARD], |&k| [k as i64]);
                    for c in 0..3 {
                        let tab = self.go(host, "agtab", &[&[amt[c], ks.len() as i32], &ks]);
                        let of: HashMap<i32, i32> = ks.iter().copied().zip(tab).collect();
                        for &(i, v) in &found { let a = of[&prof(v)]; if a > 0 { adds[i][c].push(a); } }
                    }
                }
            }
        }
        for ch in 0..3 {
            let at: Vec<usize> = (0..px.len()).filter(|&i| !adds[i][ch].is_empty()).collect();
            let items: Vec<Vec<i32>> = at.iter().map(|&i| { let mut it = vec![px[i].c[ch], adds[i][ch].len() as i32]; it.extend(&adds[i][ch]); it }).collect();
            let out = self.var_each(host, "agsum", &[], &items, 1);
            for (n, &i) in at.iter().enumerate() { px[i].c[ch] = out[n]; }
        }
    }
}
