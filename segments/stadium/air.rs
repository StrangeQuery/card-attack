// The dots of light: the city's street lights and windows, the airship's running lights and beacons, the phones
// held up, people's photo flashes, the confetti. Included after beams.rs.

/// The city's lights (fixed): their places in the camera's space.
pub fn city<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, cam: &Cam, sc: &Scene) {
    let cv = to_cam(o, t, cam, &sc.city);
    let size = vec![100; cv.len()];
    dots(o, t, f, &cv, &sc.city_c, &size);
}

/// The airship's running lights (warm, steady) and its red beacons (blinking: on for the first 35 of each 0.67 s).
pub fn blimp_lights<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, cam: &Cam, sc: &Scene, sh: &Show, bp: [i64; 3]) {
    let place = |o: &mut O, l: &[[i64; 3]]| -> Vec<[i64; 3]> {
        // the centre plus 1.01 times the light's place from it
        let w: Vec<[i64; 3]> = { let c: [V; 3] = std::array::from_fn(|i| { let x: V = l.iter().map(|p| p[i]).collect(); let x = o.lin(&[(v(&x), Some(K(101)))]); let x = o.div(v(&x), K(100), 0); o.sum(&[(v(&x), 1)], bp[i]) }); (0..l.len()).map(|k| [c[0][k], c[1][k], c[2][k]]).collect() };
        to_cam(o, t, cam, &w)
    };
    let cv = place(o, &sc.bl_lights);
    dots(o, t, f, &cv, &vec![[9000, 6750, 3600]; cv.len()], &vec![70; cv.len()]);
    // (t 1.5) mod 1 < 0.35: t12 mod 8 under 2.8
    let (_, r) = s_divmod(s_sum(&[(sh.t12, 1)], 80, o), 8, o);
    if s_lt(r, 3, o) {
        let cv = place(o, &sc.bl_beacons);
        dots(o, t, f, &cv, &vec![[30_000, 1000, 500]; cv.len()], &vec![150; cv.len()]);
    }
}

/// A phone's screen in each raised right hand (the people in view whose move is raising one): the hand's place in
/// the pose (across, up) on the person's width (its facing for the near, across the line to the eye for the
/// impostors), 15 cm nearer the eye; a cold light flickering, f 7 cm / z across.
pub fn phones<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, cam: &Cam, sc: &Scene, cr: &Crowd, sh: &Show) {
    let mut across: Vec<Option<[i64; 2]>> = vec![None; sc.cpos.len()];
    for s in &cr.sprites { let n = s_neg(s.to[1], o); across[s.person] = Some([n, s.to[0]]); }
    let ids: Vec<usize> = (0..sc.cpos.len()).filter(|&i| cr.vis[i] && cr.mv[i] == RAISE as u8).collect();
    if ids.is_empty() { return; }
    let k: Vec<usize> = ids.iter().map(|&i| pose_k(sc.cmodel[i], cr.mv[i], cr.ph[i])).collect();
    let hx: V = k.iter().map(|&k| sc.hand[k][0]).collect();
    let hy: V = k.iter().map(|&k| sc.hand[k][1]).collect();
    let sc_: V = ids.iter().map(|&i| sc.cscale[i]).collect();
    let rg: [V; 2] = std::array::from_fn(|c| ids.iter().map(|&i| across[i].unwrap_or(sc.cface[i])[c]).collect());
    let a = o.lin(&[(v(&hx), Some(v(&sc_)))]);
    let a = o.div(v(&a), K(1000), 0);
    let up = { let u = o.lin(&[(v(&hy), Some(v(&sc_)))]); o.div(v(&u), K(1000), 0) };
    let w: [V; 3] = std::array::from_fn(|c| {
        let base: V = ids.iter().map(|&i| sc.cpos[i][c]).collect();
        if c == 2 { return o.sum(&[(v(&base), 1), (v(&up), 1)], 0); }
        let d = o.lin(&[(v(&a), Some(v(&rg[c])))]);
        let d = o.div(v(&d), K(1000), 0);
        o.sum(&[(v(&base), 1), (v(&d), 1)], 0)
    });
    let pts: Vec<[i64; 3]> = (0..ids.len()).map(|j| [w[0][j], w[1][j], w[2][j]]).collect();
    let mut cv = to_cam(o, t, cam, &pts);
    let zz: V = cv.iter().map(|p| p[2]).collect();
    let zz = o.sum(&[(v(&zz), 1)], -15);
    for (j, p) in cv.iter_mut().enumerate() { p[2] = zz[j]; }
    // the flicker: 0.8 + 0.2 sin(id 1.7 + 5 t)
    let ph: V = ids.iter().map(|&i| sc.cphone[i]).collect();
    let ang = o.lin(&[(v(&ph), None), (K(sh.t12), Some(K(rate(5.0))))]);
    let ang = o.turns(v(&ang));
    let (sn, _) = o.sincos(v(&ang));
    let fl_ = { let a = o.lin(&[(v(&sn), Some(K(200)))]); let a = o.div(v(&a), K(1000), 0); o.sum(&[(v(&a), 1)], 800) };
    let lfl = o.tabo(&t.ln, 999_999, v(&fl_));
    let rgb: [V; 3] = std::array::from_fn(|c| { let k = [12_000.0, 13_600.0, 16_000.0][c]; let y = o.sum(&[(v(&lfl), 1)], lnf(k) - LN1000); expv(o, t, &y, None) });
    let rgbv: Vec<[i64; 3]> = (0..ids.len()).map(|j| [rgb[0][j], rgb[1][j], rgb[2][j]]).collect();
    let iz = iz_of(o, t, &cv);
    let size = dot_size(o, t, &iz, 7, 55, 400);
    dots(o, t, f, &cv, &rgbv, &size);
}

/// People taking pictures: 48 flashes a frame from anyone in the crowd, a frame each, a colour each (three random
/// numbers squared, over their largest, 0.3 + 0.7 of that, times 9), nothing to do with the beat. The randomness
/// is CARDIAC's own: an LCG (x 421 + 54,773 mod 10^6) seeded by the frame.
pub fn flashes<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, cam: &Cam, sc: &Scene, sh: &Show) {
    let mut x = { let a = s_lin(&[(sh.t12, Some(7919)), (100_131, None)], o); frac1m(a, o) };
    let mut draw = |o: &mut O| -> i64 { x = frac1m(s_lin(&[(x, Some(421)), (54_773, None)], o), o); x };
    let mut ids = vec![];
    let mut h: [V; 3] = std::array::from_fn(|_| vec![]);
    for _ in 0..48 {
        let r = draw(o);
        let id = s_div(s_lin(&[(s_div(r, 1000, o), Some(65_536))], o), 1000, o);
        ids.push(id as usize);
        for c in 0..3 { let u = s_div(draw(o), 1000, o); h[c].push(s_div(s_lin(&[(u, Some(u))], o), 1000, o)); }
    }
    // the colour: (0.3 + 0.7 h / max h) 9, ten-thousandths
    let m = { let a = o.max(v(&h[0]), v(&h[1])); o.max(v(&a), v(&h[2])) };
    let m = o.max(v(&m), K(1));
    let lm = o.tabo(&t.ln, 999_999, v(&m));
    let rgb: [V; 3] = std::array::from_fn(|c| {
        let lh = o.tabo(&t.ln, 999_999, v(&h[c]));
        let y = o.sum(&[(v(&lh), 1), (v(&lm), -1)], lnf(63_000.0));
        let a = expv(o, t, &y, None);
        o.sum(&[(v(&a), 1)], 27_000)
    });
    // at the head (1.8 m times the size), 30 cm nearer the eye
    let up: V = ids.iter().map(|&i| sc.cscale[i]).collect();
    let up = { let a = o.lin(&[(v(&up), Some(K(180)))]); o.div(v(&a), K(1000), 0) };
    let pts: Vec<[i64; 3]> = { let z: V = ids.iter().map(|&i| sc.cpos[i][2]).collect(); let z = o.sum(&[(v(&z), 1), (v(&up), 1)], 0); ids.iter().enumerate().map(|(j, &i)| [sc.cpos[i][0], sc.cpos[i][1], z[j]]).collect() };
    let mut cv = to_cam(o, t, cam, &pts);
    let zz: V = cv.iter().map(|p| p[2]).collect();
    let zz = o.sum(&[(v(&zz), 1)], -30);
    for (j, p) in cv.iter_mut().enumerate() { p[2] = zz[j]; }
    let iz = iz_of(o, t, &cv);
    let size = dot_size(o, t, &iz, 35, 100, 700);
    let rgbv: Vec<[i64; 3]> = (0..ids.len()).map(|j| [rgb[0][j], rgb[1][j], rgb[2][j]]).collect();
    dots(o, t, f, &cv, &rgbv, &size);
}

/// The confetti: 9,000 pieces falling from 55 m (fired just before the crest: in the air at the reveal), drifting,
/// flickering as they turn.
pub fn confetti<O: Ops>(o: &mut O, t: &Tables, f: &mut Frame, cam: &Cam, sc: &Scene, sh: &Show) {
    if s_lt(sh.t12, 33, o) { return; }
    // since: t - 1.2 s, milliseconds
    let since = s_sum(&[(s_div(s_lin(&[(sh.t12, Some(1000))], o), 12, o), 1)], -1200, o);
    let s240 = s_div(s_lin(&[(since, Some(240))], o), 1000, o);
    let n = sc.conf.len();
    let col = |k: usize| -> V { sc.conf.iter().map(|c| c[k]).collect() };
    let (px, py, z0, ph, fall) = (col(0), col(1), col(2), col(3), col(4));
    // the height: 55 m less (z0 + since fall 2.4) mod 55 m, plus 2 m
    let drop = { let a = o.lin(&[(v(&fall), Some(K(s240)))]); o.div(v(&a), K(1000), 0) };
    let zz = o.sum(&[(v(&z0), 1), (v(&drop), 1)], 0);
    let q = o.div(v(&zz), K(5500), 0);
    let r = o.lin(&[(v(&zz), None), (v(&q), Some(K(-5500)))]);
    let z = o.sum(&[(v(&r), -1)], 5700);
    let ang = |o: &mut O, rate_ms: i64, k: i64| -> V { let a = o.lin(&[(v(&ph), Some(K(k))), (K(since), Some(K(rate_ms)))]); o.turns(v(&a)) };
    let (sx, _) = { let a = ang(o, 207, 1); o.sincos(v(&a)) };
    let (_, cy) = { let a = ang(o, 175, 1); o.sincos(v(&a)) };
    let (sf, _) = { let a = ang(o, 1432, 3); o.sincos(v(&a)) };
    let x = { let a = o.lin(&[(v(&sx), Some(K(150)))]); let a = o.div(v(&a), K(1000), 0); o.sum(&[(v(&px), 1), (v(&a), 1)], 0) };
    let y = { let a = o.lin(&[(v(&cy), Some(K(100)))]); let a = o.div(v(&a), K(1000), 0); o.sum(&[(v(&py), 1), (v(&a), 1)], 0) };
    let pts: Vec<[i64; 3]> = (0..n).map(|j| [x[j], y[j], z[j]]).collect();
    let cv = to_cam(o, t, cam, &pts);
    // the flicker: 0.5 + 0.5 sin^2, times 1.1
    let flick = o.tabo(&t.flick, 999, v(&sf));
    let lfl = o.tabo(&t.ln, 999_999, v(&flick));
    let rgb: [V; 3] = std::array::from_fn(|c| { let cc: V = sc.conf_c.iter().map(|q| q[c]).collect(); let lc = o.tabo(&t.ln, 999_999, v(&cc)); let y = o.sum(&[(v(&lc), 1), (v(&lfl), 1)], lnf(1.1) - LN1000); expv(o, t, &y, None) });
    let rgbv: Vec<[i64; 3]> = (0..n).map(|j| [rgb[0][j], rgb[1][j], rgb[2][j]]).collect();
    let iz = iz_of(o, t, &cv);
    let size = dot_size(o, t, &iz, 5, 50, 300);
    dots(o, t, f, &cv, &rgbv, &size);
}

impl Tables {
    /// 0.5 + 0.5 s^2 for s -999..999 (999ths): thousandths
    pub fn flick_tab() -> Tab { tab("flick", (-999..=999).map(|s: i64| ((0.5 + 0.5 * (s as f64 / 999.0).powi(2)) * 1000.0).round() as i64).collect()) }
}
