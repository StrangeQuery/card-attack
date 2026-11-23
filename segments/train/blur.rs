// Motion blur: each pixel the mean of the pixels along its motion over the shutter (0.4 of a frame), its streak's
// length from the cameras a fifth of a frame either side. The streak is
// a box, from running sums along lines across the screen in 16 directions, so a streak costs the same however long.
// Included after tunnel.rs.

/// The lines of the blur's 16 directions at half the resolution: for each, each line's pixels in order.
pub struct BlurLines { pub dirs: Vec<Vec<Vec<usize>>> }

pub fn blur_lines(tb: &Tb) -> BlurLines {
    let line = tb.t(blur_name("blur_line"));
    let place = tb.t(blur_name("blur_place"));
    let lens = tb.t(blur_name("blur_len"));
    let mut dirs = vec![];
    for d in 0..16 {
        let nl = lens.w;
        let mut ls: Vec<Vec<usize>> = (0..nl).map(|l| vec![usize::MAX; lens.data[d * lens.w + l] as usize]).collect();
        for p in 0..N_PIX() {
            let (l, k) = (line.data[d * N_PIX() + p] as usize, place.data[d * N_PIX() + p] as usize);
            ls[l][k] = p;
        }
        dirs.push(ls);
    }
    BlurLines { dirs }
}

/// Each 2x2 block's motion over the shutter: the line it blurs along (of 16), its length (pixels, at most 138) and the
/// places either way along the line. The motion is the camera's translation (the cab's places at -0.2 and +0.2 of a
/// frame, in the camera's turn) over the block's depth, by logarithms (the flow's part by column or row times K / z,
/// over 299,700), and the turn's (worked out every 32 pixels for the far away, the block taking its nearest); the
/// other train's pixels move with it, toward us. Held to 400 each way, then the tables say the rest.
pub struct Motion { pub dir: V, pub len: V, pub h: V }

pub fn velocities<O: Ops>(o: &mut O, w: &World, cam: &Cam, k: i64, g: &GBuf) -> Motion {
    let ta = kl(o, &[(k, Some(10)), (-2, None)]);
    let tb = kl(o, &[(k, Some(10)), (2, None)]);
    let ca = camera(o, &w.sc.t, ta);
    let cb = camera(o, &w.sc.t, tb);
    velocities_c(o, w, cam, &ca, &cb, g, &[(2, [0, 0, 2800])])
}

/// The same from the cameras a fifth of a frame either side, given; `movers`: the pixels from a list (by its source
/// number) moving through the world over the shutter (mm, as the eye moves: the other train's 2800 toward us).
pub fn velocities_c<O: Ops>(o: &mut O, w: &World, cam: &Cam, ca: &Cam, cb: &Cam, g: &GBuf, movers: &[(u8, [i64; 3])]) -> Motion {
    let tb = &w.tb;
    let de = { let d: [V; 3] = std::array::from_fn(|j| o.lin(&[(K(cb.eye[j]), None), (K(ca.eye[j]), Some(K(-1)))])); [one(d[0].clone()), one(d[1].clone()), one(d[2].clone())] };
    let tr = |o: &mut O, extra: [i64; 3]| -> [i64; 3] {
        std::array::from_fn(|r| {
            let t = o.lin(&[(K(cam.b[r][0]), Some(K(de[0]))), (K(cam.b[r][0]), Some(K(extra[0]))), (K(cam.b[r][1]), Some(K(de[1]))), (K(cam.b[r][1]), Some(K(extra[1]))), (K(cam.b[r][2]), Some(K(de[2]))), (K(cam.b[r][2]), Some(K(extra[2])))]);
            one(o.div(v(&t), K(1000), 0))
        })
    };
    let ts = tr(o, [0, 0, 0]);
    let (bw, bh) = ((W() / 2) as usize, (H() / 2) as usize);
    let blocks: Vec<usize> = (0..bh).flat_map(|by| (0..bw).map(move |bx| 2 * by * W() as usize + 2 * bx)).collect();
    let iz: V = blocks.iter().map(|&p| g.iz[p]).collect();
    let izc = o.min(v(&iz), K(99_999));
    let lz = o.tab(tb.t("lnv"), v(&izc));
    // ln K'' less ln K / z, each block's, once
    let lnk = vec![tb.t("lnk").data[0]; lz.len()];
    let nz = o.ldiff(&lnk, &lz, -999, 999);

    // by column and by row: (x Tz - f Tx) / 2000 and (y Tz + f Ty) / 2000, x and y in half pixels
    let xs2: V = (0..bw as i64).map(|bx| 4 * bx + 2 - W()).collect();
    let ys2: V = (0..bh as i64).map(|by| 4 * by + 2 - H()).collect();
    let part = |o: &mut O, c: &V, tz: i64, t: i64| -> V { let ft = kl(o, &[(cam.f2, Some(t))]); let a = o.lin(&[(v(c), Some(K(tz))), (K(ft), None)]); o.div(v(&a), K(2000), 0) };
    let speed = |o: &mut O, a: &V, by_col: bool| -> V {
        // |a| K / z / 299,700 by logarithms (the column's or row's ln |a| less the block's ln K'' / (K / z)), its
        // sign by the table (e or -e)
        let m = o.lin(&[(v(a), Some(K(-1)))]);
        let size = o.max(v(a), v(&m));
        let la = o.tab(tb.t("lnv"), v(&size));
        let neg = o.lt(v(a), K(0));
        let of = |i: usize| if by_col { i % bw } else { i / bw };
        let lab: V = (0..blocks.len()).map(|i| la[of(i)]).collect();
        let i = o.ldiff(&lab, &nz, 0, 8);
        let (pos, ng): (Vec<usize>, Vec<usize>) = (0..blocks.len()).partition(|&j| neg[of(j)] == 0);
        let mut u = vec![0; blocks.len()];
        for (t, set) in [("expv", &pos), ("expv_neg", &ng)] {
            let ii: V = set.iter().map(|&j| i[j]).collect();
            let r = o.tab(tb.t(t), v(&ii));
            for (k, &j) in set.iter().enumerate() { u[j] = r[k]; }
        }
        u
    };
    let flow = |o: &mut O, t: [i64; 3]| -> (V, V) {
        let ax = { let nt = kl(o, &[(t[0], Some(-1))]); part(o, &xs2, t[2], nt) };
        let ay = part(o, &ys2, t[2], t[1]);
        (speed(o, &ax, true), speed(o, &ay, false))
    };
    let (mut ux, mut uy) = flow(o, ts);
    for &(src, extra) in movers {
        let moving: V = blocks.iter().map(|&p| (g.src[p] == src) as i64).collect();
        if moving.iter().all(|&m| m == 0) && movers.len() > 1 { continue; }
        let tt = tr(o, extra);
        let (txv, tyv) = flow(o, tt);
        ux = o.sel(&moving, v(&txv), v(&ux));
        uy = o.sel(&moving, v(&tyv), v(&uy));
    }
    // the turn's: directions every 32 pixels through each camera, the difference of their places
    let mut gpx = vec![];
    let mut gpy = vec![];
    for gy in 0..24i64 { for gxx in 0..48i64 { gpx.push(64 * gxx + 32 - W()); gpy.push(H() - 64 * gy - 32); } }
    let dir: [V; 3] = std::array::from_fn(|j| {
        let t = { let k_ = kl(o, &[(cam.f2, Some(cam.b[2][j]))]); o.lin(&[(v(&gpx), Some(K(cam.b[0][j]))), (v(&gpy), Some(K(cam.b[1][j]))), (K(k_), None)]) };
        o.div(v(&t), K(1000), 0)
    });
    let place = |o: &mut O, c: &Cam| -> (V, V) {
        let r: [V; 3] = std::array::from_fn(|i| {
            let t = o.lin(&[(v(&dir[0]), Some(K(c.b[i][0]))), (v(&dir[1]), Some(K(c.b[i][1]))), (v(&dir[2]), Some(K(c.b[i][2])))]);
            o.div(v(&t), K(1000), 0)
        });
        let z = o.max(v(&r[2]), K(1));
        let (f50, nf50) = (kl(o, &[(c.f2, Some(50))]), kl(o, &[(c.f2, Some(-50))]));
        let x = o.lin(&[(v(&r[0]), Some(K(f50)))]);
        let x = o.div(v(&x), v(&z), 0);
        let y = o.lin(&[(v(&r[1]), Some(K(nf50)))]);
        let y = o.div(v(&y), v(&z), 0);
        (x, y)
    };
    let (xa, ya) = place(o, ca);
    let (xb, yb) = place(o, cb);
    let rx = { let t = o.lin(&[(v(&xb), None), (v(&xa), Some(K(-1)))]); o.div(v(&t), K(100), 0) };
    let ry = { let t = o.lin(&[(v(&yb), None), (v(&ya), Some(K(-1)))]); o.div(v(&t), K(100), 0) };
    let rbx: V = blocks.iter().map(|&p| { let (x, y) = ((p as i64 % W()) / 32, (p as i64 / W()) / 32); rx[(y * 48 + x) as usize] }).collect();
    let rby: V = blocks.iter().map(|&p| { let (x, y) = ((p as i64 % W()) / 32, (p as i64 / W()) / 32); ry[(y * 48 + x) as usize] }).collect();
    // the two flows' sum, held to 400 each way, as the tables' places
    let (ix, iy) = o.vel(&ux, &rbx, &uy, &rby);
    let len = o.tab2(tb.t("bl_len"), v(&iy), v(&ix));
    let d = o.tab2(tb.t("bl_dir"), v(&iy), v(&ix));
    let h = o.tab2(tb.t("bl_h"), v(&d), v(&len));
    Motion { dir: d, len, h }
}

/// The blur: every pixel moving more than a pixel takes the mean of the pixels along its line (the direction nearest
/// its motion) as far as half its motion either way. Colour and the part past 999, each channel; the means by
/// logarithms (ln of the sum less ln of the count, e back).
pub fn motion_blur<O: Ops>(o: &mut O, w: &World, bl: &BlurLines, mo: &Motion, rate: &[u8], col: &mut [V; 3], hot: &mut [V; 3]) {
    let tb = &w.tb;
    let bw = (W() / 2) as usize;
    let pix_h: V = (0..N_PIX()).map(|p| { let (x, y) = (p % W() as usize, p / W() as usize); mo.h[(y / 2) * bw + x / 2] }).collect();
    let pix_d: Vec<usize> = (0..N_PIX()).map(|p| { let (x, y) = (p % W() as usize, p / W() as usize); mo.dir[(y / 2) * bw + x / 2] as usize }).collect();
    let line = tb.t(blur_name("blur_line"));
    let place = tb.t(blur_name("blur_place"));
    let lens = tb.t(blur_name("blur_len"));
    let any_hot = hot.iter().any(|h| h.iter().any(|&x| x != 0));
    let chans = if any_hot { 6 } else { 3 };
    let src: Vec<V> = (0..3).map(|c| col[c].clone()).chain((0..3).map(|c| hot[c].clone())).collect();
    for dd in 0..16usize {
        // in the tiles shaded 4x4 (streaks of 24 or more), each 2x2 block's first pixel only
        let users: Vec<usize> = (0..N_PIX()).filter(|&p| pix_d[p] == dd && pix_h[p] > 0 && (rate[p] < 4 || (p % 2 == 0 && (p / W() as usize) % 2 == 0))).collect();
        if users.is_empty() { continue; }
        let lines = &bl.dirs[dd];
        // the users' ends (machine-worked: k - h - 1 and k + h, held to the line's last place: its length in the
        // printed table less 1, for each line with users, on the machines), the count between
        let lu: V = users.iter().map(|&p| line.data[dd * N_PIX() + p]).collect();
        let ku: V = users.iter().map(|&p| place.data[dd * N_PIX() + p]).collect();
        let hu: V = users.iter().map(|&p| pix_h[p]).collect();
        let mut used = vec![false; lens.w];
        for &l in &lu { used[l as usize] = true; }
        let ls: Vec<usize> = (0..lens.w).filter(|&l| used[l]).collect();
        let lastv = o.sum(&[(v(&ls.iter().map(|&l| lens.data[dd * lens.w + l]).collect()), 1)], -1);
        let mut last_of = vec![0; lens.w];
        for (j, &l) in ls.iter().enumerate() { last_of[l] = lastv[j]; }
        let hi_lim: V = lu.iter().map(|&l| last_of[l as usize]).collect();
        let (a, b, n) = o.ends(&ku, &hu, &hi_lim);
        // each line's span: from its users' first start (at least 0) to their last end, the first cards of each
        // line's pocket when the card sorter puts the users in order by line and start, and by line and end, last first
        let mut span: std::collections::BTreeMap<usize, (i64, i64)> = Default::default();
        for i in sorter::order(&(0..users.len()).map(|i| [lu[i], a[i]]).collect::<Vec<_>>(), &[sorter::up(4), sorter::up(4)]) { span.entry(lu[i] as usize).or_insert((a[i].max(0), 0)); }
        let mut ended = vec![false; lens.w];
        for i in sorter::order(&(0..users.len()).map(|i| [lu[i], b[i]]).collect::<Vec<_>>(), &[sorter::up(4), sorter::down(4)]) {
            let l = lu[i] as usize;
            if !ended[l] { ended[l] = true; span.get_mut(&l).unwrap().1 = b[i]; }
        }
        let spans: Vec<(usize, i64, i64)> = span.iter().map(|(&l, &(a, b))| (l, a, b)).collect();
        let at: std::collections::HashMap<usize, usize> = spans.iter().scan(0usize, |acc, &(l, a, b)| { let s = *acc; *acc += (b - a + 1) as usize; Some((l, s)) }).collect();
        let lnn = o.tab(tb.t("ln_n"), v(&n));
        // the running sums by pieces of 500 places from each span's start (so a sum stays a long); a user's window
        // a + 1 ..= b reaches at most into the next piece
        const PIECE: i64 = 500;
        let pieces: Vec<(usize, i64, i64)> = spans.iter().flat_map(|&(l, s0, s1)| (0..=(s1 - s0) / PIECE).map(move |j| (l, s0 + PIECE * j, (s0 + PIECE * (j + 1) - 1).min(s1)))).collect();
        let start_of = |l: usize, x: i64| -> i64 { let s0 = spans_lo(&span, l); s0 + PIECE * ((x - s0) / PIECE) };
        for c in 0..chans {
            // the parts past 999: lines with none give none (the host sees the zeros)
            let live: std::collections::HashSet<usize> = spans.iter().filter(|&&(l, s0, s1)| c < 3 || (s0..=s1).any(|kk| src[c][lines[l][kk as usize]] != 0)).map(|s| s.0).collect();
            let dst = if c < 3 { &mut col[c] } else { &mut hot[c - 3] };
            let (users, a, b, lnn): (Vec<usize>, V, V, V) = if c < 3 { (users.clone(), a.clone(), b.clone(), lnn.clone()) } else {
                let mut keep = (vec![], vec![], vec![], vec![]);
                for (i, &p) in users.iter().enumerate() {
                    if live.contains(&(line.data[dd * N_PIX() + p] as usize)) { keep.0.push(p); keep.1.push(a[i]); keep.2.push(b[i]); keep.3.push(lnn[i]); } else { dst[p] = 0; }
                }
                keep
            };
            if users.is_empty() { continue; }
            let lp: Vec<&(usize, i64, i64)> = pieces.iter().filter(|pc| live.contains(&pc.0)).collect();
            let pat: std::collections::HashMap<(usize, i64), usize> = lp.iter().scan(0usize, |acc, &&(l, p0, p1)| { let s = *acc; *acc += (p1 - p0 + 1) as usize; Some(((l, p0), s)) }).collect();
            let lists: Vec<Vec<i64>> = lp.iter().map(|&&(l, p0, p1)| (p0..=p1).map(|kk| src[c][lines[l][kk as usize]]).collect()).collect();
            let refs: Vec<&[i64]> = lists.iter().map(|x| x.as_slice()).collect();
            let sums = o.runsum(&refs);
            // the host fetches the running sums at the window's ends: within one piece S[b] - S[a] (S before the piece's
            // start 0); across two, (S[the first's end] - S[a]) + S[b]
            let get = |l: usize, x: i64| -> i64 { let p0 = start_of(l, x); sums[pat[&(l, p0)] + (x - p0) as usize] };
            let mut x1 = Vec::with_capacity(users.len());
            let mut x2 = Vec::with_capacity(users.len());
            let mut cross = vec![];
            for (i, &p) in users.iter().enumerate() {
                let l = line.data[dd * N_PIX() + p] as usize;
                let (pa1, pb) = (start_of(l, a[i] + 1), start_of(l, b[i]));
                let lo_a = if a[i] >= pa1 { get(l, a[i]) } else { 0 };
                if pa1 == pb { x1.push(get(l, b[i])); } else { x1.push(get(l, pb - 1)); cross.push(i); }
                x2.push(lo_a);
            }
            let mut s = o.ldiff(&x1, &x2, 0, 999);
            if !cross.is_empty() {
                let d: V = cross.iter().map(|&i| s[i]).collect();
                let sb: V = cross.iter().map(|&i| { let l = line.data[dd * N_PIX() + users[i]] as usize; get(l, b[i]) }).collect();
                let t = o.sum(&[(v(&d), 1), (v(&sb), 1)], 0);
                for (j, &i) in cross.iter().enumerate() { s[i] = t[j]; }
            }
            let lns = o.tab(tb.t("ln_s"), v(&s));
            let di = o.ldiff(&lns, &lnn, 0, 79);
            let mean = o.tab(tb.t("exp_s"), v(&di));
            for (i, &p) in users.iter().enumerate() { dst[p] = mean[i]; }
        }
    }
    // the rest of those blocks take their first pixel's
    for p in 0..N_PIX() {
        if rate[p] >= 4 && pix_h[p] > 0 && !(p % 2 == 0 && (p / W() as usize) % 2 == 0) {
            let q = ((p / W() as usize) & !1) * W() as usize + ((p % W() as usize) & !1);
            for c in 0..3 { col[c][p] = col[c][q]; hot[c][p] = hot[c][q]; }
        }
    }
}

/// The blur's line tables for the resolution drawn (the night's, "_h", at half).
pub fn blur_name(n: &'static str) -> &'static str { if res() == 1 { n } else { match n { "blur_line" => "blur_line_h", "blur_place" => "blur_place_h", _ => "blur_len_h" } } }

fn spans_lo(span: &std::collections::BTreeMap<usize, (i64, i64)>, l: usize) -> i64 { span[&l].0.max(0) }
