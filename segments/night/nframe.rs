// A frame of the night: the camera, what each pixel sees
// (the world's triangles the frame considers, our own train, the drone and the airship placed for the frame, the
// ground), the sky going from sunset into night, the shading with the night's materials and lamps, the motion blur,
// the wires, what the night adds over it all (npost.rs), the lens and the tone. Included after path.rs.

/// Which picture of a frame: the night's world (New), the ride's own world it dissolves in from (Old: the first
/// XFADE frames), or the night's world from the cab (Cab: the cab's half of the dissolve to the platform).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode { New, Old, Cab }

/// What night_visible() picks the world's triangles by, worked out at the start on the machines from the triangles'
/// extents (metres): for each mode's triangles, spans of the camera's place along (cz, metres) for which a frame
/// considers one, lo < cz < hi (a crown of the night's world in two spans, short of its tree's full model and past
/// it); and for every triangle where it is near, zhi - 800 < cz < zlo + 800 and xmid - 740 < cx < xmid + 740 (cx the
/// camera's place across, metres).
#[derive(Default)]
pub struct NightCull { pub old: Spans, pub new: Spans, pub near: [V; 4] }

/// Spans of the camera's place: each one's triangle (in order), lo and hi.
#[derive(Default)]
pub struct Spans { pub k: Vec<usize>, pub lo: V, pub hi: V }

/// The bounds. The ride's world (Old): lo = zlo - 2600 (small things zlo - 900), hi = zhi + 5. The night's (New): lo
/// = zlo - 1600, hi = zhi + 1400 (small things zlo - 650 and zhi + 650). A tree's full model within 260 m (lo at least
/// iz - 260, and in the night's world hi at most iz + 260), its crown beyond (hi at most iz - 259; in the night's
/// world also a span from lo at least iz + 259).
pub fn night_cull<O: Ops>(o: &mut O, nw: &NightWorld) -> NightCull {
    let sc = &nw.w.sc;
    let of = |x: &V, ks: &[usize]| -> V { ks.iter().map(|&k| x[k]).collect() };
    let spans = |o: &mut O, on: &[bool], small: &[bool], d: [i64; 4], both: bool| -> Spans {
        let ks: Vec<usize> = (0..on.len()).filter(|&k| on[k]).collect();
        let (zlo, zhi) = (of(&sc.zlo, &ks), of(&sc.zhi, &ks));
        let sm: V = ks.iter().map(|&k| small[k] as i64).collect();
        let (a, b) = (o.sum(&[(v(&zlo), 1)], d[0]), o.sum(&[(v(&zlo), 1)], d[1]));
        let mut lo = o.sel(&sm, v(&a), v(&b));
        let (a, b) = (o.sum(&[(v(&zhi), 1)], d[2]), o.sum(&[(v(&zhi), 1)], d[3]));
        let mut hi = o.sel(&sm, v(&a), v(&b));
        let lod: Vec<i64> = ks.iter().map(|&k| sc.lod[k]).collect();
        let iz = of(&sc.iz, &ks);
        let full: Vec<usize> = (0..ks.len()).filter(|&j| lod[j] == 1).collect();
        let z = o.sum(&[(v(&of(&iz, &full)), 1)], -260);
        let m = o.max(v(&of(&lo, &full)), v(&z));
        for (i, &j) in full.iter().enumerate() { lo[j] = m[i]; }
        if both {
            let z = o.sum(&[(v(&of(&iz, &full)), 1)], 260);
            let m = o.min(v(&of(&hi, &full)), v(&z));
            for (i, &j) in full.iter().enumerate() { hi[j] = m[i]; }
        }
        let crown: Vec<usize> = (0..ks.len()).filter(|&j| lod[j] == 2).collect();
        let z = o.sum(&[(v(&of(&iz, &crown)), 1)], -259);
        let short = o.min(v(&of(&hi, &crown)), v(&z));
        let past = if both { let z = o.sum(&[(v(&of(&iz, &crown)), 1)], 259); o.max(v(&of(&lo, &crown)), v(&z)) } else { vec![] };
        let mut s = Spans::default();
        let mut c = 0;
        for j in 0..ks.len() {
            if lod[j] == 2 {
                s.k.push(ks[j]); s.lo.push(lo[j]); s.hi.push(short[c]);
                if both { s.k.push(ks[j]); s.lo.push(past[c]); s.hi.push(hi[j]); }
                c += 1;
            } else { s.k.push(ks[j]); s.lo.push(lo[j]); s.hi.push(hi[j]); }
        }
        s
    };
    let old = spans(o, &nw.old, &nw.small_old, [-900, -2600, 5, 5], false);
    let new = spans(o, &nw.new, &sc.small, [-650, -1600, 650, 1400], true);
    let (zlo, zhi, xmid) = (v(&sc.zlo), v(&sc.zhi), v(&sc.xmid));
    let near = [o.sum(&[(zhi, 1)], -800), o.sum(&[(zlo, 1)], 800), o.sum(&[(xmid, 1)], -740), o.sum(&[(xmid, 1)], 740)];
    NightCull { old, new, near }
}

/// The lists of the world's triangles a frame considers (the ride's visible() for
/// the old world): near (mm) and far (cm, those reaching 800 m off or more). By the bounds worked out at the start
/// against the camera's place (metres), on the machines: cz < hi for the mode's spans, then lo < cz for those; then
/// the four tests for near, each on those that passed the last.
pub fn night_visible<O: Ops>(o: &mut O, nw: &NightWorld, cam: &Cam, mode: Mode) -> (Vec<usize>, Vec<usize>) {
    let (cz, cx) = (metres(o, cam.z_cm), metres(o, cam.eye_cm[0]));
    let s = if mode == Mode::Old { &nw.cull.old } else { &nw.cull.new };
    let ahead = o.lt(K(cz), v(&s.hi));
    let j1: Vec<usize> = (0..s.k.len()).filter(|&j| ahead[j] == 1).collect();
    let lo: V = j1.iter().map(|&j| s.lo[j]).collect();
    let within = o.lt(v(&lo), K(cz));
    let shown: Vec<usize> = (0..j1.len()).filter(|&i| within[i] == 1).map(|i| s.k[j1[i]]).collect();
    // near: zhi - 800 < cz, cz < zlo + 800, xmid - 740 < cx, cx < xmid + 740
    let nb = &nw.cull.near;
    let mut keep: Vec<usize> = (0..shown.len()).collect();
    for (b, x, below) in [(&nb[0], cz, true), (&nb[1], cz, false), (&nb[2], cx, true), (&nb[3], cx, false)] {
        let bs: V = keep.iter().map(|&i| b[shown[i]]).collect();
        let f = if below { o.lt(v(&bs), K(x)) } else { o.lt(K(x), v(&bs)) };
        keep = (0..keep.len()).filter(|&i| f[i] == 1).map(|i| keep[i]).collect();
    }
    let mut is_near = vec![false; shown.len()];
    for &i in &keep { is_near[i] = true; }
    let (mut near, mut far) = (vec![], vec![]);
    for (i, &k) in shown.iter().enumerate() { if is_near[i] { near.push(k) } else { far.push(k) } }
    (near, far)
}

/// Whether a moving model is far: the largest of its first corner's offsets from the eye past 60,000 (cm). Each
/// offset, on the machines, against 60,000 and -60,000; the six flags added up.
pub fn mover_far<O: Ops>(o: &mut O, c: [i64; 3], eye: [i64; 3]) -> bool {
    let d = o.lin(&[(v(&c.to_vec()), None), (v(&eye.to_vec()), Some(K(-1)))]);
    let a: V = [60_000; 3].into_iter().chain(d.iter().copied()).collect();
    let b: V = d.iter().copied().chain([-60_000; 3]).collect();
    let f = o.lt(v(&a), v(&b));
    let t: Vec<(A, i64)> = f.iter().map(|&x| (K(x), 1)).collect();
    one(o.sum(&t, 0)) > 0
}

/// A list of triangles moved for the frame: their corners (cm) moved, their normals turned by rows of thousandths if
/// given, their planes' distances worked out again.
pub fn placed<O: Ops>(o: &mut O, m: &Mesh, turn: Option<[[i64; 3]; 3]>, shift: [i64; 3]) -> Mesh {
    let col = |c: &[[i64; 3]], i: usize| -> V { c.iter().map(|p| p[i]).collect() };
    let rot = |o: &mut O, x: &V, y: &V, z: &V, r: &[[i64; 3]; 3], scale: i64| -> [V; 3] {
        std::array::from_fn(|i| { let t = o.lin(&[(v(x), Some(K(r[i][0]))), (v(y), Some(K(r[i][1]))), (v(z), Some(K(r[i][2])))]); o.div(v(&t), K(scale), 0) })
    };
    let (cx, cy, cz) = (col(&m.corners, 0), col(&m.corners, 1), col(&m.corners, 2));
    let p = match &turn { Some(r) => rot(o, &cx, &cy, &cz, r, 1000), None => [cx, cy, cz] };
    let p: [V; 3] = std::array::from_fn(|i| o.lin(&[(v(&p[i]), None), (K(shift[i]), None)]));
    let corners: Vec<[i64; 3]> = (0..m.corners.len()).map(|k| [p[0][k], p[1][k], p[2][k]]).collect();
    let (fnorm, vn) = match &turn {
        None => (m.fnorm.clone(), m.vn.clone()),
        Some(r) => {
            let f = rot(o, &col(&m.fnorm, 0), &col(&m.fnorm, 1), &col(&m.fnorm, 2), r, 1000);
            let fnorm: Vec<[i64; 3]> = (0..m.fnorm.len()).map(|k| [f[0][k], f[1][k], f[2][k]]).collect();
            let mut vn = m.vn.clone();
            for j in 0..3 {
                let g = |i: usize| -> V { m.vn.iter().map(|n| n[3 * j + i]).collect() };
                let t = rot(o, &g(0), &g(1), &g(2), r, 1000);
                for k in 0..vn.len() { for i in 0..3 { vn[k][3 * j + i] = t[i][k]; } }
            }
            (fnorm, vn)
        }
    };
    let c0: [V; 3] = std::array::from_fn(|i| m.tris.iter().map(|t| corners[t[0]][i]).collect());
    let f: [V; 3] = std::array::from_fn(|i| fnorm.iter().map(|n| n[i]).collect());
    let fd = o.lin(&[(v(&f[0]), Some(v(&c0[0]))), (v(&f[1]), Some(v(&c0[1]))), (v(&f[2]), Some(v(&c0[2])))]);
    Mesh { corners, tris: m.tris.clone(), vn, fnorm, fd, mat: m.mat.clone(), smooth: m.smooth.clone() }
}

/// The drone's turn for the frame (its rows, thousandths) and its place (film, bent, cm): heading along its motion
/// over a frame, nose down as it speeds up (0.3 rad at 2 m a frame).
pub fn drone_pose<O: Ops>(o: &mut O, nw: &NightWorld, tenths: i64) -> ([[i64; 3]; 3], [i64; 3]) {
    let p = { let (a, b) = (kl(o, &[(tenths, None), (-5, None)]), kl(o, &[(tenths, None), (5, None)])); drone_at(o, nw, &[a, tenths, b]) };
    let d: [V; 3] = std::array::from_fn(|c| o.lin(&[(K(p[c][2]), None), (K(p[c][0]), Some(K(-1)))]));
    let sp = one(len3(o, &d[0], &d[1], &d[2]));
    let u = nrm3(o, &d[0], &d[1], &d[2]);
    let yaw = one(azimuth(o, &nw.w.tb, &u[0], &u[2]));
    let pk = one({ let t = o.lin(&[(K(sp), Some(K(1000)))]); o.div(v(&t), K(2000), 0) });
    let pk = one(o.min(K(pk), K(1000)));
    let pitch = one({ let t = o.lin(&[(K(pk), Some(K(-300)))]); o.div(v(&t), K(1000), 0) });
    let ang = |o: &mut O, mrad: i64| -> (i64, i64) { let a = one(mturns(o, &vec![mrad])); let (s, c) = o.sincos(K(a)); (s[0], c[0]) };
    let (sy, cy) = ang(o, yaw);
    let (sp_, cp) = ang(o, pitch);
    let m = |o: &mut O, a: i64, b: i64| -> i64 { one({ let t = o.lin(&[(K(a), Some(K(b)))]); o.div(v(&t), K(999), 0) }) };
    let (sysp, sycp, cysp, cycp) = (m(o, sy, sp_), m(o, sy, cp), m(o, cy, sp_), m(o, cy, cp));
    let msp = one(o.lin(&[(K(sp_), Some(K(-1)))]));
    let msy = one(o.lin(&[(K(sy), Some(K(-1)))]));
    let r = [[cy, sysp, sycp], [0, cp, msp], [msy, cysp, cycp]];
    let at: [i64; 3] = std::array::from_fn(|c| one({ let t = o.lin(&[(K(p[c][1]), None), (K(if c == 0 { 10 * BX } else { 0 }), None)]); o.div(v(&t), K(10), 0) }));
    // (a turn in thousandths from 999-scaled sines: within a part in a thousand)
    (r, at)
}

/// What each pixel sees, for the frame: the world's lists, what moves, the ground (streets past the town's edge in
/// the night's world), the nearest kept (the trees' leaves cut out).
pub fn night_gbuffer<O: Ops>(o: &mut O, nw: &NightWorld, cam: &Cam, scr: &Screen, f: i64, mode: Mode, vis: &(Vec<usize>, Vec<usize>)) -> (GBuf, [V; 3], Vec<(u8, Mesh)>, V) {
    let w = &nw.w;
    let sc = &w.sc;
    let ray = rays(o, cam);
    let mut g = GBuf { iz: vec![0; N_PIX()], mat: vec![-1; N_PIX()], n: [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]], src: vec![0; N_PIX()], tri: vec![0; N_PIX()] };
    let (near, far) = vis;
    let two: Vec<bool> = sc.world.mat.iter().map(|&m| sc.mats[m].kind == kind::TREE || nw.nocull[m]).collect();
    o.stage("geometry");
    let dn = draw(o, &w.tb, cam, &sc.world, &sc.world.corners, &two, near, false, 1);
    let df = draw(o, &w.tb, cam, &sc.world, &sc.world.corners, &two, far, true, 1);
    let mut movers: Vec<(u8, Mesh)> = vec![];
    if mode != Mode::Old {
        // our own train, from outside once we're through the windscreen
        if f >= F_GO && mode == Mode::New && cam.eye_cm[2] > ZS + 160 {
            let t0 = one(o.lin(&[(K(BX), None)]));
            movers.push((4, placed(o, &nw.own, None, [t0, 0, TIP])));
        }
        // the frame's tenths, on the machines
        let tenths = kl(o, &[(f, Some(10))]);
        if f >= F_BTN - 2 {
            let (r, at) = drone_pose(o, nw, tenths);
            movers.push((3, placed(o, &nw.drone, Some(r), at)));
        }
        let bp = blimp_at(o, nw, &[tenths]);
        let at: [i64; 3] = std::array::from_fn(|c| one({ let t = o.lin(&[(K(bp[c][0]), None), (K(if c == 0 { 10 * BX } else { 0 }), None)]); o.div(v(&t), K(10), 0) }));
        movers.push((5, placed(o, &nw.blimp, None, at)));
    }
    let mut drawn = vec![(1u8, dn), (1u8, df)];
    for (src, m) in &movers {
        let all: Vec<usize> = (0..m.tris.len()).collect();
        let two: Vec<bool> = m.mat.iter().map(|&q| nw.nocull[q]).collect();
        // near or far by its place
        let far = mover_far(o, m.corners[0], cam.eye_cm);
        drawn.push((*src, draw(o, &w.tb, cam, m, &m.corners, &two, &all, far, 1)));
    }
    let refs: Vec<(u8, &Drawn, &Mesh)> = drawn.iter().map(|(src, d)| (*src, d, if *src == 1 { &sc.world } else { &movers.iter().find(|(s, _)| s == src).unwrap().1 })).collect();
    ground(o, w, cam, &ray, &mut g, &refs, if mode == Mode::Old { None } else { Some(TOWN0) });
    let ground_iz = g.iz.clone();
    resolve(o, &mut g, &refs, Some(Cut { w, cam, ray: &ray, mirror_h: None, nlq7: &scr.nlq7 }));
    // at half resolution (the party version's world): where a subject (a callout's screen, the board, the airship's
    // screen, our own train) is nearest, the nearest of the rest (the ground's and the other pieces'), for the
    // full-resolution subjects' depth test; elsewhere the buffer's
    let mut wiz = vec![];
    if res() > 1 {
        wiz = g.iz.clone();
        let sp: Vec<usize> = (0..N_PIX()).filter(|&p| g.mat[p] >= 0 && is_subject(w, g.src[p], g.mat[p] as usize)).collect();
        let at: std::collections::HashMap<usize, usize> = sp.iter().enumerate().map(|(i, &p)| (p, i)).collect();
        let mut groups: Vec<Vec<i64>> = sp.iter().map(|&p| if ground_iz[p] > 0 { vec![ground_iz[p]] } else { vec![] }).collect();
        for (src, d, mesh) in &refs { for &(p, z, t, _) in &d.pieces { if let Some(&i) = at.get(&p) { if !is_subject(w, *src, mesh.mat[t]) { groups[i].push(z); } } } }
        let multi: Vec<usize> = (0..sp.len()).filter(|&i| groups[i].len() > 1).collect();
        let refs2: Vec<&[i64]> = multi.iter().map(|&i| groups[i].as_slice()).collect();
        let win = o.nearest(&refs2);
        for (i, &p) in sp.iter().enumerate() { wiz[p] = match groups[i].len() { 0 => 0, 1 => groups[i][0], _ => 0 }; }
        for (j, &i) in multi.iter().enumerate() { wiz[sp[i]] = groups[i][win[j]]; }
    }
    (g, ray, movers, wiz)
}

/// What the party version draws at full resolution: the callouts' screens, the board, the airship's screen, our own
/// train (src 4).
pub fn subject_kind(k: i64) -> bool { k == nkind::DSCREEN || k == nkind::LED || k == nkind::BSCREEN }
pub fn is_subject(w: &World, src: u8, mat: usize) -> bool { src == 4 || subject_kind(w.sc.mats[mat].kind) }



/// The sky's maps for the frame (the dusk's, its sun where it is now): the one ahead and the all-round one, unless
/// the night's violet has it all.
pub fn night_sky_maps<O: Ops>(o: &mut O, nw: &NightWorld, env: &Env, cam: &Cam, e: i64, bk: i64) -> (SkyMap, SkyMap) {
    let tb = &nw.w.tb;
    let blank = |w: i64, n: usize, az0: i64, el0: i64, step: i64| -> SkyMap { let t = || Tab { name: "skymap", data: vec![0; n], w: w as usize }; SkyMap { c: [t(), t(), t()], w, az0, el0, step } };
    if bk >= 100 { return (blank(MAP_W, (MAP_W * MAP_H) as usize, -1400, -50, 5), blank(ROUND_W, (ROUND_W * ROUND_H) as usize, -3150, 0, 50)); }
    let mk = |o: &mut O, name: &str, w: i64, az0: i64, el0: i64, step: i64| -> SkyMap {
        let d = [&tb.t(&format!("{name}_dx")).data[..], &tb.t(&format!("{name}_dy")).data[..], &tb.t(&format!("{name}_dz")).data[..]];
        let k = sky_consts(o, tb, env, &d, &tb.t(&format!("{name}_az")).data);
        let [r, g, b] = sky_model(o, tb, env, &k, cam.eye_cm, e);
        SkyMap { c: [Tab { name: "skymap", data: r, w: w as usize }, Tab { name: "skymap", data: g, w: w as usize }, Tab { name: "skymap", data: b, w: w as usize }], w, az0, el0, step }
    };
    (mk(o, "map", MAP_W, -1400, -50, 5), mk(o, "round", ROUND_W, -3150, 0, 50))
}

/// The night's falling for the frame (sky_night's numbers): the dusk dimmed by 1 - 0.93 n, the twilight's blue and the
/// town's glow added (by the way's height part), the violet by bk, the stadium's glow round its way from the eye.
pub fn sky_night<O: Ops>(o: &mut O, nw: &NightWorld, l: &NightLight, cam: &Cam) -> SkyNight {
    let tb = &nw.w.tb;
    let n = l.night;
    let dim = one({ let t = o.lin(&[(K(n), Some(K(-930)))]); let t = o.div(v(&t), K(1000), 0); o.lin(&[(v(&t), None), (K(1000), None)]) });
    let bk = one(o.div(K(l.black), K(10), 0));
    let hotk = one({ let a = o.div(K(dim), K(10), 0); let b = o.lin(&[(K(bk), Some(K(-1))), (K(100), None)]); let t = o.lin(&[(v(&a), Some(v(&b)))]); o.div(v(&t), K(100), 0) });
    // the tables by the height part (-1000..1000): R exposed (hundredths of R in, times e)
    let tw = tb.t("tw_shape").data.clone();
    let gl = tb.t("glow_shape").data.clone();
    let hz = tb.t("hz_shape").data.clone();
    let ex = |o: &mut O, x: &V| -> V { let t = o.lin(&[(v(x), Some(K(l.e)))]); let t = { let t = o.div(v(&t), K(1000), 0); o.div(v(&t), K(100), 0) }; o.clamp(v(&t), 0, 999) };
    let add: [Tab; 3] = std::array::from_fn(|c| {
        // n ((0.020, 0.030, 0.085) shape + (0.004, 0.006, 0.020)) + n 0.06 glow (1, 0.55, 0.25): hundredths of R
        let a = o.lin(&[(v(&tw), Some(K([500, 750, 2125][c]))), (K([100_000, 150_000, 500_000][c]), None), (v(&gl), Some(K([1500, 825, 375][c])))]);
        let a = o.div(v(&a), K(1000), 0);
        let a = o.lin(&[(v(&a), Some(K(n)))]);
        let a = o.div(v(&a), K(1000), 0);
        Tab { name: "night_add", data: ex(o, &a), w: 0 }
    });
    let viol: [Tab; 3] = std::array::from_fn(|c| {
        // the night's violet: (0.012, 0.004, 0.026) up high, (0.050, 0.018, 0.060) at the horizon, by hz
        let a = o.lin(&[(v(&hz), Some(K([950, 350, 850][c]))), (K([300_000, 100_000, 650_000][c]), None)]);
        let a = o.div(v(&a), K(1000), 0);
        Tab { name: "night_violet", data: ex(o, &a), w: 0 }
    });
    // the stadium's glow: n (0.35 e^-(1-cos)/0.004 + 0.08 e^-(1-cos)/0.03) (1 + 1.5 bk) (0.65, 0.45, 1.0)
    let sgs = tb.t("sg_shape").data.clone();
    let bkk = one({ let t = o.lin(&[(K(l.black), Some(K(1500)))]); let t = o.div(v(&t), K(1000), 0); o.lin(&[(v(&t), None), (K(1000), None)]) });
    let nk = one({ let t = o.lin(&[(K(n), Some(K(bkk)))]); o.div(v(&t), K(1000), 0) });
    let s1 = o.lin(&[(v(&sgs), Some(K(nk)))]);
    let s1 = o.div(v(&s1), K(1000), 0);
    let sg: [Tab; 3] = std::array::from_fn(|c| {
        let a = o.lin(&[(v(&s1), Some(K([1625, 1125, 2500][c])))]);
        let a = o.div(v(&a), K(100), 0);
        Tab { name: "night_sg", data: ex(o, &a), w: 0 }
    });
    // its way: the stadium's middle 45 m up, from the eye (bent)
    let tgt = nw.glow_at;
    let d: [V; 3] = std::array::from_fn(|i| o.lin(&[(K(tgt[i]), None), (K(cam.eye[i]), Some(K(-1)))]));
    let c = nrm3(o, &d[0], &d[1], &d[2]);
    SkyNight { dim, bk, hotk, add, viol, sg, c: [c[0][0], c[1][0], c[2][0]] }
}

/// The screen's numbers for the frame's lens: the nearest of those worked out at the start.
pub fn screen_for(nw: &NightWorld, f2: i64) -> &Screen {
    let scrs = if res() == 1 { &nw.scrs } else { &nw.scrs_h };
    let mut best = 0;
    for (i, (g, _)) in scrs.iter().enumerate() { if (g - f2).abs() < (scrs[best].0 - f2).abs() { best = i; } }
    &scrs[best].1
}

/// One frame of the night, RGB: the picture as the mode says.
pub fn night_render<O: Ops>(o: &mut O, nw: &NightWorld, f: i64, mode: Mode) -> Vec<u8> { night_render_parts(o, nw, f, mode).img }

/// A render's picture (RGB at the resolution drawn) and, for the party version's full-resolution pass, what it
/// leaves: the linear colours after the post (cards and the part past 999) and the lens's light; what each pixel sees,
/// the world's own depth behind the subjects; the light the post laid on (before it went in); each 2x2 block's blur
/// length; where balloons and sparks were laid over; the frame's full-resolution camera, light, sky maps, airship and
/// movers.
pub struct Parts { pub img: Vec<u8>, pub col: [V; 3], pub hot: [V; 3], pub add: [V; 3], pub g: GBuf, pub wiz: V, pub ov: [V; 3], pub mlen: V, pub laid: Vec<bool>,
    pub cam: Cam, pub l: NightLight, pub map: SkyMap, pub round: SkyMap, pub fx: FrameAir, pub movers: Vec<(u8, Mesh)>,
    /// the world's triangles the frame considers (near, far)
    pub vis: (Vec<usize>, Vec<usize>) }

pub fn night_render_parts<O: Ops>(o: &mut O, nw: &NightWorld, f: i64, mode: Mode) -> Parts {
    let w = &nw.w;
    o.stage("camera");
    let mut l = night_light(o, nw, f);
    let cam_full = { let t = kl(o, &[(f, Some(10))]); night_cam(o, nw, t, mode == Mode::Cab) };
    let cam = res_cam(o, &cam_full);
    let scr = screen_for(nw, cam.f2);
    if std::env::var("NIGHT_DEBUG").is_ok() { eprintln!("frame {f} {mode:?}: eye {:?} b {:?} f2 {} night {} black {} e {} beat {}", cam.eye, cam.b, cam.f2, l.night, l.black, l.e, l.beat); }
    let e = l.e;
    let nt = sky_night(o, nw, &l, &cam);
    let bk = nt.bk;
    l.env.night = Some(nt);
    let vis = night_visible(o, nw, &cam, mode);
    let (g, ray, movers, wiz) = night_gbuffer(o, nw, &cam, scr, f, mode, &vis);
    let fx = frame_air(o, nw, f);
    o.stage("skymap");
    let (map, round) = night_sky_maps(o, nw, &l.env, &cam, e, bk);
    o.stage("blur");
    let (fm, fp) = (kl(o, &[(f, Some(10)), (-2, None)]), kl(o, &[(f, Some(10)), (2, None)]));
    let ca = { let c = night_cam(o, nw, fm, mode == Mode::Cab); res_cam(o, &c) };
    let cb = { let c = night_cam(o, nw, fp, mode == Mode::Cab); res_cam(o, &c) };
    let mut mv: Vec<(u8, [i64; 3])> = vec![];
    if movers.iter().any(|(s, _)| *s == 3) {
        let p = drone_at(o, nw, &[fm, fp]);
        let d: [i64; 3] = std::array::from_fn(|c| one(o.lin(&[(K(p[c][0]), None), (K(p[c][1]), Some(K(-1)))])));
        mv.push((3, d));
    }
    let mo = velocities_c(o, w, &cam, &ca, &cb, &g, &mv);
    let (rep, rate) = shading_reps(o, &mo.len);
    o.stage("sky");
    let skyp: Vec<usize> = (0..N_PIX()).filter(|&p| g.mat[p] < 0).collect();
    let (sky_c, sky_h) = night_sky_pixels(o, nw, &l.env, e, &cam, scr, &map, &ray, &g, &skyp, bk);
    o.stage("mirrors");
    let solid: Vec<usize> = (0..N_PIX()).filter(|&p| g.mat[p] >= 0 && rep[p] == p).collect();
    let mirrors = {
        // how many pixels show the paddies and the river: their flags' running sums on the machines
        let fp: V = solid.iter().map(|&p| (g.mat[p] == kind::PADDY) as i64).collect();
        let fw: V = solid.iter().map(|&p| (g.mat[p] == kind::WATER) as i64).collect();
        let count = |o: &mut O, f: &V| -> i64 {
            // (in lists of at most 500,000, so each running sum is a long; the lists' totals added)
            let parts: Vec<&[i64]> = f.chunks(500_000).collect();
            if parts.is_empty() { return 0; }
            let s = o.runsum(&parts);
            let mut at = 0;
            let ends: Vec<(A, Option<A>)> = parts.iter().map(|q| { at += q.len(); (K(s[at - 1]), None) }).collect();
            if ends.len() == 1 { s[at - 1] } else { one(o.lin(&ends)) }
        };
        let (n_paddy, n_water) = (count(o, &fp), count(o, &fw));
        Mirrors {
            paddy: (n_paddy > 50).then(|| Mirror1 { g: mirror_gbuf_of(o, w, &cam, None, LAND, &ray, &vis.0, &scr.nlq7), h: LAND }),
            water: (n_water > 50).then(|| Mirror1 { g: mirror_gbuf_of(o, w, &cam, None, RIVER, &ray, &vis.0, &scr.nlq7), h: RIVER }),
        }
    };
    if std::env::var("NIGHT_KINDS").is_ok() {
        let mut h: std::collections::BTreeMap<i64, usize> = Default::default();
        for &p in &solid { *h.entry(w.sc.mats[g.mat[p] as usize].kind).or_insert(0) += 1; }
        eprintln!("kinds {h:?}");
    }
    o.stage("shade");
    let (c, h) = {
        let mut ns = NightShade { nw, l: &l, cam: &cam, scr, mode, movers: &movers, fx: &fx };
        let kf = kl(o, &[(f, None), (172, None)]);
        shade_solid(o, w, &cam, &l.env, e, kf, None, &g, &ray, &solid, &map, &round, Some(&mirrors), None, if mode == Mode::Old { Some(&nw.dusk_old) } else { None }, &mut ns)
    };
    let mut col: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    let mut hot: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    for (j, &p) in solid.iter().enumerate() { for ch in 0..3 { col[ch][p] = c[ch][j]; hot[ch][p] = h[ch][j]; } }
    for p in 0..N_PIX() { if rep[p] != p { for c in 0..3 { col[c][p] = col[c][rep[p]]; hot[c][p] = hot[c][rep[p]]; } } }
    for (k, &p) in skyp.iter().enumerate() { for c in 0..3 { col[c][p] = sky_c[c][k]; hot[c][p] = sky_h[c][k]; } }
    o.stage("blur");
    motion_blur(o, w, if res() == 1 { &w.lines } else { &nw.lines_h }, &mo, &rate, &mut col, &mut hot);
    o.stage("wires");
    draw_wires(o, w, &cam, &l.env, e, &g, &mut col, &mut hot);
    let (mut ov, mut laid) = ([vec![], vec![], vec![]], vec![]);
    if mode != Mode::Old {
        o.stage("post");
        (ov, laid) = night_post(o, nw, &l, &fx, &cam, &g, &mut col, &mut hot);
    }
    o.stage("lens");
    let mut add: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    let touched = flare(o, w, &cam, &l.env, &g, &mut add);
    bloom(o, w, &col, &hot, &mut add, &touched);
    o.stage("tone");
    let out = finish_v(o, w, &scr.vlev, &col, &hot, &add);
    let mut img = vec![0u8; N_PIX() * 3];
    for c in 0..3 { for p in 0..N_PIX() { img[3 * p + c] = out[c][p] as u8; } }
    Parts { img, col, hot, add, g, wiz, ov, mlen: mo.len, laid, cam: cam_full, l, map, round, fx, movers, vis }
}

/// The sky's pixels: the dusk's (where it still shows) at every fourth pixel each way, its ridges and the sun's core,
/// the night falling over it, made fine (each pixel its own where a ridge's outline crosses its cell); the stars.
pub fn night_sky_pixels<O: Ops>(o: &mut O, nw: &NightWorld, env: &Env, e: i64, cam: &Cam, scr: &Screen, map: &SkyMap, ray: &[V; 3], g: &GBuf, skyp: &[usize], bk: i64) -> ([V; 3], [V; 3]) {
    let tb = &nw.w.tb;
    let mut sky_c: [V; 3] = [vec![], vec![], vec![]];
    let mut sky_h: [V; 3] = [vec![0; skyp.len()], vec![0; skyp.len()], vec![0; skyp.len()]];
    if skyp.is_empty() { return (sky_c, sky_h); }
    let nt = env.night.as_ref().unwrap();
    let (gw, gh) = ((W() / 4) as usize, (H() / 4) as usize);
    let corner = |p: usize| -> (usize, usize) { let (x, y) = ((p % W() as usize) as i64, (p / W() as usize) as i64); (((x - 2).max(0) / 4) as usize, ((y - 2).max(0) / 4) as usize) };
    let mut need = vec![false; gw * gh];
    for &p in skyp { let (i, j) = corner(p); for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] { need[(j + dj).min(gh - 1) * gw + (i + di).min(gw - 1)] = true; } }
    let pts: Vec<usize> = (0..gw * gh).filter(|&q| need[q]).collect();
    let px_of = |q: usize| -> usize { (4 * (q / gw) + 2) * W() as usize + 4 * (q % gw) + 2 };
    let pick = |v: &V| -> V { pts.iter().map(|&q| v[px_of(q)]).collect() };
    let r = [pick(&ray[0]), pick(&ray[1]), pick(&ray[2])];
    let dn = unit_rays(o, tb, &[&r[0], &r[1], &r[2]], &pick(&scr.nlq7));
    let dusk = bk < 100;
    let mut seen = vec![0; gw * gh];
    let sun_on = |o: &mut O| -> Option<(i64, i64)> {
        let s: [i64; 3] = std::array::from_fn(|r| one({ let t = o.lin(&[(K(cam.b[r][0]), Some(K(env.sun[0]))), (K(cam.b[r][1]), Some(K(env.sun[1]))), (K(cam.b[r][2]), Some(K(env.sun[2])))]); o.div(v(&t), K(1000), 0) }));
        if s[2] <= 0 || env.sun_col == [0, 0, 0] { return None; }
        let d2 = kl(o, &[(s[2], Some(2))]);
        let sx = one({ let t = o.lin(&[(K(s[0]), Some(K(cam.f2)))]); o.div(v(&t), K(d2), 0) }) + W() / 2;
        let sy = H() / 2 - one({ let t = o.lin(&[(K(s[1]), Some(K(cam.f2)))]); o.div(v(&t), K(d2), 0) });
        Some((sx, sy))
    };
    let sun_xy = if dusk { sun_on(o) } else { None };
    let near_sun = |p: usize| -> bool { match sun_xy { Some((sx, sy)) => { let (x, y) = ((p as i64) % W(), (p as i64) / W()); (x - sx).abs() < 110 && (y - sy).abs() < 110 } None => false } };
    let mut c: [V; 3];
    let mut h: [V; 3] = [vec![0; pts.len()], vec![0; pts.len()], vec![0; pts.len()]];
    if dusk {
        let az = azimuth(o, tb, &dn[0], &dn[2]);
        c = sky_look(o, tb, map, &az, &dn[1]);
        let lowf = o.lt(v(&dn[1]), K(75));
        let low: Vec<usize> = (0..pts.len()).filter(|&k| lowf[k] == 1).collect();
        let codes = ridges(o, tb, e, &low, &az, &dn[1], &mut c);
        for (q, &k) in low.iter().enumerate() { seen[pts[k]] = codes[q]; }
        let sun: Vec<usize> = (0..pts.len()).filter(|&k| near_sun(px_of(pts[k]))).collect();
        sun_core(o, tb, env, e, &sun, &[&dn[0], &dn[1], &dn[2]], &mut c, &mut h);
    } else {
        c = [vec![0; pts.len()], vec![0; pts.len()], vec![0; pts.len()]];
    }
    night_falls(o, tb, nt, &dn[1], Some(&[&dn[0], &dn[1], &dn[2]]), &mut c, &mut h);
    let (j0, j1) = (pts[0] / gw, pts[pts.len() - 1] / gw);
    let rows = j1 - j0 + 1;
    let fine_of = |o: &mut O, vals: &V| -> V {
        let mut grid = vec![0; gw * rows];
        for (k, &q) in pts.iter().enumerate() { grid[q - j0 * gw] = vals[k]; }
        let fine = upsample(o, &grid, gw, rows);
        skyp.iter().map(|&p| { let (x, y) = (p % W() as usize, p / W() as usize); let yy = y as i64 - 4 * j0 as i64; fine[(yy.clamp(0, 4 * rows as i64 - 1) as usize) * W() as usize + x] }).collect()
    };
    for ch in 0..3 {
        sky_c[ch] = fine_of(o, &c[ch]);
        if h[ch].iter().any(|&x| x != 0) { sky_h[ch] = fine_of(o, &h[ch]); }
    }
    if dusk {
        let (cw, ch_) = (gw - 1, gh - 1);
        let corner_codes: Vec<V> = [(0, 0), (1, 0), (0, 1), (1, 1)].iter().map(|&(di, dj)| (0..cw * ch_).map(|q| seen[(q / cw + dj) * gw + q % cw + di]).collect()).collect();
        let mx = { let a = o.max(v(&corner_codes[0]), v(&corner_codes[1])); let b = o.max(v(&corner_codes[2]), v(&corner_codes[3])); o.max(v(&a), v(&b)) };
        let mn = { let a = o.min(v(&corner_codes[0]), v(&corner_codes[1])); let b = o.min(v(&corner_codes[2]), v(&corner_codes[3])); o.min(v(&a), v(&b)) };
        let edge = o.lt(v(&mn), v(&mx));
        let at: Vec<usize> = (0..skyp.len()).filter(|&k| { let (i, j) = corner(skyp[k]); i < cw && j < ch_ && edge[j * cw + i] == 1 }).collect();
        if !at.is_empty() {
            let pk = |v: &V| -> V { at.iter().map(|&k| v[skyp[k]]).collect() };
            let r = [pk(&ray[0]), pk(&ray[1]), pk(&ray[2])];
            let dn = unit_rays(o, tb, &[&r[0], &r[1], &r[2]], &pk(&scr.nlq7));
            let az = azimuth(o, tb, &dn[0], &dn[2]);
            let mut cc = sky_look(o, tb, map, &az, &dn[1]);
            let lowf = o.lt(v(&dn[1]), K(75));
            let low: Vec<usize> = (0..at.len()).filter(|&j| lowf[j] == 1).collect();
            let _ = ridges(o, tb, e, &low, &az, &dn[1], &mut cc);
            let mut hh: [V; 3] = [vec![0; at.len()], vec![0; at.len()], vec![0; at.len()]];
            let sun: Vec<usize> = (0..at.len()).filter(|&j| near_sun(skyp[at[j]])).collect();
            sun_core(o, tb, env, e, &sun, &[&dn[0], &dn[1], &dn[2]], &mut cc, &mut hh);
            night_falls(o, tb, nt, &dn[1], Some(&[&dn[0], &dn[1], &dn[2]]), &mut cc, &mut hh);
            for (j, &k) in at.iter().enumerate() { for c_ in 0..3 { sky_c[c_][k] = cc[c_][j]; sky_h[c_][k] = hh[c_][j]; } }
        }
    }
    let _ = g;
    (sky_c, sky_h)
}
