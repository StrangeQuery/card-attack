// The light (the giant screen and the LED walls as area lights, the moving heads' pools,
// the strobes, the pyro; on the airship its searchlights' pools) worked out at points, as an ambient cube: the light
// falling on the six faces of a small cube there (+x, -x, +y, -y, +z, -z), each colour. A surface takes its point's
// cube and its own normal: the sum of each face's light times the normal's part along it squared. The points: the
// probes (a grid over the pitch, rings round the tiers, the stadium's places), the near dancers, the flags' corners,
// the balloons, the airship's corners. Included after clock.rs.

/// A light for the frame: where (centimetres), its colour (ten-thousandths, already times its level), what kind.
#[derive(Clone, Debug)]
pub struct Lamp { pub pos: [i64; 3], pub rgb: [i64; 3], pub kind: LampKind }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LampKind {
    /// an area light facing -Y (the screen's parts, the LED walls): its area in square metres
    Area(i64),
    /// a moving head: its way (thousandths)
    Spot([i64; 3]),
    /// the strobes (d^2 + 100), the pyro (d^2 + 20)
    Point(i64),
    /// a searchlight on the airship's hull (no fall-off): its way
    Search([i64; 3]),
}

/// The tables the light is looked up in, by the distance squared (half-metres squared, under 1,000,000) or a
/// direction's part (thousandths).
pub struct LightTabs {
    /// ln of half the distance, ten-thousandths (for v / d)
    pub ln_d: Tab,
    /// ln (pi d^2 + A) for the screen's parts (A 256) and the LED walls (A 1056), ln (d^2 + 1), (d^2 + 100), (d^2 + 20)
    pub ln_area: [Tab; 2],
    pub ln_d2p: [Tab; 3],
    /// the square of a half-metre count (-5000..5000)
    pub sq: Tab,
    /// ln max(x, 0) and ln (0.7 max(x, 0) + 0.3 max(x + 0.4, 0)) for x -1000..1000 thousandths (ln of thousandths)
    pub ln_g: Tab,
    pub ln_gs: Tab,
    /// the moving heads' cone: ln e^(-max(1 - c, 0) / (1 - cos 5 deg)) for c -1000..1000 (thousandths)
    pub ln_cone5: Tab,
    pub ln_cone8: Tab,
}

pub fn light_tabs() -> LightTabs {
    const N: i64 = 1_000_000;
    let lnd = |f: &dyn Fn(f64) -> f64| -> Tab { tab("ln_d2", (0..N).map(|d2| { let x = f(d2 as f64 / 4.0); if x <= 0.0 { LN0 } else { lnf(x) } }).collect()) };
    let g = |f: &dyn Fn(f64) -> f64| -> Tab { tab("ln_g", (-1000..=1000).map(|x| { let y = f(x as f64 / 1000.0) * 1000.0; if y < 0.5 { LN0 } else { lnf(y) } }).collect()) };
    let cone = |deg: f64| -> Tab { let k = 1.0 - deg.to_radians().cos(); tab("ln_cone", (-1000..=1000).map(|c| lnf(((-(1.0 - c as f64 / 1000.0).max(0.0) / k).exp() * 1000.0).max(1e-30)).max(-200_000)).collect()) };
    LightTabs {
        ln_d: tab("ln_d", (0..N).map(|d2| if d2 == 0 { 0 } else { lnf((d2 as f64).sqrt()) }).collect()),
        ln_area: [lnd(&|d2| std::f64::consts::PI * d2 + 96.0 * 48.0 / 18.0), lnd(&|d2| std::f64::consts::PI * d2 + 24.0 * 44.0)],
        ln_d2p: [lnd(&|d2| d2 + 1.0), lnd(&|d2| d2 + 100.0), lnd(&|d2| d2 + 20.0)],
        sq: tab("sq", (-5000..=5000).map(|x| x * x).collect()),
        ln_g: g(&|x| x.max(0.0)),
        ln_gs: g(&|x| 0.7 * x.max(0.0) + 0.3 * (x + 0.4).max(0.0)),
        ln_cone5: cone(5.0),
        ln_cone8: cone(8.0),
    }
}

/// The ambient cubes at points: [face][colour] light (ten-thousandths), faces +x, -x, +y, -y, +z, -z.
pub type Cube = [[V; 3]; 6];

pub fn cubes<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, pts: &[[i64; 3]], lamps: &[Lamp]) -> Cube {
    let n = pts.len();
    let acc: Cube = std::array::from_fn(|_| std::array::from_fn(|_| vec![0; n]));
    if n == 0 { return acc; }
    // each face's and colour's light from every lamp, summed at the end
    let mut parts: Vec<Vec<Vec<V>>> = vec![vec![vec![]; 3]; 6];
    // the points in half-metres
    let ph: [V; 3] = std::array::from_fn(|i| { let c: V = pts.iter().map(|p| p[i]).collect(); divk(o, &c, 50) });
    for lamp in lamps {
        if lamp.rgb.iter().all(|&c| c == 0) { continue; }
        // v = the lamp less the point (half-metres), its length squared, ln of the length
        let q: [i64; 3] = std::array::from_fn(|i| s_div(lamp.pos[i], 50, o));
        // (held to 577 each way: past 288 m the light is too faint to matter, and the squares' sum stays a long)
        let d: [V; 3] = std::array::from_fn(|i| { let d = o.sum(&[(v(&ph[i]), -1)], q[i]); o.clamp(v(&d), -577, 577) });
        let sq: [V; 3] = std::array::from_fn(|i| o.tabo(&lt.sq, 5000, v(&d[i])));
        let d2 = o.sum(&[(v(&sq[0]), 1), (v(&sq[1]), 1), (v(&sq[2]), 1)], 0);
        let lnd = o.tab(&lt.ln_d, v(&d2));
        // its direction's parts (thousandths): ln |v_i| + ln 1000 less ln d, e back with the sign
        let u: [V; 3] = std::array::from_fn(|i| { let (l, ng) = lnv(o, t, &d[i]); let y = o.sum(&[(v(&l), 1), (v(&lnd), -1)], LN1000); let e = expv(o, t, &y, Some(&ng)); o.clamp(v(&e), -1000, 1000) });
        // the light's strength there (ln, before the face): by kind
        let (ln_s, gtab) = match lamp.kind {
            LampKind::Area(a) => {
                // the area times the facing (the light's side, v_y > 0) over (pi d^2 + A)
                let lcs = o.tabo(&lt.ln_g, 1000, v(&u[1]));
                let la = o.tab(&lt.ln_area[(a > 300) as usize], v(&d2));
                (o.sum(&[(v(&lcs), 1), (v(&la), -1)], lnf(a as f64) - LN1000), &lt.ln_g)
            }
            LampKind::Spot(w) => {
                // the cone: c = -(u . w), thousandths
                let c = { let n: [i64; 3] = std::array::from_fn(|k| s_neg(w[k], o)); o.lin(&[(v(&u[0]), Some(K(n[0]))), (v(&u[1]), Some(K(n[1]))), (v(&u[2]), Some(K(n[2])))]) };
                let c = o.div(v(&c), K(1000), 0);
                let c = o.clamp(v(&c), -1000, 1000);
                let lc = o.tabo(&lt.ln_cone5, 1000, v(&c));
                let ld = o.tab(&lt.ln_d2p[0], v(&d2));
                (o.sum(&[(v(&lc), 1), (v(&ld), -1)], lnf(9000.0) - LN1000), &lt.ln_gs)
            }
            LampKind::Point(k) => {
                let ld = o.tab(&lt.ln_d2p[if k == 100 { 1 } else { 2 }], v(&d2));
                let s = if k == 100 { 9000.0 } else { 2500.0 };
                (o.sum(&[(v(&ld), -1)], lnf(s)), &lt.ln_g)
            }
            LampKind::Search(w) => {
                let c = { let n: [i64; 3] = std::array::from_fn(|k| s_neg(w[k], o)); o.lin(&[(v(&u[0]), Some(K(n[0]))), (v(&u[1]), Some(K(n[1]))), (v(&u[2]), Some(K(n[2])))]) };
                let c = o.div(v(&c), K(1000), 0);
                let c = o.clamp(v(&c), -1000, 1000);
                let lc = o.tabo(&lt.ln_cone8, 1000, v(&c));
                (o.sum(&[(v(&lc), 1)], lnf(35.0) - LN1000), &lt.ln_g)
            }
        };
        // (held: under e^-30 it is nothing)
        let ln_s = o.max(v(&ln_s), K(-300_000));
        // each face: the strength times g(the direction's part along it), each colour
        let neg: [V; 3] = std::array::from_fn(|i| o.sum(&[(v(&u[i]), -1)], 0));
        let lrgb: [i64; 3] = std::array::from_fn(|c| if lamp.rgb[c] == 0 { 0 } else { s_tab_o(&t.ln, 999_999, lamp.rgb[c], o) });
        for f in 0..6 {
            let part = if f % 2 == 0 { &u[f / 2] } else { &neg[f / 2] };
            let lg = o.tabo(gtab, 1000, v(part));
            // only where the face takes any of it (the table's ln 0 elsewhere: nothing to add)
            let at: Vec<usize> = (0..n).filter(|&k| lg[k] != LN0).collect();
            if at.is_empty() { continue; }
            let ls: V = at.iter().map(|&k| ln_s[k]).collect();
            let lgs: V = at.iter().map(|&k| lg[k]).collect();
            let y0 = o.sum(&[(v(&ls), 1), (v(&lgs), 1)], -LN1000);
            for c in 0..3 {
                if lamp.rgb[c] == 0 { continue; }
                let y = o.sum(&[(v(&y0), 1)], lrgb[c]);
                let e = expv(o, t, &y, None);
                let mut full = vec![0; n];
                for (q, &k) in at.iter().enumerate() { full[k] = e[q]; }
                parts[f][c].push(full);
            }
        }
    }
    std::array::from_fn(|f| std::array::from_fn(|c| {
        if parts[f][c].is_empty() { return vec![0; n]; }
        let terms: Vec<(A, i64)> = parts[f][c].iter().map(|x| (v(x), 1)).collect();
        o.sum(&terms, 0)
    }))
}

/// The light on surfaces with normals n (hundredths, a unit 99) from their cubes: each face's light times the
/// normal's part squared over 9801, summed (ten-thousandths).
pub fn cube_light<O: Ops>(o: &mut O, t: &Tables, cube: &Cube, at: &[usize], n: &[[i64; 3]]) -> [V; 3] {
    let m = at.len();
    let mut out: [V; 3] = std::array::from_fn(|_| vec![0; m]);
    if m == 0 { return out; }
    for axis in 0..3 {
        let comp: V = n.iter().map(|q| q[axis]).collect();
        // ln of the part squared over 9801 (the printed table), the part's sign
        let lw2 = o.tabo(&t.ln_sq99, 99, v(&comp));
        let ng = o.tabo(&t.neg, 999_999, v(&comp));
        for c in 0..3 {
            let pick: V = (0..m).map(|i| cube[2 * axis + ng[i] as usize][c][at[i]]).collect();
            let (lc, _) = lnv(o, t, &pick);
            let y = o.sum(&[(v(&lc), 1), (v(&lw2), 1)], 0);
            let e = expv(o, t, &y, None);
            out[c] = o.sum(&[(v(&out[c]), 1), (v(&e), 1)], 0);
        }
    }
    out
}

/// The frame's lamps: the screen's 18 parts and the LED walls (their pictures' means, already times 0.6 and 0.3),
/// the moving heads, the strobes and the pyro when they're on; for the airship its searchlights too, and no pyro.
pub fn lamps<O: Ops>(o: &mut O, sc: &Scene, sh: &Show, spot_d: &[[i64; 3]], search_d: &[[i64; 3]], screen_means: &[[i64; 3]], led_means: &[[i64; 3]], airship: bool) -> Vec<Lamp> {
    let mut out = vec![];
    let scale = |o: &mut O, c: [i64; 3], lev: i64| -> [i64; 3] { c.map(|x| s_div(s_lin(&[(x, Some(lev))], o), 1000, o)) };
    for (i, p) in sc.screen_parts.iter().enumerate() { out.push(Lamp { pos: *p, rgb: screen_means[i], kind: LampKind::Area(256) }); }
    for (i, lw) in sc.led_walls.iter().enumerate() { out.push(Lamp { pos: [lw[0], lw[1], lw[2]], rgb: led_means[i], kind: LampKind::Area(1056) }); }
    // (the strobes fire with the flare too, at its level)
    let st = if sh.flare > 0 { s_max(sh.strobe, sh.flare, o) } else { sh.strobe };
    if st > 0 {
        for x in [-3000, 0, 3000] { let rgb = scale(o, [9000, 9200, 10_000], st); out.push(Lamp { pos: [x, 4400, 4000], rgb, kind: LampKind::Point(100) }); }
    }
    for (j, l) in sc.spots.iter().enumerate() {
        let rgb = scale(o, l.rgb, sh.spots);
        out.push(Lamp { pos: l.pos, rgb, kind: LampKind::Spot(spot_d[j]) });
    }
    if airship {
        for (j, l) in sc.search.iter().enumerate() {
            if l.mode != 0 { out.push(Lamp { pos: l.pos, rgb: l.rgb, kind: LampKind::Search(search_d[j]) }); }
        }
    } else if sh.pyro > 0 {
        // (each lamp 6 m over its pyro's place)
        let z = { let z: V = sc.pyro.iter().map(|p| p[2]).collect(); o.sum(&[(v(&z), 1)], 600) };
        for (i, p) in sc.pyro.iter().enumerate() { let rgb = scale(o, [10_000, 5500, 2000], sh.pyro); out.push(Lamp { pos: [p[0], p[1], z[i]], rgb, kind: LampKind::Point(20) }); }
    }
    out
}
