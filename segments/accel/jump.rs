// The jump, in whole numbers (hd.rs's, the reference): our car up the hill and over the crest, flying, landing and
// driving on down, and the camera that follows it; our car itself, a mesh drawn the mesh kit's way (its flat
// shading, each triangle in its material's light), at twice the size and averaged two by two; its shadow; its tail
// lamps' lights.
//
// The path (the car's distance ahead, its height, the camera's place and the car's pose), frame f, k = f - 156 (the launch):
// - how far our car's middle is ahead of the camera, 64ths: 384, and for k in 0..30 1152 sin(k / 60 of a turn) /
//   9990 more (1.8 units at the most);
// - its height (64ths, world): in the air (k in 0..18) 704 + (25920 k - 3844 k k) / 1000 (11 up at the lip, 4.86 a
//   second up and 8.65 a second a second down); bouncing (j = f - 174 in 0..6) the street's height there and 128 j
//   (6 - j) / 36; else the street's (the grade table at its place, as the march takes it);
// - its grade (thousandths): in the air 180 - 534 k / 10, else the grade table's step there (125 of its eighths over
//   2 units, / 2);
// - the view's tilt (thousandths): 55 the grades at f, f - 1 and f - 2 / 300, kept to -260..160; the horizon's row
//   152 + 440 tilt / 1000;
// - the camera's height (64ths): 2 units over the street, or the car's height at f - 1 and f - 2 halved, 109 (1.7
//   units) and the lift (k in 0..34: 896 sin(k / 68 of a turn) / 9990), whichever is more.
// All divisions round down. A sine is the swarm's table's (999 sin every thousandth of a turn, by tenths between, as
// the mesh kit takes it).
//
// The car, in the camera's space (100 units to the street's unit, x right, y up, z ahead): across and ahead the
// street's turn (c and s, 400ths) taken off: (dx c - dz s) / 256 and (dx s + dz c) / 256, dx and dz in 64ths; up
// 25 dy / 16; the view's shear (the horizon moved: kk = 1000 (192 - H0) / 440 thousandths) adds kk z / 1000 to its
// height and kk / 1000 of R's last row to R's middle one. Its turn: a yaw (millionths of a turn: 159155 the tangents
// of its heading, the street's step there and its lane's change over two frames (1000 of the eighths / 36), less the
// camera's (125 of the street's change over 12 units, eighths, / 12), / 1000; after the landing a wag, 38889
// E[8 g / 3] / 1000 sin(175070 g millionths) / 999, g = f - 174) and a pitch (-159155 its pitch in thousandths /
// 1000: its grade, but in the air (180 (18 - k) 100 + 45 k grade) / 1800, and after the landing its grade and
// -1047 E[16 g / 3] / 10000 cos(206901 g) / 999). (The design's roll in the air and its bob are left out.)
use super::{Data, Kind, Light, Px, RW, RH, SCALE, TAIL};

pub const F_LAUNCH: i64 = 156;
pub const F_LAND: i64 = 174;

fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }
/// c v / 100, cut toward zero.
fn by100(c: i64, v: i64) -> i64 { c * v / 100 }

/// The swarm's sine table (the mesh kit's reference's Trig).
pub struct Trig(pub Vec<i64>);
impl Trig {
    pub fn read(dir: &str) -> Trig {
        let text = std::fs::read_to_string(format!("{dir}/swarm/data/tables.txt")).unwrap();
        let l = text.lines().find(|l| l.split_whitespace().next() == Some("trig")).unwrap();
        Trig(l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect())
    }
    /// 999 sin of an angle (millionths of a turn): the table every thousandth of a turn, between by tenths.
    pub fn sin(&self, a: i64) -> i64 {
        let t = fl(md(a, 1_000_000), 100);
        let q = t / 2500;
        let mut r = t % 2500;
        if q % 2 == 1 { r = 2500 - r; }
        let (k, fr) = ((r / 10) as usize, r % 10);
        let v = if fr == 0 { self.0[k] } else { self.0[k] + (self.0[k + 1] - self.0[k]) * fr / 10 };
        if q >= 2 { -v } else { v }
    }
    pub fn cos(&self, a: i64) -> i64 { self.sin(a + 250_000) }
}

/// Our car as the kit's model: corners (100 units a street unit, about its pivot), triangles (corners, face normal
/// in hundredths, plane's distance, material); the materials (ambient, diffuse, highlight, 8-bit).
pub struct Car { pub v: Vec<[i64; 3]>, pub t: Vec<[usize; 3]>, pub fnorm: Vec<[i64; 3]>, pub fd: Vec<i64>, pub mat: Vec<usize>, pub mats: Vec<[[i64; 3]; 3]> }

pub struct Assets { pub trig: Trig, pub lane: Vec<i64>, pub car: Car, pub spec: Vec<i64>, pub sparks: Vec<[i64; 6]>, pub bright: Vec<i64> }

/// The camera's path across the street and our car's lane, a frame each from k = 12 t = -3 on (the camera follows the
/// car 3 frames behind), in eighths from the street's middle, as aphase, aswerve and alane work them out: 20 eighths,
/// less the swerve past the slow car ahead (data/laneswerve.txt: whole eighths and 216ths over), plus the wobble
/// (data/lanesine.txt: 3.2 sin in 216ths, a quarter of a 1000-step turn), rounded to the nearest eighth. The wobble's
/// phase turns 22.547 steps a frame, carried as steps and thousandths.
pub fn lanes(demo: &str, frames: usize) -> Vec<i64> {
    let rd = |f: &str| std::fs::read_to_string(format!("{demo}/{f}")).unwrap_or_else(|_| panic!("{}/{}", demo, f));
    let sine: Vec<i64> = rd("accel/data/lanesine.txt").split_whitespace().map(|v| v.parse().unwrap()).collect();
    let swerve: Vec<i64> = rd("accel/data/laneswerve.txt").split_whitespace().map(|v| v.parse().unwrap()).collect();
    const SLOW: [i64; 5] = [40, 95, 150, 205, 250];
    // k = -3: 3 times 22.547 steps back from 0 is 932.359 steps on
    let (mut steps, mut thousandths) = (932, 359);
    (0..frames as i64).map(|i| {
        let k = i - 3;
        // the wobble: the nearest step, its quarter of the turn and sign
        let at = (steps + if thousandths >= 500 { 1 } else { 0 }) % 1000;
        let (mut index, negative) = if at >= 500 { (at - 500, true) } else { (at, false) };
        if index > 250 { index = 500 - index; }
        let wobble = if negative { -sine[index as usize] } else { sine[index as usize] };
        thousandths += 547;
        steps = (steps + 22 + thousandths / 1000) % 1000;
        thousandths %= 1000;
        // the swerve: the one slow car within reach, if any
        let mut entry = 0;
        for s in SLOW {
            let u = s - k;
            if u < 26 && u > -10 { entry = if u > 14 { 26 - u } else { 13 + (u + 10).min(8) }; }
        }
        let (whole, over) = (swerve[2 * entry as usize], swerve[2 * entry as usize + 1]);
        20 - whole + (wobble - over + 108).div_euclid(216)
    }).collect()
}

pub fn load(demo: &str) -> Assets {
    let rd = |f: &str| std::fs::read_to_string(format!("{demo}/{f}")).unwrap_or_else(|_| panic!("{}/{}", demo, f));
    let text = rd("accel/data/car.txt");
    let rows: Vec<Vec<i64>> = text.lines().filter(|l| !l.starts_with('#')).map(|l| l.split_whitespace().filter_map(|v| v.parse().ok()).collect()).collect();
    let nv = rows[1][0] as usize;
    let mut car = Car { v: vec![], t: vec![], fnorm: vec![], fd: vec![], mat: vec![], mats: vec![] };
    for r in &rows[2..2 + nv] { car.v.push([r[0], r[1], r[2]]); }
    for r in &rows[3 + nv..] {
        car.t.push([r[0] as usize, r[1] as usize, r[2] as usize]);
        car.fnorm.push([r[3], r[4], r[5]]);
        car.fd.push(r[6]);
        car.mat.push(r[7] as usize);
    }
    for l in rd("accel/data/car_mats.txt").lines() {
        let v: Vec<i64> = l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
        car.mats.push([[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]]);
    }
    let spec = rd("mesh/data/spec.txt").lines().filter(|l| !l.starts_with('#')).flat_map(|l| l.split_whitespace().map(|v| v.parse::<i64>().unwrap()).collect::<Vec<_>>()).collect();
    // our car's lane: the lanes from k = 0, 3 frames after the camera's first
    let lane = lanes(demo, 243)[3..].to_vec();
    let sp: Vec<Vec<i64>> = rd("accel/data/sparks.txt").lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect();
    let (bright, sparks) = (sp.last().unwrap().clone(), sp[..sp.len() - 1].iter().map(|v| [v[0], v[1], v[2], v[3], v[4], v[5]]).collect());
    Assets { trig: Trig::read(demo), lane, car, spec, sparks, bright }
}

/// A table (an entry every 2 units, eighths) at z64 64ths along, in 64ths: 8 its entry and its step times the part
/// (128ths) / 16.
pub fn fine64(tab: &[i32], z64: i64) -> i64 {
    let (i, p) = (fl(z64, 128) as usize, md(z64, 128));
    8 * tab[i] as i64 + fl((tab[i + 1] - tab[i]) as i64 * p, 16)
}
/// The table's step at z64 (thousandths of a unit a unit: 125 its eighths / 2).
pub fn step(tab: &[i32], z64: i64) -> i64 { let i = fl(z64, 128) as usize; fl(125 * (tab[i + 1] - tab[i]) as i64, 2) }

pub fn ahead64(t: &Trig, f: i64) -> i64 {
    let k = f - F_LAUNCH;
    384 + if (0..30).contains(&k) { fl(1152 * t.sin(fl(50000 * k, 3)), 9990) } else { 0 }
}
pub fn car_z64(t: &Trig, f: i64) -> i64 { 144 * f + ahead64(t, f) }
pub fn car_y64(d: &Data, f: i64) -> i64 {
    let (k, j) = (f - F_LAUNCH, f - F_LAND);
    let g = fine64(&d.grade, car_z64(&d.jump.trig, f));
    if (0..18).contains(&k) { 704 + fl(25920 * k - 3844 * k * k, 1000) } else if (0..6).contains(&j) { g + fl(128 * j * (6 - j), 36) } else { g }
}
pub fn car_grade(d: &Data, f: i64) -> i64 {
    let k = f - F_LAUNCH;
    if (0..18).contains(&k) { 180 - fl(534 * k, 10) } else { step(&d.grade, car_z64(&d.jump.trig, f)) }
}

/// The camera's height (64ths) and the horizon's row.
pub fn camera(d: &Data, f: i64) -> (i64, i64) {
    let tilt = fl(55 * (car_grade(d, f) + car_grade(d, f - 1) + car_grade(d, f - 2)), 300).clamp(-260, 160);
    let k = f - F_LAUNCH;
    let lift = if (0..34).contains(&k) { fl(896 * d.jump.trig.sin(fl(250_000 * k, 17)), 9990) } else { 0 };
    let h64 = (fine64(&d.grade, 144 * f) + 128).max(fl(car_y64(d, f - 1) + car_y64(d, f - 2), 2) + 109 + lift);
    if std::env::var("ACCEL_CAM").is_ok() { eprintln!("cam {f} h64 {h64} h0 {} y {} {} {} gr {} {} {}", 152 + fl(440 * tilt, 1000), car_y64(d, f - 2), car_y64(d, f - 1), car_y64(d, f), car_grade(d, f - 2), car_grade(d, f - 1), car_grade(d, f)); }
    (h64, 152 + fl(440 * tilt, 1000))
}

/// Our car's place this frame: its pivot in the camera's space (units / 100, sheared), its turn R (thousandths,
/// sheared), how far ahead (quarters), its height over the street under it (64ths).
pub struct Pose { pub pos: [i64; 3], pub rot: [[i64; 3]; 3], pub a4: i64, pub over: i64, pub light: [i64; 3], pub half: [i64; 3] }

pub fn pose(d: &Data, cam: &super::Cam, f: i64) -> Pose {
    let (t, lane) = (&d.jump.trig, &d.jump.lane);
    let cz = car_z64(t, f);
    let y = car_y64(d, f);
    let dx = fine64(&d.road, cz) + 8 * lane[f as usize] - 8 * cam.x8;
    let dz = ahead64(t, f);
    let dy = y + 32 - cam.h64;
    let (bx, az, uy) = (fl(dx * cam.c - dz * cam.s, 256), fl(dx * cam.s + dz * cam.c, 256), fl(25 * dy, 16));
    let kk = fl(1000 * (192 - cam.h0), 440);
    // the turn
    let steer = fl(1000 * (lane[f as usize + 1] - lane[(f as usize).max(1) - 1]), 36);
    let slope = super::at(&d.road, cam.entry + 7, cam.into) - super::at(&d.road, cam.entry + 1, cam.into);
    let g = f - F_LAND;
    let ex = |i: i64| d.exp[i.min(191) as usize] as i64;
    let wag = if g >= 0 { fl(fl(38889 * ex(fl(8 * g, 3)), 1000) * t.sin(175_070 * g), 999) } else { 0 };
    let yaw = fl(159_155 * (step(&d.road, cz) + steer - fl(125 * slope, 12)), 1000) + wag;
    let k = f - F_LAUNCH;
    let gr = car_grade(d, f);
    let p1000 = if (0..18).contains(&k) { fl(180 * (18 - k) * 100 + 45 * k * gr, 1800) }
        else if g >= 0 { gr + fl(fl(-1047 * ex(fl(16 * g, 3)), 10000) * t.cos(206_901 * g), 999) } else { gr };
    let pitch = -fl(159_155 * p1000, 1000);
    // the kit's turn: the pitch after the yaw
    let (sy, cy, sp, cp) = (t.sin(yaw), t.cos(yaw), t.sin(pitch), t.cos(pitch));
    let r = [[cy, 0, sy], [fl(sp * sy, 1000), cp, fl(-sp * cy, 1000)], [fl(-cp * sy, 1000), sp, fl(cp * cy, 1000)]];
    let rot = [r[0], [0, 1, 2].map(|j| r[1][j] + fl(kk * r[2][j], 1000)), r[2]];
    if std::env::var("ACCEL_POSE").is_ok() { eprintln!("pose {f} yaw {yaw} pitch {pitch} p1000 {p1000} wag {wag} pos {:?} rot {:?}", [bx, uy + fl(kk * az, 1000), az], rot); }
    // the light and the half-way vector in the sheared space (the shear's inverse transposed: z less kk / 1000 of y),
    // so that R's transpose takes them where the unsheared R's would
    let unshear = |w: [i64; 3]| [w[0], w[1], fl(1000 * w[2] - kk * w[1], 1000)];
    Pose { pos: [bx, uy + fl(kk * az, 1000), az], rot, a4: fl(az, 25), over: y - fine64(&d.grade, cz), light: unshear(LIGHT), half: unshear(HALF) }
}

/// The kit's view: focal length 880 pixels (at twice the size), the depth test's zero, the light (toward it, and
/// half way between it and the way back to the camera: thousandths, the camera's space), the back-face margin.
pub const F10: i64 = 88;
pub const ZREF: i64 = 200;
pub const LIGHT: [i64; 3] = [289, 825, 495];
pub const HALF: [i64; 3] = [286, 817, -500];
const MARGIN: i64 = 6;
/// Twice the size: 1536 x 768.
const W2: i64 = 2 * RW;
const H2: i64 = 2 * RH;
/// The software half's: 192 x 96, focal length 110.
const F10_SOFT: i64 = 11;

/// The values of a walk from a to b (either way round): the k-th of n places on from the lesser, the value at its
/// side plus the change to the other's times (k + 1) / (n + 1), cut toward zero (the kit's).
fn walk(a: i64, b: i64, va: i64, vb: i64, k: i64) -> i64 {
    let (n, vl, vr) = if b < a { (a - b, vb, va) } else { (b - a, va, vb) };
    vl + (vr - vl) * (k + 1) / (n + 1)
}

/// Our car at twice the size, the kit's way, flat shaded: each pixel it covers (1536 x 768, row by row) and its
/// colour (8-bit). The kit's reference's sums, each triangle in its own material's light.
pub fn raster(d: &Data, p: &Pose, w2: i64, h2: i64, f10: i64) -> Vec<Option<[i64; 3]>> {
    let car = &d.jump.car;
    let r = p.rot;
    let corners: Vec<(i64, i64, i64)> = car.v.iter().map(|v| {
        let t: Vec<i64> = (0..3).map(|i| r[i][0] * v[0] + r[i][1] * v[1] + r[i][2] * v[2] + 1000 * p.pos[i]).collect();
        let z = fl(t[2], 1000);
        let (x, y) = (fl(f10 * t[0], z), fl(-f10 * t[1], z));
        let depth = z - ZREF;
        assert!((0..=999).contains(&depth), "a depth of {}", depth);
        (x, y, depth)
    }).collect();
    let tr = |w: [i64; 3]| -> [i64; 3] { std::array::from_fn(|j| fl(r[0][j] * w[0] + r[1][j] * w[1] + r[2][j] * w[2], 1000)) };
    let (lm, hm) = (tr(p.light).map(|v| fl(98 * v, 100)), tr(p.half).map(|v| fl(98 * v, 100)));
    let cam: [i64; 3] = tr(p.pos.map(|v| -v)).map(|c| fl(c, 10));
    let light = |n: [i64; 3], m: usize| -> [i64; 3] {
        let [amb, dif, spc] = car.mats[m];
        let dd: i64 = (0..3).map(|j| by100(n[j], lm[j])).sum();
        let h: i64 = (0..3).map(|j| by100(n[j], hm[j])).sum();
        let dd = if dd <= 0 { 0 } else { dd.min(989) } / 10;
        let h = if h <= 0 { 0 } else { h.min(999) };
        let s = d.jump.spec[h as usize];
        std::array::from_fn(|c| (amb[c] + by100(dd, dif[c]) + by100(s, spc[c])).clamp(0, 255))
    };
    let front: Vec<usize> = (0..car.t.len()).filter(|&k| {
        let n = car.fnorm[k];
        (0..3).map(|j| by100(n[j], cam[j])).sum::<i64>() > car.fd[k] - MARGIN
    }).collect();
    let mut edges: std::collections::BTreeMap<(usize, usize), Vec<(i64, i64, i64)>> = Default::default();
    for &k in &front {
        let t = car.t[k];
        for e in 0..3 {
            let (i, j) = (t[e], t[(e + 1) % 3]);
            let key = (i.min(j), i.max(j));
            if edges.contains_key(&key) { continue; }
            let (top, bot) = if corners[j].1 < corners[i].1 { (j, i) } else { (i, j) };
            let (a, b) = (corners[top], corners[bot]);
            let first = fl(a.1 + 49, 100);
            let rows = fl(b.1 + 49, 100) - first;
            let mut list = vec![];
            for jr in 0..rows.max(0) {
                let row = first + jr;
                if !(-h2 / 2..h2 / 2).contains(&row) { continue; }
                let x = a.0 + fl((b.0 - a.0) * (100 * row + 50 - a.1), b.1 - a.1);
                let px = fl(x + 49, 100).clamp(-w2 / 2, w2 / 2);
                list.push((row, px, walk(0, rows, a.2, b.2, jr)));
            }
            edges.insert(key, list);
        }
    }
    let mut best: Vec<Option<(i64, [i64; 3])>> = vec![None; (w2 * h2) as usize];
    for &k in &front {
        let t = car.t[k];
        let face = light(car.fnorm[k], car.mat[k]);
        let mut by_row: std::collections::BTreeMap<i64, Vec<(i64, i64)>> = Default::default();
        for e in 0..3 {
            let (i, j) = (t[e], t[(e + 1) % 3]);
            for &(row, px, v) in &edges[&(i.min(j), i.max(j))] { by_row.entry(row).or_default().push((px, v)); }
        }
        for (row, c) in by_row {
            assert_eq!(c.len(), 2, "a triangle's row met {} edges", c.len());
            let ((pa, va), (pb, vb)) = (c[0], c[1]);
            let (lo, n) = (pa.min(pb), (pb - pa).abs());
            for kx in 0..n {
                let v = walk(pa, pb, va, vb, kx);
                let x = lo + kx;
                if !(-w2 / 2..w2 / 2).contains(&x) { continue; }
                let at = ((row + h2 / 2) * w2 + x + w2 / 2) as usize;
                if best[at].is_none_or(|(dd, _)| v < dd) { best[at] = Some((v, face)); }
            }
        }
    }
    best.into_iter().map(|b| b.map(|(_, c)| c)).collect()
}

/// Our car over the frame: each pixel's four samples at twice the size, the car's colour (8-bit to tenths, v SCALE
/// / 255) where it covers one, else the pixel's; their mean ((a + b) / 2 + (c + d) / 2) / 2; the car's distance where
/// it covers three or four.
pub fn draw(d: &Data, p: &Pose, px: &mut [Px]) {
    let big = raster(d, p, W2, H2, F10);
    for y in 0..RH { for x in 0..RW {
        let s: Vec<Option<[i64; 3]>> = [(0, 0), (1, 0), (0, 1), (1, 1)].iter().map(|&(dx, dy)| big[((2 * y + dy) * W2 + 2 * x + dx) as usize]).collect();
        let n = s.iter().filter(|v| v.is_some()).count();
        if n == 0 { continue; }
        let q = &mut px[(y * RW + x) as usize];
        let v: Vec<[i64; 3]> = s.iter().map(|c| match c { Some(c) => [0, 1, 2].map(|k| c[k] * SCALE[k] / 255), None => q.c }).collect();
        q.c = [0, 1, 2].map(|k| ((v[0][k] + v[1][k]) / 2 + (v[2][k] + v[3][k]) / 2) / 2);
        if n >= 3 { q.a4 = p.a4; }
    } }
}

/// A place in the camera's space (units / 100, sheared) on the screen: across and down in 16ths of a pixel (the
/// middle 6144 and 3072, 7040 / 16ths a unit at the place's distance), and its distance (quarters).
/// (Kept to 990 pixels either way: a card holds the place.)
fn screen(p: [i64; 3]) -> (i64, i64, i64) { ((6144 + fl(7040 * p[0], p[2])).clamp(-15840, 15840), (3072 - fl(7040 * p[1], p[2])).clamp(-15840, 15840), fl(p[2], 25)) }

/// The street under our car, its middle and 2.3 units (147 64ths) ahead and behind along the street, on the screen.
fn ground_at(d: &Data, cam: &super::Cam, f: i64, dz: i64) -> (i64, i64, i64) {
    let t = &d.jump.trig;
    let cz = car_z64(t, f);
    let dx = fine64(&d.road, cz) + 8 * d.jump.lane[f as usize] - 8 * cam.x8;
    let z = ahead64(t, f) + dz;
    let y = fine64(&d.grade, cz + dz) - cam.h64;
    let kk = fl(1000 * (192 - cam.h0), 440);
    let (b, a) = (fl(dx * cam.c - z * cam.s, 256), fl(dx * cam.s + z * cam.c, 256));
    screen([b, fl(25 * y, 16) + fl(kk * a, 1000), a])
}

/// Our car's shadow on the street under it (the design's: an ellipse 1.15 units a side across, as long as the car
/// is on the screen down, k = 0.6 (1 - its height / 4 units) dark in its middle, fading from 1.25 of its size squared
/// in): k in 32nds 192 (256 - over) / 2560 (at most 19), its middle the street's under the car's middle across and
/// half way between its front's and back's down; rx8 = 1265 (1280 - over) / (4 a) eighths of a pixel, ry8 = a quarter
/// of the front's and back's rows apart (at least 16) eighths; at a pixel u = 128 |dx| / rx8, v = 128 |dy| / ry8
/// (each at most 24; within 400 pixels across and 120 down), e = u u + v v (at most 576); where e is below 320, the pixel moved toward black by k (320 - e, at
/// most 256) / 256 32nds.
pub fn shadow(d: &Data, cam: &super::Cam, p: &Pose, f: i64, px: &mut [Px]) {
    if p.over >= 256 { return; }
    let k32 = fl(192 * (256 - p.over), 2560).min(19);
    if k32 <= 0 { return; }
    let (mx, _, ma) = ground_at(d, cam, f, 0);
    let (_, fy, _) = ground_at(d, cam, f, 147);
    let (_, by, _) = ground_at(d, cam, f, -147);
    let ma = ma.max(1);
    let (cx, cy) = (fl(mx, 16), fl(fl(fy + by, 2), 16));
    let rx8 = fl(1265 * (1280 - p.over), 4 * 25 * ma);
    let ry8 = fl((fy - by).abs(), 4).max(16);
    if std::env::var("ACCEL_SHADOW").is_ok() { eprintln!("shadow k {k32} cx {cx} cy {cy} rx8 {rx8} ry8 {ry8} over {}", p.over); }
    let u2 = |k: i64, step: i64, r: i64| { let u = (step * k / r).min(24); u * u };
    for dy in -120..=120 { for dx in -400..=400 {
        let (x, y) = (cx + dx, cy + dy);
        if !(0..RW).contains(&x) || !(0..RH).contains(&y) { continue; }
        let e = (u2(dx.abs(), 128, rx8) + u2(dy.abs(), 128, ry8)).min(576);
        if e >= 320 { continue; }
        let cover = fl(k32 * (320 - e).min(256), 256);
        if cover == 0 { continue; }
        let q = &mut px[(y * RW + x) as usize];
        q.c = super::mix3(q.c, [0, 0, 0], cover, 32);
    } }
}

/// Our car's tail lamps: 0.70 either side, 2.14 back from its middle at its pivot's height, turned and placed as
/// the car is; each a light of strength 175 (500ths).
pub fn tail_lights(p: &Pose) -> Vec<Light> {
    [-70, 70].iter().map(|&x| {
        let v = [x, 0, -214];
        let w: [i64; 3] = std::array::from_fn(|i| fl(p.rot[i][0] * v[0] + p.rot[i][1] * v[1] + p.rot[i][2] * v[2], 1000) + p.pos[i]);
        let (x16, y16, a4) = screen(w);
        Light { x16, y16, a4, rgb: TAIL, s: 175, kind: Kind::OurTail }
    }).collect()
}

/// A spark's colour at full heat, tenths of a step (255, 200, 110 of 255).
pub const SPARK: [i64; 3] = [310, 494, 133];

/// The sparks where our car comes down (the design's): each (data/sparks.txt) from its end of the car on the street
/// at its frame f0 (3 64ths up), flying for 8 frames (t = f - f0 in 0..8): at T 144ths of a second its place
/// x0 + vx T / 144 across, z0 + vz T / 144 along, 3 + vy T / 144 - 640 T T / 20736 up (taken back above the street
/// where it would fall through: its size, over the street there); on the screen at T = 12 t (its head) and
/// 12 t - 9 (its tail, at least 0); left out nearer than a unit (100) or 990 pixels or more off the screen. Its brightness heat B[t] / 1000; a streak of
/// n + 1 places from head to tail (n = the greater of its pixels across and down, and 1), each place and the pixel
/// right of it taking 16 / 10 and 7 / 10 of the colour times the brightness (each / 1000, rounded down), fading to 2 / 5
/// at the tail (asfade's), where the pixel is no nearer than its head's distance less a quarter. The ten brightest
/// (the first of equals) also light up (the jet's kind: 3 / 10 of the brightness, 500ths), where the head is on
/// the screen and what it shows there is 30 units or more away.
pub fn sparks(d: &Data, cam: &super::Cam, f: i64, px: &mut [Px]) -> Vec<Light> {
    let t_ = &d.jump.trig;
    let kk = fl(1000 * (192 - cam.h0), 440);
    let mut add = vec![[0i64; 3]; px.len()];
    let mut lit: Vec<(i64, i64, i64)> = vec![];
    for sp in &d.jump.sparks {
        let [f0, end, vx, vz, vy, heat] = *sp;
        let t = f - f0;
        if !(0..8).contains(&t) { continue; }
        let cz = car_z64(t_, f0);
        let (x0, z0) = (fine64(&d.road, cz) + 8 * d.jump.lane[f0 as usize], cz + end);
        let point = |tt: i64| -> [i64; 3] {
            let (x, z) = (x0 + fl(vx * tt, 144), z0 + fl(vz * tt, 144));
            let up = (3 + fl(vy * tt, 144) - fl(640 * tt * tt, 20736)).abs() + fine64(&d.grade, z);
            let (dx, dz, dy) = (x - 8 * cam.x8, z - 144 * f, up - cam.h64);
            let (b, a) = (fl(dx * cam.c - dz * cam.s, 256), fl(dx * cam.s + dz * cam.c, 256));
            [b, fl(25 * dy, 16) + fl(kk * a, 1000), a]
        };
        let (h, tl) = (point(12 * t), point((12 * t - 9).max(0)));
        if h[2] < 100 || tl[2] < 100 { continue; }
        let (hx16, hy16, ha4) = screen(h);
        let (tx16, ty16, _) = screen(tl);
        let bright = fl(heat * d.jump.bright[t as usize], 1000);
        let (xa, ya, xb, yb) = (fl(hx16, 16), fl(hy16, 16), fl(tx16, 16), fl(ty16, 16));
        // (a spark 990 pixels or more off the screen is left out)
        if [xa, ya, xb, yb].iter().any(|v| v.abs() >= 990) { continue; }
        let n = (xb - xa).abs().max((yb - ya).abs()).max(1);
        if std::env::var("ACCEL_SPARKS").is_ok() { eprintln!("spark {xa} {ya} {xb} {yb} {n} {bright} {ha4}"); }
        let base: [i64; 3] = SPARK.map(|v| fl(v * bright, 1000));
        for j in 0..=n {
            let (x, y) = (xa + fl((xb - xa) * j, n), ya + fl((yb - ya) * j, n));
            for (ox, w) in [(0, 16), (1, 7)] {
                let (x, y) = (x + ox, y);
                if !(0..RW).contains(&x) || !(0..RH).contains(&y) { continue; }
                let i = (y * RW + x) as usize;
                if px[i].a4 < ha4 - 1 { continue; }
                for c in 0..3 { let a = fl(w * base[c], 10); add[i][c] += a - fl(3 * a * j, 5 * n); }
            }
        }
        lit.push((bright, hx16, hy16));
    }
    for (p, a) in px.iter_mut().zip(&add) { for c in 0..3 { p.c[c] = (p.c[c] + a[c]).min(999); } }
    lit.sort_by_key(|l| -l.0);
    lit.iter().take(10).filter(|&&(_, x16, y16)| {
        let (x, y) = (fl(x16, 16), fl(y16, 16));
        (0..RW).contains(&x) && (0..RH).contains(&y) && px[(y * RW + x) as usize].a4 >= 120
    }).map(|&(b, x16, y16)| Light { x16, y16, a4: 999, rgb: [255, 170, 60], s: fl(3 * b, 10), kind: Kind::Tip }).collect()
}

/// Our car in the software half: drawn at 192 x 96 (focal length 110), each of its pixels the middle of our 4 x 4
/// block (tenths, v SCALE / 255), the car's distance there.
pub fn draw_soft(d: &Data, p: &Pose, px: &mut [Px]) {
    let small = raster(d, p, RW / 4, RH / 4, F10_SOFT);
    for y in 0..RH / 4 { for x in 0..RW / 4 {
        if let Some(c) = small[(y * (RW / 4) + x) as usize] {
            let q = &mut px[((4 * y + 2) * RW + 4 * x + 2) as usize];
            q.c = [0, 1, 2].map(|k| c[k] * SCALE[k] / 255);
            q.a4 = p.a4;
        }
    } }
}
