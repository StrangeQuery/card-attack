// A frame of the stadium party: the camera and the show, what each pixel sees, its light, the lights in the air,
// the glow and the tone curve. Included last.

pub struct World { pub sc: Scene, pub t: Tables, pub lt: LightTabs, pub scr: ScreenConsts, pub scr_half: ScreenConsts, pub bt: BeamTabs, pub lz: LaserTabs, pub hr: HalfRays }

/// Each pixel's 1 / |ray| (thousandths, the ray (x, y, f) / f), worked out once: the squared distance from the
/// middle on CARDIAC (half pixels squared over 4; at the half size, as the full size's: squared, not quartered),
/// the printed table of 1000 / sqrt(1 + s / f^2). And each row's place, 2 y + 1 - H (half pixels from the middle).
pub struct ScreenConsts { pub inv: V, pub rows: V }

impl World {
    /// The screen's numbers at the size being drawn.
    pub fn scr_now(&self) -> &ScreenConsts { if is_full() { &self.scr } else { &self.scr_half } }
}

fn screen_consts<O: Ops>(o: &mut O, t: &Tables) -> ScreenConsts {
    let xs: V = (0..scr_w()).map(|x| 2 * x + 1 - scr_w()).collect();
    let ys: V = (0..scr_h()).map(|y| 2 * y + 1 - scr_h()).collect();
    let x2 = o.lin(&[(v(&xs), Some(v(&xs)))]);
    let x2 = if is_full() { divk(o, &x2, 4) } else { x2 };
    let y2 = o.lin(&[(v(&ys), Some(v(&ys)))]);
    let y2 = if is_full() { divk(o, &y2, 4) } else { y2 };
    let mut s = vec![0; npix()];
    for y in 0..scr_h() as usize {
        let r = o.sum(&[(v(&x2), 1)], y2[y]);
        s[y * scr_w() as usize..(y + 1) * scr_w() as usize].copy_from_slice(&r);
    }
    ScreenConsts { inv: o.tab(&t.inv_ray, v(&s)), rows: ys }
}

pub fn load<O: Ops>(o: &mut O, dir: &str) -> World {
    let sc = read_scene(dir);
    let t = tables(dir);
    let lt = light_tabs();
    set_res(true);
    let scr = screen_consts(o, &t);
    let hr = half_rays(o, &t);
    set_res(false);
    let scr_half = screen_consts(o, &t);
    set_res(true);
    World { sc, t, lt, scr, scr_half, bt: beam_tabs(), lz: laser_tabs(), hr }
}

impl Tables {
    /// 1000 / sqrt(1 + s / 10^6) for s 0..800,000 (a pixel's squared distance from the middle)
    pub fn inv_ray_tab() -> Tab { tab("inv_ray", (0..800_000).map(|s| (1000.0 / (1.0 + s as f64 / 1e6).sqrt()).round() as i64).collect()) }
    /// the sky's colour by the ray's height part (thousandths, -1000..1000): the prototype's night sky (with the
    /// city's glow and the ground) and the dark violet it deepens to, ten-thousandths, [r, g, b]
    pub fn sky_tabs() -> [[Tab; 3]; 2] {
        let f = |el: f64, violet: bool| -> [f64; 3] {
            let hor = (-el.max(0.0) * 9.0).exp();
            if violet { return [0.012 * (1.0 - hor) + 0.050 * hor, 0.004 * (1.0 - hor) + 0.018 * hor, 0.026 * (1.0 - hor) + 0.060 * hor]; }
            if el < 0.0 { return [0.004, 0.004, 0.005]; }
            let g = (-el.max(0.0) * 30.0).exp();
            [0.004 * (1.0 - hor) + 0.05 * hor + 0.06 * g, 0.006 * (1.0 - hor) + 0.03 * hor + 0.03 * g, 0.016 * (1.0 - hor) + 0.035 * hor + 0.01 * g]
        };
        std::array::from_fn(|k| std::array::from_fn(|c| tab("sky", (-1000..=1000).map(|e| (f(e as f64 / 1000.0, k == 1)[c] * 10_000.0).round() as i64).collect())))
    }
}

/// The night arrival's camera for the dissolve's frames (its frames F_HAND - 44 .. F_HAND - 1, our 0..43): the eye and
/// the point looked at in our frame (centimetres), as it publishes them.
/// (With the frames of the night's video they are for, when its file numbers them.)
pub struct NightCams(pub Vec<NightCam>, pub Vec<i64>);

/// A camera of the night's: its axes as published, or (the test file) an eye and a point looked at.
pub enum NightCam { Axes(Cam), Look([i64; 3], [i64; 3]) }

/// What the frame needs from outside: the night arrival's frames and camera for the dissolve.
/// `hd`: the party version (the world at half size, the subjects at the film's: party.rs), else the full one.
pub struct Inputs<'a> { pub night: Option<&'a [u8]>, pub night_cam: &'a NightCams, pub hd: bool }

/// The frame k of the segment (0..FRAMES): RGB bytes, 1536 x 768.
pub fn render<O: Ops>(o: &mut O, w: &World, inp: &Inputs, k: i64) -> Vec<u8> {
    o.stage("camera");
    // the show's clock: the frame less 8, on CARDIAC
    let t12 = s_sum(&[(k, 1)], -8, o);
    if t12 >= CUT_T12 { return vec![0; npix() * 3]; }
    let (sc, t, lt) = (&w.sc, &w.t, &w.lt);
    let cam = if k < DISSOLVE {
        match &inp.night_cam.0[k as usize] {
            NightCam::Axes(c) => *c,
            NightCam::Look(e, g) => { let d: [i64; 3] = std::array::from_fn(|i| s_sum(&[(g[i], 1), (e[i], -1)], 0, o)); look(o, *e, d) }
        }
    } else { camera(o, t, t12) };
    let sh = show(o, t, t12);
    set_light_hd(inp.hd);
    let img = party(o, w, inp, &cam, &sh, inp.hd);
    set_res(true);
    if k < DISSOLVE { return dissolve(o, t, &img, inp.night.expect("the night's frames"), k); }
    img
}

/// Our picture blended with the night arrival's own frame: theirs (1 - a) + ours a, a = smooth(k / 44)^0.6 (its
/// render's mix), thousandths, on CARDIAC (the mix's printed table of a by k).
fn dissolve<O: Ops>(o: &mut O, t: &Tables, ours: &[u8], night: &[u8], k: i64) -> Vec<u8> {
    let a = s_tab(&t.dissolve, s_sum(&[(k, 1)], 0, o), o);
    let n0 = (k as usize) * npix() * 3;
    let theirs: V = night[n0..n0 + npix() * 3].iter().map(|&x| x as i64).collect();
    let mine: V = ours.iter().map(|&x| x as i64).collect();
    // theirs + (ours - theirs) a / 1000, rounded down (the prototype's float then the byte's truncation)
    let d = o.sum(&[(v(&mine), 1), (v(&theirs), -1)], 0);
    let p = o.lin(&[(v(&d), Some(K(a)))]);
    let p = o.div(v(&p), K(1000), 0);
    let r = o.sum(&[(v(&theirs), 1), (v(&p), 1)], 0);
    r.into_iter().map(|x| x as u8).collect()
}

impl Tables {
    /// the dissolve's share of ours at its frame k (0..43): smooth(F_HAND - 44, F_HAND, f)^0.6, thousandths
    pub fn dissolve_tab() -> Tab {
        tab("dissolve", (0..DISSOLVE).map(|k| { let x = k as f64 / DISSOLVE as f64; let s = x * x * (3.0 - 2.0 * x); (s.powf(0.6) * 1000.0).round() as i64 }).collect())
    }
}

/// Which list a piece came from.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Src_ { Surf, Emit, Skin, Screen, Sprite, Cloth, Pole, Ball, Ship, Person }

/// The party at a camera and a moment: linear light into the tone curve, RGB bytes.
/// (`hd`: the party version's world, the crowd's impostors, the stadium, the balloons and the airship's hull, drawn
/// at half size; the screens, the flags, the airship's screen and the near dancers at the film's; then party.rs.)
pub fn party<O: Ops>(o: &mut O, w: &World, inp: &Inputs, cam: &Cam, sh: &Show, hd: bool) -> Vec<u8> {
    let (sc, t, lt) = (&w.sc, &w.t, &w.lt);
    let t12 = sh.t12;
    // ---- the screens' pictures and the light they give --------------------------------------------------------
    o.stage("screens");
    let nb = sc.q_kind.iter().zip(&sc.q_aux).filter(|(&k, _)| k == 2).map(|(_, &a)| a).max().map_or(0, |m| m as usize + 1);
    let shows = shows(o, sc, sh, nb.max(64));
    if std::env::var("STADIUM_SHOWS").is_ok() { eprintln!("shows {}: main {:?} leds {:?} airship {:?} banners {:?}", t12, shows.main, shows.leds, shows.airship, shows.banners); return vec![0; npix() * 3]; }
    let main_lvl = if shows.main == Src::Cardiac { s_div(s_lin(&[(sh.screen, Some(12))], o), 10, o) } else { sh.screen };
    let led_lvl = s_sum(&[(s_div(s_lin(&[(sh.pulse, Some(6))], o), 10, o), 1)], 1000, o);
    // the parodies' stand-ins on the LED walls, every texel (their light is the pictures' mean)
    let mut sd = Sd::default();
    {
        let ask = Sd::ask();
        for i in 0..2 { for y in 0..LED.1 { for x in 0..LED.0 { led_texel(sc, &ask, shows.leds[i], x, y); } } }
        standins(o, sc, t12, &ask.asked(), &mut sd);
    }
    let main_means = means(o, t, MAIN.0, MAIN.1, 64, 64, &|x, y| main_texel(sc, &sd, shows.main, x, y));
    let screen_means: Vec<[i64; 3]> = main_means.iter().map(|m| m.map(|c| { let a = s_lin(&[(c, Some(main_lvl))], o); let a = s_lin(&[(a, Some(6))], o); s_div(a, 10_000, o) })).collect();
    let led_means: Vec<[i64; 3]> = (0..2).map(|i| {
        let m = means(o, t, LED.0, LED.1, LED.0, LED.1, &|x, y| led_texel(sc, &sd, shows.leds[i], x, y))[0];
        m.map(|c| { let a = s_lin(&[(c, Some(led_lvl))], o); let a = s_lin(&[(a, Some(3))], o); s_div(a, 10_000, o) })
    }).collect();
    // ---- the beams' ways, the lamps, the light at the probes -----------------------------------------------------
    o.stage("light");
    let blimp = blimp_at(o, sc, t12);
    let spot_d: Vec<[i64; 3]> = sc.spots.iter().enumerate().map(|(j, l)| spot_dir(o, sh, l, j)).collect();
    let search_d: Vec<[i64; 3]> = sc.search.iter().map(|l| search_dir(o, t, sc, sh, l, blimp)).collect();
    let lamps_ = lamps(o, sc, sh, &spot_d, &search_d, &screen_means, &led_means, false);
    if std::env::var("STADIUM_DEBUG").is_ok() { eprintln!("eye {:?} b {:?}\nblimp {blimp:?}\nspots {spot_d:?}\nsearch {search_d:?}", cam.eye, cam.b); }
    // ---- the crowd ---------------------------------------------------------------------------------------------
    o.stage("crowd");
    if hd { set_res(false); }
    let cr = crowd(o, t, lt, sc, cam, t12);
    // ---- the stadium: its surfaces lit at their corners, the windows, the skin; the screens ----------------------
    o.stage("stadium");
    let st = &sc.stadium;
    let cv = to_cam(o, t, cam, &st.v);
    let kind_of = |ti: usize| sc.mat_kind[sc.st_mat[ti]];
    let surf: Vec<usize> = (0..st.t.len()).filter(|&ti| kind_of(ti) == 0).collect();
    let emit: Vec<usize> = (0..st.t.len()).filter(|&ti| kind_of(ti) == 1).collect();
    // the corners of the surfaces: their light by their probes (only those in front, roughly: any of the corner's
    // triangles' corners ahead of the near plane), times the material's colour
    let mut need = vec![false; st.v.len()];
    for &ti in &surf { if st.t[ti].iter().any(|&u| cv[u][2] > 0) { for &u in &st.t[ti] { need[u] = true; } } }
    let corners: Vec<usize> = (0..st.v.len()).filter(|&u| need[u]).collect();
    // the light at the probes something in view takes it from: the stadium's corners, the impostors, the flags
    o.stage("light");
    let pcube = {
        let mut want = vec![false; sc.probes().len()];
        for &u in &corners { want[sc.st_vprobe()[u]] = true; }
        for spr in &cr.sprites { want[sc.cprobe()[spr.person]] = true; }
        for &i in &sc.flag_pid { want[sc.cprobe()[i]] = true; }
        let ids: Vec<usize> = (0..sc.probes().len()).filter(|&k| want[k]).collect();
        let c = cubes(o, t, lt, &ids.iter().map(|&k| sc.probes()[k]).collect::<Vec<_>>(), &lamps_);
        let mut full: Cube = std::array::from_fn(|_| std::array::from_fn(|_| vec![0; sc.probes().len()]));
        for f in 0..6 { for ch in 0..3 { for (q, &k) in ids.iter().enumerate() { full[f][ch][k] = c[f][ch][q]; } } }
        full
    };
    o.stage("stadium");
    let probe: Vec<usize> = corners.iter().map(|&u| sc.st_vprobe()[u]).collect();
    let nrm: Vec<[i64; 3]> = corners.iter().map(|&u| st.n[u]).collect();
    let lc = cube_light(o, t, &pcube, &probe, &nrm);
    // each surface triangle's corners carry its colour: a L + a 0.004 (the corner's light, the triangle's albedo)
    let mut vals: Vec<Vec<i64>> = vec![vec![0; 3]; 3 * surf.len()];
    let mut vpos: Vec<[i64; 3]> = vec![[0; 3]; 3 * surf.len()];
    let mut tris = vec![];
    let at: std::collections::HashMap<usize, usize> = corners.iter().enumerate().map(|(i, &u)| (u, i)).collect();
    {
        let mut ln_l: [V; 3] = std::array::from_fn(|_| vec![]);
        for c in 0..3 { ln_l[c] = lnv(o, t, &lc[c]).0; }
        let mut items: Vec<(usize, usize, usize)> = vec![];        // (slot, corner index, material)
        for (j, &ti) in surf.iter().enumerate() {
            for q in 0..3 {
                let u = st.t[ti][q];
                vpos[3 * j + q] = cv[u];
                if let Some(&ci) = at.get(&u) { items.push((3 * j + q, ci, sc.st_mat[ti])); }
            }
            tris.push([3 * j, 3 * j + 1, 3 * j + 2]);
        }
        for c in 0..3 {
            let ll: V = items.iter().map(|it| ln_l[c][it.1]).collect();
            let la: V = items.iter().map(|it| t_ln(t, sc.mat_alb[it.2][c])).collect();
            let a4: V = items.iter().map(|it| sc.mat_alb[it.2][c]).collect();
            let y = o.sum(&[(v(&ll), 1), (v(&la), 1)], -LN10K);
            let e = expv(o, t, &y, None);
            let g = divk(o, &a4, 250);
            let col = o.sum(&[(v(&e), 1), (v(&g), 1)], 0);
            for (q, it) in items.iter().enumerate() { vals[it.0][c] = col[q]; }
        }
    }
    let surf_ids: Vec<u32> = surf.iter().map(|&ti| ti as u32).collect();
    let p_surf = draw(o, t, &vpos, &vals, &tris, &surf_ids, true);
    let emit_tris: Vec<[usize; 3]> = emit.iter().map(|&ti| st.t[ti]).collect();
    let emit_ids: Vec<u32> = emit.iter().map(|&ti| ti as u32).collect();
    let p_emit = draw(o, t, &cv, &vec![vec![]; cv.len()], &emit_tris, &emit_ids, false);
    // the skin: each corner's angle round the bowl and its height
    o.stage("skin");
    let skc = to_cam(o, t, cam, &sc.sk_v);
    let skv: Vec<Vec<i64>> = (0..sc.sk_v.len()).map(|i| vec![sc.sk_ang[i / 3][i % 3], sc.sk_v[i][2]]).collect();
    let skt: Vec<[usize; 3]> = (0..sc.sk_v.len() / 3).map(|j| [3 * j, 3 * j + 1, 3 * j + 2]).collect();
    let skid: Vec<u32> = (0..skt.len() as u32).collect();
    let p_skin = draw(o, t, &skc, &skv, &skt, &skid, true);
    // the screens: each corner's place in its picture (thousandths of a texel)
    o.stage("scrquads");
    set_res(true);
    let qc = to_cam(o, t, cam, &sc.q_v);
    let mut qv: Vec<Vec<i64>> = vec![vec![0, 0]; 3 * sc.q_t.len()];
    let mut qp = vec![[0; 3]; 3 * sc.q_t.len()];
    let qt: Vec<[usize; 3]> = (0..sc.q_t.len()).map(|j| [3 * j, 3 * j + 1, 3 * j + 2]).collect();
    for (j, tr) in sc.q_t.iter().enumerate() { for q in 0..3 { qp[3 * j + q] = qc[tr[q]]; qv[3 * j + q] = vec![sc.q_uv[j][2 * q], sc.q_uv[j][2 * q + 1]]; } }
    let qid: Vec<u32> = (0..qt.len() as u32).collect();
    let p_scr = draw(o, t, &qp, &qv, &qt, &qid, false);
    // ---- the flags, the balloons, the airship, the near dancers -----------------------------------------------
    o.stage("things");
    o.stage("flags");
    let (cloth, poles) = flags(o, t, sc, cam, &pcube, t12);
    o.stage("balloons");
    if hd { set_res(false); }
    let mut bal = balloons(o, t, lt, sc, cam, &lamps_, t12);
    {
        // the balloons' quads cut to their discs before the depth test: u^2 + v^2 under 1
        let pp = &bal.d.p;
        let u = o.clamp(v(&pp.a[0]), -999, 999);
        let w2 = o.clamp(v(&pp.a[1]), -999, 999);
        let u2 = o.tabo(&t.sq999, 999, v(&u));
        let v2 = o.tabo(&t.sq999, 999, v(&w2));
        let rr = o.sum(&[(v(&u2), 1), (v(&v2), 1)], 0);
        let inside = o.lt(v(&rr), K(1000));
        let keep: Vec<usize> = (0..pp.len()).filter(|&k| inside[k] == 1).collect();
        let mut np = Pieces::new(2);
        for &k in &keep { np.px.push(pp.px[k]); np.iz.push(pp.iz[k]); np.id.push(pp.id[k]); np.a[0].push(pp.a[0][k]); np.a[1].push(pp.a[1][k]); }
        bal.d.p = np;
    }
    o.stage("airship");
    let ship_lamps = lamps(o, sc, sh, &spot_d, &search_d, &screen_means, &led_means, true);
    let (ship, ship_screen) = airship(o, t, lt, sc, cam, &ship_lamps, blimp, hd, sh.t12 >= WHITE_T12);
    let empty = Pieces::new(5);
    let ship_p = ship.as_ref().map_or(&empty, |d| &d.p);
    o.stage("near");
    set_res(true);
    let near = near_people(o, t, lt, sc, cam, &cr, &lamps_);
    if hd {
        let lists = Lists { surf: &p_surf, emit: &p_emit, skin: &p_skin, scr: &p_scr, cr: &cr, cloth: &cloth, poles: &poles, bal: &bal, ship: ship.as_ref(), near: &near };
        let now = ScreensNow { shows, main_lvl, led_lvl };
        return party_hd(o, w, cam, sh, &lists, ship_screen.as_ref(), &now, &pcube, &mut sd, blimp, &spot_d, &search_d);
    }
    // ---- what each pixel sees: the nearest piece (CARDIAC's nearest of each pixel's) ------------------------------
    o.stage("visible");
    let srcs: [(&V, &Vec<u32>, Src_); 10] = [(&p_surf.iz, &p_surf.px, Src_::Surf), (&p_emit.iz, &p_emit.px, Src_::Emit), (&p_skin.iz, &p_skin.px, Src_::Skin), (&p_scr.iz, &p_scr.px, Src_::Screen), (&cr.pieces.iz, &cr.pieces.px, Src_::Sprite),
        (&cloth.p.iz, &cloth.p.px, Src_::Cloth), (&poles.p.iz, &poles.p.px, Src_::Pole), (&bal.d.p.iz, &bal.d.p.px, Src_::Ball), (&ship_p.iz, &ship_p.px, Src_::Ship), (&near.d.p.iz, &near.d.p.px, Src_::Person)];
    let (win_src, win_idx, depth) = resolve(o, &srcs);
    if std::env::var("STADIUM_PROBE").is_ok() { for &(x, y) in &[(768usize, 700usize), (400, 650), (1000, 720)] { let p = y * scr_w() as usize + x; let mut l = vec![]; for (iz, px, k) in &srcs { for (i, &q) in px.iter().enumerate() { if q as usize == p { l.push((*k, iz[i])); } } } eprintln!("pixel {x},{y}: {:?} won {:?} {}", l, win_src[p], depth[p]); } }
    if std::env::var("STADIUM_DEBUG").is_ok() { for (iz, _, k) in &srcs { eprintln!("pieces {k:?}: {}", iz.len()); } eprintln!("stadium tris {} near people {}", sc.stadium.t.len(), near.people.len()); }
    // ---- the shading -------------------------------------------------------------------------------------------
    o.stage("shade");
    let mut col: [V; 3] = sky(o, w, cam, t12);
    stars(o, w, cam, &win_src, &mut col);
    o.stage("sh_surf");
    let lists = Lists { surf: &p_surf, emit: &p_emit, skin: &p_skin, scr: &p_scr, cr: &cr, cloth: &cloth, poles: &poles, bal: &bal, ship: ship.as_ref(), near: &near };
    let now = ScreensNow { shows, main_lvl, led_lvl };
    shade_pieces(o, w, cam, sh, &lists, &now, &pcube, &mut sd, &win_src, &win_idx, &mut col);
    // ---- the lights in the air ---------------------------------------------------------------------------------
    let mut fr = Frame { col, iz: depth, hit: vec![] };
    o.stage("dots");
    city(o, t, &mut fr, cam, sc);
    blimp_lights(o, t, &mut fr, cam, sc, sh, blimp);
    phones(o, t, &mut fr, cam, sc, &cr, sh);
    flashes(o, t, &mut fr, cam, sc, sh);
    confetti(o, t, &mut fr, cam, sc, sh);
    o.stage("beams");
    let bl = beam_list(o, sc, sh, &spot_d, &search_d);
    let izm = quad_max(o, &fr.iz);
    let (acc, touched, _) = volumetric(o, t, &w.bt, &w.lz, cam, &bl, &izm, None);
    add_beams(o, t, &mut fr, &acc, &touched);
    strobe_haze(o, t, &mut fr, sh.strobe);
    o.stage("lasers");
    lasers(o, t, &w.lz, &mut fr, cam, sc, sh, 0);
    o.stage("pyro");
    pyro(o, t, &w.lz, &mut fr, cam, sc, sh, &mut vec![]);
    o.stage("bloom");
    bloom(o, t, &mut fr);
    o.stage("dstars");
    // Dope's stars: crisp, out of the glow (light adds, so behind the beams all the same)
    dope_stars(o, w, cam, &win_src, &mut fr.col);
    // ---- the tone curve ------------------------------------------------------------------------------------------
    o.stage("tone");
    tone(o, t, &fr.col)
}

/// The lists of pieces drawn this frame (each kind's), for the shading.
pub struct Lists<'a> { pub surf: &'a Pieces, pub emit: &'a Pieces, pub skin: &'a Pieces, pub scr: &'a Pieces, pub cr: &'a Crowd, pub cloth: &'a Drawn, pub poles: &'a Drawn, pub bal: &'a Balloons, pub ship: Option<&'a Drawn>, pub near: &'a Near }

/// The screens this frame: what each shows, the giant screen's and LED walls' levels.
pub struct ScreensNow { pub shows: Shows, pub main_lvl: i64, pub led_lvl: i64 }

/// Each pixel's colour from the piece it sees (`win_src`, `win_idx`: the pixels with none are left as they are):
/// the surfaces' walked colour, the emitters, the skin, the screens' texels, the impostors, the flags, the poles,
/// the balloons, the airship, the near dancers.
pub fn shade_pieces<O: Ops>(o: &mut O, w: &World, cam: &Cam, sh: &Show, l: &Lists, now: &ScreensNow, pcube: &Cube, sd: &mut Sd, win_src: &[Option<Src_>], win_idx: &[usize], col: &mut [V; 3]) {
    let (sc, t, lt) = (&w.sc, &w.t, &w.lt);
    let t12 = sh.t12;
    let (p_surf, p_emit, p_skin, p_scr, cr, cloth, poles, bal, ship, near) = (l.surf, l.emit, l.skin, l.scr, l.cr, l.cloth, l.poles, l.bal, l.ship, l.near);
    let (shows, main_lvl, led_lvl) = (&now.shows, now.main_lvl, now.led_lvl);
    let pix_of = |s: Src_| -> Vec<usize> { (0..win_src.len()).filter(|&p| win_src[p] == Some(s)).collect() };
    // the surfaces: their walked colour
    for p in pix_of(Src_::Surf) { for c in 0..3 { col[c][p] = p_surf.a[c][win_idx[p]]; } }
    for p in pix_of(Src_::Emit) { let ti = p_emit.id[win_idx[p]] as usize; for c in 0..3 { col[c][p] = sc.mat_emit[sc.st_mat[ti]][c]; } }
    let skp = pix_of(Src_::Skin);
    let skc_ = skin(o, t, sc, sh, &skp.iter().map(|&p| p_skin.a[0][win_idx[p]]).collect::<V>(), &skp.iter().map(|&p| p_skin.a[1][win_idx[p]]).collect::<V>());
    for (q, &p) in skp.iter().enumerate() { for c in 0..3 { col[c][p] = skc_[c][q]; } }
    // the screens: each pixel's texel in its quad's picture, times the level
    o.stage("sh_screen");
    let scp = pix_of(Src_::Screen);
    let mut bytes = Vec::with_capacity(scp.len());
    let mut lvl_k = Vec::with_capacity(scp.len());
    {
        // the texel: the walked place's whole part, held in its picture (by kind)
        let kinds: Vec<i64> = scp.iter().map(|&p| sc.q_kind[p_scr.id[win_idx[p]] as usize]).collect();
        let mut tx = vec![0; scp.len()];
        let mut ty = vec![0; scp.len()];
        for (kind, (pw, ph)) in [(0, MAIN), (1, LED), (2, BANNER)] {
            let sel: Vec<usize> = (0..scp.len()).filter(|&q| kinds[q] == kind).collect();
            let a: V = sel.iter().map(|&q| p_scr.a[0][win_idx[scp[q]]]).collect();
            let b: V = sel.iter().map(|&q| p_scr.a[1][win_idx[scp[q]]]).collect();
            let a = o.div(v(&a), K(1000), 0);
            let b = o.div(v(&b), K(1000), 0);
            let a = o.clamp(v(&a), 0, pw as i64 - 1);
            let b = o.clamp(v(&b), 0, ph as i64 - 1);
            for (k, &q) in sel.iter().enumerate() { tx[q] = a[k] as usize; ty[q] = b[k] as usize; }
        }
        // the texels' bytes: first the stand-ins' texels they reach worked out, then fetched
        let fetch = |sd: &Sd, q: usize, p: usize| -> [u8; 3] {
            let aux = sc.q_aux[p_scr.id[win_idx[p]] as usize] as usize;
            match kinds[q] {
                0 => main_texel(sc, sd, shows.main, tx[q], ty[q]),
                1 => led_texel(sc, sd, shows.leds[aux], tx[q], ty[q]),
                _ => banner_texel(sc, sd, shows.banners[aux], tx[q], ty[q]),
            }
        };
        let ask = Sd::ask();
        for (q, &p) in scp.iter().enumerate() { fetch(&ask, q, p); }
        standins(o, sc, t12, &ask.asked(), sd);
        for (q, &p) in scp.iter().enumerate() { bytes.push(fetch(&sd, q, p)); lvl_k.push(kinds[q]); }
    }
    let ln_main = { let l = s_tab_o(&t.ln, 999_999, main_lvl, o); s_sum(&[(l, 1)], -LN1000, o) };
    let ln_led = { let l = s_tab_o(&t.ln, 999_999, led_lvl, o); s_sum(&[(l, 1)], -LN1000, o) };
    let ban = { let a = s_sum(&[(s_div(s_lin(&[(sh.kick, Some(6))], o), 10, o), 1)], 500, o); s_div(s_lin(&[(a, Some(14))], o), 10, o) };
    let ln_ban = { let l = s_tab_o(&t.ln, 999_999, ban, o); s_sum(&[(l, 1)], -LN1000, o) };
    for (kind, lnl) in [(0, ln_main), (1, ln_led), (2, ln_ban)] {
        let sel: Vec<usize> = (0..scp.len()).filter(|&q| lvl_k[q] == kind).collect();
        let b: Vec<[u8; 3]> = sel.iter().map(|&q| bytes[q]).collect();
        let c3 = shade_texels(o, t, &b, lnl);
        for (q, &i) in sel.iter().enumerate() { for c in 0..3 { col[c][scp[i]] = c3[c][q]; } }
    }
    o.stage("sh_sprite");
    // the impostors
    let spp = pix_of(Src_::Sprite);
    if !spp.is_empty() {
        // the impostors' probes' faces turned to the eye (only those in use)
        let mut want = vec![false; sc.probes().len()];
        for spr in &cr.sprites { want[sc.cprobe()[spr.person]] = true; }
        let ids: Vec<usize> = (0..sc.probes().len()).filter(|&k| want[k]).collect();
        let sub: Cube = std::array::from_fn(|f| std::array::from_fn(|ch| ids.iter().map(|&k| pcube[f][ch][k]).collect()));
        let faces = eye_faces(o, t, lt, cam, &sub, &ids.iter().map(|&k| sc.probes()[k]).collect::<Vec<_>>());
        let mut faces_ln: [[V; 3]; 6] = std::array::from_fn(|_| std::array::from_fn(|_| vec![0; sc.probes().len()]));
        for f in 0..6 { for ch in 0..3 { let l = lnv(o, t, &faces[f][ch]).0; for (q, &k) in ids.iter().enumerate() { faces_ln[f][ch][k] = l[q]; } } }
        let win: Vec<usize> = spp.iter().map(|&p| win_idx[p]).collect();
        let wash = flare_wash(o, sh);
        let c3 = shade_sprites(o, t, sc, &cr, &faces_ln, &win, wash);
        for (q, &p) in spp.iter().enumerate() { for c in 0..3 { col[c][p] = c3[c][q]; } }
    }
    o.stage("sh_flag");
    // the flags: their picture times the light both sides
    let fp = pix_of(Src_::Cloth);
    if !fp.is_empty() {
        let tx: V = fp.iter().map(|&p| cloth.p.a[0][win_idx[p]]).collect();
        let ty: V = fp.iter().map(|&p| cloth.p.a[1][win_idx[p]]).collect();
        let tx = { let a = o.div(v(&tx), K(1000), 0); o.clamp(v(&a), 0, 319) };
        let ty = { let a = o.div(v(&ty), K(1000), 0); o.clamp(v(&a), 0, 199) };
        for c in 0..3 {
            let b: V = fp.iter().enumerate().map(|(q, &p)| sc.flags.at(sc.flag_tex[cloth.p.id[win_idx[p]] as usize], ty[q] as usize, tx[q] as usize, c)).collect();
            let lb = o.tab(&t.ln_srgb, v(&b));
            let l: V = fp.iter().map(|&p| cloth.p.a[2 + c][win_idx[p]]).collect();
            let ll = o.tabo(&t.ln, 999_999, v(&l));
            let y = o.sum(&[(v(&lb), 1), (v(&ll), 1)], -LN10K);
            let e = expv(o, t, &y, None);
            for (q, &p) in fp.iter().enumerate() { col[c][p] = e[q]; }
        }
    }
    for p in pix_of(Src_::Pole) { for c in 0..3 { col[c][p] = poles.p.a[c][win_idx[p]]; } }
    o.stage("sh_ball");
    let bp_ = pix_of(Src_::Ball);
    if !bp_.is_empty() {
        let win: Vec<usize> = bp_.iter().map(|&p| win_idx[p]).collect();
        let (_, c3) = shade_balloons(o, t, sc, &bal, &win);
        for (q, &p) in bp_.iter().enumerate() { for c in 0..3 { col[c][p] = c3[c][q]; } }
    }
    o.stage("sh_ship");
    if let Some(d) = ship {
        let sp_ = pix_of(Src_::Ship);
        let kinds: Vec<u32> = sp_.iter().map(|&p| d.kind[d.p.id[win_idx[p]] as usize]).collect();
        let mut scr_px = vec![];
        for (q, &p) in sp_.iter().enumerate() {
            match kinds[q] {
                3 => scr_px.push(p),
                4 => { for (c, x) in [14_000i64, 3920, 840].iter().enumerate() { col[c][p] = *x; } }
                _ => { for c in 0..3 { col[c][p] = d.p.a[c][win_idx[p]]; } }
            }
        }
        if !scr_px.is_empty() {
            // the picture's texel: (a (w - 1), (1 - b) (h - 1)) of the picture it shows, 1.3 times bright
            let (pw, ph) = if shows.airship == Src::Fingerman { (192, 108) } else { (160, 120) };
            let a: V = scr_px.iter().map(|&p| d.p.a[3][win_idx[p]]).collect();
            let b: V = scr_px.iter().map(|&p| d.p.a[4][win_idx[p]]).collect();
            let tx = { let x = o.lin(&[(v(&a), Some(K(pw - 1)))]); let x = o.div(v(&x), K(1000), 0); o.clamp(v(&x), 0, pw - 1) };
            let ty = { let b1 = o.sum(&[(v(&b), -1)], 1000); let x = o.lin(&[(v(&b1), Some(K(ph - 1)))]); let x = o.div(v(&x), K(1000), 0); o.clamp(v(&x), 0, ph - 1) };
            let ask = Sd::ask();
            for q in 0..scr_px.len() { airship_texel(sc, &ask, shows.airship, &shows.grid_rows, tx[q] as usize, ty[q] as usize); }
            standins(o, sc, t12, &ask.asked(), sd);
            let bytes: Vec<[u8; 3]> = (0..scr_px.len()).map(|q| airship_texel(sc, &sd, shows.airship, &shows.grid_rows, tx[q] as usize, ty[q] as usize)).collect();
            let c3 = shade_texels(o, t, &bytes, lnf(1.3));
            for (q, &p) in scr_px.iter().enumerate() { for c in 0..3 { col[c][p] = c3[c][q]; } }
        }
    }
    o.stage("sh_people");
    let pp_ = pix_of(Src_::Person);
    if !pp_.is_empty() {
        let c3 = shade_people(o, t, sc, sh, &near, &pp_.iter().map(|&p| win_idx[p]).collect::<Vec<_>>());
        for (q, &p) in pp_.iter().enumerate() { for c in 0..3 { col[c][p] = c3[c][q]; } }
    }
}

/// ln x of a constant of the scene (an albedo): the printed table's entry.
pub fn t_ln(t: &Tables, x: i64) -> i64 { t.ln.data[(x + 999_999) as usize] }

/// The nearest piece on each pixel: the pieces of every list sorted to their pixels (the host's sort, keeping the
/// lists' order), CARDIAC's nearest (the greatest K / z, the first of equals) where there are more than one. Each
/// pixel's list and piece, and its K / z (0 for none).
pub fn resolve<O: Ops>(o: &mut O, srcs: &[(&V, &Vec<u32>, Src_)]) -> (Vec<Option<Src_>>, Vec<usize>, V) {
    let mut count = vec![0u32; npix() + 1];
    for (_, px, _) in srcs { for &p in px.iter() { count[p as usize + 1] += 1; } }
    for i in 0..npix() { count[i + 1] += count[i]; }
    let total = count[npix()] as usize;
    let mut at = count.clone();
    let mut slot: Vec<(u8, u32)> = vec![(0, 0); total];
    for (s, (_, px, _)) in srcs.iter().enumerate() { for (i, &p) in px.iter().enumerate() { let k = &mut at[p as usize]; slot[*k as usize] = (s as u8, i as u32); *k += 1; } }
    let mut win_src = vec![None; npix()];
    let mut win_idx = vec![0usize; npix()];
    let mut depth = vec![0i64; npix()];
    let mut groups_px = vec![];
    for p in 0..npix() {
        let (a, b) = (count[p] as usize, count[p + 1] as usize);
        if b == a + 1 { let (s, i) = slot[a]; win_src[p] = Some(srcs[s as usize].2); win_idx[p] = i as usize; depth[p] = srcs[s as usize].0[i as usize]; }
        else if b > a + 1 { groups_px.push(p); }
    }
    let lists: Vec<Vec<i64>> = groups_px.iter().map(|&p| (count[p] as usize..count[p + 1] as usize).map(|k| { let (s, i) = slot[k]; srcs[s as usize].0[i as usize] }).collect()).collect();
    let refs: Vec<&[i64]> = lists.iter().map(|l| l.as_slice()).collect();
    let best = if refs.is_empty() { vec![] } else { o.nearest(&refs) };
    for (g, &p) in groups_px.iter().enumerate() {
        let (s, i) = slot[count[p] as usize + best[g]];
        win_src[p] = Some(srcs[s as usize].2);
        win_idx[p] = i as usize;
        depth[p] = lists[g][best[g]];
    }
    (win_src, win_idx, depth)
}

/// The sky: by each pixel's ray's height part (forward's plus the row's up part, over the ray's length: the
/// screen's constant), the printed sky; before the hand-off the night arrival's sky deepening into the violet.
pub fn sky<O: Ops>(o: &mut O, w: &World, cam: &Cam, t12: i64) -> [V; 3] {
    let t = &w.t;
    // the height part per row, hundred-thousandths: f_z 100 - (2 y + 1 - H) u_z / 20
    let rows = &w.scr_now().rows;
    let num = { let n = s_neg(cam.b[1][2], o); o.lin(&[(v(rows), Some(K(n)))]) };
    let num = divk(o, &num, 20 * focal() / 1000);
    let fz = s_lin(&[(cam.b[2][2], Some(100))], o);
    let num = o.sum(&[(v(&num), 1)], fz);
    // each pixel: num inv / 10^6, by logarithms
    let (lnum, ng) = lnv(o, t, &num);
    let ln_inv = o.tabo(&t.ln, 999_999, v(&w.scr_now().inv));
    let ly: V = (0..npix()).map(|p| lnum[p / scr_w() as usize]).collect();
    let ngp: V = (0..npix()).map(|p| ng[p / scr_w() as usize]).collect();
    let y = o.sum(&[(v(&ly), 1), (v(&ln_inv), 1)], -lnf(100_000.0));
    let el = expv(o, t, &y, Some(&ngp));
    let el = o.clamp(v(&el), -1000, 1000);
    let fade = s_clamp(s_sum(&[(s_div(s_lin(&[(t12, Some(1000))], o), 36, o), -1)], 1000, o), 0, 1000, o);
    std::array::from_fn(|c| {
        let vio = o.tabo(&t.sky[1][c], 1000, v(&el));
        if fade == 0 { return vio; }
        let old = o.tabo(&t.sky[0][c], 1000, v(&el));
        // vio + (old - vio) fade / 1000
        let d = o.sum(&[(v(&old), 1), (v(&vio), -1)], 0);
        let m = o.lin(&[(v(&d), Some(K(fade)))]);
        let m = o.div(v(&m), K(1000), 0);
        o.sum(&[(v(&vio), 1), (v(&m), 1)], 0)
    })
}

/// The stars, on the sky's pixels: each one's direction onto the screen (in front), its light added.
pub fn stars<O: Ops>(o: &mut O, w: &World, cam: &Cam, win: &[Option<Src_>], col: &mut [V; 3]) {
    let (sc, t) = (&w.sc, &w.t);
    let d: [V; 3] = std::array::from_fn(|i| sc.stars.iter().map(|s| s[i]).collect());
    let cc: [V; 3] = std::array::from_fn(|r| { let s = o.lin(&[(v(&d[0]), Some(K(cam.b[r][0]))), (v(&d[1]), Some(K(cam.b[r][1]))), (v(&d[2]), Some(K(cam.b[r][2])))]); o.div(v(&s), K(1000), 0) });
    let ahead = o.lt(K(99), v(&cc[2]));
    let at: Vec<usize> = (0..d[0].len()).filter(|&i| ahead[i] == 1).collect();
    let pts: Vec<[i64; 3]> = at.iter().map(|&i| [cc[0][i], cc[1][i], cc[2][i]]).collect();
    let pr = project(o, t, &pts);
    let xs: V = pr.iter().map(|p| p.0).collect();
    let ys: V = pr.iter().map(|p| p.1).collect();
    let (px, _) = o.divmod(v(&xs), K(100));
    let (py, _) = o.divmod(v(&ys), K(100));
    // the stars on the open sky's pixels; of those on one pixel, the first
    let mut cand: Vec<(i64, i64, usize)> = vec![];
    for (q, &i) in at.iter().enumerate() {
        let (x, y) = (px[q] + scr_w() / 2, py[q] + scr_h() / 2);
        if x < 0 || x >= scr_w() || y < 0 || y >= scr_h() { continue; }
        if win[(y * scr_w() + x) as usize].is_some() { continue; }
        cand.push((y, x, i));
    }
    let first = one_each(o, &cand.iter().map(|c| [c.0, c.1]).collect::<Vec<_>>(), false);
    let hit: Vec<(usize, usize)> = (0..cand.len()).filter(|&k| first[k]).map(|k| ((cand[k].0 * scr_w() + cand[k].1) as usize, cand[k].2)).collect();
    let b: V = hit.iter().map(|h| sc.star_b[h.1]).collect();
    for (c, k) in [5000i64, 5500, 7000].iter().enumerate() {
        let a = o.lin(&[(v(&b), Some(K(*k)))]);
        let a = divk(o, &a, 10000);
        let old: V = hit.iter().map(|h| col[c][h.0]).collect();
        let s = o.sum(&[(v(&old), 1), (v(&a), 1)], 0);
        for (q, h) in hit.iter().enumerate() { col[c][h.0] = s[q]; }
    }
}

/// Dope's stars: crisp white points fixed in the sky, on the open sky's pixels only (behind everything else:
/// light adds, so the beams' over them all the same; after the glow, so crisp), a quarter of them 2 x 2: each one's direction onto the screen, its light
/// (white: the same in each channel) added.
pub fn dope_stars<O: Ops>(o: &mut O, w: &World, cam: &Cam, win: &[Option<Src_>], col: &mut [V; 3]) -> Vec<usize> {
    let (sc, t) = (&w.sc, &w.t);
    let d: [V; 3] = std::array::from_fn(|i| sc.dstars.iter().map(|s| s[i]).collect());
    let lr: [V; 3] = std::array::from_fn(|i| o.tabo(&t.ln, 999_999, v(&d[i])));
    let nr: [V; 3] = std::array::from_fn(|i| o.tabo(&t.neg, 999_999, v(&d[i])));
    let cc: [V; 3] = std::array::from_fn(|r| logdot3(o, t, &lr, &nr, cam.b[r], 1000));
    let ahead = o.lt(K(99), v(&cc[2]));
    let at: Vec<usize> = (0..d[0].len()).filter(|&i| ahead[i] == 1).collect();
    let pts: Vec<[i64; 3]> = at.iter().map(|&i| [cc[0][i], cc[1][i], cc[2][i]]).collect();
    let pr = project(o, t, &pts);
    let xs: V = pr.iter().map(|p| p.0).collect();
    let ys: V = pr.iter().map(|p| p.1).collect();
    let (px, _) = o.divmod(v(&xs), K(100));
    let (py, _) = o.divmod(v(&ys), K(100));
    // the stars' pixels on the open sky (a big one's four: its place and the next row and column, by the host's
    // addressing); of those on one pixel, the first
    let mut cand: Vec<(i64, i64, usize)> = vec![];
    for (q, &i) in at.iter().enumerate() {
        let n = if sc.dstar_big[i] == 1 { 2 } else { 1 };
        for dy in 0..n { for dx in 0..n {
            let (x, y) = (px[q] + scr_w() / 2 + dx, py[q] + scr_h() / 2 + dy);
            if x < 0 || x >= scr_w() || y < 0 || y >= scr_h() { continue; }
            if win[(y * scr_w() + x) as usize].is_some() { continue; }
            cand.push((y, x, i));
        } }
    }
    let first = one_each(o, &cand.iter().map(|c| [c.0, c.1]).collect::<Vec<_>>(), false);
    let hit: Vec<(usize, usize)> = (0..cand.len()).filter(|&k| first[k]).map(|k| ((cand[k].0 * scr_w() + cand[k].1) as usize, cand[k].2)).collect();
    if std::env::var("STADIUM_DEBUG").is_ok() { eprintln!("dope stars: {} dirs, {} ahead, {} pixels, sky {}", d[0].len(), at.len(), hit.len(), win.iter().filter(|s| s.is_none()).count()); }
    let l: V = hit.iter().map(|h| sc.dstar_l[h.1]).collect();
    for c in 0..3 {
        let old: V = hit.iter().map(|h| col[c][h.0]).collect();
        let s = o.sum(&[(v(&old), 1), (v(&l), 1)], 0);
        for (q, h) in hit.iter().enumerate() { col[c][h.0] = s[q]; }
    }
    hit.iter().map(|h| h.0).collect()
}

/// Of places (row, column; 0 or more), which one of each place is kept: the first of those at one place (`last`:
/// the last). The card sorter puts them in order (by row, then column; those at one place in their order), and
/// CARDIAC compares each with its neighbour in that order (the one before it, or with `last` the one after it): a
/// later row, or the same row and a later column, is another place.
pub fn one_each<O: Ops>(o: &mut O, places: &[[i64; 2]], last: bool) -> Vec<bool> {
    let n = places.len();
    let mut keep = vec![true; n];
    if n < 2 { return keep; }
    let deck = sorter::order(places, &[sorter::up(4), sorter::up(4)]);
    // each card and the next in the deck
    let (a, b) = (&deck[..n - 1], &deck[1..]);
    let (ya, xa): (V, V) = (a.iter().map(|&i| places[i][0]).collect(), a.iter().map(|&i| places[i][1]).collect());
    let (yb, xb): (V, V) = (b.iter().map(|&i| places[i][0]).collect(), b.iter().map(|&i| places[i][1]).collect());
    let row = o.lt(v(&ya), v(&yb));
    let col = o.lt(v(&xa), v(&xb));
    for k in 0..n - 1 {
        if row[k] == 1 || col[k] == 1 { continue; }
        // the same place: the later card (or with `last` the earlier) is not kept
        keep[if last { a[k] } else { b[k] }] = false;
    }
    keep
}

/// The tone curve: each channel's light (held to 0..60,000) to its byte by the printed table.
pub fn tone<O: Ops>(o: &mut O, t: &Tables, col: &[V; 3]) -> Vec<u8> {
    let b: [V; 3] = std::array::from_fn(|c| { let x = o.clamp(v(&col[c]), 0, TONE_N); o.tab(&t.tone, v(&x)) });
    let mut out = vec![0u8; npix() * 3];
    for p in 0..npix() { for c in 0..3 { out[3 * p + c] = b[c][p] as u8; } }
    out
}

/// The outer wall's LED skin, as the night arrival draws it: two rows of the marquee scrolling opposite ways round
/// the bowl, dots of light, the colour turning with the angle, pulsing on the beat.
pub fn skin<O: Ops>(o: &mut O, t: &Tables, sc: &Scene, sh: &Show, ang: &V, z: &V) -> [V; 3] {
    let n = ang.len();
    if n == 0 { return [vec![], vec![], vec![]]; }
    // the night arrival's frame: 389 at the crest
    let f = s_sum(&[(sh.t12, 1)], 389 - 36, o);
    let mq = &sc.marquee;
    let len = mq.w as i64;
    // columns: the angle (millionths of a turn) times 2 pi 125 / 0.525, thousandths: ang 1.496
    let colk = o.lin(&[(v(ang), Some(K(1496)))]);
    let colk = o.div(v(&colk), K(1000), 0);
    let mut rows_lit: Vec<V> = vec![];
    for (y0, y1, dirn) in [(1100i64, 1940i64, 1i64), (2220, 3060, -1)] {
        // the row: (y1 - z) 16 / 8.4, thousandths
        let r = o.sum(&[(v(z), -1)], y1);
        let r = o.lin(&[(v(&r), Some(K(16_000)))]);
        let r = o.div(v(&r), K(y1 - y0), 0);
        // the column, scrolled 3 a frame: wrapped by the frame's offset (mod the marquee) and the printed wrap
        let off = { let s = s_lin(&[(f, Some(3 * dirn))], o); let q = s_div(s, len, o); let nl = s_neg(len, o); s_lin(&[(s, None), (q, Some(nl))], o) };
        let (ci, cf) = split1000(o, &colk);
        let ci = o.sum(&[(v(&ci), 1)], off);
        let ci = o.tabo(&t.mq_wrap, 2000, v(&ci));
        let (ri, rf) = split1000(o, &r);
        let dot_c = o.tab(&t.dot, v(&cf));
        let dot_r = o.tab(&t.dot, v(&rf));
        // the row's level at each place (0, 400 or 1000), by the marquee's dot and the punched flags
        let lit: V = (0..n).map(|i| {
            if !(0..16).contains(&ri[i]) || mq.at(0, ri[i] as usize, ci[i] as usize, 0) <= 127 { 0 }
            else if dot_c[i] == 1 && dot_r[i] == 1 { 1000 } else { 400 }
        }).collect();
        rows_lit.push(lit);
    }
    // where both rows land, the brighter (CARDIAC's max)
    let lit = o.max(v(&rows_lit[0]), v(&rows_lit[1]));
    // the hue: ang 3 + f 0.05 radians, as turns; each colour 0.5 + 0.5 sin(hue + 2.1 k) by the printed table
    let f05 = s_lin(&[(f, Some(7958))], o);
    let hue = o.lin(&[(v(ang), Some(K(3))), (K(f05), None)]);
    let hue = o.turns(v(&hue));
    let hue = o.div(v(&hue), K(1000), 0);
    let ph = s_divmod(s_sum(&[(f, 1)], -389, o), 6, o).1;
    let pulse = s_tab(&t.skin_pulse, ph, o);
    let k = s_lin(&[(pulse, Some(4))], o);
    let glow: [i64; 3] = std::array::from_fn(|c| s_div(s_lin(&[(sc.mat_alb[3][c], Some(3))], o), 1000, o));
    std::array::from_fn(|c| {
        let h = o.tab(&t.hue[c], v(&hue));
        let p = o.lin(&[(v(&h), Some(v(&lit)))]);
        let p = o.div(v(&p), K(1000), 0);
        let p = o.lin(&[(v(&p), Some(K(k)))]);
        let p = o.div(v(&p), K(100), 0);
        o.sum(&[(v(&p), 1)], glow[c])
    })
}

/// A number's thousands and its last three digits (0..999).
pub fn split1000<O: Ops>(o: &mut O, x: &V) -> (V, V) {
    let q = o.div(v(x), K(1000), 0);
    let r = o.lin(&[(v(x), None), (v(&q), Some(K(-1000)))]);
    (q, r)
}

impl Tables {
    /// the marquee's column wrapped: (c + 2000 - 2000) mod 17,360
    pub fn mq_wrap_tab(len: i64) -> Tab { tab("mq_wrap", (-2000..len + 2000).map(|c| c.rem_euclid(len)).collect()) }
    /// whether a place's fraction (thousandths) is within 0.35 of the middle (a dot of the LED's)
    pub fn dot_tab() -> Tab { tab("dot", (0..1000).map(|f: i64| ((f - 500).abs() < 350) as i64).collect()) }
    /// 0.5 + 0.5 sin(2 pi h / 1000 + 2.1 k), thousandths
    pub fn hue_tabs() -> [Tab; 3] { std::array::from_fn(|k| tab("hue", (0..1000).map(|h| ((0.5 + 0.5 * (h as f64 / 1000.0 * std::f64::consts::TAU + 2.1 * k as f64).sin()) * 1000.0).round() as i64).collect())) }
    /// the skin's pulse by the frame in its 6: 1 - 0.65 (1 - e^(-4 ph / 6)), thousandths
    pub fn skin_pulse_tab() -> Tab { tab("skin_pulse", (0..6).map(|p| ((1.0 - 0.65 * (1.0 - (-4.0 * p as f64 / 6.0).exp())) * 1000.0).round() as i64).collect()) }
}

/// The flare's wash on the crowd (the whole crowd lit up for a moment): white light, 20 a
/// thousandth of the flare (the crowd's own light is about 30 typically, 200 at its brightest).
pub fn flare_wash<O: Ops>(o: &mut O, sh: &Show) -> i64 { if sh.flare > 0 { s_lin(&[(sh.flare, Some(20))], o) } else { 0 } }
