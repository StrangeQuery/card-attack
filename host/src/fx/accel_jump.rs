//! The jump on CARDIAC (segments/accel/jump.rs's, the reference): our car's path up the hill, over the crest, in the
//! air and down, and the camera that follows it, worked out by the swarm's sums (`swarm_calc`: exact sums of
//! products, divisions rounded down, the sine table); our car drawn by the mesh kit (flat shaded, each triangle in its
//! material's light) at twice the size, its four samples a pixel averaged by `aavg4`.
//!
//! The host deals the frame's numbers, fetches the grade and the street's plan at the indexes the sums punched, picks
//! the formula the frame's part of the jump calls for (the plan's schedule), and carries the kit's pixels to the frame.
use crate::*;
use super::accel_hd::{Cam, Hd, Px, RH, RW};
use super::mesh::{Kit, Material, Model, Place, Shading, View};
use super::swarm_calc::{Arg, Calc, N};
use std::collections::HashMap;
use std::path::Path;

pub const F_LAUNCH: i32 = 156;
pub const F_LAND: i32 = 174;
const F10: i64 = 88;
const ZREF: i64 = 200;
const LIGHT: [i64; 3] = [289, 825, 495];
const HALF: [i64; 3] = [286, 817, -500];
const SCALE: [i32; 3] = [310, 630, 310];

pub struct Jump { pub kit: Kit, model: Model, mats: Vec<Material>, face_mat: Vec<usize>, pub lane: Vec<i32>, sparks: Vec<[i32; 6]>, bright: Vec<i32>,
    /// each spark's last frame (its first and 7), a table made at the start
    spark_last: Vec<i32>,
    /// 8-bit to tenths, each channel's value worked out once (apart)
    tenths: std::cell::RefCell<HashMap<(usize, i32), i32>> }

/// What the frame's sums give the scene: the camera's height (eighths and 64ths over), the horizon's row; and the
/// sums themselves, for our car.
pub struct Path64 { pub h8: i32, pub hf: i32, pub h0: i32, c: Calc, n: Nodes }

/// The frame's numbers kept for our car: at f (the car's height, ahead, the street's plan at it), the camera's
/// height and horizon, the grade there, the falloff's indexes for the landing.
struct Nodes { y: N, a64: N, p: N, ri: (i32, i32), h64: N, h0: N, gr: N, e8: Option<N>, e16: Option<N>, cz: N, gc: N }

fn card(c: &mut Calc, n: N) -> N { c.run("sw_card", vec![Arg::All(n)], 1, 1, 999) }

impl Jump {
    pub fn new(demo: &Path) -> Jump {
        let model = Model::read(demo, "accel/data/car.txt");
        let text = read(demo, "accel/data/car.txt");
        let rows: Vec<Vec<i32>> = text.lines().filter(|l| !l.starts_with('#')).map(|l| l.split_whitespace().filter_map(|v| v.parse().ok()).collect()).collect();
        let nv = rows[1][0] as usize;
        let face_mat = rows[3 + nv..].iter().map(|r| r[7] as usize).collect();
        let mats = read(demo, "accel/data/car_mats.txt").lines().map(|l| {
            let v: Vec<i32> = l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
            Material { ambient: [v[0], v[1], v[2]], diffuse: [v[3], v[4], v[5]], highlight: [v[6], v[7], v[8]] }
        }).collect();
        // our car's lane: worked out on CARDIAC with the camera's path (Hd::new)
        let lane = vec![];
        let sp: Vec<Vec<i32>> = read(demo, "accel/data/sparks.txt").lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect();
        let bright = sp.last().unwrap().clone();
        let sparks = sp[..sp.len() - 1].iter().map(|v| [v[0], v[1], v[2], v[3], v[4], v[5]]).collect();
        let spark_last = sp[..sp.len() - 1].iter().map(|v| v[0] + 7).collect();
        Jump { kit: Kit::new(demo), model, mats, face_mat, lane, sparks, bright, spark_last, tenths: Default::default() }
    }

    /// The camera over the jump (jump.rs's camera) and the numbers our car needs.
    pub fn path(&self, hd: &Hd, host: &mut Host, f: i32) -> Path64 {
        let d = &hd.d;
        let mut c = Calc::new();
        // each of f - 2, f - 1 and f: how far ahead our car is, where (64ths), the grade's entry and part there
        struct At { k: N, a64: N, i: N, p: N, cz: N }
        // (the frame dealt once, the two before it from it on the calculator; the host keeps the schedule's frames
        // to choose each stage)
        let frames = [f - 2, f - 1, f];
        let fnum = c.num(f as i64);
        let fnums = [c.addk(fnum, -2), c.addk(fnum, -1), fnum];
        let at: Vec<At> = frames.iter().zip(fnums).map(|(&fr, frn)| {
            let k = { let b = c.num(F_LAUNCH as i64); c.sub(frn, b) };
            let a64 = if (F_LAUNCH..F_LAUNCH + 30).contains(&fr) {
                let m = c.mulk(k, 50_000);
                let ang = c.divk(m, 3);
                let (s, _) = c.sin(ang);
                let m = c.mulk(s, 1152);
                let off = c.divk(m, 9990);
                let off = c.within(off, 200);
                c.addk(off, 384)
            } else { c.num(384) };
            let fz = c.mulk(frn, 144);
            let cz = c.add(fz, a64);
            let q = c.divk(cz, 128);
            let q = c.within(q, 999);
            let r = c.modk(cz, 128);
            let (i, p) = (card(&mut c, q), card(&mut c, r));
            At { k, a64, i, p, cz }
        }).collect();
        let cam_z = c.mulk(fnum, 144);
        let (q, r) = (c.divk(cam_z, 128), c.modk(cam_z, 128));
        let q = c.within(q, 999);
        let (ci, cp) = (card(&mut c, q), card(&mut c, r));
        // the falloff's indexes for the landing's wag and pitch
        let (e8, e16) = if f >= F_LAND {
            let gn = { let b = c.num(F_LAND as i64); c.sub(fnum, b) };
            let m8 = c.mulk(gn, 8);
            let m16 = c.mulk(gn, 16);
            let (q8, q16) = (c.divk(m8, 3), c.divk(m16, 3));
            (Some(card(&mut c, q8)), Some(card(&mut c, q16)))
        } else { (None, None) };
        self.kit.eval(host, &mut c);
        // the tables at the entries the sums punched
        let tab = |c: &Calc, t: &[i32], n: N| -> (i32, i32) { let i = c.cards(n)[0] as usize; (t[i], t[i + 1]) };
        // fine64: 8 the entry and the step times the part / 16
        let fine = |c: &mut Calc, (a, b): (i32, i32), p: N| -> N {
            let (na, nb) = (c.num(a as i64), c.num(b as i64));
            let step = c.lin(&[(nb, Some(p), false), (na, Some(p), true)]);
            let part = c.divk(step, 16);
            let part = c.within(part, 20_000);
            let eight = c.num(8);
            c.lin(&[(na, Some(eight), false), (part, None, false)])
        };
        let mut ys = vec![];
        let mut gcs = vec![];
        let mut grs = vec![];
        for (n, &fr) in frames.iter().enumerate() {
            let gtab = tab(&c, &d.grade, at[n].i);
            let gc = fine(&mut c, gtab, at[n].p);
            let y = if (F_LAUNCH..F_LAUNCH + 18).contains(&fr) {
                let kk = c.mul(at[n].k, at[n].k);
                let (a, b) = (c.num(25_920), c.num(3844));
                let s = c.lin(&[(at[n].k, Some(a), false), (kk, Some(b), true)]);
                let q = c.divk(s, 1000);
                let q = c.within(q, 5000);
                c.addk(q, 704)
            } else if (F_LAND..F_LAND + 6).contains(&fr) {
                let jn = { let b = c.num(F_LAND as i64); c.sub(fnums[n], b) };
                let six = c.num(6);
                let jj = c.mul(jn, jn);
                let s = c.lin(&[(jn, Some(six), false), (jj, None, true)]);
                let m = c.mulk(s, 128);
                let q = c.divk(m, 36);
                let q = c.within(q, 100);
                c.add(gc, q)
            } else { gc };
            let gr = if (F_LAUNCH..F_LAUNCH + 18).contains(&fr) {
                let m = c.mulk(at[n].k, 534);
                let q = c.divk(m, 10);
                let q = c.within(q, 2000);
                let h = c.num(180);
                c.sub(h, q)
            } else {
                let (na, nb) = (c.num(gtab.0 as i64), c.num(gtab.1 as i64));
                let s = c.lin(&[(nb, None, false), (na, None, true)]);
                let m = c.mulk(s, 125);
                let q = c.divk(m, 2);
                c.within(q, 2000)
            };
            ys.push(y);
            gcs.push(gc);
            grs.push(gr);
        }
        let sum = c.lin(&[(grs[0], None, false), (grs[1], None, false), (grs[2], None, false)]);
        let m = c.mulk(sum, 55);
        let t = c.divk(m, 300);
        let t = c.within(t, 200_000);
        let tilt = c.clamp(t, -260, 160);
        let m = c.mulk(tilt, 440);
        let q = c.divk(m, 1000);
        let q = c.within(q, 200);
        let h0 = c.addk(q, 152);
        let lift = if (F_LAUNCH..F_LAUNCH + 34).contains(&f) {
            let m = c.mulk(at[2].k, 250_000);
            let ang = c.divk(m, 17);
            let (s, _) = c.sin(ang);
            let m = c.mulk(s, 896);
            let q = c.divk(m, 9990);
            c.within(q, 100)
        } else { c.num(0) };
        let gcam = { let t = tab(&c, &d.grade, ci); fine(&mut c, t, cp) };
        let over = c.addk(gcam, 128);
        let both = c.add(ys[0], ys[1]);
        let half = c.divk(both, 2);
        let half = c.within(half, 200_000);
        let follow = c.lin(&[(half, None, false), (lift, None, false)]);
        let follow = c.addk(follow, 109);
        let h64 = c.max(over, follow);
        let (q, r) = (c.divk(h64, 8), c.modk(h64, 8));
        let q = c.within(q, 999);
        let (h8, hf, h0c) = (card(&mut c, q), card(&mut c, r), card(&mut c, h0));
        self.kit.eval(host, &mut c);
        let (h8v, hfv, h0v) = (c.cards(h8)[0], c.cards(hf)[0], c.cards(h0c)[0]);
        if std::env::var("ACCEL_CAM").is_ok() { eprintln!("cam {f} h8 {h8v} hf {hfv} h0 {h0v} y {:?} {:?} {:?} gr {:?} {:?} {:?}", c.cards(ys[0]), c.cards(ys[1]), c.cards(ys[2]), c.cards(grs[0]), c.cards(grs[1]), c.cards(grs[2])); }
        let ri = tab(&c, &d.road, at[2].i);
        let n = Nodes { y: ys[2], a64: at[2].a64, p: at[2].p, ri, h64, h0, gr: grs[2], e8, e16, cz: at[2].cz, gc: gcs[2] };
        Path64 { h8: h8v, hf: hfv, h0: h0v, c, n }
    }

    /// Our car (jump.rs's pose and draw): its place and turn in the camera's space, the kit's pixels at twice the
    /// size, four samples a pixel averaged over the frame.
    pub fn car(&self, hd: &Hd, host: &mut Host, cam: &Cam, f: i32, path: &mut Path64, px: &mut [Px]) -> Vec<super::accel_hd::Light> {
        let d = &hd.d;
        let c = &mut path.c;
        let n = &path.n;
        let num = |c: &mut Calc, v: i32| c.num(v as i64);
        // across, ahead and up from the camera (64ths)
        let (ra, rb) = (num(c, n.ri.0), num(c, n.ri.1));
        let step = c.lin(&[(rb, Some(n.p), false), (ra, Some(n.p), true)]);
        let part = c.divk(step, 16);
        let part = c.within(part, 20_000);
        let (eight, lane, x8) = (c.num(8), num(c, self.lane[f as usize]), num(c, cam.x8));
        let dx = c.lin(&[(ra, Some(eight), false), (part, None, false), (lane, Some(eight), false), (x8, Some(eight), true)]);
        let dz = n.a64;
        let dy = { let t = c.num(32); c.lin(&[(n.y, None, false), (t, None, false), (n.h64, None, true)]) };
        let (cc, ss) = (num(c, cam.c), num(c, cam.s));
        let bx = { let s = c.lin(&[(dx, Some(cc), false), (dz, Some(ss), true)]); let q = c.divk(s, 256); c.within(q, 5000) };
        let az = { let s = c.lin(&[(dx, Some(ss), false), (dz, Some(cc), false)]); let q = c.divk(s, 256); c.within(q, 5000) };
        let uy = { let m = c.mulk(dy, 25); let q = c.divk(m, 16); c.within(q, 5000) };
        let kk = { let t = c.num(192); let s = c.sub(t, n.h0); let m = c.mulk(s, 1000); let q = c.divk(m, 440); c.within(q, 2000) };
        let lift = c.lin_k(&[(kk, Some(az), false)]);
        let lift = c.within(lift, 5000);
        let py = c.add(uy, lift);
        let py = c.within(py, 5000);
        // the turn: the yaw from the tangents of the headings, the pitch from the grade
        let steer = { let (a, b) = (num(c, self.lane[f as usize + 1]), num(c, self.lane[(f as usize).max(1) - 1])); let s = c.sub(a, b); let m = c.mulk(s, 1000); let q = c.divk(m, 36); c.within(q, 2000) };
        let tcam = { let s = num(c, cam.slope); let m = c.mulk(s, 125); let q = c.divk(m, 12); c.within(q, 2000) };
        let tcar = { let s = c.sub(rb, ra); let m = c.mulk(s, 125); let q = c.divk(m, 2); c.within(q, 2000) };
        let ex = |c: &Calc, e: Option<N>| -> i32 { d.exp[(c.cards(e.unwrap())[0] as usize).min(191)] };
        let tan = c.lin(&[(tcar, None, false), (steer, None, false), (tcam, None, true)]);
        let m = c.mulk(tan, 159_155);
        let yaw = c.divk(m, 1000);
        let mut yaw = c.within(yaw, 990_000_000);
        if f >= F_LAND {
            let e = num(c, ex(c, n.e8));
            let m = c.mulk(e, 38_889);
            let w1 = c.divk(m, 1000);
            let w1 = c.within(w1, 40_000);
            let gn = { let (a, b) = (c.num(f as i64), c.num(F_LAND as i64)); c.sub(a, b) };
            let ang = c.mulk(gn, 175_070);
            let (s, _) = c.sin(ang);
            let p = c.mul(w1, s);
            let wag = c.divk(p, 999);
            let wag = c.within(wag, 40_000);
            yaw = c.add(yaw, wag);
        }
        let p1000 = if (F_LAUNCH..F_LAUNCH + 18).contains(&f) {
            let kn = { let (a, b) = (c.num(f as i64), c.num(F_LAUNCH as i64)); c.sub(a, b) };
            let (e18, h) = (c.num(18), c.num(18_000));
            let kg = c.mul(kn, n.gr);
            let ff = c.num(45);
            let s = c.lin(&[(e18, Some(h), false), (kn, Some(h), true), (kg, Some(ff), false)]);
            let q = c.divk(s, 1800);
            c.within(q, 3000)
        } else if f >= F_LAND {
            let e = num(c, ex(c, n.e16));
            let m = c.mulk(e, -1047);
            let c1 = c.divk(m, 10_000);
            let c1 = c.within(c1, 200);
            let gn = { let (a, b) = (c.num(f as i64), c.num(F_LAND as i64)); c.sub(a, b) };
            let ang = c.mulk(gn, 206_901);
            let (_, cs) = c.sin(ang);
            let p = c.mul(c1, cs);
            let q = c.divk(p, 999);
            let q = c.within(q, 200);
            c.add(n.gr, q)
        } else { n.gr };
        let m = c.mulk(p1000, 159_155);
        let q = c.divk(m, 1000);
        let q = c.within(q, 990_000_000);
        let pitch = c.neg(q);
        let r = Kit::turn(c, yaw, pitch);
        let thou = c.num(1000);
        let r1: [N; 3] = std::array::from_fn(|j| c.lin_k(&[(r[1][j], Some(thou), false), (kk, Some(r[2][j]), false)]));
        let place = Place { rot: [r[0], r1, r[2]], pos: [bx, py, az] };
        let a4 = { let q = c.divk(az, 25); let q = c.within(q, 999); card(c, q) };
        // the light in the sheared space: z less kk / 1000 of y (jump.rs's)
        let unshear = |c: &mut Calc, w: [i64; 3]| -> [N; 3] {
            let (x, y, z) = (c.num(w[0]), c.num(w[1]), c.num(w[2]));
            let z = c.lin_k(&[(z, Some(thou), false), (kk, Some(y), true)]);
            [x, y, z]
        };
        let (light, half) = (unshear(c, LIGHT), unshear(c, HALF));
        // (the software half draws it at 192 x 96: focal length 110)
        let soft = hd.soft.get();
        let view = View { focal10: if soft { F10 / 8 } else { F10 }, zref: c.num(ZREF), light, half };
        // our tail lamps: 0.70 either side, 2.14 back, at the pivot's height, turned and placed as the car
        let tails: Vec<[N; 3]> = [-70i64, 70].iter().map(|&x| {
            let v = [c.num(x), c.num(0), c.num(-214)];
            let pos = place.pos;
            let rot = place.rot;
            std::array::from_fn(|i| { let t = c.lin_k(&[(rot[i][0], Some(v[0]), false), (rot[i][1], Some(v[1]), false), (rot[i][2], Some(v[2]), false)]); let t = c.within(t, 1000); c.add(t, pos[i]) })
        }).collect();
        let tails: Vec<[N; 5]> = tails.into_iter().map(|w| screen(c, w)).collect();
        // the shadow, first
        self.shadow(hd, host, c, n, cam, kk, dx, f, px);
        let (_, winners) = self.kit.draw_pixels(host, c, &self.model, &place, &view, &self.mats, &self.face_mat, Shading::Flat, if soft { RW / 4 } else { 2 * RW }, if soft { RH / 4 } else { 2 * RH });
        if std::env::var("ACCEL_POSE").is_ok() { eprintln!("pose {f} yaw {:?} pitch {:?} p1000 {:?} pos {:?} rot {:?}", c.cards(yaw), c.cards(pitch), c.cards(p1000), place.pos.map(|n| c.cards(n).to_vec()), place.rot.map(|r| r.map(|n| c.cards(n).to_vec()))); }
        let a4 = c.cards(a4)[0];
        // each channel's tenths, worked out once for each 8-bit value
        {
            let mut want: Vec<(usize, i32)> = winners.iter().flat_map(|w| (0..3).map(move |ch| (ch, w.1[ch]))).filter(|k| !self.tenths.borrow().contains_key(k)).collect();
            sorter::distinct(&mut want, &[sorter::up(1), sorter::CARD], |&(ch, v)| [ch as i64, v as i64]);
            if !want.is_empty() {
                let out = hd.many_pub(host, "apart", &want.iter().map(|&(ch, v)| vec![0, v, SCALE[ch], 255]).collect::<Vec<_>>());
                let mut t = self.tenths.borrow_mut();
                for (k, o) in want.into_iter().zip(out) { t.insert(k, o[0]); }
            }
        }
        let t = self.tenths.borrow();
        let mut big: HashMap<usize, [i32; 3]> = HashMap::new();
        for (p, c3) in &winners { big.insert(*p as usize, [t[&(0, c3[0])], t[&(1, c3[1])], t[&(2, c3[2])]]); }
        let tail_lights = |c: &Calc| -> Vec<super::accel_hd::Light> { tails.iter().map(|t| {
            let v: Vec<i32> = t.iter().map(|&n| c.cards(n)[0]).collect();
            super::accel_hd::Light { x: [v[0], v[1]], y: [v[2], v[3]], a4: v[4], rgb: super::accel_hd::TAIL, k: super::accel_hd::TAIL_K, s: 175, kind: 4 }
        }).collect() };
        if soft {
            // each of its pixels the middle of our 4 x 4 block
            for (&p, &v) in &big { let (x, y) = (p % (RW / 4), p / (RW / 4)); px[(4 * y + 2) * RW + 4 * x + 2] = Px { c: v, a4 }; }
            return tail_lights(c);
        }
        let mut pixels: Vec<usize> = big.keys().map(|&p| (p / (2 * RW) / 2) * RW + (p % (2 * RW)) / 2).collect();
        sorter::distinct(&mut pixels, &[sorter::up(7)], |&p| [p as i64]);
        let samples = |i: usize| -> [usize; 4] { let (x, y) = (i % RW, i / RW); [(2 * y) * 2 * RW + 2 * x, (2 * y) * 2 * RW + 2 * x + 1, (2 * y + 1) * 2 * RW + 2 * x, (2 * y + 1) * 2 * RW + 2 * x + 1] };
        let mut out = vec![[0; 3]; pixels.len()];
        for ch in 0..3 {
            // four cards the same are their own mean (a run of one colour, copied); the rest averaged
            let fours: Vec<Vec<i32>> = pixels.iter().map(|&i| samples(i).iter().map(|s| big.get(s).map_or(px[i].c[ch], |v| v[ch])).collect()).collect();
            let differ = hd.differ(host, &fours.iter().flat_map(|v| [(v[1], v[0]), (v[2], v[0]), (v[3], v[0])]).collect::<Vec<_>>());
            let mixed: Vec<usize> = (0..fours.len()).filter(|&m| differ[3 * m..3 * m + 3].iter().any(|&d| d)).collect();
            let o = hd.many_pub(host, "aavg4", &mixed.iter().map(|&m| fours[m].clone()).collect::<Vec<_>>());
            for m in 0..fours.len() { out[m][ch] = fours[m][0]; }
            for (j, &m) in mixed.iter().enumerate() { out[m][ch] = o[j][0]; }
        }
        // the pixels our car covers, at least 3 of their 4 samples (athree), take its distance
        let covered = hd.many_pub(host, "athree", &pixels.iter().map(|&i| samples(i).iter().map(|s| big.contains_key(s) as i32).collect()).collect::<Vec<_>>());
        for (m, &i) in pixels.iter().enumerate() {
            px[i].c = out[m];
            if covered[m][0] == 1 { px[i].a4 = a4; }
        }
        tail_lights(c)
    }

    /// Our car's shadow (jump.rs's): the street under its middle, front and back on the screen, the ellipse's sizes
    /// and darkness by the sums; its axes (ashu), each pixel's place in it (aghs), its cover from the table (ashk)
    /// fetched by that, the pixel moved toward black (amix).
    #[allow(clippy::too_many_arguments)]
    fn shadow(&self, hd: &Hd, host: &mut Host, c: &mut Calc, n: &Nodes, cam: &Cam, kk: N, dx: N, _f: i32, px: &mut [Px]) {
        let d = &hd.d;
        let over = c.sub(n.y, n.gc);
        let k = { let t = c.num(256); let s = c.sub(t, over); let m = c.mulk(s, 192); let q = c.divk(m, 2560); let q = c.within(q, 100_000); c.clamp(q, 0, 19) };
        let k = card(c, k);
        // the street at dz = 0, 147 and -147 64ths along
        let places: Vec<(N, N, N)> = [0i64, 147, -147].iter().map(|&dz| {
            let dzn = c.num(dz);
            let z = c.add(n.cz, dzn);
            let (q, r) = (c.divk(z, 128), c.modk(z, 128));
            let q = c.within(q, 999);
            let (i, p) = (card(c, q), card(c, r));
            let along = c.add(n.a64, dzn);
            (i, p, along)
        }).collect();
        self.kit.eval(host, c);
        if c.cards(k)[0] == 0 { return; }
        let (cc, ss) = (c.num(cam.c as i64), c.num(cam.s as i64));
        let pts: Vec<[N; 3]> = places.iter().map(|&(i, p, along)| {
            let ix = c.cards(i)[0] as usize;
            let (na, nb) = (c.num(d.grade[ix] as i64), c.num(d.grade[ix + 1] as i64));
            let step = c.lin(&[(nb, Some(p), false), (na, Some(p), true)]);
            let part = c.divk(step, 16);
            let part = c.within(part, 20_000);
            let eight = c.num(8);
            let y = c.lin(&[(na, Some(eight), false), (part, None, false), (n.h64, None, true)]);
            let b = { let s = c.lin(&[(dx, Some(cc), false), (along, Some(ss), true)]); let q = c.divk(s, 256); c.within(q, 5000) };
            let a = { let s = c.lin(&[(dx, Some(ss), false), (along, Some(cc), false)]); let q = c.divk(s, 256); c.within(q, 5000) };
            let up = { let m = c.mulk(y, 25); let q = c.divk(m, 16); let q = c.within(q, 5000); let l = c.lin_k(&[(kk, Some(a), false)]); let l = c.within(l, 5000); c.add(q, l) };
            [b, up, a]
        }).collect();
        // middle across and down, the sizes
        let x16 = |c: &mut Calc, w: [N; 3]| -> N { let m = c.mulk(w[0], 7040); let q = c.div(m, w[2]); let q = c.within(q, 200_000); c.addk(q, 6144) };
        let y16 = |c: &mut Calc, w: [N; 3]| -> N { let m = c.mulk(w[1], 7040); let q = c.div(m, w[2]); let q = c.within(q, 200_000); let t = c.num(3072); c.sub(t, q) };
        let mx = x16(c, pts[0]);
        let (fy, by) = (y16(c, pts[1]), y16(c, pts[2]));
        let ma = { let q = c.divk(pts[0][2], 25); let q = c.within(q, 999); let one = c.num(1); c.max(q, one) };
        let cx = { let q = c.divk(mx, 16); let q = c.within(q, 5000); card(c, q) };
        let cy = { let s = c.add(fy, by); let h = c.divk(s, 2); let h = c.within(h, 200_000); let q = c.divk(h, 16); let q = c.within(q, 5000); card(c, q) };
        let rx8 = { let t = c.num(1280); let s = c.sub(t, over); let m = c.mulk(s, 1265); let dd = c.mulk(ma, 100); let q = c.div(m, dd); let q = c.within(q, 999); card(c, q) };
        let ry16 = { let s = c.sub(fy, by); let a = c.abs(s); let h = c.divk(a, 4); let h = c.within(h, 999); let lo = c.num(16); let m = c.max(h, lo); card(c, m) };
        self.kit.eval(host, c);
        let (k, cx, cy, rx8, ry16) = (c.cards(k)[0], c.cards(cx)[0], c.cards(cy)[0], c.cards(rx8)[0], c.cards(ry16)[0]);
        if std::env::var("ACCEL_SHADOW").is_ok() { eprintln!("shadow k {k} cx {cx} cy {cy} rx8 {rx8} ry16 {ry16} over {:?}", c.cards(over)); }
        let ux = self.kit_go(hd, host, "ashu", &[rx8, 128, 401]);
        let uy = self.kit_go(hd, host, "ashu", &[ry16, 128, 121]);
        let cover = self.kit_go(hd, host, "ashk", &[k]);
        // the ellipse's pixels on the screen: its columns either side of the middle, each row's places
        // (ashu's cards out from the middle, laid out both ways: the first at the ellipse's first column or row)
        let both = |u: &[i32]| -> Vec<i32> { if u.is_empty() { vec![] } else { u[1..].iter().rev().chain(u.iter()).copied().collect() } };
        let (bx, by) = (both(&ux), both(&uy));
        let (x0, y0) = (cx - (ux.len() as i32 - 1), cy - (uy.len() as i32 - 1));
        let cols: Vec<(i32, i32)> = bx.iter().enumerate().map(|(k, &u)| (x0 + k as i32, u)).filter(|&(x, _)| (0..RW as i32).contains(&x)).collect();
        if cols.is_empty() { return; }
        let row_u: Vec<i32> = cols.iter().map(|&(_, u)| u).collect();
        let mut dark: Vec<(usize, i32)> = vec![];
        for (k, &v) in by.iter().enumerate() {
            let y = y0 + k as i32;
            if !(0..RH as i32).contains(&y) { continue; }
            let e = hd.go_pub(host, "aghs", &[&[v, row_u.len() as i32], &row_u]);
            for (m, &v) in e.iter().enumerate() {
                if v >= 320 { continue; }
                let k = cover[v as usize];
                let i = y as usize * RW + cols[m].0 as usize;
                if k > 0 && hd.shown(i) { dark.push((i, k)); }
            }
        }
        for ch in 0..3 {
            let items: Vec<Vec<i32>> = dark.iter().map(|&(i, k)| vec![1, px[i].c[ch], 0, k]).collect();
            let out = hd.var_each_pub(host, "amix", &items);
            for (j, &(i, _)) in dark.iter().enumerate() { px[i].c[ch] = out[j]; }
        }
    }

    fn kit_go(&self, hd: &Hd, host: &mut Host, name: &str, deck: &[i32]) -> Vec<i32> { hd.go_pub(host, name, &[deck]) }
}

/// A place in the camera's space (sheared) on the screen (jump.rs's screen): across and down as pixel and 16ths,
/// its distance (quarters), each a card.
fn screen(c: &mut Calc, w: [N; 3]) -> [N; 5] {
    let m = c.mulk(w[0], 7040);
    let q = c.div(m, w[2]);
    let q = c.within(q, 200_000);
    let x16 = c.addk(q, 6144);
    let x16 = c.clamp(x16, -15840, 15840);
    let m = c.mulk(w[1], 7040);
    let q = c.div(m, w[2]);
    let q = c.within(q, 200_000);
    let t = c.num(3072);
    let y16 = c.sub(t, q);
    let y16 = c.clamp(y16, -15840, 15840);
    let a = c.divk(w[2], 25);
    let a = c.within(a, 999);
    let parts = |c: &mut Calc, v: N| -> (N, N) { let (q, r) = (c.divk(v, 16), c.modk(v, 16)); let q = c.within(q, 999); (card(c, q), card(c, r)) };
    let (xp, xf) = parts(c, x16);
    let (yp, yf) = parts(c, y16);
    let a = card(c, a);
    [xp, xf, yp, yf, a]
}

const SPARK: [i64; 3] = [310, 494, 133];

impl Jump {
    /// The sparks (jump.rs's sparks): each one's head and tail by the sums, its streak's places and amounts (asaxis,
    /// asfade), laid on where the pixel is no nearer (ageq, agsum); the ten brightest as lights.
    pub fn sparks(&self, hd: &Hd, host: &mut Host, cam: &Cam, f: i32, path: &mut Path64, px: &mut [Px]) -> Vec<super::accel_hd::Light> {
        let d = &hd.d;
        let live: Vec<[i32; 6]> = self.sparks.iter().zip(&self.spark_last).filter(|&(sp, &last)| sp[0] <= f && f <= last).map(|(sp, _)| *sp).collect();
        if live.is_empty() { return vec![]; }
        let c = &mut path.c;
        let kk = { let t = c.num(192); let s = c.sub(t, path.n.h0); let m = c.mulk(s, 1000); let q = c.divk(m, 440); c.within(q, 2000) };
        // where each started: our car's middle at its frame (as path's), its end along
        struct S { t: N, i: N, p: N, z0: N }
        let first: Vec<S> = live.iter().map(|sp| {
            let (fr, f0) = (c.num(f as i64), c.num(sp[0] as i64));
            let t = c.sub(fr, f0);
            let t = card(c, t);
            let a64 = if (F_LAUNCH..F_LAUNCH + 30).contains(&sp[0]) {
                // (the frames since the launch, on the calculator)
                let kn = { let s = c.num(sp[0] as i64); c.addk(s, -(F_LAUNCH as i64)) };
                let kn = c.within(kn, 29);
                let m = c.mulk(kn, 50_000);
                let ang = c.divk(m, 3);
                let (sn, _) = c.sin(ang);
                let m = c.mulk(sn, 1152);
                let off = c.divk(m, 9990);
                let off = c.within(off, 200);
                c.addk(off, 384)
            } else { c.num(384) };
            let fz = c.mulk(f0, 144);
            let cz = c.add(fz, a64);
            let (q, r) = (c.divk(cz, 128), c.modk(cz, 128));
            let q = c.within(q, 999);
            let (i, p) = (card(c, q), card(c, r));
            let end = c.num(sp[1] as i64);
            let z0 = c.add(cz, end);
            S { t, i, p, z0 }
        }).collect();
        self.kit.eval(host, c);
        // each one's head and tail: where along, the street's entry there
        struct P { x: N, z: N, up: N, i: N, p: N }
        let mut heads = vec![];
        for (n, sp) in live.iter().enumerate() {
            let s0 = &first[n];
            let ix = c.cards(s0.i)[0] as usize;
            let (ra, rb) = (c.num(d.road[ix] as i64), c.num(d.road[ix + 1] as i64));
            let step = c.lin(&[(rb, Some(s0.p), false), (ra, Some(s0.p), true)]);
            let part = c.divk(step, 16);
            let part = c.within(part, 20_000);
            let (eight, lane) = (c.num(8), c.num(self.lane[sp[0] as usize] as i64));
            let x0 = c.lin(&[(ra, Some(eight), false), (part, None, false), (lane, Some(eight), false)]);
            // the spark's two times along its flight, 12 t and 12 t - 9 (not before 0), on the calculator
            let t = c.cards(s0.t)[0] as i64;
            let (tnow, tback) = { let tn = c.num(t); let a = c.mulk(tn, 12); let b = c.addk(a, -9); let zero = c.num(0); let b = c.max(b, zero); let b = c.within(b, 12 * t.abs().max(1)); (a, b) };
            let pts: Vec<P> = [tnow, tback].iter().map(|&tn| {
                let (vx, vz, vy) = (c.num(sp[2] as i64), c.num(sp[3] as i64), c.num(sp[4] as i64));
                let mx = c.mul(vx, tn); let qx = c.divk(mx, 144); let qx = c.within(qx, 5000);
                let x = c.add(x0, qx);
                let mz = c.mul(vz, tn); let qz = c.divk(mz, 144); let qz = c.within(qz, 5000);
                let z = c.add(s0.z0, qz);
                let my = c.mul(vy, tn); let qy = c.divk(my, 144); let qy = c.within(qy, 5000);
                let tt2 = c.mul(tn, tn); let mg = c.mulk(tt2, 640); let qg = c.divk(mg, 20736); let qg = c.within(qg, 5000);
                let three = c.num(3);
                let up = c.lin(&[(three, None, false), (qy, None, false), (qg, None, true)]);
                let up = c.abs(up);
                let (q, r) = (c.divk(z, 128), c.modk(z, 128));
                let q = c.within(q, 999);
                let (i, p) = (card(c, q), card(c, r));
                P { x, z, up, i, p }
            }).collect();
            heads.push(pts);
        }
        self.kit.eval(host, c);
        let (cc, ss) = (c.num(cam.c as i64), c.num(cam.s as i64));
        let mut outs = vec![];
        for (n, sp) in live.iter().enumerate() {
            let mut scr = vec![];
            let mut aa = vec![];
            for pt in &heads[n] {
                let ix = c.cards(pt.i)[0] as usize;
                let (ga, gb) = (c.num(d.grade[ix] as i64), c.num(d.grade[ix + 1] as i64));
                let step = c.lin(&[(gb, Some(pt.p), false), (ga, Some(pt.p), true)]);
                let part = c.divk(step, 16);
                let part = c.within(part, 20_000);
                let eight = c.num(8);
                let y = c.lin(&[(pt.up, None, false), (ga, Some(eight), false), (part, None, false), (path.n.h64, None, true)]);
                let x8 = c.num(cam.x8 as i64);
                let dx = c.lin(&[(pt.x, None, false), (x8, Some(eight), true)]);
                let fz = { let a = c.num(f as i64); c.mulk(a, 144) };
                let dz = c.sub(pt.z, fz);
                let b = { let s = c.lin(&[(dx, Some(cc), false), (dz, Some(ss), true)]); let q = c.divk(s, 256); c.within(q, 5000) };
                let a = { let s = c.lin(&[(dx, Some(ss), false), (dz, Some(cc), false)]); let q = c.divk(s, 256); c.within(q, 5000) };
                let up = { let m = c.mulk(y, 25); let q = c.divk(m, 16); let q = c.within(q, 5000); let l = c.lin_k(&[(kk, Some(a), false)]); let l = c.within(l, 5000); c.add(q, l) };
                // nearer than a unit: left out (the flag the sums punch)
                let hundred = c.num(100);
                let near = c.lt(a, hundred);
                let flag = { let (o, z) = (c.num(1), c.num(0)); c.sel(near, o, z) };
                aa.push(flag);
                // the screen's places only where it is far enough (a divisor of at least 100)
                let safe = c.max(a, hundred);
                scr.push(screen(c, [b, up, safe]));
            }
            let t = c.cards(first[n].t)[0] as usize;
            let (heat, bt) = (c.num(sp[5] as i64), c.num(self.bright[t] as i64));
            let m = c.mul(heat, bt);
            let bright = c.divk(m, 1000);
            let bright = c.within(bright, 999);
            let bright = card(c, bright);
            outs.push((scr, aa, bright));
        }
        self.kit.eval(host, c);
        let mut adds: Vec<[Vec<i32>; 3]> = vec![Default::default(); px.len()];
        let mut lit: Vec<(i32, [i32; 4])> = vec![];
        for (scr, aa, bright) in &outs {
            if c.cards(aa[0])[0] == 1 || c.cards(aa[1])[0] == 1 { continue; }
            let h: Vec<i32> = scr[0].iter().map(|&n| c.cards(n)[0]).collect();
            let tl: Vec<i32> = scr[1].iter().map(|&n| c.cards(n)[0]).collect();
            let bright = c.cards(*bright)[0];
            let (xa, ya, xb, yb, ha4) = (h[0], h[2], tl[0], tl[2], h[4]);
            if [xa, ya, xb, yb].iter().any(|v| !(-989..=989).contains(v)) { continue; }
            let dd = hd.go_pub(host, "asum", &[&[2, xb, xa, -1, yb, ya, -1]]);
            let ab = hd.go_pub(host, "aflabs", &[&[dd[0], dd[1]]]);
            let n = ab[0];
            if std::env::var("ACCEL_SPARKS").is_ok() { eprintln!("spark {xa} {ya} {xb} {yb} {n} {bright} {ha4}"); }
            let xs = hd.go_pub(host, "asaxis", &[&[xa, dd[0], n]]);
            let ys = hd.go_pub(host, "asaxis", &[&[ya, dd[1], n]]);
            let base = hd.go_pub(host, "aamt", &[&[1, bright, SPARK[0] as i32, SPARK[1] as i32, SPARK[2] as i32]]);
            let ws: Vec<Vec<i32>> = [16, 7].iter().flat_map(|&w| base.iter().map(move |&b| vec![0, b, w, 10])).collect();
            let amt = hd.many_pub(host, "apart", &ws);
            let fades: Vec<Vec<i32>> = amt.iter().map(|a| hd.go_pub(host, "asfade", &[&[a[0], n]])).collect();
            let near = hd.go_pub(host, "asum", &[&[1, ha4, 1, -1]])[0];
            // the places on the screen, and the pixel right of each
            let mut at = vec![];
            for j in 0..=n as usize { for ox in 0..2 {
                let (x, y) = (xs[j] + ox as i32, ys[j]);
                if (0..RW as i32).contains(&x) && (0..RH as i32).contains(&y) { at.push((y as usize * RW + x as usize, j, ox)); }
            } }
            let ok = hd.many_pub(host, "ageq", &at.iter().map(|&(i, _, _)| vec![px[i].a4, near]).collect::<Vec<_>>());
            for (m, &(i, j, ox)) in at.iter().enumerate() {
                if ok[m][0] == 0 { continue; }
                for ch in 0..3 { let v = fades[3 * ox + ch][j]; if v > 0 { adds[i][ch].push(v); } }
            }
            lit.push((bright, [h[0], h[1], h[2], h[3]]));
        }
        for ch in 0..3 {
            let at: Vec<usize> = (0..px.len()).filter(|&i| !adds[i][ch].is_empty()).collect();
            let items: Vec<Vec<i32>> = at.iter().map(|&i| { let mut it = vec![px[i].c[ch], adds[i][ch].len() as i32]; it.extend(&adds[i][ch]); it }).collect();
            let out = hd.var_each_pub(host, "agsum", &items);
            for (n, &i) in at.iter().enumerate() { px[i].c[ch] = out[n]; }
        }
        // the ten brightest light up, where the head is on the screen over something 30 units away or more
        sorter::sort(&mut lit, &[sorter::down(6)], |l| [l.0 as i64]);
        let top: Vec<&(i32, [i32; 4])> = lit.iter().take(10).filter(|l| (0..RW as i32).contains(&l.1[0]) && (0..RH as i32).contains(&l.1[2])).collect();
        let far = hd.many_pub(host, "ageq", &top.iter().map(|l| vec![px[l.1[2] as usize * RW + l.1[0] as usize].a4, 120]).collect::<Vec<_>>());
        let s = hd.many_pub(host, "apart", &top.iter().map(|l| vec![0, l.0, 3, 10]).collect::<Vec<_>>());
        top.iter().enumerate().filter(|&(m, _)| far[m][0] == 1).map(|(m, l)| super::accel_hd::Light { x: [l.1[0], l.1[1]], y: [l.1[2], l.1[3]], a4: 999, rgb: [255, 170, 60], k: [243, 329, 57], s: s[m][0], kind: 3 }).collect()
    }
}
