// What the night adds: the airship placed for the frame and the stadium's up-lights' spots on its hull (FrameAir);
// the lamps' light on what's shaded (the station's tubes, the street lights, our headlights, the bowl's wash); and
// over the shaded frame (train_night.py's post()): the airship's running lights, the up-lights' and its searchlights'
// beams, the stadium's searchlights, spots and lasers on the stadium's show clock, balloons, fireworks, photo flashes,
// the stars. Included after nmat.rs.

/// The airship this frame: its centre (film, bent, cm); the stadium's up-lights: their sources and the spots they
/// hold on the hull (bent, cm), their ways (thousandths).
pub struct FrameAir { pub blimp: [i64; 3], pub up_src: Vec<[i64; 3]>, pub up_spot: Vec<[i64; 3]>, pub up_dir: Vec<[i64; 3]>,
    /// each up-light's tables for the frame: a place's part (dm, -250..250) times its way's (dm), the along's
    /// (-433..433) times its way's parts back, and a normal's part (-127..127) times its way's (facing, thousandths)
    pub up_t: Vec<[Tab; 3]>, pub up_u: Vec<[Tab; 3]>, pub up_f: Vec<[Tab; 3]> }

/// Stadium-frame places (cm) into the film's bent frame (cm): x = C.x + X + BX, y = C.y + Z, z = C.z + Y.
pub fn aud_film<O: Ops>(o: &mut O, nw: &NightWorld, p: &[[i64; 3]]) -> Vec<[i64; 3]> {
    let ac = nw.aud_c;
    let g = |i: usize| -> V { p.iter().map(|q| q[i]).collect() };
    let x = o.lin(&[(v(&g(0)), None), (K(ac[0]), None), (K(BX), None)]);
    let y = o.lin(&[(v(&g(2)), None), (K(ac[1]), None)]);
    let z = o.lin(&[(v(&g(1)), None), (K(ac[2]), None)]);
    (0..p.len()).map(|k| [x[k], y[k], z[k]]).collect()
}

/// The airship's hull's surface (the model's: blimp_hull(u, v), u along -1..1 and v round, thousandths and mrad),
/// cm at the model's scale times 1.02 times 1.4.
fn hull_at<O: Ops>(o: &mut O, tb: &Tb, u: &V, vr: &V) -> [V; 3] {
    let n = u.len();
    let au = { let m = o.lin(&[(v(u), Some(K(-1)))]); o.max(v(u), v(&m)) };
    let p22 = o.tab(tb.t("pow22"), v(&au));
    let b0 = o.lin(&[(v(&p22), Some(K(-1))), (K(1000), None)]);
    let b0 = o.max(v(&b0), K(0));
    let base = o.tab(tb.t("pow055"), v(&b0));
    let ang = { let t = o.lin(&[(v(vr), Some(K(159_155)))]); let t = o.div(v(&t), K(1000), 0); o.turns(v(&t)) };
    let (up, side) = o.sincos(v(&ang));
    // the lobes: (0.5 + 0.5 cos(6.5 pi u))^2 max(0, up + 0.35) |side|^0.5
    let la = { let t = o.lin(&[(v(u), Some(K(3250)))]); o.turns(v(&t)) };
    let (_, lc) = o.sincos(v(&la));
    let l1 = o.lin(&[(v(&lc), Some(K(500))), (K(499_500), None)]);
    let l1 = o.div(v(&l1), K(999), 0);
    let l2 = o.lin(&[(v(&l1), Some(v(&l1)))]);
    let l2 = o.div(v(&l2), K(1000), 0);
    let u35 = o.lin(&[(v(&up), None), (K(350), None)]);
    let u35 = o.max(v(&u35), K(0));
    let asd = { let m = o.lin(&[(v(&side), Some(K(-1)))]); o.max(v(&side), v(&m)) };
    let sq = o.tab(tb.t("sqrtk"), v(&asd));
    let lobe = o.lin(&[(v(&l2), Some(v(&u35)))]);
    let lobe = o.div(v(&lobe), K(1000), 0);
    let lobe = o.lin(&[(v(&lobe), Some(v(&sq)))]);
    let lobe = o.div(v(&lobe), K(1000), 0);
    // r = 1 + 0.16 lobe - 0.10 max(0, -up)^2
    let dn = o.lin(&[(v(&up), Some(K(-1)))]);
    let dn = o.max(v(&dn), K(0));
    let d2 = o.lin(&[(v(&dn), Some(v(&dn)))]);
    let d2 = o.div(v(&d2), K(1000), 0);
    let r = o.lin(&[(v(&lobe), Some(K(16))), (v(&d2), Some(K(-10))), (K(100_000), None)]);
    let r = o.div(v(&r), K(100), 0);
    let br = o.lin(&[(v(&base), Some(v(&r)))]);
    let br = o.div(v(&br), K(1000), 0);
    // (55 u, 15 base r up (1.15 above, 0.85 below), 21 base r side) m, times 1.02 1.4 (cm: 142.8)
    let pos = o.lt(K(0), v(&up));
    let k = o.sel(&pos, K(115), K(85));
    let x = o.lin(&[(v(u), Some(K(55 * 1428)))]);
    let x = o.div(v(&x), K(10_000), 0);
    let y = o.lin(&[(v(&br), Some(v(&up)))]);
    let y = o.div(v(&y), K(1000), 0);
    let y = o.lin(&[(v(&y), Some(v(&k)))]);
    let y = o.div(v(&y), K(100), 0);
    let y = o.lin(&[(v(&y), Some(K(15 * 1428)))]);
    let y = o.div(v(&y), K(10_000), 0);
    let z = o.lin(&[(v(&br), Some(v(&side)))]);
    let z = o.div(v(&z), K(1000), 0);
    let z = o.lin(&[(v(&z), Some(K(21 * 1428)))]);
    let z = o.div(v(&z), K(10_000), 0);
    let _ = n;
    [x, y, z]
}

pub fn frame_air<O: Ops>(o: &mut O, nw: &NightWorld, f: i64) -> FrameAir {
    let tb = &nw.w.tb;
    let tenths = kl(o, &[(f, Some(10))]);
    let bp = blimp_at(o, nw, &[tenths]);
    let blimp: [i64; 3] = std::array::from_fn(|c| one({ let t = o.lin(&[(K(bp[c][0]), None), (K(if c == 0 { 10 * BX } else { 0 }), None)]); o.div(v(&t), K(10), 0) }));
    let srcs: Vec<[i64; 3]> = nw.uplights.iter().map(|u| [u[0], u[1], u[2]]).collect();
    let up_src = aud_film(o, nw, &srcs);
    // each spot sweeping along the hull (u = 0.85 sin(0.55 t + ph)) and round it (v = -pi/2 + 1.25 sin(0.4 t + 1.7 ph)),
    // t = f / 12 s
    let ph: V = nw.uplights.iter().map(|u| u[3]).collect();
    let a1 = o.lin(&[(K(f), Some(K(7294))), (v(&ph), Some(K(159)))]);
    let a1 = o.turns(v(&a1));
    let (s1, _) = o.sincos(v(&a1));
    let a2 = o.lin(&[(K(f), Some(K(5305))), (v(&ph), Some(K(271)))]);
    let a2 = o.turns(v(&a2));
    let (s2, _) = o.sincos(v(&a2));
    let u = { let t = o.lin(&[(v(&s1), Some(K(850)))]); o.div(v(&t), K(999), 0) };
    let vr = { let t = o.lin(&[(v(&s2), Some(K(1250)))]); let t = o.div(v(&t), K(999), 0); o.lin(&[(v(&t), None), (K(-1571), None)]) };
    let h = hull_at(o, tb, &u, &vr);
    let up_spot: Vec<[i64; 3]> = (0..srcs.len()).map(|k| [one(o.lin(&[(K(h[0][k]), None), (K(blimp[0]), None)])), one(o.lin(&[(K(h[1][k]), None), (K(blimp[1]), None)])), one(o.lin(&[(K(h[2][k]), None), (K(blimp[2]), None)]))]).collect();
    let d: [V; 3] = std::array::from_fn(|c| { let a: V = up_spot.iter().map(|p| p[c]).collect(); let b: V = up_src.iter().map(|p| p[c]).collect(); o.lin(&[(v(&a), None), (v(&b), Some(K(-1)))]) });
    let dn = nrm3(o, &d[0], &d[1], &d[2]);
    let up_dir: Vec<[i64; 3]> = (0..srcs.len()).map(|k| [dn[0][k], dn[1][k], dn[2][k]]).collect();
    let r250: V = (-250..=250).collect();
    let r433: V = (-433..=433).collect();
    // (the facing's parts turned: the normal's parts from 127 down to -127)
    let r127t: V = (-127..=127).rev().collect();
    let tab = |o: &mut O, x: &V, k: i64, d: i64| -> Tab { let t = o.lin(&[(v(x), Some(K(k)))]); Tab { name: "up", data: o.div(v(&t), K(d), 0), w: 0 } };
    let up_t = up_dir.iter().map(|dv| std::array::from_fn(|i| tab(o, &r250, dv[i], 1000))).collect();
    let up_u = up_dir.iter().map(|dv| std::array::from_fn(|i| tab(o, &r433, dv[i], 1000))).collect();
    let up_f = up_dir.iter().map(|dv| std::array::from_fn(|i| tab(o, &r127t, dv[i], 100))).collect();
    FrameAir { blimp, up_src, up_spot, up_dir, up_t, up_u, up_f }
}

// ---- the lamps -----------------------------------------------------------------------------------------------------

/// The lamps' light on what's shaded (lights()): for each 2x2 block's first pixel shaded (the rest of the block take
/// it), the station's four lines of tubes (each the nearest point of its line, falling off past the platforms' ends),
/// the street lights within 40 m along the line (those within 500 m of the camera), our two headlights' cones, and
/// inside the bowl its rig's wash, a colour a pulse; each light's term by logarithms (the projection of the way to it
/// on the normal over its distance's function, from the printed tables), as the night has come. Lights farther than
/// 100 m each way (the headlights 150 m) are left out: what they would add is under half a step of R. R exposed.
pub fn lamps<O: Ops>(o: &mut O, s: &NightShade, px: &Px, nn: &[V; 3], kinds: &[i64], tunnel: &V, solid: &[usize]) -> [V; 3] {
    let nw = s.nw;
    let tb = &nw.w.tb;
    let l = s.l;
    let n = px.len();
    let mut out: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    // the blocks' first pixels (not in the tunnel; not the screens and lamps, which give their own light)
    let lit_kind = |k: i64| !(k == nkind::DSCREEN || k == nkind::BSCREEN || k == nkind::LED || k == nkind::NAV || k == nkind::TUBE);
    let mut rep: std::collections::BTreeMap<usize, usize> = Default::default();
    for i in 0..n { if tunnel[i] == 0 && lit_kind(kinds[i]) { let q = solid[i]; let blk = (q / (2 * W() as usize)) * W() as usize + (q % W() as usize) / 2; rep.entry(blk).or_insert(i); } }
    let reps: Vec<usize> = rep.values().copied().collect();
    if reps.is_empty() { return out; }
    let m = reps.len();
    let g = |v: &V| -> V { reps.iter().map(|&i| v[i]).collect() };
    let pc: [V; 3] = [g(&px.p[0]), g(&px.p[1]), g(&px.p[2])];
    let nr: [V; 3] = [g(&nn[0]), g(&nn[1]), g(&nn[2])];
    // places in decimetres; their parts times the normal's (dm hundredths), and the dot
    let pd: [V; 3] = std::array::from_fn(|c| o.div(v(&pc[c]), K(10), 0));
    let pnc: [V; 3] = std::array::from_fn(|c| mulw(o, v(&pd[c]), v(&nr[c]), 0));
    let pn = o.lin(&[(v(&pnc[0]), None), (v(&pnc[1]), None), (v(&pnc[2]), None)]);
    // a light term's constant, by the frame's exposure and night (ln of k e n / 1e6, ten-thousandths)
    let lnk = |o: &mut O, k: i64| -> i64 {
        let t = fk(o, l.e, l.night, 1000);
        let t = one(o.max(K(t), K(1)));
        let lt = one(o.tab(tb.t("ln6"), K(t)));
        let lk = one(o.tab(tb.t("ln6"), K(k / 1000)));
        one(o.sum(&[(K(lt), 1), (K(lk), 1)], -69_078))
    };
    // the terms of pairs (a rep, a light's place, the projection vn of the way to it on the normal), summed per rep
    // (R hundredths): the squared distance from the printed squares (dm^2; m^2 past 100 m), its function's logarithm,
    // e^(ln vn + c - that)
    let terms = |o: &mut O, at: &[usize], q: &[[i64; 3]], vn: &V, ltab: (&str, &str), c: i64| -> V {
        if at.is_empty() { return vec![0; m]; }
        let fwd = o.lt(K(0), v(vn));
        let live: Vec<usize> = (0..at.len()).filter(|&k| fwd[k] == 1).collect();
        if live.is_empty() { return vec![0; m]; }
        let vv: [V; 3] = std::array::from_fn(|i| { let qq: V = live.iter().map(|&k| q[k][i]).collect(); let pp: V = live.iter().map(|&k| pd[i][at[k]]).collect(); o.sum(&[(v(&qq), 1), (v(&pp), -1)], 0) });
        let sq: [V; 3] = std::array::from_fn(|i| o.tabo(tb.t("sq9999"), 9999, v(&vv[i])));
        let d2 = o.lin(&[(v(&sq[0]), None), (v(&sq[1]), None), (v(&sq[2]), None)]);
        let near = o.lt(v(&d2), K(1_000_000));
        let mut lt_ = vec![0; live.len()];
        let a_: Vec<usize> = (0..live.len()).filter(|&k| near[k] == 1).collect();
        if !a_.is_empty() { let idx: V = a_.iter().map(|&k| d2[k]).collect(); let r = o.tab(tb.t(ltab.0), v(&idx)); for (j, &k) in a_.iter().enumerate() { lt_[k] = r[j]; } }
        let b_: Vec<usize> = (0..live.len()).filter(|&k| near[k] == 0).collect();
        if !b_.is_empty() {
            let vm: [V; 3] = std::array::from_fn(|i| { let t: V = b_.iter().map(|&k| vv[i][k]).collect(); o.div(v(&t), K(10), 0) });
            let sqm: [V; 3] = std::array::from_fn(|i| o.tabo(tb.t("sq9999"), 9999, v(&vm[i])));
            let d2m = o.lin(&[(v(&sqm[0]), None), (v(&sqm[1]), None), (v(&sqm[2]), None)]);
            let r = o.tab(tb.t(ltab.1), v(&d2m));
            for (j, &k) in b_.iter().enumerate() { lt_[k] = r[j]; }
        }
        let vl: V = live.iter().map(|&k| vn[k]).collect();
        let vl = o.min(v(&vl), K(999_999));
        let a = o.tab(tb.t("ln6"), v(&vl));
        let b = o.sum(&[(v(&lt_), 1), (K(c), -1)], 0);
        let k = o.ldiff(&a, &b, -50, 115);
        let r = o.tabo(tb.t("exp6"), 50_000, v(&k));
        let mut lists: Vec<Vec<i64>> = vec![vec![]; m];
        for (j, &k) in live.iter().enumerate() { lists[at[k]].push(r[j]); }
        group_sum(o, &lists)
    };
    let mut sums: Vec<V> = vec![];
    // the tubes: four lines 7.6 m up at x (cm) -830, -395, 825, 1260 along the platforms; the nearest point's z the
    // place's held to them; past their ends falling off as 1 + 0.15 m a metre. The blocks within 100 m of them.
    {
        let zc = o.clamp(v(&pc[2]), PL0, PL1);
        let dz = o.sum(&[(v(&pc[2]), 1), (v(&zc), -1)], 0);
        let dzd = o.div(v(&dz), K(10), 0);
        let dzn = mulw(o, v(&dzd), v(&nr[2]), 0);
        let adz = { let mm = o.sum(&[(v(&dzd), -1)], 0); o.max(v(&dzd), v(&mm)) };
        let adz = o.min(v(&adz), K(9999));
        let att = o.tab(tb.t("lnatt"), v(&adz));
        let zcd = o.div(v(&zc), K(10), 0);
        let c1 = lnk(o, 40_000);
        let mut tot = vec![0; m];
        for xl in [-830, -395, 825, 1260] {
            let qx = (xl + BX) / 10;
            let at: Vec<usize> = (0..m).filter(|&i| (pd[0][i] - qx).abs() < 1000 && (pd[1][i] - 76).abs() < 1000 && dzd[i].abs() < 1000).collect();
            if at.is_empty() { continue; }
            // vn = (q - p).n: qx nx + 76 ny (the tube's tables of the normal's parts) - (px nx + py ny) - dz nz
            let ga = |v_: &V| -> V { at.iter().map(|&i| v_[i]).collect() };
            let tx = o.tabo(&nw.tube_qn[match xl { -830 => 0, -395 => 1, 825 => 2, _ => 3 }], 127, v(&ga(&nr[0])));
            let ty = o.tabo(&nw.tube_y, 127, v(&ga(&nr[1])));
            let vn = o.lin(&[(v(&tx), None), (v(&ty), None), (v(&ga(&pnc[0])), Some(K(-1))), (v(&ga(&pnc[1])), Some(K(-1))), (v(&ga(&dzn)), Some(K(-1)))]);
            let q: Vec<[i64; 3]> = at.iter().map(|&i| [qx, 76, zcd[i]]).collect();
            let t = terms(o, &at, &q, &vn, ("lnL1a", "lnL1b"), c1);
            // the fall-off past the ends: its logarithm taken off
            let lit: Vec<usize> = (0..m).filter(|&i| t[i] > 0).collect();
            if lit.is_empty() { continue; }
            let gl = |v_: &V| -> V { lit.iter().map(|&i| v_[i]).collect() };
            let la = o.tab(tb.t("ln6"), v(&gl(&t)));
            let k2 = o.ldiff(&la, &gl(&att), -50, 115);
            let t2 = o.tabo(tb.t("exp6"), 50_000, v(&k2));
            let cur = gl(&tot);
            let s2 = o.sum(&[(v(&cur), 1), (v(&t2), 1)], 0);
            for (j, &i) in lit.iter().enumerate() { tot[i] = s2[j]; }
        }
        sums.push(tot);
    }
    // the street lights: within 40 m along of the place (and 100 m across and up), within 500 m of the camera
    {
        let lamps_near: Vec<usize> = (0..nw.lamps.len()).filter(|&k| (nw.lamps[k][2] - s.cam.eye_cm[2]).abs() < 50_000).collect();
        let mut at = vec![];
        let mut q = vec![];
        let mut tq: [V; 3] = [vec![], vec![], vec![]];
        for i in 0..m {
            for &k in &lamps_near {
                let ld = nw.lamps_dm[k];
                if (ld[2] - pd[2][i]).abs() < 400 && (ld[0] - pd[0][i]).abs() < 1000 && (ld[1] - pd[1][i]).abs() < 1000 {
                    at.push(i); q.push(ld);
                    for c in 0..3 { tq[c].push(nw.lamp_qn[k][c][(nr[c][i] + 127) as usize]); }
                }
            }
        }
        let c2 = lnk(o, 350_000);
        let t = if at.is_empty() { vec![0; m] } else {
            let pnp: V = at.iter().map(|&i| pn[i]).collect();
            let vn = o.lin(&[(v(&tq[0]), None), (v(&tq[1]), None), (v(&tq[2]), None), (v(&pnp), Some(K(-1)))]);
            terms(o, &at, &q, &vn, ("lnL2a", "lnL2b"), c2)
        };
        sums.push(t);
    }
    // our headlights: two cones ahead from the cab's nose (where it stops), (cos)^30 about their axis
    {
        let za = nw.path.za[l.f.clamp(0, F_STOP) as usize];
        let zq = one({ let t = o.div(K(za), K(100), 0); o.lin(&[(v(&t), None), (K(44), None)]) });
        let c3 = lnk(o, 2_250_000);
        let parts: V = (-127..=127).collect();
        let mut tot = vec![0; m];
        // within 150 m up and down of them (7 dm up), and ahead of them along within 150 m: zq < z < zq + 1500, on
        // the machines for those
        let up: Vec<usize> = (0..m).filter(|&i| pd[1][i] > 7 - 1500 && pd[1][i] < 7 + 1500).collect();
        let ahead = { let zs: V = up.iter().map(|&i| pd[2][i]).collect(); o.lt(K(zq), v(&zs)) };
        let up: Vec<usize> = (0..up.len()).filter(|&j| ahead[j] == 1).map(|j| up[j]).collect();
        let reach = kl(o, &[(zq, None), (1500, None)]);
        let short = { let zs: V = up.iter().map(|&i| pd[2][i]).collect(); o.lt(v(&zs), K(reach)) };
        let along: Vec<usize> = (0..up.len()).filter(|&j| short[j] == 1).map(|j| up[j]).collect();
        for hx in [-90, 90] {
            let qq = [(BX + hx) / 10, 7, zq];
            // and within 150 m across
            let at: Vec<usize> = along.iter().copied().filter(|&i| pd[0][i] > qq[0] - 1500 && pd[0][i] < qq[0] + 1500).collect();
            if at.is_empty() { continue; }
            let ga = |v_: &V| -> V { at.iter().map(|&i| v_[i]).collect() };
            // q.n by the frame's tables of the light's parts times a normal's
            let hq: Vec<Tab> = (0..3).map(|c| Tab { name: "hq", data: o.lin(&[(v(&parts), Some(K(qq[c])))]), w: 0 }).collect();
            let tq: Vec<V> = (0..3).map(|c| o.tabo(&hq[c], 127, v(&ga(&nr[c])))).collect();
            let vn = o.lin(&[(v(&tq[0]), None), (v(&tq[1]), None), (v(&tq[2]), None), (v(&ga(&pn)), Some(K(-1)))]);
            let q: Vec<[i64; 3]> = vec![qq; at.len()];
            let t = terms(o, &at, &q, &vn, ("lnL3a", "lnL3b"), c3);
            let lit: Vec<usize> = (0..m).filter(|&i| t[i] > 0).collect();
            if lit.is_empty() { continue; }
            let gl = |v_: &V| -> V { lit.iter().map(|&i| v_[i]).collect() };
            // the cone: ((p - q).fwd / |p - q|)^30, fwd = (0, -0.05, 1) as a unit: its logarithm 30 (ln w - ln 1000 - ln d)
            let w_ = o.lin(&[(v(&gl(&pd[1])), Some(K(-50))), (v(&gl(&pd[2])), Some(K(999))), (K(qq[1]), Some(K(50))), (K(qq[2]), Some(K(-999)))]);
            // ln d (m): from the squared distance in dm^2 (less ln 10), held to 100 m
            let d2: [V; 3] = std::array::from_fn(|i| { let t_ = o.sum(&[(v(&gl(&pd[i])), 1), (K(qq[i]), -1)], 0); o.tabo(tb.t("sq9999"), 9999, v(&t_)) });
            let d2 = o.lin(&[(v(&d2[0]), None), (v(&d2[1]), None), (v(&d2[2]), None)]);
            let d2 = o.clamp(v(&d2), 1, 999_999);
            let hl = o.tab(tb.t("lnhalf"), v(&d2));
            let hl = o.sum(&[(v(&hl), 1)], -23_026);
            let wm = o.clamp(v(&w_), 10, 9_999_999);
            let wm = o.div(v(&wm), K(10), 0);
            let lw = o.tab(tb.t("ln6"), v(&wm));
            let lc = o.lin(&[(v(&lw), Some(K(30))), (v(&hl), Some(K(-30))), (K(-30 * 69_078), None)]);
            let lt_ = o.tab(tb.t("ln6"), v(&gl(&t)));
            let tot_l = o.lin(&[(v(&lt_), None), (v(&lc), None)]);
            let tot_l = o.clamp(v(&tot_l), -999_000, 999_999);
            let zz = vec![0; lit.len()];
            let k = o.ldiff(&tot_l, &zz, -50, 115);
            let r = o.tabo(tb.t("exp6"), 50_000, v(&k));
            let cur = gl(&tot);
            let s2 = o.sum(&[(v(&cur), 1), (v(&r), 1)], 0);
            for (j, &i) in lit.iter().enumerate() { tot[i] = s2[j]; }
        }
        sums.push(tot);
    }
    // the bowl: inside the stadium's wall and under 32 m, its rig's colour of the pulse, 0.45 + 0.55 n.y, times
    // 0.25 + 1.6 the beat
    let ac = nw.aud_c;
    // (across the bowl in decimetres: thousandths of its half widths 1346 and 1146 dm as 0.743 and 0.873 a dm)
    let cxd = one({ let t = o.lin(&[(K(ac[0]), None), (K(BX), None)]); o.div(v(&t), K(10), 0) });
    let cyd = one(o.div(K(ac[2]), K(10), 0));
    let bx = o.sum(&[(v(&pd[0]), 1), (K(cxd), -1)], 0);
    let by = o.sum(&[(v(&pd[2]), 1), (K(cyd), -1)], 0);
    let bz = o.lin(&[(v(&pc[1]), None), (K(ac[1]), Some(K(-1)))]);
    let ex = { let t = o.lin(&[(v(&bx), Some(K(743)))]); o.div(v(&t), K(1000), 0) };
    let ey = { let t = o.lin(&[(v(&by), Some(K(873)))]); o.div(v(&t), K(1000), 0) };
    let ex = o.clamp(v(&ex), -9999, 9999);
    let ey = o.clamp(v(&ey), -9999, 9999);
    let e2 = { let a = o.tabo(tb.t("sq9999"), 9999, v(&ex)); let b = o.tabo(tb.t("sq9999"), 9999, v(&ey)); o.lin(&[(v(&a), None), (v(&b), None)]) };
    let in1 = o.lt(v(&e2), K(940_900));
    let in2 = o.lt(v(&bz), K(3200));
    let inside = mulw(o, v(&in1), v(&in2), 0);
    let ny = o.lin(&[(v(&nr[1]), Some(K(10)))]);
    let ny = o.clamp(v(&ny), 0, 1000);
    let upk = o.lin(&[(v(&ny), Some(K(550))), (K(450_000), None)]);
    let upk = o.div(v(&upk), K(1000), 0);
    let bkk = one(o.lin(&[(K(l.beat), Some(K(1600))), (K(250_000), None)]));
    let bkk = one(o.div(K(bkk), K(1000), 0));
    let pal = [[1000, 250, 800], [250, 800, 1000], [1000, 700, 200], [500, 350, 1000]][nw_pulse_count(o, l.f) as usize];
    let bowl_k = |o: &mut O, c: usize| -> V {
        // R hundredths: up bk pal 250 e n
        let k = fk(o, bkk, pal[c], 1000);
        let k = fk(o, k, 25_000, 1000);
        let k = fk(o, k, l.e, 1000);
        let k = fk(o, k, l.night, 1000);
        let t = o.lin(&[(v(&upk), Some(K(k)))]);
        let t = o.div(v(&t), K(1000), 0);
        mulw(o, v(&t), v(&inside), 0)
    };
    if let Ok(only) = std::env::var("NIGHT_LAMPS") { for (k, sm) in sums.iter_mut().enumerate() { if !only.contains(&k.to_string()) { *sm = vec![0; m]; } } }
    // the classes' colours: tubes (1, 0.97, 0.9), street lights (1, 0.82, 0.55), headlights (0.9, 0.95, 1)
    const COL: [[i64; 3]; 3] = [[1000, 970, 900], [1000, 820, 550], [900, 950, 1000]];
    let mut rv: [V; 3] = [vec![0; m], vec![0; m], vec![0; m]];
    for c in 0..3 {
        let terms: Vec<(A, Option<A>)> = (0..3).map(|k| (v(&sums[k]), Some(K(COL[k][c])))).collect();
        let t = o.lin(&terms);
        let t = o.div(v(&t), K(1000), 0);
        let b = bowl_k(o, c);
        let t = o.sum(&[(v(&t), 1), (v(&b), 1)], 0);
        let t = o.div(v(&t), K(100), 0);
        rv[c] = o.min(v(&t), K(999));
    }
    // the rest of each block take its first's
    let by_blk: std::collections::HashMap<usize, usize> = reps.iter().enumerate().map(|(j, &i)| { let q = solid[i]; ((q / (2 * W() as usize)) * W() as usize + (q % W() as usize) / 2, j) }).collect();
    for i in 0..n {
        if tunnel[i] != 0 || !lit_kind(kinds[i]) { continue; }
        let q = solid[i];
        let blk = (q / (2 * W() as usize)) * W() as usize + (q % W() as usize) / 2;
        if let Some(&j) = by_blk.get(&blk) { for c in 0..3 { out[c][i] = rv[c][j]; } }
    }
    out
}

/// Which pulse of the bowl's rig it is (its colour, of four): the beats 1 and 3 counted on the show's clock, mod 4.
fn nw_pulse_count<O: Ops>(o: &mut O, f: i64) -> i64 {
    let m = one(o.lin(&[(K(f), Some(K(1000))), (K(-1000 * (F_END - 36) - BEAT_T0 + 1000 * 10_500), None)]));
    let q = one(o.div(K(m), K(10_500), 0));
    let r = one(o.div(K(q), K(4), 0));
    one(o.lin(&[(K(q), None), (K(r), Some(K(-4)))]))
}

// ---- over the shaded frame -------------------------------------------------------------------------------------------

/// Places (film, bent, cm) on the screen: hundredths of a pixel from the left and from the top, the depth along the
/// view (cm) and K over it (as the depth buffer's), and whether in front (30 cm on).
pub struct Proj { pub x: V, pub y: V, pub z: V, pub iz: V, pub front: Vec<bool> }

pub fn proj<O: Ops>(o: &mut O, cam: &Cam, pts: &[[i64; 3]]) -> Proj {
    let n = pts.len();
    if n == 0 { return Proj { x: vec![], y: vec![], z: vec![], iz: vec![], front: vec![] }; }
    let d: [V; 3] = std::array::from_fn(|i| { let a: V = pts.iter().map(|p| p[i]).collect(); o.lin(&[(v(&a), None), (K(cam.eye_cm[i]), Some(K(-1)))]) });
    let c: [V; 3] = std::array::from_fn(|r| { let t = o.lin(&[(v(&d[0]), Some(K(cam.b[r][0]))), (v(&d[1]), Some(K(cam.b[r][1]))), (v(&d[2]), Some(K(cam.b[r][2])))]); o.div(v(&t), K(1000), 0) });
    let front: Vec<bool> = o.lt(K(30), v(&c[2])).iter().map(|&f| f == 1).collect();
    // the host's scaling of the far ones (by tens, until the depth fits a divisor: each depth compared with 100,000,
    // 1,000,000, ...), and K over the power of ten from the printed table
    let tens: Vec<(i64, usize)> = c[2].iter().map(|&z| tens_under(z, 100_000)).collect();
    let sc: V = tens.iter().map(|t| t.0).collect();
    let z1 = o.max(v(&c[2]), K(30));
    let zs = o.div(v(&z1), v(&sc), 0);
    let zs = o.max(v(&zs), K(1));
    let xs = o.div(v(&c[0]), v(&sc), 0);
    let ys = o.div(v(&c[1]), v(&sc), 0);
    let x = { let t = o.lin(&[(v(&xs), Some(K(cam.f2)))]); let t = sdiv(o, &t, &zs); let t = o.lin(&[(v(&t), Some(K(50))), (K(100 * W() / 2), None)]); o.clamp(v(&t), -999_000, 999_999) };
    let nf2 = kl(o, &[(cam.f2, Some(-1))]);
    let y = { let t = o.lin(&[(v(&ys), Some(K(nf2)))]); let t = sdiv(o, &t, &zs); let t = o.lin(&[(v(&t), Some(K(50))), (K(100 * H() / 2), None)]); o.clamp(v(&t), -999_000, 999_999) };
    let kz: V = tens.iter().map(|t| KZ_TENS[t.1]).collect();
    let iz = o.div(v(&kz), v(&zs), 0);
    Proj { x, y, z: z1, iz, front }
}

/// K over 1, 10, 100, ... (printed).
pub const KZ_TENS: [i64; 8] = [29_970_000, 2_997_000, 299_700, 29_970, 2_997, 299, 29, 2];

/// t / z held to +-90,000 (z 1 or more; t any): the size against z 90,000 compared first.
pub fn sdiv<O: Ops>(o: &mut O, t: &V, z: &V) -> V {
    let m = o.lin(&[(v(t), Some(K(-1)))]);
    let neg = o.lt(v(t), K(0));
    let a = o.sel(&neg, v(&m), v(t));
    let ad = o.div(v(&a), K(1000), 0);
    let bound = o.lin(&[(v(z), Some(K(90)))]);
    let over = o.lt(v(&bound), v(&ad));
    let a = o.sel(&over, K(0), v(&a));
    let r = o.div(v(&a), v(z), 0);
    let r = o.sel(&over, K(90_000), v(&r));
    let mr = o.lin(&[(v(&r), Some(K(-1)))]);
    o.sel(&neg, v(&mr), v(&r))
}

/// Light to add (R tenths, a long a pixel and channel) and alpha-laid colours, gathered as the post goes.
pub struct Over { pub add: [V; 3], pub touched: Vec<bool> }

impl Over {
    pub fn new() -> Over { Over { add: [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]], touched: vec![false; N_PIX()] } }
    /// Add values at pixels (the host's carrying; the sums machines').
    pub fn put<O: Ops>(&mut self, o: &mut O, px: &[usize], vals: &[V; 3]) {
        if px.is_empty() { return; }
        // in rounds so each round meets a pixel once
        let mut round = vec![0usize; px.len()];
        let mut seen: std::collections::HashMap<usize, usize> = Default::default();
        for (k, &p) in px.iter().enumerate() { let e = seen.entry(p).or_insert(0); round[k] = *e; *e += 1; }
        let rounds = round.iter().copied().max().unwrap_or(0) + 1;
        for rd in 0..rounds {
            let at: Vec<usize> = (0..px.len()).filter(|&k| round[k] == rd).collect();
            for c in 0..3 {
                let cur: V = at.iter().map(|&k| self.add[c][px[k]]).collect();
                let add: V = at.iter().map(|&k| vals[c][k]).collect();
                // (held to 999,000: what's added held to the room left)
                let room = o.sum(&[(v(&cur), -1)], 999_000);
                let add = o.min(v(&add), v(&room));
                let s = o.sum(&[(v(&cur), 1), (v(&add), 1)], 0);
                for (j, &k) in at.iter().enumerate() { self.add[c][px[k]] = s[j]; }
            }
            for &k in &at { self.touched[px[k]] = true; }
        }
    }
    /// The added light into the frame's colours (cards and the part past 999), then cleared.
    pub fn apply<O: Ops>(&mut self, o: &mut O, col: &mut [V; 3], hot: &mut [V; 3]) {
        let px: Vec<usize> = (0..N_PIX()).filter(|&p| self.touched[p]).collect();
        if std::env::var("NIGHT_BEAMS").is_ok() { eprintln!("apply {} pixels, max add {:?}, over 100: {}", px.len(), px.iter().map(|&p| self.add[0][p]).max(), px.iter().filter(|&&p| self.add[0][p] > 100).count()); }
        if px.is_empty() { return; }
        for c in 0..3 {
            // (only where a tenth or more is added: less leaves the pixel as it is)
            let a0: V = px.iter().map(|&p| self.add[c][p]).collect();
            let big = o.lt(K(9), v(&a0));
            let sel: Vec<usize> = (0..px.len()).filter(|&j| big[j] == 1).collect();
            for &p in &px { self.add[c][p] = 0; }
            if sel.is_empty() { continue; }
            let a: V = sel.iter().map(|&j| a0[j]).collect();
            let px: Vec<usize> = sel.iter().map(|&j| px[j]).collect();
            let cc: V = px.iter().map(|&p| col[c][p]).collect();
            let hh: V = px.iter().map(|&p| hot[c][p]).collect();
            // the card and what passes 999 into the part past it (in 25ths: times 4 over 100)
            let a10 = o.div(v(&a), K(10), 0);
            let t = o.sum(&[(v(&cc), 1), (v(&a10), 1)], 0);
            let cn = o.min(v(&t), K(999));
            let over = o.sum(&[(v(&t), 1), (v(&cn), -1)], 0);
            let over = o.min(v(&over), K(99_999));
            let q = o.sum(&[(v(&over), 4)], 0);
            let q = o.div(v(&q), K(100), 0);
            let hn = o.sum(&[(v(&hh), 1), (v(&q), 1)], 0);
            let hn = o.min(v(&hn), K(999));
            for (j, &p) in px.iter().enumerate() { col[c][p] = cn[j]; hot[c][p] = hn[j]; self.add[c][p] = 0; }
        }
        for p in px { self.touched[p] = false; }
    }
}

/// Places in the camera's space (cm) from places (film, bent, cm).
fn cam_space<O: Ops>(o: &mut O, cam: &Cam, p: [i64; 3]) -> [i64; 3] {
    let d: [i64; 3] = std::array::from_fn(|i| one(o.lin(&[(K(p[i]), None), (K(cam.eye_cm[i]), Some(K(-1)))])));
    std::array::from_fn(|r| one({ let s = o.lin(&[(K(d[0]), Some(K(cam.b[r][0]))), (K(d[1]), Some(K(cam.b[r][1]))), (K(d[2]), Some(K(cam.b[r][2])))]); o.div(v(&s), K(1000), 0) }))
}

/// A camera-space place (cm, in front) on the screen: hundredths of a pixel from the left and the top, K / z.
fn on_screen<O: Ops>(o: &mut O, cam: &Cam, c: [i64; 3]) -> (i64, i64, i64) {
    // (the host's scaling of a far one by tens, until its depth fits a divisor: compared with 100,000, 1,000,000, ...)
    let (s, sk) = tens_under(c[2], 100_000);
    let q = o.div(v(&vec![c[0], c[1], c[2]]), K(s), 0);
    let z = one(o.max(K(q[2]), K(1)));
    let x = one({ let t = o.lin(&[(K(q[0]), Some(K(cam.f2)))]); let t = o.div(v(&t), K(z), 0); let t = o.clamp(v(&t), -9_000_000, 9_000_000); o.lin(&[(v(&t), Some(K(50))), (K(100 * W() / 2), None)]) });
    let nf2 = kl(o, &[(cam.f2, Some(-1))]);
    let y = one({ let t = o.lin(&[(K(q[1]), Some(K(nf2)))]); let t = o.div(v(&t), K(z), 0); let t = o.clamp(v(&t), -9_000_000, 9_000_000); o.lin(&[(v(&t), Some(K(50))), (K(100 * H() / 2), None)]) });
    let iz = one(o.div(K(KZ_TENS[sk]), K(z), 0));
    (x, y, iz)
}

/// A beam of light through the haze (the prototype's beam(): a soft line from a to b, its width w0 to w1 (cm) along
/// it, its colour (thousandths) times inten (thousandths of 1.0), fading along it as (1 - s)^1.5 and as its width's
/// 0.3th power in pixels; behind what's in front of it): its ends in the camera's space (cut at the near plane), on
/// the screen; along its longer axis at each pixel's middle (every second where it is wider than 4 pixels, those
/// filling 2x2 blocks), its place across, its depth and the place along the beam by the perspective, its width and
/// fading there; the pixels within twice its width, each e^-(d / w)^2.
pub fn beam<O: Ops>(o: &mut O, nw: &NightWorld, cam: &Cam, g: &GBuf, a: [i64; 3], b: [i64; 3], w0: i64, w1: i64, col: [i64; 3], inten: i64, e: i64, ov: &mut Over) {
    let tb = &nw.w.tb;
    if inten <= 0 { return; }
    let (mut ca, mut cb) = (cam_space(o, cam, a), cam_space(o, cam, b));
    let (mut wa, mut wb) = (w0, w1);
    let (mut sa, mut sb) = (0, 1000);
    // cut to the near plane (z 50 cm) and a guard band six times as wide as the depth each way: each plane's value
    // at the ends, the part inside kept (its ends' places, widths and places along the beam carried)
    for (kx, ky, kz, c0) in [(0i64, 0i64, 1i64, -50i64), (-1, 0, 6, 0), (1, 0, 6, 0), (0, -1, 6, 0), (0, 1, 6, 0)] {
        let pv = o.lin(&[(v(&vec![ca[0], cb[0]]), Some(K(kx))), (v(&vec![ca[1], cb[1]]), Some(K(ky))), (v(&vec![ca[2], cb[2]]), Some(K(kz))), (K(c0), None)]);
        let neg = o.lt(v(&pv), K(0));
        if neg[0] == 1 && neg[1] == 1 { return; }
        if neg[0] == 0 && neg[1] == 0 { continue; }
        // t = pa / (pa - pb) from a toward b, thousandths (by tenths of the values, so they fit a division)
        let den = one(o.lin(&[(K(pv[0]), None), (K(pv[1]), Some(K(-1)))]));
        let (num, den) = if den < 0 { (one(o.lin(&[(K(pv[0]), Some(K(-1)))])), one(o.lin(&[(K(den), Some(K(-1)))]))) } else { (pv[0], den) };
        let sc = tens_under(den, 100_000).0;
        let nd = o.div(v(&vec![num, den]), K(sc), 0);
        let t = one(o.div(K(nd[0]), K(nd[1]), 1));
        let t = one(o.clamp(K(t), 0, 1000));
        let p: [i64; 3] = std::array::from_fn(|i| one(lerpv(o, K(ca[i]), K(cb[i]), K(t))));
        let wp = one(lerpv(o, K(wa), K(wb), K(t)));
        let sp = one(lerpv(o, K(sa), K(sb), K(t)));
        if neg[0] == 1 { ca = p; wa = wp; sa = sp; } else { cb = p; wb = wp; sb = sp; }
    }
    let (ax, ay, iza) = on_screen(o, cam, ca);
    let (bx, by, izb) = on_screen(o, cam, cb);
    // (held to the screen's reach, a long: a cut end may land just past the guard band)
    let xy = o.clamp(v(&vec![ax, ay, bx, by]), -999_000, 999_000);
    let (ax, ay, bx, by) = (xy[0], xy[1], xy[2], xy[3]);
    let dd = o.lin(&[(v(&vec![bx, by]), None), (v(&vec![ax, ay]), Some(K(-1)))]);
    let nd = o.lin(&[(v(&dd), Some(K(-1)))]);
    let ngf = o.lt(v(&dd), K(0));
    let ad = o.sel(&ngf, v(&nd), v(&dd));
    let ymaj = one(o.lt(K(ad[0]), K(ad[1]))) == 1;
    let (u0, u1, m0, m1, adu) = if ymaj { (ay, by, ax, bx, ad[1]) } else { (ax, bx, ay, by, ad[0]) };
    if adu == 0 { return; }
    // the slope's cosine (thousandths): the longer part over the length
    let (big, small) = if ymaj { (ad[1], ad[0]) } else { (ad[0], ad[1]) };
    let un = nrm3(o, &vec![big], &vec![small], &vec![0]);
    let cosf = one(o.clamp(v(&un[0]), 1, 1000));
    // the steps: pixel middles along the longer axis from fl(lo + 49, 100) to short of fl(hi + 49, 100)
    let lohi = { let a_ = o.min(K(u0), K(u1)); let b_ = o.max(K(u0), K(u1)); let t = o.lin(&[(v(&vec![a_[0], b_[0]]), None), (K(49), None)]); o.div(v(&t), K(100), 0) };
    let lim = if ymaj { H() } else { W() };
    let steps: V = (lohi[0].max(0)..lohi[1].min(lim)).collect();
    if steps.is_empty() { return; }
    // u on the screen (thousandths from a), each way round
    let us = o.lin(&[(v(&steps), Some(K(100_000))), (K(50_000), None), (K(u0), Some(K(-1000)))]);
    let neg = one(o.lt(K(u1), K(u0))) == 1;
    let us = if neg { o.lin(&[(v(&us), Some(K(-1)))]) } else { us };
    // (a beam longer than 999 pixels: its length by tens till it fits a divisor, the quotient back by as many)
    let sc = tens_under(adu, 100_000).0;
    let us = if sc == 1 { o.div(v(&us), K(adu), 0) } else { let d_ = one(o.div(K(adu), K(sc), 0)); let t = o.div(v(&us), K(d_), 0); o.div(v(&t), K(sc), 0) };
    let us = o.clamp(v(&us), 0, 1000);
    let mc = lerpv(o, K(m0), K(m1), v(&us));
    // the depth: K / z linear on the screen; the place along the cut beam s = u izb / iz
    let iz = lerpv(o, K(iza), K(izb), v(&us));
    let iz = o.max(v(&iz), K(1));
    let ub = o.lin(&[(v(&us), Some(K(izb)))]);
    // (both by tens till the depths fit a divisor, near the camera)
    let sc = iz.iter().map(|&x| tens_under(x, 100_000).0).max().unwrap_or(1);
    let s3 = if sc == 1 { o.div(v(&ub), v(&iz), 0) } else {
        let izs = o.div(v(&iz), K(sc), 0);
        let izs = o.max(v(&izs), K(1));
        let t = o.div(v(&ub), v(&izs), 0);
        o.div(v(&t), K(sc), 0)
    };
    let s3 = o.clamp(v(&s3), 0, 1000);
    // its width (cm) there, and in hundredths of a pixel: w f2 iz / (2 K) 100
    let wc = lerpv(o, K(wa), K(wb), v(&s3));
    let wp = o.lin(&[(v(&wc), Some(v(&iz)))]);
    let wp = o.div(v(&wp), K(1000), 0);
    let kf = fk(o, cam.f2, 1000, 29_970);
    let wp = o.lin(&[(v(&wp), Some(K(kf)))]);
    let wp = o.div(v(&wp), K(1000), 0);
    let wp = o.lin(&[(v(&wp), Some(K(50)))]);
    // (at half resolution: a beam narrower than 0.6 of a pixel there drawn that wide, its light as if 0.3 to 0.6)
    let thin = if res() > 1 { let t = o.clamp(v(&wp), 30, 60); Some(t) } else { None };
    let wp = o.max(v(&wp), K(60));
    // the fading: (1 - s)^1.5 (s along the whole beam) over max(w px, 1)^0.3, times inten
    let sf = { let d = kl(o, &[(sb, None), (sa, Some(-1))]); o.lin(&[(v(&s3), Some(K(d))), (K(sa), Some(K(1000)))]) };
    let sf = o.div(v(&sf), K(1000), 0);
    let along = o.lin(&[(v(&sf), Some(K(-1))), (K(1000), None)]);
    let along = o.clamp(v(&along), 0, 1000);
    let p15 = o.tab(tb.t("pow15"), v(&along));
    let wq = o.div(v(&wp), K(10 / res()), 0);
    let wq = o.clamp(v(&wq), 10, 9999);
    let p03 = o.tab(tb.t("ipow03"), v(&wq));
    let amp = o.lin(&[(v(&p15), Some(v(&p03)))]);
    let amp = o.div(v(&amp), K(1000), 0);
    let amp = o.lin(&[(v(&amp), Some(K(inten)))]);
    let amp = o.div(v(&amp), K(1000), 0);
    let amp = match &thin { Some(t) => { let a = o.lin(&[(v(&amp), Some(v(t)))]); o.div(v(&a), K(60), 0) } None => amp };
    // R tenths exposed per channel: amp col 2500 e / 1e9
    let ampc: [V; 3] = std::array::from_fn(|c| { let k = fk(o, col[c], e, 1000); let k = fk(o, k, 2500, 1000); let t = o.lin(&[(v(&amp), Some(K(k)))]); o.div(v(&t), K(1000), 0) });
    // the reach across (hundredths of a pixel along the shorter axis): 2 w / cos; and across per w
    let reach = o.lin(&[(v(&wp), Some(K(2000)))]);
    let reach = o.div(v(&reach), K(cosf), 0);
    let rlo = { let t = o.lin(&[(v(&mc), None), (v(&reach), Some(K(-1))), (K(49), None)]); o.div(v(&t), K(100), 0) };
    let rhi = { let t = o.lin(&[(v(&mc), None), (v(&reach), None), (K(-50), None)]); o.div(v(&t), K(100), 0) };
    let mlim = if ymaj { W() } else { H() };
    let mut cand: Vec<(usize, i64)> = vec![];
    for k in 0..steps.len() {
        let coarse = wp[k] > 400;
        if coarse && steps[k] % 2 == 1 { continue; }
        let mut r = rlo[k].max(0);
        if coarse && r % 2 == 1 { r -= 1; }
        while r <= rhi[k].min(mlim - 1) { cand.push((k, r)); r += if coarse { 2 } else { 1 }; }
    }
    if cand.is_empty() { return; }
    if std::env::var("NIGHT_BEAMS").is_ok() { eprintln!("beam a {:?} b {:?} steps {} cand {} wp {:?} ampc0max {:?} inten {inten} sa {sa} sb {sb} iz {:?}", (ax, ay, iza), (bx, by, izb), steps.len(), cand.len(), &wp[..wp.len().min(3)], ampc[0].iter().max(), &iz[..iz.len().min(3)]); }
    let cg = |v_: &V| -> V { cand.iter().map(|&(k, _)| v_[k]).collect() };
    // across (hundredths of a pixel): the pixel's middle (from the start's table) less the line's there
    let rr = o.tab(&nw.mid, v(&cand.iter().map(|c| c.1).collect()));
    let dm = o.sum(&[(v(&rr), 1), (v(&cg(&mc)), -1)], 0);
    let dmn = o.sum(&[(v(&dm), -1)], 0);
    let dm = o.max(v(&dm), v(&dmn));
    // by logarithms: ln (d / w) = ln |dm| + ln cos - ln 1000 - ln w (each step's), its Gaussian's logarithm printed
    let lnw = o.tab(tb.t("ln6"), v(&wp));
    let lnc = one(o.tab(tb.t("ln6"), K(cosf)));
    let cst = o.sum(&[(v(&lnw), 1)], 69_078 - 0);
    let cst = o.sum(&[(v(&cst), 1), (K(lnc), -1)], 0);
    let ldm = o.tab(tb.t("ln6"), v(&dm));
    let k = o.ldiff(&ldm, &cg(&cst), -50, 30);
    let lg = o.tabo(tb.t("lngauss"), 50_000, v(&k));
    // in front of what's there (its K / z not under the pixel's)
    let pix: Vec<usize> = cand.iter().map(|&(k, r)| { let (xx, yy) = if ymaj { (r, steps[k]) } else { (steps[k], r) }; (yy * W() + xx) as usize }).collect();
    let gz: V = pix.iter().map(|&p| g.iz[p]).collect();
    let behind = o.lt(v(&cg(&iz)), v(&gz));
    // only those in front and not too far out in the Gaussian (past ln 1e-6 of the brightest, nothing's added)
    let near = o.lt(K(-145_100), v(&lg));
    let cand: Vec<(usize, i64)> = (0..cand.len()).filter(|&j| behind[j] == 0 && near[j] == 1).map(|j| cand[j]).collect();
    let keep: Vec<usize> = (0..pix.len()).filter(|&j| behind[j] == 0 && near[j] == 1).collect();
    if keep.is_empty() { return; }
    let lg: V = keep.iter().map(|&j| lg[j]).collect();
    let pix: Vec<usize> = keep.iter().map(|&j| pix[j]).collect();
    let cg = |v_: &V| -> V { cand.iter().map(|&(k, _)| v_[k]).collect() };
    // each channel: e^(ln g + ln amp)
    let vals: [V; 3] = std::array::from_fn(|c| {
        let am = o.max(v(&ampc[c]), K(1));
        let la = o.tab(tb.t("ln6"), v(&am));
        let nla = o.sum(&[(v(&la), -1)], 0);
        let kk = o.ldiff(&lg, &cg(&nla), -50, 115);
        let r = o.tabo(tb.t("exp6"), 50_000, v(&kk));
        let z = o.lt(K(0), v(&cg(&ampc[c])));
        o.sel(&z, v(&r), K(0))
    });
    let mut px = vec![];
    let mut vv: [V; 3] = [vec![], vec![], vec![]];
    for (j, &(k, _)) in cand.iter().enumerate() {
        let p = pix[j];
        if wp[k] > 400 {
            let (xx, yy) = (p % W() as usize, p / W() as usize);
            for (a_, b_) in [(0, 0), (1, 0), (0, 1), (1, 1)] { px.push((xx + a_).min(W() as usize - 1) + (yy + b_).min(H() as usize - 1) * W() as usize); for c in 0..3 { vv[c].push(vals[c][j]); } }
        } else { px.push(p); for c in 0..3 { vv[c].push(vals[c][j]); } }
    }
    ov.put(o, &px, &vv);
}

/// Unit ways (thousandths) from angles (mrad): (cos el cos az, cos el sin az, sin el) in the stadium's frame.
fn aud_way<O: Ops>(o: &mut O, az: &V, el: &V) -> [V; 3] {
    // (whole turns of 6283 mrad taken off first)
    let turn = |o: &mut O, a: &V| -> V { let q = o.div(v(a), K(6283), 0); let r = o.lin(&[(v(a), None), (v(&q), Some(K(-6283)))]); let t = o.lin(&[(v(&r), Some(K(15_916)))]); let t = o.div(v(&t), K(100), 0); o.turns(v(&t)) };
    let ta = turn(o, az);
    let te = turn(o, el);
    let (sa, ca) = o.sincos(v(&ta));
    let (se, ce) = o.sincos(v(&te));
    let x = { let t = o.lin(&[(v(&ce), Some(v(&ca)))]); o.div(v(&t), K(999), 0) };
    let y = { let t = o.lin(&[(v(&ce), Some(v(&sa)))]); o.div(v(&t), K(999), 0) };
    [x, y, se]
}

/// The stadium's show clock's bar (truncated toward 0, as the prototype's int()) at frame f: (f - 484) / 21.0004.
fn bar_at<O: Ops>(o: &mut O, f: i64) -> i64 {
    let m = one(o.lin(&[(K(f), Some(K(1000))), (K(-1000 * (F_END - 36)), None)]));
    let q = one(o.div(K(m), K(21_000), 0));
    let r = one(o.lin(&[(K(m), None), (K(q), Some(K(-21_000)))]));
    let neg = one(o.lt(K(m), K(0)));
    let rz = one(o.lt(K(0), K(r)));
    let up = one(mulw(o, K(neg), K(rz), 0));
    one(o.lin(&[(K(q), None), (K(up), None)]))
}

/// A whole number mod d (0..d - 1), d small.
fn modn<O: Ops>(o: &mut O, x: i64, d: i64) -> i64 {
    let big = one(o.lin(&[(K(x), None), (K(1000 * d), None)]));
    let q = one(o.div(K(big), K(d), 0));
    one(o.lin(&[(K(big), None), (K(q), Some(K(-d)))]))
}

/// Over the shaded frame: the airship's lights and beams, the stadium's beams, the stars; then the balloons over it
/// all; then the fireworks and the photo flashes.
pub fn night_post<O: Ops>(o: &mut O, nw: &NightWorld, l: &NightLight, fx: &FrameAir, cam: &Cam, g: &GBuf, col: &mut [V; 3], hot: &mut [V; 3]) -> ([V; 3], Vec<bool>) {
    let tb = &nw.w.tb;
    let (f, n, e) = (l.f, l.night, l.e);
    let mut ov = Over::new();
    // the airship's running lights: warm, a string round the screen's frame, red beacons blinking; brighter near
    // (120 m / z held to 0.9..4), stronger in the dark; a dot and its four neighbours at 0.35
    {
        let pts: Vec<[i64; 3]> = nw.blights.iter().map(|q| [q[0], q[1], q[2]]).collect();
        let pw: Vec<[i64; 3]> = { let c: [V; 3] = std::array::from_fn(|i| { let a: V = pts.iter().map(|p| p[i]).collect(); o.lin(&[(v(&a), None), (K(fx.blimp[i]), None)]) }); (0..pts.len()).map(|k| [c[0][k], c[1][k], c[2][k]]).collect() };
        let pr = proj(o, cam, &pw);
        let ok: Vec<usize> = (0..pw.len()).filter(|&k| pr.front[k] && pr.z[k] > 100 && pr.x[k] >= 100 && pr.x[k] < 100 * (W() - 1) && pr.y[k] >= 100 && pr.y[k] < 100 * (H() - 1)).collect();
        if !ok.is_empty() {
            let gk = |v_: &V| -> V { ok.iter().map(|&k| v_[k]).collect() };
            let ix = o.div(v(&gk(&pr.x)), K(100), 0);
            let iy = o.div(v(&gk(&pr.y)), K(100), 0);
            let pix: Vec<usize> = (0..ok.len()).map(|j| (iy[j] * W() + ix[j]) as usize).collect();
            let gz: V = pix.iter().map(|&p| g.iz[p]).collect();
            let a1 = o.lin(&[(v(&gk(&pr.iz)), Some(K(1000)))]);
            let a2 = o.lin(&[(v(&gz), Some(K(995)))]);
            let vis = o.lt(v(&a1), v(&a2));
            // the beacons blink: sin(0.9 f + its place among those shown) > 0.3
            let kinds: V = ok.iter().map(|&k| nw.blights[k][3]).collect();
            let idx: V = { let shown: V = (0..pw.len()).map(|k| ok.contains(&k) as i64).collect(); let before: V = std::iter::once(0).chain(shown[..pw.len() - 1].iter().copied()).collect(); let r = o.runsum(&[&before]); ok.iter().map(|&k| r[k]).collect() };
            let ba = o.lin(&[(v(&idx), Some(K(159_155))), (K(f), Some(K(143_239)))]);
            let ba = o.turns(v(&ba));
            let (bs, _) = o.sincos(v(&ba));
            let on = o.lt(K(299), v(&bs));
            let beacon = o.lt(K(1), v(&kinds));
            let blink = o.sel(&beacon, v(&on), K(1));
            let s = o.div(K(12_000_000), v(&gk(&pr.z)), 0);
            let s = o.clamp(v(&s), 900, 4000);
            let nk = one(o.lin(&[(K(n), Some(K(800))), (K(400_000), None)]));
            let nk = one(o.div(K(nk), K(1000), 0));
            let s = o.lin(&[(v(&s), Some(K(nk)))]);
            let s = o.div(v(&s), K(1000), 0);
            let s = mulw(o, v(&s), v(&blink), 0);
            let vis0 = o.lin(&[(v(&vis), Some(K(-1))), (K(1), None)]);
            let s = mulw(o, v(&s), v(&vis0), 0);
            // the colours: warm (3.0, 1.7, 0.7), the frame's (3.5, 2.2, 0.9), a beacon (4.0, 0.2, 0.1)
            let is1 = within_v(o, &kinds, 0, 2);
            let rgb: [V; 3] = std::array::from_fn(|c| { let t = o.sel(&is1, K([3500, 2200, 900][c]), K([3000, 1700, 700][c])); o.sel(&beacon, K([4000, 200, 100][c]), v(&t)) });
            // R tenths: rgb s w 2500 e / 1e9
            let kk = fk(o, 2500, e, 1000);
            let base: [V; 3] = std::array::from_fn(|c| { let t = o.lin(&[(v(&rgb[c]), Some(v(&s)))]); let t = o.div(v(&t), K(1000), 0); scaled(o, &t, kk, 1000) });
            let mut px = vec![];
            let mut vv: [V; 3] = [vec![], vec![], vec![]];
            // (at half resolution one pixel, 0.6 of it: about the same light over the same place)
            let side: [V; 3] = std::array::from_fn(|c| mulw(o, v(&base[c]), K(if res() == 1 { 35 } else { 60 }), 2));
            let taps: &[(i64, i64, i32)] = if res() == 1 { &[(0, 0, 0), (1, 0, 1), (-1, 0, 1), (0, 1, 1), (0, -1, 1)] } else { &[(0, 0, 1)] };
            for (j, &p) in pix.iter().enumerate() {
                for &(dx, dy, wsel) in taps {
                    px.push((p as i64 + dy * W() + dx) as usize);
                    for c in 0..3 { vv[c].push(if wsel == 0 { base[c][j] } else { side[c][j] }); }
                }
            }
            ov.put(o, &px, &vv);
        }
    }
    o.stage("p_beams");
    // the up-lights' beams onto the hull (0.5 to 5 m wide, a little past the spot), on the beat
    let upk = { let t = one(o.lin(&[(K(l.beat), Some(K(450))), (K(550_000), None)])); let t = one(o.div(K(t), K(1000), 0)); let t = fk(o, t, n, 1000); fk(o, t, 450, 1000) };
    for k in 0..fx.up_src.len() {
        let (a, q) = (fx.up_src[k], fx.up_spot[k]);
        let b: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K(q[i]), Some(K(51))), (K(a[i]), Some(K(-1)))]); o.div(v(&t), K(50), 0) }));
        beam(o, nw, cam, g, a, b, 50, 500, [850, 900, 1000], upk, e, &mut ov);
    }
    // its four searchlights from the belly raking down, 260 m long
    {
        let sk = { let t = one(o.lin(&[(K(n), Some(K(700))), (K(300_000), None)])); let t = one(o.div(K(t), K(1000), 0)); fk(o, t, 500, 1000) };
        for (k, (lx, lz)) in [(-30i64, -6i64), (-10, 5), (12, -4), (32, 6)].iter().enumerate() {
            let a: [i64; 3] = [one(o.lin(&[(K(fx.blimp[0]), None), (K(140 * lx), None)])), one(o.lin(&[(K(fx.blimp[1]), None), (K(-140 * 17), None)])), one(o.lin(&[(K(fx.blimp[2]), None), (K(140 * lz), None)]))];
            let sw = { let t = one(o.lin(&[(K(f), Some(K(5570))), (K(270_563 * k as i64), None)])); let t = one(o.turns(K(t))); let (s, _) = o.sincos(K(t)); fk(o, s[0], 500, 999) };
            let swt = fk(o, sw, 159_155, 1000);
            let (ss, _) = o.sincos(K(swt));
            let ca = { let t = one(o.lin(&[(K(f), Some(K(4775))), (K(159_155 * k as i64), None)])); one(o.turns(K(t))) };
            let (_, cc) = o.sincos(K(ca));
            let dx = fk(o, ss[0], 500, 999);
            let dz = fk(o, cc[0], 350, 999);
            let d = nrm3(o, &vec![dx], &vec![-1000], &vec![dz]);
            let b: [i64; 3] = std::array::from_fn(|i| one(o.lin(&[(K(a[i]), None), (K(d[i][0]), Some(K(26)))])));
            beam(o, nw, cam, g, a, b, 80, 1600, [550, 850, 1000], sk, e, &mut ov);
        }
    }
    if n >= 200 {
        let tt_turns = |o: &mut O, rate: i64| -> i64 { one(o.lin(&[(K(f), Some(K(rate))), (K(-(F_END - 36) * rate), None)])) };
        // the stadium's searchlights: sweeping, 900 m, on 2 and 4 (pulse 0.25)
        let pk = pulse(o, n, l.beat, 250);
        let ik = fk(o, 600, n, 1000);
        let ik = fk(o, ik, pk, 1000);
        let ph: V = nw.search.iter().map(|s_| s_[7]).collect();
        let azt = { let t = tt_turns(o, 2917); one(o.div(K(t), K(100), 0)) };
        let az = o.lin(&[(v(&ph), None), (K(azt), None)]);
        let et = tt_turns(o, 7958);
        let ea = o.lin(&[(v(&ph), Some(K(159))), (K(et), None)]);
        let ea = o.turns(v(&ea));
        let (es, _) = o.sincos(v(&ea));
        let el = o.lin(&[(v(&es), Some(K(314))), (K(1082 * 999), None)]);
        let el = o.div(v(&el), K(999), 0);
        let wy = aud_way(o, &az, &el);
        if std::env::var("NIGHT_BEAMS").is_ok() { for k in 0..3 { eprintln!("search {} {} {}", wy[0][k], wy[1][k], wy[2][k]); } }
        for (k, s_) in nw.search.iter().enumerate() {
            let src = [s_[0], s_[1], s_[2]];
            let to = [one(o.lin(&[(K(src[0]), None), (K(wy[0][k]), Some(K(90)))])), one(o.lin(&[(K(src[1]), None), (K(wy[1][k]), Some(K(90)))])), one(o.lin(&[(K(src[2]), None), (K(wy[2][k]), Some(K(90)))]))];
            let ends = aud_film(o, nw, &[src, to]);
            let colr: [i64; 3] = std::array::from_fn(|c| fk(o, s_[3 + c], s_[6], 1000));
            beam(o, nw, cam, g, ends[0], ends[1], 60, 1800, colr, ik, e, &mut ov);
        }
        // the spots: the moving heads over the crowd, a pattern a bar (of three), to the floor 1.8 m up; on 1 and 3
        let pk = pulse(o, n, l.beat, 100);
        let ik = fk(o, 450, n, 1000);
        let ik = fk(o, ik, pk, 1000);
        let bar = bar_at(o, f);
        let pat = modn(o, bar, 3);
        let wt = tt_turns(o, 20_833);
        for s_ in &nw.spots {
            let ph = s_[6];
            let sp = |o: &mut O, rate_tt: i64, k_ph: i64| -> i64 { let t = one(o.lin(&[(K(wt), Some(K(rate_tt))), (K(ph), Some(K(k_ph)))])); one(o.turns(K(t))) };
            let plus = |o: &mut O, x: i64, c: i64| -> i64 { one(o.lin(&[(K(x), None), (K(c), None)])) };
            let (pan, tilt) = match pat {
                0 => {
                    let t1 = sp(o, 1, 48); let (a1, _) = o.sincos(K(t1));
                    let t2 = sp(o, 2, 159); let (a2, _) = o.sincos(K(t2));
                    let p = fk(o, a1[0], 550, 999); let t = fk(o, a2[0], 150, 999);
                    (p, plus(o, t, 550))
                }
                1 => {
                    let t1 = sp(o, 1, 0); let (a1, _) = o.sincos(K(t1));
                    let q = one(o.div(K(ph), K(450), 0));
                    let odd = modn(o, q, 2);
                    let p = fk(o, a1[0], 600, 999);
                    let mp = one(o.lin(&[(K(p), Some(K(-1)))]));
                    let p = one(o.sel(&vec![odd], K(p), K(mp)));
                    let t2 = sp(o, 1, 159); let (_, c2) = o.sincos(K(t2));
                    let t = fk(o, c2[0], 200, 999);
                    (p, plus(o, t, 500))
                }
                _ => {
                    let t1 = sp(o, 1, 318); let (a1, c1) = o.sincos(K(t1));
                    let p = fk(o, a1[0], 400, 999); let t = fk(o, c1[0], 250, 999);
                    (p, plus(o, t, 500))
                }
            };
            let pt = one(mturns(o, &vec![pan]));
            let tl = one(mturns(o, &vec![tilt]));
            let (ps, pc) = o.sincos(K(pt));
            let (ts, tc) = o.sincos(K(tl));
            let mpc = one(o.lin(&[(K(pc[0]), Some(K(-1)))]));
            let mtc = one(o.lin(&[(K(tc[0]), Some(K(-1)))]));
            let d = [fk(o, ps[0], ts[0], 999), fk(o, mpc, ts[0], 999), mtc];
            if std::env::var("NIGHT_BEAMS").is_ok() { eprintln!("spot {:?} bar {bar} pat {pat}", d); }
            let src = [s_[0], s_[1], s_[2]];
            // to the floor: L = (z - 1.8 m) / max(-d.z, 0.05)
            let mz = one(o.lin(&[(K(d[2]), Some(K(-1)))]));
            let mz = one(o.max(K(mz), K(50)));
            let lz = one(o.lin(&[(K(src[2]), None), (K(-180), None)]));
            let to: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K(d[i]), Some(K(lz)))]); o.div(v(&t), K(mz), 0) }));
            let to: [i64; 3] = std::array::from_fn(|i| one(o.lin(&[(K(src[i]), None), (K(to[i]), None)])));
            let ends = aud_film(o, nw, &[src, to]);
            beam(o, nw, cam, g, ends[0], ends[1], 30, 400, [s_[3], s_[4], s_[5]], ik, e, &mut ov);
        }
        // the lasers: fans of beams 125 m long, a pattern a bar
        let pk = pulse(o, n, l.beat, 200);
        let base_k = fk(o, 1200, n, 1000);
        let base_k = fk(o, base_k, pk, 1000);
        let tb_ = tt_turns(o, 47_618);
        let tb2 = tt_turns(o, 23_809);
        // the beat's phase (beats of 5.25 frames): under half
        let bm = one(o.lin(&[(K(f), Some(K(1000))), (K(-1000 * (F_END - 36) - BEAT_T0), None), (K(5250 * 1000), None)]));
        let bq = one(o.div(K(bm), K(5250), 0));
        let br = one(o.lin(&[(K(bm), None), (K(bq), Some(K(-5250)))]));
        let first_half = one(o.lt(K(br), K(2625)));
        for (li, s_) in nw.lasers.iter().enumerate() {
            let src = [s_[0], s_[1], s_[2]];
            let nb = s_[6];
            let spread = fk(o, s_[7], 8727, 1000);
            let shift = one(o.div(K(s_[0]), K(2000), 0));
            let shift = if s_[0] < 0 { let r = one(o.lin(&[(K(s_[0]), None), (K(shift), Some(K(-2000)))])); let up = one(o.lt(K(0), K(r))); one(o.lin(&[(K(shift), None), (K(up), None)])) } else { shift };
            let bs_ = one(o.lin(&[(K(bar), None), (K(shift), None)]));
            let pt = modn(o, bs_, 4);
            let ss = &nw.laser_fan[li];
            let (az, el): (V, V) = match pt {
                0 => {
                    let t1 = one(o.turns(K(tb_)));
                    let (s1, _) = o.sincos(K(t1));
                    let elv = one(o.lin(&[(K(s1[0]), Some(K(26))), (K(35 + 26 * 999 / 1000 * 0), None)]));
                    let elv = one(o.div(K(elv), K(999), 0));
                    let elv = one(o.lin(&[(K(elv), None), (K(61), None)]));
                    let az = o.lin(&[(v(&ss), Some(K(spread)))]);
                    let az = o.div(v(&az), K(1000), 0);
                    (o.lin(&[(v(&az), None), (K(-1571), None)]), vec![elv; nb as usize])
                }
                1 => {
                    let t2 = one(o.turns(K(tb2)));
                    let (s2, _) = o.sincos(K(t2));
                    let t1 = one(o.turns(K(tb_)));
                    let (s1, _) = o.sincos(K(t1));
                    let sh = fk(o, s2[0], 400, 999);
                    let az = o.lin(&[(v(&ss), Some(K(spread)))]);
                    let az = o.div(v(&az), K(2000), 0);
                    let az = o.lin(&[(v(&az), None), (K(sh), None), (K(-1571), None)]);
                    let e1 = fk(o, s1[0], 175, 999);
                    let elv = one(o.lin(&[(K(e1), None), (K(436), None)]));
                    (az, vec![elv; nb as usize])
                }
                2 => {
                    let a = o.lin(&[(v(&ss), Some(K(500))), (K(tb_), None)]);
                    let a = o.turns(v(&a));
                    let (sn, _) = o.sincos(v(&a));
                    let az = o.lin(&[(v(&sn), Some(K(600)))]);
                    let az = o.div(v(&az), K(999), 0);
                    let az = o.lin(&[(v(&az), None), (K(-1571), None)]);
                    let el = o.lin(&[(v(&ss), Some(K(209))), (K(209_000 + 70_000), None)]);
                    (az, o.div(v(&el), K(1000), 0))
                }
                _ => {
                    let az = o.lin(&[(v(&ss), Some(K(spread)))]);
                    let az = o.div(v(&az), K(1000), 0);
                    (o.lin(&[(v(&az), None), (K(-1571), None)]), vec![105; nb as usize])
                }
            };
            let wy = aud_way(o, &az, &el);
            // which are on: pattern 3 chases one beam at a time, strobing on the beat
            let on: V = if pt == 3 {
                let fo = one(o.lin(&[(K(f), None), (K(-(F_END - 36)), None)]));
                let kc = modn(o, fo, nb);
                let js: V = (0..nb).collect();
                let dj = o.lin(&[(v(&js), None), (K(kc), Some(K(-1))), (K(nb), None)]);
                let q = o.div(v(&dj), K(nb), 0);
                let dj = o.lin(&[(v(&dj), None), (v(&q), Some(K(-nb)))]);
                let dj = o.lin(&[(v(&dj), Some(K(1000)))]);
                let ex = expn(o, tb, &dj);
                let k = if first_half == 1 { 1000 } else { 400 };
                scaled(o, &ex, k, 1000)
            } else { vec![1000; nb as usize] };
            for j in 0..nb as usize {
                if on[j] <= 50 { continue; }
                let to: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K(src[i]), Some(K(10))), (K(wy[i][j]), Some(K(125)))]); o.div(v(&t), K(10), 0) }));
                let ends = aud_film(o, nw, &[src, to]);
                let ik = fk(o, base_k, on[j], 1000);
                let cl: [i64; 3] = std::array::from_fn(|c| fk(o, s_[3 + c], 1400, 1000));
                beam(o, nw, cam, g, ends[0], ends[1], 5, 25, cl, ik, e, &mut ov);
            }
        }
    }
    o.stage("p_stars");
    // the stars (at half resolution, the party version's world: none here, they're drawn at full resolution)
    if res() == 1 { let sky = |p: usize| g.mat[p] < 0; stars_over(o, nw, n, e, cam, &sky, &mut ov); }
    // (the party version keeps the light laid on, for its full-resolution subjects)
    let snap = if res() > 1 { ov.add.clone() } else { [vec![], vec![], vec![]] };
    let mut laid = vec![false; if res() > 1 { N_PIX() } else { 0 }];
    ov.apply(o, col, hot);
    // the balloons over it all, then the fireworks and the photo flashes
    o.stage("p_balloons");
    if n >= 400 { balloons(o, nw, l, cam, g, col, hot, &mut laid); }
    o.stage("p_fire");
    if n >= 500 {
        fireworks(o, nw, l, cam, g, &mut ov);
        flashes(o, nw, l, cam, g, &mut ov);
        if !laid.is_empty() { for p in 0..N_PIX() { if ov.touched[p] { laid[p] = true; } } }
        ov.apply(o, col, hot);
    }
    (snap, laid)
}

/// The stars, where the sky shows (sky(pixel)), as the night comes (brightness times n - 0.3): each a pixel where
/// it projects, its light laid on.
pub fn stars_over<O: Ops>(o: &mut O, nw: &NightWorld, n: i64, e: i64, cam: &Cam, sky: &dyn Fn(usize) -> bool, ov: &mut Over) {
    if n > 300 {
        let nk = one(o.lin(&[(K(n), None), (K(-300), None)]));
        let st = &nw.stars;
        let c: [V; 3] = std::array::from_fn(|r| { let t = o.lin(&[(v(&st.iter().map(|s_| s_[0]).collect()), Some(K(cam.b[r][0]))), (v(&st.iter().map(|s_| s_[1]).collect()), Some(K(cam.b[r][1]))), (v(&st.iter().map(|s_| s_[2]).collect()), Some(K(cam.b[r][2])))]); o.div(v(&t), K(1000), 0) });
        let ahead: Vec<usize> = (0..st.len()).filter(|&k| c[2][k] > 100).collect();
        if !ahead.is_empty() {
            let gk = |v_: &V| -> V { ahead.iter().map(|&k| v_[k]).collect() };
            let z = gk(&c[2]);
            let x = { let t = o.lin(&[(v(&gk(&c[0])), Some(K(cam.f2)))]); let t = o.div(v(&t), v(&z), 0); let t = o.div(v(&t), K(2), 0); o.lin(&[(v(&t), None), (K(W() / 2), None)]) };
            let nf2 = kl(o, &[(cam.f2, Some(-1))]);
            let y = { let t = o.lin(&[(v(&gk(&c[1])), Some(K(nf2)))]); let t = o.div(v(&t), v(&z), 0); let t = o.div(v(&t), K(2), 0); o.lin(&[(v(&t), None), (K(H() / 2), None)]) };
            let on: Vec<usize> = (0..ahead.len()).filter(|&j| (0..W()).contains(&x[j]) && (0..H()).contains(&y[j]) && sky((y[j] * W() + x[j]) as usize)).collect();
            if !on.is_empty() {
                let px: Vec<usize> = on.iter().map(|&j| (y[j] * W() + x[j]) as usize).collect();
                let br: V = on.iter().map(|&j| st[ahead[j]][3]).collect();
                let b = scaled(o, &br, nk, 1000);
                let kk = fk(o, 2500, e, 1000);
                let vals: [V; 3] = std::array::from_fn(|ch| { let t = scaled(o, &b, [800, 850, 1000][ch], 1000); scaled(o, &t, kk, 1000) });
                ov.put(o, &px, &vals);
            }
        }
    }
}

/// The balloons (900, rising round the stadium and over the town, swaying): each a disc 0.9 m across (1.2 times as
/// tall), shaded from its upper left with a highlight, its colour lit by the stadium on the beat (more the nearer),
/// laid over what's behind it by its edge's coverage; a speck where under half a pixel.
fn balloons<O: Ops>(o: &mut O, nw: &NightWorld, l: &NightLight, cam: &Cam, g: &GBuf, col: &mut [V; 3], hot: &mut [V; 3], laid: &mut [bool]) {
    let tb = &nw.w.tb;
    let (f, e) = (l.f, l.e);
    let b = &nw.balloons;
    let nb = b.len();
    let g0 = |i: usize| -> V { b.iter().map(|q| q[i]).collect() };
    // rising (mm a frame) round 320 m from the land, swaying 1.5 m (sin(0.05 f + ph))
    let up = o.lin(&[(v(&g0(1)), Some(K(10))), (v(&g0(3)), Some(K(f)))]);
    let q = { let t = o.div(v(&up), K(1000), 0); o.div(v(&t), K(320), 0) };
    let up = o.lin(&[(v(&up), None), (v(&q), Some(K(-320_000)))]);
    let y = o.div(v(&up), K(10), 0);
    let y = o.lin(&[(v(&y), None), (K(LAND), None)]);
    let sa = o.lin(&[(K(f), Some(K(7958))), (v(&g0(7)), Some(K(159)))]);
    let sa = o.turns(v(&sa));
    let (ss, _) = o.sincos(v(&sa));
    let dx = scaled(o, &ss, 150, 999);
    let x = o.lin(&[(v(&g0(0)), None), (v(&dx), None)]);
    let z = g0(2);
    let pts: Vec<[i64; 3]> = (0..nb).map(|k| [x[k], y[k], z[k]]).collect();
    let pr = proj(o, cam, &pts);
    // its radius (hundredths of a pixel): 0.9 m f / z
    let zz = o.max(v(&pr.z), K(1));
    let rad = { let k = kl(o, &[(cam.f2, Some(90 * 50))]); o.div(K(k), v(&zz), 0) };
    // those in front and on the screen by their radius: the places' sums with it and less it, against the screen's
    // edges, on the machines
    let ahead: Vec<usize> = (0..nb).filter(|&k| pr.front[k] && pr.z[k] > 200).collect();
    if ahead.is_empty() { return; }
    let ga = |v_: &V| -> V { ahead.iter().map(|&k| v_[k]).collect() };
    let (ax, ay, ar) = (ga(&pr.x), ga(&pr.y), ga(&rad));
    let xp = o.lin(&[(v(&ax), None), (v(&ar), None)]);
    let xm = o.lin(&[(v(&ax), None), (v(&ar), Some(K(-1)))]);
    let yp = o.lin(&[(v(&ay), None), (v(&ar), None)]);
    let ym = o.lin(&[(v(&ay), None), (v(&ar), Some(K(-1)))]);
    let on = [o.lt(K(0), v(&xp)), o.lt(v(&xm), K(100 * W())), o.lt(K(0), v(&yp)), o.lt(v(&ym), K(100 * H()))];
    let ok: Vec<usize> = (0..ahead.len()).filter(|&j| on.iter().all(|f| f[j] == 1)).map(|j| ahead[j]).collect();
    if ok.is_empty() { return; }
    // lit: 0.6 + 1.2 beat e^-(distance from the stadium's middle 40 m up / 300 m)
    let gk = |v_: &V| -> V { ok.iter().map(|&k| v_[k]).collect() };
    let ac = nw.aud_c;
    let dd: [V; 3] = [o.lin(&[(v(&gk(&x)), None), (K(ac[0]), Some(K(-1))), (K(-BX), None)]), o.lin(&[(v(&gk(&y)), None), (K(ac[1]), Some(K(-1))), (K(-4000), None)]), o.lin(&[(v(&gk(&z)), None), (K(ac[2]), Some(K(-1)))])];
    let dist = len3(o, &dd[0], &dd[1], &dd[2]);
    let xd = o.div(v(&dist), K(30), 0);
    let fall = expn(o, tb, &xd);
    let lit = o.lin(&[(v(&fall), Some(K(l.beat)))]);
    let lit = o.div(v(&lit), K(1000), 0);
    let lit = o.lin(&[(v(&lit), Some(K(12))), (K(6000), None)]);
    let lit = o.div(v(&lit), K(10), 0);
    let kk = fk(o, 250, e, 1000);
    // per balloon: its pixels (its box round it: from its middle less the radius across and 1.2 times it down, a
    // pixel more, to the radius and two pixels on, the pixels on the machines; held to the screen by the host);
    // each pixel's place in the disc, by the radius
    let mut tiny: Vec<usize> = vec![];
    let big: Vec<usize> = (0..ok.len()).filter(|&j| { let t = rad[ok[j]] < 50; if t { tiny.push(j); } !t }).collect();
    let mut cand: Vec<(usize, usize)> = vec![];
    if !big.is_empty() {
        let gb = |v_: &V| -> V { big.iter().map(|&j| v_[ok[j]]).collect() };
        let (bx, by, br) = (gb(&pr.x), gb(&pr.y), gb(&rad));
        let r12 = { let t = o.lin(&[(v(&br), Some(K(120)))]); o.div(v(&t), K(100), 0) };
        let x0 = o.lin(&[(v(&bx), None), (v(&br), Some(K(-1))), (K(-100), None)]);
        let x1 = o.lin(&[(v(&bx), None), (v(&br), None), (K(200), None)]);
        let y0 = o.lin(&[(v(&by), None), (v(&r12), Some(K(-1))), (K(-100), None)]);
        let y1 = o.lin(&[(v(&by), None), (v(&r12), None), (K(200), None)]);
        let n_ = big.len();
        let ends = o.div(v(&[x0, x1, y0, y1].concat()), K(100), 0);
        for (i, &j) in big.iter().enumerate() {
            let (x0, x1, y0, y1) = (ends[i], ends[n_ + i], ends[2 * n_ + i], ends[3 * n_ + i]);
            for yy in y0.max(0)..y1.min(H()) { for xx in x0.max(0)..x1.min(W()) { cand.push((j, (yy * W() + xx) as usize)); } }
        }
    }
    let mut ov = Over::new();
    if !tiny.is_empty() {
        // a speck: its colour lit, times 1.2 its radius, where it's in front
        let sx: V = tiny.iter().map(|&j| pr.x[ok[j]]).collect();
        let sy: V = tiny.iter().map(|&j| pr.y[ok[j]]).collect();
        let (sx, sy) = (o.div(v(&sx), K(100), 0), o.div(v(&sy), K(100), 0));
        let pix: Vec<usize> = (0..tiny.len()).map(|i| (sy[i].clamp(0, H() - 1) * W() + sx[i].clamp(0, W() - 1)) as usize).collect();
        let gz: V = pix.iter().map(|&p| g.iz[p]).collect();
        let iz: V = tiny.iter().map(|&j| pr.iz[ok[j]]).collect();
        let behind = o.lt(v(&iz), v(&gz));
        let r: V = tiny.iter().map(|&j| rad[ok[j]]).collect();
        let lt_: V = tiny.iter().map(|&j| lit[j]).collect();
        let k = o.lin(&[(v(&r), Some(v(&lt_)))]);
        let k = o.div(v(&k), K(100), 0);
        let k = o.lin(&[(v(&k), Some(K(12)))]);
        let k = o.div(v(&k), K(10), 0);
        let k = o.sel(&behind, K(0), v(&k));
        let vals: [V; 3] = std::array::from_fn(|c| { let cc: V = tiny.iter().map(|&j| b[ok[j]][4 + c]).collect(); let t = o.lin(&[(v(&cc), Some(v(&k)))]); let t = o.div(v(&t), K(1000), 0); let t = scaled(o, &t, kk, 1000); o.lin(&[(v(&t), Some(K(10)))]) });
        ov.put(o, &pix, &vals);
        if !laid.is_empty() { for &p in &pix { laid[p] = true; } }
        ov.apply(o, col, hot);
    }
    if cand.is_empty() { return; }
    if !laid.is_empty() { for &(_, p) in &cand { laid[p] = true; } }
    let cj = |v_: &V| -> V { cand.iter().map(|&(j, _)| v_[j]).collect() };
    let okv = |v_: &V| -> V { ok.iter().map(|&k| v_[k]).collect() };
    let (cx, cy, r, iz) = (cj(&okv(&pr.x)), cj(&okv(&pr.y)), cj(&okv(&rad)), cj(&okv(&pr.iz)));
    let px = o.tab(&nw.mid, v(&cand.iter().map(|&(_, p)| p as i64 % W()).collect()));
    let py = o.tab(&nw.mid, v(&cand.iter().map(|&(_, p)| p as i64 / W()).collect()));
    // dx = (x - cx) / r, dy = (y - cy) / (1.2 r): thousandths
    let dxn = o.lin(&[(v(&px), Some(K(1000))), (v(&cx), Some(K(-1000)))]);
    let dx = o.div(v(&dxn), v(&r), 0);
    let r12 = o.lin(&[(v(&r), Some(K(12)))]);
    let r12 = o.div(v(&r12), K(10), 0);
    let dyn_ = o.lin(&[(v(&py), Some(K(1000))), (v(&cy), Some(K(-1000)))]);
    let dy = o.div(v(&dyn_), v(&r12), 0);
    let dx = o.clamp(v(&dx), -2000, 2000);
    let dy = o.clamp(v(&dy), -2000, 2000);
    let d2 = o.lin(&[(v(&dx), Some(v(&dx))), (v(&dy), Some(v(&dy)))]);
    let d2 = o.div(v(&d2), K(1000), 0);
    // coverage: (1 - d2) r (r in pixels), held to 0..1, where in front
    let cv = o.lin(&[(v(&d2), Some(K(-1))), (K(1000), None)]);
    let cv = o.lin(&[(v(&cv), Some(v(&r)))]);
    let cv = o.div(v(&cv), K(100), 0);
    let cv = o.clamp(v(&cv), 0, 1000);
    let gz: V = cand.iter().map(|&(_, p)| g.iz[p]).collect();
    let behind = o.lt(v(&iz), v(&gz));
    let cv = o.sel(&behind, K(0), v(&cv));
    // the shading: 0.35 + 0.65 (1 - ((dx + 0.35)^2 + (dy + 0.4)^2)) held, and a highlight 2 e^-(that / 0.02)
    let hx = o.lin(&[(v(&dx), None), (K(350), None)]);
    let hy = o.lin(&[(v(&dy), None), (K(400), None)]);
    let h2 = o.lin(&[(v(&hx), Some(v(&hx))), (v(&hy), Some(v(&hy)))]);
    let h2 = o.div(v(&h2), K(1000), 0);
    let sh = o.lin(&[(v(&h2), Some(K(-1))), (K(1000), None)]);
    let sh = o.clamp(v(&sh), 0, 1000);
    let sh = o.lin(&[(v(&sh), Some(K(650))), (K(350_000), None)]);
    let sh = o.div(v(&sh), K(1000), 0);
    let hx2 = o.lin(&[(v(&h2), Some(K(50)))]);
    let hl = expn(o, tb, &hx2);
    let hl = o.lin(&[(v(&hl), Some(K(2)))]);
    let lt_ = cj(&lit);
    let shl = o.lin(&[(v(&sh), Some(v(&lt_)))]);
    let shl = o.div(v(&shl), K(1000), 0);
    let hll = o.lin(&[(v(&hl), Some(v(&lt_)))]);
    let hll = o.div(v(&hll), K(1000), 0);
    for c in 0..3 {
        let bc: V = cand.iter().map(|&(j, _)| b[ok[j]][4 + c]).collect();
        let t = o.lin(&[(v(&bc), Some(v(&shl)))]);
        let t = o.div(v(&t), K(1000), 0);
        let t = o.lin(&[(v(&t), None), (v(&hll), None)]);
        let t = scaled(o, &t, kk, 1000);
        let t = o.min(v(&t), K(999));
        // over what's there: mix by the coverage (percent), in rounds so each pixel is laid once a round
        let mut round = vec![0usize; cand.len()];
        let mut seen: std::collections::HashMap<usize, usize> = Default::default();
        for (k, &(_, p)) in cand.iter().enumerate() { let e2 = seen.entry(p).or_insert(0); round[k] = *e2; *e2 += 1; }
        let rounds = round.iter().copied().max().unwrap_or(0) + 1;
        for rd in 0..rounds {
            let at: Vec<usize> = (0..cand.len()).filter(|&k| round[k] == rd).collect();
            let cur: V = at.iter().map(|&k| col[c][cand[k].1]).collect();
            let tt: V = at.iter().map(|&k| t[k]).collect();
            let cvp: V = at.iter().map(|&k| cv[k]).collect();
            let cvp = o.div(v(&cvp), K(10), 0);
            let m = o.mix(v(&cur), v(&tt), v(&cvp));
            let hh: V = at.iter().map(|&k| hot[c][cand[k].1]).collect();
            let keep = o.lin(&[(v(&cvp), Some(K(-1))), (K(100), None)]);
            let h2_ = o.pct(v(&hh), v(&keep));
            for (j, &k) in at.iter().enumerate() { col[c][cand[k].1] = m[j]; hot[c][cand[k].1] = h2_[j]; }
        }
    }
}

/// The fireworks: shells bursting high over the stadium on the first beat of every second bar (24 frames apart, on
/// the hand-off), two a burst, each 150 sparks flying out 45 (1 - e^-age/4) m and falling, fading over 30 frames;
/// each spark a dot and two neighbours at 0.4.
fn fireworks<O: Ops>(o: &mut O, nw: &NightWorld, l: &NightLight, cam: &Cam, g: &GBuf, ov: &mut Over) {
    let tb = &nw.w.tb;
    let (f, e) = (l.f, l.e);
    // (the frames since the latest burst: f - F_END mod 24, on the machines)
    let since = { let d = kl(o, &[(f, None), (-F_END, None)]); o.divmod(K(d), K(24)).1[0] };
    for k in 0..4i64 {
        let age = kl(o, &[(since, None), (24 * k, None)]);
        if !(0..=30).contains(&age) { continue; }
        let start = kl(o, &[(f, None), (age, Some(-1))]);
        for (si, sh) in nw.fw_shells.iter().enumerate().filter(|(_, sh)| sh[0] == start) {
            let ways = &nw.fw_ways[150 * si..150 * si + 150];
            // its reach (cm): 4500 (1 - e^-age/4), the fall 4 age^2 cm, the fade (1 - age/30)^1.5 6
            let a250 = kl(o, &[(age, Some(250))]);
            let rr = one(expn(o, tb, &vec![a250]));
            let rr = one(o.lin(&[(K(rr), Some(K(-45))), (K(45_000), None)]));
            let rr = one(o.div(K(rr), K(10), 0));
            // (the fall taken off the height: -4 age^2)
            let m4 = kl(o, &[(age, Some(-4))]);
            let nfall = one(o.lin(&[(K(age), Some(K(m4)))]));
            let fade = { let t = one(o.lin(&[(K(age), Some(K(-1000))), (K(30_000), None)])); let t = one(o.div(K(t), K(30), 0)); let t = one(o.clamp(K(t), 0, 1000)); one(o.tab(tb.t("pow15"), K(t))) };
            let fade = fk(o, fade, 6000, 1000);
            let pts: Vec<[i64; 3]> = { let c: [V; 3] = std::array::from_fn(|i| { let w_: V = ways.iter().map(|q| q[i]).collect(); let t = o.lin(&[(v(&w_), Some(K(rr)))]); let t = o.div(v(&t), K(1000), 0); o.lin(&[(v(&t), None), (K(sh[1 + i]), None), (K(if i == 1 { nfall } else { 0 }), None)]) }); (0..150).map(|j| [c[0][j], c[1][j], c[2][j]]).collect() };
            let pr = proj(o, cam, &pts);
            let ok: Vec<usize> = (0..150).filter(|&j| pr.front[j] && pr.z[j] > 500 && pr.x[j] >= 100 && pr.x[j] < 100 * (W() - 1) && pr.y[j] >= 100 && pr.y[j] < 100 * (H() - 1)).collect();
            if ok.is_empty() { continue; }
            let ix = o.div(v(&ok.iter().map(|&j| pr.x[j]).collect()), K(100), 0);
            let iy = o.div(v(&ok.iter().map(|&j| pr.y[j]).collect()), K(100), 0);
            let pix: Vec<usize> = (0..ok.len()).map(|j| (iy[j] * W() + ix[j]) as usize).collect();
            let gz: V = pix.iter().map(|&p| g.iz[p]).collect();
            let iz: V = ok.iter().map(|&j| pr.iz[j]).collect();
            let behind = o.lt(v(&iz), v(&gz));
            let kk = fk(o, 2500, e, 1000);
            let vals: [i64; 3] = std::array::from_fn(|c| { let t = fk(o, sh[4 + c], fade, 1000); fk(o, t, kk, 1000) });
            // (at half resolution one pixel, 0.45 of it)
            let side: [i64; 3] = std::array::from_fn(|c| fk(o, vals[c], if res() == 1 { 400 } else { 450 }, 1000));
            let taps: &[(i64, i64, i32)] = if res() == 1 { &[(0, 0, 0), (1, 0, 1), (0, 1, 1)] } else { &[(0, 0, 1)] };
            let mut px = vec![];
            let mut vv: [V; 3] = [vec![], vec![], vec![]];
            for (j, &p) in pix.iter().enumerate() {
                if behind[j] == 1 { continue; }
                for &(dx, dy, wsel) in taps {
                    px.push((p as i64 + dy * W() + dx) as usize);
                    for c in 0..3 { vv[c].push(if wsel == 0 { vals[c] } else { side[c] }); }
                }
            }
            ov.put(o, &px, &vv);
        }
    }
}

/// The photo flashes: 36 a frame anywhere in the bowl (the printed draws for the frame), each a colour and one frame,
/// a bright small spot and a soft halo by its distance.
fn flashes<O: Ops>(o: &mut O, nw: &NightWorld, l: &NightLight, cam: &Cam, g: &GBuf, ov: &mut Over) {
    let tb = &nw.w.tb;
    let (f, e) = (l.f, l.e);
    let fl: Vec<&[i64; 7]> = nw.flashes.iter().filter(|q| q[0] == f).collect();
    if fl.is_empty() { return; }
    let r: V = fl.iter().map(|q| q[1]).collect();
    let a: V = fl.iter().map(|q| q[2]).collect();
    let at = { let t = mturns(o, &a); o.turns(v(&t)) };
    let (sa, ca) = o.sincos(v(&at));
    let xa = { let t = o.lin(&[(v(&r), Some(v(&ca)))]); let t = o.div(v(&t), K(999), 0); scaled(o, &t, 13_460, 1000) };
    let ya = { let t = o.lin(&[(v(&r), Some(v(&sa)))]); let t = o.div(v(&t), K(999), 0); scaled(o, &t, 11_460, 1000) };
    let pa: Vec<[i64; 3]> = (0..fl.len()).map(|k| [xa[k], ya[k], fl[k][3]]).collect();
    let pw = aud_film(o, nw, &pa);
    let pr = proj(o, cam, &pw);
    // its colour: 0.3 + 0.7 hue / max(hue)
    let hm = { let a_: V = fl.iter().map(|q| q[4]).collect(); let b_: V = fl.iter().map(|q| q[5]).collect(); let c_: V = fl.iter().map(|q| q[6]).collect(); let m = o.max(v(&a_), v(&b_)); let m = o.max(v(&m), v(&c_)); o.max(v(&m), K(1)) };
    let kk = fk(o, 2500, e, 1000);
    // (their pixels, on the machines)
    let (ix, iy) = (o.div(v(&pr.x), K(100), 0), o.div(v(&pr.y), K(100), 0));
    for k in 0..fl.len() {
        if !(pr.front[k] && pr.z[k] > 200 && pr.x[k] >= 0 && pr.x[k] < 100 * W() && pr.y[k] >= 0 && pr.y[k] < 100 * H()) { continue; }
        let p = (iy[k] * W() + ix[k]) as usize;
        let ok = { let (a, b) = (kl(o, &[(pr.iz[k], Some(100))]), kl(o, &[(g.iz[p], Some(97))])); one(o.lt(K(a), K(b))) };
        if ok == 1 { continue; }
        // its size (hundredths of a pixel): 0.45 f / z held to 1.2..9
        let rp = { let (a, z) = (kl(o, &[(cam.f2, Some(45 * 50))]), one(o.clamp(K(pr.z[k]), 1, 999_999))); one(o.div(K(a), K(z), 0)) };
        let rp = one(o.clamp(K(rp), 120 / res(), 900 / res()));
        let reach = one(o.lin(&[(K(rp), Some(K(4)))]));
        // its box: the pixels from its place less the reach to a pixel past its place and the reach (on the
        // machines; held to the screen by the host)
        let lo_ = o.sum(&[(v(&vec![pr.x[k], pr.y[k]]), 1), (K(reach), -1)], 0);
        let hi_ = o.sum(&[(v(&vec![pr.x[k], pr.y[k]]), 1), (K(reach), 1)], 100);
        let bx_ = o.div(v(&[lo_, hi_].concat()), K(100), 0);
        let (x0, x1, y0, y1) = (bx_[0], bx_[2], bx_[1], bx_[3]);
        let mut pix = vec![];
        for yy in y0.max(0)..y1.min(H()) { for xx in x0.max(0)..x1.min(W()) { pix.push((yy * W() + xx) as usize); } }
        if pix.is_empty() { continue; }
        let dx = o.tab(&nw.mid, v(&pix.iter().map(|&q| q as i64 % W()).collect()));
        let dy = o.tab(&nw.mid, v(&pix.iter().map(|&q| q as i64 / W()).collect()));
        let dx = o.lin(&[(v(&dx), None), (K(pr.x[k]), Some(K(-1)))]);
        let dy = o.lin(&[(v(&dy), None), (K(pr.y[k]), Some(K(-1)))]);
        let d2 = o.lin(&[(v(&dx), Some(v(&dx))), (v(&dy), Some(v(&dy)))]);
        // e^-(d^2 / rp^2) 9 + e^-(d^2 / 9 rp^2) 0.8
        let r2 = one(o.lin(&[(K(rp), Some(K(rp)))]));
        let r2d = { let q = one(o.div(K(r2), K(1000), 0)); one(o.clamp(K(q), 1, 999_999)) };
        let x1_ = o.div(v(&d2), K(r2d), 0);
        let x2_ = o.div(v(&x1_), K(9), 0);
        let g1 = expn(o, tb, &x1_);
        let g2 = expn(o, tb, &x2_);
        let gg = o.lin(&[(v(&g1), Some(K(9))), (v(&g2), Some(K(800)))]);
        let gg = o.div(v(&gg), K(1000), 0);
        let vals: [V; 3] = std::array::from_fn(|c| {
            let ck = one({ let t = o.lin(&[(K(fl[k][4 + c]), Some(K(700)))]); let t = o.div(v(&t), K(hm[k]), 0); o.lin(&[(v(&t), None), (K(300), None)]) });
            let t = o.lin(&[(v(&gg), Some(K(ck)))]);
            let t = o.div(v(&t), K(1000), 0);
            scaled(o, &t, kk, 1000)
        });
        ov.put(o, &pix, &vals);
    }
}

/// Each list's sum (any numbers): the lists' k-th values (0 past a list's end) added a column at a time.
pub fn group_sum<O: Ops>(o: &mut O, lists: &[Vec<i64>]) -> V {
    let m = lists.len();
    let longest = lists.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut acc = vec![0; m];
    for k in 0..longest {
        let at: Vec<usize> = (0..m).filter(|&i| lists[i].len() > k).collect();
        let cur: V = at.iter().map(|&i| acc[i]).collect();
        let add: V = at.iter().map(|&i| lists[i][k]).collect();
        let s = if k == 0 { add } else { o.lin(&[(v(&cur), None), (v(&add), None)]) };
        for (j, &i) in at.iter().enumerate() { acc[i] = s[j]; }
    }
    acc
}
