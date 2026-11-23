// The crowd: 65,536 people, each frame their moves (the wave rolling back over the pitch every phrase, the Mexican
// wave round the tiers, the beat's phase), where each one is on the screen, the nearest drawn as their posed meshes
// and everyone else as an impostor (their pose's picture from behind, upright, turned to the camera), and the phones
// held up. Included after raster.rs.

/// The nearest dancers in view drawn as meshes, within 26 m (the rest the
/// impostors, for the budget).
pub const NEAR_N: usize = 64;
/// The party version's (its budget): the nearest 40.
pub const NEAR_N_HD: usize = 40;
pub const NEAR_CM: i64 = 2600;

pub struct Crowd {
    /// each person's move and phase this frame
    pub mv: Vec<u8>,
    pub ph: Vec<u8>,
    /// the camera-space place of each person's feet (centimetres), whether in view
    pub cam: Vec<[i64; 3]>,
    pub vis: Vec<bool>,
    /// the near ones (meshes), nearest first
    pub near: Vec<usize>,
    /// the impostors
    pub sprites: Vec<Sprite>,
    pub pieces: SpritePieces,
}

#[derive(Clone, Debug)]
pub struct Sprite {
    pub person: usize,
    /// the pose's picture (model, move, phase), and whether from the finer atlas
    pub k: usize,
    pub hi: bool,
    /// the direction to the eye along the ground (thousandths): across is (-to_y, to_x)
    pub to: [i64; 2],
    /// the eye's elevation seen from the person: cosine, sine (thousandths)
    pub el: [i64; 2],
}

/// The impostors' pieces on pixels: the pixel, K / z, the sprite, the texel (row, column).
#[derive(Default)]
pub struct SpritePieces { pub px: Vec<u32>, pub iz: V, pub sid: Vec<u32>, pub ty: Vec<u16>, pub tx: Vec<u16> }

/// Each person's move and phase at t12: on the pitch a band of arms up rolls from the stage to the back every phrase
/// (4 bars), on the tiers a Mexican wave goes round; everyone's phase on the beat, give or take a frame.
pub fn moves<O: Ops>(o: &mut O, sc: &Scene, t12: i64) -> (Vec<u8>, Vec<u8>) {
    let n = sc.cpos.len();
    let mut mv: Vec<u8> = sc.cmove.iter().map(|&m| m as u8).collect();
    let phr = { let a = cycle(t12, C_PHRASE, o); frac1m(a, o) };
    let wave_y = s_sum(&[(s_div(s_lin(&[(phr, Some(14))], o), 1000, o), -1)], 4000, o);
    let pitch: Vec<usize> = (0..n).filter(|&i| sc.ckind[i] == 0 && sc.in_crowd(i)).collect();
    let tier: Vec<usize> = (0..n).filter(|&i| sc.ckind[i] == 1 && sc.in_crowd(i)).collect();
    let cy: V = pitch.iter().map(|&i| sc.cpos[i][1]).collect();
    let d = o.sum(&[(v(&cy), 1)], -wave_y);
    let (band, _) = o.absless(&d, K(700));
    for (k, &i) in pitch.iter().enumerate() { if band[k] == 1 { mv[i] = if sc.cmove[i] == RAISE { RAISE as u8 } else { 1 }; } }
    let wave_a = s_sum(&[(angle(t12, rate(1.1), 0, o), 1)], -250_000, o);
    // the angle from the wave, in two-millionths of a turn (so the difference stays a long), wrapped to a half turn
    // either way
    let wave2 = s_div(frac1m(wave_a, o), 2, o);
    let ang: V = tier.iter().map(|&i| sc.cang[i]).collect();
    let x = o.sum(&[(v(&ang), 1)], -wave2);
    let f1 = o.lt(v(&x), K(-250_000));
    let xp = o.sum(&[(v(&x), 1)], 500_000);
    let x = o.sel(&f1, v(&xp), v(&x));
    let f2 = o.lt(K(249_999), v(&x));
    let xm = o.sum(&[(v(&x), 1)], -500_000);
    let x = o.sel(&f2, v(&xm), v(&x));
    let (wave, _) = o.absless(&x, K(17_507));
    for (k, &i) in tier.iter().enumerate() { if wave[k] == 1 { mv[i] = 0; } }
    // the beat's phase: floor((t12 - BEAT_T0) 6 / 5.250109), and each one's offset (-1, 0 or 1)
    let pb = { let a = s_lin(&[(t12, Some(1_142_833)), (-beat_off(1_142_833), None)], o); s_div(s_div(a, 1000, o), 1000, o) };
    let phs: [i64; 3] = std::array::from_fn(|k| s_divmod(s_sum(&[(pb, 1)], k as i64 - 1, o), NPHASE, o).1);
    let ph = (0..n).map(|i| phs[(sc.cphase[i] + 1) as usize] as u8).collect();
    (mv, ph)
}

/// The picture of a pose: (model, move, phase) in the atlases' order.
pub fn pose_k(model: i64, mv: u8, ph: u8) -> usize { ((model as usize) * MOVES + mv as usize) * NPHASE as usize + ph as usize }

pub fn crowd<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, sc: &Scene, cam: &Cam, t12: i64) -> Crowd {
    let n = sc.cpos.len();
    let (mv, ph) = moves(o, sc, t12);
    // everyone's feet in the camera's space, and the middle (a metre up): in view? (the party version: its crowd's)
    let all: Vec<usize> = (0..n).filter(|&i| sc.in_crowd(i)).collect();
    let fa = to_cam(o, t, cam, &all.iter().map(|&i| sc.cpos[i]).collect::<Vec<_>>());
    let mut feet = vec![[0i64; 3]; n];
    for (k, &i) in all.iter().enumerate() { feet[i] = fa[k]; }
    let up: [i64; 3] = std::array::from_fn(|r| s_div(cam.b[r][2], 10, o));
    let col = |i: usize| -> V { fa.iter().map(|p| p[i]).collect() };
    let (x, y, z) = (col(0), col(1), col(2));
    let mx = o.sum(&[(v(&x), 1)], up[0]);
    let my = o.sum(&[(v(&y), 1)], up[1]);
    let mz = o.sum(&[(v(&z), 1)], up[2]);
    let infront = o.lt(K(30), v(&mz));
    let hx = { let p = o.lin(&[(v(&mz), Some(K(768)))]); let p = o.div(v(&p), K(1000), 0); o.sum(&[(v(&p), 1)], 250) };
    let hy = { let p = o.lin(&[(v(&mz), Some(K(384)))]); let p = o.div(v(&p), K(1000), 0); o.sum(&[(v(&p), 1)], 250) };
    let (inx, _) = o.absless(&mx, A::V(&hx));
    let (iny, _) = o.absless(&my, A::V(&hy));
    let mut vis = vec![false; n];
    for (k, &i) in all.iter().enumerate() { vis[i] = infront[k] == 1 && inx[k] == 1 && iny[k] == 1; }
    // the near ones: within 26 m each way, then by the distance squared (decimetres)
    let seen: Vec<usize> = (0..n).filter(|&i| vis[i]).collect();
    let rel: [V; 3] = std::array::from_fn(|c| { let p: V = seen.iter().map(|&i| sc.cpos[i][c]).collect(); let e = s_sum(&[(cam.eye[c], -1)], if c == 2 { 100 } else { 0 }, o); o.sum(&[(v(&p), 1)], e) });
    let close: [V; 3] = std::array::from_fn(|c| o.absless(&rel[c], K(NEAR_CM)).0);
    let cand: Vec<usize> = (0..seen.len()).filter(|&k| (0..3).all(|c| close[c][k] == 1)).collect();
    let dm: [V; 3] = std::array::from_fn(|c| { let p: V = cand.iter().map(|&k| rel[c][k]).collect(); o.div(v(&p), K(10), 0) });
    let sq: [V; 3] = std::array::from_fn(|c| o.tabo(&lt.sq, 5000, v(&dm[c])));
    let d2 = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
    let within = o.lt(v(&d2), K(NEAR_CM * NEAR_CM / 100));
    // (put in order on the card sorter; the dancers come in by number, and equal distances keep that order)
    let mut near: Vec<(i64, usize)> = (0..cand.len()).filter(|&j| within[j] == 1).map(|j| (d2[j], seen[cand[j]])).collect();
    sorter::sort(&mut near, &[sorter::up(6)], |t| [t.0]);
    let near: Vec<usize> = near.into_iter().take(if light_hd() { NEAR_N_HD } else { NEAR_N }).map(|x| x.1).collect();
    if std::env::var("STADIUM_DEBUG").is_ok() { eprintln!("near {:?}", near.iter().take(8).collect::<Vec<_>>()); }
    let mut is_near = vec![false; n];
    for &i in &near { is_near[i] = true; }
    let (sprites, pieces) = sprites(o, t, lt, sc, cam, &feet, &vis, &is_near, &mv, &ph);
    Crowd { mv, ph, cam: feet, vis, near, sprites, pieces }
}

/// The ground direction from places to the eye (thousandths), and the ln of the ground distance (half-metres).
pub fn toward_eye<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, cam: &Cam, px: &V, py: &V) -> ([V; 2], V) {
    let dx = o.sum(&[(v(px), -1)], cam.eye[0]);
    let dy = o.sum(&[(v(py), -1)], cam.eye[1]);
    let hx = divk(o, &dx, 50);
    let hy = divk(o, &dy, 50);
    let hx = o.clamp(v(&hx), -707, 707);
    let hy = o.clamp(v(&hy), -707, 707);
    let sx = o.tabo(&lt.sq, 5000, v(&hx));
    let sy = o.tabo(&lt.sq, 5000, v(&hy));
    let d2 = o.sum(&[(v(&sx), 1), (v(&sy), 1)], 0);
    let d2 = o.clamp(v(&d2), 0, 999_999);
    let ld = o.tab(&lt.ln_d, v(&d2));
    let part = |o: &mut O, h: &V| -> V { let (l, ng) = lnv(o, t, h); let y = o.sum(&[(v(&l), 1), (v(&ld), -1)], LN1000); let e = expv(o, t, &y, Some(&ng)); o.clamp(v(&e), -999, 999) };
    ([part(o, &hx), part(o, &hy)], ld)
}

/// The impostors: each a picture of its pose, upright, its width turned across the line to the eye, from its feet
/// (less 10 cm) up 2.6 m and 1.5 m wide (times the person's size). Its feet's two corners and its top's middle on the
/// screen (the top's corners as the feet's, shifted: a parallelogram), the rows between walked for the left edge, the
/// depth and the picture's row, the columns for the picture's column; every covered texel a piece.
fn sprites<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, sc: &Scene, cam: &Cam, feet: &[[i64; 3]], vis: &[bool], is_near: &[bool], mv: &[u8], ph: &[u8]) -> (Vec<Sprite>, SpritePieces) {
    let ids: Vec<usize> = (0..feet.len()).filter(|&i| vis[i] && !is_near[i]).collect();
    let m = ids.len();
    let mut out = SpritePieces::default();
    if m == 0 { return (vec![], out); }
    let ks: Vec<usize> = ids.iter().map(|&i| pose_k(sc.cmodel[i], mv[i], ph[i])).collect();
    // the lift (the pose's, centimetres) less 10 cm; the size
    let lift: V = ids.iter().zip(&ks).map(|(&i, &k)| { let p = &sc.poses[sc.cmodel[i] as usize]; p.lift[k % (MOVES * NPHASE as usize)] }).collect();
    let lift = o.sum(&[(v(&lift), 1)], -10);
    let scale: V = ids.iter().map(|&i| sc.cscale[i]).collect();
    let half_w = { let p = o.lin(&[(v(&scale), Some(K(75)))]); o.div(v(&p), K(1000), 0) };
    let tall = { let p = o.lin(&[(v(&scale), Some(K(260)))]); o.div(v(&p), K(1000), 0) };
    // the direction to the eye along the ground; across = (-to_y, to_x)
    let px: V = ids.iter().map(|&i| sc.cpos[i][0]).collect();
    let py: V = ids.iter().map(|&i| sc.cpos[i][1]).collect();
    let (to, lnd) = toward_eye(o, t, lt, cam, &px, &py);
    // the camera-space foot (the feet raised by the lift), the top (up the person's height), the across (half width)
    let bz: [i64; 3] = std::array::from_fn(|r| cam.b[r][2]);
    let foot: [V; 3] = std::array::from_fn(|r| { let fz: V = ids.iter().map(|&i| feet[i][r]).collect(); let l = o.lin(&[(v(&lift), Some(K(bz[r])))]); let l = o.div(v(&l), K(1000), 0); o.sum(&[(v(&fz), 1), (v(&l), 1)], 0) });
    let top: [V; 3] = std::array::from_fn(|r| { let l = o.lin(&[(v(&tall), Some(K(bz[r])))]); let l = o.div(v(&l), K(1000), 0); o.sum(&[(v(&foot[r]), 1), (v(&l), 1)], 0) });
    let across: [V; 3] = std::array::from_fn(|r| {
        // B (across) times the half width: (b_r0 (-to_y) + b_r1 to_x) w / 10^6
        let a = { let n = s_neg(cam.b[r][0], o); o.lin(&[(v(&to[1]), Some(K(n))), (v(&to[0]), Some(K(cam.b[r][1])))]) };
        let a = o.div(v(&a), K(1000), 0);
        let a = o.lin(&[(v(&a), Some(v(&half_w)))]);
        o.div(v(&a), K(1000), 0)
    });
    let left: [V; 3] = std::array::from_fn(|r| o.sum(&[(v(&foot[r]), 1), (v(&across[r]), -1)], 0));
    let right: [V; 3] = std::array::from_fn(|r| o.sum(&[(v(&foot[r]), 1), (v(&across[r]), 1)], 0));
    // all three in front of the near plane, and none of them more than 9 times its depth off to the side or up (far
    // off the screen: at the crowd's height a person beside the eye would project past a long), else left out
    let zs = [&left[2], &right[2], &top[2]];
    let mut ok: Vec<V> = zs.iter().map(|z| o.lt(K(NEAR - 1), v(z))).collect();
    for c in [&left, &right, &top] {
        for (a, s) in [(0, 1), (0, -1), (1, 1), (1, -1)] {
            let m = o.sum(&[(v(&c[2]), 9), (v(&c[a]), s)], 0);
            ok.push(o.lt(K(0), v(&m)));
        }
    }
    let keep: Vec<usize> = (0..m).filter(|&j| ok.iter().all(|f| f[j] == 1)).collect();
    let pts = |c: &[V; 3]| -> Vec<[i64; 3]> { keep.iter().map(|&j| [c[0][j], c[1][j], c[2][j]]).collect() };
    let pl = project(o, t, &pts(&left));
    let pr = project(o, t, &pts(&right));
    let pt = project(o, t, &pts(&top));
    let nk = keep.len();
    // the rows: from the top's to the feet's (pixel rows whose middles lie between)
    let yt: V = pt.iter().map(|p| p.1).collect();
    let yl: V = pl.iter().map(|p| p.1).collect();
    let yr: V = pr.iter().map(|p| p.1).collect();
    let yf = o.max(v(&yl), v(&yr));
    let r0 = { let a = o.sum(&[(v(&yt), 1)], 49); o.divmod(v(&a), K(100)).0 };
    let r1 = { let a = o.sum(&[(v(&yf), 1)], 49); o.divmod(v(&a), K(100)).0 };
    let rows = o.sum(&[(v(&r1), 1), (v(&r0), -1)], 0);
    // the width (hundredths) and the pixels across
    let xl: V = pl.iter().map(|p| p.0).collect();
    let xr: V = pr.iter().map(|p| p.0).collect();
    let wid = o.sum(&[(v(&xr), 1), (v(&xl), -1)], 0);
    let wid = o.max(v(&wid), K(0));
    let ncol = { let a = o.sum(&[(v(&wid), 1)], 50); o.divmod(v(&a), K(100)).0 };
    // the left edge at the top: the top's middle less half the width
    let half = divk(o, &wid, 2);
    let xlt: V = { let xt: V = pt.iter().map(|p| p.0).collect(); o.sum(&[(v(&xt), 1), (v(&half), -1)], 0) };
    let live: Vec<usize> = (0..nk).filter(|&j| rows[j] > 0 && ncol[j] > 0).collect();
    let g = |x: &V| -> V { live.iter().map(|&j| x[j]).collect() };
    let (rows_l, ncol_l, xlt_l, xl_l) = (g(&rows), g(&ncol), g(&xlt), g(&xl));
    let izt: V = live.iter().map(|&j| pt[j].2).collect();
    let izl: V = live.iter().map(|&j| pl[j].2).collect();
    let r0_l = g(&r0);
    // the finer atlas for an impostor over 96 rows
    let big = o.lt(K(96 * scr_h() / FULL_H), v(&rows_l));
    // the left edge, the depth and the picture's row down the rows (from the top's toward the feet's by the slope
    // over rows + 1); the picture's column across
    let n1: V = rows_l.iter().map(|&r| r).collect();
    let n1 = o.sum(&[(v(&n1), 1)], 1);
    let sx = slopes(o, t, &xlt_l, &xl_l, &n1);
    let xs = walk_from(o, &xlt_l, &sx, &rows_l);
    // each impostor's depth: its middle's (a card at a person's place is as good as upright and square to the eye)
    let izm = { let a = o.sum(&[(v(&izt), 1), (v(&izl), 1)], 0); divk(o, &a, 2) };
    let ah: V = big.iter().map(|&b| if b == 1 { 288 } else { 96 }).collect();
    let aw: V = big.iter().map(|&b| if b == 1 { 168 } else { 56 }).collect();
    let zero = vec![0; live.len()];
    let sty = slopes(o, t, &zero, &ah, &rows_l);
    let tys = walk_half(o, &sty, &rows_l);
    let stx = slopes(o, t, &zero, &aw, &ncol_l);
    let txs = walk_half(o, &stx, &ncol_l);
    // held inside their pictures
    let ahm: V = (0..live.len()).flat_map(|q| std::iter::repeat_n(ah[q] - 1, rows_l[q] as usize)).collect();
    let awm: V = (0..live.len()).flat_map(|q| std::iter::repeat_n(aw[q] - 1, ncol_l[q] as usize)).collect();
    let tys = o.min(v(&tys), v(&ahm));
    let txs = o.min(v(&txs), v(&awm));
    // the first pixel at or right of the left edge, each row
    let pxl = { let a = o.sum(&[(v(&xs), 1)], 49); o.divmod(v(&a), K(100)).0 };
    // the sprites; their pieces nearest first (the card sorter puts them in order by their depths), every opaque
    // texel's piece kept (which is nearest on each pixel is CARDIAC's nearest, in `resolve`)
    let mut sp = vec![];
    let mut sp_j = vec![];
    let (mut r_at, mut c_at) = (vec![0usize; live.len()], vec![0usize; live.len()]);
    {
        let (mut ar, mut ac) = (0usize, 0usize);
        for (q, &j) in live.iter().enumerate() {
            let jj = keep[j];
            sp.push(Sprite { person: ids[jj], k: ks[jj], hi: big[q] == 1, to: [to[0][jj], to[1][jj]], el: [0, 0] });
            sp_j.push(jj);
            r_at[q] = ar;
            c_at[q] = ac;
            ar += rows_l[q] as usize;
            ac += ncol_l[q] as usize;
        }
    }
    let mut order: Vec<usize> = (0..live.len()).collect();
    sorter::sort(&mut order, &[sorter::down(6)], |&q| [izm[q]]);
    let hw = scr_w() / 2;
    for &q in &order {
        let s = &sp[q];
        let atl = if s.hi { &sc.atlas_hi } else { &sc.atlas };
        let (ar, ac) = (r_at[q], c_at[q]);
        for r in 0..rows_l[q] as usize {
            let row = r0_l[q] + r as i64;
            if row < -scr_h() / 2 || row >= scr_h() / 2 { continue; }
            let ty = tys[ar + r] as usize;
            for c in 0..ncol_l[q] as usize {
                let x = pxl[ar + r] + c as i64;
                if x < -hw || x >= hw { continue; }
                let p = ((row + scr_h() / 2) * scr_w() + x + hw) as usize;
                let tx = txs[ac + c] as usize;
                if atl.ats(s.k, ty, tx, 0) == 0 { continue; }
                out.px.push(p as u32);
                out.iz.push(izm[q]);
                out.sid.push(q as u32);
                out.ty.push(ty as u16);
                out.tx.push(tx as u16);
            }
        }
    }
    // the eye's elevation from each sprite's middle (a metre up): tan = dz / the ground distance, by logarithms, the
    // cosine and sine from the printed tables
    let mids: Vec<usize> = sp.iter().map(|s| s.person).collect();
    let dz: V = mids.iter().map(|&i| sc.cpos[i][2]).collect();
    let dz = { let e = s_sum(&[(cam.eye[2], 1)], -100, o); o.sum(&[(v(&dz), -1)], e) };
    let dzh = divk(o, &dz, 50);
    let lnds: V = sp_j.iter().map(|&jj| lnd[jj]).collect();
    let (lz, ng) = lnv(o, t, &dzh);
    let r = o.sum(&[(v(&lz), 1), (v(&lnds), -1)], LN1000);
    let r = o.clamp(v(&r), -300_000, 160_000);
    let ratio = expv(o, t, &r, None);
    let ratio = o.min(v(&ratio), K(99_999));
    let ce = o.tab(&t.cos_atan, v(&ratio));
    let se = o.tab(&t.sin_atan, v(&ratio));
    let nse = o.sum(&[(v(&se), -1)], 0);
    for (k, s) in sp.iter_mut().enumerate() { s.el = [ce[k], if ng[k] == 1 { nse[k] } else { se[k] }]; }
    (sp, out)
}

/// Walks from 0 by slopes, each place's value at its middle: fl(a / 2 + a k, 1000).
pub fn walk_half<O: Ops>(o: &mut O, a: &V, n: &V) -> V {
    let h = divk(o, a, 2);
    let walks: Vec<(i64, i64, i64, i64)> = (0..a.len()).filter(|&j| n[j] > 0).map(|j| (n[j], fl(h[j], 1000), a[j], md(h[j], 1000))).collect();
    o.walka(&walks)
}

impl Tables {
    /// cos and sin of atan(r / 1000) for r 0..99,999 (thousandths)
    pub fn atan_tabs() -> (Tab, Tab) {
        let c = (0..100_000).map(|r| ((r as f64 / 1000.0).atan().cos() * 1000.0).round() as i64).collect();
        let s = (0..100_000).map(|r| ((r as f64 / 1000.0).atan().sin() * 1000.0).round() as i64).collect();
        (tab("cos_atan", c), tab("sin_atan", s))
    }
}
