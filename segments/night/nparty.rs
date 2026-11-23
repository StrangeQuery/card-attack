// The party version (night/video_hd.rgb, what the live cut plays): the world drawn at half the resolution (768x384:
// the town, the fields, the street, the sky, the lamps' light, the beams and haze, the motion blur and the bloom),
// made up to 1536x768, and over it at full resolution what has to read across a room: the callouts' screens and
// boards, the station's board, the airship's screen, our own train; and the stars, a pixel each. Included after
// nseg.rs.
//
// The half-resolution render has the subjects in it too (their glow, their blur, the beams in front of them); its
// buffer keeps the world's own depth behind them. At full resolution the subjects' triangles are drawn again and
// each pixel of theirs is kept where it is nearer than that depth (the half-resolution pixel it lies in); kept
// pixels are shaded at full resolution and take the light the post laid on their half-resolution pixel (the beams
// in front of the subject, never those behind it) and its bloom, then the tone. Where a subject streaks (its blur
// over a pixel there), its half-resolution picture (blurred) is mixed in, all of it by five. The world is made up to
// full size from its half-resolution picture by its own pixels only: those a subject covered take their nearest
// world pixel's colour first, so no subject's colour bleeds round its edge.

/// The night's frame k (0 .. F_HAND - 1), RGB, the full version or the party version.
pub fn night_frame_v<O: Ops>(o: &mut O, nw: &NightWorld, k: i64, party: bool) -> Vec<u8> {
    // (the design's frame: the orbit skipped)
    let k = design_f(o, nw, k);
    let r = |o: &mut O, k: i64, m: Mode| if party { party_render(o, nw, k, m) } else { night_render(o, nw, k, m) };
    if k < XFADE {
        let a = smooth1(o, k, 0, XFADE);
        let old = r(o, k, Mode::Old);
        if a <= 0 { return old; }
        let new = r(o, k, Mode::New);
        return mix_rgb(o, &old, &new, a);
    }
    if (F_GO..F_VIEW).contains(&k) {
        let a = smooth1(o, k, F_GO, F_VIEW);
        let cab = r(o, k, Mode::Cab);
        if a <= 0 { return cab; }
        let new = r(o, k, Mode::New);
        return mix_rgb(o, &cab, &new, a);
    }
    r(o, k, Mode::New)
}

/// One picture of the party version (the mode's), RGB at 1536x768.
pub fn party_render<O: Ops>(o: &mut O, nw: &NightWorld, f: i64, mode: Mode) -> Vec<u8> {
    set_res(2);
    let lo = night_render_parts(o, nw, f, mode);
    set_res(1);
    compose(o, nw, f, mode, &lo)
}

/// The full-resolution pass of the party version over its half-resolution render.
pub fn compose<O: Ops>(o: &mut O, nw: &NightWorld, f: i64, mode: Mode, lo: &Parts) -> Vec<u8> {
    let w = &nw.w;
    let (fw, fh) = (W() as usize, H() as usize);
    let (lw, lh) = (fw / 2, fh / 2);
    let n_pix = fw * fh;
    let parent = |p: usize| -> usize { ((p / fw) / 2) * lw + (p % fw) / 2 };
    let lsubj = |q: usize| lo.g.mat[q] >= 0 && is_subject(w, lo.g.src[q], lo.g.mat[q] as usize);
    // a subject's half-resolution pixel for each full one: its own, or of the eight round it (in a fixed order) the
    // first that is a subject's, which a machine picks (the largest of their flags, the first of equals); none
    // where no flag is set
    let subj_near = |o: &mut O, ps: &[usize]| -> Vec<Option<usize>> {
        let mut out = vec![None; ps.len()];
        let (mut flags, mut cands, mut at): (Vec<V>, Vec<Vec<usize>>, Vec<usize>) = (vec![], vec![], vec![]);
        for (j, &p) in ps.iter().enumerate() {
            let q = parent(p);
            if lsubj(q) { out[j] = Some(q); continue; }
            let (x, y) = ((q % lw) as i64, (q / lw) as i64);
            let (mut fl, mut c) = (vec![], vec![]);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                let (a, b) = (x + dx, y + dy);
                let r = if a >= 0 && b >= 0 && (a as usize) < lw && (b as usize) < lh { Some(b as usize * lw + a as usize) } else { None };
                fl.push(r.is_some_and(lsubj) as i64);
                c.push(r.unwrap_or(q));
            }
            flags.push(fl); cands.push(c); at.push(j);
        }
        if !at.is_empty() {
            let groups: Vec<&[i64]> = flags.iter().map(|g| &g[..]).collect();
            let best = o.nearest(&groups);
            for (k, &j) in at.iter().enumerate() { if flags[k][best[k]] == 1 { out[j] = Some(cands[k][best[k]]); } }
        }
        out
    };
    o.stage("p_subjects");
    let cam = lo.cam;
    let scr = screen_for(nw, cam.f2);
    let sc = &w.sc;
    let subj_tri = |k: usize| subject_kind(sc.mats[sc.world.mat[k]].kind);
    let (near, far) = lo.vis.clone();
    let near: Vec<usize> = near.into_iter().filter(|&k| subj_tri(k)).collect();
    let far: Vec<usize> = far.into_iter().filter(|&k| subj_tri(k)).collect();
    let two: Vec<bool> = sc.world.mat.iter().map(|&m| sc.mats[m].kind == kind::TREE || nw.nocull[m]).collect();
    let mut drawn: Vec<(u8, Drawn)> = vec![];
    if !near.is_empty() { drawn.push((1, draw(o, &w.tb, &cam, &sc.world, &sc.world.corners, &two, &near, false, 1))); }
    if !far.is_empty() { drawn.push((1, draw(o, &w.tb, &cam, &sc.world, &sc.world.corners, &two, &far, true, 1))); }
    for (src, m) in &lo.movers {
        if *src != 4 && *src != 5 { continue; }
        let pick: Vec<usize> = (0..m.tris.len()).filter(|&t| *src == 4 || subject_kind(sc.mats[m.mat[t]].kind)).collect();
        if pick.is_empty() { continue; }
        let two: Vec<bool> = m.mat.iter().map(|&q| nw.nocull[q]).collect();
        let far = mover_far(o, m.corners[0], cam.eye_cm);
        drawn.push((*src, draw(o, &w.tb, &cam, m, &m.corners, &two, &pick, far, 1)));
    }
    let refs: Vec<(u8, &Drawn, &Mesh)> = drawn.iter().map(|(src, d)| (*src, d, if *src == 1 { &sc.world } else { &lo.movers.iter().find(|(s, _)| s == src).unwrap().1 })).collect();
    let mut gs = GBuf { iz: vec![0; n_pix], mat: vec![-1; n_pix], n: [vec![0; n_pix], vec![0; n_pix], vec![0; n_pix]], src: vec![0; n_pix], tri: vec![0; n_pix] };
    let mut seen = vec![false; n_pix];
    if !refs.is_empty() {
        resolve(o, &mut gs, &refs, None);
        // kept where nearer than the world's own depth at its half-resolution pixel
        let cand: Vec<usize> = (0..n_pix).filter(|&p| gs.mat[p] >= 0).collect();
        let sz: V = cand.iter().map(|&p| gs.iz[p]).collect();
        let wz: V = cand.iter().map(|&p| lo.wiz[parent(p)]).collect();
        let hidden = o.lt(v(&sz), v(&wz));
        for (j, &p) in cand.iter().enumerate() { if hidden[j] == 0 { seen[p] = true; } }
    }
    let vis: Vec<usize> = (0..n_pix).filter(|&p| seen[p]).collect();
    let mut out = vec![0u8; n_pix * 3];
    let mut over = vec![false; n_pix];
    let tb = &w.tb;
    if !vis.is_empty() {
        // how much of the half-resolution (blurred) picture: by the blur's length over its block (in its pixels),
        // (len - 1) / 4 held to 0..1 (thousandths); all of it where balloons or sparks were laid over
        let bw = lw / 2;
        let blk = |p: usize| -> usize { let q = parent(p); ((q / lw) / 2) * bw + (q % lw) / 2 };
        let ln: V = vis.iter().map(|&p| lo.mlen[blk(p)]).collect();
        let wv = o.lin(&[(v(&ln), Some(K(250))), (K(-250), None)]);
        let wv = o.clamp(v(&wv), 0, 1000);
        let wv: V = vis.iter().enumerate().map(|(j, &p)| if !lo.laid.is_empty() && lo.laid[parent(p)] { 1000 } else { wv[j] }).collect();
        // (asked for the pixels that take the post's light or the half-resolution picture)
        let ask: Vec<usize> = (0..vis.len()).filter(|&j| wv[j] > 0 || !lo.ov[0].is_empty()).collect();
        let mut near = vec![None; vis.len()];
        for (k, q) in subj_near(o, &ask.iter().map(|&j| vis[j]).collect::<Vec<_>>()).into_iter().enumerate() { near[ask[k]] = q; }
        let shade: Vec<usize> = (0..vis.len()).filter(|&j| wv[j] < 1000).collect();
        let sp: Vec<usize> = shade.iter().map(|&j| vis[j]).collect();
        let mut ds: Vec<[i64; 3]> = vec![[0; 3]; vis.len()];
        if !sp.is_empty() {
            let ray = rays_for(o, &cam, &sp);
            let e = lo.l.e;
            let (c, h) = {
                let mut ns = NightShade { nw, l: &lo.l, cam: &cam, scr, mode, movers: &lo.movers, fx: &lo.fx };
                let kf = kl(o, &[(f, None), (172, None)]);
                shade_solid(o, w, &cam, &lo.l.env, e, kf, None, &gs, &ray, &sp, &lo.map, &lo.round, None, None, if mode == Mode::Old { Some(&nw.dusk_old) } else { None }, &mut ns)
            };
            o.stage("p_subjects");
            let mut col: [V; 3] = [vec![0; n_pix], vec![0; n_pix], vec![0; n_pix]];
            let mut hot: [V; 3] = [vec![0; n_pix], vec![0; n_pix], vec![0; n_pix]];
            for (j, &p) in sp.iter().enumerate() { for ch in 0..3 { col[ch][p] = c[ch][j]; hot[ch][p] = h[ch][j]; } }
            // the light the post laid on the subject's half-resolution pixel (the beams in front of it)
            if !lo.ov[0].is_empty() {
                let mut ov = Over::new();
                let mut px = vec![];
                let mut vals: [V; 3] = [vec![], vec![], vec![]];
                for &j in &shade {
                    let p = vis[j];
                    if let Some(q) = near[j] {
                        if (0..3).any(|ch| lo.ov[ch][q] != 0) { px.push(p); for ch in 0..3 { vals[ch].push(lo.ov[ch][q]); } }
                    }
                }
                ov.put(o, &px, &vals);
                ov.apply(o, &mut col, &mut hot);
            }
            // its bloom (the lens's light at its half-resolution pixel), the tone by its own vignette
            let g = |a: &V| -> V { sp.iter().map(|&p| a[p]).collect() };
            let cs: [V; 3] = std::array::from_fn(|ch| g(&col[ch]));
            let hs: [V; 3] = std::array::from_fn(|ch| g(&hot[ch]));
            let ad: [V; 3] = std::array::from_fn(|ch| sp.iter().map(|&p| lo.add[ch][parent(p)]).collect());
            let vl: V = sp.iter().map(|&p| scr.vlev[p]).collect();
            let t = finish_v(o, w, &vl, &cs, &hs, &ad);
            for (k, &j) in shade.iter().enumerate() { ds[j] = [t[0][k], t[1][k], t[2][k]]; }
        }
        // where it streaks: the half-resolution picture of the subject mixed in (percent)
        let low = |j: usize| -> [i64; 3] { let q = near[j].unwrap_or(parent(vis[j])); std::array::from_fn(|ch| lo.img[3 * q + ch] as i64) };
        let mixj: Vec<usize> = (0..vis.len()).filter(|&j| wv[j] > 0 && wv[j] < 1000).collect();
        if !mixj.is_empty() {
            let t: V = mixj.iter().map(|&j| wv[j]).collect();
            let t = o.div(v(&t), K(10), 0);
            for ch in 0..3 {
                let a: V = mixj.iter().map(|&j| ds[j][ch]).collect();
                let b: V = mixj.iter().map(|&j| low(j)[ch]).collect();
                let m = o.mix(v(&a), v(&b), v(&t));
                for (k, &j) in mixj.iter().enumerate() { ds[j][ch] = m[k]; }
            }
        }
        for (j, &p) in vis.iter().enumerate() {
            let d = if wv[j] >= 1000 { low(j) } else { ds[j] };
            for ch in 0..3 { out[3 * p + ch] = d[ch] as u8; }
            over[p] = true;
        }
    }
    // the stars, a pixel each where the sky shows at full resolution
    o.stage("p_stars");
    let n = lo.l.night;
    if n > 300 {
        let mut ov = Over::new();
        let sky = |p: usize| lo.g.mat[parent(p)] < 0 && !seen[p];
        stars_over(o, nw, n, lo.l.e, &cam, &sky, &mut ov);
        let px: Vec<usize> = (0..n_pix).filter(|&p| ov.touched[p]).collect();
        if !px.is_empty() {
            let mut col: [V; 3] = [vec![0; n_pix], vec![0; n_pix], vec![0; n_pix]];
            let mut hot: [V; 3] = [vec![0; n_pix], vec![0; n_pix], vec![0; n_pix]];
            for &p in &px { for ch in 0..3 { col[ch][p] = lo.col[ch][parent(p)]; hot[ch][p] = lo.hot[ch][parent(p)]; } }
            ov.apply(o, &mut col, &mut hot);
            let g = |a: &V| -> V { px.iter().map(|&p| a[p]).collect() };
            let cs: [V; 3] = std::array::from_fn(|ch| g(&col[ch]));
            let hs: [V; 3] = std::array::from_fn(|ch| g(&hot[ch]));
            let ad: [V; 3] = std::array::from_fn(|ch| px.iter().map(|&p| lo.add[ch][parent(p)]).collect());
            let vl: V = px.iter().map(|&p| scr.vlev[p]).collect();
            let t = finish_v(o, w, &vl, &cs, &hs, &ad);
            for (k, &p) in px.iter().enumerate() { for ch in 0..3 { out[3 * p + ch] = t[ch][k] as u8; } over[p] = true; }
        }
    }
    // the world made up to full size by its own pixels: those a subject covered take the nearest world pixel's colour
    o.stage("up");
    let subj_l: Vec<bool> = (0..lw * lh).map(lsubj).collect();
    let up = up2_world(o, &lo.img, &subj_l, lw, lh);
    for p in 0..n_pix { if !over[p] { for ch in 0..3 { out[3 * p + ch] = up[3 * p + ch]; } } }
    out
}
