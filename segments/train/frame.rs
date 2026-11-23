// A frame of the ride: the camera, the ground and the triangles into a buffer of what each pixel sees, then the
// shading (shade.rs). Included after render.rs.

#[allow(non_snake_case)]
pub fn N_PIX() -> usize { (W() * H()) as usize }

/// What each pixel sees.
pub struct GBuf {
    /// K / z (0 for the sky)
    pub iz: V,
    /// the material (an index into the scene's), or none (-1, the sky)
    pub mat: Vec<i64>,
    /// the normal, hundredths
    pub n: [V; 3],
    /// which list the triangle came from (0 the ground, 1 the world, 2 the other train)
    pub src: Vec<u8>,
    /// which triangle of the list (the ground's kind for the ground)
    pub tri: Vec<u32>,
}

/// The ray through each pixel, at 0.7 of its length (the camera's forward part 700): thousandths. Along a row the ray
/// changes by the same each pixel: each quarter of a row from its first pixel's ray (the row's sum 7 (x b0 + y b1 +
/// 1900 b2) over 19,000, x and y in half pixels, its rest in thousandths) by the step 14 b0 1000 / 19,000 (walkc).
pub fn rays<O: Ops>(o: &mut O, cam: &Cam) -> [V; 3] {
    let Q: i64 = W() / 4;
    let ys: V = (0..H()).map(|y| H() - 1 - 2 * y).collect();
    let mut out: [V; 3] = [vec![], vec![], vec![]];
    for i in 0..3 {
        // each quarter's first pixel, every row: (x in half pixels 2 x0 + 1 - W)
        let xq: V = (0..H()).flat_map(|_| (0..4).map(|q| 2 * q * Q + 1 - W())).collect();
        let yq: V = (0..H() as usize).flat_map(|y| std::iter::repeat_n(ys[y], 4)).collect();
        let s = { let k_ = kl(o, &[(cam.f2, Some(cam.b[2][i]))]); o.lin(&[(v(&xq), Some(K(cam.b[0][i]))), (v(&yq), Some(K(cam.b[1][i]))), (K(k_), None)]) };
        let s = o.lin(&[(v(&s), Some(K(7)))]);
        let va = o.div(v(&s), K(10 * cam.f2), 0);
        let rest = o.lin(&[(v(&s), None), (v(&va), Some(K(-10 * cam.f2)))]);
        let f0 = { let t = o.lin(&[(v(&rest), Some(K(1000)))]); o.div(v(&t), K(10 * cam.f2), 0) };
        let step = one({ let t = o.lin(&[(K(cam.b[0][i]), Some(K(14_000)))]); o.div(v(&t), K(10 * cam.f2), 0) });
        let walks: Vec<(i64, i64, i64, i64)> = (0..va.len()).map(|j| (Q, va[j], step, f0[j])).collect();
        out[i] = o.walkc(&walks);
    }
    out
}

/// The depth of the level ground (the plane y = 0) at each pixel, K / z; 0 or less where the ray doesn't go down. The
/// ray's height part first in hundred-thousandths, then K over the eye's height: worked out at each quarter of a row's
/// ends, and between by the slope (walka).
pub fn ground_depth<O: Ops>(o: &mut O, tb: &Tb, cam: &Cam) -> V {
    let Q: i64 = W() / 4;
    let ys: V = (0..H()).map(|y| H() - 1 - 2 * y).collect();
    let xq: V = (0..H()).flat_map(|_| (0..5).map(|q| 2 * q * Q + 1 - W())).collect();
    let yq: V = (0..H() as usize).flat_map(|y| std::iter::repeat_n(ys[y], 5)).collect();
    let s = { let k_ = kl(o, &[(cam.f2, Some(cam.b[2][1]))]); o.lin(&[(v(&xq), Some(K(cam.b[0][1]))), (v(&yq), Some(K(cam.b[1][1]))), (K(k_), None)]) };
    let s = o.lin(&[(v(&s), Some(K(-100)))]);
    let dy = o.div(v(&s), K(cam.f2), 0);
    let s = o.lin(&[(v(&dy), Some(K(999)))]);
    let s = o.lin(&[(v(&s), Some(K(3)))]);
    // (an eye higher than 99.999 m: over its height in centimetres, then over 10)
    let iz = if cam.eye[1] <= 99_999 { o.div(v(&s), K(cam.eye[1]), 0) } else { let q = o.div(v(&s), K(cam.eye_cm[1]), 0); o.div(v(&q), K(10), 0) };
    let a_: V = (0..H() as usize).flat_map(|y| (0..4).map(move |q| y * 5 + q)).map(|j| iz[j]).collect();
    let b_: V = (0..H() as usize).flat_map(|y| (0..4).map(move |q| y * 5 + q + 1)).map(|j| iz[j]).collect();
    let n = vec![Q; a_.len()];
    let sl = slopes(o, tb, &a_, &b_, &n);
    let walks: Vec<(i64, i64, i64, i64)> = (0..a_.len()).map(|j| (Q, a_[j], sl[j], 0)).collect();
    o.walka(&walks)
}

/// Places where rays meet surfaces: the eye plus the ray at 0.7 times z / 0.7 (centimetres; x, z and, when asked, y).
pub fn places<O: Ops>(o: &mut O, tb: &Tb, cam: &Cam, ray: &[&V; 3], iz: &V, want_y: bool) -> [V; 3] {
    let liz = o.tab(tb.t("lniz4"), v(iz));
    places_l(o, tb, cam, ray, &liz, want_y)
}

/// The same from the depths' logarithms (ln K / z, ten-thousandths).
pub fn places_l<O: Ops>(o: &mut O, tb: &Tb, cam: &Cam, ray: &[&V; 3], liz: &V, want_y: bool) -> [V; 3] {
    let mut out: [V; 3] = [vec![], vec![], vec![]];
    for i in 0..3 {
        if i == 1 && !want_y { continue; }
        let t = reach(o, tb, ray[i], liz);
        out[i] = o.sum(&[(v(&t), 1)], cam.eye_cm[i]);
    }
    out
}

/// How far a ray's part goes to the depth (centimetres): ln |r| + ln (KZ / 0.7) / 1000 less ln K / z (ten-thousandths), e
/// back by the table of the ray's sign.
pub fn reach<O: Ops>(o: &mut O, tb: &Tb, r: &V, liz: &V) -> V {
    let n = liz.len();
    let lr = o.tabo(tb.t("lnr4"), 999, v(r));
    let k = o.ldiff(&lr, liz, 0, 139);
    let neg = o.tabo(tb.t("neg"), 999, v(r));
    let mut t = vec![0; n];
    for (name, want) in [("exp4", 0), ("exp4n", 1)] {
        let at: Vec<usize> = (0..n).filter(|&j| neg[j] == want).collect();
        let kk: V = at.iter().map(|&j| k[j]).collect();
        let e_ = o.tab(tb.t(name), v(&kk));
        for (q, &j) in at.iter().enumerate() { t[j] = e_[q]; }
    }
    t
}

/// The ground's kinds, as the prototype's ground(): the deck at y = 0 where it is (the line's half width by stretch:
/// the viaduct, the cutting, the tunnel, the bridge), else the street in the city, else the land at -1.5 m: the
/// embankment beside the line, the river's water and banks, the paddies.
/// `town0`: past this place along the line (cm) the land is the town's streets, not paddies (the night's).
pub fn ground<O: Ops>(o: &mut O, w: &World, cam: &Cam, ray: &[V; 3], g: &mut GBuf, lists: &[(u8, &Drawn, &Mesh)], town0: Option<i64>) {
    let (sc, tb) = (&w.sc, &w.tb);
    let iz0 = ground_depth(o, &w.tb, cam);
    // the pixels looking down, near enough (K / z at least 75: within 4 km)
    let down = o.lt(K(74), v(&iz0));
    // not where a piece (not a leaf, which may be cut out) lies nearer than the level ground: every ground is at or
    // below it, so none would be seen
    let mut cand: Vec<(usize, i64)> = vec![];
    for (_, d, mesh) in lists { for &(p, z, t, _) in &d.pieces { if down[p] == 1 && sc.mats[mesh.mat[t]].kind != kind::TREE { cand.push((p, z)); } } }
    let zs: V = cand.iter().map(|c| c.1).collect();
    let z0: V = cand.iter().map(|c| iz0[c.0]).collect();
    let nearer = o.lt(v(&z0), v(&zs));
    let mut covered = vec![false; N_PIX()];
    for (i, c) in cand.iter().enumerate() { if nearer[i] == 1 { covered[c.0] = true; } }
    let px: Vec<usize> = (0..N_PIX()).filter(|&p| down[p] == 1 && !covered[p]).collect();
    if px.is_empty() { return; }
    let pick = |v: &V| -> V { px.iter().map(|&p| v[p]).collect() };
    let iz = pick(&iz0);
    let r: [V; 3] = std::array::from_fn(|i| pick(&ray[i]));
    let liz = o.tab(tb.t("lniz4"), v(&iz));
    // where on a plane: along the line, across it from the middle (less the bend), and the stretch it is in (the
    // host's table of stretches by the place along)
    let at = |o: &mut O, rr: &[V; 3], liz: &V| -> (V, V, V) {
        let tz = reach(o, tb, &rr[2], liz);
        let z = o.sum(&[(v(&tz), 1)], cam.eye_cm[2]);
        // (the night's cameras look every way: its far land held to the tables' 10 km either way)
        let z = if town0.is_some() { o.clamp(v(&z), -99_999, 999_999) } else { z };
        let b = bend_at(o, &sc.t, &z);
        let tx = reach(o, tb, &rr[0], liz);
        let xr = { let c = kl(o, &[(cam.eye_cm[0], None), (-MID, None)]); o.sum(&[(v(&tx), 1), (v(&b), -1)], c) };
        let zone = o.tabo(tb.t("g_zone"), 100_000, v(&z));
        (z, xr, zone)
    };
    let (_, xr, zone) = at(o, &r, &liz);
    // the deck: within the line's half width by stretch (the viaduct, the cutting, the tunnel, the bridge)
    let hw = o.tab(tb.t("g_hw"), v(&zone));
    let (on, _) = o.absless(&xr, v(&hw));
    for (k, &q) in px.iter().enumerate() {
        if on[k] == 1 { g.iz[q] = iz[k]; g.mat[q] = kind::DECK; g.n[1][q] = 99; g.src[q] = 0; }
    }
    // the rest: the street (in the city) or the land, the plane lower: K / z times the ratio of the eye's heights, by
    // logarithms (e back by the table)
    let rest: Vec<usize> = (0..px.len()).filter(|&k| on[k] == 0).collect();
    if rest.is_empty() { return; }
    let lower = |o: &mut O, h: i64, liz: &V| -> (V, V) {
        // (the plane's ratio, its divisor and its log on the machines)
        let dm = kl(o, &[(cam.eye[1], None), (h, Some(-10))]);
        let ratio = if dm <= 99_999 { let num = kl(o, &[(cam.eye[1], Some(1000))]); one(o.div(K(num), K(dm), 0)) }
            else { let (num, d) = (kl(o, &[(cam.eye_cm[1], Some(1000))]), kl(o, &[(cam.eye_cm[1], None), (h, Some(-1))])); one(o.div(K(num), K(d), 0)) };
        let c = one(o.sum(&[(K(tb.t("lniz4").data[ratio as usize]), 1), (K(tb.t("lniz4").data[1000]), -1)], 0));
        let cs = vec![one(o.sum(&[(K(c), -1)], 0)); liz.len()];
        let l = o.ldiff(liz, &cs, 0, 139);
        let iz = o.tab(tb.t("exp4"), v(&l));
        (l, iz)
    };
    let city = cam.z_cm < TUN0 - 2000;
    let lz0: V = rest.iter().map(|&k| liz[k]).collect();
    let (liz1, iz1) = lower(o, if city { STREET } else { LAND }, &lz0);
    if city {
        for (j, &k) in rest.iter().enumerate() { let q = px[k]; g.iz[q] = iz1[j]; g.mat[q] = kind::STREET; g.n[1][q] = 99; g.src[q] = 0; }
        return;
    }
    let r1: [V; 3] = std::array::from_fn(|i| rest.iter().map(|&k| r[i][k]).collect());
    let (z1, xr1, zone1) = at(o, &r1, &liz1);
    let street = match town0 { Some(t0) => o.lt(K(t0), v(&z1)), None => vec![0; z1.len()] };
    // the embankment beside the line (not on the bridge), the river's stretch, the paddies
    let (near_line, sgn) = o.absless(&xr1, K(800));
    let on_bridge = o.tab(tb.t("g_bridge"), v(&zone1));
    let river_z = o.tab(tb.t("g_river"), v(&zone1));
    let emb = o.sel(&on_bridge, K(0), v(&near_line));
    let river = o.sel(&emb, K(0), v(&river_z));
    for (j, &k) in rest.iter().enumerate() {
        let q = px[k];
        g.iz[q] = iz1[j];
        g.src[q] = 0;
        if emb[j] == 1 {
            g.mat[q] = kind::EMBANK;
            g.n[0][q] = if sgn[j] == 1 { -54 } else { 54 }; g.n[1][q] = 82; g.n[2][q] = 0;
        } else if river[j] == 0 {
            g.mat[q] = if street[j] == 1 { kind::STREET } else { kind::PADDY }; g.n[1][q] = 99;
        }
    }
    // the river: the water's plane further down, or the bank where the ray meets the land's level short of it
    let rv: Vec<usize> = (0..rest.len()).filter(|&j| river[j] == 1).collect();
    if rv.is_empty() { return; }
    let lz1: V = rv.iter().map(|&j| lz0[j]).collect();
    let (liz2, iz2) = lower(o, RIVER, &lz1);
    let r2z: V = rv.iter().map(|&j| r1[2][j]).collect();
    let tz2 = reach(o, tb, &r2z, &liz2);
    let z2 = o.sum(&[(v(&tz2), 1)], cam.eye_cm[2]);
    let z2 = if town0.is_some() { o.clamp(v(&z2), -99_999, 999_999) } else { z2 };
    let zone2 = o.tabo(tb.t("g_zone"), 100_000, v(&z2));
    let water = o.tab(tb.t("g_water"), v(&zone2));
    let zr: V = rv.iter().map(|&j| zone1[j]).collect();
    let mid = o.tab(tb.t("g_mid"), v(&zr));
    for (i, &j) in rv.iter().enumerate() {
        let q = px[rest[j]];
        if water[i] == 1 { g.iz[q] = iz2[i]; g.mat[q] = kind::WATER; g.n[1][q] = 99; }
        else { g.mat[q] = kind::BANK; g.n[0][q] = 0; g.n[1][q] = 84; g.n[2][q] = if mid[i] == 1 { 50 } else { -50 }; }
    }
}

/// Which of the world's triangles a frame considers, as the prototype's visible(): ahead within 2.6 km; from the
/// city and the tunnel's first half only what's this side of the hill; through the exit only what the portal frames;
/// small things only within 900 m; trees as their full model within 260 m, one crown beyond. The host's sorting, by
/// the cards' punched places against the camera's (metres). Those reaching 800 m on or more are counted far
/// (centimetres).
pub fn visible(sc: &Scene, cz: i64) -> (Vec<usize>, Vec<usize>) {
    let (t0, t1) = (TUN0 / 100, TUN1 / 100);
    let (mut near, mut far) = (vec![], vec![]);
    for k in 0..sc.world.tris.len() {
        let (zlo, zhi) = (sc.zlo[k], sc.zhi[k]);
        let mut s = zhi > cz - 5 && zlo < cz + 2600;
        if cz < t0 + 120 { s &= zlo < t0 + 260; } else if cz < t1 { s &= zhi < t1 + 5 || (zlo < t1 + 1200 && (sc.xmid[k] - MID / 100 - 30).abs() < 260); }
        s &= !sc.small[k] || zlo < cz + 900;
        let close = sc.iz[k] < cz + 260;
        s &= sc.lod[k] == 0 || (sc.lod[k] == 1 && close) || (sc.lod[k] == 2 && !close);
        if s { if zhi >= cz + 800 { far.push(k) } else { near.push(k) } }
    }
    (near, far)
}

/// The other train's corners this frame: its own places (centimetres, its nose at 0) moved to the other line, its
/// nose at MEET + (MEET / 7 m - frame) 7 m, and bent with the line.
pub fn train_corners<O: Ops>(o: &mut O, sc: &Scene, k: i64) -> (i64, Vec<[i64; 3]>) {
    let front = one(o.lin(&[(K(k), Some(K(-SPEED))), (K(2 * MEET), None)]));
    let lx: V = sc.train.corners.iter().map(|c| c[0]).collect();
    let lz: V = sc.train.corners.iter().map(|c| c[2]).collect();
    let z = o.lin(&[(v(&lz), None), (K(front), None)]);
    let b = bend_at(o, &sc.t, &z);
    let x = o.lin(&[(v(&lx), None), (v(&b), None), (K(X2), None)]);
    (front, (0..lx.len()).map(|i| [x[i], sc.train.corners[i][1], z[i]]).collect())
}

/// The other train as a list at its places this frame: its corners there, its planes' distances worked out again
/// (the face normal's dot with a corner, hundredths of a cm).
pub fn placed_train<O: Ops>(o: &mut O, sc: &Scene, tc: &[[i64; 3]]) -> Mesh {
    let m = &sc.train;
    let c: [V; 3] = std::array::from_fn(|i| m.tris.iter().map(|t| tc[t[0]][i]).collect());
    let f: [V; 3] = std::array::from_fn(|i| m.fnorm.iter().map(|n| n[i]).collect());
    let fd = o.lin(&[(v(&f[0]), Some(v(&c[0]))), (v(&f[1]), Some(v(&c[1]))), (v(&f[2]), Some(v(&c[2])))]);
    Mesh { corners: tc.to_vec(), tris: m.tris.clone(), vn: m.vn.clone(), fnorm: m.fnorm.clone(), fd, mat: m.mat.clone(), smooth: m.smooth.clone() }
}

/// One frame: what each pixel sees.
pub fn gbuffer<O: Ops>(o: &mut O, w: &World, cam: &Cam, k: i64) -> (GBuf, [V; 3], Option<i64>, V) {
    let sc = &w.sc;
    let ray = rays(o, cam);
    let mut g = GBuf { iz: vec![0; N_PIX()], mat: vec![-1; N_PIX()], n: [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]], src: vec![0; N_PIX()], tri: vec![0; N_PIX()] };
    let cz = cam.z_cm / 100;
    let (near, far) = visible(sc, cz);
    let two: Vec<bool> = sc.world.mat.iter().map(|&m| sc.mats[m].kind == kind::TREE || sc.mats[m].kind == kind::LED).collect();
    let dn = draw(o, &w.tb, cam, &sc.world, &sc.world.corners, &two, &near, false, 1);
    let df = draw(o, &w.tb, cam, &sc.world, &sc.world.corners, &two, &far, true, 1);
    let (front, tc) = train_corners(o, sc, k);
    let mut lists = vec![(1u8, dn), (1u8, df)];
    let mut front_at = None;
    if front < cam.z_cm + 300_000 && front + CARS * CAR > cam.z_cm - 500 {
        let all: Vec<usize> = (0..sc.train.tris.len()).collect();
        let none = vec![false; sc.train.tris.len()];
        let far = front + CARS * CAR >= cam.z_cm + 80_000;
        let tcf: Vec<[i64; 3]> = if far { tc.clone() } else { tc.clone() };
        let tm = placed_train(o, sc, &tcf);
        lists.push((2u8, draw(o, &w.tb, cam, &tm, &tcf, &none, &all, far, 1)));
        front_at = Some(front);
    }
    let refs: Vec<(u8, &Drawn, &Mesh)> = lists.iter().map(|(src, d)| (*src, d, if *src == 2 { &sc.train } else { &sc.world })).collect();
    ground(o, w, cam, &ray, &mut g, &refs, None);
    let ground_iz = g.iz.clone();
    resolve(o, &mut g, &refs, Some(Cut { w, cam, ray: &ray, mirror_h: None, nlq7: &w.screen().nlq7 }));
    // at half resolution (the party version's world): the world's own depth behind its full-resolution subjects
    let wiz = if res() > 1 { world_depth(o, &g, &ground_iz, &refs, &|_, m| train_subject(w, m)) } else { vec![] };
    (g, ray, front_at, wiz)
}

/// What the train's party version draws at full resolution: the departure's board (its LED words).
pub fn train_subject(w: &World, mat: usize) -> bool { w.sc.mats[mat].kind == kind::LED }

/// The world's own depth at each pixel (a half-resolution buffer's): where a subject is nearest, the nearest of the
/// rest (the ground's, then the lists' other pieces; the leaves taken as they come), else the buffer's.
pub fn world_depth<O: Ops>(o: &mut O, g: &GBuf, ground_iz: &[i64], refs: &[(u8, &Drawn, &Mesh)], subj: &dyn Fn(u8, usize) -> bool) -> V {
    let mut wiz = g.iz.clone();
    let sp: Vec<usize> = (0..N_PIX()).filter(|&p| g.mat[p] >= 0 && subj(g.src[p], g.mat[p] as usize)).collect();
    if sp.is_empty() { return wiz; }
    let at: std::collections::HashMap<usize, usize> = sp.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let mut groups: Vec<Vec<i64>> = sp.iter().map(|&p| if ground_iz[p] > 0 { vec![ground_iz[p]] } else { vec![] }).collect();
    for (src, d, mesh) in refs { for &(p, z, t, _) in &d.pieces { if let Some(&i) = at.get(&p) { if !subj(*src, mesh.mat[t]) { groups[i].push(z); } } } }
    let multi: Vec<usize> = (0..sp.len()).filter(|&i| groups[i].len() > 1).collect();
    let refs2: Vec<&[i64]> = multi.iter().map(|&i| groups[i].as_slice()).collect();
    let win = o.nearest(&refs2);
    for (i, &p) in sp.iter().enumerate() { wiz[p] = if groups[i].len() == 1 { groups[i][0] } else { 0 }; }
    for (j, &i) in multi.iter().enumerate() { wiz[sp[i]] = groups[i][win[j]]; }
    wiz
}

/// What the foliage's cut-out needs: the frame's camera and rays, and the mirror's level if this is a mirror.
pub struct Cut<'a> { pub w: &'a World, pub cam: &'a Cam, pub ray: &'a [V; 3], pub mirror_h: Option<i64>, pub nlq7: &'a [i64] }

/// The depth test: each pixel's pieces (what the buffer has first, then the lists' in order), the nearest kept. With
/// the foliage cut out (the prototype's): where the nearest is a leaf, it stays only where the leaves' noise at its
/// place passes a threshold rising toward the crown's outline (seen edge on); else it's dropped and the nearest of the
/// rest taken, and so on.
pub fn resolve<O: Ops>(o: &mut O, g: &mut GBuf, lists: &[(u8, &Drawn, &Mesh)], cut: Option<Cut>) {
    let mut per: Vec<Vec<(i64, u8, usize, [i64; 3], u32)>> = vec![vec![]; N_PIX()];
    for p in 0..N_PIX() { if g.mat[p] >= 0 { per[p].push((g.iz[p], g.src[p], g.mat[p] as usize, [g.n[0][p], g.n[1][p], g.n[2][p]], g.mat[p] as u32)); } }
    for (src, d, mesh) in lists {
        for &(p, z, t, n) in &d.pieces { per[p].push((z, *src, mesh.mat[t], n, t as u32)); }
    }
    let mut pick = vec![0usize; N_PIX()];
    let mut open: Vec<usize> = (0..N_PIX()).filter(|&p| !per[p].is_empty()).collect();
    loop {
        let multi: Vec<usize> = open.iter().copied().filter(|&p| per[p].len() > 1).collect();
        let zs: Vec<Vec<i64>> = multi.iter().map(|&p| per[p].iter().map(|q| q.0).collect()).collect();
        let refs: Vec<&[i64]> = zs.iter().map(|z| z.as_slice()).collect();
        let win = o.nearest(&refs);
        for &p in &open { pick[p] = 0; }
        for (i, &p) in multi.iter().enumerate() { pick[p] = win[i]; }
        let Some(c) = &cut else { break };
        // the winners that are leaves: tested
        let test: Vec<usize> = open.iter().copied().filter(|&p| c.w.sc.mats[per[p][pick[p]].2].kind == kind::TREE).collect();
        if test.is_empty() { break; }
        let keep = leaves_kept(o, c, &test, &per, &pick);
        let mut next = vec![];
        for (i, &p) in test.iter().enumerate() {
            if keep[i] == 1 { continue; }
            per[p].remove(pick[p]);
            if !per[p].is_empty() { next.push(p); }
        }
        if next.is_empty() { break; }
        open = next;
    }
    for p in 0..N_PIX() {
        if per[p].is_empty() { g.mat[p] = -1; g.iz[p] = 0; continue; }
        let (z, src, m, n, t) = per[p][pick[p]];
        g.iz[p] = z; g.mat[p] = m as i64; g.src[p] = src; g.tri[p] = t;
        for c in 0..3 { g.n[c][p] = n[c]; }
    }
}

/// The cut-out's test for leaf pieces at pixels: the place (from the depth, flipped back in a mirror), two noises
/// (cells 1/1.7 m and 1/4.6 m, leaning with the height), 0.65 and 0.35 of them, against 0.12 + 0.75 (1 - facing)^2.
pub fn leaves_kept<O: Ops>(o: &mut O, c: &Cut, px: &[usize], per: &[Vec<(i64, u8, usize, [i64; 3], u32)>], pick: &[usize]) -> V {
    let (w, cam) = (c.w, c.cam);
    let iz: V = px.iter().map(|&p| per[p][pick[p]].0).collect();
    let r: [V; 3] = std::array::from_fn(|i| px.iter().map(|&p| c.ray[i][p]).collect());
    let pl = places(o, &w.tb, cam, &[&r[0], &r[1], &r[2]], &iz, true);
    let y = match c.mirror_h { Some(h) => o.lin(&[(v(&pl[1]), Some(K(-1))), (K(2 * h), None)]), None => pl[1].clone() };
    let (x, z) = (&pl[0], &pl[2]);
    let a1 = { let t = o.lin(&[(v(&y), Some(K(47)))]); o.div(v(&t), K(100), 0) };
    let b1 = { let t = o.lin(&[(v(&y), Some(K(-35)))]); o.div(v(&t), K(100), 0) };
    let x1 = o.lin(&[(v(x), None), (v(&a1), None)]);
    let z1 = o.lin(&[(v(z), None), (v(&b1), None)]);
    let n1 = { let i = o.wrap(v(&x1), 1); let j = o.wrap(v(&z1), 1); o.tab2(w.tb.t("n_cut1"), v(&j), v(&i)) };
    let a2 = { let t = o.lin(&[(v(&y), Some(K(-22)))]); o.div(v(&t), K(100), 0) };
    let b2 = { let t = o.lin(&[(v(&y), Some(K(28)))]); o.div(v(&t), K(100), 0) };
    let x2 = o.lin(&[(v(x), None), (v(&a2), None)]);
    let z2 = o.lin(&[(v(z), None), (v(&b2), None)]);
    let n2 = { let i = o.wrap(v(&x2), 1); let j = o.wrap(v(&z2), 1); o.tab2(w.tb.t("n_cut2"), v(&j), v(&i)) };
    let lv = o.lin(&[(v(&n1), Some(K(65))), (v(&n2), Some(K(35)))]);
    let lv = o.div(v(&lv), K(100), 0);
    // facing: the unit ray's dot with the normal
    let q: V = px.iter().map(|&p| c.nlq7[p]).collect();
    let d = unit_rays(o, &w.tb, &[&r[0], &r[1], &r[2]], &q);
    let mut nn: [V; 3] = std::array::from_fn(|i| px.iter().map(|&p| per[p][pick[p]].3[i]).collect());
    if c.mirror_h.is_some() { nn[1] = o.sum(&[(v(&nn[1]), -1)], 0); }
    let mut dd = d.clone();
    if c.mirror_h.is_some() { dd[1] = o.sum(&[(v(&d[1]), -1)], 0); }
    let a = o.mul(v(&dd[0]), v(&nn[0]), 2);
    let b = o.mul(v(&dd[1]), v(&nn[1]), 2);
    let cc = o.mul(v(&dd[2]), v(&nn[2]), 2);
    let f = o.sum(&[(v(&a), 1), (v(&b), 1), (v(&cc), 1)], 0);
    let fm = o.sum(&[(v(&f), -1)], 0);
    let f = o.max(v(&f), v(&fm));
    let f = o.min(v(&f), K(999));
    let need = o.tab(w.tb.t("cut_need"), v(&f));
    o.lt(v(&need), v(&lv))
}
