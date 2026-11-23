// The shading of what each pixel sees: the stadium's surfaces (lit at their corners by their probes' cubes), the
// lit windows and edges, the LED skin, the screens' pictures, the impostors (lit per pixel from their probes' cubes
// turned to face the eye, with the rim), the near dancers. Colours in ten-thousandths of the prototype's linear
// light. Included after crowd.rs.

/// The probes' cubes turned to face the eye: each probe's faces across (+, -), up (+, -) and toward the eye (+, -),
/// from its world cube (the face along a direction e is e_x^2 C(x) + e_y^2 C(y), the faces by e's signs).
pub fn eye_faces<O: Ops>(o: &mut O, t: &Tables, lt: &LightTabs, cam: &Cam, cube: &Cube, pts: &[[i64; 3]]) -> Cube {
    let px: V = pts.iter().map(|p| p[0]).collect();
    let py: V = pts.iter().map(|p| p[1]).collect();
    let (to, _) = toward_eye(o, t, lt, cam, &px, &py);
    let n = pts.len();
    // the squares' logarithms (over 998,001) and the signs
    let nx = o.tabo(&t.neg, 999_999, v(&to[0]));
    let ny = o.tabo(&t.neg, 999_999, v(&to[1]));
    let lx2 = o.tabo(&t.ln_sq999, 999, v(&to[0]));
    let ly2 = o.tabo(&t.ln_sq999, 999, v(&to[1]));
    let mut out: Cube = std::array::from_fn(|_| std::array::from_fn(|_| vec![0; n]));
    // toward the eye: (to_x, to_y); across: (-to_y, to_x); and their opposites
    for (face, dir) in [(4usize, 0i64), (5, 1), (0, 2), (1, 3)] {
        for c in 0..3 {
            // the x part's face and weight, the y part's
            let (wx, wy, fxp, fyp): (&V, &V, Vec<usize>, Vec<usize>) = match dir {
                0 => (&lx2, &ly2, (0..n).map(|i| nx[i] as usize).collect(), (0..n).map(|i| ny[i] as usize).collect()),
                1 => (&lx2, &ly2, (0..n).map(|i| 1 - nx[i] as usize).collect(), (0..n).map(|i| 1 - ny[i] as usize).collect()),
                2 => (&ly2, &lx2, (0..n).map(|i| 1 - ny[i] as usize).collect(), (0..n).map(|i| nx[i] as usize).collect()),
                _ => (&ly2, &lx2, (0..n).map(|i| ny[i] as usize).collect(), (0..n).map(|i| 1 - nx[i] as usize).collect()),
            };
            let cx: V = (0..n).map(|i| cube[fxp[i]][c][i]).collect();
            let cy: V = (0..n).map(|i| cube[2 + fyp[i]][c][i]).collect();
            let (lcx, _) = lnv(o, t, &cx);
            let (lcy, _) = lnv(o, t, &cy);
            let a = { let y = o.sum(&[(v(&lcx), 1), (v(wx), 1)], 0); expv(o, t, &y, None) };
            let b = { let y = o.sum(&[(v(&lcy), 1), (v(wy), 1)], 0); expv(o, t, &y, None) };
            out[face][c] = o.sum(&[(v(&a), 1), (v(&b), 1)], 0);
        }
    }
    for c in 0..3 { out[2][c] = cube[4][c].clone(); out[3][c] = cube[5][c].clone(); }
    out
}

/// The impostors' pixels: the light from their probes' faces by the atlas's normal (across, up, toward the eye; each
/// part squared over 9801 times its face), times the person's darkness plus the rim (1.2 (1 - n.v)^6, n.v from the
/// normal's parts toward the eye and up and the eye's elevation), plus a trace of their own.
pub fn shade_sprites<O: Ops>(o: &mut O, t: &Tables, sc: &Scene, cr: &Crowd, faces_ln: &[[V; 3]; 6], win: &[usize], wash: i64) -> [V; 3] {
    let n = win.len();
    let mut out: [V; 3] = std::array::from_fn(|_| vec![0; n]);
    if n == 0 { return out; }
    let pp = &cr.pieces;
    let sp = |i: usize| -> &Sprite { &cr.sprites[pp.sid[win[i]] as usize] };
    let nrm = |i: usize, ch: usize| -> i64 {
        let s = sp(i);
        let atl = if s.hi { &sc.atlas_hi } else { &sc.atlas };
        atl.ats(s.k, pp.ty[win[i]] as usize, pp.tx[win[i]] as usize, ch)
    };
    let probe: Vec<usize> = (0..n).map(|i| sc.cprobe()[sp(i).person]).collect();
    let mut parts: [Vec<V>; 3] = std::array::from_fn(|_| vec![]);
    for axis in 0..3 {
        let comp: V = (0..n).map(|i| nrm(i, 1 + axis)).collect();
        let w = o.tabo(&t.ln_sq99, 99, v(&comp));
        let ng = o.tabo(&t.neg, 999_999, v(&comp));
        for c in 0..3 {
            let lc: V = (0..n).map(|i| faces_ln[2 * axis + ng[i] as usize][c][probe[i]]).collect();
            let y = o.sum(&[(v(&lc), 1), (v(&w), 1)], 0);
            parts[c].push(expv(o, t, &y, None));
        }
    }
    // (and the flare's wash, white, on everyone: `wash`, the light's own units)
    let l: [V; 3] = std::array::from_fn(|c| o.sum(&[(v(&parts[c][0]), 1), (v(&parts[c][1]), 1), (v(&parts[c][2]), 1)], wash));
    // the rim: n.v = (toward cos el + up sin el) / 99, thousandths
    let nt: V = (0..n).map(|i| nrm(i, 3)).collect();
    let nu: V = (0..n).map(|i| nrm(i, 2)).collect();
    // each term by logarithms: ln |n_part| + ln |cos or sin| less ln 99, its sign the two's
    let ce: V = (0..n).map(|i| sp(i).el[0]).collect();
    let se: V = (0..n).map(|i| sp(i).el[1]).collect();
    let term = |o: &mut O, a: &V, b: &V| -> V {
        let (la, na) = lnv(o, t, a);
        let (lb, nb) = lnv(o, t, b);
        let y = o.sum(&[(v(&la), 1), (v(&lb), 1)], -lnf(99.0));
        // the signs differ: the printed table of it, by the two signs the machines punched
        let ng = o.tab2(&Tab { name: "differ", data: vec![0, 1, 1, 0], w: 2 }, v(&na), v(&nb));
        expv(o, t, &y, Some(&ng))
    };
    let t1 = term(o, &nt, &ce);
    let t2 = term(o, &nu, &se);
    let ndv = o.sum(&[(v(&t1), 1), (v(&t2), 1)], 0);
    let ndv = o.clamp(v(&ndv), -1000, 1000);
    let rim = o.tabo(&t.rim, 1000, v(&ndv));
    for c in 0..3 {
        let a: V = (0..n).map(|i| sc.ctone[sp(i).person][c]).collect();
        let g: V = (0..n).map(|i| sc.cglow[sp(i).person][c]).collect();
        let ar = o.sum(&[(v(&a), 1), (v(&rim), 1)], 0);
        let lar = o.tabo(&t.ln, 999_999, v(&ar));
        let (ll, _) = lnv(o, t, &l[c]);
        let y = o.sum(&[(v(&lar), 1), (v(&ll), 1)], -LN10K);
        let e = expv(o, t, &y, None);
        out[c] = o.sum(&[(v(&e), 1), (v(&g), 1)], 0);
    }
    out
}

impl Tables {
    /// ln (x^2 / 9801) for x -99..99 (offset 99)
    pub fn ln_sq99_tab() -> Tab { tab("ln_sq99", (-99..=99).map(|x: i64| if x == 0 { LN0 } else { lnf((x * x) as f64 / 9801.0) }).collect()) }
    /// ln (x^2 / 998,001) for x -999..999 (offset 999)
    pub fn ln_sq999_tab() -> Tab { tab("ln_sq999", (-999..=999).map(|x: i64| if x == 0 { LN0 } else { lnf((x * x) as f64 / 998_001.0) }).collect()) }
    /// the rim: 1.2 (1 - |x|)^6 for x -1000..1000 thousandths (ten-thousandths)
    pub fn rim_tab() -> Tab { tab("rim", (-1000..=1000).map(|x: i64| (1.2 * (1.0 - (x.abs() as f64 / 1000.0)).powi(6) * 10_000.0).round() as i64).collect()) }
}

/// The light at points with normals (hundredths) from their probes' world cubes (the cube's light for each).
pub fn probe_light<O: Ops>(o: &mut O, t: &Tables, cube: &Cube, probe: &[usize], n: &[[i64; 3]]) -> [V; 3] { cube_light(o, t, cube, probe, n) }

/// A screen's pixels: each texel's linear light (the srgb table) times the screen's level, by logarithms.
pub fn shade_texels<O: Ops>(o: &mut O, t: &Tables, bytes: &[[u8; 3]], ln_level: i64) -> [V; 3] {
    std::array::from_fn(|c| {
        let b: V = bytes.iter().map(|x| x[c] as i64).collect();
        let l = o.tab(&t.ln_srgb, v(&b));
        let y = o.sum(&[(v(&l), 1)], ln_level);
        expv(o, t, &y, None)
    })
}
