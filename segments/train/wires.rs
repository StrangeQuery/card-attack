// The wires (the prototype's wires()): the overhead line's contact wire, messenger and droppers over both lines, the
// farm road's power lines, the suburb's: segments in the world, drawn after the blur as anti-aliased lines behind
// what's in front of them. Included after blur.rs.

pub struct Wires { pub seg: Vec<[i64; 9]> }

/// The segments a frame draws, by their kind: each segment's nearer and further place along (z0, z1) against the
/// camera's place along cz and the kind's reach from it, every bound and comparison on the machines. The contact
/// wire and the messenger: cz + 1 m < z0 and z1 < cz + 700 m; a dropper: cz + 1 m < z0 < cz + 120 m; the farm road's
/// power lines (from 40 m short of the tunnel's end on): cz <= z1 and z0 <= cz + 600 m; the suburb's (short of the
/// tunnel): cz + 1 m < z1 and z0 < cz + 450 m.
pub fn wire_pick<O: Ops>(o: &mut O, w: &Wires, cz: i64) -> Vec<usize> {
    let n = w.seg.len();
    if n == 0 { return vec![]; }
    let (za, zb): (V, V) = w.seg.iter().map(|s| (s[2], s[5])).unzip();
    let z0 = o.min(v(&za), v(&zb));
    let z1 = o.max(v(&za), v(&zb));
    let of = |x: &V, ks: &[usize]| -> V { ks.iter().map(|&k| x[k]).collect() };
    let mut keep = vec![false; n];
    for kd in 0..4 {
        let ks: Vec<usize> = (0..n).filter(|&i| match w.seg[i][8] { 0 => 0, 1 => 1, 2 => 2, _ => 3 } == kd).collect();
        if ks.is_empty() || (kd == 2 && cz <= TUN1 - 4000) || (kd == 3 && cz >= TUN0) { continue; }
        let at = |o: &mut O, d: i64| -> A<'static> { K(kl(o, &[(cz, None), (d, None)])) };
        let (a, b) = (of(&z0, &ks), of(&z1, &ks));
        // two flags each, and the value each must have (the farm road's: neither z1 < cz nor cz + 600 m < z0)
        let (f, g, want) = match kd {
            0 => { let (lo, hi) = (at(o, 100), at(o, 70_000)); (o.lt(lo, v(&a)), o.lt(v(&b), hi), 1) }
            1 => { let (lo, hi) = (at(o, 100), at(o, 12_000)); (o.lt(lo, v(&a)), o.lt(v(&a), hi), 1) }
            2 => { let hi = at(o, 60_000); (o.lt(v(&b), K(cz)), o.lt(hi, v(&a)), 0) }
            _ => { let (lo, hi) = (at(o, 100), at(o, 45_000)); (o.lt(lo, v(&b)), o.lt(v(&a), hi), 1) }
        };
        for (j, &i) in ks.iter().enumerate() { keep[i] = f[j] == want && g[j] == want; }
    }
    (0..n).filter(|&i| keep[i]).collect()
}

/// Draw the wires into the frame: each segment's ends on the screen; along its longer axis, at each pixel's middle,
/// the line's place across (exact), its width (the radius at the depth, at least 0.35 pixel) and depth (carried);
/// the pixels within reach, each covered by the share of the line over it (by the distance across, times the
/// segment's slope, and the width: a table), where the wire is in front.
pub fn draw_wires<O: Ops>(o: &mut O, w: &World, cam: &Cam, env: &Env, e: i64, g: &GBuf, col: &mut [V; 3], hot: &mut [V; 3]) {
    let tb = &w.tb;
    let pick = wire_pick(o, &w.wires, cam.z_cm);
    if pick.is_empty() { return; }
    // the ends in the camera's space (millimetres) and on the screen
    let mut ends = vec![];
    for &i in &pick { let s = w.wires.seg[i]; ends.push([s[0], s[1], s[2]]); ends.push([s[3], s[4], s[5]]); }
    let ne: [i64; 3] = std::array::from_fn(|i| kl(o, &[(cam.eye[i], Some(-1))]));
    let dvec: Vec<V> = (0..3).map(|i| { let p: V = ends.iter().map(|c| c[i]).collect(); o.lin(&[(v(&p), Some(K(10))), (K(ne[i]), None)]) }).collect();
    let mut cc: Vec<[i64; 3]> = vec![[0; 3]; ends.len()];
    for r in 0..3 {
        let s = o.lin(&[(v(&dvec[0]), Some(K(cam.bt[r][0]))), (v(&dvec[1]), Some(K(cam.bt[r][1]))), (v(&dvec[2]), Some(K(cam.bt[r][2])))]);
        let s = o.div(v(&s), K(1000), 0);
        for (k, val) in s.iter().enumerate() { cc[k][r] = *val; }
    }
    let zc: V = cc.iter().map(|c| c[2]).collect();
    let near = o.lt(v(&zc), K(NEAR_MM));
    // segments with both ends in front, within the guard band (else skipped, as the prototype skips those behind)
    let gx: V = cc.iter().map(|c| c[0]).collect();
    let gy: V = cc.iter().map(|c| c[1]).collect();
    let px_ = o.lin(&[(v(&zc), Some(K(96))), (v(&gx), Some(K(-95)))]);
    let pl_ = o.lin(&[(v(&zc), Some(K(96))), (v(&gx), Some(K(95)))]);
    let pt_ = o.lin(&[(v(&zc), Some(K(48))), (v(&gy), Some(K(-95)))]);
    let pb_ = o.lin(&[(v(&zc), Some(K(48))), (v(&gy), Some(K(95)))]);
    let o1 = o.lt(v(&px_), K(0)); let o2 = o.lt(v(&pl_), K(0)); let o3 = o.lt(v(&pt_), K(0)); let o4 = o.lt(v(&pb_), K(0));
    let keep: Vec<usize> = (0..pick.len()).filter(|&j| (0..2).all(|e2| { let k = 2 * j + e2; near[k] == 0 && o1[k] == 0 && o2[k] == 0 && o3[k] == 0 && o4[k] == 0 })).collect();
    if keep.is_empty() { return; }
    let used: Vec<[i64; 3]> = keep.iter().flat_map(|&j| [cc[2 * j], cc[2 * j + 1]]).collect();
    let pr = project(o, &w.tb, &used, false, 1);
    let nk = keep.len();
    let a: Vec<(i64, i64, i64)> = (0..nk).map(|j| pr[2 * j]).collect();
    let b: Vec<(i64, i64, i64)> = (0..nk).map(|j| pr[2 * j + 1]).collect();
    // widths at the ends: radius (mm) times the depth K / z over 3155, hundredths of a pixel, at least 35
    let rad: V = keep.iter().map(|&j| w.wires.seg[pick[j]][6]).collect();
    let wid = |o: &mut O, z: &V| -> V { let t = o.lin(&[(v(&rad), Some(v(z)))]); let t = o.div(v(&t), K(3155), 0); let t = o.max(v(&t), K(35)); o.min(v(&t), K(999)) };
    let za: V = a.iter().map(|p| p.2).collect();
    let zb: V = b.iter().map(|p| p.2).collect();
    let wa = wid(o, &za);
    let wb = wid(o, &zb);
    // the longer axis, the ends in order along it
    // (the ends' differences on the machines)
    let (ax, ay, bx, by): (V, V, V, V) = ((0..nk).map(|j| a[j].0).collect(), (0..nk).map(|j| a[j].1).collect(), (0..nk).map(|j| b[j].0).collect(), (0..nk).map(|j| b[j].1).collect());
    let dxv = o.lin(&[(v(&bx), None), (v(&ax), Some(K(-1)))]);
    let dyv = o.lin(&[(v(&by), None), (v(&ay), Some(K(-1)))]);
    let adx = { let m = o.lin(&[(v(&dxv), Some(K(-1)))]); o.max(v(&dxv), v(&m)) };
    let ady = { let m = o.lin(&[(v(&dyv), Some(K(-1)))]); o.max(v(&dyv), v(&m)) };
    let ymaj = o.lt(v(&adx), v(&ady));
    let au = o.sel(&ymaj, v(&a.iter().map(|p| p.1).collect::<V>()), v(&a.iter().map(|p| p.0).collect::<V>()));
    let am = o.sel(&ymaj, v(&a.iter().map(|p| p.0).collect::<V>()), v(&a.iter().map(|p| p.1).collect::<V>()));
    let bu = o.sel(&ymaj, v(&b.iter().map(|p| p.1).collect::<V>()), v(&b.iter().map(|p| p.0).collect::<V>()));
    let bm = o.sel(&ymaj, v(&b.iter().map(|p| p.0).collect::<V>()), v(&b.iter().map(|p| p.1).collect::<V>()));
    let flip = o.lt(v(&bu), v(&au));
    let (u0, m0, u1, m1, w0, w1, z0, z1) = (o.sel(&flip, v(&bu), v(&au)), o.sel(&flip, v(&bm), v(&am)), o.sel(&flip, v(&au), v(&bu)), o.sel(&flip, v(&am), v(&bm)),
        o.sel(&flip, v(&wb), v(&wa)), o.sel(&flip, v(&wa), v(&wb)), o.sel(&flip, v(&zb), v(&za)), o.sel(&flip, v(&za), v(&zb)));
    // the slope's cosine (thousandths): the longer part over the length
    let big = o.max(v(&adx), v(&ady));
    let small = o.min(v(&adx), v(&ady));
    let b1 = o.div(v(&big), K(100), 0);
    let s1 = o.div(v(&small), K(100), 0);
    let l2 = o.lin(&[(v(&b1), Some(v(&b1))), (v(&s1), Some(v(&s1)))]);
    let len = o.sqrt(v(&l2));
    let len = o.max(v(&len), K(1));
    let cosf = { let t = o.lin(&[(v(&b1), Some(K(1000)))]); let t = o.div(v(&t), v(&len), 0); o.clamp(v(&t), 1, 999) };
    // the places along: pixel middles from fl(u0 + 49, 100) to short of fl(u1 + 49, 100)
    let c0 = { let t = o.sum(&[(v(&u0), 1)], 49); o.div(v(&t), K(100), 0) };
    let c1 = { let t = o.sum(&[(v(&u1), 1)], 49); o.div(v(&t), K(100), 0) };
    let ymv = ymaj.clone();
    let mut places = vec![]; // (segment, place)
    for j in 0..nk {
        let lim = if ymv[j] == 1 { H() / 2 } else { W() / 2 };
        for c in c0[j].max(-lim)..c1[j].min(lim) { places.push((j, c)); }
    }
    if places.is_empty() { return; }
    let g_ = |v: &V| -> V { places.iter().map(|&(j, _)| v[j]).collect() };
    let pc: V = places.iter().map(|p| p.1).collect();
    let (pu0, pm0, pu1, pm1) = (g_(&u0), g_(&m0), g_(&u1), g_(&m1));
    let off = o.lin(&[(v(&pc), Some(K(100))), (K(50), None), (v(&pu0), Some(K(-1)))]);
    let dmv = o.lin(&[(v(&pm1), None), (v(&pm0), Some(K(-1)))]);
    let duv = o.lin(&[(v(&pu1), None), (v(&pu0), Some(K(-1)))]);
    let duv = o.max(v(&duv), K(1));
    let t = o.lin(&[(v(&dmv), Some(v(&off)))]);
    let t = o.div(v(&t), v(&duv), 0);
    let mm = o.lin(&[(v(&pm0), None), (v(&t), None)]);
    // width and depth carried between the ends
    let fr = { let t = o.lin(&[(v(&off), Some(K(100)))]); let t = o.div(v(&t), v(&duv), 0); o.clamp(v(&t), 0, 100) };
    let ww = o.mix(v(&g_(&w0)), v(&g_(&w1)), v(&fr));
    let zz = { let d = o.lin(&[(v(&g_(&z1)), None), (v(&g_(&z0)), Some(K(-1)))]); let t = o.lin(&[(v(&d), Some(v(&fr)))]); let t = o.div(v(&t), K(100), 0); o.lin(&[(v(&g_(&z0)), None), (v(&t), None)]) };
    // the pixels within reach across: |offset| under 1.42 (w + 50)
    let reach = { let t = o.sum(&[(v(&ww), 1)], 50); let t = o.lin(&[(v(&t), Some(K(142)))]); o.div(v(&t), K(100), 0) };
    let r0 = { let t = o.lin(&[(v(&mm), None), (v(&reach), Some(K(-1))), (K(-50), None)]); let t = o.lin(&[(v(&t), None), (K(99), None)]); o.div(v(&t), K(100), 0) };
    let r1 = { let t = o.lin(&[(v(&mm), None), (v(&reach), None), (K(-50), None)]); o.div(v(&t), K(100), 0) };
    let mut cand = vec![]; // (place, row across)
    for (i, &(j, c)) in places.iter().enumerate() {
        let lim = if ymv[j] == 1 { W() / 2 } else { H() / 2 };
        for r in r0[i].max(-lim)..=r1[i].min(lim - 1) { cand.push((i, r, c, j)); }
    }
    if cand.is_empty() { return; }
    let cg = |v: &V| -> V { cand.iter().map(|&(i, _, _, _)| v[i]).collect() };
    let rr: V = cand.iter().map(|c| c.1).collect();
    let dm = { let t = o.lin(&[(v(&rr), Some(K(100))), (K(50), None), (v(&cg(&mm)), Some(K(-1)))]); let m = o.lin(&[(v(&t), Some(K(-1)))]); o.max(v(&t), v(&m)) };
    let dm = o.min(v(&dm), K(999));
    let cosj: V = cand.iter().map(|&(_, _, _, j)| cosf[j]).collect();
    let d = o.mul(v(&dm), v(&cosj), 3);
    let cov = o.tab2(tb.t("wire_cov"), v(&cg(&ww)), v(&d));
    // in front of what the pixel shows
    let pix: Vec<usize> = cand.iter().map(|&(_, r, c, j)| { let (x, y) = if ymv[j] == 1 { (r, c) } else { (c, r) }; ((y + H() / 2) * W() + x + W() / 2) as usize }).collect();
    let gz: V = pix.iter().map(|&p| g.iz[p]).collect();
    let wz = cg(&zz);
    let a1 = o.lin(&[(v(&wz), Some(K(200)))]);
    let a2 = o.lin(&[(v(&gz), Some(K(199)))]);
    let behind = o.lt(v(&a1), v(&a2));
    let cov = o.sel(&behind, K(0), v(&cov));
    // the wire's colour: 0.02, and at dusk the sun's colour times its shine
    let shine: V = cand.iter().map(|&(_, _, _, j)| w.wires.seg[pick[keep[j]]][7]).collect();
    let mut wc: [V; 3] = [vec![], vec![], vec![]];
    for c in 0..3 {
        let base = one(mulk(o, K(5), K(e)));
        let s = if env.day { vec![0; cand.len()] } else { let k = env_k(o, env, env.sun_col[c], e); o.mul(v(&shine), K(k), 3) };
        wc[c] = o.sum(&[(v(&s), 1)], base);
    }
    // in rounds, so each round covers a pixel at most once (in the segments' order)
    let mut round = vec![0usize; cand.len()];
    let mut seen: std::collections::HashMap<usize, usize> = Default::default();
    for (k, &p) in pix.iter().enumerate() { let e2 = seen.entry(p).or_insert(0); round[k] = *e2; *e2 += 1; }
    let rounds = round.iter().copied().max().unwrap_or(0) + 1;
    for rd in 0..rounds {
        let at: Vec<usize> = (0..cand.len()).filter(|&k| round[k] == rd).collect();
        let cv: V = at.iter().map(|&k| cov[k]).collect();
        let inv = o.sum(&[(v(&cv), -1)], 100);
        for c in 0..3 {
            let cur: V = at.iter().map(|&k| col[c][pix[k]]).collect();
            let wcol: V = at.iter().map(|&k| wc[c][k]).collect();
            let r = o.mix(v(&cur), v(&wcol), v(&cv));
            let hh: V = at.iter().map(|&k| hot[c][pix[k]]).collect();
            let h2 = o.pct(v(&hh), v(&inv));
            for (j, &k) in at.iter().enumerate() { col[c][pix[k]] = r[j]; hot[c][pix[k]] = h2[j]; }
        }
    }
}
