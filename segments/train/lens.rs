// The lens, after the blur and the wires, as the prototype's render() ends: the sun's ghosts (flare) and the glow of
// what's too bright (bloom), as light to add, which the tone curve takes with the rest (shade.rs's finish; the
// vignette is in its table). Included after wires.rs.

/// The lens's ghosts: how far along the line from the sun through the middle (thousandths), their radii
/// (thousandths of the width), 0.06 of their colours in tenths of R.
const GHOSTS: [(i64, i64, [i64; 3]); 5] = [(450, 25, [135, 75, 30]), (800, 12, [60, 120, 75]), (1250, 50, [53, 45, 120]), (1600, 18, [135, 60, 90]), (2100, 90, [75, 53, 38])];

/// A ghost's disc as a sprite, worked out once: its radius (pixels) and, for each place of the square round its
/// middle (row by row), the index into the disc's profile: the squared distance over the radius squared, thousandths,
/// at most 1499.
pub struct Ghost { pub r: i64, pub t: V }

pub fn ghost_sprites<O: Ops>(o: &mut O) -> Vec<Ghost> {
    GHOSTS.iter().map(|&(_, rad, _)| {
        let r = rad * W() / 1000;
        let dx: V = (-r..=r).flat_map(|_| -r..=r).collect();
        let dy: V = (-r..=r).flat_map(|y| std::iter::repeat_n(y, (2 * r + 1) as usize)).collect();
        let d2 = o.lin(&[(v(&dx), Some(v(&dx))), (v(&dy), Some(v(&dy)))]);
        let t = o.lin(&[(v(&d2), Some(K(1000)))]);
        let t = o.div(v(&t), K(r * r), 0);
        Ghost { r, t: o.min(v(&t), K(1499)) }
    }).collect()
}

/// The low sun's ghosts in the lens: five discs on the line from the sun through the middle, as much as the sun shows
/// (the sky's share of a box round it). Each ghost's profile (the disc times the share and its colour) is worked out
/// for the frame; the host places its sprite and looks the profile up. Light added (R) to `add`; the pixels touched.
pub fn flare<O: Ops>(o: &mut O, w: &World, cam: &Cam, env: &Env, g: &GBuf, add: &mut [V; 3]) -> Vec<usize> {
    let mut touched = vec![];
    if env.day { return touched; }
    let tb = &w.tb;
    let s: [i64; 3] = std::array::from_fn(|r| one({ let t = o.lin(&[(K(cam.b[r][0]), Some(K(env.sun[0]))), (K(cam.b[r][1]), Some(K(env.sun[1]))), (K(cam.b[r][2]), Some(K(env.sun[2])))]); o.div(v(&t), K(1000), 0) }));
    if s[2] <= 0 { return touched; }
    // (the sun's place on the screen, on the machines)
    let d2 = kl(o, &[(s[2], Some(2))]);
    let sx = { let q = one({ let t = o.lin(&[(K(s[0]), Some(K(cam.f2)))]); o.div(v(&t), K(d2), 0) }); one(o.sum(&[(K(q), 1)], W() / 2)) };
    let sy = { let q = one({ let t = o.lin(&[(K(s[1]), Some(K(cam.f2)))]); o.div(v(&t), K(d2), 0) }); one(o.sum(&[(K(q), -1)], H() / 2)) };
    if !(-W() / 10 < sx && sx < W() + W() / 10 && -H() / 10 < sy && sy < H() + H() / 10) { return touched; }
    // how much of the sun shows: the sky's share of the box (18 pixels each way), percent
    let bx: Vec<usize> = (sy - 18..=sy + 18).flat_map(|y| (sx - 18..=sx + 18).map(move |x| (x, y)))
        .filter(|&(x, y)| (0..W()).contains(&x) && (0..H()).contains(&y)).map(|(x, y)| (y * W() + x) as usize).collect();
    if bx.is_empty() { return touched; }
    let flags: V = bx.iter().map(|&p| (g.mat[p] < 0) as i64).collect();
    let n = o.runsum(&[&flags]);
    let vis = one({ let t = o.lin(&[(K(*n.last().unwrap()), Some(K(100)))]); o.div(v(&t), K(bx.len() as i64), 0) });
    let vis = one(o.clamp(K(vis), 0, 100));
    // (the night's: as much as the sun's light is left)
    let vis = if env.unit != 1 { one({ let t = o.lin(&[(K(vis), Some(K(env.sun_col[0])))]); o.div(v(&t), K(65_000), 0) }) } else { vis };
    if vis <= 1 { return touched; }
    let disc = o.pct(v(&tb.t("disc").data), K(vis));
    let mut seen = vec![false; N_PIX()];
    for (gi, &(k, _, c)) in GHOSTS.iter().enumerate() {
        let gs = &w.ghosts[gi];
        let r = gs.r;
        // the ghost along the line through the middle: (middle - sun) k / 1000, cut toward zero, on the machines
        let toward = |o: &mut O, mid: i64, at: i64| -> i64 {
            let x = kl(o, &[(at, Some(-k)), (mid * k, None)]);
            if x < 0 { let p = kl(o, &[(x, Some(-1))]); let q = one(o.div(K(p), K(1000), 0)); kl(o, &[(q, Some(-1))]) } else { one(o.div(K(x), K(1000), 0)) }
        };
        let (ox, oy) = (toward(o, W() / 2, sx), toward(o, H() / 2, sy));
        let gx = one(o.lin(&[(K(sx), None), (K(ox), None)]));
        let gy = one(o.lin(&[(K(sy), None), (K(oy), None)]));
        // the sprite's places on the screen, and their profile indices
        let mut px = vec![];
        let mut ti = vec![];
        for (j, &t) in gs.t.iter().enumerate() {
            let (x, y) = (gx - r + j as i64 % (2 * r + 1), gy - r + j as i64 / (2 * r + 1));
            if (0..W()).contains(&x) && (0..H()).contains(&y) { px.push((y * W() + x) as usize); ti.push(t); }
        }
        if px.is_empty() { continue; }
        for ch in 0..3 {
            let prof = Tab { name: "ghost", data: o.mul(v(&disc), K(c[ch]), 3), w: 0 };
            let a = o.tab(&prof, v(&ti));
            let cur: V = px.iter().map(|&p| add[ch][p]).collect();
            let s = o.sum(&[(v(&cur), 1), (v(&a), 1)], 0);
            for (j, &p) in px.iter().enumerate() { add[ch][p] = s[j]; }
        }
        for &p in &px { if !seen[p] { seen[p] = true; touched.push(p); } }
    }
    touched
}

/// A box blur of radius r along each row (or column) of a w x h grid, as the prototype's box_blur: each place the
/// mean of the places within r, as many as there are (running sums; the mean by logarithms).
pub fn box_blur<O: Ops>(o: &mut O, tb: &Tb, a: &V, gw: usize, gh: usize, r: i64, rows: bool) -> V {
    let (n_lines, len) = if rows { (gh, gw) } else { (gw, gh) };
    let at = |l: usize, k: usize| -> usize { if rows { l * gw + k } else { k * gw + l } };
    let lists: Vec<V> = (0..n_lines).map(|l| (0..len).map(|k| a[at(l, k)]).collect()).collect();
    let refs: Vec<&[i64]> = lists.iter().map(|x| x.as_slice()).collect();
    let s = o.runsum(&refs);
    // the window: lo = max(k - r, 0), hi = min(k + r + 1, n); sum = S[hi - 1] - S[lo - 1]
    let mut hi_s = vec![0; n_lines * len];
    let mut lo_s = vec![0; n_lines * len];
    let mut cnt = vec![0; n_lines * len];
    for l in 0..n_lines {
        for k in 0..len as i64 {
            let (lo, hi) = ((k - r).max(0), (k + r + 1).min(len as i64));
            let i = l * len + k as usize;
            hi_s[i] = s[l * len + (hi - 1) as usize];
            lo_s[i] = if lo > 0 { s[l * len + (lo - 1) as usize] } else { 0 };
            cnt[i] = hi - lo;
        }
    }
    let sum = o.ldiff(&hi_s, &lo_s, 0, 999);
    let ls = o.tab(tb.t("ln"), v(&sum));
    let lc = o.tab(tb.t("ln_m1"), v(&cnt));
    let d = o.ldiff(&ls, &lc, 0, 79);
    let m = o.tab(tb.t("exp_long"), v(&d));
    let mut out = vec![0; a.len()];
    for l in 0..n_lines { for k in 0..len { out[at(l, k)] = m[l * len + k]; } }
    out
}

/// A grid of w x h made four times as fine each way: along the rows, then down the columns, between its cells' middles
/// (taken as 4 i + 2, half a pixel on) the steps of up4; beyond the outer middles the outer cells' values.
pub fn upsample<O: Ops>(o: &mut O, a: &V, gw: usize, gh: usize) -> V {
    let fine = |o: &mut O, lines: &[V]| -> Vec<V> {
        let refs: Vec<&[i64]> = lines.iter().map(|l| l.as_slice()).collect();
        let u = o.up4(&refs);
        let mut at = 0;
        lines.iter().map(|l| {
            let n = 4 * (l.len() - 1);
            let mut row = vec![l[0]; 2];
            row.extend_from_slice(&u[at..at + n]);
            row.extend_from_slice(&[l[l.len() - 1]; 2]);
            at += n;
            row
        }).collect()
    };
    let rows: Vec<V> = (0..gh).map(|y| a[y * gw..(y + 1) * gw].to_vec()).collect();
    let rows = fine(o, &rows);
    let fw = 4 * gw;
    let cols: Vec<V> = (0..fw).map(|x| rows.iter().map(|r| r[x]).collect()).collect();
    let cols = fine(o, &cols);
    let mut out = vec![0; fw * 4 * gh];
    for (x, c) in cols.iter().enumerate() { for (y, &val) in c.iter().enumerate() { out[y * fw + x] = val; } }
    out
}

/// The glow: what passes 1.3 (in twentieths of R) of the frame's 4x4 blocks (each block's four samples), blurred
/// with a radius of 4 blocks twice along rows and columns (about the prototype's 3 three times), that blurred again
/// with a radius of 14 twice (about the prototype's 12 three times from the start), 0.2 of each (four times, back in R), up to the
/// full size between the blocks' middles: made fine as its square root (a card, the table glow_enc; glow_dec back).
/// Light added (R) to `add`, which holds the flare's at the pixels it touched.
pub fn bloom<O: Ops>(o: &mut O, w: &World, col: &[V; 3], hot: &[V; 3], add: &mut [V; 3], touched: &[usize]) {
    let tb = &w.tb;
    let (gw, gh) = ((W() / 4) as usize, (H() / 4) as usize);
    // each block's four samples: (1, 1), (3, 1), (1, 3) and (3, 3) in it
    let samples: [Vec<usize>; 4] = std::array::from_fn(|j| (0..gw * gh).map(|b| { let (bx, by) = (b % gw, b / gw); (4 * by + 1 + 2 * (j / 2)) * W() as usize + 4 * bx + 1 + 2 * (j % 2) }).collect());
    let bright = |o: &mut O, c: usize, s: &[Vec<usize>; 4]| -> V {
        let cs: Vec<V> = s.iter().map(|q| q.iter().map(|&p| col[c][p]).collect()).collect();
        let hs: Vec<V> = s.iter().map(|q| q.iter().map(|&p| hot[c][p]).collect()).collect();
        let zero = vec![0; cs[0].len()];
        let ts: Vec<V> = (0..4).map(|j| o.tone_index(&cs[j], &hs[j], &zero, tb.t("x25"))).collect();
        let t = o.sum(&[(v(&ts[0]), 1), (v(&ts[1]), 1), (v(&ts[2]), 1), (v(&ts[3]), 1)], 0);
        o.tab(tb.t("bloom_br"), v(&t))
    };
    for c in 0..3 {
        let br = bright(o, c, &samples);
        if br.iter().all(|&x| x == 0) { continue; }
        let mut b1 = br;
        // (the radii in the full screen's blocks: halved at half resolution)
        let (r1, r2) = (4 / res(), 14 / res());
        for _ in 0..2 { b1 = box_blur(o, tb, &b1, gw, gh, r1, true); b1 = box_blur(o, tb, &b1, gw, gh, r1, false); }
        let mut b2 = b1.clone();
        for _ in 0..2 { b2 = box_blur(o, tb, &b2, gw, gh, r2, true); b2 = box_blur(o, tb, &b2, gw, gh, r2, false); }
        // the sum, as a card by its square root (the glow's table), made fine, and back (four times, in R)
        let gl = o.sum(&[(v(&b1), 1), (v(&b2), 1)], 0);
        let e = o.tab(tb.t("glow_enc"), v(&gl));
        let fine = upsample(o, &e, gw, gh);
        let full = o.tab(tb.t("glow_dec"), v(&fine));
        // where the flare added light, both
        let f: V = touched.iter().map(|&p| add[c][p]).collect();
        let gt: V = touched.iter().map(|&p| full[p]).collect();
        let both = o.sum(&[(v(&f), 1), (v(&gt), 1)], 0);
        let mut a = full;
        for (j, &p) in touched.iter().enumerate() { a[p] = both[j]; }
        add[c] = a;
    }
}
