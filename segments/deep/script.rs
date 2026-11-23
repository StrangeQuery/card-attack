// The dive's script: every frame's numbers (the camera's place, its forward, right and up, the
// light, the phases, the jellyfish, kelp, seagrass and fish, the torch's aim, the ending's slowing), written once
// against the train's operations (segments/train/ops.rs): the reference does each natively, the host with the
// train's machines on CARDIAC. The host only fetches by an index (the path's samples, the printed tables), routes
// by a punched flag and keeps the creatures a machine found near enough.
//
// Units: angles in billionths of a turn (nt), carried as millionths of a turn (hi) and the nt past them (lo) where
// they go round; places, sines and fractions in millionths; time by the frame f, story time f / 6 s, real time
// f / 12 s. Products of longer numbers go by their cards (mulsum), divisions by large numbers a step at a time,
// every one rounded down unless it says so; rounding to the nearest is half away from 0, as the prototype's.
// Sines and arctangents are from printed tables, between entries by the rest.
//
// The printed numbers (data/script.txt, from data/make_script.py): the lips the diver swims to and the creatures'
// fixed numbers. Included after ops.rs.

pub const SC_FRAMES: usize = 353;
/// The dive's own path; past it the camera holds over the knob, slowing from frame SLOW0 to SLOW1.
pub const SC_DIVE: usize = 309;
const SLOW0: usize = 290;
const SLOW1: usize = 308;
/// The path's samples: one every 1/48 s of story time.
const STEPS: usize = 62 * 48;
const MI: i64 = 1_000_000;
const TURN: i64 = 1_000_000_000;

/// A rate of w radians a second as nt a frame (12 frames a second, or 6 for story time), and a radians as nt.
fn rate(w: f64, fps: f64) -> i64 { (w / std::f64::consts::TAU / fps * 1e9).round() as i64 }
fn rad(a: f64) -> i64 { (a / std::f64::consts::TAU * 1e9).round() as i64 }

/// The printed numbers and tables.
pub struct ScriptData {
    marks: Vec<(i64, i64)>, jelly: Vec<Vec<i64>>, kelp: Vec<Vec<i64>>, grass: Vec<Vec<i64>>, fish: Vec<Vec<i64>>,
    /// the sine every 10,000th of a turn from 0 to 1.25 turns (millionths); the arctangent of r every thousandth
    /// from 0 to 1.001 (nt)
    sin: Tab, atan: Tab,
}

impl ScriptData {
    pub fn parse(text: &str) -> ScriptData {
        let tau = std::f64::consts::TAU;
        let mut d = ScriptData { marks: vec![], jelly: vec![], kelp: vec![], grass: vec![], fish: vec![],
            sin: Tab { name: "sine", data: (0..=12_500).map(|k| ((k as f64 / 10_000.0 * tau).sin() * 1e6).round() as i64).collect(), w: 0 },
            atan: Tab { name: "atan", data: (0..=1001).map(|k| ((k as f64 / 1000.0).atan() / tau * 1e9).round() as i64).collect(), w: 0 } };
        for l in text.lines() {
            let mut w = l.split_whitespace();
            let key = w.next().unwrap_or("");
            let v: Vec<i64> = w.map(|x| x.parse().unwrap()).collect();
            match key {
                "marks" => d.marks = v.chunks(2).map(|c| (c[0], c[1])).collect(),
                "jelly" => d.jelly.push(v), "kelp" => d.kelp.push(v), "grass" => d.grass.push(v), "fish" => d.fish.push(v),
                _ => {}
            }
        }
        d
    }
}

// --- sums --------------------------------------------------------------------------------------------------------

fn kv(c: i64, n: usize) -> V { vec![c; n] }
/// k1 a1 + k2 a2 + ... + c.
fn ls<O: Ops>(o: &mut O, t: &[(&V, i64)], c: i64) -> V {
    let mut terms: Vec<(A, Option<A>)> = t.iter().map(|(a, k)| (v(a), Some(K(*k)))).collect();
    if c != 0 { terms.push((K(c), None)); }
    o.lin(&terms)
}
/// fl(x, d), the quotient a long (d up to 99,999).
fn fl1<O: Ops>(o: &mut O, x: &V, d: i64) -> V { o.div(v(x), K(d), 0) }
/// fl(x, d) for any quotient: by thousands while d is a multiple of 1000 past 99,999 (then the rest is a long and
/// one division does), else a card of x at a time.
fn fld<O: Ops>(o: &mut O, x: &V, d: i64) -> V {
    if d > 99_999 {
        assert!(d % 1000 == 0, "a divisor of {d}");
        let (mut x, mut d) = (fl1(o, x, 1000), d / 1000);
        while d > 99_999 { assert!(d % 1000 == 0, "a divisor of {d}"); x = fl1(o, &x, 1000); d /= 1000; }
        return fl1(o, &x, d);
    }
    let x1 = fl1(o, x, 1000);
    let x0 = ls(o, &[(x, 1), (&x1, -1000)], 0);
    let q1 = fl1(o, &x1, d);
    let r = ls(o, &[(&x1, 1000), (&q1, -1000 * d), (&x0, 1)], 0);
    let q2 = fl1(o, &r, d);
    ls(o, &[(&q1, 1000), (&q2, 1)], 0)
}
fn mdd<O: Ops>(o: &mut O, x: &V, d: i64) -> V { let q = fld(o, x, d); ls(o, &[(x, 1), (&q, -d)], 0) }
/// x / d to the nearest, half up (x not below 0 where it matters), d even.
fn rndp<O: Ops>(o: &mut O, x: &V, d: i64) -> V { let h = ls(o, &[(x, 1)], d / 2); fld(o, &h, d) }
/// x / d to the nearest, half away from 0, d even.
fn rnd<O: Ops>(o: &mut O, x: &V, d: i64) -> V {
    let up = rndp(o, x, d);
    let nx = ls(o, &[(x, -1)], 0);
    let dn = rndp(o, &nx, d);
    let dn = ls(o, &[(&dn, -1)], 0);
    let neg = o.lt(v(x), K(0));
    o.sel(&neg, v(&dn), v(&up))
}
/// k x / d to the nearest, half away from 0, d even: from |x| = q d + r, k q + fl((k r + d / 2) / d), signed.
fn rndk<O: Ops>(o: &mut O, x: &V, k: i64, d: i64) -> V {
    let (ax, s) = abs(o, x);
    let q = fld(o, &ax, d);
    let r = ls(o, &[(&ax, 1), (&q, -d)], 0);
    let r = ls(o, &[(&r, k)], d / 2);
    let t = fl1(o, &r, d);
    let p = ls(o, &[(&q, k), (&t, 1)], 0);
    let n = neg(o, &p);
    o.sel(&s, v(&n), v(&p))
}
fn neg<O: Ops>(o: &mut O, x: &V) -> V { ls(o, &[(x, -1)], 0) }
fn abs<O: Ops>(o: &mut O, x: &V) -> (V, V) { let n = neg(o, x); let s = o.lt(v(x), K(0)); (o.sel(&s, v(&n), v(x)), s) }

/// A number's three cards from the right (the last one signed).
fn cards<O: Ops>(o: &mut O, a: &V) -> [V; 3] {
    let q1 = fl1(o, a, 1000);
    let a0 = ls(o, &[(a, 1), (&q1, -1000)], 0);
    let a2 = fl1(o, &q1, 1000);
    let a1 = ls(o, &[(&q1, 1), (&a2, -1000)], 0);
    [a0, a1, a2]
}
/// fl(sum of a b / 1000^k) exactly: the products card by card, the cards below 1000^k carried up.
fn mulsum<O: Ops>(o: &mut O, pairs: &[(&V, &V)], k: usize) -> V {
    let cs: Vec<([V; 3], [V; 3])> = pairs.iter().map(|(a, b)| (cards(o, a), cards(o, b))).collect();
    let mut c: Vec<V> = (0..5).map(|m| {
        let mut terms: Vec<(A, Option<A>)> = vec![];
        for (a, b) in &cs { for i in 0..3 { if m >= i && m - i < 3 { terms.push((v(&a[i]), Some(v(&b[m - i])))); } } }
        o.lin(&terms)
    }).collect();
    for m in 0..k {
        let carry = fl1(o, &c[m], 1000);
        c[m + 1] = ls(o, &[(&c[m + 1], 1), (&carry, 1)], 0);
    }
    let terms: Vec<(&V, i64)> = (k..5).map(|m| (&c[m], 1000i64.pow((m - k) as u32))).collect();
    ls(o, &terms, 0)
}
fn mul6<O: Ops>(o: &mut O, a: &V, b: &V) -> V { mulsum(o, &[(a, b)], 2) }

/// The smoothstep from frame n0 over d frames (millionths): n = f - n0 held to 0..d, fl(n^2 (3 d - 2 n) 10^6 / d^3),
/// the division by d, then by d^2 a card at a time.
fn ease<O: Ops>(o: &mut O, f: &V, n0: i64, d: i64) -> V {
    let n = ls(o, &[(f, 1)], -n0);
    let n = o.clamp(v(&n), 0, d);
    let whole = o.lt(v(&n), K(d));
    let n = o.clamp(v(&n), 0, d - 1);
    let t = ls(o, &[(&n, -2)], 3 * d);
    let n2 = o.lin(&[(v(&n), Some(v(&n)))]);
    let x = o.lin(&[(v(&n2), Some(v(&t)))]);
    // x = a d + b; x 10^6 / d^3 = (a 10^6 + fl(b 10^6 / d)) / d^2
    let a = fl1(o, &x, d);
    let b = ls(o, &[(&x, 1), (&a, -d)], 0);
    let c = o.div(v(&b), K(d), 2);
    let dd = d * d;
    // a 10^6 = q d^2 + r, r by two cards' worth of remainders
    let q = o.div(v(&a), K(dd), 2);
    let a3 = ls(o, &[(&a, 1000)], 0);
    let q3 = fl1(o, &a3, dd);
    let r1 = ls(o, &[(&a3, 1), (&q3, -dd)], 0);
    let r1k = ls(o, &[(&r1, 1000)], 0);
    let q4 = fl1(o, &r1k, dd);
    let r = ls(o, &[(&r1k, 1), (&q4, -dd)], 0);
    let s = ls(o, &[(&r, 1), (&c, 1)], 0);
    let e = fl1(o, &s, dd);
    let e = ls(o, &[(&q, 1), (&e, 1)], 0);
    o.sel(&whole, v(&e), K(MI))
}

// --- angles ------------------------------------------------------------------------------------------------------

/// An angle as millionths of a turn (0..999,999) and the nt past them (0..999).
struct Ang { hi: V, lo: V }
/// sum of n c (n small, per item; c constants in nt) + c0, round the turn.
fn ang_lin<O: Ops>(o: &mut O, t: &[(&V, i64)], c0: i64) -> Ang {
    let h: Vec<(&V, i64)> = t.iter().map(|&(n, c)| (n, c.div_euclid(1000))).collect();
    let l: Vec<(&V, i64)> = t.iter().map(|&(n, c)| (n, c.rem_euclid(1000))).collect();
    let lsum = ls(o, &l, c0.rem_euclid(1000));
    let l1 = fl1(o, &lsum, 1000);
    let lo = ls(o, &[(&lsum, 1), (&l1, -1000)], 0);
    let mut h = h;
    h.push((&l1, 1));
    let hs = ls(o, &h, c0.div_euclid(1000));
    Ang { hi: o.turns(v(&hs)), lo }
}
fn ang_nt<O: Ops>(o: &mut O, a: &V) -> Ang {
    let a1 = fl1(o, a, 1000);
    let lo = ls(o, &[(a, 1), (&a1, -1000)], 0);
    Ang { hi: o.turns(v(&a1)), lo }
}
/// sin (off 0) or cos (off 2500), millionths: the table's entry and the next, between by the hundred-thousandths
/// past it.
fn wave<O: Ops>(o: &mut O, d: &ScriptData, g: &Ang, offs: &[i64]) -> Vec<V> {
    let i = fl1(o, &g.hi, 100);
    let r = ls(o, &[(&g.hi, 1), (&i, -100)], 0);
    let r = ls(o, &[(&r, 1000), (&g.lo, 1)], 0);
    offs.iter().map(|&off| {
        let s0 = o.tabo(&d.sin, off, v(&i));
        let s1 = o.tabo(&d.sin, off + 1, v(&i));
        let ds = ls(o, &[(&s1, 1), (&s0, -1)], 0);
        let p = o.lin(&[(v(&ds), Some(v(&r)))]);
        let p = fld(o, &p, 100_000);
        ls(o, &[(&s0, 1), (&p, 1)], 0)
    }).collect()
}
fn sincos<O: Ops>(o: &mut O, d: &ScriptData, g: &Ang) -> (V, V) { let mut w = wave(o, d, g, &[0, 2500]); let c = w.pop().unwrap(); (w.pop().unwrap(), c) }
fn sin1<O: Ops>(o: &mut O, d: &ScriptData, g: &Ang) -> V { wave(o, d, g, &[0]).pop().unwrap() }
/// The steps of n a turn (60, 64) nearest the angle, half up, n A / 10^9 = (n hi + fl(n lo / 1000)) / 10^6.
fn steps<O: Ops>(o: &mut O, g: &Ang, n: i64) -> V {
    let l = ls(o, &[(&g.lo, n)], 0);
    let l = fl1(o, &l, 1000);
    let s = ls(o, &[(&g.hi, n), (&l, 1)], MI / 2);
    let s = fld(o, &s, MI);
    mdd(o, &s, n)
}
/// atan r (nt), r in millionths 0..999,999.
fn atan_r<O: Ops>(o: &mut O, d: &ScriptData, r: &V) -> V {
    let i = fl1(o, r, 1000);
    let f = ls(o, &[(r, 1), (&i, -1000)], 0);
    let t0 = o.tab(&d.atan, v(&i));
    let t1 = o.tabo(&d.atan, 1, v(&i));
    let dt = ls(o, &[(&t1, 1), (&t0, -1)], 0);
    let p = o.lin(&[(v(&dt), Some(v(&f)))]);
    let p = fl1(o, &p, 1000);
    ls(o, &[(&t0, 1), (&p, 1)], 0)
}
/// fl(num 10^6 / den) for 0 <= num < den < 99,900,000: long division a digit at a time, each digit the count of
/// the multiples of den the rest reaches.
fn ratio6<O: Ops>(o: &mut O, num: &V, den: &V) -> V {
    let js: Vec<V> = (1..=9).map(|j| ls(o, &[(den, j)], -1)).collect();
    let mut r = num.clone();
    let mut q = kv(0, num.len());
    for _ in 0..6 {
        let r10 = ls(o, &[(&r, 10)], 0);
        let flags: Vec<V> = js.iter().map(|jd| o.lt(v(jd), v(&r10))).collect();
        let t: Vec<(&V, i64)> = flags.iter().map(|f| (f, 1)).collect();
        let digit = ls(o, &t, 0);
        let dd = o.lin(&[(v(&digit), Some(v(den)))]);
        r = ls(o, &[(&r10, 1), (&dd, -1)], 0);
        q = ls(o, &[(&q, 10), (&digit, 1)], 0);
    }
    q
}
/// The angle of (b, a) (nt), b > 0, both under 99,900,000 in size: from the ratio of the smaller to the larger.
fn atan2<O: Ops>(o: &mut O, d: &ScriptData, a: &V, b: &V) -> V {
    let (aa, an) = abs(o, a);
    let sw = o.lt(v(b), v(&aa));
    let num = o.sel(&sw, v(b), v(&aa));
    let den = o.sel(&sw, v(&aa), v(b));
    let less = o.lt(v(&num), v(&den));
    let num = o.sel(&less, v(&num), K(0));
    let r = ratio6(o, &num, &den);
    let t = atan_r(o, d, &r);
    let t = o.sel(&less, v(&t), K(TURN / 8));
    let u = ls(o, &[(&t, -1)], TURN / 4);
    let t = o.sel(&sw, v(&u), v(&t));
    let nt = neg(o, &t);
    o.sel(&an, v(&nt), v(&t))
}

// --- the path ----------------------------------------------------------------------------------------------------

/// The diver's path every 1/48 s: (y, z) in millionths and the pitch (nt). A straight line to the next lip, 12
/// above it, at 7 a second; the turn at each lip eased twice (each step a 52.8th of the way).
fn path<O: Ops>(o: &mut O, d: &ScriptData) -> Vec<(i64, i64, i64)> {
    let (mut y, mut z, mut dir1, mut dir) = (vec![8 * MI], vec![0i64], vec![0i64], vec![0i64]);
    let mut k = 1;
    let mut p = Vec::with_capacity(STEPS);
    for _ in 0..STEPS {
        while k < d.marks.len() - 1 && o.lt(v(&z), K(d.marks[k].0))[0] == 0 { k += 1; }
        let (mz, my) = d.marks[k];
        let want = if o.lt(K(my), v(&y))[0] == 1 {
            let dy = ls(o, &[(&y, 1), (&vec![my], -1)], 0);
            let dz = ls(o, &[(&z, -1)], mz);
            atan2(o, d, &dy, &dz)
        } else { vec![0] };
        // a 52.8th of the way: rnd(5 (want - dir1), 264)
        let e = ls(o, &[(&want, 1), (&dir1, -1)], 0);
        let e = rndk(o, &e, 5, 264);
        dir1 = ls(o, &[(&dir1, 1), (&e, 1)], 0);
        let e = ls(o, &[(&dir1, 1), (&dir, -1)], 0);
        let e = rndk(o, &e, 5, 264);
        dir = ls(o, &[(&dir, 1), (&e, 1)], 0);
        p.push((y[0], z[0], dir[0]));
        let g = ang_nt(o, &dir);
        let (s, c) = sincos(o, d, &g);
        let dz = ls(o, &[(&c, 7)], 24);
        let dz = fl1(o, &dz, 48);
        let dy = ls(o, &[(&s, 7)], 24);
        let dy = fl1(o, &dy, 48);
        z = ls(o, &[(&z, 1), (&dz, 1)], 0);
        y = ls(o, &[(&y, 1), (&dy, -1)], 0);
    }
    p
}

/// The drift across at the frames and its rate (millionths, and millionths a second): three slow sweeps, eased in
/// over story time 1..6 s.
fn drift<O: Ops>(o: &mut O, d: &ScriptData, f: &V) -> (V, V) {
    let k = ease(o, f, 6, 30);
    let a = [ang_lin(o, &[(f, rate(0.19, 6.0))], 0), ang_lin(o, &[(f, rate(0.33, 6.0))], rad(1.3)), ang_lin(o, &[(f, rate(0.61, 6.0))], rad(2.1))];
    let sc: Vec<(V, V)> = a.iter().map(|g| sincos(o, d, g)).collect();
    let x = ls(o, &[(&sc[0].0, 45), (&sc[1].0, 25), (&sc[2].0, 12)], 0);    // ten-millionths
    let x = mul6(o, &x, &k);
    let x = fld(o, &x, 10);
    let c: Vec<V> = [(0, 855), (1, 825), (2, 732)].iter().map(|&(i, w)| ls(o, &[(&sc[i].1, w)], 0)).collect();
    let vx = mulsum(o, &[(&c[0], &k), (&c[1], &k), (&c[2], &k)], 3);
    (x, vx)
}

/// Items' values by frame: each item's frame's.
fn per(src: &V, at: &[usize]) -> V { at.iter().map(|&i| src[i]).collect() }

/// The dive: every frame's numbers, as the script's lines (host/src/fx/deep.rs reads them).
pub fn script<O: Ops>(o: &mut O, d: &ScriptData) -> Vec<Vec<i64>> {
    o.stage("path");
    let p = path(o, d);
    let at = |i: i64| p[(i as usize).min(STEPS - 1)];
    o.stage("camera");
    let n = SC_DIVE;
    let fs: V = (0..n as i64).collect();
    let cam_y0: V = fs.iter().map(|&f| at(8 * f).0).collect();
    let z00: V = fs.iter().map(|&f| at(8 * f).1).collect();
    let dir: V = fs.iter().map(|&f| at(8 * f).2).collect();
    let e = ease(o, &fs, 66, 24);
    let lk = mul6(o, &kv(rad(0.12), n), &e);
    let look = ls(o, &[(&dir, 1), (&lk, 1)], 0);
    // the drift, and where it is at 51 s (the portal) and 44 s
    let mut fd = fs.clone();
    fd.extend([306, 264]);
    let (xd, vxd) = drift(o, d, &fd);
    let (px, x44) = (xd[n], xd[n + 1]);
    let (mut x0, vx) = (xd[..n].to_vec(), vxd[..n].to_vec());
    let (py, pz, _) = at(8 * 306);
    let (y44, z44, _) = at(8 * 264);
    let (avx, sneg) = abs(o, &vx);
    let r = fl1(o, &avx, 7);
    let lean = atan_r(o, d, &r);
    let nl = neg(o, &lean);
    let lean = o.sel(&sneg, v(&nl), v(&lean));
    let mut yaw = rndk(o, &lean, 8, 10);
    let rl = rndk(o, &lean, 35, 100);
    let mut roll = neg(o, &rl);
    let (mut cam_y, mut z0) = (cam_y0.clone(), z00.clone());
    let mut pitch = look.clone();
    // once the portal shows the diver turns to its middle, and swims straight at it from 44 s
    {
        let at: Vec<usize> = (265..n).collect();
        let m = at.len();
        let f: V = at.iter().map(|&i| i as i64).collect();
        let u = ls(o, &[(&f, 1)], -264);
        let full = o.lt(v(&u), K(42));
        let u = o.clamp(v(&u), 0, 41);
        let u = o.div(v(&u), K(42), 2);
        let u = o.sel(&full, v(&u), K(MI));
        let k = ease(o, &f, 264, 12);
        let go = |o: &mut O, c: &mut V, a: i64, b: i64| {
            let cur = per(c, &at);
            let w = ls(o, &[(&kv(b, m), 1), (&kv(a, m), -1)], 0);
            let w = mul6(o, &w, &u);
            let w = ls(o, &[(&w, 1), (&cur, -1)], a);
            let w = mul6(o, &w, &k);
            let w = ls(o, &[(&cur, 1), (&w, 1)], 0);
            for (j, &i) in at.iter().enumerate() { c[i] = w[j]; }
        };
        go(o, &mut x0, x44, px);
        go(o, &mut cam_y, y44, py);
        go(o, &mut z0, z44, pz);
    }
    {
        let at: Vec<usize> = (256..n).collect();
        let f: V = at.iter().map(|&i| i as i64).collect();
        let aim = ease(o, &f, 255, 18);
        let (xs, ys, zs) = (per(&x0, &at), per(&cam_y, &at), per(&z0, &at));
        let ax = ls(o, &[(&xs, -1)], px);
        let ay = ls(o, &[(&ys, -1)], py);
        let az = ls(o, &[(&zs, -1)], pz);
        let near = o.lt(v(&az), K(MI / 2));
        let az = o.sel(&near, K(MI / 2), v(&az));
        let yaw_p = atan2(o, d, &ax, &az);
        // how far off along the ground: across and along, onto the way to the portal
        let g = ang_nt(o, &yaw_p);
        let (sy, cy) = sincos(o, d, &g);
        let h = mulsum(o, &[(&ax, &sy), (&az, &cy)], 2);
        let nay = neg(o, &ay);
        let look_p = atan2(o, d, &nay, &h);
        let blend = |o: &mut O, a: &V, b: &V| { let w = ls(o, &[(b, 1), (a, -1)], 0); let w = mul6(o, &w, &aim); ls(o, &[(a, 1), (&w, 1)], 0) };
        let (pl, yl, rl) = (per(&look, &at), per(&yaw, &at), per(&roll, &at));
        let pb = blend(o, &pl, &look_p);
        let yb = blend(o, &yl, &yaw_p);
        let left = ls(o, &[(&aim, -1)], MI);
        let rb = mul6(o, &rl, &left);
        for (j, &i) in at.iter().enumerate() { pitch[i] = pb[j]; yaw[i] = yb[j]; roll[i] = rb[j]; }
    }
    let (gp, gy, gr) = (ang_nt(o, &pitch), ang_nt(o, &yaw), ang_nt(o, &roll));
    let ((sp, cp), (sy, cy), (sr, cr)) = (sincos(o, d, &gp), sincos(o, d, &gy), sincos(o, d, &gr));
    let nsp = neg(o, &sp);
    let nsy = neg(o, &sy);
    let fv = [mul6(o, &sy, &cp), nsp.clone(), mul6(o, &cy, &cp)];
    let rt0 = [cy.clone(), kv(0, n), nsy.clone()];
    let u0x = mul6(o, &fv[1], &rt0[2]);
    let (a, b) = (mul6(o, &fv[2], &rt0[0]), mul6(o, &fv[0], &rt0[2]));
    let u0y = ls(o, &[(&a, 1), (&b, -1)], 0);
    let u0z = mul6(o, &fv[1], &rt0[0]);
    let u0z = neg(o, &u0z);
    let u0 = [u0x, u0y, u0z];
    let mut rt = vec![];
    let mut u = vec![];
    for i in 0..3 {
        let (a, b) = (mul6(o, &rt0[i], &cr), mul6(o, &u0[i], &sr));
        rt.push(ls(o, &[(&a, 1), (&b, -1)], 0));
        let (a, b) = (mul6(o, &u0[i], &cr), mul6(o, &rt0[i], &sr));
        u.push(ls(o, &[(&a, 1), (&b, 1)], 0));
    }
    // s x / 10^6 to the nearest as (s / 100) x / 10^4 (s 100, 300): the same number, the sums smaller
    let card = |o: &mut O, x: &V, s: i64| { let y = ls(o, &[(x, s / 100)], 0); rnd(o, &y, 10_000) };
    let f300: Vec<V> = fv.iter().map(|x| card(o, x, 300)).collect();
    let r300: Vec<V> = rt.iter().map(|x| card(o, x, 300)).collect();
    let u300: Vec<V> = u.iter().map(|x| card(o, x, 300)).collect();
    let f100: Vec<V> = fv.iter().map(|x| card(o, x, 100)).collect();
    let r100: Vec<V> = rt.iter().map(|x| card(o, x, 100)).collect();
    let u100: Vec<V> = u.iter().map(|x| card(o, x, 100)).collect();

    o.stage("light");
    // the light, in hundred-millionths
    let s1 = ease(o, &fs, 30, 36);
    let f102 = o.clamp(v(&fs), 0, 102);
    let s2 = ease(o, &f102, 72, 168);
    let s3 = ease(o, &fs, 102, 174);
    let l8 = ls(o, &[(&s1, -55), (&s2, -45), (&s3, -25)], 100 * MI);
    // 32 l8 / 10^8 = l8 / 3,125,000; 10 l8 / 10^8 = l8 / 10^7
    let light = rndp(o, &l8, 3_125_000);
    let light10 = rndp(o, &l8, 10_000_000);
    let x = rnd(o, &x0, 10_000);
    let y = rnd(o, &cam_y, 10_000);
    let z = rnd(o, &z0, 100_000);
    let (yw, yp) = o.divmod(v(&y), K(100));
    let (zw, zp) = o.divmod(v(&z), K(10));
    let y10 = fl1(o, &y, 10);
    let snow0 = mdd(o, &y10, 140);
    let snow1 = mdd(o, &z, 140);
    let l7 = ls(o, &[(&light, 7)], 0);
    let l7 = fl1(o, &l7, 10);
    let dark = ls(o, &[(&l7, -1)], 32);
    let l4 = ls(o, &[(&light, 4)], 0);
    let l4 = fl1(o, &l4, 10);
    let show = ls(o, &[(&l4, 1)], 11);
    let w = ls(o, &[(&z0, -1)], pz);
    let plane = rnd(o, &w, 100_000);
    let w = ls(o, &[(&x0, 1), (&kv(px, n), -1)], 0);
    let from0 = rnd(o, &w, 10_000);
    let w = ls(o, &[(&cam_y, 1), (&kv(py, n), -1)], 0);
    let from1 = rnd(o, &w, 10_000);
    let w = ls(o, &[(&fs, -22_918)], 0);
    let sw0 = rnd(o, &w, 600);
    let sw0 = mdd(o, &sw0, 600);
    let w = ls(o, &[(&fs, -9_549)], 0);
    let sw1 = rnd(o, &w, 600);
    let sw1 = mdd(o, &sw1, 600);
    // the jellyfish's glow: 32 (1 - 0.6 light) ease, (1 - 0.6 light) in 500,000,000ths
    let ej = ease(o, &fs, 153, 12);
    let jl = ls(o, &[(&l8, -3)], TURN / 2);
    let jl = mul6(o, &jl, &ej);
    let jelly_light = rndp(o, &jl, 15_625_000);
    let glow_tips = o.lt(v(&l8), K(50 * MI));
    let height10 = fld(o, &cam_y, 100_000);
    let (phase, shaft, pulse, wave) = phases(o, &fs);

    o.stage("jellyfish");
    let jellies = jellies(o, d, &at, &fs, &x0, &cam_y, &z0);
    o.stage("kelp");
    let kelp = kelp(o, d, &fs, &x0, &z0);
    o.stage("grass");
    let grass = grass(o, d, &fs, &x0, &z0);
    o.stage("fish");
    let fish = fish(o, d, &fs, &l8, &x0, &cam_y, &z0);
    o.stage("torch");
    let aim = torch(o, d, &fs, &f100, &r100, &u100);

    let mut lines: Vec<Vec<i64>> = (0..n).map(|i| {
        let mut l = vec![0, x[i], yw[i], yp[i], zw[i], zp[i], f300[0][i], f300[1][i], f300[2][i], r300[0][i], r300[1][i], r300[2][i],
            u300[0][i], u300[1][i], u300[2][i], light[i], light10[i], phase[0][i], phase[1][i], phase[2][i], shaft[0][i], shaft[1][i],
            pulse[i], dark[i], f100[0][i], f100[1][i], f100[2][i], r100[0][i], r100[1][i], r100[2][i], u100[0][i], u100[1][i], u100[2][i],
            snow0[i], snow1[i], show[i], -1, 0, plane[i], from0[i], from1[i], sw0[i], sw1[i], 0, jelly_light[i], wave[i], jellies[i].len() as i64];
        for j in &jellies[i] { l.extend(j); }
        l.extend([glow_tips[i], height10[i], light[i], kelp[i].len() as i64]);
        for c in &kelp[i] { l.extend(c); }
        l.push(grass[i].len() as i64);
        for t in &grass[i] { l.extend(t); }
        l.push(fish[i].len() as i64);
        for t in &fish[i] { l.extend(t); }
        l.extend(aim[i]);
        l
    }).collect();

    o.stage("ending");
    ending(o, &mut lines);
    lines
}

/// The caustics' three phases, the god rays' two, the glowing spots' pulse and the tentacles' wave, by frame.
fn phases<O: Ops>(o: &mut O, fs: &V) -> ([V; 3], [V; 2], V, V) {
    let by = |o: &mut O, k: i64| { let a = ls(o, &[(fs, k)], 0); let a = fl1(o, &a, 1000); mdd(o, &a, 60) };
    let p0 = by(o, 1353);
    let p1 = by(o, 1035);
    let p1 = ls(o, &[(&p1, -1)], 60);
    let p1 = mdd(o, &p1, 60);
    let p2 = by(o, 796);
    let pulse = by(o, 2387);
    let w = ls(o, &[(fs, 12)], 0);
    let s0 = mdd(o, &w, 6000);
    let w = ls(o, &[(fs, -8)], 0);
    let s1 = mdd(o, &w, 6000);
    let w = ls(o, &[(fs, -23_873)], 0);
    let w = rnd(o, &w, 12_000);
    let wave = mdd(o, &w, 60);
    ([p0, p1, p2], [s0, s1], pulse, wave)
}

/// The jellyfish near the camera, per frame (from story time 25.5 s): drifting along the dive either side of the
/// path, above and below it; those within 98 units.
fn jellies<O: Ops>(o: &mut O, d: &ScriptData, at: &dyn Fn(i64) -> (i64, i64, i64), fs: &V, x0: &V, cy: &V, z0: &V) -> Vec<Vec<Vec<i64>>> {
    let mut out = vec![vec![]; fs.len()];
    let (mut fi, mut ks) = (vec![], vec![]);
    for (i, &f) in fs.iter().enumerate() { if f >= 153 { for k in 0..60 { fi.push(i); ks.push(k as i64); } } }
    let f = per(fs, &fi);
    let (xs, ys, zs) = (per(x0, &fi), per(cy, &fi), per(z0, &fi));
    let jd = |c: usize| -> V { ks.iter().map(|&k| d.jelly[k as usize][c]).collect() };
    let (py, pz): (V, V) = ks.iter().map(|&k| { let p = at(d.jelly[k as usize][0]); (p.0, p.1) }).unzip();
    let (bx, by) = (jd(1), jd(2));
    let ga = ang_lin(o, &[(&f, rate(0.3, 12.0)), (&ks, rad(1.0))], 0);
    let sa = sin1(o, d, &ga);
    let sa = ls(o, &[(&sa, 3)], 0);
    let sa = fld(o, &sa, 2);
    let jx = ls(o, &[(&bx, 1), (&sa, 1), (&xs, -1)], 0);
    let gb = ang_lin(o, &[(&f, rate(0.5, 12.0)), (&ks, rad(1.0))], 0);
    let sb = sin1(o, d, &gb);
    let sb = ls(o, &[(&sb, 6)], 0);
    let sb = fl1(o, &sb, 10);
    let jy = ls(o, &[(&py, 1), (&by, 1), (&sb, 1), (&ys, -1)], 0);
    let jz = ls(o, &[(&pz, 1), (&zs, -1)], 0);
    let rel: Vec<V> = [jx, jy, jz].iter().map(|c| rnd(o, c, 100_000)).collect();
    // only those within 98 units: a place along any of the camera's vectors then fits a card
    let r2 = o.lin(&[(v(&rel[0]), Some(v(&rel[0]))), (v(&rel[1]), Some(v(&rel[1]))), (v(&rel[2]), Some(v(&rel[2])))]);
    let far = o.lt(K(980 * 980), v(&r2));
    let keep: Vec<usize> = (0..ks.len()).filter(|&j| far[j] == 0).collect();
    let f = per(&f, &keep);
    let kk = per(&ks, &keep);
    let gp = ang_lin(o, &[(&f, rate(2.6, 12.0)), (&kk, rad(1.0))], 0);
    let sp = sin1(o, d, &gp);
    let sp = ls(o, &[(&sp, 15)], 0);
    let sp = fl1(o, &sp, 100);
    let pulse = ls(o, &[(&sp, 1)], MI);
    let w = ls(o, &[(&pulse, 100)], 0);
    let rad_ = rndp(o, &w, MI);
    // 75 / pulse to the nearest: fl((150 10^6 + pulse) / 2 pulse), from a guess by the thousands, put right by the
    // remainder
    let nn = ls(o, &[(&pulse, 1)], 150 * MI);
    let p2 = ls(o, &[(&pulse, 2)], 0);
    let nn3 = fl1(o, &nn, 1000);
    let p23 = fl1(o, &p2, 1000);
    let mut q = o.div(v(&nn3), v(&p23), 0);
    for _ in 0..2 {
        let qp = o.lin(&[(v(&q), Some(v(&p2)))]);
        let rem = ls(o, &[(&nn, 1), (&qp, -1)], 0);
        let low = o.lt(v(&rem), K(0));
        let qd = ls(o, &[(&q, 1)], -1);
        q = o.sel(&low, v(&qd), v(&q));
        let qp = o.lin(&[(v(&q), Some(v(&p2)))]);
        let rem = ls(o, &[(&nn, 1), (&qp, -1)], 0);
        let high = o.lt(v(&rem), v(&p2));
        let qu = ls(o, &[(&q, 1)], 1);
        q = o.sel(&high, v(&q), v(&qu));
    }
    for (e, &j) in keep.iter().enumerate() {
        let k = ks[j] as usize;
        let jd = &d.jelly[k];
        let mut l = vec![rel[0][j], rel[1][j], rel[2][j], jd[3], jd[4], rad_[e], q[e]];
        l.extend(&jd[5..13]);
        l.push(k as i64);
        out[fi[j]].push(l);
    }
    out
}

/// The kelp: clumps in the forest up top, going round with the camera every 45 units; three stalks to a clump,
/// swaying, their blades fluttering.
fn kelp<O: Ops>(o: &mut O, d: &ScriptData, fs: &V, x0: &V, z0: &V) -> Vec<Vec<Vec<i64>>> {
    let n = fs.len();
    let mut out = vec![vec![]; n];
    let (mut fi, mut ks) = (vec![], vec![]);
    for i in 0..n { for k in 0..d.kelp.len() { fi.push(i); ks.push(k as i64); } }
    let (zs, xs) = (per(z0, &fi), per(x0, &fi));
    let base: V = ks.iter().map(|&k| d.kelp[k as usize][0]).collect();
    let round = fld(o, &zs, 45 * MI);
    let kz = ls(o, &[(&base, 1), (&round, 45 * MI)], 0);
    let back = ls(o, &[(&zs, 1)], -5 * MI);
    let behind = o.lt(v(&kz), v(&back));
    let kz2 = ls(o, &[(&kz, 1)], 45 * MI);
    let kz = o.sel(&behind, v(&kz2), v(&kz));
    let beyond = o.lt(K(140 * MI), v(&kz));
    let behind = o.lt(v(&kz), v(&back));
    let keep: Vec<usize> = (0..ks.len()).filter(|&j| beyond[j] == 0 && behind[j] == 0).collect();
    let (kz, xs, zs, kk) = (per(&kz, &keep), per(&xs, &keep), per(&zs, &keep), per(&ks, &keep));
    let f: V = keep.iter().map(|&j| fs[fi[j]]).collect();
    let kx: V = kk.iter().map(|&k| d.kelp[k as usize][1]).collect();
    let row = rndp(o, &kz, MI);
    let w = ls(o, &[(&kx, 1), (&xs, -1)], 0);
    let ax = rnd(o, &w, 100_000);
    let w = ls(o, &[(&row, MI), (&zs, -1)], 0);
    let az = rnd(o, &w, 100_000);
    // the stalks' sways: (real time 1.2 + k + st) as steps of 60
    let m = kk.len();
    let (mut sf, mut sk) = (vec![], vec![]);
    for j in 0..m { for st in 0..3 { sf.push(f[j]); sk.push(kk[j] + st); } }
    let gs = ang_lin(o, &[(&sf, rate(1.2, 12.0)), (&sk, rad(1.0))], 0);
    let sway = steps(o, &gs, 60);
    // the blades: side (0.75 + 0.25 sin(real time 2 + 1.7 i + k)) - a quarter turn, as steps of 64
    let (mut bf, mut bi, mut bk, mut side) = (vec![], vec![], vec![], vec![]);
    for j in 0..m {
        let kd = &d.kelp[kk[j] as usize];
        for st in 0..3 { for i in 1..=15 { bf.push(f[j]); bi.push(i as i64); bk.push(kk[j]); side.push(kd[4 + 32 * st + 2 + 2 * (i - 1)]); } }
    }
    let gb = ang_lin(o, &[(&bf, rate(2.0, 12.0)), (&bi, rad(1.7)), (&bk, rad(1.0))], 0);
    let flt = sin1(o, d, &gb);
    let flt = fl1(o, &flt, 4);
    let tilt = mul6(o, &kv(rad(1.0), flt.len()), &flt);
    let tilt = ls(o, &[(&tilt, 1)], rad(0.75));
    // the angle is below 0: its size, a quarter turn less side tilt, to the nearest step, then 64 less
    let pos = o.lin(&[(v(&side), Some(v(&tilt)))]);
    let size = ls(o, &[(&pos, -1)], TURN / 4);
    let g = ang_nt(o, &size);
    let st64 = steps(o, &g, 64);
    let st64 = ls(o, &[(&st64, -1)], 64);
    let st64 = mdd(o, &st64, 64);
    for j in 0..m {
        let kd = &d.kelp[kk[j] as usize];
        let mut l = vec![kd[2], kd[3], row[j], ax[j], az[j]];
        for st in 0..3 {
            l.extend([kd[4 + 32 * st], kd[5 + 32 * st], sway[3 * j + st]]);
            for i in 0..15 { l.extend([st64[45 * j + 15 * st + i], kd[4 + 32 * st + 3 + 2 * i]]); }
        }
        out[fi[keep[j]]].push(l);
    }
    out
}

/// The seagrass: short tufts on the seabed close by, going round with the camera every 20 units.
fn grass<O: Ops>(o: &mut O, d: &ScriptData, fs: &V, x0: &V, z0: &V) -> Vec<Vec<Vec<i64>>> {
    let n = fs.len();
    let mut out = vec![vec![]; n];
    let (mut fi, mut ks) = (vec![], vec![]);
    for i in 0..n { for k in 0..d.grass.len() { fi.push(i); ks.push(k as i64); } }
    let (zs, xs) = (per(z0, &fi), per(x0, &fi));
    let base: V = ks.iter().map(|&k| d.grass[k as usize][0]).collect();
    let round = fld(o, &zs, 20 * MI);
    let gz = ls(o, &[(&base, 1), (&round, 20 * MI)], 0);
    let behind = o.lt(v(&gz), v(&zs));
    let gz2 = ls(o, &[(&gz, 1)], 20 * MI);
    let gz = o.sel(&behind, v(&gz2), v(&gz));
    let beyond = o.lt(K(140 * MI), v(&gz));
    let keep: Vec<usize> = (0..ks.len()).filter(|&j| beyond[j] == 0).collect();
    let (gz, xs, zs, kk) = (per(&gz, &keep), per(&xs, &keep), per(&zs, &keep), per(&ks, &keep));
    let f: V = keep.iter().map(|&j| fs[fi[j]]).collect();
    let gx: V = kk.iter().map(|&k| d.grass[k as usize][1]).collect();
    let row = rndp(o, &gz, MI);
    let w = ls(o, &[(&gx, 1), (&xs, -1)], 0);
    let ax = rnd(o, &w, 100_000);
    let w = ls(o, &[(&row, MI), (&zs, -1)], 0);
    let az = rnd(o, &w, 100_000);
    let g = ang_lin(o, &[(&f, rate(1.8, 12.0)), (&kk, rad(1.0))], 0);
    let ph = steps(o, &g, 60);
    for j in 0..kk.len() {
        let gd = &d.grass[kk[j] as usize];
        let mut l = vec![gd[2], gd[3], row[j], ax[j], az[j], ph[j]];
        l.extend(&gd[4..9]);
        out[fi[keep[j]]].push(l);
    }
    out
}

/// The fish: two schools crossing in front, each a leader on a curving path and fish in places round it, turned
/// with the leader's heading and wobbling.
fn fish<O: Ops>(o: &mut O, d: &ScriptData, fs: &V, l8: &V, x0: &V, cy: &V, z0: &V) -> Vec<Vec<Vec<i64>>> {
    let n = fs.len();
    let mut out = vec![vec![]; n];
    let dim = o.lt(v(l8), K(30 * MI));
    let mut first = 0;
    for (sk, &(z_mid, dir, t0, count)) in [(58i64, 1i64, 12i64, 40usize), (100, -1, 54, 24)].iter().enumerate() {
        // the frames it swims in: u = f - t0 twelfths of a second, 0..96, and light enough
        let fi: Vec<usize> = (0..n).filter(|&i| fs[i] >= t0 && fs[i] <= t0 + 96 && dim[i] == 0).collect();
        let m = fi.len();
        let u: V = fi.iter().map(|&i| fs[i] - t0).collect();
        let ones = kv(1, m);
        // the leader at u twelfths and e hundredths of a second more: across -22 + 6.5 s (13e6 / 2400 = 16250 / 3
        // millionths a hundredth of u's twelfths)
        let lead = |o: &mut O, e: i64| -> [V; 3] {
            let a = ls(o, &[(&u, 16_250 * 100)], 16_250 * 12 * e);
            let a = fld(o, &a, 3);
            let x = ls(o, &[(&a, dir)], -22 * MI * dir);
            let gy = ang_lin(o, &[(&u, rate(0.7, 12.0)), (&ones, e * rad(0.007))], 0);
            let y = sin1(o, d, &gy);
            let y = ls(o, &[(&y, 1)], 13 * MI / 2);
            let gz = ang_lin(o, &[(&u, rate(0.5, 12.0)), (&ones, e * rad(0.005) + sk as i64 * rad(1.0))], 0);
            let z = sin1(o, d, &gz);
            let z = ls(o, &[(&z, 4)], z_mid * MI);
            [x, y, z]
        };
        let p0 = lead(o, 0);
        let p1 = lead(o, 10);
        let ahead = lead(o, 30);
        let back = lead(o, -30);
        // the heading: the way the leader goes in the next tenth of a second, as an angle off its way across
        let dx = ls(o, &[(&p1[0], dir), (&p0[0], -dir)], 0);
        let dz = ls(o, &[(&p1[2], 1), (&p0[2], -1)], 0);
        let th = atan2(o, d, &dz, &dx);
        let g = ang_nt(o, &th);
        let (hz, hc) = sincos(o, d, &g);
        let hx = ls(o, &[(&hc, dir)], 0);
        let tz = ls(o, &[(&ahead[2], 1), (&back[2], -1)], 0);
        let (tz, _) = abs(o, &tz);
        let w = ls(o, &[(&tz, 10)], 0);
        let turning = fld(o, &w, 6);
        let (mut ff, mut fk, mut fj) = (vec![], vec![], vec![]);
        for j in 0..m { for k in 0..count { ff.push(fs[fi[j]]); fk.push(k as i64); fj.push(j); } }
        let pick = |src: &V| per(src, &fj);
        let (hx, hz, turning) = (pick(&hx), pick(&hz), pick(&turning));
        let (p0x, p0y, p0z) = (pick(&p0[0]), pick(&p0[1]), pick(&p0[2]));
        let fd = |c: usize| -> V { fk.iter().map(|&k| d.fish[first + k as usize][c]).collect() };
        let (ox, oy, oz) = (fd(0), fd(1), fd(2));
        let gw = ang_lin(o, &[(&ff, rate(3.0, 12.0)), (&fk, rad(1.0))], 0);
        let wob = sin1(o, d, &gw);
        let wob = fl1(o, &wob, 4);
        let (a, b, c) = (mul6(o, &ox, &hz), mul6(o, &oz, &hx), mul6(o, &wob, &hz));
        let cx = ls(o, &[(&p0x, 1), (&a, 1), (&b, 1), (&c, 1)], 0);
        let cyy = ls(o, &[(&p0y, 1), (&oy, 1)], 0);
        let (a, b, c) = (mul6(o, &ox, &hx), mul6(o, &oz, &hz), mul6(o, &wob, &hx));
        let cz = ls(o, &[(&p0z, 1), (&a, -1), (&b, 1), (&c, -1)], 0);
        let w = ls(o, &[(&hx, 11)], 0);
        let tx = fld(o, &w, 10);
        let w = ls(o, &[(&hz, 11)], 0);
        let tzz = fld(o, &w, 10);
        let tx = ls(o, &[(&cx, 1), (&tx, -1)], 0);
        let tz = ls(o, &[(&cz, 1), (&tzz, -1)], 0);
        let ci: Vec<usize> = fj.iter().map(|&j| fi[j]).collect();
        let cam: Vec<V> = [x0, cy, z0].iter().map(|c| per(c, &ci)).collect();
        let rel: Vec<V> = [(&cx, 0), (&cyy, 1), (&cz, 2), (&tx, 0), (&cyy, 1), (&tz, 2)].iter()
            .map(|&(p, c)| { let w = ls(o, &[(p, 1), (&cam[c], -1)], 0); rnd(o, &w, 100_000) }).collect();
        // 0.5 + 0.3 turning + 0.25 sin, at most 1 (hundred-millionths)
        let gf = ang_lin(o, &[(&ff, rate(2.0, 12.0)), (&fk, rad(0.7))], 0);
        let sf = sin1(o, d, &gf);
        let fl8 = ls(o, &[(&turning, 30), (&sf, 25)], 50 * MI);
        let fl8 = o.clamp(v(&fl8), -WIDE, 100 * MI);
        let flash = rndp(o, &fl8, 3_125_000);
        let far: Vec<V> = rel.iter().map(|r| { let (a, _) = abs(o, r); o.lt(K(560), v(&a)) }).collect();
        for e in 0..ff.len() {
            if far.iter().any(|x| x[e] == 1) { continue; }
            let mut l: Vec<i64> = rel.iter().map(|r| r[e]).collect();
            l.extend([flash[e], fk[e] + 100 * sk as i64]);
            out[ci[e]].push(l);
        }
        first += count;
    }
    out
}

/// The diver's torch leads: it points where the diver will face half a second on, swept twice as far, and wanders
/// a little on its own; the view follows it. Its beam's middle, aimed low at the floor ahead: x = 400 a.r / a.f +
/// 24 sin(1.3 t), y = 400 a.u / a.f - 28 + 10 sin(0.9 t), to the nearest, held to the screen's sides.
fn torch<O: Ops>(o: &mut O, d: &ScriptData, fs: &V, f100: &[V], r100: &[V], u100: &[V]) -> Vec<[i64; 2]> {
    let n = fs.len();
    let ai: Vec<usize> = (0..n).map(|i| (i + 6).min(n - 1)).collect();
    let a: Vec<V> = f100.iter().map(|c| per(c, &ai)).collect();
    let dot = |o: &mut O, b: &[V]| o.lin(&[(v(&a[0]), Some(v(&b[0]))), (v(&a[1]), Some(v(&b[1]))), (v(&a[2]), Some(v(&b[2])))]);
    let df = dot(o, f100);
    let df = o.clamp(v(&df), 1, WIDE);
    let dr = dot(o, r100);
    let du = dot(o, u100);
    let g1 = ang_lin(o, &[(fs, rate(1.3, 12.0))], 0);
    let s1 = sin1(o, d, &g1);
    let g2 = ang_lin(o, &[(fs, rate(0.9, 12.0))], 0);
    let s2 = sin1(o, d, &g2);
    // 400 a.r / a.f: its whole part and the rest in millionths; then the wander added, to the nearest
    let aim = |o: &mut O, dt: &V, s: &V, k: i64, c: i64| {
        let nn = ls(o, &[(dt, 400)], 0);
        let q = o.div(v(&nn), v(&df), 0);
        let qd = o.lin(&[(v(&q), Some(v(&df)))]);
        let r = ls(o, &[(&nn, 1), (&qd, -1)], 0);
        let g = o.div(v(&r), v(&df), 2);
        let w = ls(o, &[(&q, MI), (&g, 1), (s, k)], c);
        rnd(o, &w, MI)
    };
    let x = aim(o, &dr, &s1, 24, 0);
    let y = aim(o, &du, &s2, 10, -28 * MI);
    let x = o.clamp(v(&x), -150, 150);
    let y = o.clamp(v(&y), -80, 70);
    (0..n).map(|i| [x[i], y[i]]).collect()
}

/// The ending: no caption, portal or whiteout; past the dive's frames, its last frame's numbers with the water's
/// phases going on; from frame SLOW0 the camera slowing to a stop: frame n shows the camera fl((36 k - k^2) / 36)
/// frames on from SLOW0 and r 36ths more (k = n - SLOW0, at most 18), fl((d r + 18) / 36) of the change d in its
/// parts (x in hundredths, y in hundredths, z in tenths), its forward, right and up that frame's.
fn ending<O: Ops>(o: &mut O, lines: &mut Vec<Vec<i64>>) {
    let last = lines[SC_DIVE - 1].clone();
    let fs: V = (SC_DIVE as i64..SC_FRAMES as i64).collect();
    let (phase, shaft, pulse, wave) = phases(o, &fs);
    for j in 0..fs.len() {
        let mut l = last.clone();
        (l[17], l[18], l[19], l[20], l[21], l[22], l[45]) = (phase[0][j], phase[1][j], phase[2][j], shaft[0][j], shaft[1][j], pulse[j], wave[j]);
        lines.push(l);
    }
    let orig: Vec<Vec<i64>> = lines[..SC_DIVE].to_vec();
    let ks: V = (SLOW0..SC_FRAMES).map(|n| (n.min(SLOW1) - SLOW0) as i64).collect();
    let t = o.lin(&[(v(&ks), Some(v(&ks)))]);
    let t = ls(o, &[(&ks, 36), (&t, -1)], 0);
    let (whole, r) = o.divmod(v(&t), K(36));
    let a: Vec<usize> = whole.iter().map(|&w| SLOW0 + w as usize).collect();
    let b: Vec<usize> = a.iter().map(|&a| (a + 1).min(SC_DIVE - 1)).collect();
    for (size, w) in [(1i64, 1usize), (100, 2), (10, 4)] {
        let (va, vb): (V, V) = if size == 1 { (a.iter().map(|&i| orig[i][w]).collect(), b.iter().map(|&i| orig[i][w]).collect()) } else {
            let (wa, pa): (V, V) = (a.iter().map(|&i| orig[i][w]).collect(), a.iter().map(|&i| orig[i][w + 1]).collect());
            let (wb, pb): (V, V) = (b.iter().map(|&i| orig[i][w]).collect(), b.iter().map(|&i| orig[i][w + 1]).collect());
            (ls(o, &[(&wa, size), (&pa, 1)], 0), ls(o, &[(&wb, size), (&pb, 1)], 0))
        };
        let dv = ls(o, &[(&vb, 1), (&va, -1)], 0);
        let p = o.lin(&[(v(&dv), Some(v(&r)))]);
        let p = ls(o, &[(&p, 1)], 18);
        let p = fl1(o, &p, 36);
        let nv = ls(o, &[(&va, 1), (&p, 1)], 0);
        let (hi, lo) = if size == 1 { (nv.clone(), kv(0, nv.len())) } else { o.divmod(v(&nv), K(size)) };
        for (j, n) in (SLOW0..SC_FRAMES).enumerate() {
            lines[n][w] = hi[j];
            if size != 1 { lines[n][w + 1] = lo[j]; }
        }
    }
    for (j, n) in (SLOW0..SC_FRAMES).enumerate() {
        let src = &orig[a[j]];
        for c in (6..15).chain(24..33) { lines[n][c] = src[c]; }
    }
}
