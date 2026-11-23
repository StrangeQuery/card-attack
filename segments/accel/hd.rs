// The accelerated drive at 768x384 in ordinary Rust, whole numbers only: the scene (walls, street, sky), then what
// stands in it, the lights, the bloom and the 16-bit dither. Included by reference.rs. Every division says how it
// rounds; every number that a machine holds stays within a card (-999..999) unless it says it is carried in two.
//
// Units, as the drive's: along the street quarters of a unit (q), across it eighths (e), heights eighths; on the
// screen our pixels (768 across), the focal length 440 (110 of 192x96's), the horizon's row H0 (152, the view
// sheared up or down by moving it). A colour channel is in tenths of a 16-bit step: red and blue 0-310, green 0-630.
#![allow(dead_code)]
use std::collections::HashMap;
#[path = "jump.rs"]
pub mod jump;

pub const RW: i64 = 768;
pub const RH: i64 = 384;
/// The accelerated half: drive frames SOFT..END (after the software drive's 66, the menu's 47 and the blank's 5).
pub const SOFT: i64 = 118;
pub const END: i64 = 236;

fn nums(s: &str) -> Vec<i32> { s.split_whitespace().filter_map(|t| t.parse().ok()).collect() }
pub fn sgn(v: i64) -> i64 { if v < 0 { -1 } else { 1 } }
/// a toward b by k of n, the step rounded toward a: a + sgn (b - a) floor(|b - a| k / n).
pub fn mix(a: i64, b: i64, k: i64, n: i64) -> i64 { a + sgn(b - a) * ((b - a).abs() * k / n) }
pub fn mix3(a: [i64; 3], b: [i64; 3], k: i64, n: i64) -> [i64; 3] { [0, 1, 2].map(|c| mix(a[c], b[c], k, n)) }

pub struct Data {
    pub road: Vec<i32>, pub turn: Vec<i32>, pub blocks: Vec<i32>, pub cars: Vec<i32>, pub script: Vec<i32>, pub grade: Vec<i32>,
    pub kinds: Vec<i32>, pub palette: Vec<[i64; 3]>, pub bays: Vec<Vec<usize>>, pub roofs: Vec<Vec<usize>>, pub street: HashMap<String, Vec<usize>>,
    pub along: Vec<i32>, pub across: Vec<i32>, pub dither: Vec<i32>, pub glow: Vec<i32>, pub exp: Vec<i32>, pub ln: Vec<i32>, pub ghosts: Vec<Vec<i32>>, pub cross: Vec<Vec<i32>>, pub glowt: Vec<i32>,
    pub traffic: Vec<(i32, i32, i32, i32, usize)>, pub pictures: Vec<u8>,
    pub jump: jump::Assets,
    /// The jet's pictures for k = 1..36: width, height, each pixel's colour (tenths) and cover (32nds).
    pub jet: Vec<(i64, i64, Vec<[i64; 4]>)>,
}

pub fn load(demo: &str) -> Data {
    let rd = |f: &str| std::fs::read_to_string(format!("{demo}/{f}")).unwrap_or_else(|_| panic!("{}/{}", demo, f));
    let lines = |f: &str| rd(f).lines().map(nums).filter(|l| !l.is_empty()).collect::<Vec<Vec<i32>>>();
    let pools = lines("accel/data/pools.txt");
    let street = rd("accel/data/street.txt").lines().map(|l| { let (n, v) = l.split_once(' ').unwrap(); (n.to_string(), nums(v).iter().map(|&k| k as usize).collect()) }).collect();
    Data {
        road: nums(&rd("accel/data/road.txt")), turn: nums(&rd("drive/data/turn.txt")), blocks: nums(&rd("accel/data/blocks.txt")),
        cars: nums(&rd("accel/data/cars.txt")), script: jump::lanes(demo, 243)[..236].iter().flat_map(|&l| [l as i32, 0, 0]).collect(), grade: nums(&rd("accel/data/grade.txt")),
        kinds: nums(&rd("accel/data/kinds.txt")),
        palette: lines("accel/data/palette.txt").iter().map(|c| [c[0] as i64, c[1] as i64, c[2] as i64]).collect(),
        bays: lines("accel/data/bays.txt").iter().map(|l| l.iter().map(|&v| v as usize).collect()).collect(),
        roofs: lines("accel/data/roofs.txt").iter().map(|l| l.iter().map(|&v| v as usize).collect()).collect(),
        street, along: pools[0].clone(), across: pools[1].clone(), dither: nums(&rd("accel/data/dither.txt")), glow: nums(&rd("accel/data/glow.txt")),
        exp: lines("accel/data/exp.txt")[0].clone(), ln: lines("accel/data/exp.txt")[1].clone(), ghosts: lines("accel/data/ghosts.txt"), cross: lines("accel/data/cross.txt"), glowt: nums(&rd("accel/data/glowt.txt")),
        traffic: lines("accel/data/traffic.txt").iter().map(|l| (l[0], l[1], l[2], l[3], l[4] as usize)).collect(),
        pictures: std::fs::read(format!("{demo}/accel/data/traffic.bin")).expect("accel/data/traffic.bin: run make_data.py"),
        jump: jump::load(demo),
        jet: {
            let l = lines("accel/data/jet.txt");
            let mut out = vec![];
            let mut i = 0;
            while i < l.len() {
                let (w, h) = (l[i][1] as i64, l[i][2] as i64);
                let px = l[i + 1..i + 1 + h as usize].iter().flat_map(|r| r.chunks(4).map(|c| [c[0] as i64, c[1] as i64, c[2] as i64, c[3] as i64]).collect::<Vec<_>>()).collect();
                out.push((w, h, px));
                i += 1 + h as usize;
            }
            out
        },
    }
}

impl Data {
    pub fn sky(&self) -> usize { self.street["sky"][0] }
    /// Palette colour `id` fogged by fk 32nds toward the sky.
    pub fn fogged(&self, id: usize, fk: i64) -> [i64; 3] { mix3(self.palette[id], self.palette[self.sky()], fk, 32) }
}

/// Table `tab` (an entry every 8 quarters) at entry i plus part eighths, rounded down (the drive's `at`).
pub fn at(tab: &[i32], i: i64, part: i64) -> i64 { tab[i as usize] as i64 + ((tab[i as usize + 1] - tab[i as usize]) as i64 * part).div_euclid(8) }

/// The haze, 32nds: none to 56 units, all of it by 150, for a distance in quarters.
pub fn fog(a4: i64) -> i64 { ((a4 - 224) * 32 / 376).clamp(0, 32) }

pub struct Cam {
    pub f: i64, pub z: i64, pub entry: i64, pub into: i64, pub o: i64, pub block: i64,
    /// Across (eighths), the turn (cosine and sine times 400), height (64ths, world), the horizon's row.
    pub x8: i64, pub c: i64, pub s: i64, pub h64: i64, pub h0: i64,
}

impl Cam {
    pub fn new(d: &Data, f: i64) -> Cam {
        let z = 9 * f;
        let (entry, into) = (z / 8, z % 8);
        let lane = d.script[3 * f as usize] as i64;
        let x8 = at(&d.road, entry, into) + lane;
        let slope = at(&d.road, entry + 7, into) - at(&d.road, entry + 1, into);
        let (c, s) = (d.turn[2 * slope.unsigned_abs() as usize] as i64, sgn(slope) * d.turn[2 * slope.unsigned_abs() as usize + 1] as i64);
        // the camera's height and its view's tilt (jump.rs): 2 up over the street, or following our car over the crest
        let (h64, h0) = jump::camera(d, f);
        Cam { f, z, entry, into, o: z % 120, block: z / 120, x8, c, s, h64, h0 }
    }
    /// Column x's ray: across and along, our pixels at the screen's distance (440), rounded down.
    pub fn ray(&self, x: i64) -> (i64, i64) {
        ((880 * self.s + (2 * x - 767) * self.c).div_euclid(800), (880 * self.c - (2 * x - 767) * self.s).div_euclid(800))
    }
    /// A table (an entry every 8 quarters) q quarters ahead of the camera.
    pub fn ahead(&self, tab: &[i32], q: i64) -> i64 { let p = self.into + q; at(tab, self.entry + p.div_euclid(8), p.rem_euclid(8)) }
    /// The same z16 16ths of a quarter ahead, in 64ths (the table's eighths times 8, the part between entries in
    /// 128ths, rounded down).
    pub fn fine(&self, tab: &[i32], z16: i64) -> i64 {
        let p = 16 * self.into + z16;
        let (i, part) = ((self.entry + p.div_euclid(128)) as usize, p.rem_euclid(128));
        8 * tab[i] as i64 + ((tab[i + 1] - tab[i]) as i64 * part).div_euclid(16)
    }
}

/// What the scene leaves at a pixel: its colour (tenths), its distance (quarters; 999 for the sky).
#[derive(Clone, Copy)]
pub struct Px { pub c: [i64; 3], pub a4: i64 }

// ---- the walls, a column at a time -----------------------------------------------------------------------------

/// The blocks' faces k ahead (0-8), left (0) or right (1): across from the camera (eighths), the same in quarters
/// (rounded down), height in half units, tone, the window texture's place.
fn face(d: &Data, cam: &Cam, k: i64, side: i64) -> [i64; 5] {
    let b = &d.blocks[(8 * (cam.block + k) + 4 * side) as usize..];
    let e = d.road[(15 * (cam.block + k) + 6) as usize] as i64 - cam.x8 + if side == 0 { -(b[0] as i64) } else { b[0] as i64 };
    [e, e.div_euclid(2), b[1] as i64, b[2] as i64, b[3] as i64]
}

/// Texel (i along, j up) of a wall of the given tone, end (1 for an end wall in shade), height (half units) and
/// window texture place: its palette id.
pub fn texel(d: &Data, i: i64, j: i64, top: i64, tone: i64, end: i64, tex: i64) -> usize {
    let (bay, ii, jj) = (i.div_euclid(8), i.rem_euclid(8), j.rem_euclid(12));
    let t = (ii * 12 + jj) as usize;
    if j >= 2 * top - 4 {
        let part = if j >= 2 * top - 2 { 1 } else { 0 };
        return d.roofs[((tone * 2 + end) * 2 + part) as usize][t];
    }
    let column = (tex + 480 * end + 32 * bay).rem_euclid(960);
    let floor = j.div_euclid(12).clamp(0, 31);
    let kind = d.kinds[(column + floor).min(959) as usize] as i64;
    d.bays[((tone * 2 + end) * 9 + kind) as usize][t]
}

/// Column x's walls, nearest first, each over the rows the nearer ones left open: the pixels (row, colour, distance).
pub fn walls(d: &Data, cam: &Cam, x: i64, street: &[Px], soft: bool) -> Vec<(i64, Px)> {
    let (rx, rz) = cam.ray(x);
    let o = cam.o;
    let lat = |z: i64| sgn(rx) * (z.max(0) * rx.abs() / rz).min(600);
    let ends = |first: i64| { let mut v = vec![]; let mut z = first; for _ in 0..9 { v.push(lat(z)); if z < 864 { z += 120; } } v };
    let (near, far) = (ends(-o), ends(104 - o));
    let mut hits = vec![];
    for k in 0..9 {
        let start = 120 * k - o;
        for side in 0..2 {
            let fc = face(d, cam, k, side);
            let q = fc[1];
            let (pn, pf) = if side == 0 { (q - near[k as usize], q - far[k as usize]) } else { (near[k as usize] - q, far[k as usize] - q) };
            if start < 880 && start + 104 > 0 {
                let end = start > 0 && pn >= 0;
                if (end && pn < 120) || (pn < 0 && pf >= 0) { hits.push((k, fc, start, end)); }
            }
        }
    }
    let mut out = vec![];
    let mut open = RH - 1;
    for (k, [e, _, top, tone, tex], start, end) in hits {
        let st = start.max(16);
        // the size there as a ratio: Q = 32 / P (P pixels a unit), with denominator D; its distance (quarters)
        let (qn, den, a4) = if end { (10 * st, rz, (440 * st / rz).min(999)) } else { (5 * e.abs(), rx.abs(), (220 * e.abs() / rx.abs()).min(999)) };
        let fk = fog(a4);
        // where along it the ray meets it, tenths of a texel (a quarter unit), from its texels' middles: along a face
        // from the block's start; across an end wall from the face's corner
        let s10 = if end { 10 * start * rx.abs() / rz - 5 * e.abs() - 5 } else { 5 * e.abs() * rz / rx.abs() - 10 * start - 5 };
        let s10 = if end { s10.max(0) } else { s10.clamp(0, 1030) };
        let (i0, ws) = (s10 / 10, s10 % 10);
        if std::env::var("ACCEL_WALL").map(|v| v == x.to_string()).unwrap_or(false) { eprintln!("wall f {} k {} end {} e {} start {} i0 {} ws {} a4 {} fk {} top {} tex {} tone {}", cam.f, k, end, e, start, i0, ws, a4, fk, top, tex, tone); }
        // the block stands on the street's grade at its middle; the wall's foot where the ray meets it
        let base = (d.grade[(15 * (cam.block + k) + 6) as usize] as i64 + d.grade[(15 * (cam.block + k) + 7) as usize] as i64).div_euclid(2);
        let met = if end { start } else { e.abs() * rz / (2 * rx.abs()) };
        let foot = cam.fine(&d.grade, 16 * met);
        // twentieths of a texel up from its base, at row y: F0 - (2 (y - H0) + 1) Q, rounded down
        let f0 = (10 * (cam.h64 - 8 * base)).div_euclid(8) - 10;
        let fy = |y: i64| (f0 * den - (2 * (y - cam.h0) + 1) * qn).div_euclid(den);
        if std::env::var("ACCEL_WALL").map(|v| v == x.to_string()).unwrap_or(false) { eprintln!("wallq {} f0 {} qn {} den {} base {}", cam.f, f0, qn, den, base); }
        let (ftop, ffoot) = (40 * top - 10, (10 * (foot - 8 * base)).div_euclid(8) - 10);
        // the texel rows' colours: the two texel columns fogged, mixed by ws tenths
        let row = |j: i64| -> [i64; 3] {
            let (t0, t1) = (d.fogged(texel(d, i0, j, top, tone, end as i64, tex), fk), d.fogged(texel(d, i0 + 1, j, top, tone, end as i64, tex), fk));
            mix3(t0, t1, ws, 10)
        };
        // drawn from the lowest open row up where the street is no nearer (below it the street wins)
        let _ = ffoot;
        let mut y = open;
        while y >= 0 && street[(y * RW + x) as usize].a4 < a4 { y -= 1; }
        let mut drawn = false;
        while y >= 0 && fy(y) <= ftop {
            let v = fy(y);
            let (j, w) = (v.div_euclid(20), v.rem_euclid(20));
            let j1 = (j + 1).min(2 * top - 1);
            if std::env::var("ACCEL_WALL").map(|v| v == x.to_string()).unwrap_or(false) { eprintln!("row {} {} {} {} {} {}", cam.f, y, j, w, texel(d, i0, j, top, tone, end as i64, tex), texel(d, i0 + 1, j, top, tone, end as i64, tex)); }
            // (the software half: the nearest texel, of the two columns and the two rows)
            let c = if soft {
                let (i, jj) = (if ws >= 5 { i0 + 1 } else { i0 }, if w >= 10 { j1 } else { j });
                d.fogged(texel(d, i, jj, top, tone, end as i64, tex), fk)
            } else { mix3(row(j), row(j1), w, 20) };
            out.push((y, Px { c, a4 }));
            drawn = true;
            y -= 1;
        }
        if drawn { open = y; }
        if open < 0 { break; }
    }
    out
}

// ---- the street: each sample column's ray marched out, each row's ground between them ----------------------------

/// The march's steps (quarters along): from 3 units, a twenty-fifth further each step, to 160.
pub fn steps() -> Vec<i64> { let mut v = vec![]; let mut a = 12; while a <= 640 { v.push(a); a += (a / 25).max(1); } v }
/// The software half's march: coarser, a tenth further each step (at least 2 quarters).
pub fn steps_soft() -> Vec<i64> { let mut v = vec![]; let mut a = 12; while a <= 640 { v.push(a); a += (a / 10).max(2); } v }

/// Where a row meets the street at a sample column: distance (quarters), across from the street's middle (64ths,
/// signed, at most 999), the lamps' light there (32nds), its haze (32nds), along (16ths of a quarter into the dash's
/// 30 quarters, 0-479).
#[derive(Clone, Copy, Debug)]
pub struct Ground { pub a4: i64, pub off: i64, pub pool: i64, pub fk: i64, pub zd: i64 }

/// Sample column x's ground for every row (None where it sees none within 160 units). Each step of the march: its
/// row (16ths, the pixel's middle at y), distance, across, the lamps' light there and the haze, how far along (16ths
/// of a quarter); a row between two steps takes each of those that part of the way (the dash's place from the first,
/// by the part of the way between).
pub fn march(d: &Data, cam: &Cam, x: i64, soft: bool) -> Vec<Option<Ground>> {
    let (rx, rz) = cam.ray(x);
    let z0 = 16 * cam.o;
    let (h8, hf) = (cam.h64.div_euclid(8), cam.h64.rem_euclid(8));
    // the part of the way from a plan's entry to the next, 64ths, rounded down
    let lean = |tab: &[i32], e: i64, part: i64| ((tab[e as usize + 1] - tab[e as usize]) as i64 * part).div_euclid(16);
    let pts: Vec<(i64, i64, Ground, i64)> = if soft { steps_soft() } else { steps() }.iter().map(|&a| {
        let z16 = 16 * a * rz / 440;
        let (e, part) = (cam.entry + (16 * cam.into + z16) / 128, (16 * cam.into + z16) % 128);
        // across: the camera's place less the street's middle, eighths, then 64ths; past 117 eighths, 999
        let xs = 16 * a * rx.abs() / 440;
        let (x8, xf) = ((xs / 8).min(400), xs % 8);
        let whole = cam.x8 - d.road[e as usize] as i64 + sgn(rx) * x8;
        let off = if whole > 117 { 999 } else if whole < -117 { -999 } else { 8 * whole + sgn(rx) * xf - lean(&d.road, e, part) };
        let zp = ((z0 + z16) / 16) % 120;
        let pool = d.along[(zp - 60).abs().min(36) as usize] as i64 * d.across[(off.abs() / 4).min(160) as usize] as i64 / 32;
        // the row: 16 H0 - 8 + 16 floor(220 D / a) + floor(16 r / a) + floor(440 F / a), D the height over the street
        // in eighths, r what the division left, F the 64ths over; kept to -32..6400
        let dd = h8 - d.grade[e as usize] as i64;
        let ff = hf - lean(&d.grade, e, part);
        let (q1, r1) = ((220 * dd).div_euclid(a), (220 * dd).rem_euclid(a));
        let r16 = (16 * cam.h0 - 8 + 16 * q1 + 16 * r1 / a + (440 * ff).div_euclid(a)).clamp(-32, 6400);
        if std::env::var("ACCEL_STEPS").map(|v| v == x.to_string()).unwrap_or(false) {
            eprintln!("step a {} off {} lamp {} q {} pool {} fk {} row {} {} q1 {} {} q2 {} q3 {} D {} {} F {} {}", a, off, (zp - 60).abs().min(36), (off.abs() / 4).min(160), pool, fog(a), r16.div_euclid(16), r16.rem_euclid(16), q1, r1, 16 * r1 / a, (440 * ff).div_euclid(a), h8, d.grade[e as usize], hf, lean(&d.grade, e, part));
        }
        (r16.div_euclid(16), r16.rem_euclid(16), Ground { a4: a, off, pool, fk: fog(a), zd: (z0 + z16) % 480 }, z16)
    }).collect();
    let mut rows = vec![None; RH as usize];
    let mut y = RH - 1;
    let mut low = i64::MAX;
    for k in 0..pts.len() {
        let (row, frac, g, z16) = pts[k];
        let r = 16 * row + frac;
        while y >= 0 && 16 * y >= r && 16 * y < low {
            rows[y as usize] = Some(if k == 0 { g } else {
                let (row0, frac0, p, z0_) = pts[k - 1];
                let (num, den) = if row0 - row >= 62 { (row0 - y, row0 - row) } else { (16 * (row0 - y) + frac0, 16 * (row0 - row) + frac0 - frac) };
                let t = |u: i64, v: i64| mix(u, u + (v - u).clamp(-999, 999), num, den);
                // the dash's place carried on from the nearer step (at most 999: taken round 480 where it is used)
                Ground { a4: t(p.a4, g.a4), off: t(p.off, g.off), pool: t(p.pool, g.pool), fk: t(p.fk, g.fk), zd: p.zd + t(0, (z16 - z0_).min(999 - p.zd)) }
            });
            y -= 1;
        }
        low = low.min(r);
    }
    rows
}

/// The street's material colours at a pixel: from its distance, across, along; the haze on.
pub fn street(d: &Data, g: Ground, soft: bool) -> [i64; 3] {
    let (fk, pool) = (g.fk, g.pool);
    let off = g.off.abs();
    // the pixel's footprint across (640ths, at least 0.06 units) and along (64ths, at least 0.12); the software
    // half's 1 (hard edges)
    let wx = if soft { 1 } else { (48 * g.a4 / 110).max(38) };
    let wz = if soft { 1 } else { ((g.a4 / 8) * (g.a4 / 8) / 3).clamp(8, 999) };
    // c(l): how much of the pixel lies past an edge at l 640ths (32nds), the quotient toward zero
    let cov = |lo: i64| ((10 * off - lo) * 32 / wx + 16).clamp(0, 32);
    let zd = g.zd;    // 64ths of a unit into the dash's 7.5
    let dcov = |lo: i64| ((zd - lo) * 32 / wz + 16).clamp(0, 32);
    let dash = dcov(0) - dcov(240);
    let pick = |n: &str, k: usize| d.street[n][k];
    let p = pool as usize;
    // the cross-section: within the pixel's reach (a / 32 + 3 64ths) of an edge it is partly covered; else what lies
    // there (data/cross.txt)
    let what = d.cross[(g.a4 / 32 + 3) as usize][off.min(999) as usize];
    // (the dash is worked out only on and by the stripe: elsewhere its cover is 0 whatever it is)
    let dash = if what == 1 || what == -2 { dash } else { 0 };
    if what >= 0 {
        let id = [pick("asphalt", p), pick("stripe", dash as usize), pick("edge", 0), pick("kerb", 0), pick("walk", p), pick("beyond", 0)][what as usize];
        return d.fogged(id, fk);
    }
    // partly covered: laid on in order, as the design does
    let (cs, ce, cw, ck, cb) = (cov(-80) - cov(80), cov(3640) - cov(3800), cov(3840) - cov(5760), cov(3840) - cov(3936), cov(5760));
    let pal = |n: &str, k: usize| d.palette[pick(n, k)];
    let mut c = pal("asphalt", p);
    c = mix3(c, pal("stripe", 32), cs * dash / 32, 32);
    c = mix3(c, pal("edge", 0), ce, 32);
    c = mix3(c, pal("walk", p), cw, 32);
    c = mix3(c, pal("kerb", 0), ck, 32);
    c = mix3(c, pal("beyond", 0), cb, 32);
    mix3(c, d.palette[d.sky()], fk, 32)
}

/// Pixel x's ground on a row, between its sample columns (every 32; a pixel's middle i + 0.5 32nds of the way).
pub fn ground_at(samples: &[Vec<Option<Ground>>], x: i64, y: i64) -> Option<Ground> {
    let (s, i) = ((x / 32) as usize, x % 32);
    match (samples[s][y as usize], samples[s + 1][y as usize]) {
        (Some(p), Some(q)) => {
            let t = |a: i64, b: i64| mix(a, a + (b - a).clamp(-999, 999), 2 * i + 1, 64);
            // the dash's place wraps at 480: the nearer way round
            let (pz, qz) = (p.zd % 480, q.zd % 480);
            let dz = (qz - pz + 240).rem_euclid(480) - 240;
            Some(Ground { a4: t(p.a4, q.a4), off: t(p.off, q.off), pool: t(p.pool, q.pool), fk: t(p.fk, q.fk), zd: (pz + mix(0, dz, 2 * i + 1, 64)).rem_euclid(480) })
        }
        (Some(p), None) => Some(Ground { zd: p.zd % 480, ..p }),
        (None, Some(q)) => Some(Ground { zd: q.zd % 480, ..q }),
        (None, None) => None,
    }
}

/// The scene: walls, street and sky, each pixel's colour and distance. The street's rows from 25 sample columns'
/// rays (every 32 pixels), the pixels between by 64ths.
pub fn scene(d: &Data, cam: &Cam, with_walls: bool, soft: bool) -> Vec<Px> {
    let sky = Px { c: d.palette[d.sky()], a4: 999 };
    let mut px = vec![sky; (RW * RH) as usize];
    let samples: Vec<Vec<Option<Ground>>> = (0..=24).map(|s| march(d, cam, 32 * s, soft)).collect();
    if let Ok(v) = std::env::var("ACCEL_ROW") { let y: usize = v.parse().unwrap(); eprintln!("zs {:?}", (6..10).map(|s| samples[s][y].map(|g| g.zd % 480)).collect::<Vec<_>>()); }
    for y in 0..RH { for x in 0..RW {
        if soft && (x % 4 != 2 || y % 4 != 2) { continue; }
        if let Some(g) = ground_at(&samples, x, y) {
            if std::env::var("ACCEL_PIX").map(|v| v == (y * RW + x).to_string()).unwrap_or(false) { eprintln!("ground a4 {} off {} pool {} fk {} zd {} what {}", g.a4, g.off, g.pool, g.fk, g.zd, d.cross[(g.a4 / 32 + 3) as usize][g.off.abs().min(999) as usize]); }
            if g.a4 < 640 { px[(y * RW + x) as usize] = Px { c: street(d, g, soft), a4: g.a4 }; }
        }
    } }
    let street = px.clone();
    for x in if with_walls { 0..RW } else { 0..0 } {
        for (y, w) in walls(d, cam, x, &street, soft) {
            let p = &mut px[(y * RW + x) as usize];
            if w.a4 < p.a4 { *p = w; }
        }
    }
    px
}

/// 16-bit: each channel through the 4 x 4 ordered dither, (v + threshold) / 10 steps, then 8-bit as the card's
/// DAC would show it (the step times 255 / 31 or / 63, rounded down).
pub fn rgb565(d: &Data, px: &[Px], out: &mut Vec<u8>) {
    for (i, p) in px.iter().enumerate() {
        let (x, y) = (i as i64 % RW as i64, i as i64 / RW as i64);
        let t = d.dither[((y & 3) * 4 + (x & 3)) as usize] as i64;
        for (c, top) in [31, 63, 31].iter().enumerate() {
            let level = ((p.c[c] + t) / 10).clamp(0, *top);
            out.push((level * 255 / top) as u8);
        }
    }
}

// ---- what stands in the street: the lamp posts and the traffic ---------------------------------------------------

/// 8-bit colours, as the design gives them.
pub const LAMP: [i64; 3] = [255, 230, 170];
pub const GLARE: [i64; 3] = [255, 250, 220];
pub const TAIL: [i64; 3] = [255, 50, 50];
/// A channel's tenths of a step for 8-bit v.
pub const SCALE: [i64; 3] = [310, 630, 310];

/// A light: where (16ths of a pixel, the pixel's middle at its index), its distance (quarters), colour (8-bit),
/// strength (500ths) and kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind { Lamp, Head, Tail, Tip, OurTail }
#[derive(Clone, Copy, Debug)]
pub struct Light { pub x16: i64, pub y16: i64, pub a4: i64, pub rgb: [i64; 3], pub s: i64, pub kind: Kind }

/// A thing in the street, turned to the view: across (eighths) and along (quarters) from the camera; on the screen
/// its middle across (16ths of a pixel), and the row (16ths) of a height (64ths, world).
pub struct Seen { pub across: i64, pub a4: i64, pub x16: i64 }
/// a b / c, toward zero (the drive's t()).
fn t(a: i64, b: i64, c: i64) -> i64 { a * b / c }
impl Seen {
    /// Turned to the view (the drive's way: across t(dx, c, 400) - t(dz, s, 200) eighths, along t(t(dx, s, 400), 1,
    /// 2) + t(dz, c, 400) quarters); its middle on the screen 6136 + 3520 across / a 16ths, toward zero.
    pub fn new(cam: &Cam, dx: i64, dz: i64) -> Seen {
        let across = t(dx, cam.c, 400) - t(dz, cam.s, 200);
        let a4 = t(t(dx, cam.s, 400), 1, 2) + t(dz, cam.c, 400);
        let a = a4.max(1);
        let (q, r) = (220 * across.abs() / a, 220 * across.abs() % a);
        Seen { across, a4, x16: 6136 + sgn(across) * (16 * q + 16 * r / a) }
    }
    /// The row (16ths, the pixel's middle at y) of a height y64 (64ths, world), as the march finds its steps'.
    pub fn y16(&self, cam: &Cam, y64: i64) -> i64 { row16(cam, y64, self.a4.max(1)) }
}

/// The row (16ths) where a height y64 (64ths) a quarters ahead is seen: 16 H0 - 8 + 16 floor(220 D / a) +
/// floor(16 r / a) + 4 floor(110 F / a) + floor(4 r' / a), D the camera's height over it in eighths, F the 64ths
/// over, r and r' what the divisions leave.
pub fn row16(cam: &Cam, y64: i64, a: i64) -> i64 {
    let (h8, hf) = (cam.h64.div_euclid(8), cam.h64.rem_euclid(8));
    let (dd, ff) = (h8 - y64.div_euclid(8), hf - y64.rem_euclid(8));
    let (q1, r1) = ((220 * dd).div_euclid(a), (220 * dd).rem_euclid(a));
    let (q3, r3) = ((110 * ff).div_euclid(a), (110 * ff).rem_euclid(a));
    16 * cam.h0 - 8 + 16 * q1 + 16 * r1 / a + 4 * q3 + 4 * r3 / a
}

/// The traffic's pictures: (art, body, width) to (height, where its pixels start).
pub fn picture(d: &Data, art: i64, body: i64, tw: i64) -> (i64, i64, usize) {
    let mut best = None;
    for &(a, b, w, h, at) in &d.traffic {
        if a as i64 == art && b as i64 == body { best = Some((w as i64, h as i64, at)); if w as i64 >= tw { break; } }
    }
    best.unwrap()
}

/// The lamp posts and the traffic, far to near, each over what is further; the lights they carry.
pub fn stand(d: &Data, cam: &Cam, px: &mut [Px], soft: bool) -> Vec<Light> {
    let mut things: Vec<(i64, i64, i64)> = vec![];        // across (eighths), ahead (quarters), what (-1 left lamp, -2 right, car index)
    for k in 0..7 {
        let (a, b) = (d.road[(15 * (cam.block + k) + 7) as usize] as i64, d.road[(15 * (cam.block + k) + 8) as usize] as i64);
        let middle = a + (b - a).div_euclid(2);
        things.push((middle - 53 - cam.x8, 60 - cam.o + 120 * k, -1));
        things.push((middle + 53 - cam.x8, 60 - cam.o + 120 * k, -2));
    }
    for (i, car) in d.cars.chunks(6).enumerate() {
        let (kind, v, lane) = (car[0] as i64, car[1] as i64, car[2] as i64);
        // (the drive's dcarz: an oncoming car more than 50 frames off is 828 ahead, out of sight; a car behind is
        // placed at the camera's entry, and its distance along the view leaves it out)
        let ahead = if kind == 0 { v - 4 * cam.f } else if (v - cam.f).abs() <= 50 { 28 + 16 * (v - cam.f) } else { 828 };
        things.push((cam.ahead(&d.road, ahead.max(0)) + lane - cam.x8, ahead, i as i64));
    }
    let mut seen: Vec<(Seen, i64, i64)> = things.iter().map(|&(dx, dz, what)| (Seen::new(cam, dx, dz), dz, what)).collect();
    seen.sort_by_key(|t| -t.0.a4);
    let sky = d.palette[d.sky()];
    let mut lights = vec![];
    for (sn, dz, what) in seen {
        // (and its middle within 600 pixels of the screen's: 220 |across| / a < 600)
        if sn.a4 < 12 || sn.a4 > 600 || 220 * sn.across.abs() >= 600 * sn.a4 { continue; }
        let a4 = sn.a4;
        let fk = fog(a4);
        let sc4 = 7040 / a4;                                 // quarters of a pixel a unit
        let foot = cam.fine(&d.grade, 16 * dz);
        if std::env::var("ACCEL_THINGS").is_ok() { eprintln!("thing {what} a4 {a4} x16 {} sc4 {sc4} fk {fk} foot {} top {}", sn.x16, sn.y16(cam, foot), sn.y16(cam, foot + 320)); }
        if what < 0 {
            // a post: 5 up, its half width at least 0.6 pixels; its arm and head 0.8 out over the street
            let side: i64 = if what == -1 { -1 } else { 1 };
            let (pfoot, ptop) = (sn.y16(cam, foot), sn.y16(cam, foot + 320));
            let hw = (sc4 / 2).max(10);
            let (x0, x1) = (((sn.x16 - hw - 16).div_euclid(16)).max(0), ((sn.x16 + hw + 32).div_euclid(16)).min(RW));
            let (y0, y1) = (ptop.div_euclid(16).max(0), (pfoot.div_euclid(16) + 1).min(RH));
            let post = d.palette[d.street["post"][0]];
            for xx in x0..x1 {
                let dxx = 16 * xx - sn.x16;
                let cov = (2 * (hw + 8 - dxx.abs())).clamp(0, 32);
                if cov == 0 { continue; }
                let shade = 70 + (-side * 60 * dxx / hw).clamp(-30, 30);
                let c = mix3(mix3(post.map(|v| v * shade / 100), sky, fk, 32), sky, fk, 32);
                for yy in y0..y1 {
                    let p = &mut px[(yy * RW + xx) as usize];
                    if p.a4 > a4 { p.c = mix3(p.c, c, cov, 32); if cov > 16 { p.a4 = a4; } }
                }
            }
            let hx = sn.x16 - side * (3 * sc4 + sc4 / 5);
            let hy = ptop;
            let ah = (sc4 / 32).max(1);
            let (lo, hi) = (sn.x16.min(hx).div_euclid(16), sn.x16.max(hx).div_euclid(16));
            let arm = mix3(post, sky, fk, 32).map(|v| v * 8 / 10);
            let row = hy.div_euclid(16);
            for yy in (row - ah).max(0)..=row.min(RH - 1) { for xx in (lo - 1).max(0)..(hi + 2).min(RW) {
                let p = &mut px[(yy * RW + xx) as usize];
                if p.a4 > a4 { p.c = mix3(arm, sky, fog(p.a4), 32); }
            } }
            let (cx, cy) = (hx.div_euclid(16), hy.div_euclid(16));
            // (the software half looks at the pixel it shows there: its 4 x 4 block's)
            let (sx, sy) = if soft { (4 * cx.div_euclid(4) + 2, 4 * cy.div_euclid(4) + 2) } else { (cx, cy) };
            if (0..RW).contains(&cx) && (0..RH).contains(&cy) && px[(sy * RW + sx) as usize].a4 >= a4 - 2 {
                let s = (32 - fk) * (125 + 3 * sc4 + sc4 / 8).min(500) / 32;
                lights.push(Light { x16: hx, y16: hy + sc4 / 4, a4, rgb: LAMP, s, kind: Kind::Lamp });
            }
        } else {
            // a car: its picture (the smallest made at least its width), each pixel over what is further, hazed
            let car = &d.cars[6 * what as usize..6 * what as usize + 6];
            let (art, body) = (car[3] as i64, car[4] as i64);
            let (w16, h16) = (9 * sc4, 3 * sc4 + sc4 / 2);
            if w16 < 16 { continue; }
            let py = sn.y16(cam, foot);
            let (x0, y0) = (sn.x16 - w16 / 2, py - h16);
            let (tw, th) = ((w16 + 8) / 16 + 1, (h16 + 8) / 16 + 1);
            let (pw, ph, at) = picture(d, art, body, tw);
            let (ix, iy) = (x0.div_euclid(16), y0.div_euclid(16));
            for v in 0..th { for u in 0..tw {
                let (xx, yy) = (ix + u, iy + v);
                if !(0..RW).contains(&xx) || !(0..RH).contains(&yy) { continue; }
                let k = at + 8 * ((v * ph / th) * pw + u * pw / tw) as usize;
                let g = |i: usize| u16::from_le_bytes([d.pictures[k + 2 * i], d.pictures[k + 2 * i + 1]]) as i64;
                let cover = g(3);
                if cover == 0 { continue; }
                let p = &mut px[(yy * RW + xx) as usize];
                if p.a4 > a4 {
                    let c = mix3(mix3([g(0), g(1), g(2)], sky, fk, 32), sky, fk, 32);
                    p.c = mix3(p.c, c, cover, 32);
                    if cover > 16 { p.a4 = a4; }
                }
            } }
            for lx in [3, 13] {
                let off = lx * (w16 / 16) + lx * (w16 % 16) / 16;
                // (a card holds the place: a lamp more than 990 pixels across is left out)
                if x0.div_euclid(16) + off / 16 > 990 { continue; }
                let (x16, y16) = (x0 + off, y0 + h16 / 2);
                if art == 1 { lights.push(Light { x16, y16, a4, rgb: GLARE, s: (32 - fk) * (175 + 5 * sc4).min(650) / 32, kind: Kind::Head }); }
                else { lights.push(Light { x16, y16, a4, rgb: TAIL, s: (32 - fk) * (75 + 2 * sc4 + sc4 / 12).min(300) / 32, kind: Kind::Tail }); }
            }
        }
    }
    // the jet's lights, down the strip of sky: its wingtips blinking, its strobe
    if (104..140).contains(&cam.f) {
        let k = cam.f - 103;
        let size16 = 64 + 4 * k * k / 9;
        let (jx16, jy16) = (16 * 368 + 160 * k / 9, 16 * 120 - 160 * k * k / 81);
        let top = jy16 - 9 * size16 / 16;
        let (jw, jh) = ((2 * size16 / 16).max(2), (7 * size16 / 128).max(1));
        let ix = (jx16 - size16).div_euclid(16);
        if cam.f % 4 < 2 {
            lights.push(Light { x16: 16 * ix + 32 * jw / 100, y16: top + 96 * jh / 10, a4: 999, rgb: [255, 50, 50], s: 250, kind: Kind::Tip });
            lights.push(Light { x16: 16 * ix + 1568 * jw / 100, y16: top + 96 * jh / 10, a4: 999, rgb: [60, 255, 90], s: 250, kind: Kind::Tip });
        }
        if cam.f % 6 == 0 { lights.push(Light { x16: 16 * ix + 8 * jw, y16: top + 8 * jh, a4: 999, rgb: [255, 255, 255], s: 350, kind: Kind::Tip }); }
    }
    lights
}

/// The jet in the strip of sky (frames 104 to 139): its picture for k = f - 103 (data/jet.txt) at its left pixel
/// (368 pixels and 160 k / 9 16ths, less half its span 64 + 4 k k / 9 16ths, rounded down) and its top's row (120
/// pixels less 160 k k / 81 16ths, less 9 / 16 of the half span, rounded down), laid over the sky only (as a thing
/// at 998 quarters: the sky is at 999).
pub fn jet(d: &Data, f: i64, px: &mut [Px]) {
    if !(104..140).contains(&f) { return; }
    let k = f - 103;
    let size16 = 64 + 4 * k * k / 9;
    let (jx16, jy16) = (16 * 368 + 160 * k / 9, 16 * 120 - 160 * k * k / 81);
    let top = jy16 - 9 * size16 / 16;
    let (ix, iy) = ((jx16 - size16).div_euclid(16), top.div_euclid(16));
    let (w, h, pic) = &d.jet[k as usize - 1];
    for yy in 0..*h { for xx in 0..*w {
        let (x, y) = (ix + xx, iy + yy);
        if !(0..RW).contains(&x) || !(0..RH).contains(&y) { continue; }
        let c = pic[(yy * w + xx) as usize];
        if c[3] == 0 { continue; }
        let p = &mut px[(y * RW + x) as usize];
        if p.a4 > 998 { p.c = mix3(p.c, [c[0], c[1], c[2]], c[3], 32); if c[3] > 16 { p.a4 = 998; } }
    } }
}

// ---- the lights: glows, the bloom, the flares --------------------------------------------------------------------

/// A light's strength times its colour, tenths (8-bit amount 200 of each 255).
/// (The colour's part, K = 8-bit v 200 / 255 of the channel's tenths a step, rounded down; the strength's thousandths of it.)
pub fn light_k(rgb: [i64; 3]) -> [i64; 3] { [0, 1, 2].map(|c| rgb[c] * 200 * SCALE[c] / (255 * 255)) }
fn amount(rgb: [i64; 3], s1000: i64) -> [i64; 3] { light_k(rgb).map(|k| s1000 * k / 1000) }

/// Each light's glow added (the light's middle taken to its pixel): the profile (data/glowt.txt: exp(-rho^2 / 0.245)
/// + 0.35 / (1 + rho^2), 500ths, at rho = t / 32) times its amount, at t = max + 3 min / 8 of tx = 4 c |dx| / R
/// and ty = 128 |dy| / R (an octagon close to a circle; c = 32 / the aspect: 32, 23 for 1.4, 20 for 1.6, 18 for
/// 1.8), R the radius in quarters of a pixel; out to t = 192 (6 radii). A quarter of it is worked out and laid four
/// ways.
pub fn glows(d: &Data, lights: &[Light], px: &mut [Px]) {
    if std::env::var("ACCEL_LIGHTS").is_ok() { for l in lights { eprintln!("light {} {} a4 {} s {} {:?}", l.x16, l.y16, l.a4, l.s, l.kind); } }
    let mut add = vec![[0i64; 3]; px.len()];
    let (mut points, mut touches, mut big) = (0, 0, 0);
    for l in lights {
        if l.s <= 10 { continue; }
        let sc4 = 7040 / l.a4.max(12);
        // radius (quarters of a pixel), aspect, strength (thousandths)
        let (r4, c, s) = match l.kind {
            Kind::Lamp => ((9 * sc4 / 10).max(12), 32, 9 * l.s / 5),
            Kind::Head => ((6 * sc4 / 10).max(12), 23, 7 * l.s / 5),
            Kind::Tail => ((7 * sc4 / 20).max(8), 20, 6 * l.s / 5),
            Kind::Tip => (12, 32, 2 * l.s),
            Kind::OurTail => ((9 * sc4 / 55).clamp(8, 144), 18, 2 * l.s),
        };
        let amt = amount(l.rgb, s);
        let table: Vec<[i64; 3]> = d.glowt.iter().map(|&k| amt.map(|a| k as i64 * a / 500)).collect();
        let (cx, cy) = (l.x16.div_euclid(16), l.y16.div_euclid(16));
        for dy in 0..=(3 * r4 / 2) {
            let ty = 128 * dy / r4;
            for dx in 0..=(48 * r4 / c) {
                let tx = 4 * c * dx / r4;
                let t = tx.max(ty) + 3 * tx.min(ty) / 8;
                if t > 192 { continue; }
                points += 1;
                if table[t as usize].iter().any(|&v| v > 0) { big += 1; }
                for (sx, sy) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                    if (sx < 0 && dx == 0) || (sy < 0 && dy == 0) { continue; }
                    let (x, y) = (cx + sx * dx, cy + sy * dy);
                    if !(0..RW).contains(&x) || !(0..RH).contains(&y) { continue; }
                    touches += 1;
                    let a = &mut add[(y * RW + x) as usize];
                    for k in 0..3 { a[k] += table[t as usize][k]; }
                }
            }
        }
    }
    if std::env::var("ACCEL_STATS").is_ok() { eprintln!("glows: {} lights, {points} points ({big} not nothing), {touches} touches, {} pixels", lights.len(), add.iter().filter(|a| a.iter().any(|&v| v > 0)).count()); }
    for (p, a) in px.iter_mut().zip(add) { for c in 0..3 { p.c[c] = (p.c[c] + a[c]).min(999); } }
}

/// The bloom: each 4 x 4 block's top-left pixel past 150 (8-bit), in steps (tenths / 10, rounded down), blurred at
/// 192 x 96 by three boxes of 7 and three of 19 (down the columns, then along the rows), (495 of the first + 330 of
/// the second) / 100 tenths added to its block, at most 999 (a card holds it; past 630 every channel shows its top).
pub fn bloom(px: &mut [Px]) {
    const T: [i64; 3] = [182, 371, 182];
    let (w, h) = (192usize, 96usize);
    let small: Vec<[i64; 3]> = (0..w * h).map(|i| { let p = px[(i / w) * 4 * RW as usize + (i % w) * 4].c; [0, 1, 2].map(|c| (p[c] - T[c]).max(0) / 10) }).collect();
    let blur = |r: usize| -> Vec<[i64; 3]> {
        let mut b = small.clone();
        let n = 2 * r as i64 + 1;
        for axis in 0..2 { for _ in 0..3 {
            let mut next = b.clone();
            let (lines, len) = if axis == 0 { (w, h) } else { (h, w) };
            for l in 0..lines { for c in 0..3 {
                let at = |k: i64| -> usize { let k = k.clamp(0, len as i64 - 1) as usize; if axis == 0 { k * w + l } else { l * w + k } };
                let mut sum: i64 = (-(r as i64)..=r as i64).map(|k| b[at(k)][c]).sum();
                for k in 0..len as i64 {
                    next[at(k)][c] = sum / n;
                    sum += b[at(k + r as i64 + 1)][c] - b[at(k - r as i64)][c];
                }
            } }
            b = next;
        } }
        b
    };
    let (b3, b9) = (blur(3), blur(9));
    for (i, p) in px.iter_mut().enumerate() {
        let (x, y) = (i % RW as usize / 4, i / RW as usize / 4);
        for c in 0..3 { p.c[c] = (p.c[c] + (495 * b3[y * w + x][c] + 330 * b9[y * w + x][c]) / 100).min(999); }
    }
}

/// E[t], the falloff 1000 exp(-t / 16) (0 from 192 on); the index of v (v 1 or more): LN[v - 1], the step where E
/// falls to v; 192 (none) for v of 0 or less.
fn falloff(d: &Data, t: i64) -> i64 { if t < 192 { d.exp[t as usize] as i64 } else { 0 } }
fn index_of(d: &Data, v: i64) -> i64 { if v <= 0 { 192 } else { d.ln[(v.min(1000) - 1) as usize] as i64 } }

/// The streak's colour, thousandths of phi by channel (230 0.6 of 8-bit as 550, 700, 1000 thousandths).
pub const STREAK: [i64; 3] = [76, 97, 138];
/// The star's colour: 120 of each 255 of the light's.
pub fn star_k(rgb: [i64; 3]) -> [i64; 3] { rgb.map(|v| 120 * v / 255) }
/// The ghosts: how far to the middle and past it (hundredths), the radius (pixels), the colour (thousandths), disc
/// (0) or ring (1); each colour's part of a ghost's amount, phi G / 1000 (G = 23.8 colour SCALE / 255000).
pub const GHOSTS: [(i64, i64, [i64; 3], usize); 5] = [(45, 10, [500, 1000, 600], 0), (80, 5, [1000, 700, 300], 0), (125, 22, [400, 600, 1000], 1),
    (160, 8, [1000, 500, 800], 0), (210, 34, [500, 800, 1000], 1)];
pub fn ghost_k(col: [i64; 3]) -> [i64; 3] { [0, 1, 2].map(|c| 238 * col[c] * SCALE[c] / 2_550_000) }

/// a m / dd rounded down, at most 192 (how far along a falloff a place a 16ths from a light is).
fn term(a: i64, m: i64, dd: i64) -> i64 { (a * m / dd).min(192) }

/// A ghost's middle, as far as hundredths t of the way from a light's pixel p to the screen's middle m (pixels,
/// rounded down), or none past 990 pixels either way (off the screen).
fn ghost_at(p: i64, m: i64, t: i64) -> Option<i64> {
    let q = (m - p).abs() * t / 100;
    let r = (m - p).abs() * t % 100;
    if q > 995 { return None; }
    let g = if m >= p { p + q } else { p - q - if r > 0 { 1 } else { 0 } };
    if g > 990 || g < -990 { None } else { Some(g) }
}

/// The flares on the lamps and the headlights (the strongest first): a horizontal streak, a four-point star, and on
/// the strongest headlight (if strong) ghosts down the line through the screen's middle. phi (thousandths): the
/// light's strength (at most 600) 7 / 5, 3 / 5 of that for a lamp; out r = phi / 25 pixels down and 8 r across. At a
/// place a 16ths across and b down from the light: the streak at E[g b b / 32 + a 20 / ds + its index] (g = b 100 /
/// dg, dg = 90 + phi / 20: sigma / 10; ds = 20 + 54 phi / 50: its length / 50), the star's arms at E[b 100 / d1 + a
/// 10 / d2 + its index] and E[a 100 / d1 + b 10 / d2 + its index] (d1 = 60 + 3 phi / 100: the arm's width / 10;
/// d2 = 10 + 14 phi / 100: its length / 100), each term at most 192; the indexes of the streak's and star's colours,
/// phi STREAK / 1000 and phi star_k / 1000 as tenths of a step. All of it added at the end, at most 999.
pub fn flares(d: &Data, lights: &[Light], px: &mut [Px]) {
    let mut fl: Vec<&Light> = lights.iter().filter(|l| matches!(l.kind, Kind::Lamp | Kind::Head) && l.s > 60).collect();
    fl.sort_by_key(|l| -l.s);
    let mut add = vec![[0i64; 3]; px.len()];
    for (n, l) in fl.iter().enumerate() {
        let head = l.kind == Kind::Head;
        let phi = l.s.min(600) * 7 / 5 * if head { 5 } else { 3 } / 5;
        let r = phi / 25;
        let (dg, ds, d1, d2) = (90 + phi / 20, 20 + 54 * phi / 50, 60 + 3 * phi / 100, 10 + 14 * phi / 100);
        let streak = STREAK.map(|k| phi * k / 1000);
        let star = star_k(l.rgb).map(|k| phi * k / 1000);
        let sv = [0, 1, 2].map(|c| index_of(d, streak[c] * SCALE[c] / 255));
        let cv = [0, 1, 2].map(|c| index_of(d, star[c] * SCALE[c] / 255));
        let (cx, cy) = (l.x16.div_euclid(16), l.y16.div_euclid(16));
        for yy in (cy - r).max(0)..(cy + r + 1).min(RH) {
            let ay = (16 * yy - l.y16).abs();
            let g = term(ay, 100, dg);
            let gy = (g * g / 32).min(192);
            let (y1, y2) = (term(ay, 100, d1), term(ay, 10, d2));
            for xx in (cx - 8 * r).max(0)..(cx + 8 * r + 1).min(RW) {
                let ax = (16 * xx - l.x16).abs();
                let (gx, x1, x2) = (term(ax, 20, ds), term(ax, 100, d1), term(ax, 10, d2));
                let a = &mut add[(yy * RW + xx) as usize];
                for c in 0..3 { a[c] += falloff(d, gy + gx + sv[c]) + falloff(d, y1 + x2 + cv[c]) + falloff(d, x1 + y2 + cv[c]); }
            }
        }
        if n == 0 && head && l.s > 225 {
            // the ghosts: discs and rings, t of the way to the middle and past it; how far out a pixel is, 16 (u u + v
            // v) / 256 of the radius squared, u = 256 |dx| / rr16 (at most 24) and v the same down
            for (t, rad, col, ring) in GHOSTS {
                let (Some(gx), Some(gy)) = (ghost_at(cx, 384, t), ghost_at(cy, 192, t)) else { continue };
                let rr16 = rad * 4 * (300 + 3 * phi / 10) / 125;
                let amt = ghost_k(col).map(|k| phi * k / 1000);
                let rr = rr16 / 16 + 3;
                let u2 = |k: i64| { let u = (256 * k.abs() / rr16).min(24); u * u };
                for yy in (gy - rr).max(0)..(gy + rr + 1).min(RH) { for xx in (gx - rr).max(0)..(gx + rr + 1).min(RW) {
                    let i = u2(xx - gx) + u2(yy - gy);
                    if i >= 576 { continue; }
                    let k = d.ghosts[ring][(i / 16) as usize] as i64;
                    let a = &mut add[(yy * RW + xx) as usize];
                    for c in 0..3 { a[c] += k * amt[c] / 500; }
                } }
            }
        }
    }
    for (p, a) in px.iter_mut().zip(add) { for c in 0..3 { p.c[c] = (p.c[c] + a[c]).min(999); } }
}

/// How much of the frame is drawn (ACCEL_STAGES, for building the machines a stage at a time): 1 the street and the
/// sky, 2 the walls, 3 what stands in the street, 4 the glows, 5 the bloom, 6 the flares (all, the default).
pub fn stages() -> i64 { std::env::var("ACCEL_STAGES").ok().and_then(|v| v.parse().ok()).unwrap_or(6) }

/// Frame f of the software half (192 x 96, 8-bit): the scene at the middle of each 4 x 4 block of ours (the street
/// marched coarser, its edges hard, the walls' texels the nearest), what stands in it, the jet, our car's shadow and
/// our car (jump.rs's draw_soft: at 192 x 96, no samples between), the lights' glows; no bloom or flares; each
/// channel quantised to the drive's ten levels through the 4 x 4 ordered dither (soft_rgb).
pub fn frame_soft(d: &Data, f: i64) -> Vec<u8> {
    let cam = Cam::new(d, f);
    let mut px = scene(d, &cam, true, true);
    if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix scene {:?} {}", px[i].c, px[i].a4); }
    let mut lights = stand(d, &cam, &mut px, true);
    if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix stand {:?} {}", px[i].c, px[i].a4); }
    jet(d, f, &mut px);
    let pose = jump::pose(d, &cam, f);
    jump::shadow(d, &cam, &pose, f, &mut px);
    jump::draw_soft(d, &pose, &mut px);
    lights.extend(jump::tail_lights(&pose));
    if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix before glows {:?} {}", px[i].c, px[i].a4); for l in &lights { eprintln!("light {} {} {} {} {:?}", l.x16, l.y16, l.a4, l.s, l.kind); } }
    glows(d, &lights, &mut px);
    if let Ok(v) = std::env::var("ACCEL_PIX") { let i: usize = v.parse().unwrap(); eprintln!("pix {:?} {}", px[i].c, px[i].a4); }
    soft_rgb(d, &px)
}

/// The software renderer's colours: each channel of each block's middle pixel one of the drive's ten levels
/// (LEVELS, 8-bit), the one at or below it or the next up through the 4 x 4 ordered dither: in tenths t, the level i
/// the last whose start, SCALE LEVELS[i] / 255 rounded up, is at most t (i at most 8); the next up where t is at
/// least floor(((2 B + 1) SCALE (hi - lo) + 32 SCALE lo) / 8160) + 1 (B the Bayer matrix's 0-15 at the block).
pub const LEVELS: [[i64; 10]; 3] = [[0, 12, 26, 38, 52, 70, 100, 150, 205, 255], [0, 12, 26, 36, 48, 62, 90, 140, 205, 255], [0, 16, 28, 40, 54, 70, 100, 150, 205, 255]];
pub const BAYER: [i64; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
pub fn soft_level(c: usize, t: i64, b: i64) -> i64 {
    let lv = &LEVELS[c];
    let sc = SCALE[c];
    let mut i = 0;
    for k in 1..9 { if 255 * t >= sc * lv[k] { i = k; } }
    let (lo, hi) = (lv[i], lv[i + 1]);
    let th = ((2 * b + 1) * sc * (hi - lo) + 32 * sc * lo) / 8160 + 1;
    if t >= th { hi } else { lo }
}
pub fn soft_rgb(_d: &Data, px: &[Px]) -> Vec<u8> {
    let mut out = vec![];
    for y in 0..RH / 4 { for x in 0..RW / 4 {
        let p = px[((4 * y + 2) * RW + 4 * x + 2) as usize];
        let b = BAYER[((y % 4) * 4 + x % 4) as usize];
        for c in 0..3 { out.push(soft_level(c, p.c[c], b) as u8); }
    } }
    out
}

/// Frame f of the accelerated half, before the 16-bit dither.
pub fn frame(d: &Data, f: i64) -> Vec<Px> {
    let cam = Cam::new(d, f);
    let stage = stages();
    let mut px = scene(d, &cam, stage >= 2, false);
    if stage < 3 { return px; }
    let lights = stand(d, &cam, &mut px, false);
    jet(d, f, &mut px);
    // our car, its shadow first, and its tail lamps' lights
    let pose = jump::pose(d, &cam, f);
    jump::shadow(d, &cam, &pose, f, &mut px);
    jump::draw(d, &pose, &mut px);
    let mut lights = lights;
    lights.extend(jump::tail_lights(&pose));
    lights.extend(jump::sparks(d, &cam, f, &mut px));
    if stage >= 4 { glows(d, &lights, &mut px); }
    if stage >= 5 { bloom(&mut px); }
    if stage >= 6 { flares(d, &lights, &mut px); }
    px
}
