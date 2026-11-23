// The shading of what isn't sky: each pixel's place, its material's pattern (as filtered
// tables looked up by the place), the sun, the sky's light, the tunnel's lamps, the fog. Included after shade.rs.

/// The pixels being shaded, as vectors: their places (centimetres), the place across the line, the ray's unit way
/// (thousandths), the normal (hundredths), the depth's logarithm (ln K / z, ten-thousandths: what depends on the depth
/// alone, as the pixel's width there, is a table by it) and the footprint's level.
pub struct Px {
    pub idx: Vec<usize>,
    pub p: [V; 3],
    pub xr: V,
    pub d: [V; 3],
    pub n: [V; 3],
    pub liz: V,
    pub lvl: V,
    /// |d.n| (thousandths, at most 999)
    pub adn: V,
    pub mat: Vec<usize>,
}

impl Px {
    pub fn sub(&self, keep: &[usize]) -> Px {
        let g = |v: &V| -> V { keep.iter().map(|&i| v[i]).collect() };
        Px { idx: keep.iter().map(|&i| self.idx[i]).collect(), p: [g(&self.p[0]), g(&self.p[1]), g(&self.p[2])], xr: g(&self.xr),
             d: [g(&self.d[0]), g(&self.d[1]), g(&self.d[2])], n: [g(&self.n[0]), g(&self.n[1]), g(&self.n[2])],
             liz: g(&self.liz), lvl: g(&self.lvl), adn: g(&self.adn), mat: keep.iter().map(|&i| self.mat[i]).collect() }
    }
    pub fn len(&self) -> usize { self.idx.len() }
}

/// What a frame adds to the shading (the night arrival's): its own materials' kinds, light from its lamps, and its
/// screen's ray factors (its lens differs from the ride's). The ride's adds nothing (Plain).
pub trait Shading<O: Ops> {
    /// Shade the pixels of a kind the ride doesn't know.
    fn kind(&mut self, _o: &mut O, _w: &World, _env: &Env, _e: i64, _round: &SkyMap, _kd: i64, _px: &Px, _out: &mut Out) {}
    /// Light to add (R, exposed, each channel) at the pixels shaded (their normals after the materials; their kinds).
    fn light(&mut self, _o: &mut O, _w: &World, _px: &Px, _nn: &[V; 3], _kinds: &[i64], _tunnel: &V, _solid: &[usize]) -> Option<[V; 3]> { None }
    /// The screen's numbers for this frame's lens, if not the ride's.
    fn screen(&self) -> Option<&Screen> { None }
}

pub struct Plain;
impl<O: Ops> Shading<O> for Plain {}

/// A material's shading: albedo (thousandths), light it reflects and gives (R), and the part past R 999 (tenths).
pub struct Out { pub alb: [V; 3], pub spec: [V; 3], pub emit: [V; 3], pub hot: [V; 3], pub n: [V; 3] }

pub fn put3(dst: &mut [V; 3], at: &[usize], src: &[V; 3]) { for c in 0..3 { for (j, &i) in at.iter().enumerate() { dst[c][i] = src[c][j]; } } }

/// The footprint's level from millimetres.

/// A filtered band of half width (its table) at a distance from its middle.
pub fn band<O: Ops>(o: &mut O, tb: &Tb, name: &str, lvl: &V, u: &V, c: i64) -> V {
    let d = o.lin(&[(v(u), None), (K(-c), None)]);
    let m = o.lin(&[(v(&d), Some(K(-1)))]);
    let d = o.max(v(&d), v(&m));
    let d = o.min(v(&d), K(999));
    o.tab2(tb.t(name), v(lvl), v(&d))
}

/// A noise looked up at a place's last digits: across (a column) and along (a row), each md(fl(u, div), 1000).
pub fn noise<O: Ops>(o: &mut O, tb: &Tb, name: &str, a: &V, b: &V, div: i64) -> V {
    let i = o.wrap(v(a), div);
    let j = o.wrap(v(b), div);
    o.tab2(tb.t(name), v(&j), v(&i))
}

/// An albedo by a noise value from three tables (name0..2).
pub fn tint3<O: Ops>(o: &mut O, tb: &Tb, name: &str, n: &V) -> [V; 3] { std::array::from_fn(|c| o.tab(tb.t(&format!("{name}{c}")), v(n))) }

/// Each channel times a percentage.
pub fn scale3<O: Ops>(o: &mut O, a: &[V; 3], t: &V) -> [V; 3] { std::array::from_fn(|c| o.pct(v(&a[c]), v(t))) }

/// Each channel times thousandths (0..1999 as a card or long): a long result held to a card.
pub fn mul3<O: Ops>(o: &mut O, a: &[V; 3], t: &V) -> [V; 3] { std::array::from_fn(|c| { let r = o.lin(&[(v(&a[c]), Some(v(t)))]); let r = o.div(v(&r), K(1000), 0); o.clamp(v(&r), 0, 999) }) }

pub fn mix3<O: Ops>(o: &mut O, a: &[V; 3], b: &[V; 3], t: &V) -> [V; 3] { std::array::from_fn(|c| o.mix(v(&a[c]), v(&b[c]), v(t))) }
pub fn mix3k<O: Ops>(o: &mut O, a: &[V; 3], b: [i64; 3], t: &V) -> [V; 3] { std::array::from_fn(|c| o.mix(v(&a[c]), K(b[c]), v(t))) }
pub fn konst(n: usize, c: [i64; 3]) -> [V; 3] { [vec![c[0]; n], vec![c[1]; n], vec![c[2]; n]] }
pub fn zeros(n: usize) -> [V; 3] { konst(n, [0, 0, 0]) }

/// The ray's dot with the normal: thousandths (normals in hundredths, each product over 100).
pub fn dot_dn<O: Ops>(o: &mut O, d: &[V; 3], n: &[V; 3]) -> V {
    let a = o.mul(v(&d[0]), v(&n[0]), 2);
    let b = o.mul(v(&d[1]), v(&n[1]), 2);
    let c = o.mul(v(&d[2]), v(&n[2]), 2);
    let s = o.sum(&[(v(&a), 1), (v(&b), 1), (v(&c), 1)], 0);
    o.clamp(v(&s), -999, 999)
}

/// What a glossy surface reflects of the sky: the ray mirrored in the normal, kept from
/// going far below the horizon, looked up in the all-round map, times Schlick's reflectance (by the cosine; its table
/// for f0) and a tint (thousandths, each channel).
pub fn refl_sky<O: Ops>(o: &mut O, tb: &Tb, env: &Env, e: i64, round: &SkyMap, px: &Px, schlick: &str, tint: [i64; 3]) -> [V; 3] {
    let dn = dot_dn(o, &px.d, &px.n);
    // r = d - 2 (d.n) n: n in hundredths, so 2 (d.n) n / 100
    let mut r: [V; 3] = [vec![], vec![], vec![]];
    for i in 0..3 {
        let m = o.mul(v(&dn), v(&px.n[i]), 2);
        let m2 = o.lin(&[(v(&m), Some(K(-2))), (v(&px.d[i]), None)]);
        r[i] = o.clamp(v(&m2), -999, 999);
    }
    // up: |r.y| 0.7 + 0.3 r.y
    let neg = o.lt(v(&r[1]), K(0));
    let up = o.pct(v(&r[1]), K(40));
    let flip = o.sum(&[(v(&up), 1)], 0);
    let flip = o.lin(&[(v(&flip), Some(K(-1)))]);
    let ry = o.sel(&neg, v(&flip), v(&r[1]));
    let az = azimuth(o, tb, &r[0], &r[2]);
    let (sky, _) = sky_dir(o, tb, env, round, e, &az, &ry, None, None);
    let c = { let m = o.lin(&[(v(&dn), Some(K(-1)))]); let a = o.max(v(&dn), v(&m)); o.min(v(&a), K(1000)) };
    let f = o.tab(tb.t(schlick), v(&c));
    std::array::from_fn(|ch| { let s = o.mul(v(&sky[ch]), v(&f), 3); o.mul(v(&s), K(tint[ch]), 3) })
}

/// The deck: slab track (city, tunnel, bridge) or ballast (the farmland), the four rails shining with the sky.
pub fn deck<O: Ops>(o: &mut O, tb: &Tb, map: &SkyMap, px: &Px, out: &mut Out, in_tunnel_cam: bool) {
    let n = px.len();
    let z = &px.p[2];
    let xr = &px.xr;
    // the footprint across (the pixel's width) and along (its width times its depth over the eye's height): by depth
    let lx = o.tab(tb.t("lev_px"), v(&px.liz));
    let lx = lev_res(o, lx);
    let lz = o.tab(tb.t("lev_deck"), v(&px.liz));
    let lz = lev_res(o, lz);
    let xi = o.lin(&[(v(xr), None), (K(1000), None)]);
    let xi = o.clamp(v(&xi), 0, 1999);
    let slab = o.tab2(tb.t("deck_slab"), v(&lx), v(&xi));
    let slp = o.tab2(tb.t("deck_slp"), v(&lx), v(&xi));
    let dark = o.tab2(tb.t("deck_dark"), v(&lx), v(&xi));
    let rail = o.tab2(tb.t("deck_rail"), v(&lx), v(&xi));
    let (_, pj) = o.phase(v(z), 202, 5);
    let joint = o.tab2(tb.t("deck_joint"), v(&lz), v(&pj));
    let (_, ps) = o.phase(v(z), 167, 4);
    let sleeper = o.tab2(tb.t("deck_sleeper"), v(&lz), v(&ps));
    let nd = noise(o, tb, "n_deck", xr, z, 1);
    let ng = noise(o, tb, "n_gravel", xr, z, 1);
    let nv = noise(o, tb, "n_verge", xr, z, 1);
    let conc = tint3(o, tb, "deck_conc", &nd);
    let gravel = tint3(o, tb, "deck_gravel", &ng);
    let slabc = tint3(o, tb, "deck_slabc", &nd);
    let verge = tint3(o, tb, "deck_verge", &nv);
    // ballast beyond the tunnel, off the bridge
    let a = o.lt(K(TUN1), v(z));
    let b1 = o.lt(K(BR0), v(z));
    let b2 = o.lt(v(z), K(BR1));
    let br = o.mul(v(&b1), v(&b2), 0);
    let ballast = o.sum(&[(v(&a), 1), (v(&br), -1)], 0);
    let ballast = o.max(v(&ballast), K(0));
    // the slab less its joints (0.8 of the joint's share), the sleepers' share
    let j8 = o.pct(v(&joint), K(80));
    let jk = o.sum(&[(v(&j8), -1)], 100);
    let slab_e = o.pct(v(&slab), v(&jk));
    let sl = o.mul(v(&slp), v(&sleeper), 2);
    let on_slab = mix3(o, &conc, &slabc, &slab_e);
    let on_bal = mix3k(o, &gravel, [550, 540, 520], &sl);
    let mut col: [V; 3] = std::array::from_fn(|c| o.sel(&ballast, v(&on_bal[c]), v(&on_slab[c])));
    col = scale3(o, &col, &dark);
    // the cutting's grassy verge
    let d = o.lin(&[(v(xr), None), (K(-MID), None)]);
    let dm = o.lin(&[(v(&d), Some(K(-1)))]);
    let ad = o.max(v(&d), v(&dm));
    let wide = o.lt(K(620), v(&ad));
    let c1 = o.lt(K(TUN0 - 6000), v(z));
    let c2 = o.lt(v(z), K(TUN0));
    let cut = o.mul(v(&c1), v(&c2), 0);
    let cut = o.mul(v(&cut), v(&wide), 0);
    col = std::array::from_fn(|c| o.sel(&cut, v(&verge[c]), v(&col[c])));
    // the rails: in the tunnel lit metal, else the sky mirrored (not while we are in the tunnel)
    let t1 = o.lt(K(TUN0), v(z));
    let t2 = o.lt(v(z), K(TUN1));
    let intun = o.mul(v(&t1), v(&t2), 0);
    let railc = o.sel(&intun, K(600), K(0));
    let railc3 = [railc.clone(), railc.clone(), o.sel(&intun, K(620), K(0))];
    col = mix3(o, &col, &railc3, &rail);
    out.alb = col;
    out.spec = zeros(n);
    if !in_tunnel_cam {
        let shine: Vec<usize> = (0..n).filter(|&i| rail[i] > 0 && intun[i] == 0).collect();
        if !shine.is_empty() {
            let g = |v: &V| -> V { shine.iter().map(|&i| v[i]).collect() };
            let az = azimuth(o, tb, &g(&px.d[0]), &g(&px.d[2]));
            let dy = g(&px.d[1]);
            let up = o.lin(&[(v(&dy), Some(K(-1)))]);
            let sky = sky_look(o, tb, map, &az, &up);
            let r = g(&rail);
            let k = o.muldiv(v(&r), K(85), 100);
            for c in 0..3 {
                let s = o.pct(v(&sky[c]), v(&k));
                for (j, &i) in shine.iter().enumerate() { out.spec[c][i] = s[j]; }
            }
        }
    }
}

/// Where the paddies' water (and the river's) shows the sky: mirrored directions from rays and ripple leans.
pub struct Mirror { pub az: V, pub up: V, pub r: [V; 3] }

/// The paddies: water between the seedlings mirroring the sky, the dikes, the farm road; the wet share and the
/// reflectance for the mirror (which the caller fills: the sky, or what stands above the water).
pub fn paddy<O: Ops>(o: &mut O, tb: &Tb, px: &Px, k: i64, out: &mut Out) -> (V, V, [V; 2], Mirror) {
    let n = px.len();
    let (x, z) = (&px.p[0], &px.p[2]);
    let lf = &px.lvl;
    let xo = o.lin(&[(v(x), None), (K(700), None)]);
    let zo = o.lin(&[(v(z), None), (K(300), None)]);
    let (cx, pu) = o.phase(v(&xo), 32, 5);
    let (cz, pv) = o.phase(v(&zo), 53, 5);
    let du = o.tab2(tb.t("dike_u"), v(lf), v(&pu));
    let dv = o.tab2(tb.t("dike_v"), v(lf), v(&pv));
    let dike = o.max(v(&du), v(&dv));
    let h = o.tab2(tb.t("hash"), v(&cz), v(&cx));
    let flooded = o.lt(v(&h), K(780));
    let grown = o.lt(v(&h), K(920));
    // the rows: 30 cm across with a wobble, dashes 18 cm along (the footprint along: width times depth over 6 m)
    let wob = o.tab(tb.t("paddy_wobble"), v(&pv));
    let (_, pr) = o.phase(v(x), 333, 4);
    let pr = o.sum(&[(v(&pr), 1), (v(&wob), 1)], 1000);
    let pr = o.wrap(v(&pr), 1);
    let rowa = o.tab2(tb.t("paddy_row"), v(lf), v(&pr));
    let lz = o.tab(tb.t("lev_paddy"), v(&px.liz));
    let lz = lev_res(o, lz);
    let (_, pd) = o.phase(v(z), 556, 4);
    let rowb = o.tab2(tb.t("paddy_dash"), v(&lz), v(&pd));
    let rowc = o.mul(v(&rowa), v(&rowb), 2);
    let rowc = o.pct(v(&rowc), K(90));
    let rows = o.sel(&flooded, v(&rowc), K(0));
    let ngrn = noise(o, tb, "n_grn", x, z, 1);
    let nsoil = noise(o, tb, "n_soil", x, z, 1);
    let ngrass = noise(o, tb, "n_grass", x, z, 1);
    let seed = tint3(o, tb, "paddy_seed", &h);
    let grn = tint3(o, tb, "paddy_grn", &ngrn);
    let soil = tint3(o, tb, "paddy_soil", &nsoil);
    let grass = tint3(o, tb, "paddy_grass", &ngrass);
    let wet_a = mix3k(o, &konst(n, [100, 80, 50]), [0, 0, 0], &vec![0; n]);
    let wet_a = mix3(o, &wet_a, &seed, &rows);
    let dry: [V; 3] = std::array::from_fn(|c| o.sel(&grown, v(&grn[c]), v(&soil[c])));
    let a: [V; 3] = std::array::from_fn(|c| o.sel(&flooded, v(&wet_a[c]), v(&dry[c])));
    let road = band(o, tb, "band_road", lf, &px.xr, MID - 1640);
    let a = mix3(o, &a, &grass, &dike);
    let a = mix3k(o, &a, [330, 300, 260], &road);
    // the wet share: flooded, off the dikes, the rows and the road
    let w1 = o.sum(&[(v(&dike), -1)], 100);
    let w2 = o.sum(&[(v(&rows), -1)], 100);
    let w3 = o.sum(&[(v(&road), -1)], 100);
    let wet = o.mul(v(&w1), v(&w2), 2);
    let wet = o.mul(v(&wet), v(&w3), 2);
    let wet = o.sel(&flooded, v(&wet), K(0));
    let dk = o.pct(v(&wet), K(-60));
    let dk = o.sum(&[(v(&dk), 1)], 100);
    out.alb = scale3(o, &a, &dk);
    // the ripples (slowly moving): the normal's lean across and along, thousandths of a unit
    // 43 k / 10 cut toward zero, on the machines
    let shift = { let a = kl(o, &[(k, Some(if k < 0 { -43 } else { 43 }))]); let q = one(o.div(K(a), K(10), 0)); if k < 0 { kl(o, &[(q, Some(-1))]) } else { q } };
    let (sp, sn) = (kl(o, &[(shift, Some(10))]), kl(o, &[(shift, Some(-10))]));
    let xs = o.lin(&[(v(x), None), (K(sp), None)]);
    let zs = o.lin(&[(v(z), None), (K(sn), None)]);
    let nx = noise(o, tb, "n_ripx", &xs, z, 10);
    let nz = noise(o, tb, "n_ripz", x, &zs, 10);
    let nx = o.sum(&[(v(&nx), 1)], 0);
    // the mirrored ray: across and along less 2 (height part) times the lean, up mirrored
    let (rx, up, rz, _) = ripple_mirror(o, &px.d, &nx, &nz, 0);
    let az = azimuth(o, tb, &rx, &rz);
    let ndy = o.lin(&[(v(&px.d[1]), Some(K(-1)))]);
    let c = o.clamp(v(&ndy), 0, 1000);
    let f = o.tab(tb.t("schlick02"), v(&c));
    let upc = up.clone();
    (wet, f, [nx, nz], Mirror { az, up, r: [rx, upc, rz] })
}

/// A ray mirrored in rippled water: the water's normal (lean across, 1, lean along; thousandths), r = d - 2 (d.n) n,
/// its height part made up (plus `lift`). Returns r's parts and d.n.
pub fn ripple_mirror<O: Ops>(o: &mut O, d: &[V; 3], nx: &V, nz: &V, lift: i64) -> (V, V, V, V) {
    let a = o.mul(v(&d[0]), v(nx), 3);
    let b = o.mul(v(&d[2]), v(nz), 3);
    let dwn = o.sum(&[(v(&d[1]), 1), (v(&a), 1), (v(&b), 1)], 0);
    let dwn = o.clamp(v(&dwn), -999, 999);
    let mx = o.mul(v(&dwn), v(nx), 3);
    let rx = { let t = o.sum(&[(v(&d[0]), 1), (v(&mx), -2)], 0); o.clamp(v(&t), -999, 999) };
    let mz = o.mul(v(&dwn), v(nz), 3);
    let rz = { let t = o.sum(&[(v(&d[2]), 1), (v(&mz), -2)], 0); o.clamp(v(&t), -999, 999) };
    let ry = o.sum(&[(v(&d[1]), 1), (v(&dwn), -2)], 0);
    let neg = o.lin(&[(v(&ry), Some(K(-1)))]);
    let up = o.max(v(&ry), v(&neg));
    let up = { let t = o.sum(&[(v(&up), 1)], lift); o.clamp(v(&t), 0, 999) };
    (rx, up, rz, dwn)
}

/// Ground of one kind and a noise: the embankment, the river's banks.
pub fn ground_tint<O: Ops>(o: &mut O, tb: &Tb, px: &Px, nz: &str, t: &str) -> [V; 3] {
    let nn = noise(o, tb, nz, &px.p[0], &px.p[2], 1);
    tint3(o, tb, t, &nn)
}

/// The river: wide slow ripples, the sky and what stands above mirrored, the sun's path on it.
pub fn water<O: Ops>(o: &mut O, tb: &Tb, env: &Env, px: &Px, k: i64, e: i64, out: &mut Out) -> (V, [V; 2], Mirror) {
    let n = px.len();
    let (x, z) = (&px.p[0], &px.p[2]);
    let (sp, sn) = (kl(o, &[(k, Some(25))]), kl(o, &[(k, Some(-25))]));
    let xs = o.lin(&[(v(x), None), (K(sp), None)]);
    let zs = o.lin(&[(v(z), None), (K(sn), None)]);
    let nx = noise(o, tb, "n_rivx", &xs, z, 10);
    let nz = noise(o, tb, "n_rivz", x, &zs, 10);
    let (rx, up, rz, dwn) = ripple_mirror(o, &px.d, &nx, &nz, 2);
    let az = azimuth(o, tb, &rx, &rz);
    let ndw = { let t = o.lin(&[(v(&dwn), Some(K(-1)))]); o.clamp(v(&t), 0, 1000) };
    let f = o.tab(tb.t("schlick02"), v(&ndw));
    // the sun's glint: by the mirrored ray's squared distance from the sun's way
    let r = [rx.clone(), up.clone(), rz.clone()];
    let mut d2 = vec![];
    for i in 0..3 {
        let ns = kl(o, &[(env.sun[i], Some(-1))]);
        let d = o.sum(&[(v(&r[i]), 1)], ns);
        let d = o.clamp(v(&d), -999, 999);
        d2.push(o.mul(v(&d), v(&d), 0));
    }
    let s = o.lin(&[(v(&d2[0]), None), (v(&d2[1]), None), (v(&d2[2]), None)]);
    let s = o.clamp(v(&s), 0, 99_999);
    // times the reflectance, three times, then the exposure: R, the part past a card in tenths
    let f3 = o.lin(&[(v(&f), Some(K(3)))]);
    for c in 0..3 {
        let g = o.tab(tb.t(&format!("glint{c}")), v(&s));
        let g = o.lin(&[(v(&g), Some(v(&f3)))]);
        let g = o.div(v(&g), K(1000), 0);
        let g = o.min(v(&g), K(99_999));
        let g = o.lin(&[(v(&g), Some(K(e)))]);
        let g = o.div(v(&g), K(1000), 0);
        let over = { let t = o.lin(&[(v(&g), None), (K(-999), None)]); let t = o.max(v(&t), K(0)); let t = o.div(v(&t), K(25), 0); o.min(v(&t), K(999)) };
        out.emit[c] = o.min(v(&g), K(999));
        out.hot[c] = over;
    }
    out.alb = konst(n, [30, 50, 50]);
    (f, [nx, nz], Mirror { az, up: up.clone(), r: [rx, up, rz] })
}

/// Grime on a wall: streaks from the top, drips under it, dirt over the foot,
/// blotches; a factor in thousandths (at most 0.6 off).
pub fn weathering<O: Ops>(o: &mut O, tb: &Tb, u: &V, y: &V, top: &V, base: &V, seed: &V) -> V {
    let us = o.lin(&[(v(u), None), (v(seed), None)]);
    let st = noise(o, tb, "n_streak", &us, y, 1);
    let st = { let i = o.wrap(v(&us), 1); let j = o.wrap(v(y), 10); let _ = st; o.tab2(tb.t("n_streak"), v(&j), v(&i)) };
    let below = o.lin(&[(v(top), None), (v(y), Some(K(-1)))]);
    let bi = o.div(v(&below), K(10), 0);
    let bi = o.clamp(v(&bi), 0, 999);
    let stt = o.tab(tb.t("w_streak_top"), v(&bi));
    let streak = o.mul(v(&st), v(&stt), 3);
    let dr = { let i = o.wrap(v(&us), 10); let j = o.wrap(v(y), 10); o.tab2(tb.t("n_drip"), v(&j), v(&i)) };
    let dk = o.mul(v(&dr), K(400), 3);
    let dk = o.sum(&[(v(&dk), 1)], 600);
    let de = o.tab(tb.t("w_drip"), v(&bi));
    let drip = o.mul(v(&de), v(&dk), 3);
    let above = o.lin(&[(v(y), None), (v(base), Some(K(-1)))]);
    let ai = o.div(v(&above), K(10), 0);
    let ai = o.clamp(v(&ai), 0, 999);
    let dirt = o.tab(tb.t("w_dirt"), v(&ai));
    let bl = { let i = o.wrap(v(&us), 10); let j = o.wrap(v(y), 10); o.tab2(tb.t("n_blot"), v(&j), v(&i)) };
    let s1 = o.mul(v(&streak), K(220), 3);
    let s2 = o.mul(v(&drip), K(250), 3);
    let s3 = o.mul(v(&dirt), K(250), 3);
    let s4 = o.mul(v(&bl), K(120), 3);
    let s = o.sum(&[(v(&s1), 1), (v(&s2), 1), (v(&s3), 1), (v(&s4), 1)], 0);
    let s = o.clamp(v(&s), 0, 600);
    o.sum(&[(v(&s), -1)], 1000)
}

/// Along a wall: z where the normal faces across, else x.
pub fn along<O: Ops>(o: &mut O, px: &Px) -> V {
    let a = o.lin(&[(v(&px.n[0]), Some(K(-1)))]);
    let ax = o.max(v(&px.n[0]), v(&a));
    let side = o.lt(K(50), v(&ax));
    o.sel(&side, v(&px.p[2]), v(&px.p[0]))
}

pub fn par(sc: &Scene, px: &Px, i: usize) -> V { px.mat.iter().map(|&m| sc.mats[m].par[i]).collect() }
pub fn albedo(sc: &Scene, px: &Px) -> [V; 3] { std::array::from_fn(|c| px.mat.iter().map(|&m| sc.mats[m].alb[c].min(999)).collect()) }

/// Buildings: windows by floor and column (by the block's style), the glass mirroring the sky, blinds, weathering.
pub fn building<O: Ops>(o: &mut O, w: &World, env: &Env, e: i64, round: &SkyMap, px: &Px, out: &mut Out) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = px.len();
    let style = par(sc, px, 0);
    let gk = par(sc, px, 2);
    let rnd = par(sc, px, 3);
    let topv = par(sc, px, 4);
    let base = par(sc, px, 5);
    let u = along(o, px);
    let y = &px.p[1];
    let lf = &px.lvl;
    let top = o.lt(K(50), v(&px.n[1]));
    let vv = o.lin(&[(v(y), None), (K(-STREET), None)]);
    let v1 = o.lin(&[(v(&vv), None), (K(-100), None)]);
    let (fi, pv) = o.phase(v(&v1), 278, 5);
    let ur = o.lin(&[(v(&rnd), Some(K(3))), (v(&u), None)]);
    // the column's pitch as a rate, 100000 / pitch (at least 100), and the windows' second rate, 10 / 14 of it: for
    // each of the pixels' materials, on the machines
    let mut has = vec![false; sc.mats.len()];
    for &m in &px.mat { has[m] = true; }
    let ms: Vec<usize> = (0..sc.mats.len()).filter(|&m| has[m]).collect();
    let mp: V = ms.iter().map(|&m| sc.mats[m].par[1]).collect();
    let mp = o.max(v(&mp), K(100));
    let mr = o.div(K(100_000), v(&mp), 0);
    let mr2 = { let t = o.lin(&[(v(&mr), Some(K(10)))]); o.div(v(&t), K(14), 0) };
    let mut rate_of = vec![(0, 0); sc.mats.len()];
    for (j, &m) in ms.iter().enumerate() { rate_of[m] = (mr[j], mr2[j]); }
    let rp: V = px.mat.iter().map(|&m| rate_of[m].0).collect();
    let mut cu = vec![0; n];
    let mut pu = vec![0; n];
    let mut pa = vec![0; n];
    // the pixels gathered by rate on the card sorter, each rate's pixels in their order
    for at in sorter::groups(&rp.iter().map(|&r| [r]).collect::<Vec<_>>(), &[sorter::LONG]) {
        let (r, r2) = rate_of[px.mat[at[0]]];
        let g: V = at.iter().map(|&i| ur[i]).collect();
        let (c, p) = o.phase(v(&g), r, 5);
        let (_, p2) = o.phase(v(&g), r2, 5);
        for (j, &i) in at.iter().enumerate() { cu[i] = c[j]; pu[i] = p[j]; pa[i] = p2[j]; }
    }
    let (_, pm) = o.phase(v(&ur), 625, 5);
    let winv = o.tab2(tb.t("bld_winv"), v(lf), v(&pv));
    let winu = o.tab2(tb.t("bld_winu"), v(lf), v(&pu));
    let mull = o.tab2(tb.t("bld_mull"), v(lf), v(&pm));
    let p88 = { let t = o.sum(&[(v(&pv), 1)], 278); o.wrap(v(&t), 1) };
    let f88 = o.tab2(tb.t("bld_floor88"), v(lf), v(&p88));
    let pb = { let t = o.sum(&[(v(&pv), 1)], 333); o.wrap(v(&t), 1) };
    let balc = o.tab2(tb.t("bld_balc"), v(lf), v(&pb));
    let wina = o.tab2(tb.t("bld_wina"), v(lf), v(&pa));
    let g0 = o.mul(v(&winv), v(&winu), 2);
    let g1 = o.mul(v(&winv), v(&mull), 2);
    let g2 = o.mul(v(&f88), v(&mull), 2);
    let nb = o.sum(&[(v(&balc), -1)], 100);
    let g3 = o.mul(v(&nb), v(&wina), 2);
    let is = |o: &mut O, s: i64| -> V { let a = o.lt(K(s * 100 - 50), v(&style)); let b = o.lt(v(&style), K(s * 100 + 50)); o.mul(v(&a), v(&b), 0) };
    let s1 = is(o, 1);
    let s2 = is(o, 2);
    let s3 = is(o, 3);
    let glass = o.sel(&s1, v(&g1), v(&g0));
    let glass = o.sel(&s2, v(&g2), v(&glass));
    let glass = o.sel(&s3, v(&g3), v(&glass));
    let glass = o.sel(&top, K(0), v(&glass));
    let low = o.lt(v(&vv), K(900));
    let g30 = o.pct(v(&glass), K(30));
    let glass = o.sel(&low, v(&g30), v(&glass));
    // the wall: its colour a little varied over the block
    let alb = albedo(sc, px);
    let uvar = o.lin(&[(v(&u), None), (v(&rnd), Some(K(900)))]);
    let var = noise(o, tb, "n_bldvar", &uvar, &vv, 10);
    let vk = o.mul(v(&var), K(240), 3);
    let vk = o.sum(&[(v(&vk), 1)], 880);
    let a = mul3(o, &alb, &vk);
    let a: [V; 3] = std::array::from_fn(|c| o.sel(&top, K([380, 380, 370][c]), v(&a[c])));
    // each window: its hash, the inside dark, blinds half or all down, the lintel's shadow
    let ci = o.lin(&[(v(&cu), None), (v(&rnd), Some(K(31)))]);
    let ci = o.wrap(v(&ci), 1);
    let fj = o.lin(&[(v(&fi), None), (v(&rnd), Some(K(17)))]);
    let fj = o.wrap(v(&fj), 1);
    let wh = o.tab2(tb.t("hash"), v(&fj), v(&ci));
    let ik = o.mul(v(&wh), K(800), 3);
    let ik = o.sum(&[(v(&ik), 1)], 600);
    let inner = mul3(o, &konst(n, [25, 30, 35]), &ik);
    let b1 = o.lt(K(720), v(&wh));
    let b2 = o.sum(&[(v(&s2), -1)], 1);
    let blind = o.mul(v(&b1), v(&b2), 0);
    let all = o.lt(K(860), v(&wh));
    let need = o.sel(&all, K(0), K(247));
    let down = o.lt(v(&need), v(&pv));
    let blind = o.mul(v(&blind), v(&down), 0);
    let bl100 = o.sel(&blind, K(100), K(0));
    let inner = mix3k(o, &inner, [420, 400, 350], &bl100);
    let lint = o.tab(tb.t("bld_lintel"), v(&pv));
    let lint = o.mul(v(&lint), v(&b2), 0);
    let lk = o.pct(v(&lint), K(-60));
    let lk = o.sum(&[(v(&lk), 1)], 100);
    let inner = scale3(o, &inner, &lk);
    let a = mix3(o, &a, &inner, &glass);
    let walls = o.sel(&top, K(0), K(1));
    let wf = weathering(o, tb, &u, y, &topv, &base, &rnd);
    let wf1 = o.sel(&walls, v(&wf), K(1000));
    out.alb = mul3(o, &a, &wf1);
    // the glass mirrors the sky, where there is glass
    out.spec = zeros(n);
    let gl: Vec<usize> = (0..n).filter(|&i| glass[i] > 0).collect();
    if !gl.is_empty() {
        let sub = px.sub(&gl);
        let gkv: V = gl.iter().map(|&i| gk[i] / 100).collect();
        let rs = refl_sky(o, tb, env, e, round, &sub, "schlick25", [999, 999, 999]);
        const GT: [[i64; 3]; 4] = [[550, 750, 850], [500, 700, 600], [750, 620, 480], [620, 700, 800]];
        let g = |v: &V| -> V { gl.iter().map(|&i| v[i]).collect() };
        let whg = g(&wh);
        let k1 = o.mul(v(&whg), K(500), 3);
        let k1 = o.sum(&[(v(&k1), 1)], 750);
        let bk = { let b = g(&blind); let t = o.sel(&b, K(400), K(999)); t };
        let lk2 = { let l = g(&lint); let t = o.pct(v(&l), K(-70)); o.sum(&[(v(&t), 1)], 100) };
        let gg = g(&glass);
        for c in 0..3 {
            let tint: V = gkv.iter().map(|&k| GT[(k as usize).min(3)][c]).collect();
            let s = o.mul(v(&rs[c]), v(&tint), 3);
            let s = o.pct(v(&s), v(&lk2));
            let s = o.lin(&[(v(&s), Some(v(&k1)))]);
            let s = o.div(v(&s), K(1000), 0);
            let s = o.min(v(&s), K(999));
            let s = o.mul(v(&s), v(&bk), 3);
            let s = o.pct(v(&s), v(&gg));
            for (j, &i) in gl.iter().enumerate() { out.spec[c][i] = s[j]; }
        }
    }
    // after dark, windows lit here and there (each window its own hash), warm, the offices' cool; brighter as the
    // hash goes up
    if env.win > 0 {
        let gl: Vec<usize> = (0..n).filter(|&i| glass[i] > 0).collect();
        if !gl.is_empty() {
            let g = |v: &V| -> V { gl.iter().map(|&i| v[i]).collect() };
            let fo = { let t = o.sum(&[(v(&g(&fj)), 1)], 400); o.wrap(v(&t), 1) };
            let co = { let t = o.sum(&[(v(&g(&ci)), 1)], 600); o.wrap(v(&t), 1) };
            let hon = o.tab2(tb.t("hash"), v(&fo), v(&co));
            let on = { let t = { let wk = kl(o, &[(env.win, Some(7))]); one(o.div(K(wk), K(10), 0)) }; o.lt(v(&hon), K(t)) };
            let k = { let t = o.mul(v(&g(&wh)), K(600), 3); o.sum(&[(v(&t), 1)], 1000) };
            let k = { let t = o.lin(&[(v(&k), Some(K(env.win_k)))]); o.div(v(&t), K(1000), 0) };
            let k = { let t = o.mul(v(&g(&glass)), v(&on), 0); o.lin(&[(v(&t), Some(v(&k)))]) };
            let s2g = g(&s2);
            for c in 0..3 {
                let cool = env_k(o, env, [188, 213, 250][c] * env.unit, e);
                let warm = env_k(o, env, [250, 205, 145][c] * env.unit, e);
                let col = o.sel(&s2g, K(cool), K(warm));
                let t = o.lin(&[(v(&col), Some(v(&k)))]);
                let t = { let t = o.div(v(&t), K(1000), 0); o.div(v(&t), K(100), 0) };
                let s = o.sum(&[(v(&g(&out.emit[c])), 1), (v(&t), 1)], 0);
                let s = o.min(v(&s), K(999));
                for (j, &i) in gl.iter().enumerate() { out.emit[c][i] = s[j]; }
            }
        }
    }
}

/// The other materials, each a pattern on its albedo (and some a gloss).
pub fn simple<O: Ops>(o: &mut O, w: &World, env: &Env, round: &SkyMap, kd: i64, px: &Px, front: i64, e: i64, out: &mut Out) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = px.len();
    let (x, y, z) = (&px.p[0], &px.p[1], &px.p[2]);
    let lf = &px.lvl;
    let alb = albedo(sc, px);
    out.alb = alb.clone();
    out.spec = zeros(n);
    match kd {
        kind::ROOFTOP => {
            let nn = noise(o, tb, "n_rooftop", x, z, 1);
            let k = o.mul(v(&nn), K(400), 3);
            let k = o.sum(&[(v(&k), 1)], 800);
            out.alb = mul3(o, &alb, &k);
        }
        kind::CONC => {
            let u = along(o, px);
            let jl = par(sc, px, 0);
            let jl = o.clamp(v(&jl), 50, 999);
            let rate = o.tab(tb.t("conc_jl"), v(&jl));
            // joints: the place's phase in its joint length (rates differ by material)
            let mut ph = vec![0; n];
            for at in sorter::groups(&rate.iter().map(|&r| [r]).collect::<Vec<_>>(), &[sorter::LONG]) {
                let r = rate[at[0]];
                let g: V = at.iter().map(|&i| u[i]).collect();
                // (a rate past a card in tenths, over 10 on the machines)
                let (_, p) = if r > 999 { let r10 = one(o.div(K(r), K(10), 0)); o.phase(v(&g), r10, 4) } else { o.phase(v(&g), r, 5) };
                for (j, &i) in at.iter().enumerate() { ph[i] = p[j]; }
            }
            let j = o.tab2(tb.t("conc_joint"), v(lf), v(&ph));
            let nn = noise(o, tb, "n_conc", &u, y, 1);
            let st = o.mul(v(&nn), K(350), 3);
            let st = o.sum(&[(v(&st), 1)], 800);
            let jk = o.pct(v(&j), K(-50));
            let jk = o.sum(&[(v(&jk), 1)], 100);
            let a = mul3(o, &alb, &st);
            out.alb = scale3(o, &a, &jk);
        }
        kind::HILL => {
            // a wooded hillside: crowns as bumps, the normal leaned by the bumps' slope
            let xs = o.lin(&[(v(x), None), (v(y), Some(K(1)))]);
            let xs = { let t = o.div(v(y), K(10), 0); o.lin(&[(v(x), None), (v(&t), None)]) };
            let zs = { let t = o.div(v(y), K(10), 0); o.lin(&[(v(z), None), (v(&t), Some(K(-1)))]) };
            let _ = &xs;
            let h0 = noise(o, tb, "n_hill", &xs, &zs, 10);
            let xe = o.lin(&[(v(&xs), None), (K(60), None)]);
            let ze = o.lin(&[(v(&zs), None), (K(60), None)]);
            let hx = noise(o, tb, "n_hill", &xe, &zs, 10);
            let hz = noise(o, tb, "n_hill", &xs, &ze, 10);
            // the slope (per 0.6 m), times 9, onto the normal (hundredths)
            let gx = o.sum(&[(v(&hx), 1), (v(&h0), -1)], 0);
            let gz = o.sum(&[(v(&hz), 1), (v(&h0), -1)], 0);
            let bx = o.muldiv(v(&gx), K(-150), 100);
            let bz = o.muldiv(v(&gz), K(-150), 100);
            let nx = o.lin(&[(v(&px.n[0]), None), (v(&bx), None)]);
            let nz = o.lin(&[(v(&px.n[2]), None), (v(&bz), None)]);
            out.n = normalize(o, tb, &[nx, px.n[1].clone(), nz]);
            let gaps = o.tab(tb.t("hill_gaps"), v(&h0));
            let gk = o.muldiv(v(&gaps), K(11), 10);
            let gk = o.sum(&[(v(&gk), 1)], 350);
            let vv = noise(o, tb, "n_hillvar", x, z, 10);
            let vk = o.mul(v(&vv), K(400), 3);
            let vk = o.sum(&[(v(&vk), 1)], 800);
            let a = mul3(o, &alb, &gk);
            out.alb = mul3(o, &a, &vk);
        }
        kind::TUNNEL => {
            let zt = o.lin(&[(v(z), None), (K(-TUN0), None)]);
            let (_, pr) = o.phase(v(&zt), 95, 5);
            let ring = o.tab2(tb.t("tun_ring"), v(lf), v(&pr));
            let yx = { let xc = o.clamp(v(&px.xr), -999, 999); let a = o.pct(v(&xc), K(50)); o.lin(&[(v(y), None), (v(&a), None)]) };
            let nn = noise(o, tb, "n_tunnel", z, &yx, 1);
            let st = o.mul(v(&nn), K(500), 3);
            let st = o.sum(&[(v(&st), 1)], 700);
            let rk = o.pct(v(&ring), K(-60));
            let rk = o.sum(&[(v(&rk), 1)], 100);
            let a = mul3(o, &alb, &st);
            out.alb = scale3(o, &a, &rk);
        }
        kind::LAMP => {
            // 30 times (1, 0.72, 0.38): past a card, in tenths
            for (c, k) in [300i64, 216, 114].iter().enumerate() {
                let h = mulk(o, K(*k), K(e));
                out.hot[c] = vec![h[0]; n];
                out.emit[c] = vec![999; n];
            }
            out.alb = zeros(n);
        }
        kind::METAL | kind::TRUSS => {
            if kd == kind::METAL {
                let nn = noise(o, tb, "n_metal", y, z, 1);
                let k = o.mul(v(&nn), K(300), 3);
                let k = o.sum(&[(v(&k), 1)], 850);
                out.alb = mul3(o, &alb, &k);
            }
            out.spec = refl_sky(o, tb, env, e, round, px, "schlick6", [500, 500, 500]);
        }
        kind::TREE => tree(o, w, env, px, e, out),
        kind::CHOUSE => chouse(o, w, env, e, round, px, out),
        kind::SOLAR => {
            let (_, p1) = o.phase(v(x), 100, 4);
            let (_, p2) = o.phase(v(z), 606, 5);
            let g1 = o.tab2(tb.t("solar_x"), v(lf), v(&p1));
            let g2 = o.tab2(tb.t("solar_z"), v(lf), v(&p2));
            let grid = o.max(v(&g1), v(&g2));
            out.alb = mix3k(o, &alb, [500, 500, 520], &grid);
            let rs = refl_sky(o, tb, env, e, round, px, "schlick8", [800, 800, 800]);
            let ng = o.sum(&[(v(&grid), -1)], 100);
            out.spec = scale3(o, &rs, &ng);
        }
        kind::HOUSE => {
            let u = along(o, px);
            let vv = o.lin(&[(v(y), None), (K(-LAND), None)]);
            let (_, pp) = o.phase(v(&u), 549, 5);
            let post = o.tab2(tb.t("house_post"), v(lf), v(&pp));
            let vi = o.clamp(v(&vv), 0, 999);
            let lower = o.tab(tb.t("house_lower"), v(&vi));
            let k = o.sum(&[(v(&post), 1), (v(&lower), 1)], 0);
            let k = o.min(v(&k), K(100));
            let (_, pw) = o.phase(v(&u), 275, 5);
            let w1 = o.tab2(tb.t("house_win"), v(lf), v(&pw));
            let w2 = band(o, tb, "band_housewin", lf, &vv, 190);
            let win = o.mul(v(&w1), v(&w2), 2);
            let a = mix3k(o, &alb, [220, 140, 80], &k);
            out.alb = mix3k(o, &a, [40, 40, 50], &win);
            let rs = refl_sky(o, tb, env, e, round, px, "schlick10", [600, 600, 600]);
            out.spec = scale3(o, &rs, &win);
            if env.win > 0 {
                // after dark: each window (by its place along and the house) lit or not
                let (cw, _) = o.phase(v(&u), 275, 5);
                let hseed = { let t = par(sc, px, 0); let t = o.div(v(&t), K(20), 0); o.wrap(v(&t), 1) };
                let cw = { let t = o.sum(&[(v(&cw), 1)], 123); o.wrap(v(&t), 1) };
                let hon = o.tab2(tb.t("hash"), v(&hseed), v(&cw));
                let on = { let t = { let wk = kl(o, &[(env.win, Some(8))]); one(o.div(K(wk), K(10), 0)) }; o.lt(v(&hon), K(t)) };
                let k = o.mul(v(&win), v(&on), 0);
                let k = o.lin(&[(v(&k), Some(K(env.win_k)))]);
                for c in 0..3 {
                    let col = env_k(o, env, [325, 244, 146][c] * env.unit, e);
                    let t = o.lin(&[(v(&k), Some(K(col)))]);
                    let t = { let t = o.div(v(&t), K(1000), 0); o.div(v(&t), K(100), 0) };
                    out.emit[c] = o.min(v(&t), K(999));
                }
            }
        }
        kind::TILE => {
            let (_, pr) = o.phase(v(y), 300, 4);
            let l3 = o.tab(tb.t("lev_px3"), v(&px.liz));
            let l3 = lev_res(o, l3);
            let rows = o.tab2(tb.t("tile_rows"), v(&l3), v(&pr));
            let rk = o.pct(v(&rows), K(-40));
            let rk = o.sum(&[(v(&rk), 1)], 100);
            out.alb = scale3(o, &alb, &rk);
            out.spec = refl_sky(o, tb, env, e, round, px, "schlick8", [700, 700, 700]);
        }
        kind::PLATFORM => {
            // the platform: concrete with a little noise; on its top the yellow tactile strip and the white edge line
            let nd = noise(o, tb, "n_deck", x, z, 1);
            let k = o.mul(v(&nd), K(300), 3);
            let k = o.sum(&[(v(&k), 1)], 850);
            let mut a = mul3(o, &alb, &k);
            let top = o.lt(K(50), v(&px.n[1]));
            let left = o.lt(v(&px.xr), K(MID));
            for (side, strip_c, edge_c) in [(1, -260, -185), (0, X2 + 260, X2 + 185)] {
                let at: Vec<usize> = (0..n).filter(|&i| top[i] == 1 && left[i] == side).collect();
                if at.is_empty() { continue; }
                let sub_ = px.sub(&at);
                let s_ = band(o, tb, "band_plat16", &sub_.lvl, &sub_.xr, strip_c);
                let e_ = band(o, tb, "band_plat8", &sub_.lvl, &sub_.xr, edge_c);
                let a_: [V; 3] = std::array::from_fn(|c| at.iter().map(|&i| a[c][i]).collect());
                let a_ = mix3k(o, &a_, [850, 650, 50], &s_);
                let a_ = mix3k(o, &a_, [800, 800, 780], &e_);
                put3(&mut a, &at, &a_);
            }
            out.alb = a;
        }
        kind::LED => {
            // the departure board's dots, 7.25 cm apart (29 quarter centimetres), 128 across and 40 down, their block
            // 16 cm in from the board's left edge (x -940) and down from its top (715 cm): each dot's code from the
            // printed bitmap (edged with a dark dot all round, for the board's margin), its shape by the footprint
            // (the printed tables: the dot, and the glow a lit dot spreads when the footprint is wider than the
            // pitch), its colour (dark: a faint green; amber; green) with the exposure
            let u = o.lin(&[(v(x), Some(K(4))), (K(4 * 924 + 29), None)]);
            let u = o.clamp(v(&u), 0, 29 * 130 - 1);
            let vv = o.lin(&[(v(y), Some(K(-4))), (K(4 * 715 + 29), None)]);
            let vv = o.clamp(v(&vv), 0, 29 * 42 - 1);
            let (col, fu) = o.divmod(v(&u), K(29));
            let (row, fv) = o.divmod(v(&vv), K(29));
            let code = o.tab2(tb.t("led_bits"), v(&row), v(&col));
            let du = o.tab2(tb.t("led_dot"), v(lf), v(&fu));
            let dv = o.tab2(tb.t("led_dot"), v(lf), v(&fv));
            let dot = o.mul(v(&du), v(&dv), 2);
            let fill = o.tab(tb.t("led_fill"), v(lf));
            let dot = o.sum(&[(v(&dot), 1), (v(&fill), 1)], 0);
            let dot = o.min(v(&dot), K(100));
            out.emit = std::array::from_fn(|c| {
                let cols: V = vec![[300, 950, 350][c] * 3 / 400, [999, 620, 100][c], [300, 950, 350][c]];
                let t = Tab { name: "led_col", data: mulk(o, v(&cols), K(e)), w: 0 };
                let ce = o.tab(&t, v(&code));
                o.pct(v(&ce), v(&dot))
            });
            out.alb = konst(n, [20, 20, 20]);
        }
        kind::TUBE => {
            // the canopy's light tubes, on by day too (2.5 times their colour)
            out.emit = std::array::from_fn(|c| { let t = o.lin(&[(v(&alb[c]), Some(K(625)))]); let t = o.div(v(&t), K(1000), 0); mulk(o, v(&t), K(e)) });
        }
        kind::SHED => {
            let u = along(o, px);
            let (_, pc) = o.phase(v(&u), 500, 4);
            let cor = o.tab2(tb.t("shed_cor"), v(lf), v(&pc));
            let zy = o.lin(&[(v(z), None), (v(y), None)]);
            let nn = noise(o, tb, "n_shed", x, &zy, 1);
            let rust = o.tab(tb.t("shed_rust"), v(&nn));
            let ck = o.muldiv(v(&cor), K(3), 1);
            let ck = o.sum(&[(v(&ck), 1)], 750);
            let rk = o.mul(v(&rust), K(-300), 3);
            let rk = o.sum(&[(v(&rk), 1)], 1000);
            let a = mul3(o, &alb, &ck);
            let a = mul3(o, &a, &rk);
            let rr: [V; 3] = std::array::from_fn(|c| o.mul(v(&rust), K([120, 50, 20][c]), 3));
            out.alb = std::array::from_fn(|c| { let s = o.sum(&[(v(&a[c]), 1), (v(&rr[c]), 1)], 0); o.min(v(&s), K(999)) });
        }
        kind::WOOD => {
            let nn = noise(o, tb, "n_wood", x, y, 1);
            let k = o.mul(v(&nn), K(400), 3);
            let k = o.sum(&[(v(&k), 1)], 800);
            out.alb = mul3(o, &alb, &k);
        }
        kind::DARK => { out.alb = konst(n, [30, 30, 35]); }
        kind::SIGN => sign(o, w, env, px, e, out),
        kind::TRAIN => train(o, w, env, round, px, front, e, out),
        _ => {}
    }
}

/// A normal (hundredths) made a unit again: its size from the parts' squares (the printed table), each part 99 over
/// it by logarithms: ln |c| + ln 99 less half ln s (the printed tables), e back by the part's sign.
pub fn normalize<O: Ops>(o: &mut O, tb: &Tb, n: &[V; 3]) -> [V; 3] {
    let c: [V; 3] = std::array::from_fn(|i| o.clamp(v(&n[i]), -577, 577));
    let sq: [V; 3] = std::array::from_fn(|i| o.tabo(tb.t("sq577"), 577, v(&c[i])));
    let s = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
    let hl = o.tab(tb.t("half_ln4"), v(&s));
    std::array::from_fn(|i| {
        let l = o.tabo(tb.t("ln99c4"), 577, v(&c[i]));
        let k = o.ldiff(&l, &hl, 0, 139);
        let neg = o.tabo(tb.t("neg"), 999, v(&c[i]));
        let mut r = vec![0; k.len()];
        for (name, want) in [("exp4", 0), ("exp4n", 1)] {
            let at: Vec<usize> = (0..k.len()).filter(|&j| neg[j] == want).collect();
            let kk: V = at.iter().map(|&j| k[j]).collect();
            let e = o.tab(tb.t(name), v(&kk));
            for (q, &j) in at.iter().enumerate() { r[j] = e[q]; }
        }
        r
    })
}

/// Trees: leaves by a noise (its scale by species), the far side darker seen through the holes, the normal leaned,
/// backlit at their edges by the low sun.
pub fn tree<O: Ops>(o: &mut O, w: &World, env: &Env, px: &Px, e: i64, out: &mut Out) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = px.len();
    let (x, y, z) = (&px.p[0], &px.p[1], &px.p[2]);
    let sp = par(sc, px, 0);
    let seed = par(sc, px, 1);
    let dn = dot_dn(o, &px.d, &px.n);
    let away = o.lt(K(0), v(&dn));
    // the far side: -0.6 n + (0, 0.4, 0)
    let nf: [V; 3] = std::array::from_fn(|i| { let t = o.pct(v(&px.n[i]), K(-60)); if i == 1 { o.sum(&[(v(&t), 1)], 40) } else { t } });
    let nn: [V; 3] = std::array::from_fn(|i| o.sel(&away, v(&nf[i]), v(&px.n[i])));
    let y7 = { let t = o.lin(&[(v(y), Some(K(7)))]); o.div(v(&t), K(10), 0) };
    let y5 = { let t = o.lin(&[(v(y), Some(K(-1)))]); o.div(v(&t), K(2), 0) };
    let xs = o.lin(&[(v(x), None), (v(&y7), None)]);
    let zs = o.lin(&[(v(z), None), (v(&y5), None)]);
    let mut lv = vec![0; n];
    for (s, name) in [(0i64, "n_tree1"), (100, "n_tree2"), (200, "n_tree1"), (300, "n_tree3")] {
        let at: Vec<usize> = (0..n).filter(|&i| sp[i] == s).collect();
        if at.is_empty() { continue; }
        let g = |v: &V| -> V { at.iter().map(|&i| v[i]).collect() };
        let r = noise(o, tb, name, &g(&xs), &g(&zs), 1);
        for (j, &i) in at.iter().enumerate() { lv[i] = r[j]; }
    }
    let xo = o.lin(&[(v(x), None), (v(&seed), None)]);
    let tv = noise(o, tb, "n_treevar", &xo, z, 10);
    let tk = o.mul(v(&tv), K(300), 3);
    let tk = o.sum(&[(v(&tk), 1)], 850);
    let lk = o.muldiv(v(&lv), K(12), 10);
    let lk = o.sum(&[(v(&lk), 1)], 450);
    let ak = o.sel(&away, K(550), K(1000));
    let a = mul3(o, &albedo(sc, px), &lk);
    let a = mul3(o, &a, &tk);
    out.alb = mul3(o, &a, &ak);
    // the normal leaned by the leaves: + (lv - 0.5, 0, 0.5 - lv) 0.8
    let l5 = o.sum(&[(v(&lv), 1)], -500);
    let bx = o.muldiv(v(&l5), K(8), 100);
    let bz = o.lin(&[(v(&bx), Some(K(-1)))]);
    let nx = o.lin(&[(v(&nn[0]), None), (v(&bx), None)]);
    let nz = o.lin(&[(v(&nn[2]), None), (v(&bz), None)]);
    out.n = normalize(o, tb, &[nx, nn[1].clone(), nz]);
    // backlit: (d.sun)^6, the rim (1 - |d.n|)^2, (0.4 + lv) 1.6, times the sun's colour
    let s0 = o.mul(v(&px.d[0]), K(env.sun[0]), 3);
    let s1 = o.mul(v(&px.d[1]), K(env.sun[1]), 3);
    let s2 = o.mul(v(&px.d[2]), K(env.sun[2]), 3);
    let near = o.lin(&[(v(&s0), None), (v(&s1), None), (v(&s2), None), (K(1000), None)]);
    let near = o.clamp(v(&near), 0, 2000);
    let back = o.tab(tb.t("pow6"), v(&near));
    let dn2 = dot_dn(o, &px.d, &out.n);
    let adn = { let m = o.lin(&[(v(&dn2), Some(K(-1)))]); let a = o.max(v(&dn2), v(&m)); o.min(v(&a), K(1000)) };
    let one_less = o.sum(&[(v(&adn), -1)], 1000);
    let one_less = o.clamp(v(&one_less), 0, 1000);
    let rim = o.tab(tb.t("sq"), v(&one_less));
    let lk2 = o.sum(&[(v(&lv), 1)], 400);
    let lk2 = { let t = o.lin(&[(v(&lk2), Some(K(16)))]); o.div(v(&t), K(10), 0) };
    let k = o.mul(v(&back), v(&rim), 3);
    let k = o.lin(&[(v(&k), Some(v(&lk2)))]);
    let k = o.div(v(&k), K(1000), 0);
    let k = o.min(v(&k), K(999));
    for c in 0..3 {
        let sc_ = env_k(o, env, env.sun_col[c], e);
        let t = o.mul(v(&out.alb[c]), K(sc_), 3);
        out.emit[c] = o.mul(v(&t), v(&k), 3);
    }
}

/// The suburb's houses: siding, sashes and their glass, curtains, balcony rails, weathering.
pub fn chouse<O: Ops>(o: &mut O, w: &World, env: &Env, e: i64, round: &SkyMap, px: &Px, out: &mut Out) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = px.len();
    let y = &px.p[1];
    let lf = &px.lvl;
    let seed = par(sc, px, 0);
    let topv = par(sc, px, 1);
    let base = par(sc, px, 2);
    let sty = par(sc, px, 4);
    let u = along(o, px);
    let us = o.lin(&[(v(&u), None), (v(&seed), None)]);
    let vv = o.lin(&[(v(y), None), (v(&base), Some(K(-1)))]);
    let (cu, pu) = o.phase(v(&us), 312, 5);
    let (cv, pv) = o.phase(v(&vv), 351, 5);
    let hs = o.tab2(tb.t("hash"), v(&cv), v(&cu));
    let w1 = o.tab2(tb.t("ch_win"), v(lf), v(&pu));
    let w2 = o.tab2(tb.t("ch_winv"), v(lf), v(&pv));
    let win = o.mul(v(&w1), v(&w2), 2);
    let some = o.lt(K(350), v(&hs));
    let under = { let t = o.lin(&[(v(&topv), None), (v(&base), Some(K(-1))), (K(-30), None)]); o.lt(v(&vv), v(&t)) };
    let win = o.mul(v(&win), v(&some), 0);
    let win = o.mul(v(&win), v(&under), 0);
    let (_, ph) = o.phase(v(&vv), 476, 4);
    let lh = o.tab2(tb.t("ch_siding_h"), v(lf), v(&ph));
    let (_, pw) = o.phase(v(&u), 222, 4);
    let lw = o.tab2(tb.t("ch_siding_v"), v(lf), v(&pw));
    let s1 = o.lt(K(50), v(&sty));
    let s2 = o.lt(K(150), v(&sty));
    let lines = o.sel(&s1, v(&lh), K(0));
    let lines = o.sel(&s2, v(&lw), v(&lines));
    let lk = o.pct(v(&lines), K(-18));
    let lk = o.sum(&[(v(&lk), 1)], 100);
    let a = scale3(o, &albedo(sc, px), &lk);
    let gk = o.mul(v(&hs), K(900), 3);
    let gk = o.sum(&[(v(&gk), 1)], 600);
    let glass = mul3(o, &konst(n, [30, 35, 40]), &gk);
    let cur = o.lt(K(750), v(&hs));
    let glass: [V; 3] = std::array::from_fn(|c| { let t = o.sum(&[(v(&glass[c]), 1)], [400, 360, 300][c]); o.sel(&cur, v(&t), v(&glass[c])) });
    let a = mix3(o, &a, &glass, &win);
    let r1 = o.tab2(tb.t("ch_rail"), v(lf), v(&pv));
    let r2 = o.lt(K(600), v(&hs));
    let r3 = o.lt(K(200), v(&vv));
    let rail = o.mul(v(&r1), v(&r2), 0);
    let rail = o.mul(v(&rail), v(&r3), 0);
    let a = mix3k(o, &a, [550, 560, 580], &rail);
    let wf = weathering(o, tb, &u, y, &topv, &base, &seed);
    out.alb = mul3(o, &a, &wf);
    out.spec = zeros(n);
    let gl: Vec<usize> = (0..n).filter(|&i| win[i] > 0 && cur[i] == 0).collect();
    if !gl.is_empty() {
        let sub = px.sub(&gl);
        let rs = refl_sky(o, tb, env, e, round, &sub, "schlick12", [800, 800, 800]);
        let wg: V = gl.iter().map(|&i| win[i]).collect();
        let s = scale3(o, &rs, &wg);
        for c in 0..3 { for (j, &i) in gl.iter().enumerate() { out.spec[c][i] = s[c][j]; } }
    }
    if env.win > 0 {
        // after dark, lights coming on behind the windows (each window its own hash); behind the curtains warmer
        let lw: Vec<usize> = (0..n).filter(|&i| win[i] > 0).collect();
        if !lw.is_empty() {
            let g = |v: &V| -> V { lw.iter().map(|&i| v[i]).collect() };
            let cvo = { let t = o.sum(&[(v(&g(&cv)), 1)], 777); o.wrap(v(&t), 1) };
            let cuo = { let t = o.lin(&[(v(&g(&cu)), Some(K(3))), (v(&g(&seed)), None)]); o.wrap(v(&t), 1) };
            let hon = o.tab2(tb.t("hash"), v(&cvo), v(&cuo));
            let on = { let t = { let wk = kl(o, &[(env.win, Some(75))]); one(o.div(K(wk), K(100), 0)) }; o.lt(v(&hon), K(t)) };
            let k = o.mul(v(&g(&win)), v(&on), 0);
            let k = o.lin(&[(v(&k), Some(K(env.win_k)))]);
            let curg = g(&cur);
            for c in 0..3 {
                let warm = env_k(o, env, [350, 217, 105][c] * env.unit, e);
                let pale = env_k(o, env, [350, 280, 193][c] * env.unit, e);
                let col = o.sel(&curg, K(warm), K(pale));
                let t = o.lin(&[(v(&k), Some(v(&col)))]);
                let t = { let t = o.div(v(&t), K(1000), 0); o.div(v(&t), K(100), 0) };
                let s = o.sum(&[(v(&g(&out.emit[c])), 1), (v(&t), 1)], 0);
                let s = o.min(v(&s), K(999));
                for (j, &i) in lw.iter().enumerate() { out.emit[c][i] = s[j]; }
            }
        }
    }
}

/// Signs: characters of strokes (a table of 64), 1.25 m cells, stacked down a vertical sign or in a row on a board,
/// in white (red on a white sign); lit, and a quarter of their colour given out.
pub fn sign<O: Ops>(o: &mut O, w: &World, env: &Env, px: &Px, e: i64, out: &mut Out) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = px.len();
    let base = albedo(sc, px);
    let seed = par(sc, px, 0);
    let board = par(sc, px, 1);
    let u = along(o, px);
    let vert = o.lt(v(&board), K(50));
    let a = o.sel(&vert, v(&px.p[1]), v(&u));
    let b = o.sel(&vert, v(&u), v(&px.p[1]));
    let (ca, fa) = o.phase(v(&a), 800, 5);
    let (cb, fb) = o.phase(v(&b), 800, 5);
    let sd = o.div(v(&seed), K(100), 0);
    let g = o.lin(&[(v(&ca), None), (v(&sd), Some(K(13))), (v(&cb), Some(K(71)))]);
    let (_, g) = o.divmod(v(&g), K(64));
    let fa32 = o.div(v(&fa), K(32), 0);
    let fb32 = o.div(v(&fb), K(32), 0);
    let fa32 = o.min(v(&fa32), K(31));
    let fb32 = o.min(v(&fb32), K(31));
    let at = o.lin(&[(v(&fa32), Some(K(32))), (v(&fb32), None)]);
    let far = o.tab(tb.t("far_px60"), v(&px.liz));
    let g0 = o.tab2(tb.t("glyph0"), v(&g), v(&at));
    let g1 = o.tab2(tb.t("glyph1"), v(&g), v(&at));
    let gl = o.sel(&far, v(&g1), v(&g0));
    let ay = { let m = o.lin(&[(v(&px.n[1]), Some(K(-1)))]); o.max(v(&px.n[1]), v(&m)) };
    let face = o.lt(v(&ay), K(50));
    let gl = o.mul(v(&gl), v(&face), 0);
    let bright = { let s = o.sum(&[(v(&base[0]), 1), (v(&base[1]), 1), (v(&base[2]), 1)], 0); o.lt(K(2500), v(&s)) };
    let ink: [V; 3] = std::array::from_fn(|c| o.sel(&bright, K([600, 50, 50][c]), K([999, 999, 950][c])));
    out.alb = mix3(o, &base, &ink, &gl);
    let _ = n;
    // lit: a quarter of their colour given out, and after dark up to 2.75 times it
    let k = { let q = { let wk = kl(o, &[(env.win, Some(625))]); one(o.div(K(wk), K(1000), 0)) }; kl(o, &[(q, None), (63, None)]) };
    for c in 0..3 { let t = mulk(o, v(&out.alb[c]), K(e)); out.emit[c] = o.mul(v(&t), K(k), 3); }
}

/// The other train: white paint with a clear coat, the blue stripes, the windows, the nose's stripe and crease, its
/// lamps, a rim of the low sun.
pub fn train<O: Ops>(o: &mut O, w: &World, env: &Env, round: &SkyMap, px: &Px, front: i64, e: i64, out: &mut Out) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = px.len();
    let y = &px.p[1];
    let lf = &px.lvl;
    let role = par(sc, px, 0);
    let is = |o: &mut O, r: i64| -> V { let a = o.lt(K(r * 100 - 50), v(&role)); let b = o.lt(v(&role), K(r * 100 + 50)); o.mul(v(&a), v(&b), 0) };
    let stripe = is(o, 1);
    let skirt = is(o, 2);
    let winband = is(o, 3);
    let screen = is(o, 4);
    let lamp = is(o, 5);
    let nose = is(o, 6);
    let nf = kl(o, &[(front, Some(-1))]);
    let lz = o.lin(&[(v(&px.p[2]), None), (K(nf), None)]);
    let lz = o.clamp(v(&lz), 0, CARS * CAR - 1);
    let (car, cz) = o.divmod(v(&lz), K(250));
    let (car, czr) = { let (c2, r2) = o.divmod(v(&car), K(10)); let cz2 = o.lin(&[(v(&r2), Some(K(250))), (v(&cz), None)]); let _ = car; (c2, cz2) };
    let cz = czr;
    let first = o.lt(v(&car), K(1));
    let last = o.lt(K(CARS - 2), v(&car));
    let from_tail = o.lin(&[(v(&cz), Some(K(-1))), (K(CAR), None)]);
    let dnose = o.sel(&first, v(&cz), v(&from_tail));
    let dni = o.clamp(v(&dnose), 0, 2999);
    let sy = o.tab(tb.t("train_sy"), v(&dni));
    let b1 = band(o, tb, "band_ns1", lf, y, 0);
    let _ = b1;
    let yrel = o.lin(&[(v(y), None), (v(&sy), Some(K(-1)))]);
    let n1 = band(o, tb, "band_ns1", lf, &yrel, 0);
    let n2 = band(o, tb, "band_ns2", lf, &yrel, 30);
    let ns = o.sum(&[(v(&n1), 1), (v(&n2), 1)], 0);
    let ns = o.min(v(&ns), K(100));
    let ns = o.mul(v(&ns), v(&nose), 0);
    let white = [700, 700, 690];
    let blue = [20, 100, 420];
    let mut a = konst(n, white);
    a = std::array::from_fn(|c| o.sel(&stripe, K(blue[c]), v(&a[c])));
    a = std::array::from_fn(|c| o.sel(&skirt, K(white[c] * 55 / 100), v(&a[c])));
    a = mix3k(o, &a, blue, &ns);
    let low = o.lt(v(y), K(55));
    let lownose = o.mul(v(&low), v(&nose), 0);
    let dimk = o.sel(&lownose, K(60), K(100));
    a = scale3(o, &a, &dimk);
    // the canopy's crease along the nose's top
    let s30 = o.lt(K(321), v(&dnose));
    let s85 = o.lt(v(&dnose), K(910));
    let lx = { let b = bend_at(o, &sc.t, &px.p[2]); o.lin(&[(v(&px.p[0]), None), (v(&b), Some(K(-1))), (K(-X2), None)]) };
    let alx = { let m = o.lin(&[(v(&lx), Some(K(-1)))]); o.max(v(&lx), v(&m)) };
    let hwn = o.tab(tb.t("train_hwn"), v(&dni));
    let r = o.lin(&[(v(&alx), Some(K(1000)))]);
    let hwn1 = o.max(v(&hwn), K(1));
    let r = o.div(v(&r), v(&hwn1), 0);
    let r = o.clamp(v(&r), 0, 2999);
    let cr = o.tab(tb.t("train_crease"), v(&r));
    let cr = o.mul(v(&cr), v(&nose), 0);
    let cr = o.mul(v(&cr), v(&s30), 0);
    let cr = o.mul(v(&cr), v(&s85), 0);
    let ck = o.pct(v(&cr), K(-45));
    let ck = o.sum(&[(v(&ck), 1)], 100);
    a = scale3(o, &a, &ck);
    // the windows: rows of 1.05 m along each car's window band, not at its ends, the nose cars' back half only
    let (_, pw) = o.phase(v(&cz), 952, 5);
    let ws = o.tab2(tb.t("train_win"), v(lf), v(&pw));
    let e1 = o.lt(K(150), v(&cz));
    let e2 = o.lt(v(&cz), K(CAR - 150));
    let mid = o.mul(v(&e1), v(&e2), 0);
    let end = o.sum(&[(v(&first), 1), (v(&last), 1)], 0);
    let past = o.lt(K(1300), v(&dnose));
    let endok = o.mul(v(&end), v(&past), 0);
    let midcar = o.sum(&[(v(&end), -1)], 1);
    let ok = o.sum(&[(v(&midcar), 1), (v(&endok), 1)], 0);
    let ok = o.mul(v(&ok), v(&mid), 0);
    let ok = o.mul(v(&ok), v(&winband), 0);
    let win = o.mul(v(&ws), v(&ok), 0);
    let dark = o.sel(&screen, K(100), v(&win));
    a = mix3k(o, &a, [15, 20, 25], &dark);
    out.alb = a;
    let rs = refl_sky(o, tb, env, e, round, px, "schlick45", [999, 999, 999]);
    let grey = { let s = o.lin(&[(v(&rs[0]), None), (v(&rs[1]), None), (v(&rs[2]), None)]); let (g, _) = o.divmod(v(&s), K(3)); g };
    let wk = o.sum(&[(v(&win), 1)], 100);
    out.spec = std::array::from_fn(|c| {
        let ws_ = o.mix(v(&rs[c]), v(&grey), K(70));
        let ws_ = o.mul(v(&ws_), K(180), 3);
        let s = o.sel(&screen, v(&ws_), v(&rs[c]));
        let s = o.lin(&[(v(&s), Some(v(&wk)))]);
        let s = o.div(v(&s), K(100), 0);
        o.min(v(&s), K(999))
    });
    // the lamps: white at the front, red at the tail (24 and 18 bright, past a card, in tenths)
    for c in 0..3 {
        let fh = one(mulk(o, K([240, 228, 204][c]), K(e)));
        let th = one(mulk(o, K([180, 6, 3][c]), K(e)));
        let h = o.sel(&last, K(th), K(fh));
        out.hot[c] = o.sel(&lamp, v(&h), K(0));
        out.emit[c] = o.sel(&lamp, K(999), K(0));
    }
    // the low sun behind it: a rim of light, (d.sun)^3 (1 - |d.n|)^3 on what faces up
    let s0 = o.mul(v(&px.d[0]), K(env.sun[0]), 3);
    let s1 = o.mul(v(&px.d[1]), K(env.sun[1]), 3);
    let s2 = o.mul(v(&px.d[2]), K(env.sun[2]), 3);
    let near = o.lin(&[(v(&s0), None), (v(&s1), None), (v(&s2), None), (K(1000), None)]);
    let near = o.clamp(v(&near), 0, 2000);
    let back = o.tab(tb.t("pow3"), v(&near));
    let dn2 = dot_dn(o, &px.d, &px.n);
    let adn = { let m = o.lin(&[(v(&dn2), Some(K(-1)))]); let a = o.max(v(&dn2), v(&m)); o.min(v(&a), K(1000)) };
    let ol = o.sum(&[(v(&adn), -1)], 1000);
    let ol = o.clamp(v(&ol), 0, 1000);
    let rim = o.tab(tb.t("cube"), v(&ol));
    let ry = o.tabo(tb.t("train_rimy"), 127, v(&px.n[1]));
    let nd = o.sum(&[(v(&dark), -1)], 100);
    let k = o.mul(v(&back), v(&rim), 3);
    let k = o.pct(v(&k), v(&ry));
    let k = o.pct(v(&k), v(&nd));
    let k = o.mul(v(&k), K(350), 3);
    for c in 0..3 {
        let sc_ = env_k(o, env, env.sun_col[c], e);
        let rr = o.mul(v(&k), K(sc_), 3);
        let s = o.sum(&[(v(&out.emit[c]), 1), (v(&rr), 1)], 0);
        out.emit[c] = o.min(v(&s), K(999));
    }
}

/// Every pixel that isn't sky: its place, its material, its light, its fog. Returns the colour and the part past R
/// 999 (tenths), each channel, for the pixels given.
/// A footprint's level (quarter octaves) at the resolution drawn: 4 more at half resolution, at most 79.
pub fn lev_res<O: Ops>(o: &mut O, l: V) -> V { if res() == 1 { return l; } let t = o.sum(&[(v(&l), 1)], 4); o.min(v(&t), K(79)) }

pub fn shade_solid<O: Ops, S: Shading<O>>(o: &mut O, w: &World, cam: &Cam, env: &Env, e: i64, k: i64, front: Option<i64>, g: &GBuf, ray: &[V; 3], solid: &[usize], map: &SkyMap, round: &SkyMap, mirrors: Option<&Mirrors>, mirror_h: Option<i64>, fmap: Option<&SMap>, sh_: &mut S) -> ([V; 3], [V; 3]) {
    let (sc, tb) = (&w.sc, &w.tb);
    let n = solid.len();
    if n == 0 { return (zeros(0), zeros(0)); }
    let pick = |v: &V| -> V { solid.iter().map(|&p| v[p]).collect() };
    let iz = pick(&g.iz);
    let r: [V; 3] = std::array::from_fn(|i| pick(&ray[i]));
    if mirror_h.is_none() { o.stage("places"); }
    // the places: the eye plus the ray (0.7 of its length) times z / 0.7, by logarithms: ln |r| + ln (KZ / 0.7) / 1000 less
    // ln K / z (ten-thousandths), e back by the table of the ray's sign
    let liz = o.tab(tb.t("lniz4"), v(&iz));
    let mut p = places_l(o, tb, cam, &[&r[0], &r[1], &r[2]], &liz, true);
    // in a mirror: the place, the ray and the normal flipped back above the water
    if let Some(h) = mirror_h { p[1] = o.lin(&[(v(&p[1]), Some(K(-1))), (K(2 * h), None)]); }
    let b = bend_at(o, &sc.t, &p[2]);
    let xr = o.sum(&[(v(&p[0]), 1), (v(&b), -1)], 0);
    let qv = pick(&sh_.screen().unwrap_or(w.screen()).nlq7);
    let mut d = unit_rays(o, tb, &[&r[0], &r[1], &r[2]], &qv);
    let mut nrm: [V; 3] = std::array::from_fn(|i| pick(&g.n[i]));
    if mirror_h.is_some() { d[1] = o.sum(&[(v(&d[1]), -1)], 0); nrm[1] = o.sum(&[(v(&nrm[1]), -1)], 0); }
    let dn = dot_dn(o, &d, &nrm);
    let adn = { let m = o.lin(&[(v(&dn), Some(K(-1)))]); let a = o.max(v(&dn), v(&m)); o.min(v(&a), K(999)) };
    // the footprint: the pixel's width over the cosine, its level by logarithms
    let lic = o.tab(tb.t("ln_foot_ic"), v(&adn));
    // (a pixel twice as wide at half resolution: ln 2 more)
    let lic = if res() == 1 { lic } else { o.sum(&[(v(&lic), 1)], 6931) };
    let lf = o.ldiff(&lic, &liz, 0, 115);
    let lvl = o.tab(tb.t("lev_ln"), v(&lf));
    let mats: Vec<usize> = solid.iter().map(|&q| g.mat[q] as usize).collect();
    let px = Px { idx: (0..n).collect(), p, xr, d, n: nrm, liz, lvl, adn: adn.clone(), mat: mats.clone() };
    let mut alb = zeros(n);
    let mut spec = zeros(n);
    let mut emit = zeros(n);
    let mut hot = zeros(n);
    let mut nn = px.n.clone();
    let kinds: Vec<i64> = mats.iter().map(|&m| sc.mats[m].kind).collect();
    let in_tun = cam.z_cm > TUN0 - 500 && cam.z_cm < TUN1 + 1000;
    // the pixels gathered by kind on the card sorter, each kind's pixels in their order
    for at in sorter::groups(&kinds.iter().map(|&kd| [kd]).collect::<Vec<_>>(), &[sorter::CARD]) {
        let kd = kinds[at[0]];
        let sub = px.sub(&at);
        let m = at.len();
        let mut out = Out { alb: zeros(m), spec: zeros(m), emit: zeros(m), hot: zeros(m), n: sub.n.clone() };
        if mirror_h.is_none() { o.stage(match kd { kind::DECK => "deck", kind::PADDY => "paddy", kind::WATER => "water", kind::BUILD => "build", kind::TREE => "tree", kind::TRAIN => "train", kind::TUNNEL => "tunnel", kind::CHOUSE => "chouse", kind::CONC => "conc", kind::HILL => "hill", _ => "other" }); }
        match kd {
            kind::DECK => deck(o, tb, map, &sub, &mut out, in_tun),
            kind::PADDY => {
                let (wet, f, lean, mir) = paddy(o, tb, &sub, k, &mut out);
                let all = vec![true; m];
                let (mut sky, _) = sky_dir(o, tb, env, map, e, &mir.az, &mir.up, Some(&[&mir.r[0], &mir.r[1], &mir.r[2]]), Some(&all));
                if let Some(m) = mirrors.and_then(|m| m.paddy.as_ref()) {
                    let scr: Vec<usize> = sub.idx.iter().map(|&i| solid[i]).collect();
                    let izs: V = scr.iter().map(|&q| g.iz[q]).collect();
                    mirrored(o, w, cam, env, e, k, front, ray, map, round, m, &scr, &lean, 461, &izs, &mut sky, fmap, sh_);
                }
                for c in 0..3 { let s = o.mul(v(&sky[c]), v(&f), 3); out.spec[c] = o.pct(v(&s), v(&wet)); }
            }
            kind::WATER => {
                let (f, lean, mir) = water(o, tb, env, &sub, k, e, &mut out);
                let (mut sky, _) = sky_dir(o, tb, env, map, e, &mir.az, &mir.up, None, None);
                if let Some(m) = mirrors.and_then(|m| m.water.as_ref()) {
                    let scr: Vec<usize> = sub.idx.iter().map(|&i| solid[i]).collect();
                    let izs: V = scr.iter().map(|&q| g.iz[q]).collect();
                    mirrored(o, w, cam, env, e, k, front, ray, map, round, m, &scr, &lean, 230, &izs, &mut sky, fmap, sh_);
                }
                for c in 0..3 { out.spec[c] = o.mul(v(&sky[c]), v(&f), 3); }
            }
            kind::EMBANK => { out.alb = ground_tint(o, tb, &sub, "n_embank", "embank"); }
            kind::BANK => { out.alb = ground_tint(o, tb, &sub, "n_bank", "bank"); }
            kind::STREET => { out.alb = konst(m, [120, 120, 130]); }
            kind::BUILD => building(o, w, env, e, round, &sub, &mut out),
            _ => { simple(o, w, env, round, kd, &sub, front.unwrap_or(0), e, &mut out); sh_.kind(o, w, env, e, round, kd, &sub, &mut out); }
        }
        put3(&mut alb, &at, &out.alb);
        put3(&mut spec, &at, &out.spec);
        put3(&mut emit, &at, &out.emit);
        put3(&mut hot, &at, &out.hot);
        put3(&mut nn, &at, &out.n);
    }
    // the light: the sun (its shadows: shadows.rs), the sky's from above and the ground's from below
    if mirror_h.is_none() { o.stage("light"); }
    let (y, z) = (&px.p[1], &px.p[2]);
    let ke = |o: &mut O, k: i64| -> i64 { one(mulk(o, K(k), K(e))) };
    // in the tunnel: places past its mouth (when the camera is short of its far end), near the line, under its roof
    let mut tunnel = vec![0; n];
    if cam.z_cm < TUN1 + 50 {
        let t1 = o.lt(K(TUN0 - 50), v(z));
        let t2 = o.lt(v(z), K(TUN1 + 50));
        let dm = { let t = o.lin(&[(v(&px.xr), None), (K(-MID), None)]); let m = o.lin(&[(v(&t), Some(K(-1)))]); o.max(v(&t), v(&m)) };
        let t3 = o.lt(v(&dm), K(575));
        let t4 = o.lt(v(y), K(920));
        let t = o.sel(&t1, v(&t2), K(0));
        let t = o.sel(&t, v(&t3), K(0));
        tunnel = o.sel(&t, v(&t4), K(0));
    }
    // the sun's share by the normal: the dot with its way by tables of each part (worked out for the frame)
    let parts: V = (-127..=127).collect();
    let ndl = {
        let t: Vec<V> = (0..3).map(|i| { let tb_ = Tab { name: "sun_dot", data: o.muldiv(v(&parts), K(env.sun[i]), 99), w: 0 }; o.tabo(&tb_, 127, v(&nn[i])) }).collect();
        let s = o.sum(&[(v(&t[0]), 1), (v(&t[1]), 1), (v(&t[2]), 1)], 0);
        o.clamp(v(&s), 0, 999)
    };
    // the sun's shadows, a 2x2 block at a time (its first pixel), where the sun faces the surface (not in mirrors)
    let mut sh = vec![100; n];
    if mirror_h.is_none() && env.sun_col != [0, 0, 0] {
        let lo = o.lt(K(0), v(&ndl));
        let zok = if env.day { o.lt(v(z), K(TUN0 + 200)) } else { o.lt(K(TUN1 - 100), v(z)) };
        let mut rep: std::collections::BTreeMap<usize, usize> = Default::default();
        for i in 0..n { let q = solid[i]; let blk = (q / (2 * W() as usize)) * W() as usize + (q % W() as usize) / 2; rep.entry(blk).or_insert(i); }
        let reps: Vec<usize> = rep.values().copied().filter(|&i| lo[i] == 1 && zok[i] == 1 && tunnel[i] == 0).collect();
        if !reps.is_empty() {
            let g3 = |v: &V| -> V { reps.iter().map(|&i| v[i]).collect() };
            let pp = [g3(&px.p[0]), g3(&px.p[1]), g3(&px.p[2])];
            let nn3 = [g3(&nn[0]), g3(&nn[1]), g3(&nn[2])];
            let lit = if env.day {
                // behind the ride's start (the departure) by its own map
                let behind = o.lt(v(&pp[2]), K(-10_000));
                let mut lit = vec![100; reps.len()];
                for (map, want) in [(&w.sh.day, 0), (&w.sh.depart, 1)] {
                    let at: Vec<usize> = (0..reps.len()).filter(|&j| behind[j] == want).collect();
                    if at.is_empty() { continue; }
                    let g4 = |v_: &V| -> V { at.iter().map(|&j| v_[j]).collect() };
                    let l = sun_lit(o, tb, map, &[&g4(&pp[0]), &g4(&pp[1]), &g4(&pp[2])], &[&g4(&nn3[0]), &g4(&nn3[1]), &g4(&nn3[2])], 30, 35);
                    for (q, &j) in at.iter().enumerate() { lit[j] = l[q]; }
                }
                lit
            }
                else { sun_lit(o, tb, fmap.unwrap_or(&w.sh.dusk), &[&pp[0], &pp[1], &pp[2]], &[&nn3[0], &nn3[1], &nn3[2]], 25, 25) };
            let by: std::collections::HashMap<usize, i64> = reps.iter().zip(&lit).map(|(&i, &l)| { let q = solid[i]; ((q / (2 * W() as usize)) * W() as usize + (q % W() as usize) / 2, l) }).collect();
            for i in 0..n { let q = solid[i]; let blk = (q / (2 * W() as usize)) * W() as usize + (q % W() as usize) / 2; if let Some(&l) = by.get(&blk) { if lo[i] == 1 && zok[i] == 1 && tunnel[i] == 0 { sh[i] = l; } } }
        }
    }
    // the sun's part, shadowed where it is
    let shaded: Vec<usize> = (0..n).filter(|&i| sh[i] != 100).collect();
    let mut s1 = ndl.clone();
    if !shaded.is_empty() {
        let a: V = shaded.iter().map(|&i| ndl[i]).collect();
        let b: V = shaded.iter().map(|&i| sh[i]).collect();
        let r = o.pct(v(&a), v(&b));
        for (j, &i) in shaded.iter().enumerate() { s1[i] = r[j]; }
    }
    // the canyon's darkening of the sky's light on the city's walls, by height above the street
    let bs: Vec<usize> = (0..n).filter(|&i| env.day && (kinds[i] == kind::BUILD || kinds[i] == kind::SIGN)).collect();
    let canyon = if bs.is_empty() { vec![] } else {
        let yb: V = bs.iter().map(|&i| y[i]).collect();
        let yi = { let t = o.lin(&[(v(&yb), None), (K(-STREET), None)]); let t = o.div(v(&t), K(10), 0); o.clamp(v(&t), 0, 999) };
        o.tab(tb.t("canyon"), v(&yi))
    };
    // in the tunnel: no sun, the mouths' light leaking in (the lamps: tunnel.rs)
    let inside: Vec<usize> = (0..n).filter(|&i| tunnel[i] == 1).collect();
    let mut leak = vec![999; n];
    let mut lamps = zeros(n);
    let mut first_half = vec![0; n];
    if !inside.is_empty() {
        let zi: V = inside.iter().map(|&i| z[i]).collect();
        let a = o.lin(&[(v(&zi), None), (K(-TUN0), None)]);
        let am = o.lin(&[(v(&a), Some(K(-1)))]);
        let a = o.max(v(&a), v(&am));
        let b = o.lin(&[(v(&zi), Some(K(-1))), (K(TUN1), None)]);
        let bm = o.lin(&[(v(&b), Some(K(-1)))]);
        let b = o.max(v(&b), v(&bm));
        let dmouth = o.min(v(&a), v(&b));
        let di = { let t = o.div(v(&dmouth), K(10), 0); o.clamp(v(&t), 0, 999) };
        let lk = o.tab(tb.t("leak"), v(&di));
        let fh = o.lt(v(&zi), K((TUN0 + TUN1) / 2));
        let l = tunnel_lamps(o, w, cam, &px, &tunnel, e);
        for (j, &i) in inside.iter().enumerate() { leak[i] = lk[j]; first_half[i] = fh[j]; for c in 0..3 { lamps[c][i] = l[c][j]; } }
    }
    // the fog: by the distance along the ray, from the logarithms of the screen's factor and the depth
    let fogt = {
        let lq = pick(&sh_.screen().unwrap_or(w.screen()).lq);
        let lz = o.tab(tb.t("lniz"), v(&iz));
        let k = o.ldiff(&lq, &lz, 0, 12);
        o.tab(tb.t(if env.day { "fogl_day" } else { "fogl_dusk" }), v(&k))
    };
    let sunw = if env.day { vec![0; n] } else {
        let s0 = o.mul(v(&px.d[0]), K(env.sun[0]), 3);
        let s1_ = o.mul(v(&px.d[1]), K(env.sun[1]), 3);
        let s2 = o.mul(v(&px.d[2]), K(env.sun[2]), 3);
        let near = o.lin(&[(v(&s0), None), (v(&s1_), None), (v(&s2), None), (K(1000), None)]);
        let near = o.clamp(v(&near), 0, 2000);
        let p4 = o.tab(tb.t("pow4"), v(&near));
        let (p, _) = o.divmod(v(&p4), K(10));
        p
    };
    // what the frame adds (its lamps)
    let extra = sh_.light(o, w, &px, &nn, &kinds, &tunnel, solid);
    let mut col = zeros(n);
    let mut hot_out = zeros(n);
    let notdeck: Vec<bool> = kinds.iter().map(|&k| k != kind::DECK).collect();
    let fogged_px: Vec<usize> = (0..n).filter(|&i| tunnel[i] == 0).collect();
    let hemi = { let t = o.sum(&[(v(&parts), 1)], 99); let (h, _) = o.divmod(v(&t), K(2)); o.clamp(v(&h), 0, 100) };
    for c in 0..3 {
        // the frame's tables: the sun's part by its share; the sky's and the ground's by the normal's up part
        let (sc_, am, bo) = (env_k(o, env, env.sun_col[c], e), env_k(o, env, env.amb[c], e), env_k(o, env, env.bounce[c], e));
        let shares: V = (0..1000).collect();
        let ts = Tab { name: "sun_part", data: o.mul(v(&shares), K(sc_), 3), w: 0 };
        let ta = Tab { name: "amb_part", data: o.mix(K(bo), K(am), v(&hemi)), w: 0 };
        let sunp = o.tab(&ts, v(&s1));
        let mut amb = o.tabo(&ta, 127, v(&nn[1]));
        if !bs.is_empty() {
            let ab: V = bs.iter().map(|&i| amb[i]).collect();
            let r = o.pct(v(&ab), v(&canyon));
            for (j, &i) in bs.iter().enumerate() { amb[i] = r[j]; }
        }
        let light = match &extra { Some(x) => o.sum(&[(v(&sunp), 1), (v(&amb), 1), (v(&x[c]), 1)], 0), None => o.sum(&[(v(&sunp), 1), (v(&amb), 1)], 0) };
        let mut light = o.min(v(&light), K(999));
        let mut sp = spec[c].clone();
        if !inside.is_empty() {
            // the tunnel's: the lamps, the mouth's sky (the day's 0.8, the sunset's 1.6) times the leak, a little
            let (m1, m2) = (ke(o, DAY.amb[c] * 8 / 10), ke(o, DUSK.amb[c] * 16 / 10));
            let g = |v: &V| -> V { inside.iter().map(|&i| v[i]).collect() };
            let fh = g(&first_half);
            let ma = o.sel(&fh, K(m1), K(m2));
            let lk = g(&leak);
            let ml = o.mul(v(&ma), v(&lk), 3);
            let small = ke(o, 2);
            let tl = o.sum(&[(v(&g(&lamps[c])), 1), (v(&ml), 1)], small);
            let tl = o.min(v(&tl), K(999));
            let ins_nd: Vec<usize> = inside.iter().copied().filter(|&i| notdeck[i]).collect();
            let sd: V = ins_nd.iter().map(|&i| spec[c][i]).collect();
            let ld: V = ins_nd.iter().map(|&i| leak[i]).collect();
            let sl = o.mul(v(&sd), v(&ld), 3);
            for (j, &i) in inside.iter().enumerate() { light[i] = tl[j]; }
            for (j, &i) in ins_nd.iter().enumerate() { sp[i] = sl[j]; }
        }
        let a = o.mul(v(&alb[c]), v(&light), 3);
        let (cc, over) = o.light_sum(&a, &sp, &emit[c]);
        // the part past 999 in 25ths, to the tenths
        let mut hh = hot[c].clone();
        let past: Vec<usize> = (0..n).filter(|&i| over[i] > 0).collect();
        if !past.is_empty() {
            let ov: V = past.iter().map(|&i| over[i]).collect();
            let q = o.tab(tb.t("div25"), v(&ov));
            let h0: V = past.iter().map(|&i| hot[c][i]).collect();
            let t = o.sum(&[(v(&h0), 1), (v(&q), 1)], 0);
            let t = o.min(v(&t), K(999));
            for (j, &i) in past.iter().enumerate() { hh[i] = t[j]; }
        }
        col[c] = cc.clone();
        hot_out[c] = hh.clone();
        // the haze: by day its colour, at dusk pink away from the sun and the sunset's toward it (not in the tunnel)
        if fogged_px.is_empty() { continue; }
        let g = |v: &V| -> V { fogged_px.iter().map(|&i| v[i]).collect() };
        let (ccf, ft) = (g(&cc), g(&fogt));
        let hz = if env.day { vec![env_k(o, env, env.haze[c], e); fogged_px.len()] } else {
            let h0 = env_k(o, env, env.low[c], e);
            let h1 = env_k(o, env, env.haze[c], e);
            o.mix(K(h0), K(h1), v(&g(&sunw)))
        };
        let fogged = o.mix(v(&hz), v(&ccf), v(&ft));
        for (j, &i) in fogged_px.iter().enumerate() { col[c][i] = fogged[j]; }
        let warm: Vec<usize> = fogged_px.iter().copied().filter(|&i| hh[i] != 0).collect();
        if !warm.is_empty() {
            let hv: V = warm.iter().map(|&i| hh[i]).collect();
            let fv: V = warm.iter().map(|&i| fogt[i]).collect();
            let hf = o.pct(v(&hv), v(&fv));
            for (j, &i) in warm.iter().enumerate() { hot_out[c][i] = hf[j]; }
        }
    }
    (col, hot_out)
}

/// The tunnel's lamps and our headlights on what's in the tunnel (for those places only).
pub fn tunnel_lamps<O: Ops>(o: &mut O, w: &World, cam: &Cam, px: &Px, tunnel: &V, e: i64) -> [V; 3] {
    let at: Vec<usize> = (0..px.len()).filter(|&i| tunnel[i] == 1).collect();
    let g = |v: &V| -> V { at.iter().map(|&i| v[i]).collect() };
    tunnel_light(o, &w.tun, cam, e, &g(&px.xr), &g(&px.p[1]), &g(&px.p[2]))
}

/// What the water mirrors where its ripples point at something above it: the mirror's
/// pixel the ripple's lean moves to (amp pixels for a full lean), where it shows something farther than the water,
/// shaded as itself; else the sky as given.
pub fn mirrored<O: Ops>(o: &mut O, w: &World, cam: &Cam, env: &Env, e: i64, k: i64, front: Option<i64>, ray: &[V; 3], map: &SkyMap, round: &SkyMap,
                    m: &Mirror1, scr: &[usize], lean: &[V; 2], amp: i64, iz: &V, sky: &mut [V; 3], fmap: Option<&SMap>, sh_: &mut impl Shading<O>) {
    let xs: V = scr.iter().map(|&q| (q as i64) % W()).collect();
    let ys: V = scr.iter().map(|&q| (q as i64) / W()).collect();
    let ox = o.mul(v(&lean[0]), K(amp), 3);
    let oy = o.mul(v(&lean[1]), K(amp), 3);
    let sx = { let t = o.sum(&[(v(&xs), 1), (v(&ox), 1)], 0); o.clamp(v(&t), 0, W() - 1) };
    let sy = { let t = o.sum(&[(v(&ys), 1), (v(&oy), -1)], 0); o.clamp(v(&t), 0, H() - 1) };
    // the mirror's pixel there: the host's address of its row and column (the mirror's at every MS-th)
    let si: V = (0..sx.len()).map(|j| (sy[j] - sy[j] % MS as i64) * W() + sx[j] - sx[j] % MS as i64).collect();
    let miz: V = si.iter().map(|&s| m.g.iz[s as usize]).collect();
    let farther = o.lt(v(&miz), v(iz));
    let has: Vec<usize> = (0..scr.len()).filter(|&j| m.g.mat[si[j] as usize] >= 0 && farther[j] == 1).collect();
    if has.is_empty() { return; }
    let mut uniq: Vec<usize> = has.iter().map(|&j| si[j] as usize).collect();
    sorter::distinct(&mut uniq, &[sorter::up(7)], |&p| [p as i64]);
    let (c, _h) = shade_solid(o, w, cam, env, e, k, front, &m.g, ray, &uniq, map, round, None, Some(m.h), fmap, sh_);
    let at: std::collections::HashMap<usize, usize> = uniq.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    for &j in &has { let i = at[&(si[j] as usize)]; for ch in 0..3 { sky[ch][j] = c[ch][i]; } }
}
