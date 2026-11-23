// The train's renderer: one frame, every number through the operations of ops.rs. Included after ops.rs and
// scene.rs; the reference runs it natively, the host on CARDIAC.

/// Each vector's one number.
pub fn one(v: V) -> i64 { assert_eq!(v.len(), 1); v[0] }
/// A sum of products of single numbers, on the machines (lin): for the frame's numbers the host had worked out
/// between operations.
pub fn kl<O: Ops>(o: &mut O, terms: &[(i64, Option<i64>)]) -> i64 {
    let t: Vec<(A, Option<A>)> = terms.iter().map(|&(a, b)| (K(a), b.map(K))).collect();
    one(o.lin(&t))
}

/// The cards of the S-curve's bend at the start: for each metre m 0..2600 along the line, 7000 (1 - cos (pi u)) / 2
/// centimetres, u = (m - 530) / 370 held to 0..1 (the cosine from the swarm's table at u / 2 of a turn).
pub fn bend_table<O: Ops>(o: &mut O) -> Tab {
    let m: V = (0..=2600).collect();
    let m = o.clamp(v(&m), 530, 900);
    let off = o.lin(&[(v(&m), None), (K(-530), None)]);
    let off = o.lin(&[(v(&off), Some(K(1000)))]);
    let ang = o.div(v(&off), K(740), 1);
    let (_, c) = o.sincos(v(&ang));
    let rest = o.lin(&[(v(&c), Some(K(-7000))), (K(999 * 7000), None)]);
    let b = o.div(v(&rest), K(1998), 0);
    Tab { name: "bend", data: b, w: 0 }
}

/// The bend at places along the line (centimetres, 0 or more): the table's entry at the metre and the step to the
/// next times the centimetres over, by hundredths.
pub fn bend<O: Ops>(o: &mut O, t: &Tables, z: A) -> V {
    let zc = o.clamp(z, 0, 259_999);
    let (m, r) = o.divmod(v(&zc), K(100));
    let b0 = o.tab(&t.bend, v(&m));
    let m1 = o.lin(&[(v(&m), None), (K(1), None)]);
    let b1 = o.tab(&t.bend, v(&m1));
    let step = o.lin(&[(v(&b1), None), (v(&b0), Some(K(-1)))]);
    let part = o.pct(v(&step), v(&r));
    o.lin(&[(v(&b0), None), (v(&part), None)])
}

/// The bend at every centimetre from -10 km to 10 km, worked out once: where the curve is (530 m to 900 m) by bend(),
/// before it and after it the ends' values (the host repeats them).
pub fn bend_cm_table<O: Ops>(o: &mut O, t: &Tables) -> Tab {
    let zs: V = (53_000..=90_000).collect();
    let mid = bend(o, t, v(&zs));
    let ends = bend(o, t, v(&vec![0, 259_999]));
    let mut d = vec![ends[0]; 1_053_000];
    d.extend_from_slice(&mid);
    d.resize(2_000_000, ends[1]);
    Tab { name: "bend_cm", data: d, w: 0 }
}

/// The bend at places (centimetres, -10 km to 10 km): the host's look-up.
pub fn bend_at<O: Ops>(o: &mut O, t: &Tables, z: &V) -> V { o.tabo(&t.bend_cm, 1_000_000, v(z)) }

/// The line's heading at places along it, thousandths: the bend 2 m on less 2 m back, over 4 m.
pub fn heading<O: Ops>(o: &mut O, t: &Tables, z: A) -> V {
    let ahead = o.lin(&[(z, None), (K(200), None)]);
    let back = o.lin(&[(z, None), (K(-200), None)]);
    let a = bend(o, t, v(&ahead));
    let b = bend(o, t, v(&back));
    let d = o.lin(&[(v(&a), None), (v(&b), Some(K(-1)))]);
    let d5 = o.lin(&[(v(&d), Some(K(5)))]);
    o.div(v(&d5), K(2), 0)
}

/// The cab's camera at a time (tenths of a frame): the eye (millimetres, and centimetres), and its turn (rows across,
/// up and ahead, thousandths), as the prototype's Cam: on the line at 7 m a frame, a little sway, looking along the
/// line 25 m on and a little down, leaning into the curve.
#[derive(Clone, Copy, Debug)]
/// `bt` turns places into the camera's space for the screen (the rows across and up times f / 950, so the printed
/// tables' projection at 950 gives the camera's focal length f); `f2` is twice f in pixels (1900 for the ride).
pub struct Cam { pub eye: [i64; 3], pub eye_cm: [i64; 3], pub b: [[i64; 3]; 3], pub bt: [[i64; 3]; 3], pub f2: i64, pub z_cm: i64 }

pub fn camera<O: Ops>(o: &mut O, t: &Tables, tenths: i64) -> Cam {
    let tt = vec![tenths];
    // departing (before the ride's time 0): from standing at -252 m a steady push to the ride's 7 m a frame at 0, the
    // place -25,200 + 7 w^2 / 144 cm (w tenths since the push began, 0..720); the sway with the speed (full from
    // w = 700 on)
    let w = if tenths < 0 { let t = kl(o, &[(tenths, None), (720, None)]); Some(one(o.clamp(K(t), 0, 720))) } else { None };
    let z = match w {
        None => o.lin(&[(v(&tt), Some(K(70)))]),
        Some(w) => { let w7 = kl(o, &[(w, Some(7))]); let t7 = o.lin(&[(K(w), Some(K(w7)))]); let q = o.div(v(&t7), K(144), 0); o.sum(&[(v(&q), 1)], -25_200) }
    };
    // the sway, the bob and the lean's wobble: sines of the time (each angle's step a tenth of a frame, millionths of a turn)
    let (fr, te) = o.divmod(v(&tt), K(10));
    let ang = |o: &mut O, rate: i64| -> V { let a = o.lin(&[(K(10 * rate), Some(v(&fr))), (K(rate), Some(v(&te)))]); o.turns(v(&a)) };
    let a1 = ang(o, 11_141);
    let a2 = ang(o, 36_606);
    let a3 = ang(o, 30_239);
    let a4 = ang(o, 20_690);
    let (s1, _) = o.sincos(v(&a1));
    let (s2, _) = o.sincos(v(&a2));
    let (s3, _) = o.sincos(v(&a3));
    let (s4, _) = o.sincos(v(&a4));
    let sway = o.lin(&[(v(&s1), Some(K(35))), (v(&s2), Some(K(12)))]);
    let sway = o.div(v(&sway), K(999), 0);
    let sway = match w { Some(w) if w < 700 => { let s = o.mul(v(&sway), K(w), 0); o.div(v(&s), K(700), 0) }, _ => sway };
    let bob = o.lin(&[(v(&s3), Some(K(12)))]);
    let bob = o.div(v(&bob), K(999), 0);
    let bz = bend(o, t, v(&z));
    let ex = o.lin(&[(v(&bz), Some(K(10))), (v(&sway), None)]);
    let ey = o.lin(&[(v(&bob), None), (K(10 * EYE_Y), None)]);
    let ez = o.lin(&[(v(&z), Some(K(10)))]);
    // looking along the line 25 m on: (h, -0.05, 1) made a unit
    let z25 = o.lin(&[(v(&z), None), (K(2500), None)]);
    let h = heading(o, t, v(&z25));
    let len2 = o.lin(&[(v(&h), Some(v(&h))), (K(1_002_500), None)]);
    let len = o.sqrt(v(&len2));
    let fx = o.lin(&[(v(&h), Some(K(1000)))]);
    let fx = o.div(v(&fx), v(&len), 0);
    let fy = o.div(K(-50_000), v(&len), 0);
    let fz = o.div(K(1_000_000), v(&len), 0);
    let rl2 = o.lin(&[(v(&fz), Some(v(&fz))), (v(&fx), Some(v(&fx)))]);
    let rl = o.sqrt(v(&rl2));
    let rx = o.lin(&[(v(&fz), Some(K(999)))]);
    let rx = o.div(v(&rx), v(&rl), 0);
    let rz = o.lin(&[(v(&fx), Some(K(-999)))]);
    let rz = o.div(v(&rz), v(&rl), 0);
    let ux = o.lin(&[(v(&fy), Some(v(&rz)))]);
    let ux = o.div(v(&ux), K(1000), 0);
    let uy = o.lin(&[(v(&fz), Some(v(&rx))), (v(&fx), Some(v(&rz))), ]);
    let fxrz = o.lin(&[(v(&fx), Some(v(&rz)))]);
    let fzrx = o.lin(&[(v(&fz), Some(v(&rx)))]);
    let uy = { let _ = uy; let d = o.lin(&[(v(&fzrx), None), (v(&fxrz), Some(K(-1)))]); o.div(v(&d), K(1000), 0) };
    let uz = o.lin(&[(v(&fy), Some(v(&rx)))]);
    let uz = o.lin(&[(v(&uz), Some(K(-1)))]);
    let uz = o.div(v(&uz), K(1000), 0);
    // the lean: into the curve (the heading's change over 20 m) and a little wobble, millionths of a turn
    let zp = o.lin(&[(v(&z), None), (K(1000), None)]);
    let zm = o.lin(&[(v(&z), None), (K(-1000), None)]);
    let hp = heading(o, t, v(&zp));
    let hm = heading(o, t, v(&zm));
    let dh = o.lin(&[(v(&hp), None), (v(&hm), Some(K(-1)))]);
    let lean = o.lin(&[(v(&dh), Some(K(-143_240)))]);
    let lean = o.div(v(&lean), K(1000), 0);
    let wob = o.lin(&[(v(&s4), Some(K(318)))]);
    let wob = o.div(v(&wob), K(999), 0);
    let roll = o.lin(&[(v(&lean), None), (v(&wob), None)]);
    let (sr, cr) = o.sincos(v(&roll));
    let r = [rx, vec![0], rz];
    let u = [ux, uy, uz];
    let mut b = [[0i64; 3]; 3];
    for i in 0..3 {
        let x = o.lin(&[(v(&cr), Some(v(&r[i]))), (v(&sr), Some(v(&u[i])))]);
        b[0][i] = one(o.div(v(&x), K(999), 0));
        let msr = o.lin(&[(v(&sr), Some(K(-1)))]);
        let y = o.lin(&[(v(&msr), Some(v(&r[i]))), (v(&cr), Some(v(&u[i])))]);
        b[1][i] = one(o.div(v(&y), K(999), 0));
    }
    b[2] = [one(fx), one(fy), one(fz)];
    let eye = [one(ex), one(ey), one(ez)];
    let ecm: [i64; 3] = std::array::from_fn(|i| { let q = o.div(K(eye[i]), K(10), 0); one(q) });
    Cam { eye, eye_cm: ecm, b, bt: b, f2: 1900, z_cm: one(z) }
}

// ---- the triangles ----------------------------------------------------------------------------------------------

/// What a frame draws of a list of triangles: their corners in the camera's space, worked out once each.
pub struct Drawn {
    /// pieces: pixel, depth (K / z), triangle (index into the list), normal (hundredths; the face's when flat)
    pub pieces: Vec<(usize, i64, usize, [i64; 3])>,
    pub tris: usize,
    pub clipped: usize,
}

/// A vertex in the camera's space: place (millimetres, or centimetres far off), normal.
#[derive(Clone, Copy, Debug)]
struct Vx { c: [i64; 3], n: [i64; 3] }

const PLANES: usize = 5;

/// The five planes' values at camera-space places: the near plane and the guard band's four sides (each >= 0
/// inside). `near` is the near plane's distance in the places' unit.
pub fn planes<O: Ops>(o: &mut O, c: &[[i64; 3]], near: i64) -> Vec<[i64; PLANES]> {
    let col = |i: usize| -> V { c.iter().map(|p| p[i]).collect() };
    let (x, y, z) = (col(0), col(1), col(2));
    let pn = o.lin(&[(v(&z), None), (K(-near), None)]);
    let pr = o.lin(&[(v(&z), Some(K(96))), (v(&x), Some(K(-95)))]);
    let pl = o.lin(&[(v(&z), Some(K(96))), (v(&x), Some(K(95)))]);
    let pt = o.lin(&[(v(&z), Some(K(48))), (v(&y), Some(K(-95)))]);
    let pb = o.lin(&[(v(&z), Some(K(48))), (v(&y), Some(K(95)))]);
    (0..c.len()).map(|i| [pn[i], pr[i], pl[i], pt[i], pb[i]]).collect()
}

/// The planes' values over this, for the place where an edge crosses one, so the two fit a division.
const PLANE_SCALE: [i64; PLANES] = [100, 10_000, 10_000, 10_000, 10_000];

/// Projection of camera-space places to the screen: hundredths of a pixel from the middle (y down; of a half-size
/// screen's pixel when s is 2) and the depth K / z. Places in millimetres (centimetres far). By logarithms: ln |x| +
/// ln 1000 less ln (z / 10) less ln (1000 s / 9500) (the printed tables), e back by the sign's table; K over z / 10
/// from the printed table of reciprocals.
pub fn project<O: Ops>(o: &mut O, tb: &Tb, c: &[[i64; 3]], far: bool, s: i64) -> Vec<(i64, i64, i64)> {
    if c.is_empty() { return vec![]; }
    let col = |i: usize| -> V { c.iter().map(|p| p[i]).collect() };
    let (x, y, z) = (col(0), col(1), col(2));
    let z10 = o.div(v(&z), K(10), 0);
    let lz = o.tab(tb.t(match s * res() { 1 => "lnz_f", 2 => "lnz_f2", _ => "lnz_f4" }), v(&z10));
    let part = |o: &mut O, u: &V, flip: bool| -> V {
        let lu = o.tabo(tb.t("lnd4"), 999_999, v(u));
        let k = o.ldiff(&lu, &lz, 0, 139);
        let neg = o.tabo(tb.t("negd"), 999_999, v(u));
        let mut r = vec![0; u.len()];
        for (name, want) in [("exp4", flip as i64), ("exp4n", !flip as i64)] {
            let at: Vec<usize> = (0..u.len()).filter(|&j| neg[j] == want).collect();
            let kk: V = at.iter().map(|&j| k[j]).collect();
            let e = o.tab(tb.t(name), v(&kk));
            for (q, &j) in at.iter().enumerate() { r[j] = e[q]; }
        }
        r
    };
    let sx = part(o, &x, false);
    let sy = part(o, &y, true);
    let iz = o.tab(tb.t(if far { "kz10_z10" } else { "kz_z10" }), v(&z10));
    (0..c.len()).map(|i| (sx[i], sy[i], iz[i])).collect()
}

/// Draw a list of triangles: the ones that face the camera (or all, for the cut-out foliage), clipped to the near
/// plane and the guard band, their corners on the screen, their edges and spans; every piece on a pixel, in the
/// list's order. `pick` the triangles to consider; `shift` a place to add to every corner (centimetres) as the
/// numbers of the sums; `far` whether the corners are counted in centimetres (else millimetres). `s` 2 for a screen of
/// half the size each way (a mirror's), its pixels put at the full screen's even rows and columns.
pub fn draw<O: Ops>(o: &mut O, tb: &Tb, cam: &Cam, mesh: &Mesh, corners: &[[i64; 3]], two_sided: &[bool], pick: &[usize], far: bool, s: i64) -> Drawn {
    let (hw, hh) = (W() / (2 * s), H() / (2 * s));
    let mut d = Drawn { pieces: vec![], tris: 0, clipped: 0 };
    if pick.is_empty() { return d; }
    // which face the camera: the face normal's dot with the eye (centimetres) against the plane's distance, less a margin
    let fx: V = pick.iter().map(|&k| mesh.fnorm[k][0]).collect();
    let fy: V = pick.iter().map(|&k| mesh.fnorm[k][1]).collect();
    let fz: V = pick.iter().map(|&k| mesh.fnorm[k][2]).collect();
    let fd: V = pick.iter().map(|&k| -mesh.fd[k] + 500).collect();
    // the normal's parts (hundredths) times the eye's: tables for the frame
    let parts: V = (-100..=100).collect();
    let t: Vec<V> = (0..3).map(|i| { let tab = Tab { name: "eye_part", data: o.lin(&[(v(&parts), Some(K(cam.eye_cm[i])))]), w: 0 }; o.tabo(&tab, 100, v(&[&fx, &fy, &fz][i])) }).collect();
    let dot = o.lin(&[(v(&t[0]), None), (v(&t[1]), None), (v(&t[2]), None), (v(&fd), None)]);
    let back = o.lt(v(&dot), K(0));
    let front: Vec<usize> = pick.iter().zip(&back).filter(|&(&k, &b)| b == 0 || two_sided[k]).map(|(&k, _)| k).collect();
    if front.is_empty() { return d; }
    // their corners in the camera's space: B (10 p - eye) in thousandths of a millimetre, then millimetres (or
    // p - eye in centimetres far off)
    let mut used: Vec<usize> = front.iter().flat_map(|&k| mesh.tris[k]).collect();
    used.sort();
    used.dedup();
    let unit = if far { 1 } else { 10 };
    let e = if far { cam.eye_cm } else { cam.eye };
    let dvec: Vec<V> = (0..3).map(|i| { let p: V = used.iter().map(|&u| corners[u][i]).collect(); o.lin(&[(v(&p), Some(K(unit))), (K(-e[i]), None)]) }).collect();
    let mut cc: Vec<[i64; 3]> = vec![[0; 3]; used.len()];
    let ld: [LogV; 3] = std::array::from_fn(|i| logv(o, tb, &dvec[i]));
    for r in 0..3 {
        let s = logdot(o, tb, [&ld[0], &ld[1], &ld[2]], cam.bt[r]);
        for (k, val) in s.iter().enumerate() { cc[k][r] = *val; }
    }
    let at: std::collections::HashMap<usize, usize> = used.iter().enumerate().map(|(i, &u)| (u, i)).collect();
    // the planes at each corner; the triangles inside all, outside one, or crossing
    let near = if far { NEAR_MM / 10 } else { NEAR_MM };
    let pv = planes(o, &cc, near);
    let flat: V = pv.iter().flatten().copied().collect();
    let outside = o.lt(v(&flat), K(0));
    let out_of = |i: usize, p: usize| outside[i * PLANES + p] == 1;
    let mut whole = vec![];
    let mut cross = vec![];
    for &k in &front {
        let c = mesh.tris[k].map(|u| at[&u]);
        if (0..PLANES).any(|p| c.iter().all(|&i| out_of(i, p))) { continue; }
        if c.iter().all(|&i| (0..PLANES).all(|p| !out_of(i, p))) { whole.push(k); } else { cross.push(k); }
    }
    // the crossing ones clipped, one plane after another, into fans; a vertex made where an edge crosses a plane is
    // made once, from its inside end and its outside end
    let normal = |k: usize, j: usize| -> [i64; 3] { if mesh.smooth[k] { [mesh.vn[k][3 * j], mesh.vn[k][3 * j + 1], mesh.vn[k][3 * j + 2]] } else { mesh.fnorm[k] } };
    // polygons: vertex keys; vertices: corners by index, made ones after
    let mut verts: Vec<Vx> = cc.iter().map(|&c| Vx { c, n: [0; 3] }).collect();
    let mut vplanes: Vec<[i64; PLANES]> = pv.clone();
    let mut polys: Vec<(usize, Vec<usize>, Vec<[i64; 3]>)> = vec![];
    for &k in &cross {
        let c = mesh.tris[k].map(|u| at[&u]);
        polys.push((k, c.to_vec(), (0..3).map(|j| normal(k, j)).collect()));
    }
    let mut made: std::collections::HashMap<(usize, usize, usize, [i64; 3], [i64; 3]), usize> = Default::default();
    for p in 0..PLANES {
        // the edges that cross this plane, each once
        let mut need: Vec<(usize, usize, [i64; 3], [i64; 3])> = vec![];
        for (_, poly, ns) in &polys {
            let n = poly.len();
            for j in 0..n {
                let (a, b) = (poly[j], poly[(j + 1) % n]);
                let (ia, ib) = (vplanes[a][p] >= 0, vplanes[b][p] >= 0);
                if ia != ib {
                    let (pi, qo, npi, nqo) = if ia { (a, b, ns[j], ns[(j + 1) % n]) } else { (b, a, ns[(j + 1) % n], ns[j]) };
                    if !made.contains_key(&(p, pi, qo, npi, nqo)) && !need.contains(&(pi, qo, npi, nqo)) { need.push((pi, qo, npi, nqo)); }
                }
            }
        }
        if !need.is_empty() {
            // s = dp / (dp - dq), thousandths, the plane's values scaled down to fit
            let dp: V = need.iter().map(|e| vplanes[e.0][p]).collect();
            let dq: V = need.iter().map(|e| vplanes[e.1][p]).collect();
            let sp = o.div(v(&dp), K(PLANE_SCALE[p]), 0);
            let sq = o.div(v(&dq), K(PLANE_SCALE[p]), 0);
            let den = o.lin(&[(v(&sp), None), (v(&sq), Some(K(-1)))]);
            let s = o.div(v(&sp), v(&den), 1);
            // the new place: p + fl((q - p) s, 1000); the normal the same way
            let mut newc: Vec<[i64; 3]> = vec![[0; 3]; need.len()];
            let mut newn: Vec<[i64; 3]> = vec![[0; 3]; need.len()];
            for i in 0..3 {
                let a: V = need.iter().map(|e| verts[e.0].c[i]).collect();
                let b: V = need.iter().map(|e| verts[e.1].c[i]).collect();
                let dd = o.lin(&[(v(&b), None), (v(&a), Some(K(-1)))]);
                let m = o.lin(&[(v(&dd), Some(v(&s)))]);
                let m = o.div(v(&m), K(1000), 0);
                let r = o.lin(&[(v(&a), None), (v(&m), None)]);
                let na: V = need.iter().map(|e| e.2[i]).collect();
                let nb: V = need.iter().map(|e| e.3[i]).collect();
                let nd = o.sum(&[(v(&nb), 1), (v(&na), -1)], 0);
                let nm = o.mul(v(&nd), v(&s), 3);
                let nr = o.sum(&[(v(&na), 1), (v(&nm), 1)], 0);
                for k in 0..need.len() { newc[k][i] = r[k]; newn[k][i] = nr[k]; }
            }
            let np = planes(o, &newc, near);
            for (k, e) in need.iter().enumerate() {
                verts.push(Vx { c: newc[k], n: newn[k] });
                vplanes.push(np[k]);
                made.insert((p, e.0, e.1, e.2, e.3), verts.len() - 1);
            }
        }
        // each polygon cut at the plane
        for (_, poly, ns) in polys.iter_mut() {
            let n = poly.len();
            let (mut np, mut nn) = (vec![], vec![]);
            for j in 0..n {
                let (a, b) = (poly[j], poly[(j + 1) % n]);
                let (ia, ib) = (vplanes[a][p] >= 0, vplanes[b][p] >= 0);
                if ia { np.push(a); nn.push(ns[j]); }
                if ia != ib {
                    let (pi, qo, npi, nqo) = if ia { (a, b, ns[j], ns[(j + 1) % n]) } else { (b, a, ns[(j + 1) % n], ns[j]) };
                    let m = made[&(p, pi, qo, npi, nqo)];
                    np.push(m);
                    nn.push(verts[m].n);
                }
            }
            *poly = np;
            *ns = nn;
        }
    }
    // the triangles to draw: whole ones, and the clipped polygons' fans
    struct T { k: usize, v: [usize; 3], n: [[i64; 3]; 3] }
    let mut tris: Vec<T> = vec![];
    let mut ci = 0;
    // which are whole, by triangle (a scan of `whole` for each one was most of a night frame's host time)
    let mut is_whole = vec![false; mesh.tris.len()];
    for &k in &whole { is_whole[k] = true; }
    for &k in &front {
        if ci < cross.len() && cross[ci] == k {
            let (_, poly, ns) = &polys[ci];
            ci += 1;
            for j in 1..poly.len().saturating_sub(1) { tris.push(T { k, v: [poly[0], poly[j], poly[j + 1]], n: [ns[0], ns[j], ns[j + 1]] }); }
            d.clipped += 1;
        } else if is_whole[k] {
            let c = mesh.tris[k].map(|u| at[&u]);
            tris.push(T { k, v: c, n: [normal(k, 0), normal(k, 1), normal(k, 2)] });
        }
    }
    d.tris = tris.len();
    // the vertices in use, on the screen
    let mut usedv: Vec<usize> = tris.iter().flat_map(|t| t.v).collect();
    usedv.sort();
    usedv.dedup();
    let pr = project(o, tb, &usedv.iter().map(|&i| verts[i].c).collect::<Vec<_>>(), far, s);
    let scr: std::collections::HashMap<usize, (i64, i64, i64)> = usedv.iter().zip(pr).map(|(&i, p)| (i, p)).collect();
    // each edge once (its two vertices and their normals), its upper end first
    type EKey = (usize, usize, [i64; 3], [i64; 3]);
    let mut eindex: std::collections::HashMap<EKey, usize> = Default::default();
    let mut edges: Vec<(EKey, bool)> = vec![];
    let key = |a: usize, b: usize, na: [i64; 3], nb: [i64; 3]| -> EKey { if (a, na) <= (b, nb) { (a, b, na, nb) } else { (b, a, nb, na) } };
    for t in &tris {
        for e in 0..3 {
            let k = key(t.v[e], t.v[(e + 1) % 3], t.n[e], t.n[(e + 1) % 3]);
            eindex.entry(k).or_insert_with(|| { edges.push((k, mesh.smooth[t.k])); edges.len() - 1 });
        }
    }
    // the edges' crossings of the rows, and the values down them
    let ends: Vec<([i64; 2], [i64; 2])> = edges.iter().map(|((a, b, _, _), _)| {
        let (pa, pb) = (scr[a], scr[b]);
        if pb.1 < pa.1 { ([pb.0, pb.1], [pa.0, pa.1]) } else { ([pa.0, pa.1], [pb.0, pb.1]) }
    }).collect();
    let crossings = edge_rows(o, tb, &ends, (-hh, hh));
    // the values down each edge: from the top's by the slope (the change over the rows and one, by logarithms)
    let (mut ta, mut tb_, mut tn, mut tc) = (vec![], vec![], vec![], vec![]);
    for (i, ((a, b, na, nb), smooth)) in edges.iter().enumerate() {
        let (pa, pb) = (scr[a], scr[b]);
        let up = pb.1 < pa.1;
        let (top, bot, nt, nbm) = if up { (pb, pa, nb, na) } else { (pa, pb, na, nb) };
        let rows = crossings[i].0;
        if crossings[i].1.is_empty() { continue; }
        ta.push(top.2); tb_.push(bot.2); tn.push(rows); tc.push(false);
        if *smooth { for c in 0..3 { ta.push(nt[c]); tb_.push(nbm[c]); tn.push(rows); tc.push(true); } }
    }
    let n1: V = tn.iter().map(|&r| r + 1).collect();
    let sl = slopes(o, tb, &ta, &tb_, &n1);
    let wl = walk_from_c(o, &ta, &sl, &tn, &tc);
    let mut wat = 0;
    // per edge: (row, pixel, depth, normal)
    let mut on_rows: Vec<Vec<(i64, i64, i64, [i64; 3])>> = vec![vec![]; edges.len()];
    for (i, (_, smooth)) in edges.iter().enumerate() {
        if crossings[i].1.is_empty() { continue; }
        let rows = crossings[i].0 as usize;
        let ch = if *smooth { 4 } else { 1 };
        for &(row, px, j) in &crossings[i].1 {
            let val = |c: usize| wl[wat + c * rows + j as usize];
            on_rows[i].push((row, px, val(0), if *smooth { [val(1), val(2), val(3)] } else { [0; 3] }));
        }
        wat += ch * rows;
    }
    // each triangle's spans: from its two edges' pixels on each row; the values across by each span's slope (the
    // change over its pixels, thousandths a pixel, by logarithms), from the left edge's; pieces on the screen
    struct Span { k: usize, row: i64, from: i64, n: i64, smooth: bool, fnorm: [i64; 3] }
    let mut spans: Vec<Span> = vec![];
    // each walk's ends (its length pb - pa on the machines below) and where each span's first walk is
    let (mut va, mut vb, mut ends, mut cards, mut first) = (vec![], vec![], vec![], vec![], vec![]);
    for t in &tris {
        let mut by_row: std::collections::BTreeMap<i64, Vec<(i64, i64, [i64; 3])>> = Default::default();
        for e in 0..3 {
            let k = key(t.v[e], t.v[(e + 1) % 3], t.n[e], t.n[(e + 1) % 3]);
            for &(row, px, z, n) in &on_rows[eindex[&k]] { by_row.entry(row).or_default().push((px, z, n)); }
        }
        let smooth = mesh.smooth[t.k];
        for (row, mut c) in by_row {
            assert_eq!(c.len(), 2, "a triangle's row met {} edges", c.len());
            if c[1].0 < c[0].0 { c.swap(0, 1); }
            let ((pa, za, na), (pb, zb, nb)) = (c[0], c[1]);
            if pa == pb || pb <= -hw || pa >= hw { continue; }
            first.push(ends.len());
            spans.push(Span { k: t.k, row, from: pa, n: 0, smooth, fnorm: mesh.fnorm[t.k] });
            va.push(za); vb.push(zb); ends.push((pa, pb)); cards.push(false);
            if smooth { for c in 0..3 { va.push(na[c]); vb.push(nb[c]); ends.push((pa, pb)); cards.push(true); } }
        }
    }
    let nn = { let (pa, pb): (V, V) = (ends.iter().map(|e| e.0).collect(), ends.iter().map(|e| e.1).collect()); o.lin(&[(v(&pb), None), (v(&pa), Some(K(-1)))]) };
    for (sp, &i) in spans.iter_mut().zip(&first) { sp.n = nn[i]; }
    let a = slopes(o, tb, &va, &vb, &nn);
    let sw = walk_from_c(o, &va, &a, &nn, &cards);
    let mut at_ = 0;
    for sp in &spans {
        let m = sp.n as usize;
        for kx in 0..m {
            let x = sp.from + kx as i64;
            if x < -hw || x >= hw { continue; }
            let z = sw[at_ + kx];
            let n = if sp.smooth { [sw[at_ + m + kx], sw[at_ + 2 * m + kx], sw[at_ + 3 * m + kx]] } else { sp.fnorm };
            d.pieces.push((((sp.row + hh) * s * W() + (x + hw) * s) as usize, z, sp.k, n));
        }
        at_ += if sp.smooth { 4 } else { 1 } * m;
    }
    assert_eq!(at_, sw.len());
    d
}

/// Slopes: (b - a) 1000 / n for each (longs a, b; n places, 1 or more), by logarithms (ln |b - a| + ln 1000 less ln n,
/// ten-thousandths; e back by the table of the change's sign: three cards).
pub fn slopes<O: Ops>(o: &mut O, tb: &Tb, a: &V, b: &V, n: &V) -> V {
    let d = o.sum(&[(v(b), 1), (v(a), -1)], 0);
    let l = o.tabo(tb.t("lnd4"), 999_999, v(&d));
    let ln = o.tab(tb.t("lniz4"), v(n));
    let k = o.ldiff(&l, &ln, 0, 207);
    let neg = o.tabo(tb.t("negd"), 999_999, v(&d));
    let mut out = vec![0; d.len()];
    for (name, want) in [("exps4", 0), ("exps4n", 1)] {
        let at: Vec<usize> = (0..d.len()).filter(|&j| neg[j] == want).collect();
        let kk: V = at.iter().map(|&j| k[j]).collect();
        let e = o.tab(tb.t(name), v(&kk));
        for (q, &j) in at.iter().enumerate() { out[j] = e[q]; }
    }
    out
}

/// Where edges (each from its upper end to its lower: hundredths of a pixel from the middle) cross the rows' middles:
/// for each edge, how many rows it meets and, for those in the range, (row, pixel, which of its rows). The first row
/// and the place past its middle from (y + 49) over 100; the crossing on the first row from the top's x by
/// logarithms (dx times the part of a row over dy), then down the rows by the slope (dx 1000 / dy, thousandths of a
/// pixel a row) by walka, starting from the first crossing's thousandths; the pixel fl(x + 49, 100).
pub fn edge_rows<O: Ops>(o: &mut O, tb: &Tb, ends: &[([i64; 2], [i64; 2])], range: (i64, i64)) -> Vec<(i64, Vec<(i64, i64, i64)>)> {
    let n = ends.len();
    if n == 0 { return vec![]; }
    let g = |f: &dyn Fn(&([i64; 2], [i64; 2])) -> i64| -> V { ends.iter().map(f).collect() };
    let (ax, ay, bx, by) = (g(&|e| e.0[0]), g(&|e| e.0[1]), g(&|e| e.1[0]), g(&|e| e.1[1]));
    let ya = o.sum(&[(v(&ay), 1)], 49);
    let (fa, ra) = o.divmod(v(&ya), K(100));
    let yb = o.sum(&[(v(&by), 1)], 49);
    let (fb, _) = o.divmod(v(&yb), K(100));
    let rows = o.sum(&[(v(&fb), 1), (v(&fa), -1)], 0);
    let mut out: Vec<(i64, Vec<(i64, i64, i64)>)> = (0..n).map(|i| (rows[i], vec![])).collect();
    let live: Vec<usize> = (0..n).filter(|&i| rows[i] > 0).collect();
    if live.is_empty() { return out; }
    let l = |v_: &V| -> V { live.iter().map(|&i| v_[i]).collect() };
    let (ax, ay, bx, by, fa, ra) = (l(&ax), l(&ay), l(&bx), l(&by), l(&fa), l(&ra));
    let dx = o.sum(&[(v(&bx), 1), (v(&ax), -1)], 0);
    let dy = o.sum(&[(v(&by), 1), (v(&ay), -1)], 0);
    let neg = o.tabo(tb.t("negd"), 999_999, v(&dx));
    let signed = |o: &mut O, k: &V, wide: bool| -> V {
        let mut r = vec![0; k.len()];
        for (name, want) in if wide { [("exps4", 0), ("exps4n", 1)] } else { [("exp4", 0), ("exp4n", 1)] } {
            let at: Vec<usize> = (0..k.len()).filter(|&j| neg[j] == want).collect();
            let kk: V = at.iter().map(|&j| k[j]).collect();
            let e = o.tab(tb.t(name), v(&kk));
            for (q, &j) in at.iter().enumerate() { r[j] = e[q]; }
        }
        r
    };
    // the slope: ln |dx| + ln 1000 less ln dy
    let lx = o.tabo(tb.t("lnd4"), 999_999, v(&dx));
    let ly = o.tab(tb.t("lniz4"), v(&dy));
    let k1 = o.ldiff(&lx, &ly, 0, 207);
    let slope = signed(o, &k1, true);
    // the first crossing: dx (99 - r) / dy past the top's x: ln |dx| + ln (99 - r) less ln dy (the table less ln 1000)
    let lo = o.tab(tb.t("ln_off4"), v(&ra));
    let lxo = o.sum(&[(v(&lx), 1), (v(&lo), 1)], 0);
    let k2 = o.ldiff(&lxo, &ly, 0, 139);
    let t = signed(o, &k2, false);
    let x0 = o.sum(&[(v(&ax), 1), (v(&t), 1)], 49);
    let (p0, fh) = o.divmod(v(&x0), K(100));
    let f0 = o.tab(tb.t("x10"), v(&fh));
    let walks: Vec<(i64, i64, i64, i64)> = (0..live.len()).map(|j| (rows[live[j]], p0[j], slope[j], f0[j])).collect();
    let px = o.walka(&walks);
    let starts: Vec<usize> = live.iter().scan(0usize, |a, &i| { let s = *a; *a += rows[i] as usize; Some(s) }).collect();
    for (j, &i) in live.iter().enumerate() {
        for r in 0..rows[i] {
            let row = fa[j] + r;
            if (range.0..range.1).contains(&row) { out[i].1.push((row, px[starts[j] + r as usize], r)); }
        }
    }
    out
}

/// Walks from values by slopes (thousandths a place), the first place a step on: the start and the slope's
/// thousands added on CARDIAC, its rest the walk's first thousandths, so the k-th place has va + fl(a (k + 1), 1000).
pub fn walk_from<O: Ops>(o: &mut O, va: &V, a: &V, n: &V) -> V { walk_from_c(o, va, a, n, &vec![false; va.len()]) }

/// The same, the walks marked as over cards (a normal's parts) by walkc where they are under 1000 places.
pub fn walk_from_c<O: Ops>(o: &mut O, va: &V, a: &V, n: &V, cards: &[bool]) -> V {
    let whole: V = a.iter().map(|&x| fl(x, 1000)).collect();
    let v1 = o.sum(&[(v(va), 1), (v(&whole), 1)], 0);
    let small: Vec<bool> = (0..va.len()).map(|j| cards[j] && n[j] <= 999).collect();
    let pick = |want: bool| -> Vec<(i64, i64, i64, i64)> { (0..va.len()).filter(|&j| small[j] == want).map(|j| (n[j], v1[j], a[j], md(a[j], 1000))).collect() };
    let (wl, wc) = (pick(false), pick(true));
    let (ol, oc) = (o.walka(&wl), if wc.is_empty() { vec![] } else { o.walkc(&wc) });
    let (mut il, mut ic) = (0, 0);
    let mut out = Vec::with_capacity(ol.len() + oc.len());
    for j in 0..va.len() {
        let m = n[j] as usize;
        if small[j] { out.extend_from_slice(&oc[ic..ic + m]); ic += m; } else { out.extend_from_slice(&ol[il..il + m]); il += m; }
    }
    out
}

/// Longs ready for logdot: ln |q| + ln 1000 (ten-thousandths; the printed table) and their signs.
pub struct LogV { pub l: V, pub neg: V }

pub fn logv<O: Ops>(o: &mut O, tb: &Tb, q: &V) -> LogV {
    LogV { l: o.tabo(tb.t("lnd4"), 999_999, v(q)), neg: o.tabo(tb.t("negd"), 999_999, v(q)) }
}

/// q.a / 1000 for three longs and constants a (thousandths): each term by logarithms, ln |q| + ln 1000 less
/// 2 ln 1000 - ln |a|, e back by the term's sign (the printed tables); the terms added.
pub fn logdot<O: Ops>(o: &mut O, tb: &Tb, q: [&LogV; 3], a: [i64; 3]) -> V {
    let n = q[0].l.len();
    let terms: Vec<V> = (0..3).map(|i| {
        if a[i] == 0 { return vec![0; n]; }
        let c = vec![one(o.sum(&[(K(tb.t("lniz4").data[a[i].unsigned_abs() as usize]), -1)], 2 * 69_078)); n];
        let k = o.ldiff(&q[i].l, &c, 0, 139);
        let mut r = vec![0; n];
        for (name, want) in [("exp4", (a[i] < 0) as i64), ("exp4n", (a[i] >= 0) as i64)] {
            let at: Vec<usize> = (0..n).filter(|&j| q[i].neg[j] == want).collect();
            let kk: V = at.iter().map(|&j| k[j]).collect();
            let e = o.tab(tb.t(name), v(&kk));
            for (s_, &j) in at.iter().enumerate() { r[j] = e[s_]; }
        }
        r
    }).collect();
    let used: Vec<usize> = (0..3).filter(|&i| a[i] != 0).collect();
    match used.len() {
        0 => vec![0; n],
        1 => terms[used[0]].clone(),
        2 => o.sum(&[(v(&terms[used[0]]), 1), (v(&terms[used[1]]), 1)], 0),
        _ => o.sum(&[(v(&terms[0]), 1), (v(&terms[1]), 1), (v(&terms[2]), 1)], 0),
    }
}

/// a t / 1000, rounded down and held to 999, for a factor t that may pass a card (a long).
pub fn mulk<O: Ops>(o: &mut O, a: A, t: A) -> V {
    let r = o.lin(&[(a, Some(t))]);
    let r = o.div(v(&r), K(1000), 0);
    o.min(v(&r), K(999))
}
