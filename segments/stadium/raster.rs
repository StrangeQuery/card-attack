// Triangles onto the screen: clipped to the near plane and a guard band, their corners projected by logarithms, their
// edges' crossings of the rows (the crossings operation), the values down each edge and across each span by slopes
// (logarithms, walka), as the train draws. Every triangle's every piece on a pixel, with its depth and its values.
// Included after maths.rs.

/// Pieces of surfaces on pixels: the pixel, K / z, the triangle's id, and the values carried (k of them).
#[derive(Default)]
pub struct Pieces { pub px: Vec<u32>, pub iz: V, pub id: Vec<u32>, pub a: Vec<V> }

impl Pieces {
    pub fn new(k: usize) -> Pieces { Pieces { px: vec![], iz: vec![], id: vec![], a: vec![vec![]; k] } }
    pub fn len(&self) -> usize { self.px.len() }
}

/// A camera-space place (centimetres: right, up, forward) of a world place: B (p - eye) / 1000, each product by
/// logarithms (to a part in 10,000).
pub fn to_cam<O: Ops>(o: &mut O, t: &Tables, cam: &Cam, p: &[[i64; 3]]) -> Vec<[i64; 3]> {
    if p.is_empty() { return vec![]; }
    let d: Vec<V> = (0..3).map(|i| { let c: V = p.iter().map(|q| q[i]).collect(); let e = s_neg(cam.eye[i], o); o.sum(&[(v(&c), 1)], e) }).collect();
    let lr: [V; 3] = std::array::from_fn(|i| o.tabo(&t.ln, 999_999, v(&d[i])));
    let nr: [V; 3] = std::array::from_fn(|i| o.tabo(&t.neg, 999_999, v(&d[i])));
    let mut out = vec![[0; 3]; p.len()];
    for r in 0..3 {
        let s = logdot3(o, t, &lr, &nr, cam.b[r], 1000);
        for (k, x) in s.into_iter().enumerate() { out[k][r] = x; }
    }
    out
}

/// The guard band: x within 8 z / 9, y within 4 z / 9 (the screen is 0.768 z by 0.384 z; rows and columns stay
/// cards, and an edge's height under 1000 pixels).
const PLANES: usize = 5;
fn planes<O: Ops>(o: &mut O, c: &[[i64; 3]]) -> Vec<[i64; PLANES]> {
    let col = |i: usize| -> V { c.iter().map(|p| p[i]).collect() };
    let (x, y, z) = (col(0), col(1), col(2));
    let pn = o.sum(&[(v(&z), 1)], -NEAR);
    // the guard band's planes in centimetres, each place held to 550 m each way (so the sums stay longs; past that,
    // only the far ground's corners, the sides are told roughly)
    let (x, y, z) = (o.clamp(v(&x), -55_000, 55_000), o.clamp(v(&y), -55_000, 55_000), o.clamp(v(&z), -55_000, 55_000));
    let pr = o.sum(&[(v(&z), 8), (v(&x), -9)], 0);
    let pl = o.sum(&[(v(&z), 8), (v(&x), 9)], 0);
    let pt = o.sum(&[(v(&z), 4), (v(&y), -9)], 0);
    let pb = o.sum(&[(v(&z), 4), (v(&y), 9)], 0);
    (0..c.len()).map(|i| [pn[i], pr[i], pl[i], pt[i], pb[i]]).collect()
}

/// Camera-space places onto the screen: hundredths of a pixel from the middle (x right, y down) and K / z, by
/// logarithms: ln |x| + ln 100,000 less ln z, e back with x's sign.
pub fn project<O: Ops>(o: &mut O, t: &Tables, c: &[[i64; 3]]) -> Vec<(i64, i64, i64)> {
    if c.is_empty() { return vec![]; }
    let col = |i: usize| -> V { c.iter().map(|p| p[i]).collect() };
    let (x, y, z) = (col(0), col(1), col(2));
    let my = o.sum(&[(v(&y), -1)], 0);
    let lz = o.tabo(&t.ln, 999_999, v(&z));
    let part = |o: &mut O, u: &V| -> V {
        let (l, ng) = lnv(o, t, u);
        let s = o.sum(&[(v(&l), 1), (v(&lz), -1)], lnf(100_000.0 * focal() as f64 / 1000.0));
        expv(o, t, &s, Some(&ng))
    };
    let sx = part(o, &x);
    let sy = part(o, &my);
    let iz = { let s = o.sum(&[(v(&lz), -1)], lnf(KZ as f64)); expv(o, t, &s, None) };
    (0..c.len()).map(|i| (sx[i], sy[i], iz[i])).collect()
}

/// Slopes (b - a) 1000 / n, by logarithms: three cards.
pub fn slopes<O: Ops>(o: &mut O, t: &Tables, a: &V, b: &V, n: &V) -> V {
    if a.is_empty() { return vec![]; }
    let d = o.sum(&[(v(b), 1), (v(a), -1)], 0);
    let (l, ng) = lnv(o, t, &d);
    let ln_n = o.tabo(&t.ln, 999_999, v(n));
    let y = o.sum(&[(v(&l), 1), (v(&ln_n), -1)], LN1000);
    let mut out = vec![0; d.len()];
    for (table, want) in [(&t.expw, 0), (&t.expwn, 1)] {
        let at: Vec<usize> = (0..d.len()).filter(|&i| ng[i] == want).collect();
        if at.is_empty() { continue; }
        let yy: V = at.iter().map(|&i| y[i]).collect();
        let e = o.tabo(table, -EXP_LO, v(&yy));
        for (k, &i) in at.iter().enumerate() { out[i] = e[k]; }
    }
    out
}

/// Walks from values by slopes (thousandths a place): the k-th of n places va + fl(a (k + 1), 1000).
pub fn walk_from<O: Ops>(o: &mut O, va: &V, a: &V, n: &[i64]) -> V {
    if va.is_empty() { return vec![]; }
    let whole: V = a.iter().map(|&x| fl(x, 1000)).collect();
    let v1 = o.sum(&[(v(va), 1), (v(&whole), 1)], 0);
    let walks: Vec<(i64, i64, i64, i64)> = (0..va.len()).filter(|&j| n[j] > 0).map(|j| (n[j], v1[j], a[j], md(a[j], 1000))).collect();
    o.walka(&walks)
}

/// Draw triangles (corners in the camera's space, centimetres; k values at each corner): every piece on a pixel, in
/// the list's order. `cull`: leave out the ones facing away (closed models).
pub fn draw<O: Ops>(o: &mut O, t: &Tables, cv: &[[i64; 3]], vals: &[Vec<i64>], tris: &[[usize; 3]], ids: &[u32], cull: bool) -> Pieces {
    let k = vals.first().map_or(0, |a| a.len());
    let mut out = Pieces::new(k);
    if tris.is_empty() { return out; }
    // the corners in use, their planes
    let mut used: Vec<usize> = tris.iter().flatten().copied().collect();
    used.sort();
    used.dedup();
    let at: std::collections::HashMap<usize, usize> = used.iter().enumerate().map(|(i, &u)| (u, i)).collect();
    let mut verts: Vec<[i64; 3]> = used.iter().map(|&u| cv[u]).collect();
    let mut vval: Vec<Vec<i64>> = used.iter().map(|&u| vals[u].clone()).collect();
    let mut vpl = planes(o, &verts);
    let flat: V = vpl.iter().flatten().copied().collect();
    let outside = o.lt(v(&flat), K(0));
    let out_of = |i: usize, p: usize| outside[i * PLANES + p] == 1;
    let mut polys: Vec<(usize, Vec<usize>)> = vec![];
    for (ti, tr) in tris.iter().enumerate() {
        let c = tr.map(|u| at[&u]);
        if (0..PLANES).any(|p| c.iter().all(|&i| out_of(i, p))) { continue; }
        polys.push((ti, c.to_vec()));
    }
    // the ones crossing a plane clipped, plane after plane; a corner made where an edge crosses one is made once
    let crossing: Vec<usize> = (0..polys.len()).filter(|&j| polys[j].1.iter().any(|&i| (0..PLANES).any(|p| out_of(i, p)))).collect();
    let mut made: std::collections::HashMap<(usize, usize, usize), usize> = Default::default();
    for p in 0..PLANES {
        let mut need: Vec<(usize, usize)> = vec![];
        for &j in &crossing {
            let poly = &polys[j].1;
            let n = poly.len();
            for e in 0..n {
                let (a, b) = (poly[e], poly[(e + 1) % n]);
                let (ia, ib) = (vpl[a][p] >= 0, vpl[b][p] >= 0);
                if ia != ib {
                    let key = if ia { (a, b) } else { (b, a) };
                    if !made.contains_key(&(p, key.0, key.1)) && !need.contains(&key) { need.push(key); }
                }
            }
        }
        if !need.is_empty() {
            // s = dp / (dp - dq), thousandths; the new corner and its values p + fl((q - p) s, 1000)
            let dp: V = need.iter().map(|e| vpl[e.0][p]).collect();
            let dq: V = need.iter().map(|e| vpl[e.1][p]).collect();
            // (dp >= 0 > dq: both sides positive, by logarithms)
            let den = o.lin(&[(v(&dp), None), (v(&dq), Some(K(-1)))]);
            let lp = o.tabo(&t.ln, 999_999, v(&dp));
            let (ld, _) = lnw_v(o, t, &den);
            let y = o.sum(&[(v(&lp), 1), (v(&ld), -1)], LN1000);
            let s = expv(o, t, &y, None);
            let s = o.min(v(&s), K(1000));
            let lerp = |o: &mut O, a: &V, b: &V| -> V {
                let d = o.sum(&[(v(b), 1), (v(a), -1)], 0);
                let m = o.lin(&[(v(&d), Some(v(&s)))]);
                let m = o.div(v(&m), K(1000), 0);
                o.sum(&[(v(a), 1), (v(&m), 1)], 0)
            };
            let mut nc: Vec<[i64; 3]> = vec![[0; 3]; need.len()];
            for i in 0..3 {
                let a: V = need.iter().map(|e| verts[e.0][i]).collect();
                let b: V = need.iter().map(|e| verts[e.1][i]).collect();
                for (q, x) in lerp(o, &a, &b).into_iter().enumerate() { nc[q][i] = x; }
            }
            let mut nv: Vec<Vec<i64>> = vec![vec![0; k]; need.len()];
            for i in 0..k {
                let a: V = need.iter().map(|e| vval[e.0][i]).collect();
                let b: V = need.iter().map(|e| vval[e.1][i]).collect();
                for (q, x) in lerp(o, &a, &b).into_iter().enumerate() { nv[q][i] = x; }
            }
            // (a corner made at a plane is put on it: at the near plane its depth the plane's; at a side of the guard
            // band its x or y 8 z / 9 or 4 z / 9 (rounded inward), so no rounding of the cut leaves it outside)
            if p == 0 { for c in nc.iter_mut() { c[2] = NEAR; } } else {
                let z: V = nc.iter().map(|c| c[2]).collect();
                let k = if p <= 2 { 8 } else { 4 };
                let a = o.lin(&[(v(&z), Some(K(k)))]);
                let a = o.div(v(&a), K(9), 0);
                let na = o.sum(&[(v(&a), -1)], 0);
                for (q, c) in nc.iter_mut().enumerate() { match p { 1 => c[0] = a[q], 2 => c[0] = na[q], 3 => c[1] = a[q], _ => c[1] = na[q] } }
            }
            let np = planes(o, &nc);
            // a made corner is inside this plane by construction (its value 0 or a little either side)
            for (q, e) in need.iter().enumerate() {
                verts.push(nc[q]);
                vval.push(nv[q].clone());
                let mut pl = np[q];
                pl[p] = pl[p].max(0);
                vpl.push(pl);
                made.insert((p, e.0, e.1), verts.len() - 1);
                if std::env::var("STADIUM_CLIPCHECK").is_ok() && vpl.last().unwrap()[..p].iter().any(|&x| x < -20) { eprintln!("made at plane {p} from {:?} {:?} and {:?} {:?}: {:?} {:?} (s {})", verts[e.0], vpl[e.0], verts[e.1], vpl[e.1], nc[q], vpl.last().unwrap(), s[q]); }
            }
        }
        for &j in &crossing {
            let poly = polys[j].1.clone();
            let n = poly.len();
            let mut np = vec![];
            for e in 0..n {
                let (a, b) = (poly[e], poly[(e + 1) % n]);
                let (ia, ib) = (vpl[a][p] >= 0, vpl[b][p] >= 0);
                if ia { np.push(a); }
                if ia != ib { let key = if ia { (a, b) } else { (b, a) }; np.push(made[&(p, key.0, key.1)]); }
            }
            polys[j].1 = np;
        }
    }
    // the triangles: the whole ones, the clipped polygons' fans
    let mut tl: Vec<(usize, [usize; 3])> = vec![];
    for (ti, poly) in &polys {
        for j in 1..poly.len().saturating_sub(1) { tl.push((*ti, [poly[0], poly[j], poly[j + 1]])); }
    }
    if tl.is_empty() { return out; }
    if std::env::var("STADIUM_CLIPCHECK").is_ok() {
        let mut bad = 0;
        for (_, tr) in &tl { for &u in tr { if vpl[u].iter().any(|&x| x < 0) { bad += 1; if bad < 4 { eprintln!("outside: {:?} {:?}", verts[u], vpl[u]); } } } }
        if bad > 0 { eprintln!("clip: {bad} corners outside of {} triangles", tl.len()); }
    }
    // the corners on the screen
    let mut usedv: Vec<usize> = tl.iter().flat_map(|t| t.1).collect();
    usedv.sort();
    usedv.dedup();
    let pr = project(o, t, &usedv.iter().map(|&i| verts[i]).collect::<Vec<_>>());
    let mut scr: std::collections::HashMap<usize, (i64, i64, i64)> = Default::default();
    for (q, &i) in usedv.iter().enumerate() { scr.insert(i, pr[q]); }
    // the ones facing away left out: the sign of twice the area (tenths of a pixel)
    if cull {
        let g = |c: usize, f: usize| -> V { tl.iter().map(|t| { let p = scr[&t.1[c]]; if f == 0 { p.0 } else { p.1 } }).collect() };
        let (x0, x1, x2, y0, y1, y2) = (g(0, 0), g(1, 0), g(2, 0), g(0, 1), g(1, 1), g(2, 1));
        let tenth = |o: &mut O, a: &V, b: &V| -> V { let s = o.sum(&[(v(a), 1), (v(b), -1)], 0); o.div(v(&s), K(10), 0) };
        let (ax, ay, bx, by) = (tenth(o, &x1, &x0), tenth(o, &y1, &y0), tenth(o, &x2, &x0), tenth(o, &y2, &y0));
        let ayb = o.lin(&[(v(&ay), Some(v(&bx)))]);
        let axb = o.lin(&[(v(&ax), Some(v(&by)))]);
        // facing the camera: x right, y down, so a front face (counter-clockwise in the world) has area < 0
        let back = o.lt(v(&ayb), v(&axb));
        // only the ones wholly 20 m away or more: a solid's back can only show from inside it (the camera passes
        // through the stands), and there the prototype draws it
        let zmin = { let z: [V; 3] = std::array::from_fn(|c| tl.iter().map(|t| verts[t.1[c]][2]).collect()); let m = o.min(v(&z[0]), v(&z[1])); o.min(v(&m), v(&z[2])) };
        let far = o.lt(K(1999), v(&zmin));
        let drop: Vec<bool> = (0..tl.len()).map(|i| back[i] == 1 && far[i] == 1).collect();
        tl = tl.into_iter().zip(drop).filter(|(_, d)| !*d).map(|(t, _)| t).collect();
        if tl.is_empty() { return out; }
    }
    // each edge once, its upper end first
    let mut eindex: std::collections::HashMap<(usize, usize), usize> = Default::default();
    let mut edges: Vec<(usize, usize)> = vec![];
    for (_, tr) in &tl {
        for e in 0..3 {
            let (a, b) = (tr[e], tr[(e + 1) % 3]);
            let key = if a < b { (a, b) } else { (b, a) };
            eindex.entry(key).or_insert_with(|| { edges.push(key); edges.len() - 1 });
        }
    }
    let ends: Vec<([i64; 2], [i64; 2])> = edges.iter().map(|&(a, b)| {
        let (pa, pb) = (scr[&a], scr[&b]);
        if (pb.1, b) < (pa.1, a) { ([pb.0, pb.1], [pa.0, pa.1]) } else { ([pa.0, pa.1], [pb.0, pb.1]) }
    }).collect();
    let cross = o.crossings(&ends, (-scr_h() / 2, scr_h() / 2));
    // the values down each edge (depth and the k): from the top's by the slope over the rows and one
    let (mut ta, mut tb_, mut tn) = (vec![], vec![], vec![]);
    for (i, &(a, b)) in edges.iter().enumerate() {
        if cross[i].1.is_empty() { continue; }
        let (pa, pb) = (scr[&a], scr[&b]);
        let up = (pb.1, b) < (pa.1, a);
        let (top, bot) = if up { (b, a) } else { (a, b) };
        let rows = cross[i].0;
        ta.push(scr[&top].2); tb_.push(scr[&bot].2); tn.push(rows);
        for c in 0..k { ta.push(vval[top][c]); tb_.push(vval[bot][c]); tn.push(rows); }
    }
    let n1: V = tn.iter().map(|&r| r + 1).collect();
    let n1 = o.sum(&[(v(&n1), 1)], 0);
    let sl = slopes(o, t, &ta, &tb_, &n1);
    // only the rows on the screen are walked: each edge's first and how many
    let (mut k0s, mut cnts) = (vec![], vec![]);
    for i in 0..edges.len() {
        if cross[i].1.is_empty() { continue; }
        let j0 = cross[i].1[0].2;
        let cnt = cross[i].1.len() as i64;
        for _ in 0..=k { k0s.push(j0); cnts.push(cnt); }
    }
    let wl = walk_from_off(o, t, &ta, &sl, &k0s, &cnts);
    let mut wat = 0;
    // per edge: (row, pixel, depth, values)
    let mut on_rows: Vec<Vec<(i64, i64, Vec<i64>)>> = vec![vec![]; edges.len()];
    for i in 0..edges.len() {
        if cross[i].1.is_empty() { continue; }
        let cnt = cross[i].1.len();
        for (q, &(row, px, _)) in cross[i].1.iter().enumerate() {
            let vals: Vec<i64> = (0..=k).map(|c| wl[wat + c * cnt + q]).collect();
            on_rows[i].push((row, px, vals));
        }
        wat += (k + 1) * cnt;
    }
    // each triangle's spans: from its two edges' pixels on each row; values across by each span's slope
    let hw = scr_w() / 2;
    struct Span { id: u32, row: i64, from: i64, n: i64 }
    let mut spans: Vec<Span> = vec![];
    let (mut va, mut vb, mut nn) = (vec![], vec![], vec![]);
    for (ti, tr) in &tl {
        let mut by_row: std::collections::BTreeMap<i64, Vec<(i64, &Vec<i64>)>> = Default::default();
        for e in 0..3 {
            let (a, b) = (tr[e], tr[(e + 1) % 3]);
            let key = if a < b { (a, b) } else { (b, a) };
            for (row, px, vals) in &on_rows[eindex[&key]] { by_row.entry(*row).or_default().push((*px, vals)); }
        }
        for (row, mut c) in by_row {
            if c.len() != 2 { continue; }
            if c[1].0 < c[0].0 { c.swap(0, 1); }
            let ((pa, a), (pb, b)) = (c[0], c[1]);
            if pa == pb || pb <= -hw || pa >= hw { continue; }
            spans.push(Span { id: ids[*ti], row, from: pa, n: pb - pa });
            for q in 0..=k { va.push(a[q]); vb.push(b[q]); nn.push(pb - pa); }
        }
    }
    let nnv = o.sum(&[(v(&nn), 1)], 0);
    let sl = slopes(o, t, &va, &vb, &nnv);
    // only the pixels on the screen are walked
    let (mut k0s, mut cnts) = (vec![], vec![]);
    for sp in &spans {
        let k0 = (-hw - sp.from).max(0);
        let k1 = (hw - sp.from).min(sp.n);
        for _ in 0..=k { k0s.push(k0); cnts.push(k1 - k0); }
    }
    let sw = walk_from_off(o, t, &va, &sl, &k0s, &cnts);
    let mut at_ = 0;
    for (si, sp) in spans.iter().enumerate() {
        let (k0, m) = (k0s[si * (k + 1)], cnts[si * (k + 1)] as usize);
        for kx in 0..m {
            let x = sp.from + k0 + kx as i64;
            out.px.push(((sp.row + scr_h() / 2) * scr_w() + x + hw) as u32);
            out.iz.push(sw[at_ + kx]);
            out.id.push(sp.id);
            for c in 0..k { out.a[c].push(sw[at_ + (c + 1) * m + kx]); }
        }
        at_ += (k + 1) * m;
    }
    out
}

/// Walks from values by slopes, from place k0 for cnt places: the k-th va + fl(a (k + 1), 1000); from the first
/// place exactly (as walk_from), from a later one its start by logarithms (to a part in 10,000) and on by the slope.
pub fn walk_from_off<O: Ops>(o: &mut O, t: &Tables, va: &V, a: &V, k0: &[i64], cnt: &[i64]) -> V {
    let n = va.len();
    if n == 0 { return vec![]; }
    let later: Vec<usize> = (0..n).filter(|&j| k0[j] > 0 && cnt[j] > 0).collect();
    let mut start: V = { let whole: V = a.iter().map(|&x| fl(x, 1000)).collect(); o.sum(&[(v(va), 1), (v(&whole), 1)], 0) };
    let mut f0: V = a.iter().map(|&x| md(x, 1000)).collect();
    if !later.is_empty() {
        let g = |x: &[i64]| -> V { later.iter().map(|&j| x[j]).collect() };
        let kk: V = later.iter().map(|&j| k0[j] + 1).collect();
        let lk = o.tabo(&t.ln, 999_999, v(&kk));
        let (la, ng) = lnw_v(o, t, &g(a));
        let y = o.sum(&[(v(&la), 1), (v(&lk), 1)], -LN1000);
        let y = o.min(v(&y), K(138_000));
        let d = expv(o, t, &y, Some(&ng));
        let s = o.sum(&[(v(&g(va)), 1), (v(&d), 1)], 0);
        for (q, &j) in later.iter().enumerate() { start[j] = s[q]; f0[j] = 0; }
    }
    let walks: Vec<(i64, i64, i64, i64)> = (0..n).filter(|&j| cnt[j] > 0).map(|j| (cnt[j], start[j], a[j], f0[j])).collect();
    o.walka(&walks)
}

impl Tables {
    /// e of y / 10,000 up to three cards (for slopes), and its negative
    pub fn expw_tabs() -> (Tab, Tab) {
        let e: V = (EXP_LO..EXP_HI).map(|y| ((y as f64 / 10_000.0).exp().round() as i64).min(WIDE)).collect();
        let n = e.iter().map(|&x| -x).collect();
        (tab("expw", e), tab("expwn", n))
    }
}
