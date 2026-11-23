// The night's cameras: the cab braking into the station, the platform, and the chase:
// one flight path from the platform to the stadium's crest, worked out at the start from the design's keyframes
// (a Catmull-Rom curve through them, resampled every metre, smoothed along its length 300
// times, resampled every quarter metre), the camera's speed along it frame by frame (eased, looking ahead to brake,
// smoothed), and each frame's aim (ahead, at the drone, the airship's screen, round the airship, the bowl), averaged
// over a second, and its bank. Millimetres in the film's unbent frame. Included after night.rs.

#[derive(Default)]
pub struct Path {
    /// the path every quarter metre (mm) and the distance along it at each (mm, its chords added)
    pub p: Vec<[i64; 3]>, pub l: Vec<i64>,
    /// the camera's distance along the path at each frame of the chase from F_CHASE (mm)
    pub d: Vec<i64>,
    /// where along it: the hand-off, the airship's screen read, the orbit's start and end, the street run's turn
    pub d_hand: i64, pub d_read: i64, pub d_orb0: i64, pub d_orb1: i64, pub d_turn: i64,
    /// the frame we are slowest in front of the airship's screen
    pub f_read: i64,
    /// worked out from it at the start: the click to VOL 11 (VOL_OFF, f_read + 20), the knob's first frame (f_read -
    /// 46), the smoothstep's constant for the 11's coming over the 10 frames from f_read - 30 (-1000 times that), the
    /// knob's click (f_read - 12) and the two frames after it, the join's design frames (VOL_OFF + CUT_N and JOIN on)
    pub f_cut: i64, pub f_vol_on: i64, pub k_v11: i64, pub f_click: [i64; 3], pub f_join: [i64; 2],
    /// and in tenths of a frame: the read's camera and the one a frame before it, the join's ends (and a frame
    /// before its last), and -100 times its first (the constant of the join's place)
    pub t_cut: [i64; 2], pub t_join: [i64; 3], pub k_join: i64,
    /// tables worked out at the start for finding a place's segment: each quarter metre's first point past its start
    /// (its place in l), the running lengths l, and l a point on (both past the end 1 m past the path's length)
    pub l_tabs: Vec<Tab>,
    /// the cab's place along the line at each frame 0..=F_STOP + 1 (mm), and its speed a frame (mm)
    pub za: Vec<i64>, pub zv: Vec<i64>,
}

// ---- vectors of numbers ---------------------------------------------------------------------------------------------

/// Running sums of any numbers (three cards): by doubling, each pass adding the sums so far from as far back again.
pub fn cumsum<O: Ops>(o: &mut O, xs: &[i64]) -> V {
    let mut s: V = xs.to_vec();
    let n = s.len();
    let mut k = 1;
    while k < n {
        let a: V = s[k..].to_vec();
        let b: V = s[..n - k].to_vec();
        let t = o.lin(&[(v(&a), None), (v(&b), None)]);
        s[k..].copy_from_slice(&t);
        k *= 2;
    }
    s
}

/// The whole square root rounded to the nearest (0..9,999,999).
pub fn rsqrt<O: Ops>(o: &mut O, s: &V) -> V {
    let r = o.sqrt(v(s));
    let rr = o.lin(&[(v(&r), Some(v(&r)))]);
    let rest = o.lin(&[(v(s), None), (v(&rr), Some(K(-1)))]);
    let up = o.lt(v(&r), v(&rest));
    o.lin(&[(v(&r), None), (v(&up), None)])
}

/// The power of ten, and its place (0 for 1), that a number under lim times it is scaled down by: the number
/// compared with lim, 10 lim, 100 lim and so on.
pub fn tens_under(x: i64, lim: i64) -> (i64, usize) { let (mut s, mut k) = (1, 0); while x >= lim * s { s *= 10; k += 1; } (s, k) }

/// The size of vectors (any unit; parts up to three cards): each scaled down by a power of ten (the host picks it by
/// comparing the largest part's size, which a machine works out, with 10,000, 100,000, ...) to under 10,000, the square root of the squares' sum over
/// 10,000 made fine by two Newton steps; the size in the scaled unit, and the scale.
pub fn size3<O: Ops>(o: &mut O, x: &V, y: &V, z: &V) -> (V, V, [V; 3]) {
    // sizes and the largest by comparisons (the parts may pass a long)
    let ab = |o: &mut O, a: &V| -> V { let m = o.lin(&[(v(a), Some(K(-1)))]); let neg = o.lt(v(a), K(0)); o.sel(&neg, v(&m), v(a)) };
    let mx = |o: &mut O, a: &V, b: &V| -> V { let f = o.lt(v(a), v(b)); o.sel(&f, v(b), v(a)) };
    let (ax, ay, az) = (ab(o, x), ab(o, y), ab(o, z));
    let m = mx(o, &ax, &ay);
    let m = mx(o, &m, &az);
    let sc: V = m.iter().map(|&m| tens_under(m, 10_000).0).collect();
    let p: [V; 3] = [x, y, z].map(|a| { if sc.iter().all(|&d| d == 1) { a.clone() } else { o.div(v(a), v(&sc), 0) } });
    let s = o.lin(&[(v(&p[0]), Some(v(&p[0]))), (v(&p[1]), Some(v(&p[1]))), (v(&p[2]), Some(v(&p[2])))]);
    let s4 = o.div(v(&s), K(10_000), 0);
    let r0 = o.sqrt(v(&s4));
    let mut r = o.lin(&[(v(&r0), Some(K(100))), (K(50), None)]);
    for _ in 0..2 {
        let q = o.div(v(&s), v(&r), 0);
        let t = o.lin(&[(v(&r), None), (v(&q), None)]);
        let t = o.div(v(&t), K(2), 0);
        r = o.max(v(&t), K(1));
    }
    (r, sc, p)
}

/// Vectors made units (thousandths).
pub fn nrm3<O: Ops>(o: &mut O, x: &V, y: &V, z: &V) -> [V; 3] {
    let (r, _, p) = size3(o, x, y, z);
    p.map(|a| { let t = o.lin(&[(v(&a), Some(K(1000)))]); o.div(v(&t), v(&r), 0) })
}

/// Vectors' sizes in their own unit (mm).
pub fn len3<O: Ops>(o: &mut O, x: &V, y: &V, z: &V) -> V {
    let (r, sc, _) = size3(o, x, y, z);
    o.lin(&[(v(&r), Some(v(&sc)))])
}

// ---- the flight path, at the start ----------------------------------------------------------------------------------

/// Points along a polyline (pts, its running lengths l) at distances s (mm, each held to the polyline): the host
/// finds each one's segment by the lengths the machines punched, the machines the place between its ends.
pub fn at_dist<O: Ops>(o: &mut O, pts: &[[i64; 3]], l: &[i64], s: &[i64]) -> [V; 3] {
    let seg: Vec<usize> = s.iter().map(|&x| {
        let i = l.partition_point(|&q| q <= x);
        i.clamp(1, l.len() - 1)
    }).collect();
    at_segs(o, pts, l, s, &seg)
}

/// Points along a polyline at distances s, each in its segment seg (from the point before seg to seg).
pub fn at_segs<O: Ops>(o: &mut O, pts: &[[i64; 3]], l: &[i64], s: &[i64], seg: &[usize]) -> [V; 3] {
    let l0: V = seg.iter().map(|&i| l[i - 1]).collect();
    let l1: V = seg.iter().map(|&i| l[i]).collect();
    let num = o.lin(&[(A::V(s), None), (v(&l0), Some(K(-1)))]);
    let den = o.lin(&[(v(&l1), None), (v(&l0), Some(K(-1)))]);
    let den = o.max(v(&den), K(1));
    // held to the ends, as np.interp holds
    let num = o.max(v(&num), K(0));
    let num = o.min(v(&num), v(&den));
    std::array::from_fn(|c| {
        let a: V = seg.iter().map(|&i| pts[i - 1][c]).collect();
        let b: V = seg.iter().map(|&i| pts[i][c]).collect();
        let d = o.lin(&[(v(&b), None), (v(&a), Some(K(-1)))]);
        let m = o.lin(&[(v(&d), Some(v(&num)))]);
        let m = o.div(v(&m), v(&den), 0);
        o.lin(&[(v(&a), None), (v(&m), None)])
    })
}

/// A polyline's running lengths (mm; its chords).
pub fn chords<O: Ops>(o: &mut O, pts: &[[i64; 3]]) -> V {
    let d: [V; 3] = std::array::from_fn(|c| { let a: V = pts[..pts.len() - 1].iter().map(|p| p[c]).collect(); let b: V = pts[1..].iter().map(|p| p[c]).collect(); o.lin(&[(v(&b), None), (v(&a), Some(K(-1)))]) });
    let s = o.lin(&[(v(&d[0]), Some(v(&d[0]))), (v(&d[1]), Some(v(&d[1]))), (v(&d[2]), Some(v(&d[2])))]);
    let len = rsqrt(o, &s);
    let mut l = vec![0];
    l.extend(cumsum(o, &len));
    l
}

/// The flight: the path, the places along it, the speed frame by frame,
/// the cab's braking.
pub fn flight<O: Ops>(o: &mut O, nw: &NightWorld) -> Path {
    let keys = &nw.path_keys;
    let n = keys.len() - 1;
    // the Catmull-Rom curve through the keys (cm) at 8001 places: the j-th in segment 37 j / 8000, u its rest
    // (thousandths); each segment's terms, then u's powers by Horner; mm
    let coef: Vec<[V; 4]> = (0..3).map(|c| {
        let p = |i: i64| -> V { (0..n as i64).map(|s| keys[(s + i).clamp(0, n as i64) as usize][c]).collect() };
        let (p0, p1, p2, p3) = (p(-1), p(0), p(1), p(2));
        let a = o.lin(&[(v(&p1), Some(K(2)))]);
        let b = o.lin(&[(v(&p2), None), (v(&p0), Some(K(-1)))]);
        let cc = o.lin(&[(v(&p0), Some(K(2))), (v(&p1), Some(K(-5))), (v(&p2), Some(K(4))), (v(&p3), Some(K(-1)))]);
        let d = o.lin(&[(v(&p0), Some(K(-1))), (v(&p1), Some(K(3))), (v(&p2), Some(K(-3))), (v(&p3), None)]);
        [a, b, cc, d]
    }).collect::<Vec<_>>().try_into().ok().unwrap_or_else(|| unreachable!());
    let nj = 8001usize;
    let segs: Vec<usize> = (0..nj).map(|j| ((n * j) / 8000).min(n - 1)).collect();
    let u8_: V = (0..nj).map(|j| if (n * j) / 8000 >= n { 8000 } else { ((n * j) % 8000) as i64 }).collect();
    let u = o.div(v(&u8_), K(8), 0);
    let raw: Vec<[i64; 3]> = {
        let pc: [V; 3] = std::array::from_fn(|c| {
            let g = |k: usize| -> V { segs.iter().map(|&s| coef[c][k][s]).collect() };
            let h = o.lin(&[(v(&g(3)), Some(v(&u)))]);
            let h = o.div(v(&h), K(1000), 0);
            let h = o.lin(&[(v(&h), None), (v(&g(2)), None)]);
            let h = o.lin(&[(v(&h), Some(v(&u)))]);
            let h = o.div(v(&h), K(1000), 0);
            let h = o.lin(&[(v(&h), None), (v(&g(1)), None)]);
            let h = o.lin(&[(v(&h), Some(v(&u)))]);
            let h = o.div(v(&h), K(1000), 0);
            o.lin(&[(v(&h), None), (v(&g(0)), None)]).iter().map(|&x| x).collect::<V>()
        });
        let pc: [V; 3] = std::array::from_fn(|c| o.lin(&[(v(&pc[c]), Some(K(5)))]));
        (0..nj).map(|j| [pc[0][j], pc[1][j], pc[2][j]]).collect()
    };
    // every metre along it
    let l = chords(o, &raw);
    let total = *l.last().unwrap();
    let s: V = (0..).map(|k| 1000 * k).take_while(|&x| x < total).collect();
    let q = at_dist(o, &raw, &l, &s);
    let mut qp: Vec<[i64; 3]> = (0..s.len()).map(|k| [q[0][k], q[1][k], q[2][k]]).collect();
    // smoothed along its length 300 times (a quarter, a half and a quarter), each step's share from 0 over its first
    // 2 m to all from 30 m on (its own lengths)
    let lq = chords(o, &qp);
    let lqd = o.div(v(&lq), K(100), 0);
    let taper = smoothv(o, &lqd, 20, 300);
    let m = qp.len();
    let tin: V = taper[1..m - 1].to_vec();
    for _ in 0..300 {
        for c in 0..3 {
            let a: V = qp[..m - 2].iter().map(|p| p[c]).collect();
            let b: V = qp[1..m - 1].iter().map(|p| p[c]).collect();
            let d: V = qp[2..].iter().map(|p| p[c]).collect();
            let lap = o.lin(&[(v(&a), None), (v(&b), Some(K(-2))), (v(&d), None)]);
            let t = o.lin(&[(v(&lap), Some(v(&tin))), (K(2000), None)]);
            let t = o.div(v(&t), K(4000), 0);
            let nb = o.lin(&[(v(&b), None), (v(&t), None)]);
            for k in 0..m - 2 { qp[k + 1][c] = nb[k]; }
        }
    }
    // every quarter metre
    let l2 = chords(o, &qp);
    let total = *l2.last().unwrap();
    let s: V = (0..).map(|k| 250 * k).take_while(|&x| x < total).collect();
    let p = at_dist(o, &qp, &l2, &s);
    let p: Vec<[i64; 3]> = (0..s.len()).map(|k| [p[0][k], p[1][k], p[2][k]]).collect();
    let l = chords(o, &p);
    let mut path = Path { p, l, ..Default::default() };
    // where along it the design's places are: the nearest point (by squared distances in tenths of a metre)
    let pl = |i: usize| -> [i64; 3] { nw.places_mm[i] };
    path.d_hand = nearest_along(o, &path, pl(place::HANDOFF), None);
    path.d_read = nearest_along(o, &path, pl(place::READ), None);
    path.d_orb0 = nearest_along(o, &path, pl(place::ORBIT0), None);
    let after = one(o.lin(&[(K(path.d_orb0), None), (K(200_000), None)]));
    path.d_orb1 = nearest_along(o, &path, pl(place::ORBIT1), Some(after));
    path.d_turn = nearest_along(o, &path, pl(place::TURN), None);
    path.d = profile(o, &nw.w.tb, &path);
    // the frame we read the airship's screen: the first whose distance reaches it
    path.f_read = F_CHASE + path.d.partition_point(|&x| x < path.d_read) as i64;
    let fr = path.f_read;
    path.f_cut = kl(o, &[(fr, None), (20, None)]);
    path.f_vol_on = kl(o, &[(fr, None), (-46, None)]);
    path.k_v11 = kl(o, &[(fr, Some(-1000)), (30_000, None)]);
    path.f_click = [-12, -11, -10].map(|d| kl(o, &[(fr, None), (d, None)]));
    path.f_join = [20 + CUT_N, 20 + CUT_N + JOIN].map(|d| kl(o, &[(fr, None), (d, None)]));
    path.t_cut = [200, 190].map(|d| kl(o, &[(fr, Some(10)), (d, None)]));
    path.t_join = [0, 10 * JOIN, 10 * JOIN - 10].map(|d| kl(o, &[(fr, Some(10)), (10 * (20 + CUT_N) + d, None)]));
    path.k_join = kl(o, &[(fr, Some(-1000)), (-1000 * (20 + CUT_N), None)]);
    // the path's quarter metres: the first point past each, and the lengths a point on (past the end, past any place)
    let (ql, last) = (path.l.len(), *path.l.last().unwrap());
    assert!((0..ql - 2).all(|i| path.l[i + 2] - path.l[i] >= 250), "two of the path's chords under a quarter metre");
    let past = kl(o, &[(last, None), (1000, None)]);
    path.l_tabs = vec![
        Tab { name: "l_at", data: (0..=last / 250).map(|b| path.l.partition_point(|&q| q <= 250 * b) as i64).collect(), w: 0 },
        Tab { name: "l", data: path.l.iter().copied().chain([past]).collect(), w: 0 },
        Tab { name: "l_next", data: path.l[1..].iter().copied().chain([past, past]).collect(), w: 0 },
    ];
    let (za, zv) = braking(o, &nw.w.tb);
    path.za = za;
    path.zv = zv;
    path
}

/// The distance along the path of its point nearest to p (mm), only from `after` on if given (else
/// all of it).
fn nearest_along<O: Ops>(o: &mut O, path: &Path, p: [i64; 3], after: Option<i64>) -> i64 {
    let d: [V; 3] = std::array::from_fn(|c| { let a: V = path.p.iter().map(|q| q[c]).collect(); let t = o.lin(&[(v(&a), None), (K(-p[c]), None)]); o.div(v(&t), K(100), 0) });
    let s = o.lin(&[(v(&d[0]), Some(v(&d[0]))), (v(&d[1]), Some(v(&d[1]))), (v(&d[2]), Some(v(&d[2])))]);
    // (held to 499,000 m^2: the nearest is always well within it)
    let s = o.clamp(v(&s), 0, 499_000);
    let mut key = o.lin(&[(v(&s), Some(K(-1)))]);
    if let Some(a) = after {
        let before = o.lt(v(&path.l), K(a));
        if before.iter().any(|&b| b == 0) {
            let far = o.lin(&[(v(&key), None), (K(-500_000), None)]);
            key = o.sel(&before, v(&far), v(&key));
        }
    }
    let i = nearest_big(o, &key);
    path.l[i]
}

/// The largest of any number of keys (a machine takes 999 at a time): each run of 999's, then theirs.
fn nearest_big<O: Ops>(o: &mut O, key: &V) -> usize {
    let runs: Vec<&[i64]> = key.chunks(999).collect();
    let best = o.nearest(&runs);
    let at: Vec<usize> = best.iter().enumerate().map(|(r, &b)| r * 999 + b).collect();
    let k: V = at.iter().map(|&i| key[i]).collect();
    at[o.nearest(&[&k])[0]]
}

/// The speed wanted at places along the path (mm a frame): flat out down the street to the turn under the stadium's
/// lip, then the climb's, slow in front of the airship's screen, the orbit's, the hand-off's.
fn wanted<O: Ops>(o: &mut O, path: &Path, d: &V) -> V {
    let t1 = path.d_turn - 10_000;
    let t2 = path.d_read - 30_000;
    let t3 = path.d_read + 12_000;
    let t4 = path.d_orb1;
    let t5 = path.d_orb1 + 10_000;
    let _ = (t1, t2, t3, t4, t5);
    let lim = |o: &mut O, base: i64, off: i64| -> i64 { one(o.lin(&[(K(base), None), (K(off), None)])) };
    let (t1, t2, t3, t4, t5) = (lim(o, path.d_turn, -10_000), lim(o, path.d_read, -30_000), lim(o, path.d_read, 12_000), path.d_orb1, lim(o, path.d_orb1, 10_000));
    let f1 = o.lt(v(d), K(t1));
    let f2 = o.lt(v(d), K(t2));
    let f3 = o.lt(v(d), K(t3));
    let f4 = o.lt(v(d), K(t4));
    let f5 = o.lt(v(d), K(t5));
    let s = o.sel(&f5, K(3200), K(2200));
    let s = o.sel(&f4, K(6800), v(&s));
    let s = o.sel(&f3, K(1100), v(&s));
    let s = o.sel(&f2, K(3500), v(&s));
    o.sel(&f1, K(5200), v(&s))
}

/// The speed profile: frame by frame the speed eased toward the least wanted over the stretch it could brake
/// in (24 places), never changing by more than the frame's acceleration, then smoothed (a Gaussian of 4 frames,
/// its weights printed in thousandths), the distances scaled to reach the hand-off at frame NCH; 26 frames on at
/// the hand-off's 2.2 m.
fn profile<O: Ops>(o: &mut O, tb: &Tb, path: &Path) -> V {
    let (mut d, mut vv) = (0i64, 0i64);
    let mut dd = vec![0i64];
    let turn20 = one(o.lin(&[(K(path.d_turn), None), (K(20_000), None)]));
    loop {
        let reached = one(o.lt(K(d), K(path.d_hand)));
        if reached == 0 { break; }
        let early = one(o.lt(K(d), K(turn20)));
        let dec = if early == 1 { 150 } else { 50 };
        // how far it could brake in, and on 10 m
        let v2 = one(o.lin(&[(K(vv), Some(K(vv)))]));
        let look = one(o.div(K(v2), K(2 * dec), 0));
        let look = one(o.lin(&[(K(look), None), (K(10_000), None)]));
        let ks: V = (0..24).collect();
        let at = o.lin(&[(K(look), Some(v(&ks)))]);
        let at = o.div(v(&at), K(23), 0);
        let at = o.lin(&[(v(&at), None), (K(d), None)]);
        let sp = wanted(o, path, &at);
        let neg = o.lin(&[(v(&sp), Some(K(-1)))]);
        let want = sp[o.nearest(&[&neg])[0]];
        let fast = one(o.lt(K(d), K(path.d_turn)));
        let acc = if fast == 1 { 420 } else { 140 };
        let n = dd.len() as i64;
        let ramp = one(o.lin(&[(K(n), Some(K(125))), (K(250), None)]));
        let ramp = one(o.min(K(ramp), K(1000)));
        let up = one(o.lin(&[(K(acc), Some(K(ramp)))]));
        let up = one(o.div(K(up), K(1000), 0));
        let lo = one(o.lin(&[(K(vv), None), (K(-dec), None)]));
        let hi = one(o.lin(&[(K(vv), None), (K(up), None)]));
        let t = one(o.max(K(want), K(lo)));
        let t = one(o.min(K(t), K(hi)));
        vv = one(o.max(K(t), K(50)));
        d = one(o.lin(&[(K(d), None), (K(vv), None)]));
        dd.push(d);
    }
    for _ in 0..26 { let x = one(o.lin(&[(K(*dd.last().unwrap()), None), (K(2200), None)])); dd.push(x); }
    // the speeds, smoothed: each frame's the weighted sum of the 25 round it (the ends held)
    let vs: V = { let a = &dd[..dd.len() - 1]; let b = &dd[1..]; o.lin(&[(v(&b.to_vec()), None), (v(&a.to_vec()), Some(K(-1)))]) };
    let wts = &tb.t("gauss4").data;
    let nv = vs.len() as i64;
    let terms: Vec<V> = (0..25).map(|k| (0..nv).map(|i| vs[(i + k - 12).clamp(0, nv - 1) as usize]).collect()).collect();
    let tl: Vec<(A, Option<A>)> = (0..25).map(|k| (v(&terms[k]), Some(K(wts[k])))).collect();
    let sm = o.lin(&tl);
    let sm = o.div(v(&sm), K(1000), 0);
    let mut d = vec![0];
    d.extend(cumsum(o, &sm));
    // scaled to reach the hand-off at NCH: the ratio in hundred-thousandths (by tenths of a metre)
    let dn = one(o.div(K(d[NCH as usize]), K(100), 0));
    let ratio = one(o.div(K(path.d_hand), K(dn), 1));
    let delta = one(o.lin(&[(K(ratio), None), (K(-100_000), None)]));
    let d100 = o.div(v(&d), K(100), 0);
    let t = o.lin(&[(v(&d100), Some(K(delta)))]);
    let t = o.div(v(&t), K(1000), 0);
    o.lin(&[(v(&d), None), (v(&t), None)])
}

/// The cab braking into the station: its speed (mm a frame) at each frame's middle, full until CRUISE, then 7000
/// (1 - s)^1.15 over the 96 frames to F_STOP (the printed power), and its place at each frame from Z0 (mm).
fn braking<O: Ops>(o: &mut O, tb: &Tb) -> (V, V) {
    let ks: V = (0..=F_STOP + 1).collect();
    // s at k + 0.5: (2k + 1 - 48) 1000 / 192, held to 0..1000
    let s = o.lin(&[(v(&ks), Some(K(2000))), (K(1000 - 48_000), None)]);
    let s = o.div(v(&s), K(192), 0);
    let s = o.clamp(v(&s), 0, 1000);
    let rest = o.lin(&[(v(&s), Some(K(-1))), (K(1000), None)]);
    let pw = o.tab(tb.t("pow115"), v(&rest));
    let sp = o.lin(&[(v(&pw), Some(K(7)))]);
    let z0 = one(o.lin(&[(K(Z0), Some(K(10)))]));
    let c = cumsum(o, &sp);
    // scaled to stop exactly where the station is laid out (ZS): the ratio in hundred-thousandths
    let want = one(o.lin(&[(K(ZS), Some(K(10))), (K(z0), Some(K(-1)))]));
    let got = one(o.div(K(c[(F_STOP - 1) as usize]), K(100), 0));
    let ratio = one(o.div(K(want), K(got), 1));
    let delta = one(o.lin(&[(K(ratio), None), (K(-100_000), None)]));
    let c100 = o.div(v(&c), K(100), 0);
    let t = o.lin(&[(v(&c100), Some(K(delta)))]);
    let t = o.div(v(&t), K(1000), 0);
    let c = o.lin(&[(v(&c), None), (v(&t), None)]);
    let sp = { let d = o.lin(&[(v(&sp), Some(K(delta)))]); let d = o.div(v(&d), K(100_000 / 100), 0); let d = o.div(v(&d), K(100), 0); o.lin(&[(v(&sp), None), (v(&d), None)]) };
    let mut za = vec![z0];
    za.extend(o.lin(&[(v(&c), None), (K(z0), None)]));
    (za, sp)
}

// ---- each frame's camera ----------------------------------------------------------------------------------------------

/// The camera's distance along the path at a time (tenths of a frame since F_CHASE), between the frames' (mm).
pub fn cam_dist<O: Ops>(o: &mut O, p: &Path, tenths: i64) -> i64 { one(cam_dists(o, p, &[tenths])) }

pub fn cam_dists<O: Ops>(o: &mut O, p: &Path, tenths: &[i64]) -> V {
    let last = (p.d.len() - 1) as i64;
    // the frame and its tenth on the machines (the time held to the table's: at its end, the last step whole)
    let tc = o.clamp(A::V(tenths), 0, 10 * last);
    let (q, r) = o.divmod(v(&tc), K(10));
    let i: Vec<i64> = q.iter().map(|&q| q.min(last - 1)).collect();
    let fr: V = q.iter().zip(&r).map(|(&q, &r)| if q == last { 10 } else { r }).collect();
    let a: V = i.iter().map(|&i| p.d[i as usize]).collect();
    let b: V = i.iter().map(|&i| p.d[i as usize + 1]).collect();
    let dlt = o.lin(&[(v(&b), None), (v(&a), Some(K(-1)))]);
    let m = o.lin(&[(v(&dlt), Some(v(&fr)))]);
    let m = o.div(v(&m), K(10), 0);
    o.lin(&[(v(&a), None), (v(&m), None)])
}

/// Points along the path at distances (mm).
pub fn path_at<O: Ops>(o: &mut O, p: &Path, d: &V) -> [V; 3] {
    // each place's segment on the machines: its quarter metre (the distance held to the path, over 250), that
    // quarter's first point past it from the start's table, and which of that point and the next the place has
    // reached; the host fetches by those indices
    let last = *p.l.last().unwrap();
    let dc = o.clamp(v(d), 0, last);
    let (b, _) = o.divmod(v(&dc), K(250));
    let i0 = o.tab(&p.l_tabs[0], v(&b));
    let la = o.tab(&p.l_tabs[1], v(&i0));
    let lb = o.tab(&p.l_tabs[2], v(&i0));
    let fa = o.lt(v(&dc), v(&la));
    let fb = o.lt(v(&dc), v(&lb));
    let seg = o.lin(&[(v(&i0), None), (K(2), None), (v(&fa), Some(K(-1))), (v(&fb), Some(K(-1)))]);
    let seg: Vec<usize> = seg.iter().map(|&i| i.clamp(1, p.l.len() as i64 - 1) as usize).collect();
    at_segs(o, &p.p, &p.l, d, &seg)
}

/// Angles (millionths of a turn) at times: rate a tenth of a frame (millionths) times the tenths from a start.
fn angles<O: Ops>(o: &mut O, tenths: &[i64], from: i64, rate: i64) -> V {
    let t: V = tenths.to_vec();
    let a = o.lin(&[(v(&t), Some(K(rate))), (K(-from * rate), None)]);
    o.turns(v(&a))
}

/// The airship's centre (film, unbent, mm) at times (tenths of a frame): its place over the bowl drifting 0.4 m/s
/// along X and bobbing 1.2 sin(0.6 t) in Z on the stadium's clock (t = 0 36 frames before the hand-off).
pub fn blimp_at<O: Ops>(o: &mut O, nw: &NightWorld, tenths: &[i64]) -> [V; 3] {
    let t: V = tenths.to_vec();
    // 0.4 m/s: 10 / 3 mm a tenth of a frame
    let dx = o.lin(&[(v(&t), Some(K(10))), (K(-10 * 10 * (F_END - 36)), None)]);
    let dx = o.div(v(&dx), K(3), 0);
    let a = angles(o, tenths, 10 * (F_END - 36), 796);
    let (s, _) = o.sincos(v(&a));
    let dz = o.lin(&[(v(&s), Some(K(1200)))]);
    let dz = o.div(v(&dz), K(999), 0);
    // its centre at rest, in the film's frame (worked out at the start), plus the drift and the bob
    let c = nw.blimp0;
    [o.lin(&[(K(c[0]), None), (v(&dx), None)]), o.lin(&[(K(c[1]), None), (v(&dz), None)]), vec![c[2]; t.len()]]
}

/// The drone (film, unbent, mm) at times (tenths of a frame): bobbing, swooping in from ahead to hover under the
/// canopy, then leading the camera along the path; aside while we read the airship's screen; away up over the
/// stadium while we orbit.
pub fn drone_at<O: Ops>(o: &mut O, nw: &NightWorld, tenths: &[i64]) -> [V; 3] {
    let tb = &nw.w.tb;
    let p = &nw.path;
    let n = tenths.len();
    let t: V = tenths.to_vec();
    let bx = { let a = angles(o, tenths, 0, 14_324); let (s, _) = o.sincos(v(&a)); let t = o.lin(&[(v(&s), Some(K(50)))]); o.div(v(&t), K(999), 0) };
    let by = { let a = angles(o, tenths, 0, 9549); let (s, _) = o.sincos(v(&a)); let t = o.lin(&[(v(&s), Some(K(80)))]); o.div(v(&t), K(999), 0) };
    // (the tenths since the chase, on the machines)
    let fc = { let d = o.sum(&[(A::V(tenths), 1)], -10 * F_CHASE); o.clamp(v(&d), 0, 999_999) };
    let dd = cam_dists(o, p, &fc);
    let ahead = o.lin(&[(v(&dd), None), (K(7500), None)]);
    let lead = path_at(o, p, &ahead);
    let lead = [o.lin(&[(v(&lead[0]), None), (K(600), None)]), o.lin(&[(v(&lead[1]), None), (K(-900), None)]), lead[2].clone()];
    // arriving: a = t - (F_BTN + 1), over 13 frames: from DRONE_FROM to DRONE_HOVER (eased), 1.5 m up at its middle
    let a = o.lin(&[(v(&t), Some(K(100))), (K(-100 * 10 * (F_BTN + 1)), None)]);
    let a = o.div(v(&a), K(13), 0);
    let a = o.clamp(v(&a), 0, 1000);
    let ea = ease_t(o, &a);
    let half = o.lin(&[(v(&a), Some(K(500)))]);
    let (sa, _) = o.sincos(v(&half));
    let (fr, hv) = (nw.places_mm[place::DRONE_FROM], nw.places_mm[place::DRONE_HOVER]);
    let arr: [V; 3] = std::array::from_fn(|c| lerpv(o, K(fr[c]), K(hv[c]), v(&ea)));
    let lift = { let t = o.lin(&[(v(&sa), Some(K(1500)))]); o.div(v(&t), K(999), 0) };
    let arr = [o.lin(&[(v(&arr[0]), None), (v(&bx), None)]), o.lin(&[(v(&arr[1]), None), (v(&lift), None), (v(&by), None)]), arr[2].clone()];
    // taking the lead over 18 frames from the chase's start
    let wv = o.lin(&[(v(&t), Some(K(100))), (K(-100 * 10 * F_CHASE), None)]);
    let wv = o.div(v(&wv), K(18), 0);
    let wv = o.clamp(v(&wv), 0, 1000);
    let w = ease_t(o, &wv);
    let mut pos: [V; 3] = std::array::from_fn(|c| lerpv(o, v(&arr[c]), v(&lead[c]), v(&w)));
    // aside while we read: (-14, -6, 0) m by e^-((d - read) / 60 m)^2
    let x = o.lin(&[(v(&dd), None), (K(p.d_read), Some(K(-1)))]);
    let x = o.div(v(&x), K(60), 0);
    let x2 = o.lin(&[(v(&x), Some(v(&x)))]);
    let x2 = o.div(v(&x2), K(1000), 0);
    let side = expn(o, tb, &x2);
    let sx = o.lin(&[(v(&side), Some(K(-14)))]);
    let sy = o.lin(&[(v(&side), Some(K(-6)))]);
    pos[0] = o.lin(&[(v(&pos[0]), None), (v(&sx), None)]);
    pos[1] = o.lin(&[(v(&pos[1]), None), (v(&sy), None)]);
    // away up over the stadium while we orbit: ease((d - read - 40) / 140) (1 - ease((d - orb1 + 30) / 110))
    let u1 = { let t = o.lin(&[(v(&dd), None), (K(p.d_read), Some(K(-1))), (K(-40_000), None)]); let t = o.div(v(&t), K(140), 0); let t = o.clamp(v(&t), 0, 1000); ease_t(o, &t) };
    let u2 = { let t = o.lin(&[(v(&dd), None), (K(p.d_orb1), Some(K(-1))), (K(30_000), None)]); let t = o.div(v(&t), K(110), 0); let t = o.clamp(v(&t), 0, 1000); ease_t(o, &t) };
    let u2 = o.lin(&[(v(&u2), Some(K(-1))), (K(1000), None)]);
    let u = o.lin(&[(v(&u1), Some(v(&u2)))]);
    let u = o.div(v(&u), K(1000), 0);
    let away = nw.places_mm[place::DRONE_AWAY];
    let _ = n;
    std::array::from_fn(|c| lerpv(o, v(&pos[c]), K(away[c]), v(&u)))
}

/// The cab's eye (film, unbent, mm) at a time (tenths of a frame, 0 the night's first frame): along the line braking
/// to a stop, swaying and bobbing with its speed, as the ride's camera does.
pub fn cab_eye<O: Ops>(o: &mut O, nw: &NightWorld, tenths: i64) -> [i64; 3] {
    let p = &nw.path;
    // (the tenths up to the stop, a frame and its tenth, on the machines)
    let tt = one(o.clamp(K(tenths), -999_999, 10 * F_STOP));
    // (cut toward zero, as Rust's / and %: the kernels reach a few tenths below 0)
    let (i, fr) = if tt < 0 {
        let p = kl(o, &[(tt, Some(-1))]);
        let (q, r) = o.divmod(K(p), K(10));
        (kl(o, &[(q[0], Some(-1))]) as usize, kl(o, &[(r[0], Some(-1))]))
    } else { let (q, r) = o.divmod(K(tt), K(10)); (q[0] as usize, r[0]) };
    let z = { let a = one(o.lin(&[(K(p.zv[i]), Some(K(fr)))])); let a = one(o.div(K(a), K(10), 0)); one(o.lin(&[(K(p.za[i]), None), (K(a), None)])) };
    // the speed's share (thousandths) at the time: (1 - s)^1.15, s = (t - 24) / 96 held to 0..1
    let s = one(o.lin(&[(K(tt), Some(K(100))), (K(-100 * 10 * CRUISE), None)]));
    let s = one(o.div(K(s), K(96), 0));
    let s = one(o.clamp(K(s), 0, 1000));
    let rest = one(o.lin(&[(K(s), Some(K(-1))), (K(1000), None)]));
    let vk = one(o.tab(nw.w.tb.t("pow115"), K(rest)));
    let k = kl(o, &[(tt, None), (10 * 172, None)]);
    let a1 = one(angles(o, &[k], 0, 11_141));
    let a2 = one(angles(o, &[k], 0, 36_606));
    let a3 = one(angles(o, &[k], 0, 30_239));
    let (s1, _) = o.sincos(K(a1));
    let (s2, _) = o.sincos(K(a2));
    let (s3, _) = o.sincos(K(a3));
    let sway = one(o.lin(&[(K(s1[0]), Some(K(35))), (K(s2[0]), Some(K(12)))]));
    let sway = one(o.lin(&[(K(sway), Some(K(vk)))]));
    let sway = one(o.div(K(sway), K(999), 0));
    let sway = one(o.div(K(sway), K(1000), 0));
    let bob = one(o.lin(&[(K(s3[0]), Some(K(12)))]));
    let bob = one(o.lin(&[(K(bob), Some(K(vk)))]));
    let bob = one(o.div(K(bob), K(999), 0));
    let bob = one(o.div(K(bob), K(1000), 0));
    [sway, one(o.lin(&[(K(bob), None), (K(10 * EYE_Y), None)])), z]
}

/// The camera's eye (film, unbent, mm) at times (tenths of a frame) from the platform on: on the platform still
/// but for a slow drift in (15 cm over the shot), then along the path.
pub fn chase_eyes<O: Ops>(o: &mut O, nw: &NightWorld, tenths: &[i64]) -> [V; 3] {
    let n = tenths.len();
    let mut out: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    let (plat, fly): (Vec<usize>, Vec<usize>) = (0..n).partition(|&i| tenths[i] <= 10 * F_CHASE);
    if !plat.is_empty() {
        let t: V = plat.iter().map(|&i| tenths[i]).collect();
        let u = o.lin(&[(v(&t), Some(K(100))), (K(-100 * 10 * F_GO), None)]);
        let u = o.div(v(&u), K(F_CHASE - F_GO), 0);
        let u = o.clamp(v(&u), 0, 1000);
        let dx = o.lin(&[(v(&u), Some(K(-150))), (K(150_000), None)]);
        let dx = o.div(v(&dx), K(1000), 0);
        let vw = nw.places_mm[place::VIEW];
        let x = o.lin(&[(v(&dx), None), (K(vw[0]), None)]);
        for (j, &i) in plat.iter().enumerate() { out[0][i] = x[j]; out[1][i] = vw[1]; out[2][i] = vw[2]; }
    }
    if !fly.is_empty() {
        let fc = { let t: V = fly.iter().map(|&i| tenths[i]).collect(); o.sum(&[(v(&t), 1)], -10 * F_CHASE) };
        let d = cam_dists(o, &nw.path, &fc);
        let pt = path_at(o, &nw.path, &d);
        for (j, &i) in fly.iter().enumerate() { for c in 0..3 { out[c][i] = pt[c][j]; } }
    }
    out
}

fn pick3(a: &[V; 3], at: &[usize]) -> [V; 3] { std::array::from_fn(|c| at.iter().map(|&i| a[c][i]).collect()) }

/// Where the camera wants to look at times (tenths of a frame): unit ways (thousandths):
/// on the platform up it and then at the drone swooping in; flying, ahead along the path (further the faster),
/// half at the drone early on, at the airship's screen as we read it, at the airship as we orbit it, down at the
/// bowl over the last 30 frames; past the hand-off at the bowl.
pub fn raw_aims<O: Ops>(o: &mut O, nw: &NightWorld, tenths: &[i64]) -> [V; 3] {
    let tb = &nw.w.tb;
    let p = &nw.path;
    let n = tenths.len();
    let mut out: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    let plat: Vec<usize> = (0..n).filter(|&i| tenths[i] <= 10 * F_CHASE).collect();
    let past: Vec<usize> = (0..n).filter(|&i| tenths[i] >= 10 * F_END).collect();
    let fly: Vec<usize> = (0..n).filter(|&i| tenths[i] > 10 * F_CHASE && tenths[i] < 10 * F_END).collect();
    let put = |out: &mut [V; 3], at: &[usize], d: &[V; 3]| { for (j, &i) in at.iter().enumerate() { for c in 0..3 { out[c][i] = d[c][j]; } } };
    let hl = nw.places_mm[place::HAND_LOOK];
    if !plat.is_empty() {
        let t: Vec<i64> = plat.iter().map(|&i| tenths[i]).collect();
        let e = chase_eyes(o, nw, &t);
        let dr = drone_at(o, nw, &t);
        let to: [V; 3] = std::array::from_fn(|c| o.lin(&[(v(&dr[c]), None), (v(&e[c]), Some(K(-1)))]));
        let to = nrm3(o, &to[0], &to[1], &to[2]);
        let a0 = nrm3(o, &vec![-1200], &vec![600], &vec![6000]);
        let wd = o.lin(&[(v(&t), Some(K(100))), (K(-100 * 10 * F_BTN), None)]);
        let wd = o.div(v(&wd), K(12), 0);
        let wd = o.clamp(v(&wd), 0, 1000);
        let wd = ease_t(o, &wd);
        let m: [V; 3] = std::array::from_fn(|c| lerpv(o, K(a0[c][0]), v(&to[c]), v(&wd)));
        let d = nrm3(o, &m[0], &m[1], &m[2]);
        put(&mut out, &plat, &d);
    }
    if !past.is_empty() {
        let t: Vec<i64> = past.iter().map(|&i| tenths[i]).collect();
        let e = chase_eyes(o, nw, &t);
        let to: [V; 3] = std::array::from_fn(|c| o.lin(&[(K(hl[c]), None), (v(&e[c]), Some(K(-1)))]));
        let d = nrm3(o, &to[0], &to[1], &to[2]);
        put(&mut out, &past, &d);
    }
    if fly.is_empty() { return out; }
    let t: Vec<i64> = fly.iter().map(|&i| tenths[i]).collect();
    let fc = o.sum(&[(v(&t), 1)], -10 * F_CHASE);
    let d = cam_dists(o, p, &fc);
    let (fa, fb) = (o.sum(&[(v(&fc), 1)], 5), o.sum(&[(v(&fc), 1)], -5));
    let da = cam_dists(o, p, &fa);
    let db = cam_dists(o, p, &fb);
    let vel = o.lin(&[(v(&da), None), (v(&db), Some(K(-1)))]);
    let eye = chase_eyes(o, nw, &t);
    // ahead: 18 m on and 5 frames' travel, 2 m down
    let ah = o.lin(&[(v(&d), None), (v(&vel), Some(K(5))), (K(18_000), None)]);
    let mut a = path_at(o, p, &ah);
    a[1] = o.lin(&[(v(&a[1]), None), (K(-2000), None)]);
    // half at the drone, from the push for 10 frames, until 80 m short of the screen's read (over 90 m)
    let w1 = o.lin(&[(v(&t), Some(K(100))), (K(-100 * 10 * F_GO), None)]);
    let w1 = o.div(v(&w1), K(10), 0);
    let w1 = o.clamp(v(&w1), 0, 1000);
    let w1 = ease_t(o, &w1);
    let w2 = { let x = o.lin(&[(v(&d), None), (K(p.d_read), Some(K(-1))), (K(80_000), None)]); let x = o.div(v(&x), K(90), 0); let x = o.clamp(v(&x), 0, 1000); ease_t(o, &x) };
    let w2 = o.lin(&[(v(&w2), Some(K(-1))), (K(1000), None)]);
    let wdr = o.lin(&[(v(&w1), Some(v(&w2)))]);
    let wdr = o.div(v(&wdr), K(2000), 0);
    let dr = drone_at(o, nw, &t);
    a = std::array::from_fn(|c| lerpv(o, v(&a[c]), v(&dr[c]), v(&wdr)));
    // the airship's screen as we read it: e^-((d - read) / (80 m short of it, 45 m past))^2
    let x = o.lin(&[(v(&d), None), (K(p.d_read), Some(K(-1)))]);
    let short = o.lt(v(&x), K(0));
    let x80 = o.div(v(&x), K(80), 0);
    let x45 = o.div(v(&x), K(45), 0);
    let xs = o.sel(&short, v(&x80), v(&x45));
    let x2 = o.lin(&[(v(&xs), Some(v(&xs)))]);
    let x2 = o.div(v(&x2), K(1000), 0);
    let ws = expn(o, tb, &x2);
    let bl = blimp_at(o, nw, &t);
    let so = nw.screen_off_mm;
    let scr_at: [V; 3] = std::array::from_fn(|c| o.lin(&[(v(&bl[c]), None), (K(so[c]), None)]));
    a = std::array::from_fn(|c| lerpv(o, v(&a[c]), v(&scr_at[c]), v(&ws)));
    // the airship as we orbit it (6 m under its middle): min(ease((d - read - 20) / 90), 1 - ease((d - orb1 + 120) / 170))
    let o1 = { let x = o.lin(&[(v(&d), None), (K(p.d_read), Some(K(-1))), (K(-20_000), None)]); let x = o.div(v(&x), K(90), 0); let x = o.clamp(v(&x), 0, 1000); ease_t(o, &x) };
    let o2 = { let x = o.lin(&[(v(&d), None), (K(p.d_orb1), Some(K(-1))), (K(120_000), None)]); let x = o.div(v(&x), K(170), 0); let x = o.clamp(v(&x), 0, 1000); ease_t(o, &x) };
    let o2 = o.lin(&[(v(&o2), Some(K(-1))), (K(1000), None)]);
    let wo = o.min(v(&o1), v(&o2));
    let under = [bl[0].clone(), o.lin(&[(v(&bl[1]), None), (K(-6000), None)]), bl[2].clone()];
    a = std::array::from_fn(|c| lerpv(o, v(&a[c]), v(&under[c]), v(&wo)));
    let to: [V; 3] = std::array::from_fn(|c| o.lin(&[(v(&a[c]), None), (v(&eye[c]), Some(K(-1)))]));
    let dirn = nrm3(o, &to[0], &to[1], &to[2]);
    // the dive onto the bowl: over the last 30 frames, and (0.8 of it) from 30 m before the orbit's end over 50 m
    let e1 = { let x = o.lin(&[(v(&fc), Some(K(100))), (K(-100 * 10 * (NCH - 30)), None)]); let x = o.div(v(&x), K(30), 0); let x = o.clamp(v(&x), 0, 1000); ease_t(o, &x) };
    let e2 = { let x = o.lin(&[(v(&d), None), (K(p.d_orb1), Some(K(-1))), (K(30_000), None)]); let x = o.div(v(&x), K(50), 0); let x = o.clamp(v(&x), 0, 1000); let e = ease_t(o, &x); let e = o.lin(&[(v(&e), Some(K(8)))]); o.div(v(&e), K(10), 0) };
    let w = o.max(v(&e1), v(&e2));
    let h1: [V; 3] = std::array::from_fn(|c| o.lin(&[(K(hl[c]), None), (v(&eye[c]), Some(K(-1)))]));
    let h1 = nrm3(o, &h1[0], &h1[1], &h1[2]);
    let hd: [V; 3] = std::array::from_fn(|c| lerpv(o, v(&h1[c]), K(nw.hand_dir[c]), v(&e1)));
    let m: [V; 3] = std::array::from_fn(|c| lerpv(o, v(&dirn[c]), v(&hd[c]), v(&w)));
    let dd = nrm3(o, &m[0], &m[1], &m[2]);
    put(&mut out, &fly, &dd);
    out
}

/// The camera's bank at times (tenths of a frame; mrad): from the path's turn over 12 m and the speed, a coordinated
/// turn's halved, at most 0.35; level in the cab, on the platform and at the hand-off (easing level over the last 30
/// frames).
pub fn raw_rolls<O: Ops>(o: &mut O, nw: &NightWorld, tenths: &[i64]) -> V {
    let tb = &nw.w.tb;
    let p = &nw.path;
    let n = tenths.len();
    let mut out = vec![0; n];
    let fly: Vec<usize> = (0..n).filter(|&i| tenths[i] > 10 * (F_CHASE + 1) && tenths[i] < 10 * F_END).collect();
    if fly.is_empty() { return out; }
    let fc = { let t: V = fly.iter().map(|&i| tenths[i]).collect(); o.sum(&[(v(&t), 1)], -10 * F_CHASE) };
    let d = cam_dists(o, p, &fc);
    let (fa, fb) = (o.sum(&[(v(&fc), 1)], 5), o.sum(&[(v(&fc), 1)], -5));
    let da = cam_dists(o, p, &fa);
    let db = cam_dists(o, p, &fb);
    let vel = o.lin(&[(v(&da), None), (v(&db), Some(K(-1)))]);
    let dm = o.lin(&[(v(&d), None), (K(-6000), None)]);
    let dp = o.lin(&[(v(&d), None), (K(6000), None)]);
    let a = path_at(o, p, &dm);
    let b = path_at(o, p, &dp);
    let c = path_at(o, p, &d);
    // the turn from (c - a) to (b - c), level: atan2 of the cross and the dot (each over 1000), by the printed atan
    let t1x = o.lin(&[(v(&c[0]), None), (v(&a[0]), Some(K(-1)))]);
    let t1z = o.lin(&[(v(&c[2]), None), (v(&a[2]), Some(K(-1)))]);
    let t2x = o.lin(&[(v(&b[0]), None), (v(&c[0]), Some(K(-1)))]);
    let t2z = o.lin(&[(v(&b[2]), None), (v(&c[2]), Some(K(-1)))]);
    let nt1x = o.lin(&[(v(&t1x), Some(K(-1)))]);
    let cr = o.lin(&[(v(&t2x), Some(v(&t1z))), (v(&nt1x), Some(v(&t2z)))]);
    let dt = o.lin(&[(v(&t1x), Some(v(&t2x))), (v(&t1z), Some(v(&t2z)))]);
    let cr = o.div(v(&cr), K(1000), 0);
    let dt = o.div(v(&dt), K(1000), 0);
    let dt = o.max(v(&dt), K(1));
    let crc = o.clamp(v(&cr), -999_000, 999_999);
    let rq = o.lin(&[(v(&crc), Some(K(10)))]);
    let rq = o.div(v(&rq), v(&dt), 1);
    let rq = o.clamp(v(&rq), -10_000, 10_000);
    let dh = o.tabo(tb.t("atan4"), 10_000, v(&rq));
    let dh = o.clamp(v(&dh), -2000, 2000);
    // x = vs^2 kappa / g (thousandths) = v_cm^2 dh / 40,875 (v in cm a frame, dh in 1e-4 rad)
    let vc = o.div(v(&vel), K(10), 0);
    let v2 = o.lin(&[(v(&vc), Some(v(&vc)))]);
    let x = o.lin(&[(v(&v2), Some(v(&dh)))]);
    let x = o.div(v(&x), K(40_875), 0);
    let x = o.clamp(v(&x), -5000, 5000);
    let at = o.tabo(tb.t("atan_x"), 5000, v(&x));
    let roll = o.lin(&[(v(&at), Some(K(-1)))]);
    let roll = o.div(v(&roll), K(2), 0);
    let roll = o.clamp(v(&roll), -350, 350);
    let e1 = { let x = o.lin(&[(v(&fc), Some(K(100))), (K(-100 * 10 * (NCH - 30)), None)]); let x = o.div(v(&x), K(30), 0); let x = o.clamp(v(&x), 0, 1000); ease_t(o, &x) };
    let k = o.lin(&[(v(&e1), Some(K(-1))), (K(1000), None)]);
    let r = o.lin(&[(v(&roll), Some(v(&k)))]);
    let r = o.div(v(&r), K(1000), 0);
    for (j, &i) in fly.iter().enumerate() { out[i] = r[j]; }
    out
}

/// The camera's aim and bank at a time (tenths of a frame): the wanted ways averaged over a second (a Gaussian of 4
/// frames, 25 of them), the banks over 2.5 times as long (49), narrowing to none over the last 20 frames so it lands
/// on the hand-off's aim exactly.
pub fn aim<O: Ops>(o: &mut O, nw: &NightWorld, tenths: i64) -> ([i64; 3], i64) {
    let tb = &nw.w.tb;
    // sigma (thousandths of a frame): 4 times (F_END - t) / 20 held to 0..1
    let sg = one(o.lin(&[(K(tenths), Some(K(-1))), (K(10 * F_END), None)]));
    let sg = one(o.lin(&[(K(sg), Some(K(20)))]));
    let sg = one(o.clamp(K(sg), 0, 4000));
    if sg < 300 {
        let d = raw_aims(o, nw, &[tenths]);
        let r = raw_rolls(o, nw, &[tenths]);
        return ([d[0][0], d[1][0], d[2][0]], r[0]);
    }
    let ks: Vec<i64> = (-12..=12).collect();
    let t = { let off: V = ks.iter().map(|&k| 10 * k).collect(); o.sum(&[(K(tenths), 1), (v(&off), 1)], 0) };
    let dirs = raw_aims(o, nw, &t);
    let kk: V = ks.iter().map(|&k| 1_000_000 * k).collect();
    let r = o.div(v(&kk), K(sg), 0);
    let r = o.clamp(v(&r), -20_000, 20_000);
    let a = o.lin(&[(v(&r), Some(v(&r)))]);
    let a = o.div(v(&a), K(2000), 0);
    let w = expn(o, tb, &a);
    let s: [V; 3] = std::array::from_fn(|c| { let terms: Vec<(A, Option<A>)> = (0..25).map(|k| (K(dirs[c][k]), Some(K(w[k])))).collect(); o.lin(&terms) });
    let d = nrm3(o, &s[0], &s[1], &s[2]);
    let k2: Vec<i64> = (-24..=24).collect();
    let t2 = { let off: V = k2.iter().map(|&k| 10 * k).collect(); o.sum(&[(K(tenths), 1), (v(&off), 1)], 0) };
    let rolls = raw_rolls(o, nw, &t2);
    let kk: V = k2.iter().map(|&k| 2_000_000 * k).collect();
    let sg5 = one(o.lin(&[(K(sg), Some(K(5)))]));
    let r = o.div(v(&kk), K(sg5), 0);
    let r = o.clamp(v(&r), -20_000, 20_000);
    let a = o.lin(&[(v(&r), Some(v(&r)))]);
    let a = o.div(v(&a), K(2000), 0);
    let w2 = expn(o, tb, &a);
    let num: Vec<(A, Option<A>)> = (0..49).map(|k| (K(rolls[k]), Some(K(w2[k])))).collect();
    let num = one(o.lin(&num));
    let den = *cumsum(o, &w2).last().unwrap();
    let roll = if num == 0 { 0 } else { one(o.div(K(num), K(den), 0)) };
    ([d[0][0], d[1][0], d[2][0]], roll)
}

/// The camera at a time (tenths of a frame): the cab's (before the dissolve to the platform, or held for its first
/// half), else the flight's; its turn from the aim and the bank; its lens widening from the ride's (f 950) to the
/// stadium's (1000) over the chase.
pub fn night_cam<O: Ops>(o: &mut O, nw: &NightWorld, tenths: i64, cab: bool) -> Cam {
    // past the skip (the design's time, tenths): over JOIN frames from the screen's read (the design's camera at the
    // click, VOL_OFF) onto the dive's: the eye along a cubic (Hermite) from the read's place and speed to the dive's,
    // the way, the bank and the lens from the read's to the dive's as they go, eased
    let (tc, tj) = (nw.path.t_cut, nw.path.t_join);
    if !cab && tenths > tj[0] && tenths < tj[1] {
        let a = cam_parts(o, nw, tc[0], false);
        let a1 = cam_parts(o, nw, tc[1], false);
        let b = cam_parts(o, nw, tj[1], false);
        let b1 = cam_parts(o, nw, tj[2], false);
        let bt = cam_parts(o, nw, tenths, false);
        let u = one(o.lin(&[(K(tenths), Some(K(100))), (K(nw.path.k_join), None)]));
        let s_ = one(o.div(K(u), K(JOIN), 0));
        let s2 = one({ let t = o.lin(&[(K(s_), Some(K(s_)))]); o.div(v(&t), K(1000), 0) });
        let s3 = one({ let t = o.lin(&[(K(s2), Some(K(s_)))]); o.div(v(&t), K(1000), 0) });
        let h10 = one(o.lin(&[(K(s3), None), (K(s2), Some(K(-2))), (K(s_), None)]));
        let h01 = one(o.lin(&[(K(s3), Some(K(-2))), (K(s2), Some(K(3)))]));
        let h11 = one(o.lin(&[(K(s3), None), (K(s2), Some(K(-1)))]));
        let eye: [i64; 3] = std::array::from_fn(|c| {
            let va = one(o.lin(&[(K(a.0[c]), Some(K(JOIN))), (K(a1.0[c]), Some(K(-JOIN)))]));
            let vb = one(o.lin(&[(K(b.0[c]), Some(K(JOIN))), (K(b1.0[c]), Some(K(-JOIN)))]));
            let dp = one(o.lin(&[(K(b.0[c]), None), (K(a.0[c]), Some(K(-1)))]));
            let t = o.lin(&[(K(h10), Some(K(va))), (K(h01), Some(K(dp))), (K(h11), Some(K(vb)))]);
            let t = o.div(v(&t), K(1000), 0);
            one(o.lin(&[(K(a.0[c]), None), (v(&t), None)]))
        });
        let w = one(ease_t(o, &vec![s_]));
        let m: [V; 3] = std::array::from_fn(|c| lerpv(o, K(a.1[c]), K(bt.1[c]), K(w)));
        let d = nrm3(o, &m[0], &m[1], &m[2]);
        let roll = one(lerpv(o, K(a.2), K(bt.2), K(w)));
        let f2 = one(lerpv(o, K(a.3), K(bt.3), K(w)));
        return basis_cam(o, nw, eye, [d[0][0], d[1][0], d[2][0]], roll, f2);
    }
    let (eye, dir, roll, f2) = cam_parts(o, nw, tenths, cab);
    basis_cam(o, nw, eye, dir, roll, f2)
}

/// The camera's eye (film, unbent, mm), its way (thousandths), bank (mrad) and twice its focal length at a time of
/// the design (tenths of a frame).
pub fn cam_parts<O: Ops>(o: &mut O, nw: &NightWorld, tenths: i64, cab: bool) -> ([i64; 3], [i64; 3], i64, i64) {
    if cab || tenths < 10 * F_GO {
        let e = cab_eye(o, nw, tenths);
        (e, nw.cab_dir, 0, 1900)
    } else {
        let e = chase_eyes(o, nw, &[tenths]);
        let (d, r) = aim(o, nw, tenths);
        // (the tenths since the chase, a hundred times, over its frames: held to 0 before it)
        let u = one(o.lin(&[(K(tenths), Some(K(100))), (K(-1000 * F_CHASE), None)]));
        let u = one(o.div(K(u), K(NCH), 0));
        let u = one(o.clamp(K(u), 0, 1000));
        let ez = one(ease_t(o, &vec![u]));
        let f2 = one(o.lin(&[(K(ez), Some(K(100)))]));
        let f2 = one(o.div(K(f2), K(1000), 0));
        let f2 = one(o.lin(&[(K(f2), None), (K(1900), None)]));
        ([e[0][0], e[1][0], e[2][0]], d, r, f2)
    }
}

/// A camera from its eye (film, unbent, mm), the way it looks (thousandths), its bank (mrad) and twice its focal
/// length (pixels): across level, up square to both; banked.
pub fn basis_cam<O: Ops>(o: &mut O, _nw: &NightWorld, eye: [i64; 3], f: [i64; 3], roll: i64, f2: i64) -> Cam {
    let nf0 = kl(o, &[(f[0], Some(-1))]);
    let r = nrm3(o, &vec![f[2]], &vec![0], &vec![nf0]);
    let r = [r[0][0], 0, r[2][0]];
    // (the across axis's parts' negatives, on the machines)
    let nr = o.lin(&[(v(&vec![r[0], r[2]]), Some(K(-1)))]);
    let nr = [nr[0], 0, nr[1]];
    let u = [one({ let t = o.lin(&[(K(f[1]), Some(K(r[2])))]); o.div(v(&t), K(1000), 0) }),
             one({ let t = o.lin(&[(K(f[2]), Some(K(r[0]))), (K(f[0]), Some(K(nr[2])))]); o.div(v(&t), K(1000), 0) }),
             one({ let t = o.lin(&[(K(f[1]), Some(K(nr[0])))]); o.div(v(&t), K(1000), 0) })];
    let (rr, uu) = if roll == 0 { (r, u) } else {
        let ang = one({ let t = o.lin(&[(K(roll), Some(K(159_155)))]); o.div(v(&t), K(1000), 0) });
        let (s, c) = o.sincos(K(ang));
        let (s, c) = (s[0], c[0]);
        let rr: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K(c), Some(K(r[i]))), (K(s), Some(K(u[i])))]); o.div(v(&t), K(999), 0) }));
        let uu: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K(s), Some(K(nr[i]))), (K(c), Some(K(u[i])))]); o.div(v(&t), K(999), 0) }));
        (rr, uu)
    };
    let sc = |o: &mut O, a: [i64; 3]| -> [i64; 3] { if f2 == 1900 { a } else { std::array::from_fn(|i| one({ let t = o.lin(&[(K(a[i]), Some(K(f2)))]); o.div(v(&t), K(1900), 0) })) } };
    let bt = [sc(o, rr), sc(o, uu), f];
    let eye = [one(o.lin(&[(K(eye[0]), None), (K(10 * BX), None)])), eye[1], eye[2]];
    let ecm: [i64; 3] = std::array::from_fn(|i| one(o.div(K(eye[i]), K(10), 0)));
    Cam { eye, eye_cm: ecm, b: [rr, uu, f], bt, f2, z_cm: ecm[2] }
}
