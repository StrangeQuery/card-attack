// The world drawn at half size, the subjects at the film's. Included after frame.rs.
//
// As the dive does (segments/deep): what is far and soft (the bowl, the tiers, the crowd's impostors beyond the near
// ring, the balloons, the airship's hull, the sky, the probes' light on all of them (coarse probes), the beams in
// the haze, the strobe's haze, the pyro, the lasers' glow, the bloom) is worked out at 768 x 384 (the same view, the
// focal length halved) and each of its pixels stands for the 2 x 2 of the film's under it; what must stay sharp
// (every screen with its pictures and words, the flags and poles, the airship's screen, the 40 near dancers, the
// lasers' lines and the glow within 4 pixels of them, the dots of light, Dope's stars) is drawn at 1536 x 768.
//
// The seams: a subject shows where it is nearer than the world's half-size depth under it (CARDIAC's comparison);
// the world under a subject is drawn without it, so a subject's edge meets the world behind it, never a half pixel
// of itself. The beams are worked out once at half size against the world's depth, and again (whether in front) at
// the half pixels holding a subject, against the subject's nearest there: a subject's pixel takes the beams in front
// of it, the world's pixels beside it the beams as the world has them (no dark rim round a dancer in a beam, no beam
// over it). The pyro's flame likewise (the depth it is hidden past kept for the subjects). The lasers' lines, the
// dots and the stars go on at full size against each pixel's own depth (the subject's or the world's).
//
// The pixels the full size touched (the subjects', the lasers' lines', the dots', the stars') are the film's own;
// every other pixel is its half-size pixel's (the tone curve worked out once for the four). The glow is the film's
// glow of that picture: each 4 x 4 block's light over 1.1 summed from the half pixels (each standing for those of its
// four the full size didn't touch) and the touched pixels, blurred, added to both.

/// The film's pixel's half-size pixel.
pub fn half_of(p: usize) -> usize { let (x, y) = (p % FULL_W as usize, p / FULL_W as usize); (y / 2) * (FULL_W as usize / 2) + x / 2 }

#[allow(clippy::too_many_arguments)]
pub fn party_hd<O: Ops>(o: &mut O, w: &World, cam: &Cam, sh: &Show, l: &Lists, ship_screen: Option<&Drawn>, now: &ScreensNow, pcube: &Cube, sd: &mut Sd, blimp: [i64; 3], spot_d: &[[i64; 3]], search_d: &[[i64; 3]]) -> Vec<u8> {
    let (sc, t) = (&w.sc, &w.t);
    let t12 = sh.t12;
    let nh = FULL_NPIX / 4;
    let empty = Pieces::new(5);
    let ship_p = l.ship.map_or(&empty, |d| &d.p);
    let scr_p = ship_screen.map_or(&empty, |d| &d.p);
    // ---- what each pixel sees: the world's at half size, the subjects' at full, the subject where it is nearer -----
    o.stage("visible");
    set_res(false);
    let srcs_w: [(&V, &Vec<u32>, Src_); 6] = [(&l.surf.iz, &l.surf.px, Src_::Surf), (&l.emit.iz, &l.emit.px, Src_::Emit), (&l.skin.iz, &l.skin.px, Src_::Skin), (&l.cr.pieces.iz, &l.cr.pieces.px, Src_::Sprite), (&l.bal.d.p.iz, &l.bal.d.p.px, Src_::Ball), (&ship_p.iz, &ship_p.px, Src_::Ship)];
    let (win_w, idx_w, dep_w) = resolve(o, &srcs_w);
    set_res(true);
    let srcs_s: [(&V, &Vec<u32>, Src_); 5] = [(&l.scr.iz, &l.scr.px, Src_::Screen), (&l.cloth.p.iz, &l.cloth.p.px, Src_::Cloth), (&l.poles.p.iz, &l.poles.p.px, Src_::Pole), (&scr_p.iz, &scr_p.px, Src_::Ship), (&l.near.d.p.iz, &l.near.d.p.px, Src_::Person)];
    let (win_s0, idx_s, dep_s) = resolve(o, &srcs_s);
    let cand: Vec<usize> = (0..FULL_NPIX).filter(|&p| win_s0[p].is_some()).collect();
    let subj: Vec<usize> = if cand.is_empty() { vec![] } else {
        let wz: V = cand.iter().map(|&p| dep_w[half_of(p)]).collect();
        let sz: V = cand.iter().map(|&p| dep_s[p]).collect();
        let nearer = o.lt(v(&wz), v(&sz));
        (0..cand.len()).filter(|&k| nearer[k] == 1).map(|k| cand[k]).collect()
    };
    let mut win_s: Vec<Option<Src_>> = vec![None; FULL_NPIX];
    for &p in &subj { win_s[p] = win_s0[p]; }
    // ---- the shading: the world at half size, the subjects at full ------------------------------------------------
    o.stage("shade");
    set_res(false);
    let mut col_h: [V; 3] = sky(o, w, cam, t12);
    stars(o, w, cam, &win_w, &mut col_h);
    o.stage("sh_surf");
    shade_pieces(o, w, cam, sh, l, now, pcube, sd, &win_w, &idx_w, &mut col_h);
    set_res(true);
    let mut col_s: [V; 3] = std::array::from_fn(|_| vec![0; FULL_NPIX]);
    let ls = Lists { surf: l.surf, emit: l.emit, skin: l.skin, scr: l.scr, cr: l.cr, cloth: l.cloth, poles: l.poles, bal: l.bal, ship: ship_screen, near: l.near };
    shade_pieces(o, w, cam, sh, &ls, now, pcube, sd, &win_s, &idx_s, &mut col_s);
    // ---- the beams: at half size against the world, and in front of the subjects where they are ------------------
    o.stage("beams");
    let mut has = vec![false; nh];
    let mut four: Vec<[i64; 4]> = vec![[0; 4]; nh];
    for &p in &subj {
        let h = half_of(p);
        has[h] = true;
        let (x, y) = (p % FULL_W as usize, p / FULL_W as usize);
        four[h][(y % 2) * 2 + x % 2] = dep_s[p];
    }
    let hs: Vec<usize> = (0..nh).filter(|&h| has[h]).collect();
    let mut izs = vec![0i64; nh];
    if !hs.is_empty() {
        // the subjects' nearest in each half pixel holding one
        let q: [V; 4] = std::array::from_fn(|k| hs.iter().map(|&h| four[h][k]).collect());
        let m1 = o.max(v(&q[0]), v(&q[1]));
        let m2 = o.max(v(&q[2]), v(&q[3]));
        let m = o.max(v(&m1), v(&m2));
        for (k, &h) in hs.iter().enumerate() { izs[h] = m[k]; }
    }
    let bl = beam_list(o, sc, sh, spot_d, search_d);
    let (acc, touched, acc_s) = volumetric(o, t, &w.bt, &w.lz, cam, &bl, &dep_w, Some((&izs, &has)));
    set_res(false);
    let mut fh = Frame { col: col_h, iz: dep_w, hit: vec![] };
    let tp: Vec<usize> = (0..nh).filter(|&h| touched[h]).collect();
    if !tp.is_empty() {
        for c in 0..3 { let a: V = tp.iter().map(|&h| acc[c][h]).collect(); add_to(o, &mut fh.col[c], &tp, &a); }
    }
    strobe_haze(o, t, &mut fh, sh.strobe);
    o.stage("pyro");
    let mut fp = Frame { col: std::array::from_fn(|_| vec![0; nh]), iz: fh.iz.clone(), hit: vec![false; nh] };
    let mut lims = vec![0i64; nh];
    pyro(o, t, &w.lz, &mut fp, cam, sc, sh, &mut lims);
    let pp: Vec<usize> = (0..nh).filter(|&h| fp.hit[h]).collect();
    if !pp.is_empty() {
        for c in 0..3 { let a: V = pp.iter().map(|&h| fp.col[c][h]).collect(); add_to(o, &mut fh.col[c], &pp, &a); }
    }
    // ---- the film's size: every pixel its half pixel's, the subjects' their own ---------------------------------
    set_res(true);
    let mut ff = Frame {
        col: std::array::from_fn(|c| (0..FULL_NPIX).map(|p| fh.col[c][half_of(p)]).collect()),
        iz: (0..FULL_NPIX).map(|p| fh.iz[half_of(p)]).collect(),
        hit: vec![false; FULL_NPIX],
    };
    if !subj.is_empty() {
        o.stage("subjects");
        // the subject's light, the beams in front of it, the strobe's haze at its depth, the flame where in front
        let hz: Vec<usize> = subj.iter().map(|&p| half_of(p)).collect();
        let zs: V = subj.iter().map(|&p| dep_s[p]).collect();
        let haze = if sh.strobe != 0 { Some(o.tab(&t.haze_k, v(&zs))) } else { None };
        let fl: Vec<usize> = (0..subj.len()).filter(|&k| fp.hit[hz[k]]).collect();
        let fl_on = if fl.is_empty() { vec![] } else {
            let z: V = fl.iter().map(|&k| zs[k]).collect();
            let lim: V = fl.iter().map(|&k| lims[hz[k]]).collect();
            o.lt(v(&z), v(&lim))
        };
        for (c, k) in [900i64, 920, 1000].iter().enumerate() {
            let base: V = subj.iter().map(|&p| col_s[c][p]).collect();
            let b: V = hz.iter().map(|&h| acc_s[c][h]).collect();
            let mut terms = vec![(v(&base), 1), (v(&b), 1)];
            let hk = haze.as_ref().map(|h| { let a = o.lin(&[(v(h), Some(K(*k)))]); o.div(v(&a), K(1000), 0) });
            if let Some(h) = &hk { terms.push((v(h), 1)); }
            let fla_ = if fl.is_empty() { None } else {
                // the flame's light where it is in front (the host's pick by the flag), 0 elsewhere
                let a: V = fl.iter().map(|&kk| fp.col[c][hz[kk]]).collect();
                let s_ = o.sel(&fl_on, v(&a), K(0));
                let mut full = vec![0i64; subj.len()];
                for (j, &kk) in fl.iter().enumerate() { full[kk] = s_[j]; }
                Some(full)
            };
            if let Some(f) = &fla_ { terms.push((v(f), 1)); }
            let s_ = o.sum(&terms, 0);
            for (q, &p) in subj.iter().enumerate() { ff.col[c][p] = s_[q]; }
        }
        for &p in &subj { ff.iz[p] = dep_s[p]; }
        ff.mark(&subj);
    }
    // ---- the lights in the air at full size --------------------------------------------------------------------
    o.stage("dots");
    city(o, t, &mut ff, cam, sc);
    blimp_lights(o, t, &mut ff, cam, sc, sh, blimp);
    phones(o, t, &mut ff, cam, sc, l.cr, sh);
    flashes(o, t, &mut ff, cam, sc, sh);
    confetti(o, t, &mut ff, cam, sc, sh);
    o.stage("lasers");
    // the core at full size (its pixels the film's); the glow at half size, after the full size took its copy (the
    // core's pixels don't take it twice), for the pixels round it
    lasers(o, t, &w.lz, &mut ff, cam, sc, sh, 1);
    set_res(false);
    lasers(o, t, &w.lz, &mut fh, cam, sc, sh, 2);
    set_res(true);
    o.stage("bloom");
    bloom_hd(o, t, &mut fh, &mut ff);
    o.stage("dstars");
    let winc: Vec<Option<Src_>> = (0..FULL_NPIX).map(|p| if win_s[p].is_some() { win_s[p] } else { win_w[half_of(p)] }).collect();
    let st = dope_stars(o, w, cam, &winc, &mut ff.col);
    ff.mark(&st);
    // ---- the tone curve: the half size's once, the touched pixels' their own ---------------------------------------
    o.stage("tone");
    set_res(false);
    let bh = tone(o, t, &fh.col);
    set_res(true);
    let hp: Vec<usize> = (0..FULL_NPIX).filter(|&p| ff.hit[p]).collect();
    let mut out = vec![0u8; FULL_NPIX * 3];
    for p in 0..FULL_NPIX { let h = half_of(p); out[3 * p..3 * p + 3].copy_from_slice(&bh[3 * h..3 * h + 3]); }
    if !hp.is_empty() {
        for c in 0..3 {
            let x: V = hp.iter().map(|&p| ff.col[c][p]).collect();
            let x = o.clamp(v(&x), 0, TONE_N);
            let b = o.tab(&t.tone, v(&x));
            for (q, &p) in hp.iter().enumerate() { out[3 * p + c] = b[q] as u8; }
        }
    }
    if std::env::var("STADIUM_DEBUG").is_ok() { eprintln!("party: {} subject pixels, {} full-size pixels in all", subj.len(), hp.len()); }
    out
}

/// The glow of the picture: each 4 x 4 block's light over 1.1 from the half pixels (each standing for those of its
/// four not touched at full size) and the touched full-size pixels; blurred three times each way by 5 (the edges
/// held, each blur only round the blocks with any glow); 0.8 of it added to the half pixels and the touched pixels in
/// the blocks with any. Means by logarithms (a sum's ln less ln 16 or ln 5).
/// (The touched pixels' copies of their half pixels brought up to date after, for the stars.)
pub fn bloom_hd<O: Ops>(o: &mut O, t: &Tables, fh: &mut Frame, ff: &mut Frame) {
    let (fw, hw_) = (FULL_W as usize, FULL_W as usize / 2);
    let (bw, bh) = (fw / 4, FULL_H as usize / 4);
    let nb = bw * bh;
    let nh = fh.col[0].len();
    let hblock = |h: usize| -> usize { ((h / hw_) / 2) * bw + (h % hw_) / 2 };
    let fblock = |p: usize| -> usize { ((p / fw) / 4) * bw + (p % fw) / 4 };
    let hp: Vec<usize> = (0..FULL_NPIX).filter(|&p| ff.hit[p]).collect();
    // how many of each half pixel's four are its own (not touched)
    let mut own = vec![4i64; nh];
    for &p in &hp { own[half_of(p)] -= 1; }
    for c in 0..3 {
        let over = o.lt(K(11_000), v(&fh.col[c]));
        let bright: Vec<usize> = (0..nh).filter(|&h| over[h] == 1 && own[h] > 0).collect();
        let fv: V = hp.iter().map(|&p| ff.col[c][p]).collect();
        let fover = if hp.is_empty() { vec![] } else { o.lt(K(11_000), v(&fv)) };
        let fbright: Vec<usize> = (0..hp.len()).filter(|&k| fover[k] == 1).collect();
        if bright.is_empty() && fbright.is_empty() { continue; }
        let mut small = vec![0i64; nb];
        let mut live = vec![false; nb];
        if !bright.is_empty() {
            // a half pixel's excess times its own count over 16 (by logarithms: ln 16 less the count's, as printed)
            let x: V = bright.iter().map(|&h| fh.col[c][h]).collect();
            let x = o.sum(&[(v(&x), 1)], -11_000);
            let l = o.tabo(&t.ln, 999_999, v(&x));
            let k: V = bright.iter().map(|&h| -lnf(16.0 / own[h] as f64)).collect();
            let y = o.sum(&[(v(&l), 1), (v(&k), 1)], 0);
            let x = expv(o, t, &y, None);
            let bpx: Vec<usize> = bright.iter().map(|&h| hblock(h)).collect();
            add_to(o, &mut small, &bpx, &x);
            for &b in &bpx { live[b] = true; }
        }
        if !fbright.is_empty() {
            let x: V = fbright.iter().map(|&k| fv[k]).collect();
            let x = o.sum(&[(v(&x), 1)], -11_000);
            let x = mean_ln(o, t, &x, 16);
            let bpx: Vec<usize> = fbright.iter().map(|&k| fblock(hp[k])).collect();
            add_to(o, &mut small, &bpx, &x);
            for &b in &bpx { live[b] = true; }
        }
        // each lit block held to 180,000 (far past white), so five of them sum on a card (the means rounding up a little) (the white envelope under
        // the searchlights)
        let lit: Vec<usize> = (0..nb).filter(|&b| live[b]).collect();
        let held = o.min(v(&lit.iter().map(|&b| small[b]).collect::<V>()), K(180_000));
        for (q, &b) in lit.iter().enumerate() { small[b] = held[q]; }
        for _ in 0..3 {
            for axis in 0..2 {
                let (step, len) = if axis == 0 { (1usize, bw) } else { (bw, bh) };
                let pos = |i: usize| -> i64 { (if axis == 0 { i % bw } else { i / bw }) as i64 };
                let mut next = live.clone();
                for i in 0..nb { if live[i] { for k in -2..=2i64 { let q = (pos(i) + k).clamp(0, len as i64 - 1) - pos(i); next[(i as i64 + q * step as i64) as usize] = true; } } }
                live = next;
                let idx: Vec<usize> = (0..nb).filter(|&b| live[b]).collect();
                let nbv: Vec<V> = (-2..=2i64).map(|k| idx.iter().map(|&i| { let q = (pos(i) + k).clamp(0, len as i64 - 1) - pos(i); small[(i as i64 + q * step as i64) as usize] }).collect()).collect();
                let terms: Vec<(A, i64)> = nbv.iter().map(|x| (v(x), 1)).collect();
                let s = o.sum(&terms, 0);
                let m = mean_ln(o, t, &s, 5);
                let mut ns = vec![0i64; nb];
                for (q, &b) in idx.iter().enumerate() { ns[b] = m[q]; }
                small = ns;
            }
        }
        // 0.8 of it, once a block, added where it is
        let lb: Vec<usize> = (0..nb).filter(|&b| live[b]).collect();
        let big: V = lb.iter().map(|&b| small[b]).collect();
        let a = { let l = o.tabo(&t.ln, 999_999, v(&big)); let y = o.sum(&[(v(&l), 1)], lnf(0.8)); expv(o, t, &y, None) };
        let mut add = vec![0i64; nb];
        for (q, &b) in lb.iter().enumerate() { add[b] = a[q]; }
        let px: Vec<usize> = (0..nh).filter(|&h| live[hblock(h)]).collect();
        let old: V = px.iter().map(|&h| fh.col[c][h]).collect();
        let ad: V = px.iter().map(|&h| add[hblock(h)]).collect();
        let s = o.sum(&[(v(&old), 1), (v(&ad), 1)], 0);
        for (q, &h) in px.iter().enumerate() { fh.col[c][h] = s[q]; }
        let fp: Vec<usize> = hp.iter().copied().filter(|&p| live[fblock(p)]).collect();
        if !fp.is_empty() {
            let old: V = fp.iter().map(|&p| ff.col[c][p]).collect();
            let ad: V = fp.iter().map(|&p| add[fblock(p)]).collect();
            let s = o.sum(&[(v(&old), 1), (v(&ad), 1)], 0);
            for (q, &p) in fp.iter().enumerate() { ff.col[c][p] = s[q]; }
        }
    }
    for p in 0..FULL_NPIX { if !ff.hit[p] { let h = half_of(p); for c in 0..3 { ff.col[c][p] = fh.col[c][h]; } } }
}
