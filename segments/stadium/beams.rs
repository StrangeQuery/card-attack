// The beams through the haze (the moving heads and the searchlights), the lasers and the pyro. Included after
// glow.rs.
//
// The beams are the prototype's: for each ray (at half size) the place along each beam's axis nearest it, the light
// a Gaussian across the beam there (its width growing from the source), fading with the height (the haze), the
// distance along it, and behind what stands nearer. Worked out only where it can show: in a band round each beam's
// line on the screen, four widths wide (the band from the line's ends and its widest width there).

/// The beams' half-size grid (of the film's screen).
pub fn hw() -> i64 { scr_w() / 2 }
pub fn hh() -> i64 { scr_h() / 2 }

/// The rays of the half-size grid in the camera's space, worked out once: each part's logarithm (thousandths:
/// (x, y, 1000) times 1 / |ray|) and sign.
pub struct HalfRays { pub ln: [V; 3], pub neg: [V; 2] }

pub fn half_rays<O: Ops>(o: &mut O, t: &Tables) -> HalfRays {
    let n = (hw() * hh()) as usize;
    // the ray through (2 xs + 1, 2 ys + 1) of the full screen: x = 2 xs + 1 - 768, y = -(2 ys + 1 - 384)
    let xs: V = (0..n).map(|i| 2 * (i as i64 % hw()) + 1 - scr_w() / 2).collect();
    let ys: V = (0..n).map(|i| -(2 * (i as i64 / hw()) + 1 - scr_h() / 2)).collect();
    let x2 = o.lin(&[(v(&xs), Some(v(&xs))), (v(&ys), Some(v(&ys)))]);
    let inv = o.tab(&t.inv_ray, v(&x2));
    let li = o.tabo(&t.ln, 999_999, v(&inv));
    let lx = { let l = o.tabo(&t.ln, 999_999, v(&xs)); o.sum(&[(v(&l), 1), (v(&li), 1)], -LN1000) };
    let ly = { let l = o.tabo(&t.ln, 999_999, v(&ys)); o.sum(&[(v(&l), 1), (v(&li), 1)], -LN1000) };
    let nx = o.tabo(&t.neg, 999_999, v(&xs));
    let ny = o.tabo(&t.neg, 999_999, v(&ys));
    HalfRays { ln: [lx, ly, li], neg: [nx, ny] }
}

/// A beam for the frame: its source (centimetres), way (thousandths), length (centimetres), width at the source
/// (centimetres), spread (thousandths of its length), colour (ten-thousandths), fall-off (0: 90 m, 1: 900 m).
pub struct Beam { pub o: [i64; 3], pub u: [i64; 3], pub len: i64, pub s0: i64, pub spread: i64, pub rgb: [i64; 3], pub fall: usize }

/// The frame's beams: the moving heads (down to the crowd's heads), the searchlights (2.5 km).
pub fn beam_list<O: Ops>(o: &mut O, sc: &Scene, sh: &Show, spot_d: &[[i64; 3]], search_d: &[[i64; 3]]) -> Vec<Beam> {
    let mut out = vec![];
    for (j, l) in sc.spots.iter().enumerate() {
        let u = spot_d[j];
        // the length down to the heads: (z - 1.8 m) / max(-u_z, 0.05)
        let dz = s_sum(&[(l.pos[2], 1)], -180, o);
        let uz = { let n = s_neg(u[2], o); s_max(n, 50, o) };
        let len = s_div(s_lin(&[(dz, Some(1000))], o), uz, o);
        let rgb = l.rgb.map(|c| s_div(s_lin(&[(c, Some(sh.spots))], o), 1000, o)).map(|c| s_div(s_lin(&[(c, Some(7))], o), 10, o));
        out.push(Beam { o: l.pos, u, len, s0: 20, spread: 22, rgb, fall: 0 });
    }
    for (j, l) in sc.search.iter().enumerate() {
        let lev = if l.mode != 0 { 1000 } else { sh.search };
        let rgb = l.rgb.map(|c| { let a = s_div(s_lin(&[(c, Some(l.power))], o), 1000, o); let a = s_div(s_lin(&[(a, Some(lev))], o), 1000, o); s_div(s_lin(&[(a, Some(55))], o), 100, o) });
        out.push(Beam { o: l.pos, u: search_d[j], len: 250_000, s0: 60, spread: if l.mode != 0 { 21 } else { 12 }, rgb, fall: 1 });
    }
    out
}

/// A band of the half-size screen as pixel runs: (row, first column, columns), from a convex polygon (half-size
/// pixels, hundredths, from the middle), by its edges' crossings.
pub fn band<O: Ops>(o: &mut O, poly: &[(i64, i64)]) -> Vec<(i64, i64, i64)> {
    let n = poly.len();
    let ends: Vec<([i64; 2], [i64; 2])> = (0..n).map(|i| { let (a, b) = (poly[i], poly[(i + 1) % n]); if b.1 < a.1 { ([b.0, b.1], [a.0, a.1]) } else { ([a.0, a.1], [b.0, b.1]) } }).collect();
    let cross = o.crossings(&ends, (-hh() / 2, hh() / 2));
    let mut rows: std::collections::BTreeMap<i64, Vec<i64>> = Default::default();
    for (_, l) in &cross { for &(row, px, _) in l { rows.entry(row).or_default().push(px); } }
    let mut out = vec![];
    for (row, mut xs) in rows {
        if xs.len() < 2 { continue; }
        xs.sort();
        let (a, b) = (xs[0].max(-hw() / 2), xs[xs.len() - 1].min(hw() / 2));
        if b > a { out.push((row, a, b - a)); }
    }
    out
}

/// The beams' light at half size (each colour), and which half-size pixels it reached. Each beam's axis (clipped to
/// the near plane and the guard band) sampled every 8 half pixels along its line on the screen: at each sample the
/// place on the axis (1 / z straight on the screen), its width (sigma), how bright the haze glows there (2.5 /
/// sqrt(max(1 - (r.u)^2, 0.04)) / (1 + sigma), the haze by the height, the fall-off along it) and the depth past which
/// the beam is hidden. Each pixel of its band (3.5 widths either side, round its ends) takes its sample's: the
/// Gaussian across the beam by its distance from the line (from the end's point past the ends), on the screen made
/// the distance in the world at the sample's depth, if nothing nearer stands in front.
pub fn volumetric<O: Ops>(o: &mut O, t: &Tables, bt: &BeamTabs, lz: &LaserTabs, cam: &Cam, beams: &[Beam], izm: &V, sub: Option<(&V, &[bool])>) -> ([V; 3], Vec<bool>, [V; 3]) {
    let n = (hw() * hh()) as usize;
    let mut acc: [V; 3] = std::array::from_fn(|_| vec![0; n]);
    let mut acc_s: [V; 3] = std::array::from_fn(|_| if sub.is_some() { vec![0; n] } else { vec![] });
    let mut touched = vec![false; n];
    for b in beams {
        let oc = to_cam(o, t, cam, &[b.o])[0];
        let uc: [i64; 3] = std::array::from_fn(|r| s_div(s_lin(&[(cam.b[r][0], Some(b.u[0])), (cam.b[r][1], Some(b.u[1])), (cam.b[r][2], Some(b.u[2]))], o), 1000, o));
        let ec: [i64; 3] = std::array::from_fn(|r| s_sum(&[(oc[r], 1), (s_div(s_lin(&[(uc[r], Some(b.len))], o), 1000, o), 1)], 0, o));
        let Some((pa, pb)) = clip_seg(o, t, &oc, &ec) else { continue };
        // the places along the beam at the clipped ends: (P - o).u
        let along = |o: &mut O, p: &[i64; 3]| -> i64 { let d: [i64; 3] = std::array::from_fn(|i| s_sum(&[(p[i], 1), (oc[i], -1)], 0, o)); s_div(s_lin(&[(d[0], Some(uc[0])), (d[1], Some(uc[1])), (d[2], Some(uc[2]))], o), 1000, o) };
        let (sba, sbb) = (along(o, &pa), along(o, &pb));
        let pr = project(o, t, &[pa, pb]);
        let (xa, ya, iza) = (s_div(pr[0].0, 2, o), s_div(pr[0].1, 2, o), pr[0].2);
        let (xb, yb, izb) = (s_div(pr[1].0, 2, o), s_div(pr[1].1, 2, o), pr[1].2);
        let (dx, dy) = (s_sum(&[(xb, 1), (xa, -1)], 0, o), s_sum(&[(yb, 1), (ya, -1)], 0, o));
        let (dxp, dyp) = (s_div(dx, 100, o), s_div(dy, 100, o));
        let ll = s_lin(&[(dxp, Some(dxp)), (dyp, Some(dyp))], o);
        let len = { let h = s_div(s_lnw({ if s_lt(ll, 1, o) { 1 } else { ll } }, t, o), 2, o); s_tab_o(&t.exp, -EXP_LO, h, o) };
        // the samples: every 8 half pixels, 1 to 256 of them; tau (thousandths) at each one's middle
        let ns = s_clamp(s_div(len, 8, o), 1, 256, o);
        let jj: V = (0..ns).map(|j| 2 * j + 1).collect();
        let tau = { let a = o.lin(&[(v(&jj), Some(K(500)))]); o.div(v(&a), K(ns), 0) };
        // depth on the screen straight in 1 / z: iz = iz_a + tau (iz_b - iz_a); the share along the 3D line w = tau iz_b / iz
        let diz = s_sum(&[(izb, 1), (iza, -1)], 0, o);
        let izj = { let (lt_, _) = lnv(o, t, &tau); let ld = s_tab_o(&t.ln, 999_999, diz, o); let nd = s_tab_o(&t.neg, 999_999, diz, o); let y = { let k_ = s_sum(&[(ld, 1)], -LN1000, o); o.sum(&[(v(&lt_), 1)], k_) }; let e = expv(o, t, &y, Some(&vec![nd; tau.len()])); o.sum(&[(v(&e), 1)], iza) };
        let izj = o.max(v(&izj), K(1));
        let lizj = o.tabo(&t.ln, 999_999, v(&izj));
        let w = { let (lt_, _) = lnv(o, t, &tau); let lb = s_tab_o(&t.ln, 999_999, izb, o); let y = o.sum(&[(v(&lt_), 1), (v(&lizj), -1)], lb); let e = expv(o, t, &y, None); o.clamp(v(&e), 0, 1000) };
        let lerp = |o: &mut O, a: i64, b: i64| -> V { let d = s_sum(&[(b, 1), (a, -1)], 0, o); let (lw, _) = lnv(o, t, &w); let ld = s_tab_o(&t.ln, 999_999, d, o); let nd = s_tab_o(&t.neg, 999_999, d, o); let y = { let k_ = s_sum(&[(ld, 1)], -LN1000, o); o.sum(&[(v(&lw), 1)], k_) }; let e = expv(o, t, &y, Some(&vec![nd; w.len()])); o.sum(&[(v(&e), 1)], a) };
        let sb = lerp(o, sba, sbb);
        let p: [V; 3] = std::array::from_fn(|c| lerp(o, pa[c], pb[c]));
        // |P| (centimetres): z |ray| at the sample's place on the screen (x_j = xa + tau dx, full-size pixels)
        let lzj = o.sum(&[(v(&lizj), -1)], lnf(KZ as f64));
        let xj = { let (lt_, _) = lnv(o, t, &tau); let ldx = s_tab_o(&t.ln, 999_999, dx, o); let ndx = s_tab_o(&t.neg, 999_999, dx, o); let y = { let k_ = s_sum(&[(ldx, 1)], -LN1000, o); o.sum(&[(v(&lt_), 1)], k_) }; let e = expv(o, t, &y, Some(&vec![ndx; tau.len()])); let a = o.sum(&[(v(&e), 1)], xa); divk(o, &a, 50 * focal() / 1000) };
        let yj = { let (lt_, _) = lnv(o, t, &tau); let ldy = s_tab_o(&t.ln, 999_999, dy, o); let ndy = s_tab_o(&t.neg, 999_999, dy, o); let y = { let k_ = s_sum(&[(ldy, 1)], -LN1000, o); o.sum(&[(v(&lt_), 1)], k_) }; let e = expv(o, t, &y, Some(&vec![ndy; tau.len()])); let a = o.sum(&[(v(&e), 1)], ya); divk(o, &a, 50 * focal() / 1000) };
        let xj = o.clamp(v(&xj), -632, 632);
        let yj = o.clamp(v(&yj), -632, 632);
        let s2 = o.lin(&[(v(&xj), Some(v(&xj))), (v(&yj), Some(v(&yj)))]);
        let inv = o.tab(&t.inv_ray, v(&s2));
        let linv = o.tabo(&t.ln, 999_999, v(&inv));
        let lp = o.sum(&[(v(&lzj), 1), (v(&linv), -1)], LN1000);
        let pu = { let a = o.lin(&[(v(&p[0]), Some(K(uc[0]))), (v(&p[1]), Some(K(uc[1]))), (v(&p[2]), Some(K(uc[2])))]); o.div(v(&a), K(1000), 0) };
        let bdot = { let (l, ng) = lnv(o, t, &pu); let y = o.sum(&[(v(&l), 1), (v(&lp), -1)], LN1000); let e = expv(o, t, &y, Some(&ng)); o.clamp(v(&e), -1000, 1000) };
        // the width (centimetres), the haze by the height, the fall-off, the peak: the sample's light (ln)
        let sig = { let a = o.lin(&[(v(&sb), Some(K(b.spread)))]); let a = o.div(v(&a), K(1000), 0); o.sum(&[(v(&a), 1)], b.s0) };
        let lsig = o.tabo(&t.ln, 999_999, v(&sig));
        let pz = { let a = o.lin(&[(v(&sb), Some(K(b.u[2])))]); let a = o.div(v(&a), K(1000), 0); let a = o.sum(&[(v(&a), 1)], b.o[2]); o.clamp(v(&a), 0, 999_999) };
        let lhaze = o.tab(&bt.ln_haze, v(&pz));
        let sbd = { let a = o.div(v(&sb), K(10), 0); o.clamp(v(&a), 0, 29_999) };
        let lfall = o.tab(&bt.ln_fall[b.fall], v(&sbd));
        let lpeak = o.tabo(&bt.ln_peak, 1000, v(&bdot));
        let sigc = o.min(v(&sig), K(99_999));
        let lw = o.tab(&bt.ln_wide, v(&sigc));
        let la = o.sum(&[(v(&lhaze), 1), (v(&lfall), 1), (v(&lpeak), 1), (v(&lw), -1)], -LN1000);
        let la = o.max(v(&la), K(-300_000));
        // the screen distance (hundredths of a half pixel) to the world's: times |P| / (f / 2) / sigma
        let cj = o.sum(&[(v(&lp), 1), (v(&lsig), -1)], -lnf(focal() as f64 / 2.0) - LN100);
        // hidden behind a surface nearer than the sample less half its width: K / (z - sigma / 2)
        let zj = { let y = o.sum(&[(v(&lizj), -1)], lnf(KZ as f64)); let y = o.min(v(&y), K(138_000)); expv(o, t, &y, None) };
        let half = divk(o, &sig, 2);
        let zt = { let a = o.sum(&[(v(&zj), 1), (v(&half), -1)], 0); o.max(v(&a), K(30)) };
        let izt = { let l = o.tabo(&t.ln, 999_999, v(&zt)); let y = o.sum(&[(v(&l), -1)], lnf(KZ as f64)); expv(o, t, &y, None) };
        // the band: 3.5 widths either side at each end, on past the ends by as much, and 2 half pixels
        let wend = |o: &mut O, k: usize| -> i64 { let a = s_sum(&[(cj[k], -1)], lnf(3.5), o); let a = s_min(a, 138_000, o); let e = s_tab_o(&t.exp, -EXP_LO, a, o); s_sum(&[(s_min(e, 60_000, o), 1)], 200, o) };
        let (wa, wb) = (wend(o, 0), wend(o, ns as usize - 1));
        let (ux, uy) = if !s_lt(0, len, o) { (1000, 0) } else { (s_div(s_lin(&[(dxp, Some(1000))], o), len, o), s_div(s_lin(&[(dyp, Some(1000))], o), len, o)) };
        let pt = |o: &mut O, x: i64, y: i64, wd: i64, along: i64, side: i64| -> (i64, i64) {
            let ax = s_div(s_lin(&[(wd, Some(ux))], o), 1000, o);
            let ay = s_div(s_lin(&[(wd, Some(uy))], o), 1000, o);
            (s_sum(&[(x, 1), (ax, along), (ay, -side)], 0, o), s_sum(&[(y, 1), (ay, along), (ax, side)], 0, o))
        };
        let poly = vec![pt(o, xa, ya, wa, -1, -1), pt(o, xb, yb, wb, 1, -1), pt(o, xb, yb, wb, 1, 1), pt(o, xa, ya, wa, -1, 1)];
        let poly = clip_y(o, t, &poly, -(hh() / 2 + 4) * 100, (hh() / 2 + 4) * 100);
        let poly = clip_axis(o, t, &poly, 0, -(hw() / 2 + 4) * 100, (hw() / 2 + 4) * 100);
        if poly.len() < 3 { continue; }
        let runs = band(o, &poly);
        if runs.is_empty() { continue; }
        // along (hundred-thousandths of the line) and across (hundredths of a half pixel) at each run's first pixel's
        // middle, and their steps a pixel
        let row_c: V = runs.iter().map(|r| 100 * r.0 + 50).collect();
        let col_c: V = runs.iter().map(|r| 100 * r.1 + 50).collect();
        let rx = { let a = { let k_ = s_neg(xa, o); o.sum(&[(v(&col_c), 1)], k_) }; o.div(v(&a), K(100), 0) };
        let ry = { let a = { let k_ = s_neg(ya, o); o.sum(&[(v(&row_c), 1)], k_) }; o.div(v(&a), K(100), 0) };
        let ll1 = { if s_lt(ll, 1, o) { 1 } else { ll } };
        let alongv = { let a = o.lin(&[(v(&rx), Some(K(dxp))), (v(&ry), Some(K(dyp)))]); let (l_, ng) = lnw_v(o, t, &a); let lll = s_lnw(ll1, t, o); let y = { let k_ = s_sum(&[(lll, -1)], 3 * LN10, o); o.sum(&[(v(&l_), 1)], k_) }; expv(o, t, &y, Some(&ng)) };
        let across = { let nuy = s_neg(uy, o); let a = o.lin(&[(v(&rx), Some(K(nuy))), (v(&ry), Some(K(ux)))]); o.div(v(&a), K(10), 0) };
        let s_step = { let la_ = s_tab_o(&t.ln, 999_999, dxp, o); let na = s_tab_o(&t.neg, 999_999, dxp, o); let y = s_sum(&[(la_, 1), (s_lnw(ll1, t, o), -1)], 6 * LN10, o); s_tab_o(if na == 1 { &t.expwn } else { &t.expw }, -EXP_LO, y, o) };
        let p_step = s_lin(&[(uy, Some(-100))], o);
        let nr: Vec<i64> = runs.iter().map(|r| r.2).collect();
        // (a line under 2 half pixels long is a point: all of it its first end's glow)
        let point = s_lt(len, 2, o);
        let sv = if point { vec![-1; nr.iter().sum::<i64>() as usize] } else { o.walka(&(0..runs.len()).map(|k| (nr[k], alongv[k], s_step, 0)).collect::<Vec<_>>()) };
        let dv = o.walka(&(0..runs.len()).map(|k| (nr[k], across[k], p_step, 0)).collect::<Vec<_>>());
        let px: Vec<usize> = runs.iter().flat_map(|&(row, a0, m)| (0..m).map(move |k| ((row + hh() / 2) * hw() + a0 + k + hw() / 2) as usize)).collect();
        // the sample: k = tau ns, held to the samples
        let sc_ = o.clamp(v(&sv), 0, 999);
        let kk = { let a = o.lin(&[(v(&sc_), Some(K(ns)))]); let a = o.div(v(&a), K(1000), 0); o.clamp(v(&a), 0, ns - 1) };
        // the distance: across inside the line, to the nearer end past it (tenths of a half pixel, then hundredths)
        let before = o.lt(v(&sv), K(0));
        let after = o.lt(K(1000), v(&sv));
        let mut dist = o.clamp(v(&dv), -999_000, 999_000);
        for (end, flag) in [((xa, ya), &before), ((xb, yb), &after)] {
            let at: Vec<usize> = (0..px.len()).filter(|&k| flag[k] == 1).collect();
            if at.is_empty() { continue; }
            let gx: V = at.iter().map(|&k| 100 * (px[k] as i64 % hw() - hw() / 2) + 50).collect();
            let gy: V = at.iter().map(|&k| 100 * (px[k] as i64 / hw() - hh() / 2) + 50).collect();
            let ex = { let a = { let k_ = s_neg(end.0, o); o.sum(&[(v(&gx), 1)], k_) }; let a = o.div(v(&a), K(10), 0); o.clamp(v(&a), -700, 700) };
            let ey = { let a = { let k_ = s_neg(end.1, o); o.sum(&[(v(&gy), 1)], k_) }; let a = o.div(v(&a), K(10), 0); o.clamp(v(&a), -700, 700) };
            let q2 = { let a = o.tabo(&lz.sq, 5000, v(&ex)); let b = o.tabo(&lz.sq, 5000, v(&ey)); o.sum(&[(v(&a), 1), (v(&b), 1)], 0) };
            let d = o.tab(&lz.sqrt, v(&q2));
            let d = o.lin(&[(v(&d), Some(K(10)))]);
            for (q, &k) in at.iter().enumerate() { dist[k] = d[q]; }
        }
        let g = |x: &V| -> V { kk.iter().map(|&k| x[k as usize]).collect() };
        let ld = o.tabo(&t.ln, 999_999, v(&dist));
        let y = o.sum(&[(v(&ld), 1), (v(&g(&cj)), 1)], 0);
        let y = o.clamp(v(&y), -200_000, 100_000);
        let lg = o.tabo(&bt.ln_gauss, 200_000, v(&y));
        let lg = o.max(v(&lg), K(-300_000));
        let pz_: V = px.iter().map(|&p| izm[p]).collect();
        let vis = o.lt(v(&pz_), v(&g(&izt)));
        let keep: Vec<usize> = (0..px.len()).filter(|&k| vis[k] == 1).collect();
        for &p in &px { touched[p] = true; }
        if keep.is_empty() { continue; }
        let base = { let a: V = keep.iter().map(|&k| lg[k]).collect(); let la_: V = keep.iter().map(|&k| la[kk[k] as usize]).collect(); o.sum(&[(v(&a), 1), (v(&la_), 1)], 0) };
        let kpx: Vec<usize> = keep.iter().map(|&k| px[k]).collect();
        // (the party version: where a half pixel holds a subject drawn at full size, whether the beam is in front
        // of it too: its light there kept apart)
        let ks: Vec<usize> = match sub {
            Some((izs, has)) => {
                let at: Vec<usize> = (0..keep.len()).filter(|&q| has[kpx[q]]).collect();
                if at.is_empty() { vec![] } else {
                    let zs: V = at.iter().map(|&q| izs[kpx[q]]).collect();
                    let lim: V = at.iter().map(|&q| izt[kk[keep[q]] as usize]).collect();
                    let f = o.lt(v(&zs), v(&lim));
                    (0..at.len()).filter(|&j| f[j] == 1).map(|j| at[j]).collect()
                }
            }
            None => vec![],
        };
        for c in 0..3 {
            if b.rgb[c] == 0 { continue; }
            let lc = s_tab_o(&t.ln, 999_999, b.rgb[c], o);
            let yy = o.sum(&[(v(&base), 1)], lc);
            let a = expv(o, t, &yy, None);
            add_to(o, &mut acc[c], &kpx, &a);
            if !ks.is_empty() {
                let a2: V = ks.iter().map(|&q| a[q]).collect();
                let p2: Vec<usize> = ks.iter().map(|&q| kpx[q]).collect();
                add_to(o, &mut acc_s[c], &p2, &a2);
            }
        }
    }
    (acc, touched, acc_s)
}

/// Each half pixel's nearest surface (its 2 x 2's greatest K / z) from the film's depths.
pub fn quad_max<O: Ops>(o: &mut O, iz: &V) -> V {
    let n = (hw() * hh()) as usize;
    let quad: [V; 4] = std::array::from_fn(|k| (0..n).map(|i| { let (x, y) = (2 * (i % hw() as usize) + k % 2, 2 * (i / hw() as usize) + k / 2); iz[y * scr_w() as usize + x] }).collect());
    let m1 = o.max(v(&quad[0]), v(&quad[1]));
    let m2 = o.max(v(&quad[2]), v(&quad[3]));
    o.max(v(&m1), v(&m2))
}

/// sum_i r_i k_i / s for the rays' parts (by their logarithms and signs) and a constant vector: each term e of the
/// sum of the logarithms, its sign the product of the two (the table of e or of -e by the ray's sign and the
/// constant's).
pub fn logdot3<O: Ops>(o: &mut O, t: &Tables, lr: &[V; 3], nr: &[V; 3], k: [i64; 3], s: i64) -> V {
    let n = lr[0].len();
    let mut terms: Vec<V> = vec![];
    for i in 0..3 {
        if k[i] == 0 { continue; }
        let lk = s_tab_o(&t.ln, 999_999, k[i], o);
        let kneg = s_tab_o(&t.neg, 999_999, k[i], o);
        let ls = s_tab_o(&t.ln, 999_999, s, o);
        let k_ = s_sum(&[(lk, 1), (ls, -1)], 0, o);
        let y = o.sum(&[(v(&lr[i]), 1)], k_);
        // the sign differs from the constant's: the term's sign as it is, or the other way, on the machines
        let ng = if kneg == 0 { nr[i].clone() } else { o.sum(&[(v(&nr[i]), -1)], 1) };
        terms.push(expv(o, t, &y, Some(&ng)));
    }
    match terms.len() {
        0 => vec![0; n],
        1 => terms.pop().unwrap(),
        _ => { let tt: Vec<(A, i64)> = terms.iter().map(|x| (v(x), 1)).collect(); o.sum(&tt, 0) }
    }
}

/// A line (camera space, centimetres) clipped to the near plane (50 cm) and the guard band (x and y within 8 z /
/// 9), or none.
pub fn clip_seg<O: Ops>(o: &mut O, t: &Tables, a: &[i64; 3], b: &[i64; 3]) -> Option<([i64; 3], [i64; 3])> {
    let plane = |o: &mut O, p: &[i64; 3], k: usize| -> i64 {
        match k {
            0 => s_sum(&[(p[2], 1)], -50, o),
            1 => s_sum(&[(s_clamp(p[2], -55_000, 55_000, o), 8), (s_clamp(p[0], -55_000, 55_000, o), -9)], 0, o),
            2 => s_sum(&[(s_clamp(p[2], -55_000, 55_000, o), 8), (s_clamp(p[0], -55_000, 55_000, o), 9)], 0, o),
            3 => s_sum(&[(s_clamp(p[2], -55_000, 55_000, o), 4), (s_clamp(p[1], -55_000, 55_000, o), -9)], 0, o),
            _ => s_sum(&[(s_clamp(p[2], -55_000, 55_000, o), 4), (s_clamp(p[1], -55_000, 55_000, o), 9)], 0, o),
        }
    };
    let (mut pa, mut pb) = (*a, *b);
    for k in 0..5 {
        let (da, db) = (plane(o, &pa, k), plane(o, &pb, k));
        let (ia, ib) = (!s_lt(da, 0, o), !s_lt(db, 0, o));
        if !ia && !ib { return None; }
        if ia && ib { continue; }
        // the outside end moved to where the plane is crossed: out + (in - out) d_out / (d_out - d_in)
        let (out, inn, dout, din) = if ia { (pb, pa, db, da) } else { (pa, pb, da, db) };
        let den = s_lin(&[(dout, None), (din, Some(-1))], o);
        let s = s_ldivw(dout, den, 3, t, o);
        let c: [i64; 3] = std::array::from_fn(|i| { let d = s_sum(&[(inn[i], 1), (out[i], -1)], 0, o); s_sum(&[(out[i], 1), (s_div(s_lin(&[(d, Some(s))], o), 1000, o), 1)], 0, o) });
        if ia { pb = c; } else { pa = c; }
    }
    Some((pa, pb))
}

/// A convex polygon (hundredths) cut to lo <= y <= hi (each side in turn: a corner made where an edge crosses it).
pub fn clip_y<O: Ops>(o: &mut O, t: &Tables, poly: &[(i64, i64)], lo: i64, hi: i64) -> Vec<(i64, i64)> { clip_axis(o, t, poly, 1, lo, hi) }

/// The same along x (axis 0) or y (1).
pub fn clip_axis<O: Ops>(o: &mut O, t: &Tables, poly: &[(i64, i64)], axis: usize, lo: i64, hi: i64) -> Vec<(i64, i64)> {
    let get = |q: (i64, i64)| if axis == 0 { q.0 } else { q.1 };
    let other = |q: (i64, i64)| if axis == 0 { q.1 } else { q.0 };
    let mut p = poly.to_vec();
    for (lim, sign) in [(lo, 1i64), (hi, -1)] {
        if p.is_empty() { break; }
        let d: Vec<i64> = p.iter().map(|&q| s_sum(&[(get(q), sign)], -sign * lim, o)).collect();
        let ins: Vec<bool> = d.iter().map(|&x| !s_lt(x, 0, o)).collect();
        let n = p.len();
        let mut out = vec![];
        for i in 0..n {
            let j = (i + 1) % n;
            if ins[i] { out.push(p[i]); }
            if ins[i] != ins[j] {
                // p_i + (p_j - p_i) d_i / (d_i - d_j)
                let den = s_sum(&[(d[i], 1), (d[j], -1)], 0, o);
                let s = s_ldiv(d[i], den, 3, t, o);
                let w = s_sum(&[(other(p[i]), 1), (s_div(s_lin(&[(s_sum(&[(other(p[j]), 1), (other(p[i]), -1)], 0, o), Some(s))], o), 1000, o), 1)], 0, o);
                out.push(if axis == 0 { (lim, w) } else { (w, lim) });
            }
        }
        p = out;
    }
    p
}

/// The beams' printed tables.
pub struct BeamTabs {
    /// by r.u (thousandths, -1000..1000): ln max(1 - x^2, 0.04) (thousandths... of the unit), ln sqrt(1 - x^2), ln
    /// 2.5 / sqrt(max(1 - x^2, 0.04))
    pub ln_den: Tab,
    pub ln_sin: Tab,
    pub ln_peak: Tab,
    /// by ln(dist / sigma) (ten-thousandths, -200,000..100,000): ln e^(-0.5 (dist / sigma)^2)
    pub ln_gauss: Tab,
    /// by the height (centimetres): ln e^(-max(z - 45 m, 0) / 400 m)
    pub ln_haze: Tab,
    /// by the place along (decimetres): ln (1 + s / fall)^-2, fall 90 m and 900 m
    pub ln_fall: [Tab; 2],
    /// by sigma (centimetres): ln (1 + sigma)
    pub ln_wide: Tab,
    /// half the ln of x (0..999,999), ten-thousandths
    pub half_ln: Tab,
}

pub fn beam_tabs() -> BeamTabs {
    let by_dot = |f: &dyn Fn(f64) -> f64| -> Tab { tab("dot", (-1000..=1000).map(|x| { let y = f(x as f64 / 1000.0); if y <= 0.0 { LN0 } else { lnf(y) } }).collect()) };
    BeamTabs {
        ln_den: by_dot(&|x| (1.0 - x * x).max(0.04)),
        ln_sin: by_dot(&|x| (1.0 - x * x).max(1e-9).sqrt()),
        ln_peak: by_dot(&|x| 2.5 / (1.0 - x * x).max(0.04).sqrt() * 1000.0),
        ln_gauss: tab("gauss", (-200_000..=100_000).map(|y| { let r = (y as f64 / 10_000.0).exp(); let g = (-0.5 * r * r).exp(); if g < 1e-30 { -400_000 } else { lnf(g * 1000.0) - LN1000 } }).collect()),
        ln_haze: tab("haze", (0..1_000_000).map(|z| lnf((-((z as f64 / 100.0 - 45.0).max(0.0)) / 400.0).exp())).collect()),
        ln_fall: [tab("fall90", (0..30_000).map(|s| lnf((1.0 + s as f64 / 10.0 / 90.0).powi(-2) * 1e6) - lnf(1e6)).collect()), tab("fall900", (0..30_000).map(|s| lnf((1.0 + s as f64 / 10.0 / 900.0).powi(-2) * 1e6) - lnf(1e6)).collect())],
        ln_wide: tab("wide", (0..100_000).map(|s| lnf(1.0 + s as f64 / 100.0)).collect()),
        half_ln: tab("half_ln", (0..1_000_000).map(|x| if x == 0 { LN0 } else { lnf((x as f64).sqrt()) }).collect()),
    }
}

/// The beams' half-size light made full size (the prototype's bilinear: across, each pixel 3/4 of its own half pixel
/// and 1/4 of its neighbour on its side; then the same up and down) and added, where any beam reached. The quarters
/// and three quarters by logarithms.
pub fn add_beams<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, acc: &[V; 3], touched: &[bool]) {
    let (hw_, hh_) = (hw() as usize, hh() as usize);
    // the half pixels to work on: those reached and their neighbours
    let mut near_h = vec![false; hw_ * hh_];
    for i in 0..hw_ * hh_ { if touched[i] { let (x, y) = (i % hw_, i / hw_); for j in y.saturating_sub(1)..=(y + 1).min(hh_ - 1) { for k in x.saturating_sub(1)..=(x + 1).min(hw_ - 1) { near_h[j * hw_ + k] = true; } } } }
    let hs: Vec<usize> = (0..hw_ * hh_).filter(|&i| near_h[i]).collect();
    if hs.is_empty() { return; }
    // across: for each half row and full column whose half pixel is in use
    let cols: Vec<(usize, usize)> = hs.iter().flat_map(|&i| { let (x, y) = (i % hw_, i / hw_); [(y, 2 * x), (y, 2 * x + 1)] }).collect();
    let px: Vec<usize> = hs.iter().flat_map(|&i| { let (x, y) = (i % hw_, i / hw_); [(2 * y) * scr_w() as usize + 2 * x, (2 * y) * scr_w() as usize + 2 * x + 1, (2 * y + 1) * scr_w() as usize + 2 * x, (2 * y + 1) * scr_w() as usize + 2 * x + 1] }).collect();
    let scale = |o: &mut O, x: &V, k: f64| -> V { let l = o.tabo(&t.ln, 999_999, v(x)); let y = o.sum(&[(v(&l), 1)], lnf(k)); expv(o, t, &y, None) };
    for c in 0..3 {
        let own: V = cols.iter().map(|&(y, x)| acc[c][y * hw_ + x / 2]).collect();
        let nb: V = cols.iter().map(|&(y, x)| { let hx = (x / 2) as i64; let n = (if x % 2 == 0 { hx - 1 } else { hx + 1 }).clamp(0, hw() - 1); acc[c][y * hw_ + n as usize] }).collect();
        let a = scale(o, &own, 0.75);
        let b = scale(o, &nb, 0.25);
        let h = o.sum(&[(v(&a), 1), (v(&b), 1)], 0);
        // the across values by (half row, full column)
        let mut hv: std::collections::HashMap<(usize, usize), i64> = Default::default();
        for (q, &k) in cols.iter().enumerate() { hv.insert(k, h[q]); }
        // up and down: each full pixel 3/4 of its half row's, 1/4 of the neighbouring half row's (0 where unused)
        let at = |y: usize, x: usize| -> i64 { *hv.get(&(y, x)).unwrap_or(&0) };
        let own: V = px.iter().map(|&p| { let (x, y) = (p % scr_w() as usize, p / scr_w() as usize); at(y / 2, x) }).collect();
        let nb: V = px.iter().map(|&p| { let (x, y) = (p % scr_w() as usize, (p / scr_w() as usize) as i64); let hy = y / 2; let n = (if y % 2 == 0 { hy - 1 } else { hy + 1 }).clamp(0, hh() - 1); at(n as usize, x) }).collect();
        let a = scale(o, &own, 0.75);
        let b = scale(o, &nb, 0.25);
        let old: V = px.iter().map(|&p| f.col[c][p]).collect();
        let s = o.sum(&[(v(&old), 1), (v(&a), 1), (v(&b), 1)], 0);
        for (q, &p) in px.iter().enumerate() { f.col[c][p] = s[q]; }
    }
}

/// Pixel runs of a convex polygon (hundredths, from the middle) on the full screen: (row, first column, columns).
pub fn runs_full<O: Ops>(o: &mut O, poly: &[(i64, i64)]) -> Vec<(i64, i64, i64)> {
    let n = poly.len();
    let ends: Vec<([i64; 2], [i64; 2])> = (0..n).map(|i| { let (a, b) = (poly[i], poly[(i + 1) % n]); if b.1 < a.1 { ([b.0, b.1], [a.0, a.1]) } else { ([a.0, a.1], [b.0, b.1]) } }).collect();
    let cross = o.crossings(&ends, (-scr_h() / 2, scr_h() / 2));
    let mut rows: std::collections::BTreeMap<i64, Vec<i64>> = Default::default();
    for (_, l) in &cross { for &(row, px, _) in l { rows.entry(row).or_default().push(px); } }
    let mut out = vec![];
    for (row, mut xs) in rows {
        if xs.len() < 2 { continue; }
        xs.sort();
        let (a, b) = (xs[0].max(-scr_w() / 2), xs[xs.len() - 1].min(scr_w() / 2));
        if b > a { out.push((row, a, b - a)); }
    }
    out
}

/// ln |x| and the sign for numbers up to three cards (past a long by their thousands).
pub fn lnw_v<O: Ops>(o: &mut O, t: &Tables, x: &V) -> (V, V) {
    let big = { let lo = o.lt(v(x), K(-999_000)); let hi = o.lt(K(999_999), v(x)); (0..x.len()).map(|i| (lo[i] == 1 || hi[i] == 1) as i64).collect::<V>() };
    let x3 = o.div(v(x), K(1000), 0);
    let (l3, _) = lnv(o, t, &x3);
    let l3 = o.sum(&[(v(&l3), 1)], LN1000);
    let xs = o.sel(&big, K(0), v(x));
    let (l1, _) = lnv(o, t, &xs);
    let n3 = o.tabo(&t.neg, 999_999, v(&x3));
    let n1 = o.tabo(&t.neg, 999_999, v(&xs));
    let ng = o.sel(&big, v(&n3), v(&n1));
    (o.sel(&big, v(&l3), v(&l1)), ng)
}

/// ln of a positive number up to three cards (past a long by its thousands: to a part in a thousand).
pub fn s_lnw<O: Ops>(x: i64, t: &Tables, o: &mut O) -> i64 {
    if s_lt(x, 1_000_000, o) { return s_tab_o(&t.ln, 999_999, x, o); }
    s_sum(&[(s_tab_o(&t.ln, 999_999, s_div(x, 1000, o), o), 1)], LN1000, o)
}

/// The same with b up to three cards (its ln by its thousands past a long).
pub fn s_ldivw<O: Ops>(a: i64, b: i64, k: i64, t: &Tables, o: &mut O) -> i64 {
    let la = s_tab_o(&t.ln, 999_999, a, o);
    let (lb, nb) = lnw_v(o, t, &vec![b]);
    let na = s_tab_o(&t.neg, 999_999, a, o);
    let y = s_sum(&[(la, 1), (lb[0], -1)], k * LN10, o);
    let neg = s_divmod(s_sum(&[(na, 1), (nb[0], 1)], 0, o), 2, o).1;
    s_tab_o(if neg == 1 { &t.expwn } else { &t.expw }, -EXP_LO, y, o)
}

/// a / b times 10^k for any a and b != 0, by logarithms (the sign of a b), up to three cards.
pub fn s_ldiv<O: Ops>(a: i64, b: i64, k: i64, t: &Tables, o: &mut O) -> i64 {
    let la = s_tab_o(&t.ln, 999_999, a, o);
    let lb = s_tab_o(&t.ln, 999_999, b, o);
    let na = s_tab_o(&t.neg, 999_999, a, o);
    let nb = s_tab_o(&t.neg, 999_999, b, o);
    let y = s_sum(&[(la, 1), (lb, -1)], k * LN10, o);
    let neg = s_divmod(s_sum(&[(na, 1), (nb, 1)], 0, o), 2, o).1;
    s_tab_o(if neg == 1 { &t.expwn } else { &t.expw }, -EXP_LO, y, o)
}

/// The lasers: each beam a line from its source 125 m out, a bright core and a glow (by the distance from the
/// line's piece on the screen, pixels), hidden behind what stands nearer (its depth along the line, 1 / z linear on
/// the screen), fading with the depth and along it. Worked out within 20 pixels of the line and the prototype's box
/// round it (12 pixels past its ends).
///
/// `band`: 1 only within 4 pixels of the line and its ends (the core and the glow next to it: drawn at the film's
/// size, its pixels marked); 2 the glow, at the size being drawn (the half size: widths in its pixels, distances in
/// the film's).
pub fn lasers<O: Ops>(o: &mut O, t: &Tables, bt: &LaserTabs, f: &mut Frame, cam: &Cam, sc: &Scene, sh: &Show, band: u8) {
    const NEARL: i64 = 50;
    let qs = [0i64, -2, 2, -1, 1];
    for (li, l) in sc.lasers.iter().enumerate() {
        let beams = laser_beams(o, t, sh, l, qs[li]);
        // the source's beams' light (one colour: its own) gathered, then coloured once
        let mut inten = vec![0i64; npix()];
        let mut lit = vec![false; npix()];
        for (d, on) in beams {
            if s_lt(on, 20, o) { continue; }
            let far: [i64; 3] = std::array::from_fn(|i| s_sum(&[(l.pos[i], 1), (s_div(s_lin(&[(d[i], Some(12_500))], o), 1000, o), 1)], 0, o));
            let cv = to_cam(o, t, cam, &[l.pos, far]);
            let Some((a, b)) = clip_seg(o, t, &cv[0], &cv[1]) else { continue };
            let pr = project(o, t, &[a, b]);
            let ((ax, ay, iza), (bx, by, izb)) = (pr[0], pr[1]);
            // the line on the screen: its run (pixels) and its length squared
            let (vx, vy) = (s_sum(&[(bx, 1), (ax, -1)], 0, o), s_sum(&[(by, 1), (ay, -1)], 0, o));
            let (vxp, vyp) = (s_div(vx, 100, o), s_div(vy, 100, o));
            let ll = s_lin(&[(vxp, Some(vxp)), (vyp, Some(vyp))], o);
            if !s_lt(0, ll, o) { continue; }
            let len = { let h = s_div(s_lnw(ll, t, o), 2, o); s_tab_o(&t.exp, -EXP_LO, h, o) };
            // the band: the line pushed 12 px past its ends, 30 px either side; inside the prototype's box
            let ux = s_div(s_lin(&[(vxp, Some(1000))], o), len, o);
            let uy = s_div(s_lin(&[(vyp, Some(1000))], o), len, o);
            let w = |o: &mut O, k: i64, px: i64| -> i64 { s_div(s_lin(&[(k, Some(px * 100))], o), 1000, o) };
            // (the core: 4 px either side and past the ends; at half size the widths halved)
            let (we, ws) = match band { 1 => (4, 4), _ => (12 * focal() / 1000, 20 * focal() / 1000) };
            let (ex, ey, sx, sy) = (w(o, ux, we), w(o, uy, we), w(o, ux, ws), w(o, uy, ws));
            let poly = [
                (s_sum(&[(ax, 1), (ex, -1), (sy, 1)], 0, o), s_sum(&[(ay, 1), (ey, -1), (sx, -1)], 0, o)),
                (s_sum(&[(bx, 1), (ex, 1), (sy, 1)], 0, o), s_sum(&[(by, 1), (ey, 1), (sx, -1)], 0, o)),
                (s_sum(&[(bx, 1), (ex, 1), (sy, -1)], 0, o), s_sum(&[(by, 1), (ey, 1), (sx, 1)], 0, o)),
                (s_sum(&[(ax, 1), (ex, -1), (sy, -1)], 0, o), s_sum(&[(ay, 1), (ey, -1), (sx, 1)], 0, o)),
            ];
            let bm = 100 * we;
            let bx0 = s_div(s_sum(&[(s_min(ax, bx, o), 1)], -bm, o), 100, o);
            let bx1 = s_div(s_sum(&[(s_max(ax, bx, o), 1)], bm, o), 100, o);
            let by0 = s_div(s_sum(&[(s_min(ay, by, o), 1)], -bm, o), 100, o);
            let by1 = s_div(s_sum(&[(s_max(ay, by, o), 1)], bm, o), 100, o);
            let poly = clip_y(o, t, &poly, -(scr_h() / 2 + 4) * 100, (scr_h() / 2 + 4) * 100);
            let poly = clip_axis(o, t, &poly, 0, -(scr_w() / 2 + 4) * 100, (scr_w() / 2 + 4) * 100);
            if poly.len() < 3 { continue; }
            let runs: Vec<(i64, i64, i64)> = runs_full(o, &poly).into_iter().filter(|r| r.0 >= by0 && r.0 <= by1).map(|(row, a0, m)| { let a1 = a0.max(bx0); let b1 = (a0 + m).min(bx1 + 1); (row, a1, b1 - a1) }).filter(|r| r.2 > 0).collect();
            if runs.is_empty() { continue; }
            // along the line (thousandths) and across it (tenths of a pixel) at each run's first pixel's middle, and
            // their steps a pixel (thousandths of those)
            let row_c: V = runs.iter().map(|r| 100 * r.0 + 50).collect();
            let col_c: V = runs.iter().map(|r| 100 * r.1 + 50).collect();
            let dxs = { let k_ = s_neg(ax, o); o.sum(&[(v(&col_c), 1)], k_) };
            let dys = { let k_ = s_neg(ay, o); o.sum(&[(v(&row_c), 1)], k_) };
            let dxp = o.div(v(&dxs), K(100), 0);
            let dyp = o.div(v(&dys), K(100), 0);
            let along = { let a = o.lin(&[(v(&dxp), Some(K(vxp))), (v(&dyp), Some(K(vyp)))]); let (l_, ng) = lnw_v(o, t, &a); let lll = s_lnw(ll, t, o); let y = { let k_ = s_sum(&[(lll, -1)], 3 * LN10, o); o.sum(&[(v(&l_), 1)], k_) }; expv(o, t, &y, Some(&ng)) };
            let across = { let nuy = s_neg(uy, o); let a = o.lin(&[(v(&dxp), Some(K(nuy))), (v(&dyp), Some(K(ux)))]); o.div(v(&a), K(100), 0) };
            let s_step = { let la = s_tab_o(&t.ln, 999_999, vxp, o); let na = s_tab_o(&t.neg, 999_999, vxp, o); let y = s_sum(&[(la, 1), (s_lnw(ll, t, o), -1)], 6 * LN10, o); s_tab_o(if na == 1 { &t.expwn } else { &t.expw }, -EXP_LO, y, o) };
            let p_step = s_lin(&[(uy, Some(-10))], o);
            let n: Vec<i64> = runs.iter().map(|r| r.2).collect();
            let wa: Vec<(i64, i64, i64, i64)> = (0..runs.len()).map(|k| (n[k], along[k], s_step, 0)).collect();
            let wp: Vec<(i64, i64, i64, i64)> = (0..runs.len()).map(|k| (n[k], across[k], p_step, 0)).collect();
            let sv = o.walka(&wa);
            let pv = o.walka(&wp);
            let px: Vec<usize> = runs.iter().flat_map(|&(row, a0, m)| (0..m).map(move |k| ((row + scr_h() / 2) * scr_w() + a0 + k + scr_w() / 2) as usize)).collect();
            // the distance (tenths of a pixel): across where the nearest point is inside the line, else to the
            // nearer end
            let s_c = o.clamp(v(&sv), 0, 1000);
            let before = o.lt(v(&sv), K(0));
            let after = o.lt(K(1000), v(&sv));
            let mut dist = o.clamp(v(&pv), -9999, 9999);
            for (end, flag) in [((ax, ay), &before), ((bx, by), &after)] {
                let at: Vec<usize> = (0..px.len()).filter(|&k| flag[k] == 1).collect();
                if at.is_empty() { continue; }
                let gx: V = at.iter().map(|&k| 100 * (px[k] as i64 % scr_w() - scr_w() / 2) + 50).collect();
                let gy: V = at.iter().map(|&k| 100 * (px[k] as i64 / scr_w() - scr_h() / 2) + 50).collect();
                let dx = { let a = { let k_ = s_neg(end.0, o); o.sum(&[(v(&gx), 1)], k_) }; let a = o.div(v(&a), K(10), 0); o.clamp(v(&a), -700, 700) };
                let dy = { let a = { let k_ = s_neg(end.1, o); o.sum(&[(v(&gy), 1)], k_) }; let a = o.div(v(&a), K(10), 0); o.clamp(v(&a), -700, 700) };
                let q2 = { let a = o.tabo(&bt.sq, 5000, v(&dx)); let b = o.tabo(&bt.sq, 5000, v(&dy)); o.sum(&[(v(&a), 1), (v(&b), 1)], 0) };
                let d = o.tab(&bt.sqrt, v(&q2));
                for (q, &k) in at.iter().enumerate() { dist[k] = d[q]; }
            }
            // (at half size: the distance in the film's pixels)
            let dist = if is_full() { dist } else { let d = o.lin(&[(v(&dist), Some(K(2)))]); o.clamp(v(&d), -9999, 9999) };
            let lt_ = o.tabo(&bt.ln_core, 9999, v(&dist));
            // samples every 8 pixels along the line: their depth (iz_a + s (iz_b - iz_a): 1 / z straight on the
            // screen) and the fading there (with the depth, along the line), with the beam's level
            let lenp = { let h = s_div(s_lnw(ll, t, o), 2, o); s_tab_o(&t.exp, -EXP_LO, h, o) };
            let ns = s_clamp(s_div(lenp, 8, o), 1, 256, o);
            let jj: V = (0..ns).map(|j| 2 * j + 1).collect();
            let tau = { let a = o.lin(&[(v(&jj), Some(K(500)))]); o.div(v(&a), K(ns), 0) };
            let diz = s_sum(&[(izb, 1), (iza, -1)], 0, o);
            let izj = {
                let (ls, _) = lnv(o, t, &tau);
                let ld = s_tab_o(&t.ln, 999_999, diz, o);
                let nd = s_tab_o(&t.neg, 999_999, diz, o);
                let y = { let k_ = s_sum(&[(ld, 1)], -3 * LN10, o); o.sum(&[(v(&ls), 1)], k_) };
                let m = expv(o, t, &y, Some(&vec![nd; ls.len()]));
                let m = o.sum(&[(v(&m), 1)], iza);
                o.clamp(v(&m), 0, 999_999)
            };
            let lev = s_div(s_lin(&[(s_div(s_lin(&[(on, Some(14))], o), 10, o), Some(sh.lasers))], o), 1000, o);
            let llev = s_tab_o(&t.ln, 999_999, lev, o);
            let cj = { let lf = o.tab(&bt.ln_fade, v(&izj)); let la = o.tab(&bt.ln_along, v(&tau)); { let k_ = s_sum(&[(llev, 1)], -4 * LN1000 + LN10K, o); o.sum(&[(v(&lf), 1), (v(&la), 1)], k_) } };
            // each pixel's sample; seen where its surface is farther than the sample
            let kk = { let a = o.lin(&[(v(&s_c), Some(K(ns)))]); let a = o.div(v(&a), K(1000), 0); o.clamp(v(&a), 0, ns - 1) };
            let pz: V = px.iter().map(|&p| f.iz[p]).collect();
            let izk: V = kk.iter().map(|&k| izj[k as usize]).collect();
            let vis = o.lt(v(&pz), v(&izk));
            let keep: Vec<usize> = (0..px.len()).filter(|&k| vis[k] == 1).collect();
            if keep.is_empty() { continue; }
            // the light (ten-thousandths of the unit) before its colour
            let a1: V = keep.iter().map(|&k| lt_[k]).collect();
            let a2: V = keep.iter().map(|&k| cj[kk[k] as usize]).collect();
            let base = o.sum(&[(v(&a1), 1), (v(&a2), 1)], 0);
            let e = expv(o, t, &base, None);
            let kpx: Vec<usize> = keep.iter().map(|&k| px[k]).collect();
            for &p in &kpx { lit[p] = true; }
            add_to(o, &mut inten, &kpx, &e);
        }
        let px: Vec<usize> = (0..npix()).filter(|&p| lit[p]).collect();
        if px.is_empty() { continue; }
        let li_: V = px.iter().map(|&p| inten[p]).collect();
        let ll = o.tabo(&t.ln, 999_999, v(&li_));
        for c in 0..3 {
            if l.rgb[c] == 0 { continue; }
            let lc = s_tab_o(&t.ln, 999_999, l.rgb[c], o);
            let y = { let k_ = s_sum(&[(lc, 1)], -LN10K, o); o.sum(&[(v(&ll), 1)], k_) };
            let a = expv(o, t, &y, None);
            add_to(o, &mut f.col[c], &px, &a);
        }
        f.mark(&px);
    }
}

pub struct LaserTabs {
    /// by the distance (tenths of a pixel, either sign): ln (1.6 e^-(d / 0.9)^2 + 0.25 e^(-d / 5)), thousandths
    pub ln_core: Tab,
    /// by K / z: ln e^(-z / 500 m) (thousandths); by s (thousandths): ln (1 - 0.7 s) (thousandths)
    pub ln_fade: Tab,
    pub ln_along: Tab,
    /// x^2 (-5000..5000) and the whole square root (0..999,999)
    pub sq: Tab,
    pub sqrt: Tab,
    /// the pyro: by d (thousandths, either sign) ln e^(-d^2); by s (thousandths) ln (1000 sqrt(1 - s)) and each
    /// colour's ln (1000 (3, 1.6 - 0.8 s, 0.5 - 0.4 s))
    pub ln_flame: Tab,
    pub ln_flame_row: Tab,
    pub ln_flame_col: [Tab; 3],
}

pub fn laser_tabs() -> LaserTabs {
    LaserTabs {
        ln_core: tab("core", (-9999..=9999).map(|d: i64| { let x = d.abs() as f64 / 10.0; lnf((1.6 * (-(x / 0.9).powi(2)).exp() + 0.25 * (-x / 5.0).exp()) * 1000.0).max(-300_000) }).collect()),
        ln_fade: tab("fade", (0..1_000_000).map(|iz: i64| { let z = if iz == 0 { 1e12 } else { KZ as f64 / iz as f64 / 100.0 }; lnf((-z / 500.0).exp() * 1000.0).max(-300_000) }).collect()),
        ln_along: tab("along", (0..=1000).map(|s: i64| lnf((1.0 - 0.7 * s as f64 / 1000.0) * 1000.0)).collect()),
        sq: tab("sq", (-5000..=5000i64).map(|x| x * x).collect()),
        sqrt: tab("sqrt", (0..1_000_000).map(|x| (x as f64).sqrt().round() as i64).collect()),
        ln_flame: tab("flame", (-9999..=9999).map(|d: i64| { let x = d as f64 / 1000.0; (-(x * x) * 10_000.0).round().max(-300_000.0) as i64 }).collect()),
        ln_flame_row: tab("flame_row", (0..=1000).map(|s: i64| { let x = (1.0 - s as f64 / 1000.0).sqrt() * 1000.0; if x < 0.5 { LN0 } else { lnf(x) } }).collect()),
        ln_flame_col: [
            tab("flame_r", (0..=1000).map(|_| lnf(3000.0)).collect()),
            tab("flame_g", (0..=1000).map(|s: i64| lnf((1.6 - 0.8 * s as f64 / 1000.0) * 1000.0)).collect()),
            tab("flame_b", (0..=1000).map(|s: i64| lnf((0.5 - 0.4 * s as f64 / 1000.0) * 1000.0)).collect()),
        ],
    }
}

/// The pyro: eight columns of flame on the stage's edge, a burst at the crest and every second bar, the flame's
/// width wavering, white-hot at the base going orange up, hidden behind what stands nearer.
/// (`lims`, when not empty: each pixel's depth past which the flame it got is hidden, for the party version's subjects.)
pub fn pyro<O: Ops>(o: &mut O, t: &Tables, bt: &LaserTabs, f: &mut Frame, cam: &Cam, sc: &Scene, sh: &Show, lims: &mut Vec<i64>) {
    let lev = sh.pyro;
    if s_lt(lev, 11, o) { return; }
    let k = s_sum(&[(s_div(s_lin(&[(lev, Some(4))], o), 10, o), 1)], 600, o);
    let h = s_div(s_lin(&[(1400, Some(k))], o), 1000, o);
    let llev = s_tab_o(&t.ln, 999_999, lev, o);
    for (i, q) in sc.pyro.iter().enumerate() {
        let top = [q[0], q[1], s_sum(&[(q[2], 1), (h, 1)], 0, o)];
        let cv = to_cam(o, t, cam, &[*q, top]);
        if s_lt(cv[0][2], 100, o) || s_lt(cv[1][2], 100, o) { continue; }
        let pr = project(o, t, &[cv[0], cv[1]]);
        // (held well off the screen: a column far outside it draws nothing)
        let hold = |x: i64, o: &mut O| s_clamp(x, -200_000, 200_000, o);
        let ((ax, ay, iza), (bx, by)) = ((hold(pr[0].0, o), hold(pr[0].1, o), pr[0].2), (hold(pr[1].0, o), hold(pr[1].1, o)));
        // the width: f 2.2 m / z (hundredths)
        let wid = s_min(s_div(22_000 * focal(), cv[0][2], o), 100_000, o);
        let x0 = s_div(s_sum(&[(s_min(ax, bx, o), 1), (wid, -3)], 0, o), 100, o).max(-scr_w() / 2);
        let x1 = s_div(s_sum(&[(s_max(ax, bx, o), 1), (wid, 3)], 0, o), 100, o).min(scr_w() / 2);
        let y0 = s_div(s_sum(&[(by, 1), (wid, -2)], 0, o), 100, o).max(-scr_h() / 2);
        let y1 = s_div(s_sum(&[(ay, 1), (wid, 1)], 0, o), 100, o).min(scr_h() / 2);
        if x1 <= x0 || y1 <= y0 { continue; }
        // the rows above ay + 2 px; on each: s = clip((ay - y) / (ay - by)), the flame's middle and width there
        let ys: V = (y0..y1).map(|y| 100 * y + 50).collect();
        let lim = s_sum(&[(ay, 1)], 200, o);
        let above = o.lt(v(&ys), K(lim));
        let rows: Vec<usize> = (0..ys.len()).filter(|&k| above[k] == 1).collect();
        if rows.is_empty() { continue; }
        let ys: V = rows.iter().map(|&k| ys[k]).collect();
        let span = s_sum(&[(ay, 1), (by, -1)], 0, o);
        if !s_lt(0, span, o) { continue; }
        let dy = o.sum(&[(v(&ys), -1)], ay);
        let s = { let (l_, ng) = lnv(o, t, &dy); let ls = s_tab_o(&t.ln, 999_999, span, o); let y = { let k_ = s_sum(&[(ls, -1)], LN1000, o); o.sum(&[(v(&l_), 1)], k_) }; let e = expv(o, t, &y, Some(&ng)); o.clamp(v(&e), 0, 1000) };
        let tt = s_lin(&[(sh.t12, Some(265_258)), (159_155 * i as i64, None)], o);
        let ang = { let a = o.lin(&[(v(&s), Some(K(1_432))), (K(tt), None)]); o.turns(v(&a)) };
        let (sn, _) = o.sincos(v(&ang));
        let dxl = s_sum(&[(bx, 1), (ax, -1)], 0, o);
        let mid = { let (ls, _) = lnv(o, t, &s); let ld = s_tab_o(&t.ln, 999_999, dxl, o); let nd = s_tab_o(&t.neg, 999_999, dxl, o); let y = { let k_ = s_sum(&[(ld, 1)], -LN1000, o); o.sum(&[(v(&ls), 1)], k_) }; let a = expv(o, t, &y, Some(&vec![nd; s.len()])); let w4 = o.lin(&[(v(&sn), Some(K(wid)))]); let w4 = o.div(v(&w4), K(3996), 0); o.sum(&[(v(&a), 1), (v(&w4), 1)], ax) };
        let wl = { let a = o.lin(&[(v(&s), Some(K(-7)))]); let a = o.div(v(&a), K(10), 0); let a = o.sum(&[(v(&a), 1)], 1000); let a = o.lin(&[(v(&a), Some(K(wid)))]); let a = o.div(v(&a), K(1000), 0); o.max(v(&a), K(50)) };
        // across each row: (x - mid) / wl, thousandths, from the row's first pixel by the step 100 000 / wl
        let n = x1 - x0;
        let first = { let a = o.sum(&[(v(&mid), -1)], 100 * x0 + 50); let (l_, ng) = lnv(o, t, &a); let lw = o.tabo(&t.ln, 999_999, v(&wl)); let y = o.sum(&[(v(&l_), 1), (v(&lw), -1)], LN1000); expv(o, t, &y, Some(&ng)) };
        let step = { let lw = o.tabo(&t.ln, 999_999, v(&wl)); let y = o.sum(&[(v(&lw), -1)], lnf(100_000.0)); let e = expv(o, t, &y, None); o.lin(&[(v(&e), Some(K(1000)))]) };
        let walks: Vec<(i64, i64, i64, i64)> = (0..ys.len()).map(|k| (n, first[k], step[k], 0)).collect();
        let dv = o.walka(&walks);
        let dv = o.clamp(v(&dv), -9999, 9999);
        let lg = o.tabo(&bt.ln_flame, 9999, v(&dv));
        let lrow = o.tab(&bt.ln_flame_row, v(&s));
        let px: Vec<usize> = (0..ys.len()).flat_map(|k| { let row = (ys[k] - 50) / 100; (0..n).map(move |c| ((row + scr_h() / 2) * scr_w() + x0 + c + scr_w() / 2) as usize) }).collect();
        let pz: V = px.iter().map(|&p| f.iz[p]).collect();
        let lim = s_div(s_lin(&[(iza, Some(100))], o), 98, o);
        let vis = o.lt(v(&pz), K(lim));
        let keep: Vec<usize> = (0..px.len()).filter(|&k| vis[k] == 1).collect();
        let kpx: Vec<usize> = keep.iter().map(|&k| px[k]).collect();
        if !lims.is_empty() { for &p in &kpx { lims[p] = lim; } }
        f.mark(&kpx);
        let lgk: V = keep.iter().map(|&k| lg[k]).collect();
        for c in 0..3 {
            let lc = o.tab(&bt.ln_flame_col[c], v(&s));
            let rowl = { let k_ = s_sum(&[(llev, 1)], -3 * LN1000 + LN10K, o); o.sum(&[(v(&lrow), 1), (v(&lc), 1)], k_) };
            let rk: V = keep.iter().map(|&k| rowl[k / n as usize]).collect();
            let y = o.sum(&[(v(&lgk), 1), (v(&rk), 1)], 0);
            let y = o.max(v(&y), K(-900_000));
            let a = expv(o, t, &y, None);
            add_to(o, &mut f.col[c], &kpx, &a);
        }
    }
}
