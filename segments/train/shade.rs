// The frame put together: the sky, the shading (materials.rs), the motion blur, the wires and the lens (post.rs).
// Included after sky.rs.

/// What the start works out once: the screen's ray factors and the vignette.
pub struct Screen { pub vlev: V, pub nlq7: V, pub lq: V }

pub struct World { pub sc: Scene, pub tb: Tb, pub scr: Screen, pub tun: TunnelMaps, pub lines: BlurLines, pub wires: Wires, pub sh: Shadows, pub ghosts: Vec<Ghost>, pub skyk: SkyConsts,
    /// the screen's numbers and the blur's lines at half the resolution (the party version's world)
    pub scr_h: Screen, pub lines_h: BlurLines }

impl World {
    /// The screen's numbers for the resolution drawn.
    pub fn screen(&self) -> &Screen { if res() == 1 { &self.scr } else { &self.scr_h } }
    /// The blur's lines for the resolution drawn.
    pub fn blur_lines(&self) -> &BlurLines { if res() == 1 { &self.lines } else { &self.lines_h } }
}

pub fn load<O: Ops>(o: &mut O, dir: &str) -> World {
    let b = Bin::read(&format!("{dir}/train/data/scene.bin"));
    let kinds = b.get("m_kind");
    let alb: Vec<[i64; 3]> = b.rows("m_alb");
    let par: Vec<[i64; 6]> = b.rows("m_par");
    let mats = (0..kinds.len()).map(|i| Mat { kind: kinds[i], alb: alb[i], par: par[i] }).collect();
    let bend = bend_table(o);
    let mut t = Tables { trig: read_trig(dir), recip: recip_table(), bend, bend_cm: Tab { name: "bend_cm", data: vec![], w: 0 } };
    t.bend_cm = bend_cm_table(o, &t);
    let wires = { let (s, d) = &b.0["city_wires"]; d.chunks(s[1]).map(|c| c.to_vec()).collect() };
    let sc = Scene {
        world: Mesh::read(&b, "w_"),
        train: Mesh::read(&b, "t_"),
        mats,
        zlo: b.get("w_zlo").to_vec(), zhi: b.get("w_zhi").to_vec(), xmid: b.get("w_xmid").to_vec(),
        small: b.get("w_small").iter().map(|&s| s != 0).collect(), lod: b.get("w_lod").to_vec(), iz: b.get("w_iz").to_vec(),
        poles: b.get("poles").to_vec(),
        city_wires: wires,
        t,
        ride: b.get("w_ride")[0] as usize,
    };
    let mut tb = Tb::read(&format!("{dir}/train/data/tables.bin"));
    // the half resolution's blur lines (data/tables_h.bin, make_tables.py --half), if made
    if let Ok(true) = std::fs::exists(format!("{dir}/train/data/tables_h.bin")) { for (k, v) in Tb::read(&format!("{dir}/train/data/tables_h.bin")).0 { tb.0.insert(k, v); } }
    let scr = screen(o, &tb);
    let tun = tunnel_maps(o, &tb);
    let lines = blur_lines(&tb);
    let wires = Wires { seg: b.rows("wires") };
    let sh = shadow_maps(o, &tb, &sc);
    let ghosts = ghost_sprites(o);
    let skyk = sky_consts_all(o, &tb);
    // at half the resolution (the screen's base by the full screen's distances: the same vignette)
    set_res(2);
    let scr_h = screen(o, &tb);
    let lines_h = if tb.0.contains_key("blur_line_h") { blur_lines(&tb) } else { BlurLines { dirs: vec![] } };
    set_res(1);
    World { sc, tb, scr, tun, lines, wires, sh, ghosts, skyk, scr_h, lines_h }
}

/// The screen's numbers, once, each a function of the pixel's squared distance from the middle s (pixels squared:
/// the column's part and the row's, each a quarter of the half-pixel offset squared, added on CARDIAC), by the printed
/// tables of s: the vignette's level for the tone (1 - 0.22 r^2, r over the corner's, in eighths), the logarithm of
/// what makes a ray at 0.7 of its length a unit (ln (500 / q7), q7 = q / 1.4; see unit_rays) and the fog's logarithm
/// of the screen's factor (lq; see materials.rs).
pub fn screen<O: Ops>(o: &mut O, tb: &Tb) -> Screen {
    // (in the full screen's half pixels, at whatever resolution it's drawn)
    let xs: V = (0..W()).map(|x| (2 * x + 1 - W()) * res()).collect();
    let ys: V = (0..H()).map(|y| (2 * y + 1 - H()) * res()).collect();
    let x2 = o.lin(&[(v(&xs), Some(v(&xs)))]);
    let x2 = o.div(v(&x2), K(4), 0);
    let y2 = o.lin(&[(v(&ys), Some(v(&ys)))]);
    let y2 = o.div(v(&y2), K(4), 0);
    let mut s = vec![0; N_PIX()];
    for y in 0..H() as usize {
        let r = o.sum(&[(v(&x2), 1)], y2[y]);
        s[y * W() as usize..(y + 1) * W() as usize].copy_from_slice(&r);
    }
    Screen { vlev: o.tab(tb.t("scr_vlev"), v(&s)), nlq7: o.tab(tb.t("scr_nlq7"), v(&s)), lq: o.tab(tb.t("scr_lq"), v(&s)) }
}

/// The exposure (thousandths): 0.78 by day, opening up to 2.4 in the tunnel's dark over its first 50 m, closing to
/// 0.9 over 65 m out into the sunset (the second ramp to the 0.7th power).
pub fn exposure<O: Ops>(o: &mut O, tb: &Tb, z: i64) -> i64 {
    let a = o.lin(&[(K(z), None), (K(-25_500), None)]);
    let a = o.lin(&[(v(&a), Some(K(1000)))]);
    let r1 = o.div(v(&a), K(5000), 0);
    let r1 = o.clamp(v(&r1), 0, 1000);
    let b = o.lin(&[(K(z), None), (K(-49_500), None)]);
    let b = o.lin(&[(v(&b), Some(K(1000)))]);
    let r2 = o.div(v(&b), K(6500), 0);
    let r2 = o.clamp(v(&r2), 0, 1000);
    let p = o.tab(tb.t("pow07"), v(&r2));
    let e = o.lin(&[(v(&r1), Some(K(1620))), (v(&p), Some(K(-1500))), (K(780_000), None)]);
    one(o.div(v(&e), K(1000), 0))
}

/// One frame (of the departure and the ride: our frame k), RGB: the full version.
pub fn render<O: Ops>(o: &mut O, w: &World, k: i64) -> Vec<u8> { render_parts(o, w, k).img }

/// The same, the full version or the party version (party.rs).
pub fn render_v<O: Ops>(o: &mut O, w: &World, k: i64, party: bool) -> Vec<u8> {
    if !party { return render(o, w, k); }
    set_res(2);
    let lo = render_parts(o, w, k);
    set_res(1);
    train_compose(o, w, k, &lo)
}

/// A render's picture (RGB at the resolution drawn) and what the party version's full-resolution pass takes from it:
/// the linear colours (cards and the part past 999) and the lens's light, what each pixel sees and the world's own
/// depth behind the subjects, each 2x2 block's blur length, and the frame's full-resolution camera, light and sky.
pub struct TParts { pub img: Vec<u8>, pub add: [V; 3], pub g: GBuf, pub wiz: V, pub mlen: V,
    pub cam: Cam, pub day: bool, pub e: i64, pub map: SkyMap, pub round: SkyMap, pub front: Option<i64>, pub fmap: Option<SMap>, pub kr: i64 }

pub fn render_parts<O: Ops>(o: &mut O, w: &World, k: i64) -> TParts {
    // the ride's time from here on (before 0 while departing)
    // (the ride time, on the machines)
    let k = { let d = kl(o, &[(k, None), (-DEPART, None)]); one(o.max(K(d), K(HOLD - DEPART))) };
    let (sc, tb) = (&w.sc, &w.tb);
    o.stage("camera");
    let cam_full = { let t = kl(o, &[(k, Some(10))]); camera(o, &sc.t, t) };
    let cam = res_cam(o, &cam_full);
    o.stage("geometry");
    let (g, ray, front, wiz) = gbuffer(o, w, &cam, k);
    o.stage("skymap");
    let env = if cam.z_cm < TUN0 + 1000 { &DAY } else { &DUSK };
    let e = exposure(o, tb, cam.z_cm);
    let (map, round) = sky_maps(o, tb, env, &w.skyk, &cam, e);
    let mut col: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    let mut hot: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    // the shading's rate: where the blur will smear a pixel over 6 or more, one pixel a 2x2 block is shaded; over 24 or
    // more, one a 4x4 tile (each tile by its slowest block); the rest take their shaded pixel's colour
    o.stage("blur");
    let ca = { let t = kl(o, &[(k, Some(10)), (-2, None)]); let c = camera(o, &sc.t, t); res_cam(o, &c) };
    let cb = { let t = kl(o, &[(k, Some(10)), (2, None)]); let c = camera(o, &sc.t, t); res_cam(o, &c) };
    let mo = velocities_c(o, w, &cam, &ca, &cb, &g, &[(2, [0, 0, 2800])]);
    let (rep, rate) = shading_reps(o, &mo.len);
    // the sky: worked out at every fourth pixel each way (from (2, 2)) where the sky's pixels need it (at dusk with the
    // far ridges and the sun's core), made fine by up4; each pixel its own where a ridge's outline crosses its cell
    o.stage("sky");
    let skyp: Vec<usize> = (0..N_PIX()).filter(|&p| g.mat[p] < 0).collect();
    let mut sky_c: [V; 3] = [vec![], vec![], vec![]];
    let mut sky_h: [V; 3] = [vec![0; skyp.len()], vec![0; skyp.len()], vec![0; skyp.len()]];
    if !skyp.is_empty() {
        let (gw, gh) = ((W() / 4) as usize, (H() / 4) as usize);
        let corner = |p: usize| -> (usize, usize) { let (x, y) = ((p % W() as usize) as i64, (p / W() as usize) as i64); (((x - 2).max(0) / 4) as usize, ((y - 2).max(0) / 4) as usize) };
        let mut need = vec![false; gw * gh];
        for &p in &skyp { let (i, j) = corner(p); for (di, dj) in [(0, 0), (1, 0), (0, 1), (1, 1)] { need[(j + dj).min(gh - 1) * gw + (i + di).min(gw - 1)] = true; } }
        let pts: Vec<usize> = (0..gw * gh).filter(|&q| need[q]).collect();
        let px_of = |q: usize| -> usize { (4 * (q / gw) + 2) * W() as usize + 4 * (q % gw) + 2 };
        let pick = |v: &V| -> V { pts.iter().map(|&q| v[px_of(q)]).collect() };
        let r = [pick(&ray[0]), pick(&ray[1]), pick(&ray[2])];
        let dn = unit_rays(o, tb, &[&r[0], &r[1], &r[2]], &pick(&w.screen().nlq7));
        let az = azimuth(o, tb, &dn[0], &dn[2]);
        let mut c = sky_look(o, tb, &map, &az, &dn[1]);
        let mut h: [V; 3] = [vec![0; pts.len()], vec![0; pts.len()], vec![0; pts.len()]];
        // at dusk: the ridges and the sun's core at the grid's points too; what each point sees
        let mut seen = vec![0; gw * gh];
        let near_sun = {
            let s: [i64; 3] = std::array::from_fn(|r| one({ let t = o.lin(&[(K(cam.b[r][0]), Some(K(env.sun[0]))), (K(cam.b[r][1]), Some(K(env.sun[1]))), (K(cam.b[r][2]), Some(K(env.sun[2])))]); o.div(v(&t), K(1000), 0) }));
            // (the sun's place on the screen, on the machines)
            let d2 = { let m = one(o.clamp(K(s[2]), 1, 999_999)); kl(o, &[(m, Some(2))]) };
            let sx = { let q = one({ let t = o.lin(&[(K(s[0]), Some(K(cam.f2)))]); o.div(v(&t), K(d2), 0) }); one(o.sum(&[(K(q), 1)], W() / 2)) };
            let sy = { let q = one({ let t = o.lin(&[(K(s[1]), Some(K(cam.f2)))]); o.div(v(&t), K(d2), 0) }); one(o.sum(&[(K(q), -1)], H() / 2)) };
            move |p: usize| { let (x, y) = ((p as i64) % W(), (p as i64) / W()); !env.day && s[2] > 0 && (x - sx).abs() < 110 && (y - sy).abs() < 110 }
        };
        if !env.day {
            let lowf = o.lt(v(&dn[1]), K(75));
            let low: Vec<usize> = (0..pts.len()).filter(|&k| lowf[k] == 1).collect();
            let codes = ridges(o, tb, e, &low, &az, &dn[1], &mut c);
            for (q, &k) in low.iter().enumerate() { seen[pts[k]] = codes[q]; }
            let sun: Vec<usize> = (0..pts.len()).filter(|&k| near_sun(px_of(pts[k]))).collect();
            sun_core(o, tb, env, e, &sun, &[&dn[0], &dn[1], &dn[2]], &mut c, &mut h);
        }
        // made fine (the hot part only where there is any)
        let (j0, j1) = (pts[0] / gw, pts[pts.len() - 1] / gw);
        let rows = j1 - j0 + 1;
        let fine_of = |o: &mut O, vals: &V| -> V {
            let mut grid = vec![0; gw * rows];
            for (k, &q) in pts.iter().enumerate() { grid[q - j0 * gw] = vals[k]; }
            let fine = upsample(o, &grid, gw, rows);
            skyp.iter().map(|&p| { let (x, y) = (p % W() as usize, p / W() as usize); let yy = y as i64 - 4 * j0 as i64; fine[(yy.clamp(0, 4 * rows as i64 - 1) as usize) * W() as usize + x] }).collect()
        };
        for ch in 0..3 {
            sky_c[ch] = fine_of(o, &c[ch]);
            if h[ch].iter().any(|&x| x != 0) { sky_h[ch] = fine_of(o, &h[ch]); }
        }
        if !env.day {
            // where a cell's corners see different things (a ridge's outline crosses it): each pixel its own
            let (cw, ch_) = (gw - 1, gh - 1);
            let corner_codes: Vec<V> = [(0, 0), (1, 0), (0, 1), (1, 1)].iter().map(|&(di, dj)| (0..cw * ch_).map(|q| seen[(q / cw + dj) * gw + q % cw + di]).collect()).collect();
            let mx = { let a = o.max(v(&corner_codes[0]), v(&corner_codes[1])); let b = o.max(v(&corner_codes[2]), v(&corner_codes[3])); o.max(v(&a), v(&b)) };
            let mn = { let a = o.min(v(&corner_codes[0]), v(&corner_codes[1])); let b = o.min(v(&corner_codes[2]), v(&corner_codes[3])); o.min(v(&a), v(&b)) };
            let edge = o.lt(v(&mn), v(&mx));
            let at: Vec<usize> = (0..skyp.len()).filter(|&k| {
                let (i, j) = corner(skyp[k]);
                i < cw && j < ch_ && edge[j * cw + i] == 1
            }).collect();
            if std::env::var("TRAIN_COUNT").is_ok() { eprintln!("frame {k}: sky outline {} of {}", at.len(), skyp.len()); }
            if !at.is_empty() {
                let pk = |v: &V| -> V { at.iter().map(|&k| v[skyp[k]]).collect() };
                let r = [pk(&ray[0]), pk(&ray[1]), pk(&ray[2])];
                let dn = unit_rays(o, tb, &[&r[0], &r[1], &r[2]], &pk(&w.screen().nlq7));
                let az = azimuth(o, tb, &dn[0], &dn[2]);
                let mut cc = sky_look(o, tb, &map, &az, &dn[1]);
                let all: Vec<usize> = (0..at.len()).collect();
                let lowf = o.lt(v(&dn[1]), K(75));
                let low: Vec<usize> = all.iter().copied().filter(|&j| lowf[j] == 1).collect();
                let _ = ridges(o, tb, e, &low, &az, &dn[1], &mut cc);
                let mut hh: [V; 3] = [vec![0; at.len()], vec![0; at.len()], vec![0; at.len()]];
                let sun: Vec<usize> = (0..at.len()).filter(|&j| near_sun(skyp[at[j]])).collect();
                sun_core(o, tb, env, e, &sun, &[&dn[0], &dn[1], &dn[2]], &mut cc, &mut hh);
                for (j, &k) in at.iter().enumerate() { for c_ in 0..3 { sky_c[c_][k] = cc[c_][j]; sky_h[c_][k] = hh[c_][j]; } }
            }
        }
    }
    // the rest, shaded
    o.stage("mirrors");
    let solid: Vec<usize> = (0..N_PIX()).filter(|&p| g.mat[p] >= 0 && rep[p] == p).collect();
    let mirrors = if env.day { Mirrors { paddy: None, water: None } } else {
        let n_paddy = solid.iter().filter(|&&p| g.mat[p] == kind::PADDY).count();
        let n_water = solid.iter().filter(|&&p| g.mat[p] == kind::WATER).count();
        Mirrors {
            paddy: (n_paddy > 50).then(|| Mirror1 { g: mirror_gbuf(o, w, &cam, k, LAND, &ray), h: LAND }),
            water: (n_water > 50).then(|| Mirror1 { g: mirror_gbuf(o, w, &cam, k, RIVER, &ray), h: RIVER }),
        }
    };
    if std::env::var("TRAIN_COUNT").is_ok() { let r = |q: u8| solid.iter().filter(|&&p| rate[p] == q).count(); eprintln!("frame {k}: sky {} solid {} ({} {} {} {}) of {}", skyp.len(), solid.len(), r(1), r(2), r(4), r(8), N_PIX()); }
    o.stage("shadows");
    let fmap = match front { Some(f) if !env.day => Some(train_shadow(o, w, k, f)), _ => None };
    o.stage("shade");
    let (c, h) = shade_solid(o, w, &cam, env, e, k, front, &g, &ray, &solid, &map, &round, Some(&mirrors), None, fmap.as_ref(), &mut Plain);
    for (j, &p) in solid.iter().enumerate() { for ch in 0..3 { col[ch][p] = c[ch][j]; hot[ch][p] = h[ch][j]; } }
    // the shaded pixels' colours copied to the rest of their blocks; the sky's pixels their own
    for p in 0..N_PIX() { if rep[p] != p { for c in 0..3 { col[c][p] = col[c][rep[p]]; hot[c][p] = hot[c][rep[p]]; } } }
    for (k, &p) in skyp.iter().enumerate() { for c in 0..3 { col[c][p] = sky_c[c][k]; hot[c][p] = sky_h[c][k]; } }
    if std::env::var("TRAIN_NOBLUR").is_err() {
        o.stage("blur");
        motion_blur(o, w, w.blur_lines(), &mo, &rate, &mut col, &mut hot);
    }
    o.stage("wires");
    draw_wires(o, w, &cam, env, e, &g, &mut col, &mut hot);
    o.stage("lens");
    let mut add: [V; 3] = [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]];
    let touched = flare(o, w, &cam, env, &g, &mut add);
    bloom(o, w, &col, &hot, &mut add, &touched);
    o.stage("tone");
    let mut out = finish(o, w, &col, &hot, &add);
    // (the words over the city at full resolution: the party version lays them on after its pass)
    if res() == 1 { overlays(o, &w.tb, k, &mut out); }
    let mut img = vec![0u8; N_PIX() * 3];
    for c in 0..3 { for p in 0..N_PIX() { img[3 * p + c] = out[c][p] as u8; } }
    TParts { img, add, g, wiz, mlen: mo.len, cam: cam_full, day: env.day, e, map, round, front, fmap, kr: k }
}

/// The tone curve, the vignette in it: each channel (the card, its tenths past 999 and the lens's light) through the curve by the
/// pixel's vignette (its level, from the start), 0..255; then a little more colour (each channel pushed 15% from the
/// three's mean).
pub fn finish<O: Ops>(o: &mut O, w: &World, col: &[V; 3], hot: &[V; 3], add: &[V; 3]) -> [V; 3] { finish_v(o, w, &w.screen().vlev, col, hot, add) }

/// The same by the pixels' vignette levels given.
pub fn finish_v<O: Ops>(o: &mut O, w: &World, vlev: &[i64], col: &[V; 3], hot: &[V; 3], add: &[V; 3]) -> [V; 3] {
    let tb = &w.tb;
    let t: Vec<V> = (0..3).map(|c| { let i = o.tone_index(&col[c], &hot[c], &add[c], tb.t("x25")); let vl: V = vlev.to_vec(); o.tab2(tb.t("tonev"), v(&vl), v(&i)) }).collect();
    o.tone_push([&t[0], &t[1], &t[2]], tb.t("div20"))
}

/// Which pixel's shading each pixel takes: its own; its 2x2 block's first where the block's blur is more than R2
/// pixels long; its 4x4 tile's first where each of the tile's four blocks' is more than R4; its 8x8 tile's first
/// where each of its blocks' is more than R8 (the lengths compared on CARDIAC; the host groups). And each pixel's
/// rate: 1, 2, 4 or 8.
pub const R2: i64 = 5;
pub const R4: i64 = 15;
pub const R8: i64 = 47;
pub fn shading_reps<O: Ops>(o: &mut O, len: &V) -> (Vec<usize>, Vec<u8>) {
    // the least of each 2x2 of a grid's cells (w x h, even), a grid half the size
    let least = |o: &mut O, g: &V, w: usize, h: usize| -> V {
        let (hw, hh) = (w / 2, h / 2);
        let q = |dx: usize, dy: usize| -> V { (0..hw * hh).map(|t| g[(2 * (t / hw) + dy) * w + 2 * (t % hw) + dx]).collect() };
        let ab = o.min(v(&q(0, 0)), v(&q(1, 0)));
        let cd = o.min(v(&q(0, 1)), v(&q(1, 1)));
        o.min(v(&ab), v(&cd))
    };
    let (bw, bh) = ((W() / 2) as usize, (H() / 2) as usize);
    let t4 = least(o, len, bw, bh);
    let t8 = least(o, &t4, bw / 2, bh / 2);
    let f2 = o.lt(K(R2), v(len));
    let f4 = o.lt(K(R4), v(&t4));
    let f8 = o.lt(K(R8), v(&t8));
    (0..N_PIX()).map(|p| {
        let (x, y) = (p % W() as usize, p / W() as usize);
        if f8[(y / 8) * (bw / 4) + x / 8] == 1 { ((y & !7) * W() as usize + (x & !7), 8) }
        else if f4[(y / 4) * (bw / 2) + x / 4] == 1 { ((y & !3) * W() as usize + (x & !3), 4) }
        else if f2[(y / 2) * bw + x / 2] == 1 { ((y & !1) * W() as usize + (x & !1), 2) }
        else { (p, 1) }
    }).unzip()
}
