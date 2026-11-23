// The night's materials and lamps (train_night.py's materials() and lights() hooks), as the train's shading's
// Shading: the board's dots, the canopies' tubes, the drone and its lights, the callouts' screens, the stadium's lit
// bands and its LED skin, the airship's hull, frame and screen; the light of the station's tubes, the street lights,
// our headlights, and the bowl's wash on the beat. Colours R (250 for 1.0) with the exposure in. Included after
// nframe.rs.

pub struct NightShade<'a> { pub nw: &'a NightWorld, pub l: &'a NightLight, pub cam: &'a Cam, pub scr: &'a Screen, pub mode: Mode, pub movers: &'a [(u8, Mesh)], pub fx: &'a FrameAir }

/// Light (R, a long) as a card and the part past 999 in 25ths (tenths of 1.0), at most 999.
pub fn split_hot<O: Ops>(o: &mut O, tb: &Tb, t: &V) -> (V, V) {
    let c = o.clamp(v(t), 0, 999);
    let over = o.lin(&[(v(t), None), (K(-999), None)]);
    let over = o.max(v(&over), K(0));
    // (over 25 by the printed table, held to 999)
    let oc = o.min(v(&over), K(24_999));
    (c, o.tab(tb.t("div25l"), v(&oc)))
}

/// x k / d (k a per-frame factor, d a constant).
pub fn scaled<O: Ops>(o: &mut O, x: &V, k: i64, d: i64) -> V { let t = o.lin(&[(v(x), Some(K(k)))]); o.div(v(&t), K(d), 0) }

/// A per-frame factor: a k / d.
pub fn fk<O: Ops>(o: &mut O, a: i64, k: i64, d: i64) -> i64 { one(scaled(o, &vec![a], k, d)) }

/// 1 where lo < x < hi.
pub fn within_v<O: Ops>(o: &mut O, x: &V, lo: i64, hi: i64) -> V { let a = o.lt(K(lo), v(x)); let b = o.lt(v(x), K(hi)); mulw(o, v(&a), v(&b), 0) }

impl<'a, O: Ops> Shading<O> for NightShade<'a> {
    fn kind(&mut self, o: &mut O, w: &World, _env: &Env, e: i64, _round: &SkyMap, kd: i64, px: &Px, out: &mut Out) {
        let n = px.len();
        if n == 0 || kd < 61 { return; }
        let alb = albedo(&w.sc, px);
        out.alb = alb.clone();
        let l = self.l;
        let f = l.f;
        let set = |out: &mut Out, o: &mut O, t: [V; 3]| { for c in 0..3 { let (a, b) = split_hot(o, &w.tb, &t[c]); out.emit[c] = a; out.hot[c] = b; } };
        o.stage(match kd { nkind::LED => "n_board", nkind::DSCREEN => "n_screens", nkind::STADSKIN => "n_skin", nkind::BHULL => "n_hull", nkind::BSCREEN => "n_bscreen", _ => "n_other" });
        match kd {
            nkind::LED => { let t = board(o, self, px, e); set(out, o, t); out.alb = konst(n, [20, 20, 20]); }
            nkind::TUBE => {
                // the canopies' tubes: 9 times their colour (alb thousandths: 2.25 R each)
                let k = fk(o, 2250, e, 1000);
                let t: [V; 3] = std::array::from_fn(|c| scaled(o, &alb[c], k, 1000));
                set(out, o, t);
            }
            nkind::NAV => {
                // the street lights' lamps and the drone's lights: 14 times their colour; the strobe 40 times, two
                // frames in twelve, else dark
                let code = par(&w.sc, px, 0);
                let strobe = within_v(o, &code, 150, 250);
                let flash = if f % 12 < 2 { 40 } else { 0 };
                let k = o.sel(&strobe, K(flash), K(14));
                let k = o.lin(&[(v(&k), Some(K(e)))]);
                let t: [V; 3] = std::array::from_fn(|c| { let t = o.lin(&[(v(&alb[c]), Some(v(&k)))]); o.div(v(&t), K(4000), 0) });
                set(out, o, t);
            }
            nkind::DRONE => {
                // its shell's sheen: the reflection's up part, 0.15
                let dn = dot_dn(o, &px.d, &px.n);
                let m = mulw(o, v(&dn), v(&px.n[1]), 2);
                let ry = o.lin(&[(v(&m), Some(K(-2))), (v(&px.d[1]), None)]);
                let ry = o.clamp(v(&ry), 0, 999);
                let k = fk(o, 375, e, 1000);
                let s = scaled(o, &ry, k, 10_000);
                out.spec = [s.clone(), s.clone(), s];
            }
            nkind::EMITX => {
                // the stadium's lit bands and canopy edge: 3 times their colour times their brightness (hundredths),
                // on the beat (down to 0.15 between)
                let p0 = par(&w.sc, px, 0);
                let pk = pulse(o, l.night, l.beat, 150);
                let k = fk(o, pk, e, 1000);
                let k = fk(o, k, 1000, 1333);
                let t: [V; 3] = std::array::from_fn(|c| { let t = o.lin(&[(v(&alb[c]), Some(v(&p0)))]); let t = o.div(v(&t), K(100), 0); let t = o.lin(&[(v(&t), Some(K(k)))]); o.div(v(&t), K(1000), 0) });
                set(out, o, t);
            }
            nkind::DSCREEN => { let t = screens(o, w, self, px, e); set(out, o, t); out.alb = zeros(n); }
            nkind::STADSKIN => {
                let t = skin(o, self, px, e);
                set(out, o, t);
                out.alb = std::array::from_fn(|c| o.pct(v(&alb[c]), K(30)));
            }
            nkind::BFRAME => {
                // the screen's frame glowing: 4 + 0.6 sin(0.4 f) times its colour
                let a = one(o.lin(&[(K(f), Some(K(63_662)))]));
                let a = one(o.turns(K(a)));
                let (s, _) = o.sincos(K(a));
                let k = one({ let t = o.lin(&[(K(s[0]), Some(K(600)))]); let t = o.div(v(&t), K(999), 0); o.lin(&[(v(&t), None), (K(4000), None)]) });
                let k = fk(o, k, e, 4000);
                let t: [V; 3] = std::array::from_fn(|c| scaled(o, &alb[c], k, 1000));
                set(out, o, t);
            }
            nkind::BHULL => hull(o, self, px, e, out),
            nkind::BSCREEN => { let t = airship_screen(o, self, px, e); set(out, o, t); out.alb = zeros(n); }
            _ => {}
        }
    }

    fn light(&mut self, o: &mut O, _w: &World, px: &Px, nn: &[V; 3], kinds: &[i64], tunnel: &V, solid: &[usize]) -> Option<[V; 3]> {
        if self.mode == Mode::Old || self.l.night < 50 { return None; }
        o.stage("n_lamps");
        let r = lamps(o, self, px, nn, kinds, tunnel, solid);
        o.stage("light");
        Some(r)
    }

    fn screen(&self) -> Option<&Screen> { Some(self.scr) }
}

// ---- the board ---------------------------------------------------------------------------------------------------

/// The big board's dots (TONIGHT: in orange over DEMOSPLASH 0x10 in amber), 7.25 cm apart as the departure's board,
/// the text the right way round from either side; each dot's share of the footprint and a lit dot's glow when the
/// footprint is wider than the pitch from the printed tables; 9 times its colour lit, 0.06 dark.
fn board<O: Ops>(o: &mut O, s: &NightShade, px: &Px, e: i64) -> [V; 3] {
    let tb = &s.nw.w.tb;
    let (x, y) = (&px.p[0], &px.p[1]);
    // across the board (cm) from its left as seen (from its right end when seen from +z), 16 cm in to the dots,
    // a pitch of padding: quarter centimetres
    let back = o.lt(v(&px.d[2]), K(0));
    let ul = o.lin(&[(v(x), Some(K(-1))), (K(-460 + 480 + BX), None)]);
    let ur = o.lin(&[(v(x), None), (K(460 + 480 - BX), None)]);
    let u = o.sel(&back, v(&ul), v(&ur));
    let uq = o.lin(&[(v(&u), Some(K(4))), (K(-4 * 16 + 29), None)]);
    let uq = o.clamp(v(&uq), 0, 29 * 130 - 1);
    let vq = o.lin(&[(v(y), Some(K(-4))), (K(4 * (425 + 290) + 29), None)]);
    let vq = o.clamp(v(&vq), 0, 29 * 42 - 1);
    let (col, fu) = o.divmod(v(&uq), K(29));
    let (row, fv) = o.divmod(v(&vq), K(29));
    let code = o.tab2(tb.t("nled_bits"), v(&row), v(&col));
    let lf = &px.lvl;
    let du = o.tab2(tb.t("led_dot"), v(lf), v(&fu));
    let dv = o.tab2(tb.t("led_dot"), v(lf), v(&fv));
    let dot = mulw(o, v(&du), v(&dv), 2);
    let lit = o.lt(K(0), v(&code));
    let fill = o.tab(tb.t("nled_fill"), v(lf));
    let fill = mulw(o, v(&fill), v(&lit), 0);
    let share = o.sum(&[(v(&dot), 1), (v(&fill), 1)], 0);
    std::array::from_fn(|c| {
        // R exposed by code (each times e / 1000, thousandths of R): dark (1, 0.62, 0.10) 0.06; orange
        // (1, 0.30, 0.06) 9; amber (1, 0.62, 0.10) 9
        let cols: V = vec![[15_000, 9300, 1500][c], [2_250_000, 675_000, 135_000][c], [2_250_000, 1_395_000, 225_000][c]];
        let t = o.div(v(&cols), K(1000), 0);
        let t = o.lin(&[(v(&t), Some(K(e)))]);
        let t = o.div(v(&t), K(1000), 0);
        let ce = o.tab(&Tab { name: "nled_col", data: t, w: 0 }, v(&code));
        let r = o.lin(&[(v(&ce), Some(v(&share)))]);
        o.div(v(&r), K(100), 0)
    })
}

// ---- the callouts' screens ------------------------------------------------------------------------------------------

/// The screens beside the line, under the canopies and on the towers: each pixel's place on its screen (across and
/// down, thousandths), the demo's footage there (its caption over the bottom quarter) through an LED grid, 2.2 times
/// its colour; or the sign's banner, 1.8 times.
fn screens<O: Ops>(o: &mut O, w: &World, s: &NightShade, px: &Px, e: i64) -> [V; 3] {
    let nw = s.nw;
    let n = px.len();
    let f = s.l.f;
    let key = par(&w.sc, px, 0);
    let mut out: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    for (i, sc) in nw.screens.iter().enumerate() {
        // the screen's pixels: the host's grouping by the material's number (100 i)
        let at: Vec<usize> = (0..n).filter(|&k| key[k] == 100 * i as i64).collect();
        if at.is_empty() { continue; }
        let g = |v: &V| -> V { at.iter().map(|&k| v[k]).collect() };
        let d: [V; 3] = std::array::from_fn(|c| o.lin(&[(v(&g(&px.p[c])), None), (K(sc[c]), Some(K(-1))), (K(if c == 0 { -BX } else { 0 }), None)]));
        // a = (d.r) / w + 0.5, b = 0.5 - (d.u) / h (thousandths, held to 0..999)
        let dr = o.lin(&[(v(&d[0]), Some(K(sc[3]))), (v(&d[1]), Some(K(sc[4]))), (v(&d[2]), Some(K(sc[5])))]);
        let a = o.div(v(&dr), K(sc[9]), 0);
        let a = o.lin(&[(v(&a), None), (K(500), None)]);
        let a = o.clamp(v(&a), 0, 999);
        let du = o.lin(&[(v(&d[0]), Some(K(sc[6]))), (v(&d[1]), Some(K(sc[7]))), (v(&d[2]), Some(K(sc[8])))]);
        let b = o.div(v(&du), K(sc[10]), 0);
        let b = o.lin(&[(v(&b), Some(K(-1))), (K(500), None)]);
        let b = o.clamp(v(&b), 0, 999);
        let demo = sc[11];
        let res: [V; 3] = if sc[12] == 0 {
            let fr = clip_frame(o, nw, demo, (f + 7 * i as i64).rem_euclid(60));
            clip_texels(o, nw, &fr, demo, &a, &b, 2200, e, true, true)
        } else {
            let t = Tab { name: "banner", data: nw.pics.get(&format!("banner_{demo}")).to_vec(), w: 768 };
            let row = scaled(o, &b, 96, 1000);
            let col = scaled(o, &a, 256, 1000);
            // 1.8 times: thousandths of 1.0 to R exposed, 0.45 e / 1000
            let k = fk(o, 450, e, 1000);
            std::array::from_fn(|c| {
                let ci = o.lin(&[(v(&col), Some(K(3))), (K(c as i64), None)]);
                let tx = o.tab2(&t, v(&row), v(&ci));
                scaled(o, &tx, k, 1000)
            })
        };
        for c in 0..3 { for (j, &k) in at.iter().enumerate() { out[c][k] = res[c][j]; } }
    }
    out
}

/// A demo's picture at its frame k (0..59), 120 rows of 160 pixels' RGB (0..255): a clip's footage, or a parody's
/// stand-in worked out here (its 80 x 60 cells, each laid 2 x 2 by the host).
fn clip_frame<O: Ops>(o: &mut O, nw: &NightWorld, demo: i64, k: i64) -> Tab {
    let kind = nw.pics.get("si_kind")[demo as usize];
    if kind < 0 { return nw.clips.frame(demo, k); }
    let cells = standin(o, nw, demo, kind, k);
    let mut data = vec![0; 160 * 120 * 3];
    for y in 0..120 { for x in 0..160 { let c = (y / 2) * 80 + x / 2; for ch in 0..3 { data[(y * 160 + x) * 3 + ch] = cells[ch][c]; } } }
    Tab { name: "clip", data, w: 480 }
}

/// A parody's stand-in (callouts.py's standin_cells): its frame k of 60, 80 x 60 cells, RGB 0..255, by the printed
/// tables (si_*): a plasma of four waves through its palette; a tunnel of checks rushing at us, darker at its heart; a
/// rotozoomed check turning and breathing; stars flying at us; rings running out from the middle. Each parody its own
/// colours and time (137 places into its palette for each entry).
fn standin<O: Ops>(o: &mut O, nw: &NightWorld, demo: i64, kind: i64, k: i64) -> [V; 3] {
    let tb = &nw.w.tb;
    let (sw, sh) = (80usize, 60usize);
    let n = sw * sh;
    let off = 137 * demo;
    let off1000 = one(o.wrap(K(off), 1));
    let pal = tb.t("si_pal");
    let pc = |o: &mut O, row: i64| -> [i64; 3] { std::array::from_fn(|c| one(o.tab2(pal, K(kind * 6000 + row), K(c as i64)))) };
    let cell = |t: &str| -> V { tb.t(t).data.clone() };
    let colours = |o: &mut O, idx: &V| -> [V; 3] { std::array::from_fn(|c| o.tab2(pal, v(idx), K(c as i64))) };
    match kind {
        0 => {
            let sin = tb.t("si_sin");
            let xs: V = (0..sw as i64).collect();
            let ys: V = (0..sh as i64).collect();
            let ds: V = (0..(sw + sh - 1) as i64).collect();
            let ia = o.lin(&[(v(&xs), Some(K(12))), (K(k), Some(K(50)))]);
            let a = o.tab(sin, v(&ia));
            let ib = o.lin(&[(v(&ys), Some(K(17))), (K(k), Some(K(-50))), (K(2950), None)]);
            let b = o.tab(sin, v(&ib));
            let ic = o.lin(&[(v(&ds), Some(K(9))), (K(k), Some(K(100)))]);
            let c = o.tab(sin, v(&ic));
            let c6 = one(o.lin(&[(K(k), Some(K(-100))), (K(6000), None)]));
            let id = o.sum(&[(v(&cell("si_rad3")), 1)], c6);
            let d = o.tab(sin, v(&id));
            let c0 = one({ let t = o.lin(&[(K(k), Some(K(50))), (K(off), None)]); o.wrap(v(&t), 1) });
            let c0 = one(o.lin(&[(K(c0), None), (K(kind * 6000), None)]));
            let ap: V = (0..n).map(|q| a[q % sw]).collect();
            let bp: V = (0..n).map(|q| b[q / sw]).collect();
            let cp: V = (0..n).map(|q| c[q % sw + q / sw]).collect();
            let idx = o.sum(&[(v(&ap), 1), (v(&bp), 1), (v(&cp), 1), (v(&d), 1)], c0);
            colours(o, &idx)
        }
        1 => {
            let pu = one({ let t = o.lin(&[(K(k), Some(K(40)))]); o.wrap(v(&t), 1) });
            let pv = one({ let t = o.lin(&[(K(k), Some(K(10))), (K(off), None)]); o.wrap(v(&t), 1) });
            let iu = o.sum(&[(v(&cell("si_tunu")), 1)], pu);
            let cu = o.tab(tb.t("si_cu"), v(&iu));
            let iv = o.sum(&[(v(&cell("si_ang")), 1)], pv);
            let cv = o.tab(tb.t("si_cv"), v(&iv));
            let chk = o.sum(&[(v(&cu), 1), (v(&cv), 1)], 0);
            let is1 = within_v(o, &chk, 0, 2);
            let (pa, pb) = (pc(o, 0), pc(o, 500));
            let br = cell("si_tunb");
            std::array::from_fn(|c| { let t = o.sel(&is1, K(pa[c]), K(pb[c])); o.pct(v(&t), v(&br)) })
        }
        2 => {
            let a = one({ let t = o.lin(&[(K(k), Some(K(16_667))), (K(off1000), Some(K(1000)))]); o.turns(v(&t)) });
            let (sa, ca) = o.sincos(K(a));
            let z = one({ let t = o.lin(&[(K(k), Some(K(33_333)))]); o.turns(v(&t)) });
            let (sz, _) = o.sincos(K(z));
            let ks = one({ let t = o.lin(&[(K(sz[0]), Some(K(300)))]); let t = o.div(v(&t), K(999), 0); o.lin(&[(v(&t), None), (K(600), None)]) });
            let ka = one({ let t = o.lin(&[(K(ks), Some(K(ca[0])))]); o.div(v(&t), K(999), 0) });
            let kb = one({ let t = o.lin(&[(K(ks), Some(K(sa[0])))]); o.div(v(&t), K(999), 0) });
            let xs: V = (0..sw as i64).map(|x| x - 40).collect();
            let ys: V = (0..sh as i64).map(|y| y - 30).collect();
            let ux = o.lin(&[(v(&xs), Some(K(ka)))]);
            let uy = o.lin(&[(v(&ys), Some(K(-kb))), (K(80_000), None)]);
            let vx = o.lin(&[(v(&xs), Some(K(kb)))]);
            let vy = o.lin(&[(v(&ys), Some(K(ka))), (K(80_000), None)]);
            let g = |a: &V, b: &V| -> (V, V) { ((0..n).map(|q| a[q % sw]).collect(), (0..n).map(|q| b[q / sw]).collect()) };
            let (p1, p2) = g(&ux, &uy);
            let u = o.sum(&[(v(&p1), 1), (v(&p2), 1)], 0);
            let (p1, p2) = g(&vx, &vy);
            let w_ = o.sum(&[(v(&p1), 1), (v(&p2), 1)], 0);
            let cu = o.tab(tb.t("si_ck"), v(&u));
            let cv = o.tab(tb.t("si_ck"), v(&w_));
            let chk = o.sum(&[(v(&cu), 1), (v(&cv), 1)], 0);
            let is1 = within_v(o, &chk, 0, 2);
            let (pa, pb) = (pc(o, 0), pc(o, 500));
            std::array::from_fn(|c| o.sel(&is1, K(pa[c]), K(pb[c])))
        }
        3 => {
            let st = tb.t("si_stars").data.clone();
            let m = st.len() / 3;
            let g = |j: usize| -> V { (0..m).map(|q| st[3 * q + j]).collect() };
            let (_, o100) = o.divmod(K(off), K(100));
            let c = one(o.lin(&[(K(k), Some(K(-15))), (K(o100[0]), Some(K(7))), (K(900), None)]));
            let t = o.sum(&[(v(&g(2)), 1)], c);
            let (_, r) = o.divmod(v(&t), K(900));
            let z = o.sum(&[(v(&r), 1)], 50);
            let x = { let t = o.lin(&[(v(&g(0)), Some(K(40)))]); let t = o.div(v(&t), v(&z), 0); o.sum(&[(v(&t), 1)], 40) };
            let y = { let t = o.lin(&[(v(&g(1)), Some(K(30)))]); let t = o.div(v(&t), v(&z), 0); o.sum(&[(v(&t), 1)], 30) };
            let br = o.tab(tb.t("si_brz"), v(&z));
            let mut out = vec![0; n];
            for q in 0..m { if (0..sw as i64).contains(&x[q]) && (0..sh as i64).contains(&y[q]) { out[y[q] as usize * sw + x[q] as usize] = br[q]; } }
            [out.clone(), out.clone(), out]
        }
        _ => {
            let c = one(o.lin(&[(K(k), Some(K(-50))), (K(off1000), None), (K(3000 + kind * 6000), None)]));
            let idx = o.sum(&[(v(&cell("si_rad4")), 1)], c);
            colours(o, &idx)
        }
    }
}

/// A demo's footage at places on its picture (across a, down b: thousandths), its caption laid over the bottom
/// quarter (the footage a quarter as bright there under the caption's ground, the caption 0.4 to 1 by its ink),
/// through the LED grid if asked (0.75, full in each LED's middle 0.76), times k thousandths of its colour, R
/// exposed.
fn clip_texels<O: Ops>(o: &mut O, nw: &NightWorld, fr: &Tab, demo: i64, a: &V, b: &V, k: i64, e: i64, caption: bool, grid: bool) -> [V; 3] {
    let n = a.len();
    let yb = o.lin(&[(v(b), Some(K(120)))]);
    let row = o.div(v(&yb), K(1000), 0);
    let gy = o.lin(&[(v(&yb), None), (v(&row), Some(K(-1000)))]);
    let xa = o.lin(&[(v(a), Some(K(160)))]);
    let col = o.div(v(&xa), K(1000), 0);
    let gx = o.lin(&[(v(&xa), None), (v(&col), Some(K(-1000)))]);
    let mut tex: [V; 3] = std::array::from_fn(|c| { let ci = o.lin(&[(v(&col), Some(K(3))), (K(c as i64), None)]); o.tab2(fr, v(&row), v(&ci)) });
    // the caption: rows 90..119
    let capr: Vec<usize> = if caption { (0..n).filter(|&i| row[i] >= 90).collect() } else { vec![] };
    if !capr.is_empty() {
        let cap = Tab { name: "cap", data: nw.pics.get(&format!("cap_{demo}")).to_vec(), w: 480 };
        let capm = Tab { name: "capm", data: nw.pics.get(&format!("capm_{demo}")).to_vec(), w: 160 };
        let g = |v: &V| -> V { capr.iter().map(|&i| v[i]).collect() };
        let r = o.lin(&[(v(&g(&row)), None), (K(-90), None)]);
        let cl = g(&col);
        let m = o.tab2(&capm, v(&r), v(&cl));
        // the footage times 0.25 (1 - m), the caption (thousandths, to 0..255) times 0.4 + 0.6 m (m hundredths)
        let keep = o.lin(&[(v(&m), Some(K(-25))), (K(2500), None)]);
        let ink = o.lin(&[(v(&m), Some(K(60))), (K(4000), None)]);
        for c in 0..3 {
            let ci = o.lin(&[(v(&cl), Some(K(3))), (K(c as i64), None)]);
            let cp = o.tab2(&cap, v(&r), v(&ci));
            let t0 = o.lin(&[(v(&g(&tex[c])), Some(v(&keep)))]);
            let t0 = o.div(v(&t0), K(1000), 0);
            let t0 = o.div(v(&t0), K(10), 0);
            let t1 = o.lin(&[(v(&cp), Some(v(&ink)))]);
            let t1 = o.div(v(&t1), K(1000), 0);
            let t1 = o.div(v(&t1), K(10), 0);
            let t1 = o.lin(&[(v(&t1), Some(K(255)))]);
            let t1 = o.div(v(&t1), K(1000), 0);
            let t = o.sum(&[(v(&t0), 1), (v(&t1), 1)], 0);
            for (j, &i) in capr.iter().enumerate() { tex[c][i] = t[j]; }
        }
    }
    let gk = if grid {
        // the LED grid: full inside each LED's middle (0.12..0.88 each way), else 0.75
        let ix = within_v(o, &gx, 119, 880);
        let iy = within_v(o, &gy, 119, 880);
        let both = mulw(o, v(&ix), v(&iy), 0);
        o.sel(&both, K(1000), K(750))
    } else { vec![1000; n] };
    // 0..255 times the grid times k, R exposed: x 250 / 255 (by 1020) times e / 1000 (the factor for the frame)
    let kk = fk(o, k, e, 1000);
    let kk = fk(o, kk, 1000, 1020);
    std::array::from_fn(|c| {
        let t = if grid { let t = o.lin(&[(v(&tex[c]), Some(v(&gk)))]); o.div(v(&t), K(1000), 0) } else { tex[c].clone() };
        let t = o.lin(&[(v(&t), Some(K(kk)))]);
        o.div(v(&t), K(1000), 0)
    })
}

// ---- the stadium's skin -------------------------------------------------------------------------------------------

/// The bowl's outside as an LED skin: every callout in one long line of the VGA font (the printed marquee), in two
/// rows scrolling opposite ways, 3 LED cells a frame; each LED a dot (0.4 between); a rainbow round the bowl drifting
/// with the frames; 5.5 times, pulsing on the beat (down to 0.6).
fn skin<O: Ops>(o: &mut O, s: &NightShade, px: &Px, e: i64) -> [V; 3] {
    let nw = s.nw;
    let tb = &nw.w.tb;
    let f = s.l.f;
    let n = px.len();
    let ac = nw.aud_c;
    // the stadium's frame: X across, Y toward the stage, Z up (cm)
    let xx = o.lin(&[(v(&px.p[0]), None), (K(-BX), None), (K(ac[0]), Some(K(-1)))]);
    let yy = o.lin(&[(v(&px.p[2]), None), (K(ac[2]), Some(K(-1)))]);
    let zz = o.lin(&[(v(&px.p[1]), None), (K(ac[1]), Some(K(-1)))]);
    let u = nrm3(o, &xx, &yy, &vec![0; n]);
    let ang = azimuth(o, tb, &u[1], &u[0]);
    let mq = tb.t("marquee");
    let lm = mq.w as i64;
    let mut img = vec![0; n];
    for (y0, y1, dir) in [(1100, 1940, 1i64), (2220, 3060, -1)] {
        // the row (thousandths of rows from the top), the column (thousandths of cells round the wall) scrolling
        let rk = o.lin(&[(v(&zz), Some(K(-16_000))), (K(16_000 * y1), None)]);
        let rk = o.div(v(&rk), K(y1 - y0), 0);
        let ok = within_v(o, &rk, -1, 16_000);
        let rkc = o.clamp(v(&rk), 0, 15_999);
        let r = o.div(v(&rkc), K(1000), 0);
        let rf = o.lin(&[(v(&rkc), None), (v(&r), Some(K(-1000)))]);
        let ck = o.lin(&[(v(&ang), Some(K(238))), (K(f), Some(K(3000 * dir))), (K(1000 * lm), None)]);
        let c = o.div(v(&ck), K(1000), 0);
        let cf = o.lin(&[(v(&ck), None), (v(&c), Some(K(-1000)))]);
        let q = o.div(v(&c), K(lm), 0);
        let c = o.lin(&[(v(&c), None), (v(&q), Some(K(-lm)))]);
        let lit = o.tab2(mq, v(&r), v(&c));
        let d1 = within_v(o, &rf, 150, 850);
        let d2 = within_v(o, &cf, 150, 850);
        let dot = mulw(o, v(&d1), v(&d2), 0);
        let k = o.sel(&dot, K(1000), K(400));
        let k = mulw(o, v(&k), v(&lit), 0);
        let k = mulw(o, v(&k), v(&ok), 0);
        img = o.max(v(&img), v(&k));
    }
    // the rainbow: 0.5 + 0.5 sin(3 ang + 0.05 f + 2.1 c)
    let pk = pulse(o, s.l.night, s.l.beat, 600);
    let kk = fk(o, 1375, pk, 1000);
    let kk = fk(o, kk, e, 1000);
    std::array::from_fn(|c| {
        let a = o.lin(&[(v(&ang), Some(K(477))), (K(f), Some(K(7958))), (K(334_225 * c as i64), None)]);
        let a = o.turns(v(&a));
        let (sn, _) = o.sincos(v(&a));
        let h = o.lin(&[(v(&sn), Some(K(500))), (K(499_500), None)]);
        let h = o.div(v(&h), K(999), 0);
        let t = o.lin(&[(v(&h), Some(v(&img)))]);
        let t = o.div(v(&t), K(1000), 0);
        scaled(o, &t, kk, 1000)
    })
}

// ---- the airship ---------------------------------------------------------------------------------------------------

/// The airship's hull: dark plates (their seams every 3.1 m along and 2.3 m round, filtered by the footprint),
/// grime, lit faintly from below by the town and the stadium; the stadium's eight up-lights' pools sweeping over it
/// (on the beat); a dull rim. Its albedo's factor by the grime and the seams (a table worked out at the start), the
/// products by logarithms; the pools only where a place is within 25 m of a spot each way, their parts by the frame's
/// tables of the spot's way.
fn hull<O: Ops>(o: &mut O, s: &NightShade, px: &Px, e: i64, out: &mut Out) {
    let nw = s.nw;
    let tb = &nw.w.tb;
    let n = px.len();
    let lf = &px.lvl;
    let c = s.fx.blimp;
    let p: [V; 3] = std::array::from_fn(|i| o.sum(&[(v(&px.p[i]), 1)], -c[i]));
    let (_, ph1) = o.phase(v(&p[0]), 323, 5);
    let yz = o.sum(&[(v(&p[1]), 1), (v(&p[2]), 1)], 0);
    let (_, ph2) = o.phase(v(&yz), 435, 5);
    let s1 = o.tab2(tb.t("bh_s31"), v(lf), v(&ph1));
    let s2 = o.tab2(tb.t("bh_s23"), v(lf), v(&ph2));
    let plate = o.sum(&[(v(&s1), 1), (v(&s2), 1)], 0);
    let plate = o.min(v(&plate), K(100));
    let nz = noise(o, tb, "n_bhull", &p[0], &yz, 10);
    let f = o.tab2(&nw.hull_f, v(&nz), v(&plate));
    // the albedo and its mean by the material's tables of f (the host picks each pixel's material's)
    let mi: Vec<usize> = px.mat.iter().map(|&m| nw.hull_mats.iter().position(|&h| h == m).unwrap_or(0)).collect();
    let mut a: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    let mut mean = vec![0; n];
    for (k, _) in nw.hull_mats.iter().enumerate() {
        let at: Vec<usize> = (0..n).filter(|&i| mi[i] == k).collect();
        if at.is_empty() { continue; }
        let fv: V = at.iter().map(|&i| f[i]).collect();
        for ch in 0..3 { let t = o.tab(&nw.hull_a[k][ch], v(&fv)); for (j, &i) in at.iter().enumerate() { a[ch][i] = t[j]; } }
        let t = o.tab(&nw.hull_mean[k], v(&fv));
        for (j, &i) in at.iter().enumerate() { mean[i] = t[j]; }
    }
    out.alb = a.clone();
    let la: [V; 3] = std::array::from_fn(|ch| { let t = o.max(v(&a[ch]), K(1)); o.tab(tb.t("ln6"), v(&t)) });
    // from below: (1.4, 0.8, 1.0) 0.35 times -n.y; in R exposed: a under K / 1e6 by logarithms
    let under = o.tabo(tb.t("hull_under"), 127, v(&px.n[1]));
    let lu = o.tab(tb.t("ln6"), v(&under));
    let mut em: [V; 3] = std::array::from_fn(|ch| {
        let k = fk(o, [1225, 700, 875][ch], e, 10);
        let lk = one(o.tab(tb.t("ln6"), K(k)));
        let t = o.sum(&[(v(&la[ch]), 1), (v(&lu), 1)], 0);
        let kk = { let c = one(o.sum(&[(K(lk), -1)], 3 * 69_078 - 46_052)); o.ldiff(&t, &vec![c; n], -50, 115) };
        let r = o.tabo(tb.t("exp6"), 50_000, v(&kk));
        o.div(v(&r), K(100), 0)
    });
    // the up-lights' pools: e^-(r^2 / 30 m^2) of the place's distance from each beam's axis at its spot, facing it
    let mut pools = vec![0; n];
    let mut lp: Vec<Vec<i64>> = vec![vec![]; n];
    for k in 0..s.fx.up_spot.len() {
        let q = s.fx.up_spot[k];
        let rel: [V; 3] = std::array::from_fn(|i| o.sum(&[(v(&px.p[i]), 1)], -q[i]));
        // within 25 m of the spot each way: the host's picking by the places punched
        let near: Vec<usize> = (0..n).filter(|&i| (0..3).all(|c_| rel[c_][i].abs() < 2500)).collect();
        if near.is_empty() { continue; }
        let g = |v_: &V| -> V { near.iter().map(|&i| v_[i]).collect() };
        let rd: [V; 3] = std::array::from_fn(|i| o.div(v(&g(&rel[i])), K(10), 0));
        let tparts: Vec<V> = (0..3).map(|i| o.tabo(&s.fx.up_t[k][i], 250, v(&rd[i]))).collect();
        let t = o.sum(&[(v(&tparts[0]), 1), (v(&tparts[1]), 1), (v(&tparts[2]), 1)], 0);
        let ac: [V; 3] = std::array::from_fn(|i| { let u = o.tabo(&s.fx.up_u[k][i], 433, v(&t)); let r = o.sum(&[(v(&rd[i]), 1), (v(&u), -1)], 0); o.clamp(v(&r), -9999, 9999) });
        let sq: [V; 3] = std::array::from_fn(|i| o.tabo(tb.t("sq9999"), 9999, v(&ac[i])));
        let r2 = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
        let r2 = o.min(v(&r2), K(199_999));
        let pool = o.tab(tb.t("hull_pool"), v(&r2));
        let fparts: Vec<V> = (0..3).map(|i| o.tabo(&s.fx.up_f[k][i], 127, v(&g(&px.n[i])))).collect();
        let fc = o.sum(&[(v(&fparts[0]), 1), (v(&fparts[1]), 1), (v(&fparts[2]), 1)], 0);
        let fc = o.clamp(v(&fc), 1, 1000);
        // pool times facing, by logarithms (thousandths)
        let pool1 = o.max(v(&pool), K(1));
        let lpo = o.tab(tb.t("ln6"), v(&pool1));
        let lfc = o.tab(tb.t("ln6"), v(&fc));
        let ll = o.sum(&[(v(&lpo), 1), (v(&lfc), 1)], -69_078);
        for (j, &i) in near.iter().enumerate() { if pool[j] > 0 { lp[i].push(ll[j]); } }
    }
    let lit: Vec<usize> = (0..n).filter(|&i| !lp[i].is_empty()).collect();
    if !lit.is_empty() {
        let terms: Vec<Vec<i64>> = lit.iter().map(|&i| lp[i].clone()).collect();
        // each term back from its logarithm, summed (thousandths)
        let flat: V = terms.iter().flatten().copied().collect();
        let zz = vec![0; flat.len()];
        let kk = o.ldiff(&flat, &zz, -50, 115);
        let ex = o.tabo(tb.t("exp6"), 50_000, v(&kk));
        let mut at = 0;
        let lists: Vec<Vec<i64>> = terms.iter().map(|t_| { let l = ex[at..at + t_.len()].to_vec(); at += t_.len(); l }).collect();
        let ps = group_sum(o, &lists);
        for (j, &i) in lit.iter().enumerate() { pools[i] = ps[j]; }
        // times 0.6 + 0.4 the beat, the albedo's mean, 18 (0.9, 0.95, 1.0): R exposed, by logarithms
        let bk = one(o.lin(&[(K(s.l.beat), Some(K(400))), (K(600_000), None)]));
        let bk = one(o.div(K(bk), K(1000), 0));
        let g = |v_: &V| -> V { lit.iter().map(|&i| v_[i]).collect() };
        let pl = { let t = o.max(v(&g(&pools)), K(1)); let t = o.min(v(&t), K(999_999)); o.tab(tb.t("ln6"), v(&t)) };
        let lm = { let t = o.max(v(&g(&mean)), K(1)); o.tab(tb.t("ln6"), v(&t)) };
        let pm = o.sum(&[(v(&pl), 1), (v(&lm), 1)], 0);
        for ch in 0..3 {
            let k = fk(o, [4050, 4275, 4500][ch], e, 1000);
            let k = fk(o, k, bk, 10);
            let k = one(o.max(K(k), K(1)));
            let lk = one(o.tab(tb.t("ln6"), K(k)));
            let kk = { let c = one(o.sum(&[(K(lk), -1)], 2 * 69_078)); o.ldiff(&pm, &vec![c; lit.len()], -50, 115) };
            let r = o.tabo(tb.t("exp6"), 50_000, v(&kk));
            let r = o.div(v(&r), K(100), 0);
            let cur = g(&em[ch]);
            let t = o.sum(&[(v(&cur), 1), (v(&r), 1)], 0);
            for (j, &i) in lit.iter().enumerate() { em[ch][i] = t[j]; }
        }
    }
    for ch in 0..3 { let (a_, h_) = split_hot(o, &s.nw.w.tb, &em[ch]); out.emit[ch] = a_; out.hot[ch] = h_; }
    // the dull rim: 0.4 alb (1 - |d.n|)^3, by logarithms
    let rest = o.sum(&[(v(&px.adn), -1)], 1000);
    let cube = o.tab(tb.t("cube"), v(&rest));
    let lcu = { let t = o.max(v(&cube), K(1)); o.tab(tb.t("ln6"), v(&t)) };
    let k = fk(o, 100, e, 1000);
    let k = one(o.max(K(k), K(1)));
    let lk = one(o.tab(tb.t("ln6"), K(k)));
    out.spec = std::array::from_fn(|ch| {
        let t = o.sum(&[(v(&la[ch]), 1), (v(&lcu), 1)], 0);
        let kk = { let c = one(o.sum(&[(K(lk), -1)], 2 * 69_078 - 46_052)); o.ldiff(&t, &vec![c; n], -50, 115) };
        let r = o.tabo(tb.t("exp6"), 50_000, v(&kk));
        let r = o.div(v(&r), K(100), 0);
        let z = o.lt(K(0), v(&cube));
        o.sel(&z, v(&r), K(0))
    });
}

/// The airship's screen at its pixels: its place on the picture (a across, b up: thousandths) in the model's frame;
/// the volume knob while we come up to it; DEMOSPLASH 0x10 over the laser grid (its first slot of three, before we
/// stop, and while we read it); else a demo's footage, 2.4 times; scan lines.
fn airship_screen<O: Ops>(o: &mut O, s: &NightShade, px: &Px, e: i64) -> [V; 3] {
    let nw = s.nw;
    let tb = &nw.w.tb;
    let f = s.l.f;
    let c = s.fx.blimp;
    // P - centre (cm), then across: a = ((P - C).x / 1.4 + 1250) / 3.3, and up its tilted face: b = ((P - C).y 0.966 -
    // (P - C).z 0.257) / 1.4 / 1.86 + 57.6 (thousandths of the picture; hundred-thousandths before the cards move down)
    let d: [V; 3] = std::array::from_fn(|i| o.lin(&[(v(&px.p[i]), None), (K(c[i]), Some(K(-1)))]));
    let a = o.lin(&[(v(&d[0]), Some(K(21_645))), (K(37_878_788), None)]);
    let a = o.div(v(&a), K(1000), 0);
    let a = o.div(v(&a), K(100), 0);
    let a = o.clamp(v(&a), 0, 999);
    let b = o.lin(&[(v(&d[1]), Some(K(37_097))), (v(&d[2]), Some(K(-9_869))), (K(5_760_800), None)]);
    let b = o.div(v(&b), K(1000), 0);
    let b = o.div(v(&b), K(100), 0);
    let b = o.clamp(v(&b), 0, 999);
    let pa = &nw.path;
    let (vol_on, vol_off) = (pa.f_read - 46, pa.f_read + 20);
    let seg = ((f + 6) / 30) % 3;
    let reading = f >= F_CHASE && { let d = { let t = kl(o, &[(f, Some(10)), (-10 * F_CHASE, None)]); cam_dist(o, pa, t) }; let x = one(o.lin(&[(K(d), None), (K(pa.d_read), Some(K(-1)))])); let mx = one(o.lin(&[(K(x), Some(K(-1)))])); let ax = one(o.max(K(x), K(mx))); one(o.lt(K(ax), K(60_000))) == 1 };
    // (VOL 11 held through the join onto the dive, the orbit skipped)
    let held = vol_off + CUT_N <= f && f < vol_off + CUT_N + JOIN;
    let col: [V; 3] = if (vol_on <= f && f < vol_off) || held { vol_screen(o, s, &a, &b) }
        else if seg == 0 || f < F_STOP || reading { blimp_screen(o, s, &a, &b) }
        else {
            let i = ((f + 6) / 90 * 2 + seg) % 63;
            let fr = clip_frame(o, nw, i, f.rem_euclid(60));
            // nearest: row (1 - b) 120, column a 160; 2.4 times (as R per 1.0: by clip_texels at k 2400)
            let bb = o.lin(&[(v(&b), Some(K(-1))), (K(999), None)]);
            let t = clip_texels(o, nw, &fr, i, &a, &bb, 2400, 1000, true, false);
            // (back to 1.0 = 1000 for the scan: R over 250)
            std::array::from_fn(|ch| o.lin(&[(v(&t[ch]), Some(K(4)))]))
        };
    // the picture's lines (0.85 + 0.15 sin(560 pi b), the printed table) times 250 e / 1e6 (to R exposed), the frame's
    // table by b; a colour (thousandths of 1.0) times that
    let k = fk(o, 250, e, 1000);
    let rows: V = (0..1000).collect();
    let sc = o.tab(tb.t("bs_scan"), v(&rows));
    let ks = Tab { name: "bs_ks", data: scaled(o, &sc, k, 1000), w: 0 };
    let kb = o.tab(&ks, v(&b));
    std::array::from_fn(|ch| { let t = o.lin(&[(v(&col[ch]), Some(v(&kb)))]); o.div(v(&t), K(1000), 0) })
}

/// The airship's own picture (thousandths of 1.0): a magenta glow over a horizon at 0.62, an orange laser grid racing
/// toward us under it, DEMOSPLASH 0x10 in blue over it all with a shadow (the printed words). What depends on a or b
/// alone is printed (bs_*), the along lines' by b worked out for the frame.
fn blimp_screen<O: Ops>(o: &mut O, s: &NightShade, a: &V, b: &V) -> [V; 3] {
    let nw = s.nw;
    let tb = &nw.w.tb;
    let f = s.l.f;
    let n = a.len();
    let above = o.lt(K(620), v(b));
    let mut col: [V; 3] = [vec![0; n], vec![0; n], vec![0; n]];
    let up: Vec<usize> = (0..n).filter(|&i| above[i] == 1).collect();
    let dn: Vec<usize> = (0..n).filter(|&i| above[i] == 0).collect();
    if !up.is_empty() {
        let g = |v: &V| -> V { up.iter().map(|&i| v[i]).collect() };
        let (ba, aa) = (g(b), g(a));
        // the sky's (1 - sky) part by b; the glow by e^-(xb + xa)
        let xb = o.tab(tb.t("bs_xb"), v(&ba));
        let xa = o.tab(tb.t("bs_xa"), v(&aa));
        let x = o.sum(&[(v(&xb), 1), (v(&xa), 1)], 0);
        let x = o.min(v(&x), K(19_999));
        for c in 0..3 {
            let u = o.tab2(tb.t("bs_up"), K(c as i64), v(&ba));
            let gl = o.tab2(tb.t("bs_glow"), K(c as i64), v(&x));
            let t = o.sum(&[(v(&u), 1), (v(&gl), 1)], 0);
            for (j, &i) in up.iter().enumerate() { col[c][i] = t[j]; }
        }
    }
    if !dn.is_empty() {
        let g = |v: &V| -> V { dn.iter().map(|&i| v[i]).collect() };
        let (ba, aa) = (g(b), g(a));
        let half = |o: &mut O, x: &V| -> V { let t = o.lin(&[(v(x), None), (K(-500), None)]); let m = o.lin(&[(v(&t), Some(K(-1)))]); o.max(v(&t), v(&m)) };
        // the along lines, the frame's table by b (0 .. 620): fz = |((1.2 Zg - 0.12 f) mod 1) - 0.5| (in
        // ten-thousandths, a whole number of periods added to keep it above 0), smooth(wz, 0.5, fz)
        let rows: V = (0..=620).collect();
        let zg = o.tab(tb.t("bs_zg"), v(&rows));
        let fzr = o.lin(&[(v(&zg), Some(K(12))), (K(f), Some(K(-1200))), (K(100_000_000), None)]);
        let q = o.div(v(&fzr), K(10_000), 0);
        let fzr = o.lin(&[(v(&fzr), None), (v(&q), Some(K(-10_000)))]);
        let fz = o.div(v(&fzr), K(10), 0);
        let fz = half(o, &fz);
        let wz = o.tab(tb.t("bs_wz"), v(&rows));
        let den = o.lin(&[(v(&wz), Some(K(-1))), (K(500), None)]);
        let den = o.max(v(&den), K(1));
        let tz = o.lin(&[(v(&fz), Some(K(1000))), (v(&wz), Some(K(-1000)))]);
        let tz = o.div(v(&tz), v(&den), 0);
        let tz = o.clamp(v(&tz), 0, 1000);
        let lzt = Tab { name: "bs_lz", data: ease_t(o, &tz), w: 0 };
        let lz = o.tab(&lzt, v(&ba));
        // the cross lines: Xg = (a - 0.5) Zg 3, fx = |(Xg mod 1) - 0.5|, smooth(0.465, 0.5, fx) clip(1.5 / Zg, 0, 1)
        let zgp = o.tab(tb.t("bs_zg"), v(&ba));
        let xg = o.lin(&[(v(&aa), Some(K(3))), (K(-1500), None)]);
        let xg = o.lin(&[(v(&xg), Some(v(&zgp)))]);
        let xg = o.div(v(&xg), K(1000), 0);
        let xgo = o.lin(&[(v(&xg), None), (K(100_000_000), None)]);
        let fx = modk(o, &xgo);
        let fx = half(o, &fx);
        let mut lx = o.tab(tb.t("bs_lx"), v(&fx));
        let cx = o.tab(tb.t("bs_cx"), v(&ba));
        let part = o.lt(v(&cx), K(1000));
        let pp: Vec<usize> = (0..dn.len()).filter(|&j| part[j] == 1 && lx[j] > 0).collect();
        if !pp.is_empty() {
            let x1: V = pp.iter().map(|&j| lx[j]).collect();
            let x2: V = pp.iter().map(|&j| cx[j]).collect();
            let t = o.lin(&[(v(&x1), Some(v(&x2)))]);
            let t = o.div(v(&t), K(1000), 0);
            for (q, &j) in pp.iter().enumerate() { lx[j] = t[q]; }
        }
        let line = o.max(v(&lz), v(&lx));
        // the colour by the line, faded at the horizon
        let fade = o.tab(tb.t("bs_fade"), v(&ba));
        let fd = o.lt(v(&fade), K(1000));
        let ff: Vec<usize> = (0..dn.len()).filter(|&j| fd[j] == 1).collect();
        for c in 0..3 {
            let mut t = o.tab2(tb.t("bs_line"), K(c as i64), v(&line));
            if !ff.is_empty() {
                let x1: V = ff.iter().map(|&j| t[j]).collect();
                let x2: V = ff.iter().map(|&j| fade[j]).collect();
                let r = o.lin(&[(v(&x1), Some(v(&x2)))]);
                let r = o.div(v(&r), K(1000), 0);
                for (q, &j) in ff.iter().enumerate() { t[j] = r[q]; }
            }
            for (j, &i) in dn.iter().enumerate() { col[c][i] = t[j]; }
        }
    }
    // the words and their shadow (6 texels up and left), where they cover: col (1 - 0.85 max) + the words' colour
    let text = tb.t("blimp_text");
    let iy = o.tab(tb.t("bs_iy"), v(b));
    let ix = o.tab(tb.t("bs_ix"), v(a));
    let tm = o.tab2(text, v(&iy), v(&ix));
    let iy6 = o.tab(tb.t("bs_iy6"), v(b));
    let ix6 = o.tab(tb.t("bs_ix6"), v(a));
    let sh = o.tab2(text, v(&iy6), v(&ix6));
    let m = o.max(v(&tm), v(&sh));
    let cov = o.lt(K(0), v(&m));
    let on: Vec<usize> = (0..n).filter(|&i| cov[i] == 1).collect();
    if !on.is_empty() {
        let g = |v: &V| -> V { on.iter().map(|&i| v[i]).collect() };
        let keep = o.lin(&[(v(&g(&m)), Some(K(-85))), (K(10_000), None)]);
        let tmo = g(&tm);
        for c in 0..3 {
            let t = o.lin(&[(v(&g(&col[c])), Some(v(&keep)))]);
            let t = o.div(v(&t), K(1000), 0);
            let t = o.div(v(&t), K(10), 0);
            let w_ = o.tab2(tb.t("bs_word"), K(c as i64), v(&tmo));
            let t = o.sum(&[(v(&t), 1), (v(&w_), 1)], 0);
            for (j, &i) in on.iter().enumerate() { col[c][i] = t[j]; }
        }
    }
    col
}

/// x mod 1000 for x 0 or more (a long or three cards).
fn modk<O: Ops>(o: &mut O, x: &V) -> V {
    let q = o.div(v(x), K(1000), 0);
    o.lin(&[(v(x), None), (v(&q), Some(K(-1000)))])
}

/// The volume knob on the airship's screen (train_night.py's vol_tex, its printed parts laid together here): the
/// scale 0-10 round a knob at 10; the surprise 11 past the stop materialising, white; the knob's knurls and its
/// pointer turned to its setting; VOL 10, and from the click VOL 11, the 11 big and white (red
/// was hard to make out against the purple); the whole as light
/// (squared, the gamma), a red glow round the 11 as it comes; the click kicking the screen 1.6 times for two frames.
/// Thousandths of 1.0.
fn vol_screen<O: Ops>(o: &mut O, s: &NightShade, a: &V, b: &V) -> [V; 3] {
    let nw = s.nw;
    let tb = &nw.w.tb;
    let f = s.l.f;
    let pa = &nw.path;
    let (v11a, v11b, click) = (pa.f_read - 30, pa.f_read - 20, pa.f_read - 12);
    let n = a.len();
    // the texel (nearest): row (1 - b) 560, column a 1000
    let row = o.tab(tb.t("vol_row"), v(b));
    let col = a.clone();
    let base = tb.t("vol_base");
    let mut c: [V; 3] = std::array::from_fn(|ch| { let ci = o.lin(&[(v(&col), Some(K(3))), (K(ch as i64), None)]); o.tab2(base, v(&row), v(&ci)) });
    // the 11: white m, m its coming (thousandths)
    let m11 = smooth1(o, f, v11a, v11b);
    let red = f >= click;
    if m11 > 0 {
        let mk = o.tab2(tb.t("vol_m11"), v(&row), v(&col));
        let c11: [i64; 3] = std::array::from_fn(|_| { let t = fk(o, 255_000, m11, 1000); fk(o, t, 1, 1000) });
        for ch in 0..3 { c[ch] = over_mask(o, tb, &c[ch], c11[ch], &mk); }
    }
    // the knob's knurls (40, from 0.86 R to R, 4 wide) and pointer (0.2 R to 0.95 R, 16 wide), turned 27 degrees a
    // step of its setting
    let kv = if f < click { 10_000 } else if f < click + 1 { 10_600 } else if f < click + 2 { 11_120 } else { 11_000 };
    let dx = o.lin(&[(v(&col), None), (K(-290), None)]);
    let dy = o.lin(&[(v(&row), Some(K(-1))), (K(215), None)]);
    let r2 = o.lin(&[(v(&dx), Some(v(&dx))), (v(&dy), Some(v(&dy)))]);
    let inside = o.lt(v(&r2), K(130 * 130 + 1));
    let kn: Vec<usize> = (0..n).filter(|&i| inside[i] == 1).collect();
    if !kn.is_empty() {
        let g = |v: &V| -> V { kn.iter().map(|&i| v[i]).collect() };
        let (gx, gy, gr2) = (g(&dx), g(&dy), g(&r2));
        let u = nrm3(o, &gx, &gy, &vec![0; kn.len()]);
        let ang = azimuth(o, tb, &u[0], &u[1]);
        // knurls: the angle less the knob's turn (mrad), its distance to the nearest of every 2 pi / 40
        let turn = fk(o, kv, 471, 1000);
        let rel = o.lin(&[(v(&ang), None), (K(turn), Some(K(-1))), (K(100 * 157_080), None)]);
        let q = o.div(v(&rel), K(157), 0);
        let fr = o.lin(&[(v(&rel), None), (v(&q), Some(K(-157)))]);
        let back = o.lin(&[(v(&fr), Some(K(-1))), (K(157), None)]);
        let dd = o.min(v(&fr), v(&back));
        let rr = rsqrt(o, &gr2);
        let arc = o.lin(&[(v(&dd), Some(v(&rr)))]);
        let ring = within_v(o, &rr, 111, 131);
        let thin = o.lt(v(&arc), K(2001));
        let knurl = mulw(o, v(&ring), v(&thin), 0);
        // the pointer: along its way and across it (texels)
        let pa_ = fk(o, kv, 471, 1000);
        let pa_ = one(o.lin(&[(K(pa_), None), (K(-2356), None)]));
        let pa_m = fk(o, pa_, 159_155, 1000);
        let (ps, pc) = o.sincos(K(pa_m));
        let (ps, pc) = (ps[0], pc[0]);
        let along = o.lin(&[(v(&gx), Some(K(ps))), (v(&gy), Some(K(pc)))]);
        let along = o.div(v(&along), K(1000), 0);
        let across = o.lin(&[(v(&gx), Some(K(pc))), (v(&gy), Some(K(-ps)))]);
        let across = o.div(v(&across), K(1000), 0);
        let am = o.lin(&[(v(&across), Some(K(-1)))]);
        let aab = o.max(v(&across), v(&am));
        let p1 = within_v(o, &along, 25, 124);
        let p2 = o.lt(v(&aab), K(9));
        let ptr = mulw(o, v(&p1), v(&p2), 0);
        for ch in 0..3 {
            let cur = g(&c[ch]);
            let t = o.sel(&knurl, K([60, 60, 66][ch]), v(&cur));
            let t = o.sel(&ptr, K([255, 255, 250][ch]), v(&t));
            for (j, &i) in kn.iter().enumerate() { c[ch][i] = t[j]; }
        }
    }
    // VOL, and 10 or 11
    let mv = o.tab2(tb.t("vol_mvol"), v(&row), v(&col));
    let mn = o.tab2(tb.t(if red { "vol_m11b" } else { "vol_m10" }), v(&row), v(&col));
    let m = o.max(v(&mv), v(&mn));
    let tc = if red { [255, 255, 255] } else { [240, 238, 232] };
    for ch in 0..3 { c[ch] = over_mask(o, tb, &c[ch], 1000 * tc[ch], &m); }
    // as light: (c / 255)^2.2 2.2 (thousandths)
    let mut lc: [V; 3] = std::array::from_fn(|ch| o.tab(tb.t("pow22x"), v(&c[ch])));
    if m11 > 0 {
        // the glow round the 11: e^-(d^2 / 6000) (m (1 - m) 4 + 0.6 lit or 0.2), (3.0, 0.3, 0.2)
        let gx = o.lin(&[(v(&col), None), (K(-362), None)]);
        let gy = o.lin(&[(v(&row), None), (K(-438), None)]);
        let d2 = o.lin(&[(v(&gx), Some(v(&gx))), (v(&gy), Some(v(&gy)))]);
        // (d^2 / 6 as 1667 ten-thousandths, held where e^- is gone)
        let d2 = o.min(v(&d2), K(120_000));
        let x = o.lin(&[(v(&d2), Some(K(1667)))]);
        let x = o.div(v(&x), K(1000), 0);
        let x = o.div(v(&x), K(10), 0);
        let gw = expn(o, tb, &x);
        let rest = one(o.lin(&[(K(m11), Some(K(-1))), (K(1000), None)]));
        let mm = fk(o, m11, rest, 250);
        let kk = one(o.lin(&[(K(mm), None), (K(if red { 600 } else { 200 }), None)]));
        for ch in 0..3 {
            let k = fk(o, kk, [3000, 300, 200][ch], 1000);
            let t = scaled(o, &gw, k, 1000);
            lc[ch] = o.sum(&[(v(&lc[ch]), 1), (v(&t), 1)], 0);
        }
    }
    if click <= f && f < click + 2 { for ch in 0..3 { lc[ch] = scaled(o, &lc[ch], 16, 10); } }
    lc
}

/// A colour (0..255; `col` thousandths of it) laid over by a mask's coverage (0..255).
fn over_mask<O: Ops>(o: &mut O, tb: &Tb, under: &V, col: i64, m: &V) -> V {
    // (only where the mask covers: elsewhere what's under stays)
    let cov = o.lt(K(0), v(m));
    let at: Vec<usize> = (0..m.len()).filter(|&i| cov[i] == 1).collect();
    let mut out = under.clone();
    if at.is_empty() { return out; }
    let g = |x: &V| -> V { at.iter().map(|&i| x[i]).collect() };
    let mk = o.tab(tb.t("mask1000"), v(&g(m)));
    let t = o.lin(&[(v(&g(under)), Some(K(1000)))]);
    let d = o.lin(&[(K(col), None), (v(&t), Some(K(-1)))]);
    let d = o.lin(&[(v(&d), Some(v(&mk)))]);
    let d = o.div(v(&d), K(1000), 0);
    let r = o.lin(&[(v(&t), None), (v(&d), None)]);
    let r = o.div(v(&r), K(1000), 0);
    for (j, &i) in at.iter().enumerate() { out[i] = r[j]; }
    out
}
