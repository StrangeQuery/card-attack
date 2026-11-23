// The sky: a function of direction only (the prototype's sky_day and sky_dusk), worked out each frame on a map of
// directions (azimuth by the ray's height part, a step of 5 thousandths each way), and looked up between its
// entries for every ray that sees it: the sky itself, the water's and the glass's mirrors. What is too sharp for the
// map (the setting sun's core, the far ridges' outline) is worked out at each pixel. Colours in R: 250 for 1.0, the
// frame's exposure in. Included after frame.rs.

/// The printed tables (data/tables.bin), by name.
pub struct Tb(pub std::collections::BTreeMap<String, Tab>);

impl Tb {
    pub fn read(path: &str) -> Tb {
        let b = Bin::read(path);
        let mut m = std::collections::BTreeMap::new();
        for (name, (shape, data)) in b.0 {
            let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
            m.insert(name, Tab { name: leaked, data, w: shape[1] });
        }
        Tb(m)
    }
    pub fn t(&self, name: &str) -> &Tab { self.0.get(name).unwrap_or_else(|| panic!("no table {name}")) }
}

pub const MAP_W: i64 = 561;
pub const MAP_H: i64 = 151;
pub const ROUND_W: i64 = 127;
pub const ROUND_H: i64 = 21;

/// The day's and the dusk's light, R (1.0 is 250), before the exposure.
/// `low`: the dusk's haze away from the sun; `night`: the evening going into night (the night arrival's frames, which
/// make their own Env each frame: the sun going down, the light and the haze with it).
/// `unit`: the light's colours (sun_col, amb, bounce, haze, low) are in R over this (1 for the ride; the night's, dim,
/// in tenths: 10).
/// `win`: how dark it is for the windows (the prototype's NIGHT, thousandths: lit ones come on with it), and
/// `win_k` what their light is times (thousandths: the night's beat).
pub struct Env { pub day: bool, pub sun: [i64; 3], pub sun_col: [i64; 3], pub amb: [i64; 3], pub bounce: [i64; 3], pub haze: [i64; 3], pub fog: &'static str, pub low: [i64; 3], pub night: Option<SkyNight>, pub unit: i64, pub win: i64, pub win_k: i64 }

/// One of the Env's colours exposed (e thousandths), in R, at most 999.
pub fn env_k<O: Ops>(o: &mut O, env: &Env, k: i64, e: i64) -> i64 {
    if env.unit == 1 { return one(mulk(o, K(k), K(e))); }
    let t = o.lin(&[(K(k), Some(K(e)))]);
    let t = o.div(v(&t), K(1000), 0);
    let t = o.div(v(&t), K(env.unit), 0);
    one(o.min(v(&t), K(999)))
}

pub const DAY: Env = Env { day: true, sun: [-302, 864, -402], sun_col: [575, 550, 500], amb: [60, 75, 105], bounce: [50, 45, 38], haze: [175, 200, 238], fog: "fog_day", low: [0, 0, 0], night: None, unit: 1, win: 0, win_k: 1000 };
pub const DUSK: Env = Env { day: false, sun: [-312, 95, 945], sun_col: [650, 263, 88], amb: [35, 28, 43], bounce: [20, 13, 10], haze: [275, 120, 55], fog: "fog_dusk", low: [105, 65, 100], night: None, unit: 1, win: 0, win_k: 1000 };

/// The evening going into night, over the dusk's sky (the night arrival's sky_night): the dusk's colour dimmed
/// (thousandths) with the twilight's blue and the town's glow low down added (tables by the way's height part, from
/// -1000, R exposed), then toward the night's violet (by the height part) by `bk` percent; and the stadium's glow in
/// the haze round its way `c` (thousandths), by the squared distance of the way from it over 100 (millionths, a
/// table for each channel, R exposed).
/// `hotk`: what the part past 999 is left (percent: the dimming and the violet's share taken off).
pub struct SkyNight { pub dim: i64, pub bk: i64, pub hotk: i64, pub add: [Tab; 3], pub viol: [Tab; 3], pub sg: [Tab; 3], pub c: [i64; 3] }

/// The night's falling over sky colours at directions (height parts dy; the unit ways, when known, for the
/// stadium's glow). The part past 999 dimmed with the rest.
pub fn night_falls<O: Ops>(o: &mut O, tb: &Tb, nt: &SkyNight, dy: &[i64], dn: Option<&[&[i64]; 3]>, col: &mut [V; 3], hot: &mut [V; 3]) {
    let n = dy.len();
    if n == 0 { return; }
    let d2 = dn.map(|d| {
        let sq: Vec<V> = (0..3).map(|i| { let t = o.sum(&[(A::V(d[i]), 1)], -nt.c[i]); o.tabo(tb.t("sq2000"), 2000, v(&t)) }).collect();
        let s = o.lin(&[(v(&sq[0]), None), (v(&sq[1]), None), (v(&sq[2]), None)]);
        let (q, _) = o.divmod(v(&s), K(100));
        o.min(v(&q), K(29_999))
    });
    for c in 0..3 {
        let mut a = if nt.dim < 1000 { o.mul(v(&col[c]), K(nt.dim), 3) } else { col[c].clone() };
        let t = o.tabo(&nt.add[c], 1000, A::V(dy));
        a = o.sum(&[(v(&a), 1), (v(&t), 1)], 0);
        a = o.min(v(&a), K(999));
        if nt.bk > 0 { let vi = o.tabo(&nt.viol[c], 1000, A::V(dy)); a = o.mix(v(&a), v(&vi), K(nt.bk)); }
        if let Some(d2) = &d2 { let g = o.tab(&nt.sg[c], v(d2)); a = o.sum(&[(v(&a), 1), (v(&g), 1)], 0); a = o.min(v(&a), K(999)); }
        col[c] = a;
        if (nt.dim < 1000 || nt.bk > 0) && hot[c].iter().any(|&h| h != 0) { hot[c] = o.pct(v(&hot[c]), K(nt.hotk)); }
    }
}

/// A sky map: each entry's colour (R, exposed), and how many columns.
pub struct SkyMap { pub c: [Tab; 3], pub w: i64, pub az0: i64, pub el0: i64, pub step: i64 }

/// What a sky map's entries keep from the start (for the day or the dusk): the cloud plane's place along each ray
/// (metres from the eye), the cloud tables' level, the colour the clouds lie over (each channel, before the exposure)
/// and the entry's factors the clouds take (by day the low sky's cover and the far's; at dusk the band's density, the
/// sun's gold and the high light).
pub struct SkyConst { pub hx: V, pub hz: V, pub lvl: V, pub base: [V; 3], pub a: V, pub b: V, pub c: V }

/// A map's entries' constant parts, at the start: the gradient, the sun's glow, the cloud plane.
pub fn sky_consts<O: Ops>(o: &mut O, tb: &Tb, env: &Env, d: &[&[i64]; 3], az: &[i64]) -> SkyConst {
    let n = d[0].len();
    let (dx, dy, dz) = (d[0], d[1], d[2]);
    // the sun's way: the dot (thousandths) as a table index from -1000
    let s0 = o.mul(A::V(dx), K(env.sun[0]), 3);
    let s1 = o.mul(A::V(dy), K(env.sun[1]), 3);
    let s2 = o.mul(A::V(dz), K(env.sun[2]), 3);
    let near = o.lin(&[(v(&s0), None), (v(&s1), None), (v(&s2), None), (K(1000), None)]);
    let near = o.clamp(v(&near), 0, 2000);
    let dyc = o.clamp(A::V(dy), 1, 999);
    // the cloud plane's place along each ray (metres from the eye)
    let k = o.tab(tb.t(if env.day { "cloudk_day" } else { "cloudk_dusk" }), v(&dyc));
    let hx = o.lin(&[(A::V(dx), Some(v(&k)))]);
    let hx = o.div(v(&hx), K(1000), 0);
    let hz = o.lin(&[(A::V(dz), Some(v(&k)))]);
    let hz = o.div(v(&hz), K(1000), 0);
    let lvl = o.tab(tb.t("cloud_level"), v(&dyc));
    if env.day {
        let up = o.clamp(A::V(dy), 0, 999);
        let t = o.tab(tb.t("sqrt"), v(&up));
        let (t, _) = o.divmod(v(&t), K(10));
        const C1: [i64; 3] = [224, 247, 282];
        const C2: [i64; 3] = [50, 100, 220];
        let base = std::array::from_fn(|c| {
            let g = o.tab(tb.t(&format!("glowday{c}")), v(&near));
            let b = o.mix(K(C1[c]), K(C2[c]), v(&t));
            let b = o.lin(&[(v(&b), None), (v(&g), None)]);
            o.clamp(v(&b), 0, 999)
        });
        let low = o.tab(tb.t("smooth_low_day"), v(&dyc));
        let far = o.tab(tb.t("smooth_far_day"), v(&dyc));
        let (far, _) = o.divmod(v(&far), K(10));
        SkyConst { hx, hz, lvl, base, a: low, b: far, c: vec![0; n] }
    } else {
        const L0: [i64; 3] = [105, 65, 105];
        const L1: [i64; 3] = [338, 125, 40];
        const M0: [i64; 3] = [75, 35, 75];
        const M1: [i64; 3] = [138, 40, 75];
        const HI: [i64; 3] = [13, 13, 55];
        let ei = o.sum(&[(A::V(dy), 1)], 50);
        let ei = o.clamp(v(&ei), 0, 1049);
        let lm = o.tab(tb.t("dusk_lowmid"), v(&ei));
        let mh = o.tab(tb.t("dusk_midhigh"), v(&ei));
        let islow = o.tab(tb.t("dusk_islow"), v(&ei));
        let azi = o.sum(&[(A::V(az), 1)], 3150);
        let w = o.tab(tb.t("dusk_warm"), v(&azi));
        let base = std::array::from_fn(|c| {
            let lo = o.mix(K(L0[c]), K(L1[c]), v(&w));
            let mi = o.mix(K(M0[c]), K(M1[c]), v(&w));
            let a = o.mix(v(&lo), v(&mi), v(&lm));
            let b = o.mix(v(&mi), K(HI[c]), v(&mh));
            let base = o.sel(&islow, v(&a), v(&b));
            let g = o.tab(tb.t(&format!("glowdusk{c}")), v(&near));
            let base = o.lin(&[(v(&base), None), (v(&g), None)]);
            o.clamp(v(&base), 0, 999)
        });
        let band = o.tab(tb.t("dusk_band"), v(&dyc));
        let p5 = o.tab(tb.t("pow5"), v(&near));
        let hl = o.tab(tb.t("smooth_hl"), v(&dyc));
        let hk = o.mul(v(&hl), K(600), 3);
        let hk = o.sum(&[(v(&hk), 1)], 600);
        SkyConst { hx, hz, lvl, base, a: band, b: p5, c: hk }
    }
}

/// The colour of the sky at a map's entries this frame: the clouds where the eye is over the colour beneath, times
/// the exposure (thousandths; by a table for the frame).
pub fn sky_model<O: Ops>(o: &mut O, tb: &Tb, env: &Env, sc: &SkyConst, eye_cm: [i64; 3], e: i64) -> [V; 3] {
    let n = sc.hx.len();
    // (the eye's place over the cloud plane, metres, on the machines)
    let (ex, ez) = (one(o.div(K(eye_cm[0]), K(100), 0)), one(o.div(K(eye_cm[2]), K(100), 0)));
    let px = o.sum(&[(v(&sc.hx), 1)], ex);
    let pz = o.sum(&[(v(&sc.hz), 1)], ez);
    // the cloud tables' texels (10 m) and how far between (tenths made percent); the next texel's the host's
    let (qx, rx) = o.divmod(v(&px), K(10));
    let (qz, rz) = o.divmod(v(&pz), K(10));
    let ix = o.wrap(v(&qx), 1);
    let iz = o.wrap(v(&qz), 1);
    let ix1: V = ix.iter().map(|&i| (i + 1) % 1000).collect();
    let iz1: V = iz.iter().map(|&i| (i + 1) % 1000).collect();
    let fx = o.tab(tb.t("x10"), v(&rx));
    let fz = o.tab(tb.t("x10"), v(&rz));
    let lvl = &sc.lvl;
    let bil = |o: &mut O, name: &str| -> V {
        // the host picks each entry's table by its level
        let mut out = vec![0; n];
        for l in 0..3 {
            let pick: Vec<usize> = (0..n).filter(|&i| lvl[i] == l).collect();
            if pick.is_empty() { continue; }
            let g = |v: &V| -> V { pick.iter().map(|&i| v[i]).collect() };
            let t = tb.t(&format!("{name}{l}"));
            let (a, b, c, d) = (o.tab2(t, v(&g(&iz)), v(&g(&ix))), o.tab2(t, v(&g(&iz)), v(&g(&ix1))), o.tab2(t, v(&g(&iz1)), v(&g(&ix))), o.tab2(t, v(&g(&iz1)), v(&g(&ix1))));
            let (fx, fz) = (g(&fx), g(&fz));
            let top = o.mix(v(&a), v(&b), v(&fx));
            let bot = o.mix(v(&c), v(&d), v(&fx));
            let r = o.mix(v(&top), v(&bot), v(&fz));
            for (j, &i) in pick.iter().enumerate() { out[i] = r[j]; }
        }
        out
    };
    // the exposure, by a table for the frame
    let all: V = (0..1000).collect();
    let te = { let t = mulk(o, v(&all), K(e)); Tab { name: "exposed", data: o.clamp(v(&t), 0, 999), w: 0 } };
    let pct: V = (0..=100).collect();
    if env.day {
        const CC0: [i64; 3] = [125, 140, 170];
        const CC1: [i64; 3] = [388, 388, 375];
        let cl = bil(o, "cloud_day");
        let lit = bil(o, "cloudlit_day");
        let cov = o.tab(tb.t("smooth_cover_day"), v(&cl));
        let cover = o.mul(v(&cov), v(&sc.a), 4);
        let (litp, _) = o.divmod(v(&lit), K(10));
        std::array::from_fn(|c| {
            let tc = Tab { name: "cloud_c", data: o.mix(K(CC0[c]), K(CC1[c]), v(&pct)), w: 0 };
            let cc = o.tab(&tc, v(&litp));
            let cc = o.mix(v(&sc.base[c]), v(&cc), v(&sc.b));
            let r = o.mix(v(&sc.base[c]), v(&cc), v(&cover));
            o.tab(&te, v(&r))
        })
    } else {
        const B0: [i64; 3] = [40, 18, 38];
        const B1: [i64; 3] = [325, 105, 105];
        const G: [i64; 3] = [900, 400, 125];
        let cl = bil(o, "cloud_dusk");
        let dens = o.tab(tb.t("smooth_dens"), v(&cl));
        let dens = o.mul(v(&dens), v(&sc.a), 4);
        let thin = o.tab(tb.t("dusk_thin"), v(&cl));
        let (thinp, _) = o.divmod(v(&thin), K(10));
        // gold: near^5 (0.2 + 1.2 thin)
        let tg = { let t = o.mul(v(&pct), K(12), 0); Tab { name: "gold_k", data: o.sum(&[(v(&t), 1)], 200), w: 0 } };
        let gk = o.tab(&tg, v(&thinp));
        let gold = mulk(o, v(&sc.b), v(&gk));
        std::array::from_fn(|c| {
            let tb_ = Tab { name: "cloud_b", data: o.mix(K(B0[c]), K(B1[c]), v(&pct)), w: 0 };
            let cb = o.tab(&tb_, v(&thinp));
            let cb = mulk(o, v(&cb), v(&sc.c));
            let tgc = Tab { name: "gold_c", data: o.mul(v(&all), K(G[c]), 3), w: 0 };
            let gc = o.tab(&tgc, v(&gold));
            let cc = o.sum(&[(v(&cb), 1), (v(&gc), 1)], 0);
            let cc = o.clamp(v(&cc), 0, 999);
            let r = o.mix(v(&sc.base[c]), v(&cc), v(&dens));
            o.tab(&te, v(&r))
        })
    }
}

/// The maps' constant parts, for the day and the dusk: the map ahead and the coarse all-round one.
pub struct SkyConsts { pub day: [SkyConst; 2], pub dusk: [SkyConst; 2] }

pub fn sky_consts_all<O: Ops>(o: &mut O, tb: &Tb) -> SkyConsts {
    let mk = |o: &mut O, env: &Env, name: &str| -> SkyConst {
        let d = [&tb.t(&format!("{name}_dx")).data[..], &tb.t(&format!("{name}_dy")).data[..], &tb.t(&format!("{name}_dz")).data[..]];
        sky_consts(o, tb, env, &d, &tb.t(&format!("{name}_az")).data)
    };
    SkyConsts { day: [mk(o, &DAY, "map"), mk(o, &DAY, "round")], dusk: [mk(o, &DUSK, "map"), mk(o, &DUSK, "round")] }
}

/// This frame's sky maps: the one ahead and the coarse all-round one.
pub fn sky_maps<O: Ops>(o: &mut O, tb: &Tb, env: &Env, k: &SkyConsts, cam: &Cam, e: i64) -> (SkyMap, SkyMap) {
    let cs = if env.day { &k.day } else { &k.dusk };
    let mk = |o: &mut O, sc: &SkyConst, w: i64, az0: i64, el0: i64, step: i64| -> SkyMap {
        let [r, g, b] = sky_model(o, tb, env, sc, cam.eye_cm, e);
        SkyMap { c: [Tab { name: "skymap", data: r, w: w as usize }, Tab { name: "skymap", data: g, w: w as usize }, Tab { name: "skymap", data: b, w: w as usize }], w, az0, el0, step }
    };
    (mk(o, &cs[0], MAP_W, -1400, -50, 5), mk(o, &cs[1], ROUND_W, -3150, 0, 50))
}

/// Azimuths of directions (thousandths of a radian, from straight ahead along the line, -3142..3141): the atan of the
/// smaller part over the larger, turned to its quarter.
pub fn azimuth<O: Ops>(o: &mut O, tb: &Tb, dx: &[i64], dz: &[i64]) -> V {
    let ax = { let m = o.sum(&[(A::V(dx), -1)], 0); o.max(A::V(dx), v(&m)) };
    let az_ = { let m = o.sum(&[(A::V(dz), -1)], 0); o.max(A::V(dz), v(&m)) };
    // the larger part's reciprocal, the smaller times it
    let xbig = o.lt(v(&az_), v(&ax));
    let big = o.sel(&xbig, v(&ax), v(&az_));
    let small = o.sel(&xbig, v(&az_), v(&ax));
    let big = o.clamp(v(&big), 1, 999);
    let r = o.tab(tb.t("recip_ahead"), v(&big));
    let q = o.lin(&[(v(&small), Some(v(&r)))]);
    let q = o.div(v(&q), K(1000), 0);
    let q = o.clamp(v(&q), 0, 1000);
    let qi = o.lin(&[(v(&q), None), (K(4000), None)]);
    let a = o.tab(tb.t("atan"), v(&qi));
    // a in 0..785; from the z axis: a when z is the larger, else 1571 - a; then the signs
    let b = o.sum(&[(v(&a), -1)], 1571);
    let a = o.sel(&xbig, v(&b), v(&a));
    let zneg = o.lt(A::V(dz), K(0));
    let back = o.lin(&[(v(&a), Some(K(-1))), (K(3142), None)]);
    let a = o.sel(&zneg, v(&back), v(&a));
    let xneg = o.lt(A::V(dx), K(0));
    let m = o.lin(&[(v(&a), Some(K(-1)))]);
    o.sel(&xneg, v(&m), v(&a))
}

/// The sky's colour for directions (thousandths, unit), between a map's entries.
pub fn sky_look<O: Ops>(o: &mut O, tb: &Tb, map: &SkyMap, az: &[i64], dy: &[i64]) -> [V; 3] {
    let (mw, mh) = (map.w, map.c[0].data.len() as i64 / map.w);
    // twice the place from the map's corner, so its entry and the fraction come by moving digits down (the steps are
    // 5 or 50): over 10 the tenths made percent by the printed table, over 100 already percent
    let a = o.sum(&[(A::V(az), 2)], -2 * map.az0);
    let a = o.clamp(v(&a), 0, 2 * (mw - 1) * map.step - 1);
    let e = o.sum(&[(A::V(dy), 2)], -2 * map.el0);
    let e = o.clamp(v(&e), 0, 2 * (mh - 1) * map.step - 1);
    assert!(map.step == 5 || map.step == 50);
    let (ci, cr) = o.divmod(v(&a), K(2 * map.step));
    let (ri, rr) = o.divmod(v(&e), K(2 * map.step));
    let (fx, fy) = if map.step == 5 { (o.tab(tb.t("x10"), v(&cr)), o.tab(tb.t("x10"), v(&rr))) } else { (cr, rr) };
    // the next entries: the host's
    let ci1: V = ci.iter().map(|&i| i + 1).collect();
    let ri1: V = ri.iter().map(|&i| i + 1).collect();
    std::array::from_fn(|c| {
        let t = &map.c[c];
        let (p, q, r, s) = (o.tab2(t, v(&ri), v(&ci)), o.tab2(t, v(&ri), v(&ci1)), o.tab2(t, v(&ri1), v(&ci)), o.tab2(t, v(&ri1), v(&ci1)));
        let top = o.mix(v(&p), v(&q), v(&fx));
        let bot = o.mix(v(&r), v(&s), v(&fx));
        o.mix(v(&top), v(&bot), v(&fy))
    })
}

/// The unit way of each pixel's ray (thousandths): the ray (0.7 of its length, from rays()) times the screen's
/// factor 2 q7 / 1000 (q / 1.4, from the start), by logarithms: ln |r| less the screen's ln (500 / q7) (the printed
/// tables), e back by the part's sign, held to cards.
pub fn unit_rays<O: Ops>(o: &mut O, tb: &Tb, ray: &[&[i64]; 3], nlq7: &[i64]) -> [V; 3] {
    std::array::from_fn(|i| {
        let l = o.tabo(tb.t("lnr0"), 999, A::V(ray[i]));
        let k = o.ldiff(&l, &nlq7.to_vec(), 0, 139);
        let neg = o.tabo(tb.t("neg"), 999, A::V(ray[i]));
        let mut r = vec![0; k.len()];
        for (name, want) in [("exp4c", 0), ("exp4cn", 1)] {
            let at: Vec<usize> = (0..k.len()).filter(|&j| neg[j] == want).collect();
            let kk: V = at.iter().map(|&j| k[j]).collect();
            let e = o.tab(tb.t(name), v(&kk));
            for (q, &j) in at.iter().enumerate() { r[j] = e[q]; }
        }
        r
    })
}

/// The sky in directions given by azimuth and height part (and, for the sun's core, the unit way; `core_at` which
/// of them may be near the sun).
pub fn sky_dir<O: Ops>(o: &mut O, tb: &Tb, env: &Env, map: &SkyMap, e: i64, az: &[i64], dy: &[i64], dn: Option<&[&[i64]; 3]>, sun_box: Option<&[bool]>) -> ([V; 3], [V; 3]) {
    let n = az.len();
    let mut col = sky_look(o, tb, map, az, dy);
    let mut hot: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    if env.day { return (col, hot); }
    let lowf = o.lt(A::V(dy), K(75));
    let low: Vec<usize> = (0..n).filter(|&i| lowf[i] == 1).collect();
    ridges(o, tb, e, &low, az, dy, &mut col);
    if let (Some(bx), Some(dn)) = (sun_box, dn) {
        let at: Vec<usize> = (0..n).filter(|&i| bx[i]).collect();
        sun_core(o, tb, env, e, &at, dn, &mut col, &mut hot);
    }
    if let Some(nt) = &env.night { night_falls(o, tb, nt, dy, dn, &mut col, &mut hot); }
    (col, hot)
}

/// The far ridges at dusk (Fuji, the hills, a tree line, by azimuth), over the sky's colour, at the directions picked
/// (those whose height part is under 75 thousandths): where the ray's height part is under theirs (ten-thousandths).
/// Returns which each picked direction sees.
pub fn ridges<O: Ops>(o: &mut O, tb: &Tb, e: i64, low: &[usize], az: &[i64], dy: &[i64], col: &mut [V; 3]) -> V {
    if low.is_empty() { return vec![]; }
    let g = |v: &[i64]| -> V { low.iter().map(|&i| v[i]).collect() };
    let azl = g(az);
    let ai = o.sum(&[(v(&azl), 1)], 3150);
    let ai = o.clamp(v(&ai), 0, 6300);
    let el = g(dy);
    let el10 = o.lin(&[(v(&el), Some(K(10)))]);
    let el10 = o.max(v(&el10), K(-1));
    let fuji = o.tab(tb.t("ridge_fuji"), v(&ai));
    let hills = o.tab(tb.t("ridge_hills"), v(&ai));
    let trees = o.tab(tb.t("ridge_trees"), v(&ai));
    let snow = o.tab(tb.t("ridge_snow"), v(&ai));
    let lit = o.tab(tb.t("ridge_lit"), v(&ai));
    let on_f = o.lt(v(&el10), v(&fuji));
    let on_h = o.lt(v(&el10), v(&hills));
    let on_t = o.lt(v(&el10), v(&trees));
    let snowy = o.lt(v(&snow), v(&el10));
    const ROCK0: [i64; 3] = [50, 35, 75];
    const ROCK1: [i64; 3] = [138, 70, 80];
    const SNOW0: [i64; 3] = [138, 105, 175];
    const SNOW1: [i64; 3] = [450, 213, 188];
    const HILL: [i64; 3] = [55, 33, 63];
    const TREE: [i64; 3] = [18, 13, 20];
    for c in 0..3 {
        let haze = g(&col[c]);
        let ke = |o: &mut O, k: i64| -> i64 { one(mulk(o, K(k), K(e))) };
        let (r0, r1, s0, s1, h0, t0) = (ke(o, ROCK0[c]), ke(o, ROCK1[c]), ke(o, SNOW0[c]), ke(o, SNOW1[c]), ke(o, HILL[c]), ke(o, TREE[c]));
        let rock = o.mix(K(r0), K(r1), v(&lit));
        let sn = o.mix(K(s0), K(s1), v(&lit));
        let fc = o.sel(&snowy, v(&sn), v(&rock));
        let fc = o.mix(v(&fc), v(&haze), K(45));
        let hc = o.mix(K(h0), v(&haze), K(40));
        let tc = o.mix(K(t0), v(&haze), K(25));
        let r = o.sel(&on_f, v(&fc), v(&haze));
        let r = o.sel(&on_h, v(&hc), v(&r));
        let r = o.sel(&on_t, v(&tc), v(&r));
        for (j, &i) in low.iter().enumerate() { col[c][i] = r[j]; }
    }
    // which is seen: 0 the sky, 1 Fuji's rock, 2 its snow, 3 the hills, 4 the trees
    let f = { let s = o.sel(&snowy, K(2), K(1)); o.sel(&on_f, v(&s), K(0)) };
    let f = o.sel(&on_h, K(3), v(&f));
    o.sel(&on_t, K(4), v(&f))
}

/// The sun's core over the sky's colour at the directions picked (near the sun): by the squared distance of the ray
/// from the sun's way (each part's square from the printed table, held to 20,000); the part past 999 to the tenths.
/// Only where the distance is under the table's reach (elsewhere the core adds nothing).
pub fn sun_core<O: Ops>(o: &mut O, tb: &Tb, env: &Env, e: i64, at: &[usize], dn: &[&[i64]; 3], col: &mut [V; 3], hot: &mut [V; 3]) {
    if at.is_empty() { return; }
    let g = |v: &[i64]| -> V { at.iter().map(|&i| v[i]).collect() };
    let mut d2 = vec![];
    for i in 0..3 {
        let d = g(dn[i]);
        let d = o.sum(&[(v(&d), 1)], -env.sun[i]);
        let d = o.clamp(v(&d), -999, 999);
        d2.push(o.tabo(tb.t("sq999c"), 999, v(&d)));
    }
    let s = o.sum(&[(v(&d2[0]), 1), (v(&d2[1]), 1), (v(&d2[2]), 1)], 0);
    let near = o.lt(v(&s), K(19_999));
    let close: Vec<usize> = (0..at.len()).filter(|&j| near[j] == 1).collect();
    if close.is_empty() { return; }
    let s: V = close.iter().map(|&j| s[j]).collect();
    // the exposure by tables for the frame
    let all: V = (0..1000).collect();
    let te = Tab { name: "core_e", data: mulk(o, v(&all), K(e)), w: 0 };
    for c in 0..3 {
        let core = o.tab(tb.t(&format!("core{c}")), v(&s));
        let ch = o.tab(tb.t(&format!("corehot{c}")), v(&s));
        let core = o.tab(&te, v(&core));
        let ch = o.tab(&te, v(&ch));
        let base: V = close.iter().map(|&j| col[c][at[j]]).collect();
        let zero = vec![0; close.len()];
        let (cc, over) = o.light_sum(&base, &core, &zero);
        let over10 = o.tab(tb.t("div25"), v(&over));
        let tot_hot = o.sum(&[(v(&ch), 1), (v(&over10), 1)], 0);
        let tot_hot = o.min(v(&tot_hot), K(999));
        for (q, &j) in close.iter().enumerate() { col[c][at[j]] = cc[q]; hot[c][at[j]] = tot_hot[q]; }
    }
}
