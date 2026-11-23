// The sun's shadows: depth maps looking down the sun's rays, one for the city by day and one
// for the evening's land, worked out once at the start from the world's triangles (the other train added each
// frame at dusk), and looked up with 16 taps for each 2x2 block of pixels. Texels 25 cm by day,
// 20 cm at dusk: a map's rows must be numbered by cards. Included after mirror.rs.

pub struct SMap {
    /// the sun's frame: across, up and along its rays (thousandths)
    pub r: [i64; 3], pub u: [i64; 3], pub f: [i64; 3],
    /// the map's middle (cm), its first across and last up (cm from the middle), texels (cm)
    pub c: [i64; 3], pub u0: i64, pub v1: i64, pub du: i64, pub dv: i64, pub w: i64, pub h: i64,
    /// the nearest depth along the rays at each texel (cm), 999,000 where nothing is
    pub d: Vec<i64>,
}

/// A map's frame from the sun's way: f its rays, r across (level), u up.
pub fn sun_frame<O: Ops>(o: &mut O, sun: [i64; 3]) -> ([i64; 3], [i64; 3], [i64; 3]) {
    let f = [-sun[0], -sun[1], -sun[2]];
    let l2 = o.lin(&[(K(f[2]), Some(K(f[2]))), (K(f[0]), Some(K(f[0])))]);
    let l = one(o.sqrt(v(&l2)));
    let rx = one({ let t = o.lin(&[(K(f[2]), Some(K(999)))]); o.div(v(&t), K(l), 0) });
    let rz = one({ let t = o.lin(&[(K(f[0]), Some(K(-999)))]); o.div(v(&t), K(l), 0) });
    let r = [rx, 0, rz];
    let u0 = one({ let t = o.lin(&[(K(f[1]), Some(K(rz)))]); o.div(v(&t), K(1000), 0) });
    let u1 = one({ let t = o.lin(&[(K(f[2]), Some(K(rx))), (K(f[0]), Some(K(-rz)))]); o.div(v(&t), K(1000), 0) });
    let u2 = one({ let t = o.lin(&[(K(f[1]), Some(K(-rx)))]); o.div(v(&t), K(1000), 0) });
    (r, [u0, u1, u2], f)
}

/// Places (cm) in a map's frame: across and up in hundredths of a texel from the map's corner, and the depth along
/// the rays (cm).
pub fn smap_coords<O: Ops>(o: &mut O, tb: &Tb, m: &SMap, p: &[&[i64]; 3]) -> (V, V, V) {
    let q: [V; 3] = std::array::from_fn(|i| { let t = o.lin(&[(A::V(p[i]), None), (K(-m.c[i]), None)]); o.clamp(v(&t), -250_000, 250_000) });
    let lq: [LogV; 3] = std::array::from_fn(|i| logv(o, tb, &q[i]));
    let dot = |o: &mut O, a: [i64; 3]| -> V { logdot(o, tb, [&lq[0], &lq[1], &lq[2]], a) };
    let pr = dot(o, m.r);
    let pr = o.clamp(v(&pr), -90_000, 90_000);
    let pu = dot(o, m.u);
    let pu = o.clamp(v(&pu), -90_000, 90_000);
    let pd = dot(o, m.f);
    // texels dividing a metre: hundredths of a texel by small multiples
    let (ku, kv) = (100 / m.du, 100 / m.dv);
    let su = o.lin(&[(v(&pr), Some(K(ku))), (K(-ku * m.u0), None)]);
    let sv = o.lin(&[(v(&pu), Some(K(-kv))), (K(kv * m.v1), None)]);
    (su, sv, pd)
}

/// Rasterize triangles (their corners' places, cm) into a map: each texel the nearest depth.
pub fn smap_raster<O: Ops>(o: &mut O, tb: &Tb, m: &mut SMap, corners: &[[i64; 3]], tris: &[[usize; 3]]) {
    if tris.is_empty() { return; }
    let mut used: Vec<usize> = tris.iter().flatten().copied().collect();
    used.sort();
    used.dedup();
    let at: std::collections::HashMap<usize, usize> = used.iter().enumerate().map(|(i, &u)| (u, i)).collect();
    let px: [V; 3] = std::array::from_fn(|i| used.iter().map(|&u| corners[u][i]).collect());
    let (su, sv, pd) = smap_coords(o, tb, m, &[&px[0], &px[1], &px[2]]);
    // as a screen: hundredths of a texel from the map's middle, the depth turned so the nearest is the largest
    // held to just past the map (a triangle reaching off it is drawn squeezed there)
    let su = o.clamp(v(&su), -1000, 100 * m.w + 1000);
    let sv = o.clamp(v(&sv), -1000, 100 * m.h + 1000);
    let x = o.lin(&[(v(&su), None), (K(-50 * m.w), None)]);
    let y = o.lin(&[(v(&sv), None), (K(-50 * m.h), None)]);
    let key = o.lin(&[(v(&pd), Some(K(-1)))]);
    let key = o.clamp(v(&key), -999_000, 999_000);
    let mut eidx: std::collections::HashMap<(usize, usize), usize> = Default::default();
    let mut edges: Vec<(usize, usize)> = vec![];
    for t in tris { for e in 0..3 { let (a, b) = (at[&t[e]], at[&t[(e + 1) % 3]]); let k = (a.min(b), a.max(b)); eidx.entry(k).or_insert_with(|| { edges.push(k); edges.len() - 1 }); } }
    // each edge's upper end, on the machines
    let (ya, yb): (V, V) = edges.iter().map(|&(a, b)| (y[a], y[b])).unzip();
    let ups = o.lt(v(&yb), v(&ya));
    let ends: Vec<([i64; 2], [i64; 2])> = edges.iter().enumerate().map(|(i, &(a, b))| if ups[i] == 1 { ([x[b], y[b]], [x[a], y[a]]) } else { ([x[a], y[a]], [x[b], y[b]]) }).collect();
    let rows = (-m.h / 2, m.h / 2);
    let cr = edge_rows(o, tb, &ends, rows);
    let (mut ta, mut tb_, mut tn) = (vec![], vec![], vec![]);
    for (i, &(a, b)) in edges.iter().enumerate() {
        if cr[i].1.is_empty() { continue; }
        let (top, bot) = if ups[i] == 1 { (b, a) } else { (a, b) };
        ta.push(key[top]); tb_.push(key[bot]); tn.push(cr[i].0);
    }
    let sl = slopes_rows(o, tb, &ta, &tb_, &tn);
    let wl = walk_from(o, &ta, &sl, &tn);
    let mut on_rows: Vec<Vec<(i64, i64, i64)>> = vec![vec![]; edges.len()];
    let mut wat = 0;
    for i in 0..edges.len() {
        if cr[i].1.is_empty() { continue; }
        for &(row, pxl, j) in &cr[i].1 { on_rows[i].push((row, pxl, wl[wat + j as usize])); }
        wat += cr[i].0 as usize;
    }
    let mut met: Vec<(i64, [(i64, i64); 2])> = vec![];
    for t in tris {
        let mut by_row: std::collections::BTreeMap<i64, Vec<(i64, i64)>> = Default::default();
        for e in 0..3 {
            let (a, b) = (at[&t[e]], at[&t[(e + 1) % 3]]);
            for &(row, pxl, z) in &on_rows[eidx[&(a.min(b), a.max(b))]] { by_row.entry(row).or_default().push((pxl, z)); }
        }
        for (row, c) in by_row {
            assert_eq!(c.len(), 2);
            met.push((row, [c[0], c[1]]));
        }
    }
    // the second crossing's texel less the first's on the machines: its sign says which end is the left, its size
    // (the second sum where it is below 0) the span's length
    let d = { let (pa, pb): (V, V) = met.iter().map(|m| (m.1[0].0, m.1[1].0)).unzip(); o.lin(&[(v(&pb), None), (v(&pa), Some(K(-1)))]) };
    let back: Vec<usize> = (0..met.len()).filter(|&i| d[i] < 0).collect();
    let nd = o.sum(&[(v(&back.iter().map(|&i| d[i]).collect()), -1)], 0);
    let mut len = d.clone();
    for (j, &i) in back.iter().enumerate() { len[i] = nd[j]; }
    let mut spans = vec![];
    let (mut za, mut zb, mut zn) = (vec![], vec![], vec![]);
    for (i, &(row, c)) in met.iter().enumerate() {
        if len[i] == 0 { continue; }
        let (l, r) = if d[i] < 0 { (c[1], c[0]) } else { (c[0], c[1]) };
        spans.push((row, l.0, r.0));
        za.push(l.1); zb.push(r.1); zn.push(len[i]);
    }
    let sl = slopes(o, tb, &za, &zb, &zn);
    let vals = walk_from(o, &za, &sl, &zn);
    // the pieces, by texel, the nearest kept (the depth test; ties keep the first)
    let mut per: std::collections::HashMap<usize, Vec<i64>> = Default::default();
    let mut at_ = 0;
    for &(row, a, b) in &spans {
        for xx in a..b {
            if xx >= -m.w / 2 && xx < m.w / 2 {
                let t = ((row + m.h / 2) * m.w + xx + m.w / 2) as usize;
                per.entry(t).or_default().push(vals[at_]);
            }
            at_ += 1;
        }
    }
    // (the map's depths turned on the machines to be weighed with the pieces, and the winning pieces' turned back)
    let texels: Vec<usize> = per.keys().copied().collect();
    let held: V = texels.iter().map(|t| m.d[*t]).collect();
    let turned = o.sum(&[(v(&held), -1)], 0);
    let groups: Vec<Vec<i64>> = texels.iter().enumerate().map(|(i, t)| { let mut g = vec![turned[i]]; g.extend(&per[t]); g }).collect();
    let refs: Vec<&[i64]> = groups.iter().map(|g| g.as_slice()).collect();
    let win = o.nearest(&refs);
    let new: Vec<usize> = (0..texels.len()).filter(|&i| win[i] > 0).collect();
    let back = o.sum(&[(v(&new.iter().map(|&i| groups[i][win[i]]).collect()), -1)], 0);
    for (j, &i) in new.iter().enumerate() { m.d[texels[i]] = back[j]; }
}

pub struct Shadows { pub day: SMap, pub dusk: SMap, pub depart: SMap }

/// The two maps, from the world's triangles: by day what stands in the city, at dusk what stands past the tunnel.
pub fn shadow_maps<O: Ops>(o: &mut O, tb: &Tb, sc: &Scene) -> Shadows {
    let mk = |o: &mut O, sun: [i64; 3], c: [i64; 3], u0: i64, u1: i64, v0: i64, v1: i64, du: i64, dv: i64| -> SMap {
        let (r, u, f) = sun_frame(o, sun);
        let (w, h) = ((u1 - u0) / du, (v1 - v0) / dv);
        let (w, h) = (w + w % 2, h + h % 2);
        SMap { r, u, f, c, u0, v1, du, dv, w, h, d: vec![999_000; (w * h) as usize] }
    };
    let mut day = mk(o, DAY.sun, [MID, 0, 6000], -15_000, 15_000, -23_000, 23_000, 25, 25);
    let mut dusk = mk(o, DUSK.sun, [MID + 4000, 0, 105_000], -45_000, 55_000, -8_000, 7_500, 20, 20);
    let near: Vec<[usize; 3]> = (0..sc.ride).filter(|&t| sc.world.tris[t].iter().any(|&u| sc.world.corners[u][2] < TUN0 + 6000)).map(|t| sc.world.tris[t]).collect();
    smap_raster(o, tb, &mut day, &sc.world.corners, &near);
    // the departure's: behind the ride's start (what stands there, the ride's own included)
    let mut depart = mk(o, DAY.sun, [MID, 0, -35_100], -15_000, 15_000, -24_000, 24_000, 25, 25);
    let behind: Vec<[usize; 3]> = (0..sc.world.tris.len()).filter(|&t| sc.world.tris[t].iter().any(|&u| sc.world.corners[u][2] < -6000)).map(|t| sc.world.tris[t]).collect();
    smap_raster(o, tb, &mut depart, &sc.world.corners, &behind);
    let far: Vec<[usize; 3]> = (0..sc.world.tris.len()).filter(|&t| sc.world.tris[t].iter().any(|&u| sc.world.corners[u][2] > TUN1 - 1000)).map(|t| sc.world.tris[t]).collect();
    smap_raster(o, tb, &mut dusk, &sc.world.corners, &far);
    Shadows { day, dusk, depart }
}

/// How much of the sun reaches places (percent): 16 taps of the map round each place (moved off its surface along
/// its normal), each in shadow when the map's depth there is nearer than the place's less a bias; 100 off the map.
pub fn sun_lit<O: Ops>(o: &mut O, tb: &Tb, m: &SMap, p: &[&[i64]; 3], n: &[&[i64]; 3], push: i64, bias: i64) -> V {
    let q: [V; 3] = std::array::from_fn(|i| { let t = o.mul(A::V(n[i]), K(push), 2); o.lin(&[(A::V(p[i]), None), (v(&t), None)]) });
    let (su, sv, pd) = smap_coords(o, tb, m, &[&q[0], &q[1], &q[2]]);
    let need = o.lin(&[(v(&pd), None), (K(-bias), None)]);
    let inu = { let a = o.lt(K(0), v(&su)); let b = o.lt(v(&su), K(100 * m.w)); o.mul(v(&a), v(&b), 0) };
    let inv = { let a = o.lt(K(0), v(&sv)); let b = o.lt(v(&sv), K(100 * m.h)); o.mul(v(&a), v(&b), 0) };
    let inmap = o.mul(v(&inu), v(&inv), 0);
    // the 4x4 texels round the place: its corner's column and row (held so the block is on the map); the host
    // fetches the block
    let bx = { let t = o.lin(&[(v(&su), None), (K(-150), None)]); let t = o.div(v(&t), K(100), 0); o.clamp(v(&t), 0, m.w - 4) };
    let by = { let t = o.lin(&[(v(&sv), None), (K(-150), None)]); let t = o.div(v(&t), K(100), 0); o.clamp(v(&t), 0, m.h - 4) };
    let taps: Vec<V> = (0..16).map(|j| (0..su.len()).map(|i| m.d[((by[i] + j / 4) * m.w + bx[i] + j % 4) as usize]).collect()).collect();
    let lit = o.lit16(&need, &taps);
    o.sel(&inmap, v(&lit), K(100))
}

/// The dusk's map with the other train in it this frame (its triangles at their places this frame).
pub fn train_shadow<O: Ops>(o: &mut O, w: &World, k: i64, _front: i64) -> SMap {
    let m = &w.sh.dusk;
    let mut t = SMap { r: m.r, u: m.u, f: m.f, c: m.c, u0: m.u0, v1: m.v1, du: m.du, dv: m.dv, w: m.w, h: m.h, d: m.d.clone() };
    let (_, tc) = train_corners(o, &w.sc, k);
    smap_raster(o, &w.tb, &mut t, &tc, &w.sc.train.tris);
    t
}
