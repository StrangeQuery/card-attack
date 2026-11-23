// The party versions' common parts (the train's, and the night's in segments/night/nparty.rs): the world drawn at
// half the resolution (768x384) and made up to 1536x768, what has to read across a room drawn again at full
// resolution, kept where nearer than the world's own depth at its half-resolution pixel, shaded, given its
// half-resolution pixel's bloom, toned; the world made up to full size from its own pixels only (those a subject
// covered first take their nearest world pixel's colour), so no subject's colour bleeds round its edge. And the
// train's: its subject is the departure's board (TONIGHT's LED words: 14:03 NOZOMI 37 FOR SHIN-OSAKA); the words over
// the city (the shader, the brag) are laid on at full resolution after the pass. Included after overlays.rs.

/// The camera for the resolution drawn: at half, its lens's twice f in those pixels (half the full screen's; the
/// projection's tables by the resolution, its turn the same).
pub fn res_cam<O: Ops>(o: &mut O, cam: &Cam) -> Cam {
    if res() == 1 { return *cam; }
    let mut c = *cam;
    c.f2 = one(o.div(K(cam.f2), K(res()), 0));
    c
}

/// The rays (as rays()) at the quarter rows holding the pixels asked for; 0 elsewhere.
pub fn rays_for<O: Ops>(o: &mut O, cam: &Cam, need: &[usize]) -> [V; 3] {
    let q = (W() / 4) as usize;
    let wd = W() as usize;
    let mut want = vec![false; H() as usize * 4];
    for &p in need { want[(p / wd) * 4 + (p % wd) / q] = true; }
    let sel: Vec<usize> = (0..want.len()).filter(|&i| want[i]).collect();
    let mut out: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    if sel.is_empty() { return out; }
    // ten times twice f, and its negative, on the machines
    let (f10, nf10) = (kl(o, &[(cam.f2, Some(10))]), kl(o, &[(cam.f2, Some(-10))]));
    for i in 0..3 {
        let xq: V = sel.iter().map(|&s| 2 * (s % 4) as i64 * q as i64 + 1 - W()).collect();
        let yq: V = sel.iter().map(|&s| H() - 1 - 2 * (s / 4) as i64).collect();
        let s = { let k_ = kl(o, &[(cam.f2, Some(cam.b[2][i]))]); o.lin(&[(v(&xq), Some(K(cam.b[0][i]))), (v(&yq), Some(K(cam.b[1][i]))), (K(k_), None)]) };
        let s = o.lin(&[(v(&s), Some(K(7)))]);
        let va = o.div(v(&s), K(f10), 0);
        let rest = o.lin(&[(v(&s), None), (v(&va), Some(K(nf10)))]);
        let f0 = { let t = o.lin(&[(v(&rest), Some(K(1000)))]); o.div(v(&t), K(f10), 0) };
        let step = one({ let t = o.lin(&[(K(cam.b[0][i]), Some(K(14_000)))]); o.div(v(&t), K(f10), 0) });
        let walks: Vec<(i64, i64, i64, i64)> = (0..va.len()).map(|j| (q as i64, va[j], step, f0[j])).collect();
        let r = o.walkc(&walks);
        for (j, &s) in sel.iter().enumerate() { let base = (s / 4) * wd + (s % 4) * q; out[i][base..base + q].copy_from_slice(&r[j * q..(j + 1) * q]); }
    }
    out
}

/// A picture (RGB at half resolution) made twice the size each way, each pixel between its four nearest by their
/// places (a quarter and three quarters of the way: up4's second and fourth steps), the edges held.
pub fn up2<O: Ops>(o: &mut O, img: &[u8], lw: usize, lh: usize) -> Vec<u8> {
    let (fw, fh) = (2 * lw, 2 * lh);
    let fine = |o: &mut O, lines: &[V]| -> Vec<V> {
        let refs: Vec<&[i64]> = lines.iter().map(|l| l.as_slice()).collect();
        let u = o.up4(&refs);
        let mut at = 0;
        lines.iter().map(|l| {
            let n = l.len();
            let mut row = Vec::with_capacity(2 * n);
            row.push(l[0]);
            for k in 0..n - 1 { row.push(u[at + 4 * k + 1]); row.push(u[at + 4 * k + 3]); }
            row.push(l[n - 1]);
            at += 4 * (n - 1);
            row
        }).collect()
    };
    let rows: Vec<V> = (0..3 * lh).map(|i| { let (c, y) = (i / lh, i % lh); (0..lw).map(|x| img[3 * (y * lw + x) + c] as i64).collect() }).collect();
    let rows = fine(o, &rows);
    let cols: Vec<V> = (0..3 * fw).map(|i| { let (c, x) = (i / fw, i % fw); (0..lh).map(|y| rows[c * lh + y][x]).collect() }).collect();
    let cols = fine(o, &cols);
    let mut out = vec![0u8; fw * fh * 3];
    for (i, col) in cols.iter().enumerate() { let (c, x) = (i / fw, i % fw); for (y, &val) in col.iter().enumerate() { out[3 * (y * fw + x) + c] = val as u8; } }
    out
}


/// The world made up to full size from a half-resolution picture (RGB) by its own pixels: those flagged as a
/// subject's first take their nearest unflagged pixel's colour (the host's copying), then up2.
pub fn up2_world<O: Ops>(o: &mut O, img: &[u8], subj: &[bool], lw: usize, lh: usize) -> Vec<u8> {
    let mut bg = img.to_vec();
    for q in 0..lw * lh {
        if !subj[q] { continue; }
        let (x, y) = ((q % lw) as i64, (q / lw) as i64);
        'find: for r in 1..12i64 {
            for dy in -r..=r { for dx in -r..=r {
                if dx.abs().max(dy.abs()) != r { continue; }
                let (a, b) = (x + dx, y + dy);
                if a < 0 || b < 0 || a >= lw as i64 || b >= lh as i64 { continue; }
                let s = b as usize * lw + a as usize;
                if !subj[s] { for ch in 0..3 { bg[3 * q + ch] = img[3 * s + ch]; } break 'find; }
            } }
        }
    }
    up2(o, &bg, lw, lh)
}

/// The train's full-resolution pass over its half-resolution render: the board drawn again and kept where nearer
/// than the world's own depth, shaded at full resolution where it's still (its half-resolution, blurred picture mixed
/// in as its blur grows over a pixel, all of it by 5), its bloom, the tone; the world made up; the words laid on.
pub fn train_compose<O: Ops>(o: &mut O, w: &World, _k: i64, lo: &TParts) -> Vec<u8> {
    let (fw, fh) = (W() as usize, H() as usize);
    let (lw, lh) = (fw / 2, fh / 2);
    let n_pix = fw * fh;
    let sc = &w.sc;
    let parent = |p: usize| -> usize { ((p / fw) / 2) * lw + (p % fw) / 2 };
    let lsubj = |q: usize| lo.g.mat[q] >= 0 && train_subject(w, lo.g.mat[q] as usize);
    // a half-resolution pixel's neighbour (0 itself, then the eight round it in a fixed order), if on the screen
    const NEAR: [(i64, i64); 9] = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];
    let next_to = |q: usize, (dx, dy): (i64, i64)| -> Option<usize> {
        let (a, b) = ((q % lw) as i64 + dx, (q / lw) as i64 + dy);
        (a >= 0 && b >= 0 && (a as usize) < lw && (b as usize) < lh).then(|| b as usize * lw + a as usize)
    };
    o.stage("p_subjects");
    let cam = lo.cam;
    let env = if lo.day { &DAY } else { &DUSK };
    let mut out = vec![0u8; n_pix * 3];
    let mut over = vec![false; n_pix];
    let subj_l: Vec<bool> = (0..lw * lh).map(lsubj).collect();
    if subj_l.iter().any(|&s| s) {
        let (near, far) = lo.vis.clone();
        let near: Vec<usize> = near.into_iter().filter(|&t| train_subject(w, sc.world.mat[t])).collect();
        let far: Vec<usize> = far.into_iter().filter(|&t| train_subject(w, sc.world.mat[t])).collect();
        let two: Vec<bool> = sc.world.mat.iter().map(|&m| sc.mats[m].kind == kind::TREE || sc.mats[m].kind == kind::LED).collect();
        let mut drawn: Vec<(u8, Drawn)> = vec![];
        if !near.is_empty() { drawn.push((1, draw(o, &w.tb, &cam, &sc.world, &sc.world.corners, &two, &near, false, 1))); }
        if !far.is_empty() { drawn.push((1, draw(o, &w.tb, &cam, &sc.world, &sc.world.corners, &two, &far, true, 1))); }
        let refs: Vec<(u8, &Drawn, &Mesh)> = drawn.iter().map(|(s, d)| (*s, d, &sc.world)).collect();
        let mut gs = GBuf { iz: vec![0; n_pix], mat: vec![-1; n_pix], n: [vec![0; n_pix], vec![0; n_pix], vec![0; n_pix]], src: vec![0; n_pix], tri: vec![0; n_pix] };
        let mut seen = vec![false; n_pix];
        if !refs.is_empty() {
            resolve(o, &mut gs, &refs, None);
            let cand: Vec<usize> = (0..n_pix).filter(|&p| gs.mat[p] >= 0).collect();
            let sz: V = cand.iter().map(|&p| gs.iz[p]).collect();
            let wz: V = cand.iter().map(|&p| lo.wiz[parent(p)]).collect();
            let hidden = o.lt(v(&sz), v(&wz));
            for (j, &p) in cand.iter().enumerate() { if hidden[j] == 0 { seen[p] = true; } }
        }
        let vis: Vec<usize> = (0..n_pix).filter(|&p| seen[p]).collect();
        if !vis.is_empty() {
            let bw = lw / 2;
            let blk = |p: usize| -> usize { let q = parent(p); ((q / lw) / 2) * bw + (q % lw) / 2 };
            let ln: V = vis.iter().map(|&p| lo.mlen[blk(p)]).collect();
            let wv = o.lin(&[(v(&ln), Some(K(250))), (K(-250), None)]);
            let wv = o.clamp(v(&wv), 0, 1000);
            let shade: Vec<usize> = (0..vis.len()).filter(|&j| wv[j] < 1000).collect();
            let sp: Vec<usize> = shade.iter().map(|&j| vis[j]).collect();
            let mut ds: Vec<[i64; 3]> = vec![[0; 3]; vis.len()];
            if !sp.is_empty() {
                let ray = rays_for(o, &cam, &sp);
                let (c, h) = shade_solid(o, w, &cam, env, lo.e, lo.kr, lo.front, &gs, &ray, &sp, &lo.map, &lo.round, None, None, lo.fmap.as_ref(), &mut Plain);
                o.stage("p_subjects");
                let ad: [V; 3] = std::array::from_fn(|ch| sp.iter().map(|&p| lo.add[ch][parent(p)]).collect());
                let vl: V = sp.iter().map(|&p| w.scr.vlev[p]).collect();
                let t = finish_v(o, w, &vl, &c, &h, &ad);
                for (kk, &j) in shade.iter().enumerate() { ds[j] = [t[0][kk], t[1][kk], t[2][kk]]; }
            }
            // which half-resolution pixel's colour each pixel takes where it isn't all shaded again: its own where that
            // is a subject's, else the first of its neighbours that is (the first flag set, by tr_near: the first of
            // the largest), else its own
            let mut from: Vec<usize> = vis.iter().map(|&p| parent(p)).collect();
            let ask: Vec<usize> = (0..vis.len()).filter(|&j| wv[j] > 0 && !lsubj(from[j])).collect();
            if !ask.is_empty() {
                let flags: Vec<Vec<i64>> = ask.iter().map(|&j| NEAR.iter().map(|&d| next_to(from[j], d).map_or(0, |r| lsubj(r) as i64)).collect()).collect();
                let refs: Vec<&[i64]> = flags.iter().map(|f| f.as_slice()).collect();
                let win = o.nearest(&refs);
                for (k, &j) in ask.iter().enumerate() { from[j] = next_to(from[j], NEAR[win[k]]).unwrap(); }
            }
            let low = |j: usize| -> [i64; 3] { let q = from[j]; std::array::from_fn(|ch| lo.img[3 * q + ch] as i64) };
            let mixj: Vec<usize> = (0..vis.len()).filter(|&j| wv[j] > 0 && wv[j] < 1000).collect();
            if !mixj.is_empty() {
                let t: V = mixj.iter().map(|&j| wv[j]).collect();
                let t = o.div(v(&t), K(10), 0);
                for ch in 0..3 {
                    let a: V = mixj.iter().map(|&j| ds[j][ch]).collect();
                    let b: V = mixj.iter().map(|&j| low(j)[ch]).collect();
                    let m = o.mix(v(&a), v(&b), v(&t));
                    for (kk, &j) in mixj.iter().enumerate() { ds[j][ch] = m[kk]; }
                }
            }
            for (j, &p) in vis.iter().enumerate() {
                let d = if wv[j] >= 1000 { low(j) } else { ds[j] };
                for ch in 0..3 { out[3 * p + ch] = d[ch] as u8; }
                over[p] = true;
            }
        }
    }
    o.stage("up");
    let up = up2_world(o, &lo.img, &subj_l, lw, lh);
    for p in 0..n_pix { if !over[p] { for ch in 0..3 { out[3 * p + ch] = up[3 * p + ch]; } } }
    // the words over the city, at full resolution
    o.stage("tone");
    let mut img: [V; 3] = std::array::from_fn(|ch| (0..n_pix).map(|p| out[3 * p + ch] as i64).collect());
    overlays(o, &w.tb, lo.kr, &mut img);
    for ch in 0..3 { for p in 0..n_pix { out[3 * p + ch] = img[ch][p] as u8; } }
    out
}
