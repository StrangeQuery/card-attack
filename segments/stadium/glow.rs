// The lights that aren't surfaces, added to the frame's light: dots (the city's lights, the airship's lights, the
// phones, the photo flashes, the confetti), the strobe in the haze, the lasers, the pyro, and the glow (bloom).
// Included after shade.rs.

/// The frame's light (ten-thousandths, each channel) and each pixel's K / z (0 for the sky).
pub struct Frame { pub col: [V; 3], pub iz: V, pub hit: Vec<bool> }

impl Frame {
    /// The pixels light was added to (the party version's full-size ones), when kept (`hit` not empty).
    pub fn mark(&mut self, px: &[usize]) { if !self.hit.is_empty() { for &p in px { self.hit[p] = true; } } }
}

/// The hypotenuse from a pixel's middle to a place in a neighbouring pixel: by the neighbour (dx, dy -1..1) and the
/// place's hundredths in its own pixel (fx, fy), hundredths: [((dx + 1) 3 + dy + 1) 10,000 + fx 100 + fy].
pub fn hyp_tab() -> Tab {
    let mut d = vec![0; 9 * 10_000];
    for dx in -1..=1i64 { for dy in -1..=1i64 { for fx in 0..100i64 { for fy in 0..100i64 {
        let (a, b) = ((dx * 100 + 50 - fx) as f64, (dy * 100 + 50 - fy) as f64);
        d[(((dx + 1) * 3 + dy + 1) * 10_000 + fx * 100 + fy) as usize] = (a.hypot(b)).round() as i64;
    } } } }
    tab("hyp", d)
}

/// Dots of light: camera-space places (centimetres), colours (ten-thousandths), sizes (hundredths of a pixel, at
/// least 50): each added to its pixel and the eight round it by how much of the dot covers each (size + 50 less
/// the distance to the pixel's middle, 0..100 hundredths), where nothing nearer stands in front (the pixel's K / z
/// under 1.005 times the dot's).
pub fn dots<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, pts: &[[i64; 3]], rgb: &[[i64; 3]], size: &V) {
    // in front (20 cm)
    let z: V = pts.iter().map(|p| p[2]).collect();
    if z.is_empty() { return; }
    let ahead = o.lt(K(20), v(&z));
    let at: Vec<usize> = (0..pts.len()).filter(|&i| ahead[i] == 1).collect();
    let pr = project(o, t, &at.iter().map(|&i| pts[i]).collect::<Vec<_>>());
    let xs: V = pr.iter().map(|p| p.0).collect();
    let ys: V = pr.iter().map(|p| p.1).collect();
    let (ix, fx) = o.divmod(v(&xs), K(100));
    let (iy, fy) = o.divmod(v(&ys), K(100));
    let sz: V = at.iter().map(|&i| size[i]).collect();
    let r = o.max(v(&sz), K(50));
    let izp: V = pr.iter().map(|p| p.2).collect();
    let lim = { let a = o.lin(&[(v(&izp), Some(K(5)))]); let a = o.div(v(&a), K(1000), 0); let izc = o.min(v(&izp), K(994_000)); o.sum(&[(v(&izc), 1), (v(&a), 1)], 0) };
    // the nine neighbours of each dot on the screen
    let (mut pix, mut who, mut hyp_i) = (vec![], vec![], vec![]);
    for (q, _) in at.iter().enumerate() {
        for dx in -1..=1i64 {
            for dy in -1..=1i64 {
                let (x, y) = (ix[q] + dx + scr_w() / 2, iy[q] + dy + scr_h() / 2);
                if x < 0 || x >= scr_w() || y < 0 || y >= scr_h() { continue; }
                pix.push((y * scr_w() + x) as usize);
                who.push(q);
                hyp_i.push(((dx + 1) * 3 + dy + 1) * 10_000 + fx[q] * 100 + fy[q]);
            }
        }
    }
    if pix.is_empty() { return; }
    let hy = o.tab(&t.hyp, v(&hyp_i));
    let rr: V = who.iter().map(|&q| r[q]).collect();
    let wgt = o.sum(&[(v(&rr), 1), (v(&hy), -1)], 50);
    let wgt = o.clamp(v(&wgt), 0, 100);
    let pz: V = pix.iter().map(|&p| f.iz[p]).collect();
    let lm: V = who.iter().map(|&q| lim[q]).collect();
    let vis = o.lt(v(&pz), v(&lm));
    let keep: Vec<usize> = (0..pix.len()).filter(|&k| vis[k] == 1).collect();
    let lw = { let w: V = keep.iter().map(|&k| wgt[k]).collect(); o.tabo(&t.ln, 999_999, v(&w)) };
    for c in 0..3 {
        let col: V = keep.iter().map(|&k| rgb[at[who[k]]][c]).collect();
        let lc = o.tabo(&t.ln, 999_999, v(&col));
        let y = o.sum(&[(v(&lc), 1), (v(&lw), 1)], -LN100);
        let add = expv(o, t, &y, None);
        let px: Vec<usize> = keep.iter().map(|&k| pix[k]).collect();
        add_to(o, &mut f.col[c], &px, &add);
        f.mark(&px);
    }
}

/// Light added to pixels (some more than once): in rounds, each pixel at most once a round.
pub fn add_to<O: Ops>(o: &mut O, img: &mut V, px: &[usize], add: &V) {
    // which pixels a round has taken: the round's number by pixel, kept between calls (a HashSet a round was 1.6% of
    // the workers' time); the first of a pixel's in `left` order wins, as before
    thread_local! { static STAMP: std::cell::RefCell<(Vec<u32>, u32)> = const { std::cell::RefCell::new((Vec::new(), 0)) }; }
    let mut left: Vec<usize> = (0..px.len()).collect();
    while !left.is_empty() {
        let (now, later): (Vec<usize>, Vec<usize>) = STAMP.with(|st| {
            let (stamp, round) = &mut *st.borrow_mut();
            if stamp.len() < img.len() { stamp.resize(img.len(), 0); }
            if *round == u32::MAX { stamp.fill(0); *round = 0; }
            *round += 1;
            let r = *round;
            left.iter().partition(|&&k| { let p = &mut stamp[px[k]]; if *p == r { false } else { *p = r; true } })
        });
        let old: V = now.iter().map(|&k| img[px[k]]).collect();
        let a: V = now.iter().map(|&k| add[k]).collect();
        let s = o.sum(&[(v(&old), 1), (v(&a), 1)], 0);
        for (q, &k) in now.iter().enumerate() { img[px[k]] = s[q]; }
        left = later;
    }
}

/// Sizes (hundredths of a pixel) of dots s centimetres across at depths K / z: f s / z, held to lo..hi.
pub fn dot_size<O: Ops>(o: &mut O, t: &Tables, iz: &V, s_cm: i64, lo: i64, hi: i64) -> V {
    // 100 f s / z = iz (100 000 s / KZ), by logarithms
    let l = o.tabo(&t.ln, 999_999, v(iz));
    let y = o.sum(&[(v(&l), 1)], lnf(100_000.0 * s_cm as f64 / KZ as f64));
    let a = expv(o, t, &y, None);
    o.clamp(v(&a), lo, hi)
}

/// The depth (K / z) at camera-space places, from the projection's table (for sizes).
pub fn iz_of<O: Ops>(o: &mut O, t: &Tables, pts: &[[i64; 3]]) -> V {
    let z: V = pts.iter().map(|p| p[2]).collect();
    let z = o.max(v(&z), K(20));
    let lz = o.tabo(&t.ln, 999_999, v(&z));
    let y = o.sum(&[(v(&lz), -1)], lnf(KZ as f64));
    expv(o, t, &y, None)
}

/// The strobe lighting the haze from 15 m out: everything nearer stands black against it. strobe 0.3 clip((min(z,
/// 150 m) - 15 m) / 60 m, 0, 1) (0.9, 0.92, 1), by the depth's printed table.
pub fn strobe_haze<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, strobe: i64) {
    if strobe == 0 { return; }
    let h = o.tab(&t.haze_k, v(&f.iz));
    for (c, k) in [900i64, 920, 1000].iter().enumerate() {
        let a = o.lin(&[(v(&h), Some(K(*k)))]);
        let a = o.div(v(&a), K(1000), 0);
        f.col[c] = o.sum(&[(v(&f.col[c]), 1), (v(&a), 1)], 0);
    }
}

impl Tables {
    /// the strobe's haze by K / z (0 for the sky: as far as 150 m): 3000 clip((min(z, 150) - 15) / 60, 0, 1),
    /// ten-thousandths; z = KZ / iz centimetres
    pub fn haze_k_tab() -> Tab {
        tab("haze_k", (0..1_000_000).map(|iz: i64| { let z = if iz == 0 { 1e9 } else { KZ as f64 / iz as f64 / 100.0 }; (3000.0 * ((z.min(150.0) - 15.0) / 60.0).clamp(0.0, 1.0)).round() as i64 }).collect())
    }
}

/// The glow: the light over 1.1 (each channel), its 4 x 4 blocks' means, blurred three times each way by 5 (the
/// edges held), back at full size, 0.8 of it added. Only the pixels over 1.1 add to a block (the rest add nothing),
/// each blur only round the blocks with any glow, the glow added only where there is any (the rest is nothing).
/// Means by logarithms (a sum's ln less ln 16 or ln 5).
pub fn bloom<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame) {
    let (bw, bh) = ((scr_w() / 4) as usize, (scr_h() / 4) as usize);
    let nb = bw * bh;
    let block = |p: usize| -> usize { ((p / scr_w() as usize) / 4) * bw + (p % scr_w() as usize) / 4 };
    for c in 0..3 {
        let over = o.lt(K(11_000), v(&f.col[c]));
        let bright: Vec<usize> = (0..npix()).filter(|&p| over[p] == 1).collect();
        if bright.is_empty() { continue; }
        let x: V = bright.iter().map(|&p| f.col[c][p]).collect();
        let x = o.sum(&[(v(&x), 1)], -11_000);
        // each pixel's excess over 16, summed into its block
        let x = mean_ln(o, t, &x, 16);
        let mut small = vec![0i64; nb];
        let bpx: Vec<usize> = bright.iter().map(|&p| block(p)).collect();
        add_to(o, &mut small, &bpx, &x);
        let mut live = vec![false; nb];
        for &b in &bpx { live[b] = true; }
        for _ in 0..3 {
            for axis in 0..2 {
                let (step, len) = if axis == 0 { (1usize, bw) } else { (bw, bh) };
                let pos = |i: usize| -> i64 { (if axis == 0 { i % bw } else { i / bw }) as i64 };
                // the blocks within 2 of a live one, along this axis, become live
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
        // 0.8 of it added, where it is
        let px: Vec<usize> = (0..npix()).filter(|&p| live[block(p)]).collect();
        let big: V = px.iter().map(|&p| small[block(p)]).collect();
        let a = { let l = o.tabo(&t.ln, 999_999, v(&big)); let y = o.sum(&[(v(&l), 1)], lnf(0.8)); expv(o, t, &y, None) };
        let old: V = px.iter().map(|&p| f.col[c][p]).collect();
        let s = o.sum(&[(v(&old), 1), (v(&a), 1)], 0);
        for (q, &p) in px.iter().enumerate() { f.col[c][p] = s[q]; }
    }
}

/// Sums over n by logarithms (0 stays 0).
fn mean_ln<O: Ops>(o: &mut O, t: &Tables, s: &V, n: i64) -> V {
    let l = o.tabo(&t.ln, 999_999, v(s));
    let y = o.sum(&[(v(&l), 1)], -lnf(n as f64));
    expv(o, t, &y, None)
}
