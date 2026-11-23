// The things drawn as triangles besides the stadium: the flags waving on their poles, the balloons, the airship, the
// near dancers. Each is put together here each frame (on CARDIAC: the cloth's wave, the balloons' rise, the
// airship's drift, the dancers' turn and place), drawn by raster.rs, and shaded by its own light. Included after
// air.rs.

/// The flag's grid, printed: u (thousandths) by its column, v (thousandths) by its row, and the picture's places there
/// (thousandths of a texel: 320 u across, 200 (1 - v) down).
const FLAG_U: [i64; 11] = [0, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];
const FLAG_V: [i64; 3] = [0, 500, 1000];
const FLAG_TU: [i64; 11] = [0, 32_000, 64_000, 96_000, 128_000, 160_000, 192_000, 224_000, 256_000, 288_000, 320_000];
const FLAG_TV: [i64; 3] = [200_000, 100_000, 0];
/// A pole's corners from its holder's place (centimetres along x and up).
const POLE: [(i64, i64); 4] = [(-3, 195), (3, 195), (3, 455), (-3, 455)];

/// A list of triangles drawn this frame: their pieces, and for each triangle what it is (the list's own index).
pub struct Drawn { pub p: Pieces, pub kind: Vec<u32> }

/// The flags: each a cloth 2 x 1.25 m on a grid of 11 x 3 places, from the top of a pole 3.9 m over its holder,
/// waving (y + 0.28 u sin(2 pi (0.9 u - 0.8 t) + phase), z less 0.12 u^2); its picture by the place; the light on
/// both its faces (the light through the cloth half) at each place from the holder's probe. Within 140 m, not
/// behind. And each pole: 5 cm by 2.6 m, lit from the probe. Values carried: the picture's place (thousandths of a
/// texel), the light (ten-thousandths: front + half the back + 0.1).
pub fn flags<O: Ops>(o: &mut O, t: &Tables, sc: &Scene, cam: &Cam, cube: &Cube, t12: i64) -> (Drawn, Drawn) {
    let nf = sc.flag_pid.len();
    let pos: Vec<[i64; 3]> = sc.flag_pid.iter().map(|&i| sc.cpos[i]).collect();
    // which ones: within 140 m (each way, then the squares in decimetres), not 3 m behind the eye
    let rel: [V; 3] = std::array::from_fn(|c| { let p: V = pos.iter().map(|q| q[c]).collect(); let e = s_neg(cam.eye[c], o); o.sum(&[(v(&p), 1)], e) });
    let ahead = { let d = o.lin(&[(v(&rel[0]), Some(K(cam.b[2][0]))), (v(&rel[1]), Some(K(cam.b[2][1]))), (v(&rel[2]), Some(K(cam.b[2][2])))]); let d = o.div(v(&d), K(1000), 0); o.lt(K(-300), v(&d)) };
    let dm: [V; 3] = std::array::from_fn(|c| { let a = divk(o, &rel[c], 50); o.clamp(v(&a), -577, 577) });
    let sq: [V; 3] = std::array::from_fn(|c| o.tabo(&t.sq577, 577, v(&dm[c])));
    let d2 = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
    let near = o.lt(v(&d2), K(280 * 280));
    let use_: Vec<usize> = (0..nf).filter(|&j| ahead[j] == 1 && near[j] == 1).collect();
    let (gu, gv) = (10usize, 2usize);
    let wt = angle(t12, rate(-0.8 * 2.0 * std::f64::consts::PI), 0, o);
    // the grid's places for every flag: u in tenths, v in halves
    let mut fl_ = vec![];        // (flag, iu, iv)
    for &j in &use_ { for iv in 0..=gv { for iu in 0..=gu { fl_.push((j, iu as i64, iv as i64)); } } }
    let n = fl_.len();
    if n == 0 { return (Drawn { p: Pieces::new(5), kind: vec![] }, Drawn { p: Pieces::new(3), kind: vec![] }); }
    let uu: V = fl_.iter().map(|q| FLAG_U[q.1 as usize]).collect();
    let vv: V = fl_.iter().map(|q| FLAG_V[q.2 as usize]).collect();
    let ph: V = fl_.iter().map(|q| sc.flag_ph[q.0]).collect();
    // the wave's angle: 0.9 u turns + the phase + the time's
    let ang = o.lin(&[(v(&uu), Some(K(900))), (v(&ph), None), (K(wt), None)]);
    let ang = o.turns(v(&ang));
    let (sn, cs) = o.sincos(v(&ang));
    let base: [V; 3] = std::array::from_fn(|c| fl_.iter().map(|q| pos[q.0][c]).collect());
    let x = { let a = divk(o, &uu, 5); o.sum(&[(v(&base[0]), 1), (v(&a), 1)], 0) };
    let y = { let a = o.lin(&[(v(&uu), Some(v(&sn)))]); let a = o.lin(&[(v(&a), Some(K(28)))]); let a = o.div(v(&a), K(1000), 0); let a = o.div(v(&a), K(1000), 0); o.sum(&[(v(&base[1]), 1), (v(&a), 1)], 0) };
    let z = { let u2 = o.lin(&[(v(&uu), Some(v(&uu)))]); let a = o.div(v(&u2), K(1000), 0); let a = o.lin(&[(v(&a), Some(K(12)))]); let a = o.div(v(&a), K(1000), 0); let b = o.lin(&[(v(&vv), Some(K(125)))]); let b = o.div(v(&b), K(1000), 0); o.sum(&[(v(&base[2]), 1), (v(&a), -1), (v(&b), 1)], 390 - 125) };
    // the normal: (dY/dU / 2, -1, 0) made a unit; dY/dU = 0.28 (sin + 2 pi 0.9 u cos)
    let dy = { let a = o.lin(&[(v(&uu), Some(v(&cs)))]); let a = o.div(v(&a), K(1000), 0); let a = o.lin(&[(v(&a), Some(K(5655)))]); let a = o.div(v(&a), K(1000), 0); let s = o.sum(&[(v(&sn), 1), (v(&a), 1)], 0); let s = o.lin(&[(v(&s), Some(K(14)))]); o.div(v(&s), K(100), 0) };
    let nrm_ln = o.tabo(&t.unit2, 1000, v(&dy));
    let nx = { let (l, ng) = lnv(o, t, &dy); let y = o.sum(&[(v(&l), 1), (v(&nrm_ln), 1)], LN100 - LN1000); expv(o, t, &y, Some(&ng)) };
    let ny = { let y = o.sum(&[(v(&nrm_ln), 1)], LN100); let e = expv(o, t, &y, None); o.sum(&[(v(&e), -1)], 0) };
    let nx = o.clamp(v(&nx), -99, 99);
    let ny = o.clamp(v(&ny), -99, 99);
    let nrm: Vec<[i64; 3]> = (0..n).map(|k| [nx[k], ny[k], 0]).collect();
    let (bx, by) = (o.sum(&[(v(&nx), -1)], 0), o.sum(&[(v(&ny), -1)], 0));
    let back: Vec<[i64; 3]> = (0..n).map(|k| [bx[k], by[k], 0]).collect();
    let probe: Vec<usize> = fl_.iter().map(|q| sc.cprobe()[sc.flag_pid[q.0]]).collect();
    let lf = cube_light(o, t, cube, &probe, &nrm);
    let lb = cube_light(o, t, cube, &probe, &back);
    let lt_: [V; 3] = std::array::from_fn(|c| { let h = divk(o, &lb[c], 2); o.sum(&[(v(&lf[c]), 1), (v(&h), 1)], 1000) });
    let pts: Vec<[i64; 3]> = (0..n).map(|k| [x[k], y[k], z[k]]).collect();
    let cv = to_cam(o, t, cam, &pts);
    let vals: Vec<Vec<i64>> = (0..n).map(|k| vec![FLAG_TU[fl_[k].1 as usize], FLAG_TV[fl_[k].2 as usize], lt_[0][k], lt_[1][k], lt_[2][k]]).collect();
    let per = (gu + 1) * (gv + 1);
    let mut tris = vec![];
    let mut ids = vec![];
    for (q, &j) in use_.iter().enumerate() {
        for a in 0..gv { for b in 0..gu {
            let c = |iv: usize, iu: usize| q * per + iv * (gu + 1) + iu;
            tris.push([c(a, b), c(a, b + 1), c(a + 1, b + 1)]);
            tris.push([c(a, b), c(a + 1, b + 1), c(a + 1, b)]);
            ids.push(j as u32); ids.push(j as u32);
        } }
    }
    let cloth = draw(o, t, &cv, &vals, &tris, &ids, false);
    // the poles: a quad along x, 5 cm wide, from 0.65 m to 3.25 m (its middle at 3.25 m less... as the prototype:
    // centre 3.25 m, 2.6 m tall), lit from the probe facing -y
    let mut ptri = vec![];
    let mut pid = vec![];
    for (q, &j) in use_.iter().enumerate() {
        let b0 = 4 * q;
        ptri.push([b0, b0 + 1, b0 + 2]); ptri.push([b0, b0 + 2, b0 + 3]);
        pid.push(j as u32); pid.push(j as u32);
    }
    // the corners: the holder's place plus the offsets, on CARDIAC
    let pv: Vec<[i64; 3]> = {
        let c = |i: usize| -> V { use_.iter().flat_map(|&j| [pos[j][i]; 4]).collect() };
        let (dx, dz): (V, V) = (use_.iter().flat_map(|_| POLE.map(|d| d.0)).collect(), use_.iter().flat_map(|_| POLE.map(|d| d.1)).collect());
        let (x, y, z) = (c(0), c(1), c(2));
        let x = o.sum(&[(v(&x), 1), (v(&dx), 1)], 0);
        let z = o.sum(&[(v(&z), 1), (v(&dz), 1)], 0);
        (0..x.len()).map(|k| [x[k], y[k], z[k]]).collect()
    };
    let pprobe: Vec<usize> = use_.iter().map(|&j| sc.cprobe()[sc.flag_pid[j]]).collect();
    let pl = cube_light(o, t, cube, &pprobe, &vec![[0, -99, 0]; use_.len()]);
    let pcol: [V; 3] = std::array::from_fn(|c| { let a = o.lin(&[(v(&pl[c]), Some(K(3)))]); let a = o.div(v(&a), K(10), 0); o.sum(&[(v(&a), 1)], 12) });
    let pvals: Vec<Vec<i64>> = (0..pv.len()).map(|k| { let q = k / 4; vec![pcol[0][q], pcol[1][q], pcol[2][q]] }).collect();
    let pc = to_cam(o, t, cam, &pv);
    let poles = draw(o, t, &pc, &pvals, &ptri, &pid, false);
    (Drawn { kind: vec![], p: cloth }, Drawn { kind: vec![], p: poles })
}

impl Tables {
    /// ln (100 / sqrt(1 + (x / 1000)^2)) - ln 100 for x -1000..1000 (the flag's normal's length's reciprocal)
    pub fn unit2_tab() -> Tab { tab("unit2", (-1000..=1000).map(|x: i64| -lnf((1.0 + (x as f64 / 1000.0).powi(2)).sqrt() * 1.0) ).collect()) }
    /// x^2 for x -577..577
    pub fn sq577_tab() -> Tab { tab("sq577", (-577..=577i64).map(|x| x * x).collect()) }
}

/// The balloons in the air this frame: let go at their times, rising (2.3 m + v (t - t0), under 75 m), drifting
/// (x + 1.2 sin(0.6 t + ph) + 0.15 (t - t0), y + 0.8 cos(0.5 t + ph)); each a quad facing the camera (2 r by 2.4 r),
/// cut to a disc when shaded. Within 220 m, in front. Values carried: the place on the quad, -1000..1000 across and
/// up (thousandths of the prototype's u_ and v_). And each one's centre and light (its own cube, turned to the
/// camera's axes: across, up, toward the eye).
pub struct Balloons { pub d: Drawn, pub which: Vec<usize>, pub faces: Cube }

pub fn balloons<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, sc: &Scene, cam: &Cam, lamps: &[Lamp], t12: i64) -> Balloons {
    let n = sc.bal.len();
    let col = |k: usize| -> V { sc.bal.iter().map(|b| b[k]).collect() };
    let (bx, by, t0, vel, r, ph) = (col(0), col(1), col(2), col(3), col(4), col(5));
    // a = t - t0, milliseconds
    let tms = s_div(s_lin(&[(t12, Some(1000))], o), 12, o);
    let a = o.sum(&[(v(&t0), -1)], tms);
    let up = { let p = o.lin(&[(v(&vel), Some(v(&a)))]); let p = o.div(v(&p), K(1000), 0); o.sum(&[(v(&p), 1)], 230) };
    let on1 = o.lt(K(0), v(&a));
    let on2 = o.lt(v(&up), K(7500));
    let (s1, _) = { let x = o.lin(&[(v(&ph), None), (K(t12), Some(K(rate(0.6))))]); let x = o.turns(v(&x)); o.sincos(v(&x)) };
    let (_, c2) = { let x = o.lin(&[(v(&ph), None), (K(t12), Some(K(rate(0.5))))]); let x = o.turns(v(&x)); o.sincos(v(&x)) };
    let x = { let w = o.lin(&[(v(&s1), Some(K(120)))]); let w = o.div(v(&w), K(1000), 0); let d = o.lin(&[(v(&a), Some(K(15)))]); let d = o.div(v(&d), K(1000), 0); o.sum(&[(v(&bx), 1), (v(&w), 1), (v(&d), 1)], 0) };
    let y = { let w = o.lin(&[(v(&c2), Some(K(80)))]); let w = o.div(v(&w), K(1000), 0); o.sum(&[(v(&by), 1), (v(&w), 1)], 0) };
    let pts: Vec<[i64; 3]> = (0..n).map(|k| [x[k], y[k], up[k]]).collect();
    let cv = to_cam(o, t, cam, &pts);
    // within 220 m (half-metres, held), in front
    let ahead: V = cv.iter().map(|p| p[2]).collect();
    let ahead = o.lt(K(0), v(&ahead));
    let dm: [V; 3] = std::array::from_fn(|c| { let p: V = cv.iter().map(|q| q[c]).collect(); let a = divk(o, &p, 50); o.clamp(v(&a), -577, 577) });
    let sq: [V; 3] = std::array::from_fn(|c| o.tabo(&t.sq577, 577, v(&dm[c])));
    let d2 = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
    let near = o.lt(v(&d2), K(440 * 440));
    let which: Vec<usize> = (0..n).filter(|&k| on1[k] == 1 && on2[k] == 1 && ahead[k] == 1 && near[k] == 1).collect();
    // the quads (camera space: across is x, up is y)
    let mut qv = vec![];
    let mut vals = vec![];
    let mut tris = vec![];
    let mut ids = vec![];
    let rr: V = which.iter().map(|&k| r[k]).collect();
    let r12 = { let a = o.lin(&[(v(&rr), Some(K(12)))]); o.div(v(&a), K(10), 0) };
    for (q, &k) in which.iter().enumerate() {
        let c = cv[k];
        let b0 = qv.len();
        for (sx, sy) in [(-1i64, -1i64), (1, -1), (1, 1), (-1, 1)] {
            let px = s_sum(&[(c[0], 1), (rr[q], sx)], 0, o);
            // up: 1.2 r from the middle, the middle 0.05 of the height above the quad's (v_ 0.55: a little up)
            let py = s_sum(&[(c[1], 1), (r12[q], sy)], 0, o);
            qv.push([px, py, c[2]]);
            vals.push(vec![sx * 1000, sy * 1200 - 120]);
        }
        tris.push([b0, b0 + 1, b0 + 2]); tris.push([b0, b0 + 2, b0 + 3]);
        ids.push(q as u32); ids.push(q as u32);
    }
    let p = draw(o, t, &qv, &vals, &tris, &ids, false);
    // each one's light: its cube at its middle, turned to the camera's axes
    let wpts: Vec<[i64; 3]> = which.iter().map(|&k| pts[k]).collect();
    let cube = cubes(o, t, lt, &wpts, lamps);
    let faces = cam_faces(o, t, cam, &cube);
    Balloons { d: Drawn { p, kind: vec![] }, which, faces }
}

/// Cubes turned to the camera's axes: the light on faces across (+, -), up (+, -), toward the eye (+, -: back
/// along forward, and along it), each e_x^2 C(x) + e_y^2 C(y) + e_z^2 C(z) for the axis e (thousandths).
pub fn cam_faces<O: Ops>(o: &mut O, t: &Tables, cam: &Cam, cube: &Cube) -> Cube {
    let n = cube[0][0].len();
    let axes = [cam.b[0], cam.b[1], std::array::from_fn(|k| s_neg(cam.b[2][k], o))];
    let mut out: Cube = std::array::from_fn(|_| std::array::from_fn(|_| vec![0; n]));
    for (ai, e) in axes.iter().enumerate() {
        for sgn in [1i64, -1] {
            let face = 2 * ai + (sgn < 0) as usize;
            for c in 0..3 {
                let mut acc = vec![0; n];
                for i in 0..3 {
                    let ei = s_lin(&[(e[i], Some(sgn))], o);
                    if ei == 0 { continue; }
                    let w = { let c = s_clamp(ei, -999, 999, o); s_tab_o(&t.ln_sq999, 999, c, o) };
                    let f = 2 * i + (ei < 0) as usize;
                    let (lc, _) = lnv(o, t, &cube[f][c]);
                    let y = o.sum(&[(v(&lc), 1)], w);
                    let a = expv(o, t, &y, None);
                    acc = o.sum(&[(v(&acc), 1), (v(&a), 1)], 0);
                }
                out[face][c] = acc;
            }
        }
    }
    out
}

/// The balloons' pixels: inside the disc (u^2 + v^2 under 1), the sphere's normal (u across, v up, nz toward the
/// eye), the light on it and through the rubber, the rim and the highlight: c (0.9 L + 0.7 Lt + 0.06) + L (rim +
/// spec) 0.5, rim (1 - nz)^3, spec 4 max(nz - 0.85, 0).
pub fn shade_balloons<O: Ops>(o: &mut O, t: &Tables, sc: &Scene, b: &Balloons, win: &[usize]) -> (Vec<bool>, [V; 3]) {
    let n = win.len();
    let u: V = win.iter().map(|&i| b.d.p.a[0][i]).collect();
    let w: V = win.iter().map(|&i| b.d.p.a[1][i]).collect();
    let u = o.clamp(v(&u), -999, 999);
    let w = o.clamp(v(&w), -999, 999);
    let u2 = o.tabo(&t.sq999, 999, v(&u));
    let w2 = o.tabo(&t.sq999, 999, v(&w));
    let rr = o.sum(&[(v(&u2), 1), (v(&w2), 1)], 0);
    let inside = o.lt(v(&rr), K(1000));
    let keep: Vec<bool> = inside.iter().map(|&x| x == 1).collect();
    let nz2 = o.sum(&[(v(&rr), -1)], 1000);
    let nz2 = o.max(v(&nz2), K(0));
    let nz = o.tab(&t.sqrt_m, v(&nz2));
    let nz = o.min(v(&nz), K(999));
    let who: Vec<usize> = win.iter().map(|&i| b.d.p.id[i] as usize).collect();
    let ln_n = [o.tabo(&t.ln_sq999, 999, v(&u)), o.tabo(&t.ln_sq999, 999, v(&w)), o.tabo(&t.ln_sq999, 999, v(&nz))];
    let ng = [o.tabo(&t.neg, 999_999, v(&u)), o.tabo(&t.neg, 999_999, v(&w))];
    // L(n) and L(-n): faces by the signs (toward the eye is +nz)
    let light = |o: &mut O, flip: i64, c: usize| -> V {
        let mut acc = vec![0; n];
        for ax in 0..3 {
            let f: Vec<usize> = (0..n).map(|k| { let neg = if ax < 2 { ng[ax][k] } else { 0 }; 2 * ax + ((neg + flip) % 2) as usize }).collect();
            let cv: V = (0..n).map(|k| b.faces[f[k]][c][who[k]]).collect();
            let (lc, _) = lnv(o, t, &cv);
            let y = o.sum(&[(v(&lc), 1), (v(&ln_n[ax]), 1)], 0);
            let a = expv(o, t, &y, None);
            acc = o.sum(&[(v(&acc), 1), (v(&a), 1)], 0);
        }
        acc
    };
    let rim = o.tab(&t.ball_rim, v(&nz));
    let out: [V; 3] = std::array::from_fn(|c| {
        let lb = light(o, 0, c);
        let ltr = light(o, 1, c);
        let bc: V = who.iter().map(|&q| sc.bal_c[b.which[q]][c]).collect();
        // c (0.9 Lb + 0.7 Lt + 0.06)
        let a1 = o.lin(&[(v(&lb), Some(K(9)))]);
        let a1 = o.div(v(&a1), K(10), 0);
        let a2 = o.lin(&[(v(&ltr), Some(K(7)))]);
        let a2 = o.div(v(&a2), K(10), 0);
        let s = o.sum(&[(v(&a1), 1), (v(&a2), 1)], 600);
        let ls = o.tabo(&t.ln, 999_999, v(&s));
        let lc = o.tabo(&t.ln, 999_999, v(&bc));
        let y = o.sum(&[(v(&ls), 1), (v(&lc), 1)], -LN10K);
        let body = expv(o, t, &y, None);
        // L (rim + spec) / 2 (the table has the half in it, thousandths)
        let ll = o.tabo(&t.ln, 999_999, v(&lb));
        let lr = o.tabo(&t.ln, 999_999, v(&rim));
        let y = o.sum(&[(v(&ll), 1), (v(&lr), 1)], -LN1000);
        let glow = expv(o, t, &y, None);
        o.sum(&[(v(&body), 1), (v(&glow), 1)], 0)
    });
    (keep, out)
}

impl Tables {
    /// x^2 / 1000 for x -999..999 (thousandths squared, in thousandths)
    pub fn sq999_tab() -> Tab { tab("sq999", (-999..=999i64).map(|x| (x * x + 500) / 1000).collect()) }
    /// the square root of thousandths (0..1000) in thousandths
    pub fn sqrt_m_tab() -> Tab { tab("sqrt_m", (0..=1000).map(|x| ((x as f64 * 1000.0).sqrt()).round() as i64).collect()) }
    /// a balloon's rim and highlight by nz (thousandths): ((1 - nz)^3 + 4 max(nz - 0.85, 0)) / 2, thousandths
    pub fn ball_rim_tab() -> Tab { tab("ball_rim", (0..=1000).map(|z: i64| { let n = z as f64 / 1000.0; (((1.0 - n).powi(3) + 4.0 * (n - 0.85).max(0.0)) * 0.5 * 1000.0).round() as i64 }).collect()) }
}

/// From here (t12; stadium frame 308, the airship out of shot since frame 10) its envelope is white, for its second
/// appearance; grey as the night's before (switched while it's out of shot).
pub const WHITE_T12: i64 = 300;

/// The airship: its corners where it hangs this frame; the light at its probes (riding with it: the screen and the
/// LED walls, the strobe, the moving heads, its searchlights' pools; no pyro) by each corner's normal, and the
/// bowl's glow from below (max(-n_z, 0) (0.10, 0.07, 0.12) 14); each triangle's colour its material's times that
/// (hull, greebles, masts), the frame glowing red-orange, the screen its picture. Values carried: the colour
/// (ten-thousandths), the place on the screen's picture (thousandths across and up). Not drawn when its bounding
/// sphere (125 m) is out of view.
/// (`split`, the party version: the hull drawn at the size being drawn without its screen, and the screen at the
/// film's size, apart: the second.) `white`: the envelope white (WHITE_T12 on).
pub fn airship<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, sc: &Scene, cam: &Cam, lamps: &[Lamp], bp: [i64; 3], split: bool, white: bool) -> (Option<Drawn>, Option<Drawn>) {
    let c = to_cam(o, t, cam, &[bp])[0];
    // in view: z > -125 m, |x| < 0.77 z + 160 m, |y| < 0.39 z + 140 m
    if !s_lt(-12_500, c[2], o) { return (None, None); }
    let hx = s_sum(&[(s_div(s_lin(&[(c[2], Some(77))], o), 100, o), 1)], 16_000, o);
    let hy = s_sum(&[(s_div(s_lin(&[(c[2], Some(39))], o), 100, o), 1)], 14_000, o);
    if !s_lt(s_abs(c[0], o), hx, o) || !s_lt(s_abs(c[1], o), hy, o) { return (None, None); }
    let bl = &sc.blimp;
    let world = |o: &mut O, l: &[[i64; 3]]| -> Vec<[i64; 3]> {
        let cc: [V; 3] = std::array::from_fn(|i| { let x: V = l.iter().map(|p| p[i]).collect(); o.sum(&[(v(&x), 1)], bp[i]) });
        (0..l.len()).map(|k| [cc[0][k], cc[1][k], cc[2][k]]).collect()
    };
    let probes = world(o, &sc.bl_probes());
    let cube = cubes(o, t, lt, &probes, lamps);
    let lv = cube_light(o, t, &cube, &sc.bl_vprobe(), &bl.n);
    // the glow from below: max(-n_z, 0) / 99 times (1.4, 0.98, 1.68), ten-thousandths
    let nz: V = bl.n.iter().map(|n| n[2]).collect();
    let dn = { let a = o.sum(&[(v(&nz), -1)], 0); o.max(v(&a), K(0)) };
    let lv: [V; 3] = std::array::from_fn(|c| { let k = [14_000, 9800, 16_800][c]; let a = o.lin(&[(v(&dn), Some(K(k)))]); let a = o.div(v(&a), K(99), 0); o.sum(&[(v(&lv[c]), 1), (v(&a), 1)], 0) });
    let wv = world(o, &bl.v);
    let cv = to_cam(o, t, cam, &wv);
    // each triangle's own corners: its material's albedo times the corner's light (ten-thousandths)
    const ALB: [[i64; 3]; 3] = [[550, 550, 650], [850, 800, 850], [1200, 1200, 1300]];
    // its second appearance, the envelope white (the plates, pods and gondola still dark): WHITE_T12
    let alb = |m: usize| if white && m == 0 { [7500, 7500, 7500] } else { ALB[m] };
    let nt = bl.t.len();
    let mut pts = vec![];
    let mut items: Vec<(usize, usize, usize)> = vec![];        // (corner slot, welded corner, material)
    for (j, tr) in bl.t.iter().enumerate() { for q in 0..3 { pts.push(cv[tr[q]]); items.push((3 * j + q, tr[q], sc.bl_mat[j] as usize)); } }
    let mut vals: Vec<Vec<i64>> = (0..3 * nt).map(|k| { let u = bl.t[k / 3][k % 3]; vec![0, 0, 0, sc.bl_ab[u][0], sc.bl_ab[u][1]] }).collect();
    let lit: Vec<usize> = (0..items.len()).filter(|&k| items[k].2 < 3).collect();
    for ch in 0..3 {
        let l: V = lit.iter().map(|&k| lv[ch][items[k].1]).collect();
        let a: V = lit.iter().map(|&k| alb(items[k].2)[ch]).collect();
        let ll = o.tabo(&t.ln, 999_999, v(&l));
        let la = o.tabo(&t.ln, 999_999, v(&a));
        let y = o.sum(&[(v(&ll), 1), (v(&la), 1)], -LN10K);
        let e = expv(o, t, &y, None);
        for (q, &k) in lit.iter().enumerate() { vals[k][ch] = e[q]; }
    }
    let kind: Vec<u32> = sc.bl_mat.iter().map(|&m| m as u32).collect();
    let part = |o: &mut O, on: &dyn Fn(usize) -> bool| -> Drawn {
        let js: Vec<usize> = (0..nt).filter(|&j| on(j)).collect();
        let tris: Vec<[usize; 3]> = js.iter().map(|&j| [3 * j, 3 * j + 1, 3 * j + 2]).collect();
        let ids: Vec<u32> = js.iter().map(|&j| j as u32).collect();
        Drawn { p: draw(o, t, &pts, &vals, &tris, &ids, false), kind: kind.clone() }
    };
    if !split { return (Some(part(o, &|_| true)), None); }
    let hull = part(o, &|j| sc.bl_mat[j] != 3);
    let was_full = is_full();
    set_res(true);
    let screen = part(o, &|j| sc.bl_mat[j] == 3);
    set_res(was_full);
    (Some(hull), Some(screen))
}

/// The near dancers (meshes): each one's pose (its model, move and phase), turned to its facing, sized, lifted and
/// placed: its corners in the camera's space by M v + o (M = B R s, by logarithms with the signs), the light at each
/// corner from the person's own cube (turned to its facing: the faces along its across, its forward and up) by the
/// pose's normal, the rim (1.2 (1 - |n.v|)^6, n.v from the normal along the place, its along the person's place from
/// the eye, over the distance), and for the ones in a printed shirt the print's place. Values carried: the light (3),
/// the rim, and the print's across, up and depth (thousandths, thousandths, millimetres).
pub struct Near { pub d: Drawn, pub people: Vec<usize> }

pub fn near_people<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, sc: &Scene, cam: &Cam, cr: &Crowd, lamps: &[Lamp]) -> Near {
    let people = cr.near.clone();
    let mut all = Pieces::new(7);
    if people.is_empty() { return Near { d: Drawn { p: all, kind: vec![] }, people }; }
    // each one's place (the feet raised by the lift), its cube there (a metre up), turned to its facing
    let lifts: V = people.iter().map(|&i| { let k = pose_k(sc.cmodel[i], cr.mv[i], cr.ph[i]); sc.poses[sc.cmodel[i] as usize].lift[k % (MOVES * NPHASE as usize)] }).collect();
    let mids: Vec<[i64; 3]> = { let z: V = people.iter().map(|&i| sc.cpos[i][2]).collect(); let z = o.sum(&[(v(&z), 1)], 100); people.iter().enumerate().map(|(q, &i)| [sc.cpos[i][0], sc.cpos[i][1], z[q]]).collect() };
    let cube = cubes(o, t, lt, &mids, lamps);
    let faces = facing_faces(o, t, &cube, &people.iter().map(|&i| sc.cface[i]).collect::<Vec<_>>());
    let faces_ln: [[V; 3]; 6] = std::array::from_fn(|f| std::array::from_fn(|c| lnv(o, t, &faces[f][c]).0));
    for (q, &i) in people.iter().enumerate() {
        let m = sc.cmodel[i] as usize;
        let pose = &sc.poses[m];
        let k = pose_k(sc.cmodel[i], cr.mv[i], cr.ph[i]) % (MOVES * NPHASE as usize);
        let nv = pose.nv;
        let base = k * nv;
        let vv: [V; 3] = std::array::from_fn(|c| (0..nv).map(|j| pose.v[(base + j) * 3 + c]).collect());
        let nn: [V; 3] = std::array::from_fn(|c| (0..nv).map(|j| pose.n[(base + j) * 3 + c]).collect());
        // M = B R s (thousandths): R turns the pose's x, y to (cos, sin), (-sin, cos)
        let (ca, sa) = (sc.cface[i][0], sc.cface[i][1]);
        let s = sc.cscale[i];
        let mm: [[i64; 3]; 3] = std::array::from_fn(|r| {
            let b = cam.b[r];
            let m0 = s_div(s_lin(&[(b[0], Some(ca)), (b[1], Some(sa))], o), 1000, o);
            let m1 = s_div(s_lin(&[(b[0], Some(-sa)), (b[1], Some(ca))], o), 1000, o);
            [s_div(s_lin(&[(m0, Some(s))], o), 1000, o), s_div(s_lin(&[(m1, Some(s))], o), 1000, o), s_div(s_lin(&[(b[2], Some(s))], o), 1000, o)]
        });
        let foot = [sc.cpos[i][0], sc.cpos[i][1], s_sum(&[(sc.cpos[i][2], 1), (lifts[q], 1)], 0, o)];
        let off = to_cam(o, t, cam, &[foot])[0];
        let lr: [V; 3] = std::array::from_fn(|c| o.tabo(&t.ln, 999_999, v(&vv[c])));
        let nr: [V; 3] = std::array::from_fn(|c| o.tabo(&t.neg, 999_999, v(&vv[c])));
        let cvv: [V; 3] = std::array::from_fn(|r| { let d = logdot3(o, t, &lr, &nr, mm[r], 1000); o.sum(&[(v(&d), 1)], off[r]) });
        let cv: Vec<[i64; 3]> = (0..nv).map(|j| [cvv[0][j], cvv[1][j], cvv[2][j]]).collect();
        // the light at each corner: the faces by the normal's signs, times its parts squared
        let mut l: [V; 3] = std::array::from_fn(|_| vec![0; nv]);
        for ax in 0..3 {
            let w = o.tabo(&t.ln_sq99, 99, v(&nn[ax]));
            let ng = o.tabo(&t.neg, 999_999, v(&nn[ax]));
            // pose x is the across face (0, 1), y the forward (4, 5), z up (2, 3)
            let fbase = [0usize, 4, 2][ax];
            for c in 0..3 {
                let lc: V = (0..nv).map(|j| faces_ln[fbase + ng[j] as usize][c][q]).collect();
                let y = o.sum(&[(v(&lc), 1), (v(&w), 1)], 0);
                let e = expv(o, t, &y, None);
                l[c] = o.sum(&[(v(&l[c]), 1), (v(&e), 1)], 0);
            }
        }
        // the rim: n.(p - eye) = s (n.v) / 1000 + n.D', D' = R^T (foot - eye); n.v / |p - eye| (thousandths)
        let dw: [i64; 3] = std::array::from_fn(|c| s_sum(&[(foot[c], 1), (cam.eye[c], -1)], 0, o));
        let dp = [s_div(s_lin(&[(dw[0], Some(ca)), (dw[1], Some(sa))], o), 1000, o), s_div(s_lin(&[(dw[0], Some(-sa)), (dw[1], Some(ca))], o), 1000, o), dw[2]];
        let lnr: [V; 3] = std::array::from_fn(|c| o.tabo(&t.ln, 999_999, v(&nn[c])));
        let nnr: [V; 3] = std::array::from_fn(|c| o.tabo(&t.neg, 999_999, v(&nn[c])));
        let nd = logdot3(o, t, &lnr, &nnr, dp, 1);
        let ndv0: V = (0..nv).map(|j| pose.ndotv[base + j]).collect();
        let ls_ = s_tab_o(&t.ln, 999_999, s, o);
        let snv = lscale(o, t, &ndv0, K(ls_), 3);
        let num = o.sum(&[(v(&nd), 1), (v(&snv), 1)], 0);
        let dmv: [V; 3] = std::array::from_fn(|c| { let a = o.div(v(&cvv[c]), K(10), 0); o.clamp(v(&a), -5000, 5000) });
        let sq: [V; 3] = std::array::from_fn(|c| o.tabo(&lt.sq, 5000, v(&dmv[c])));
        let d2 = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
        let d2 = o.clamp(v(&d2), 1, 999_999);
        let ld = o.tab(&lt.ln_d, v(&d2));
        let (ln_num, ngn) = lnv(o, t, &num);
        let y = o.sum(&[(v(&ln_num), 1), (v(&ld), -1)], lnf(100.0 / 99.0));
        let ndv = expv(o, t, &y, Some(&ngn));
        let ndv = o.clamp(v(&ndv), -1000, 1000);
        let rim = o.tabo(&t.rim, 1000, v(&ndv));
        let shirt = sc.cshirt[i] >= 0;
        let vals: Vec<Vec<i64>> = (0..nv).map(|j| {
            let mut x = vec![l[0][j], l[1][j], l[2][j], rim[j]];
            if shirt { x.extend_from_slice(&[pose.su[base + j], pose.sv[base + j], pose.sy[base + j]]); } else { x.extend_from_slice(&[-1, -1, 999]); }
            x
        }).collect();
        let ids = vec![q as u32; pose.t.len()];
        let p = draw(o, t, &cv, &vals, &pose.t, &ids, true);
        if std::env::var("STADIUM_DEBUG").is_ok() && q < 4 { let zs: Vec<i64> = cv.iter().map(|c| c[2]).collect(); eprintln!("person {i} q {q} pieces {} z {}..{} first {:?}", p.len(), zs.iter().min().unwrap(), zs.iter().max().unwrap(), cv[0]); }
        all.px.extend(p.px); all.iz.extend(p.iz); all.id.extend(p.id);
        for c in 0..7 { all.a[c].extend_from_slice(&p.a[c]); }
    }
    Near { d: Drawn { p: all, kind: vec![] }, people }
}

/// Cubes turned to facings (cos, sin thousandths): faces across (+, -), up (+, -), forward (+, -), as eye_faces
/// turns them to the eye (forward here where the eye was).
pub fn facing_faces<O: Ops>(o: &mut O, t: &Tables, cube: &Cube, dirs: &[[i64; 2]]) -> Cube {
    let n = dirs.len();
    let fx: V = dirs.iter().map(|d| d[0]).collect();
    let fy: V = dirs.iter().map(|d| d[1]).collect();
    // forward is the facing turned a quarter: the pose's +y is (-sin, cos); across its +x is (cos, sin)
    let to = [o.sum(&[(v(&fy), -1)], 0), fx.clone()];
    let nx = o.tabo(&t.neg, 999_999, v(&to[0]));
    let ny = o.tabo(&t.neg, 999_999, v(&to[1]));
    let to0c = o.clamp(v(&to[0]), -999, 999);
    let to1c = o.clamp(v(&to[1]), -999, 999);
    let lx2 = o.tabo(&t.ln_sq999, 999, v(&to0c));
    let ly2 = o.tabo(&t.ln_sq999, 999, v(&to1c));
    let mut out: Cube = std::array::from_fn(|_| std::array::from_fn(|_| vec![0; n]));
    // "toward" is forward (4, 5); "across" (-to_y, to_x) = (-cos, -sin)... the pose's +x is (cos, sin) = -across:
    // so its +x face is eye_faces' face 1 and -x face 0
    for (face, dir) in [(4usize, 0i64), (5, 1), (1, 2), (0, 3)] {
        for c in 0..3 {
            let (wx, wy, fxp, fyp): (&V, &V, Vec<usize>, Vec<usize>) = match dir {
                0 => (&lx2, &ly2, (0..n).map(|i| nx[i] as usize).collect(), (0..n).map(|i| ny[i] as usize).collect()),
                1 => (&lx2, &ly2, (0..n).map(|i| 1 - nx[i] as usize).collect(), (0..n).map(|i| 1 - ny[i] as usize).collect()),
                2 => (&ly2, &lx2, (0..n).map(|i| 1 - ny[i] as usize).collect(), (0..n).map(|i| nx[i] as usize).collect()),
                _ => (&ly2, &lx2, (0..n).map(|i| ny[i] as usize).collect(), (0..n).map(|i| 1 - nx[i] as usize).collect()),
            };
            let cx: V = (0..n).map(|i| cube[fxp[i]][c][i]).collect();
            let cy: V = (0..n).map(|i| cube[2 + fyp[i]][c][i]).collect();
            let (lcx, _) = lnv(o, t, &cx);
            let (lcy, _) = lnv(o, t, &cy);
            let a = { let y = o.sum(&[(v(&lcx), 1), (v(wx), 1)], 0); expv(o, t, &y, None) };
            let b = { let y = o.sum(&[(v(&lcy), 1), (v(wy), 1)], 0); expv(o, t, &y, None) };
            out[face][c] = o.sum(&[(v(&a), 1), (v(&b), 1)], 0);
        }
    }
    for c in 0..3 { out[2][c] = cube[4][c].clone(); out[3][c] = cube[5][c].clone(); }
    out
}

/// The near dancers' pixels: (darkness + rim) L + their trace of light, and where a printed back shows, the print
/// (its ink lit at 0.6 L plus its own glow under the show's blacklight: 0.05 + 0.04 kick + 0.25 strobe) over it by
/// the ink's cover.
pub fn shade_people<O: Ops>(o: &mut O, t: &Tables, sc: &Scene, sh: &Show, near: &Near, win: &[usize]) -> [V; 3] {
    let n = win.len();
    let pp = &near.d.p;
    let who: Vec<usize> = win.iter().map(|&i| near.people[pp.id[i] as usize]).collect();
    let rim: V = win.iter().map(|&i| pp.a[3][i]).collect();
    let wash = flare_wash(o, sh);
    let mut out: [V; 3] = std::array::from_fn(|c| {
        let a: V = who.iter().map(|&i| sc.ctone[i][c]).collect();
        let g: V = who.iter().map(|&i| sc.cglow[i][c]).collect();
        let l: V = win.iter().map(|&i| pp.a[c][i]).collect();
        let l = if wash > 0 { o.sum(&[(v(&l), 1)], wash) } else { l };
        let ar = o.sum(&[(v(&a), 1), (v(&rim), 1)], 0);
        let lar = o.tabo(&t.ln, 999_999, v(&ar));
        let ll = o.tabo(&t.ln, 999_999, v(&l));
        let y = o.sum(&[(v(&lar), 1), (v(&ll), 1)], -LN10K);
        let e = expv(o, t, &y, None);
        o.sum(&[(v(&e), 1), (v(&g), 1)], 0)
    });
    // the prints: on the back (depth under 20 mm), inside the print (0..1000 each way)
    let su: V = win.iter().map(|&i| pp.a[4][i]).collect();
    let sv: V = win.iter().map(|&i| pp.a[5][i]).collect();
    let sy: V = win.iter().map(|&i| pp.a[6][i]).collect();
    let a1 = o.lt(K(0), v(&su));
    let a2 = o.lt(v(&su), K(1000));
    let a3 = o.lt(K(0), v(&sv));
    let a4 = o.lt(v(&sv), K(1000));
    let a5 = o.lt(v(&sy), K(20));
    let on: Vec<usize> = (0..n).filter(|&k| a1[k] == 1 && a2[k] == 1 && a3[k] == 1 && a4[k] == 1 && a5[k] == 1 && sc.cshirt[who[k]] >= 0).collect();
    if on.is_empty() { return out; }
    let g = |x: &V| -> V { on.iter().map(|&k| x[k]).collect() };
    let tx = { let a = o.lin(&[(v(&g(&su)), Some(K(96)))]); let a = o.div(v(&a), K(1000), 0); o.clamp(v(&a), 0, 95) };
    let ty = { let b = o.sum(&[(v(&g(&sv)), -1)], 1000); let a = o.lin(&[(v(&b), Some(K(100)))]); let a = o.div(v(&a), K(1000), 0); o.clamp(v(&a), 0, 99) };
    let ink = |c: usize| -> V { on.iter().enumerate().map(|(q, &k)| sc.shirts.at(sc.cshirt[who[k]] as usize, ty[q] as usize, tx[q] as usize, c)).collect() };
    let alpha = ink(3);
    let glow = { let k = s_div(s_lin(&[(sh.kick, Some(4))], o), 100, o); let s = s_div(s_lin(&[(sh.strobe, Some(25))], o), 100, o); s_lin(&[(k, Some(10)), (s, Some(10)), (500, None)], o) };
    for c in 0..3 {
        let b = ink(c);
        let lb = o.tab(&t.ln_srgb, v(&b));
        let l: V = on.iter().map(|&k| pp.a[c][win[k]]).collect();
        let l6 = { let a = o.lin(&[(v(&l), Some(K(6)))]); let a = o.div(v(&a), K(10), 0); o.sum(&[(v(&a), 1)], glow) };
        let ll = o.tabo(&t.ln, 999_999, v(&l6));
        let y = o.sum(&[(v(&lb), 1), (v(&ll), 1)], -LN10K);
        let pr = expv(o, t, &y, None);
        // col (1 - a) + pr a, a the ink's cover (0..255)
        let old: V = on.iter().map(|&k| out[c][k]).collect();
        let d = o.sum(&[(v(&pr), 1), (v(&old), -1)], 0);
        let m = o.lin(&[(v(&d), Some(v(&alpha)))]);
        let m = o.div(v(&m), K(255), 0);
        let r = o.sum(&[(v(&old), 1), (v(&m), 1)], 0);
        for (q, &k) in on.iter().enumerate() { out[c][k] = r[q]; }
    }
    out
}
