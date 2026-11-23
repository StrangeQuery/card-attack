// The frame's own numbers: the camera, the show's levels on the beat, where every beam points, the airship. All on
// CARDIAC, one number at a time (a frame has a few thousand of them). Included after maths.rs.
//
// Time is the show's clock in twelfths of a second (t12; the crest, where the night arrival hands over, at 36).
// Angles are millionths of a turn, sines and cosines 999ths (the swarm's table), directions thousandths.

/// The camera keys after the crest (and the two before it it eases from): times in twelve-thousandths of a second,
/// eye and the point looked at in centimetres. (The first three are the flight's keys; the rest at the
/// crowd's height.)
pub const KEYS: [(i64, [i64; 3], [i64; 3]); 41] = [
    (0, [-4000, -15000, 1000], [0, -10000, 4500]),
    (18000, [-6350, -12560, 5200], [0, -4000, 3000]),
    (36000, [-4800, -11000, 4800], [0, 4000, 1000]),
    // at the airship's height, still looking ahead, over to above the left aisle's end; straight down into it
    // (two 4 m aisles up the pitch, x -6 m, almost under the airship, and 30 m) and up it
    // just over the heads (2.4 m), looking along it at the stage, the hands in the air either side; up to 5 m and across to the right
    // aisle, turning a full circle over the crowd on the way (the stage, the east wall, the faces, the west wall,
    // the stage again); down into the right aisle and pulled back down it, facing the stage
    // (sampled from two smooth moves, a glide from the crest to above the aisle's end, slowing, still
    // looking ahead; a descent into it, easing out at 2 m into the run)
    (40000, [-4384, -10581, 4681], [-21, 4082, 1017]),
    (44000, [-3853, -9970, 4574], [-76, 4301, 1063]),
    (48000, [-3252, -9230, 4479], [-156, 4618, 1130]),
    (52000, [-2623, -8424, 4396], [-250, 4995, 1209]),
    (56000, [-2011, -7615, 4327], [-350, 5390, 1291]),
    (60000, [-1459, -6865, 4273], [-444, 5767, 1370]),
    (64000, [-1011, -6238, 4233], [-524, 6084, 1437]),
    (68000, [-710, -5795, 4208], [-579, 6303, 1483]),
    (72000, [-600, -5600, 4200], [-600, 6385, 1500]),
    (76000, [-600, -5572, 4006], [-600, 6385, 1515]),
    (80000, [-600, -5571, 3498], [-600, 6385, 1553]),
    (84000, [-600, -5575, 2792], [-600, 6385, 1606]),
    (88000, [-600, -5558, 2000], [-600, 6385, 1665]),
    (92000, [-600, -5498, 1237], [-600, 6385, 1722]),
    (96000, [-600, -5370, 616], [-600, 6385, 1769]),
    (100000, [-600, -5151, 251], [-600, 6385, 1796]),
    (102000, [-600, -5000, 200], [-600, 6385, 1800]),
    (118000, [-600, -3400, 200], [-600, 6385, 1900]),
    (134000, [-600, -1800, 200], [-600, 6385, 1900]),
    (150000, [-600, -200, 205], [-600, 6385, 1900]),
    (166000, [-600, 1200, 280], [-300, 6385, 1500]),
    (182000, [0, 1300, 500], [0, 6385, 1500]),
    (200000, [600, 1300, 500], [3600, 1300, 380]),
    (218000, [1200, 1300, 500], [1200, -1700, 380]),
    (236000, [1800, 1300, 500], [-1200, 1300, 380]),
    (254000, [2400, 1300, 500], [2400, 6385, 1500]),
    (270000, [3000, 1100, 300], [3000, 6385, 1500]),
    (282000, [3000, 400, 205], [3000, 6385, 1900]),
    (306000, [3000, -1500, 200], [3000, 6385, 1900]),
    (326000, [3000, -3300, 210], [3000, 6385, 1900]),
    (338000, [2900, -4100, 320], [2700, 6385, 1800]),
    // up at the airship (THANK YOU SAUL FINGERMAN, t12 378..421), as before
    (352000, [2200, -5000, 650], [300, -2000, 8000]),
    (383157, [0, -8600, 2200], [500, -4400, 11600]),
    (415000, [100, -8800, 2300], [600, -4400, 11600]),
    // down again, low: THANK YOU D. W. HAGELBARGER on the giant screen past the dancers; the rise to the CARDIAC
    // (on the pitch, which ends at y -55 m: lower than that the eye would be in the south stand)
    (428647, [0, -5000, 600], [0, 4000, 2000]),
    (446843, [600, -4800, 130], [0, 6400, 3000]),
    (487784, [0, -5000, 500], [0, 6400, 3100]),
    (531000, [-200, -4000, 1400], [0, 6400, 3200]),
];

/// The show's moments, in t12 (nominal times stretched, from the crowd's weave at 12.25 s to the cut at 44.25 s).
pub const HOLD: (i64, i64) = (378, 421);       // the airship's hold: THANK YOU SAUL FINGERMAN on its screen
pub const HAG: (i64, i64) = (445, 492);        // the giant screen: THANK YOU D. W. HAGELBARGER
pub const CARDIAC_T: i64 = 493;                // the giant screen's last picture: the CARDIAC
pub const CALM_T: i64 = 441;                   // the beams and lasers eased to 40%
pub const STROBE_T: i64 = 155;                 // no strobe in the orbit
pub const CREST_T: i64 = 36;
pub const FLARE_T: i64 = 214;                // the flare: the whole crowd lit at once at the circle's middle, decaying over 2 s

#[derive(Clone, Copy, Debug)]
pub struct Cam { pub eye: [i64; 3], pub b: [[i64; 3]; 3] }

/// The camera through the keys: Catmull-Rom, then turned 10 degrees right from 2 s after the crest
/// over 1.5 s (the stream chat sits at the right), and its axes.
pub fn camera<O: Ops>(o: &mut O, t: &Tables, t12: i64) -> Cam {
    let tt = s_lin(&[(t12, Some(1000))], o);
    // the key it's past (the keys before it counted on CARDIAC), its neighbours
    let n = KEYS.len() as i64;
    // (counted from -1: the last key it's past)
    let mut past = -1;
    for k in &KEYS { if s_lt(k.0, tt, o) { past = s_sum(&[(past, 1)], 1, o); } }
    let i = s_clamp(past, 0, n - 2, o) as usize;
    let (k0, k1, k2, k3) = (KEYS[i.saturating_sub(1)], KEYS[i], KEYS[i + 1], KEYS[(i + 2).min(n as usize - 1)]);
    let d21 = s_sum(&[(k2.0, 1), (k1.0, -1)], 0, o);
    let d20 = { let d = s_sum(&[(k2.0, 1), (k0.0, -1)], 0, o); s_least1(d, o) };
    let d31 = { let d = s_sum(&[(k3.0, 1), (k1.0, -1)], 0, o); s_least1(d, o) };
    let u = { let a = s_sum(&[(tt, 1), (k1.0, -1)], 0, o); let a = s_lin(&[(a, Some(1000))], o); s_clamp(s_div(a, d21, o), 0, 1000, o) };
    let h: [i64; 4] = std::array::from_fn(|j| s_tab(&t.herm[j], u, o));
    let path = |o: &mut O, sel: fn(&(i64, [i64; 3], [i64; 3])) -> [i64; 3]| -> [i64; 3] {
        std::array::from_fn(|c| {
            let (p0, p1, p2, p3) = (sel(&k0)[c], sel(&k1)[c], sel(&k2)[c], sel(&k3)[c]);
            let m1 = { let d = s_sum(&[(p2, 1), (p0, -1)], 0, o); let x = s_lin(&[(d, Some(d21))], o); s_div(x, d20, o) };
            let m2 = { let d = s_sum(&[(p3, 1), (p1, -1)], 0, o); let x = s_lin(&[(d, Some(d21))], o); s_div(x, d31, o) };
            let x = s_lin(&[(h[0], Some(p1)), (h[1], Some(m1)), (h[2], Some(p2)), (h[3], Some(m2))], o);
            s_div(x, 1000, o)
        })
    };
    let eye = path(o, |k| k.1);
    let tgt = path(o, |k| k.2);
    let d: [i64; 3] = std::array::from_fn(|i| s_sum(&[(tgt[i], 1), (eye[i], -1)], 0, o));
    look(o, eye, d)
}

/// The camera's axes looking along d (centimetres) from the eye, no roll: right, up, forward (thousandths).
pub fn look<O: Ops>(o: &mut O, eye: [i64; 3], d: [i64; 3]) -> Cam {
    let dd: [i64; 3] = std::array::from_fn(|i| s_div(d[i], 10, o));
    let l2 = s_lin(&[(dd[0], Some(dd[0])), (dd[1], Some(dd[1])), (dd[2], Some(dd[2]))], o);
    let l = { let r = s_sqrt(l2, o); s_least1(r, o) };
    let f: [i64; 3] = std::array::from_fn(|i| s_clamp(s_div(s_lin(&[(dd[i], Some(1000))], o), l, o), -999, 999, o));
    let r2 = s_lin(&[(f[0], Some(f[0])), (f[1], Some(f[1]))], o);
    let rl = { let r = s_sqrt(r2, o); s_least1(r, o) };
    let rx = s_clamp(s_div(s_lin(&[(f[1], Some(1000))], o), rl, o), -999, 999, o);
    let ry = s_clamp(s_div(s_lin(&[(f[0], Some(-1000))], o), rl, o), -999, 999, o);
    let ux = s_div(s_lin(&[(ry, Some(f[2]))], o), 1000, o);
    let uy = { let nf = s_neg(f[2], o); s_div(s_lin(&[(rx, Some(nf))], o), 1000, o) };
    let uz = { let nf = s_neg(f[0], o); s_div(s_lin(&[(rx, Some(f[1])), (ry, Some(nf))], o), 1000, o) };
    Cam { eye, b: [[rx, ry, 0], [ux, uy, uz], f] }
}

/// The show's levels (thousandths) and its clocks at a frame.
#[derive(Clone, Debug, Default)]
pub struct Show {
    pub t12: i64,
    pub kick: i64, pub off: i64, pub strobe: i64, pub crest: i64,
    pub spots: i64, pub search: i64, pub screen: i64, pub lasers: i64,
    pub pyro: i64,
    pub flare: i64,
    /// the LED walls' pulse (decay 3)
    pub pulse: i64,
    /// the bar (a slot of the screens) and the frame in the beat for the laser chase (thousandths)
    pub bar: i64, pub beat_m: i64,
    /// the screens' slot: the bar on the show's own clock (not the soundtrack's phase: the footage keeps its
    /// own timing)
    pub slot: i64,
}

/// Cycles at t12 (millionths, all of them) for a cycle of `per` twelfths of a second: t12 c, c = 10^6 / per.
pub fn cycle<O: Ops>(t12: i64, c: i64, o: &mut O) -> i64 { s_lin(&[(t12, Some(c)), (-beat_off(c), None)], o) }

/// The beat's phase against the soundtrack (its mix): the lights' hits (beats 1 and 3 at 137.14 BPM) fall at
/// t = BEAT_T0 + 0.875 j on the show's clock. Thousandths of a second; every beat and bar phase is counted from it
/// (the dancers' phase, the hits, the LED walls' pulse, the strobe, the pyro, the moving heads' and lasers'
/// patterns; the screens' slots and their footage keep the show's own clock).
pub const BEAT_T0_MS: i64 = 378;
/// The cycles (millionths) of a rate c (millionths a twelfth) in BEAT_T0: BEAT_T0 12 c.
pub const fn beat_off(c: i64) -> i64 { (BEAT_T0_MS * 12 * c + 500) / 1000 }
/// How far into the cycle (millionths), and how many whole cycles.
pub fn frac1m<O: Ops>(x: i64, o: &mut O) -> i64 { one(o.turns(K(x))) }
pub fn whole1m<O: Ops>(x: i64, o: &mut O) -> i64 { s_div(s_div(x, 1000, o), 1000, o) }

/// One beat: 5.250109 frames (137.14 BPM at 12 fps).
pub const C_BEAT: i64 = 190_472;         // 10^6 / 5.250109
pub const C_2BEAT: i64 = 95_236;
pub const C_8TH: i64 = 380_944;
pub const C_BAR: i64 = 47_618;
pub const C_PHRASE: i64 = 11_905;
pub const C_2BAR: i64 = 23_809;

pub fn show<O: Ops>(o: &mut O, t: &Tables, t12: i64) -> Show {
    // the lights hit on beats 1 and 3: the phase in the 2-beat cycle, the decays from the printed table
    let hb = cycle(t12, C_2BEAT, o);
    let ph = frac1m(hb, o);
    let kick = s_tab(&t.decay5, s_div(ph, 1000, o), o);
    let ph2 = { let a = s_lin(&[(ph, None), (500_000, None)], o); frac1m(a, o) };
    let off = s_tab(&t.decay5, s_div(ph2, 1000, o), o);
    let pulse = s_tab(&t.decay3, s_div(ph, 1000, o), o);
    // the strobe: in the last bar of each 4-bar phrase, on the eighth notes, not in the orbit, the holds or the end
    let phr = { let a = cycle(t12, C_PHRASE, o); frac1m(a, o) };
    let e8 = { let a = cycle(t12, C_8TH, o); frac1m(a, o) };
    let strobing = s_lt(749_999, phr, o) && s_lt(STROBE_T - 1, t12, o) && !(s_lt(HOLD.0 - 1, t12, o) && s_lt(t12, HOLD.1 + 1, o)) && s_lt(t12, HAG.0, o);
    let strobe = if strobing && s_lt(e8, 340_000, o) { 1000 } else { 0 };
    // the crest: everything firing at once, settling over 2 s
    let crest = if s_lt(t12, CREST_T, o) { 0 } else { let d = s_sum(&[(t12, 1)], -CREST_T, o); let d = s_clamp(d, 0, 999, o); s_tab(&t.crest, d, o) };
    // the flare (the whole crowd lit up for a moment at the circle's middle; spiked at once and
    // decaying over 2 s, a concert's): the crest's own settling from FLARE_T; everything firing, and the wash on the
    // crowd (frame.rs flare_wash)
    let flare = if s_lt(t12, FLARE_T, o) { 0 } else { let d = s_sum(&[(t12, 1)], -FLARE_T, o); s_tab(&t.crest, s_clamp(d, 0, 999, o), o) };
    let crest = s_max(crest, flare, o);
    let calm = !s_lt(t12, CALM_T, o);
    let m3 = |o: &mut O, a: i64, b: i64, c: i64| -> i64 { let m = s_max(a, b, o); s_max(m, c, o) };
    let c85 = s_div(s_lin(&[(crest, Some(85))], o), 100, o);
    let mut spots = { let m = m3(o, kick, strobe, c85); s_sum(&[(s_div(s_lin(&[(m, Some(95))], o), 100, o), 1)], 50, o) };
    let search = { let m = s_max(off, crest, o); s_sum(&[(s_div(s_lin(&[(m, Some(92))], o), 100, o), 1)], 80, o) };
    let screen = { let c8 = s_div(s_lin(&[(crest, Some(8))], o), 10, o); let m = s_max(kick, c8, o); s_sum(&[(s_div(m, 2, o), 1)], 300, o) };
    let mut lasers = { let m = m3(o, kick, strobe, crest); s_sum(&[(s_div(s_lin(&[(m, Some(88))], o), 100, o), 1)], 120, o) };
    if calm { spots = s_div(s_lin(&[(spots, Some(4))], o), 10, o); lasers = s_div(s_lin(&[(lasers, Some(4))], o), 10, o); }
    // the pyro: a burst at the crest and on the first beat of every second bar, 0.75 s
    let pyro = if s_lt(t12, CREST_T, o) { 0 } else {
        // from the downbeat at the crest (BEAT_T0 + 3 x 0.875 s, just after t = 3): 3 more half-cycles of the 2-bar
        let p = { let a = s_lin(&[(t12, Some(C_2BAR)), (-beat_off(C_2BAR) - 3 * 1_000_000 / 4, None)], o); frac1m(a, o) };
        if s_lt(p, 285_707, o) { s_tab(&t.decay_pyro(), s_div(p, 1000, o), o) } else { 0 }
    };
    let bar = { let a = cycle(t12, C_BAR, o); whole1m(a, o) };
    let slot = { let a = s_lin(&[(t12, Some(C_BAR))], o); whole1m(a, o) };
    let beat_m = { let a = cycle(t12, C_BEAT, o); s_div(frac1m(a, o), 1000, o) };
    Show { t12, kick, off, strobe, crest, spots, search, screen, lasers, pyro, flare, pulse, bar, beat_m, slot }
}

impl Tables {
    /// e^(-4 s) for s the thousandths of two bars (3.50007 s) past a burst: thousandths.
    pub fn decay_pyro(&self) -> Tab { tab("decay_pyro", (0..1000).map(|x| ((-4.0 * 3.500073 * x as f64 / 1000.0).exp() * 1000.0).round() as i64).collect()) }
}

/// An angle at t12 (millionths of a turn, its whole turns dropped) for a rate of `rad_s` radians a second and a
/// phase (millionths of a turn): the rate as millionths of a turn a twelfth, from the radians a second.
pub fn rate(rad_s: f64) -> i64 { (rad_s / (2.0 * std::f64::consts::PI) * 1e6 / 12.0).round() as i64 }
pub fn angle<O: Ops>(t12: i64, r: i64, phase: i64, o: &mut O) -> i64 {
    let a = s_lin(&[(t12, Some(r)), (phase, None)], o);
    one(o.turns(K(a)))
}
/// A sine (999ths) times an amplitude in radians, as millionths of a turn.
pub fn amp<O: Ops>(s: i64, rad: f64, o: &mut O) -> i64 {
    let k = (rad * 1e8 / (2.0 * std::f64::consts::PI * 999.0)).round() as i64;
    s_div(s_lin(&[(s, Some(k))], o), 100, o)
}
pub fn rad(x: f64) -> i64 { (x / (2.0 * std::f64::consts::PI) * 1e6).round() as i64 }

/// Where the airship's centre is at t12 (centimetres): drifting along X, bobbing.
pub fn blimp_at<O: Ops>(o: &mut O, sc: &Scene, t12: i64) -> [i64; 3] {
    let x = s_sum(&[(s_div(s_lin(&[(t12, Some(sc.bl_at[3]))], o), 12, o), 1)], sc.bl_at[0], o);
    let (s, _) = s_sincos(angle(t12, rate(0.6), 0, o), o);
    let z = s_sum(&[(s_div(s_lin(&[(s, Some(120))], o), 999, o), 1)], sc.bl_at[2], o);
    [x, sc.bl_at[1], z]
}

/// A moving head's beam (thousandths): sweeping over the crowd in figures, the pattern changing every bar of three.
pub fn spot_dir<O: Ops>(o: &mut O, sh: &Show, l: &Light, j: usize) -> [i64; 3] {
    let t12 = sh.t12;
    let pat = s_divmod(sh.bar, 3, o).1;
    let wt = angle(t12, rate(std::f64::consts::PI / 2.0), 0, o);
    let (pan, tilt) = match pat {
        0 => {
            let a = { let p = s_div(s_lin(&[(l.phase, Some(3))], o), 10, o); s_lin(&[(wt, None), (p, None)], o) };
            let (s1, _) = s_sincos(a, o);
            let b = s_lin(&[(wt, Some(2)), (l.phase, None)], o);
            let (s2, _) = s_sincos(b, o);
            (amp(s1, 0.55, o), s_sum(&[(amp(s2, 0.15, o), 1)], rad(0.55), o))
        }
        1 => {
            let (s1, _) = s_sincos(wt, o);
            let (_, c2) = s_sincos(s_lin(&[(wt, None), (l.phase, None)], o), o);
            let p = amp(s1, 0.6, o);
            (if j % 2 == 1 { p } else { s_sum(&[(p, -1)], 0, o) }, s_sum(&[(amp(c2, 0.2, o), 1)], rad(0.5), o))
        }
        _ => {
            let a = s_lin(&[(l.phase, Some(2)), (wt, None)], o);
            let (s1, c1) = s_sincos(a, o);
            (amp(s1, 0.4, o), s_sum(&[(amp(c1, 0.25, o), 1)], rad(0.5), o))
        }
    };
    let (sp, cp) = s_sincos(pan, o);
    let (st, ct) = s_sincos(tilt, o);
    let x = s_div(s_lin(&[(sp, Some(st))], o), 998, o);
    let y = { let n = s_neg(cp, o); s_div(s_lin(&[(n, Some(st))], o), 998, o) };
    let z = { let n = s_neg(ct, o); s_div(s_lin(&[(n, Some(1000))], o), 999, o) };
    [x, y, z]
}

/// The hull's surface (centimetres, the model's frame: along, up, across) at u (thousandths along -1..1) and v (an
/// angle round it), as the night arrival's model builds it.
pub fn blimp_hull<O: Ops>(o: &mut O, t: &Tables, u: i64, vang: i64) -> [i64; 3] {
    let au = s_abs(u, o);
    let base = s_tab(&t.hull_base(), s_clamp(au, 0, 1000, o), o);
    let (up, side) = s_sincos(vang, o);
    let (_, cl) = s_sincos(s_lin(&[(u, Some(3250))], o), o);
    let lobe_a = s_tab(&t.half_sq(), s_sum(&[(cl, 1)], 999, o), o);
    let lobe_b = s_max(s_sum(&[(up, 1)], 350, o), 0, o);
    let lobe_c = s_tab(&t.sqrt_th(), s_clamp(s_abs(side, o), 0, 999, o), o);
    let lobe = s_div(s_lin(&[(s_div(s_lin(&[(lobe_a, Some(lobe_b))], o), 999, o), Some(lobe_c))], o), 1000, o);
    let dn = { let n = s_neg(up, o); s_max(n, 0, o) };
    let r = s_sum(&[(s_div(s_lin(&[(lobe, Some(16))], o), 100, o), 1), (s_div(s_lin(&[(dn, Some(dn))], o), 9990, o), -1)], 1000, o);
    let br = s_div(s_lin(&[(base, Some(r))], o), 1000, o);
    let along = s_div(s_lin(&[(u, Some(55))], o), 10, o);
    let k = if s_lt(0, up, o) { 1725 } else { 1275 };       // 15 m times 1.15 or 0.85, centimetres
    let hup = s_div(s_lin(&[(s_div(s_lin(&[(br, Some(up))], o), 999, o), Some(k))], o), 1000, o);
    let hside = s_div(s_lin(&[(s_div(s_lin(&[(br, Some(side))], o), 999, o), Some(2100))], o), 1000, o);
    [along, hup, hside]
}

impl Tables {
    /// max(0, 1 - u^2.2)^0.55 for u 0..1000 thousandths (thousandths)
    pub fn hull_base(&self) -> Tab { tab("hull_base", (0..=1000).map(|u| ((1.0 - (u as f64 / 1000.0).powf(2.2)).max(0.0).powf(0.55) * 1000.0).round() as i64).collect()) }
    /// (0.5 + 0.5 c)^2 for c -999..999 (offset 999; thousandths)
    pub fn half_sq(&self) -> Tab { tab("half_sq", (-999..=999).map(|c| { let x = 0.5 + 0.5 * c as f64 / 999.0; (x * x * 1000.0).round() as i64 }).collect()) }
    /// the square root of x 0..999 thousandths (thousandths)
    pub fn sqrt_th(&self) -> Tab { tab("sqrt_th", (0..1000).map(|x| ((x as f64 / 1000.0).sqrt() * 1000.0).round() as i64).collect()) }
}

/// A searchlight's beam (thousandths): sweeping the sky (mode 0), or held on the airship's hull, its spot sweeping
/// along it and round its flanks (mode 2) on the night arrival's clock.
pub fn search_dir<O: Ops>(o: &mut O, t: &Tables, sc: &Scene, sh: &Show, l: &Light, blimp: [i64; 3]) -> [i64; 3] {
    let t12 = sh.t12;
    if l.mode == 2 {
        // the night arrival's clock: ours plus 42.5 s
        let tn = s_sum(&[(t12, 1)], 510, o);
        let (s1, _) = s_sincos(angle(tn, rate(0.55), l.phase, o), o);
        // 1.7 times the phase (radians, not wrapped: the data's thousandths) as a turn
        let ph17 = {
            let x = s_lin(&[(l.par, Some(27_056))], o);
            let whole = s_div(s_div(s_div(x, 1000, o), 1000, o), 100, o);
            let r = s_lin(&[(x, None), (whole, Some(-100_000_000))], o);
            s_div(r, 100, o)
        };
        let (s2, _) = s_sincos(angle(tn, rate(0.4), ph17, o), o);
        let u = s_div(s_lin(&[(s1, Some(850))], o), 999, o);
        let va = s_sum(&[(amp(s2, 1.25, o), 1)], -250_000, o);
        let hl = blimp_hull(o, t, u, va);
        // times 1.02 and the airship's scale, into the stadium's frame (along, across, up)
        let k = s_div(s_lin(&[(sc.bl_at[4], Some(102))], o), 100, o);
        let d: [i64; 3] = std::array::from_fn(|i| {
            let h = [hl[0], hl[2], hl[1]][i];
            let w = s_div(s_lin(&[(h, Some(k))], o), 1000, o);
            s_sum(&[(blimp[i], 1), (w, 1), (l.pos[i], -1)], 0, o)
        });
        return unit(o, d);
    }
    let az = angle(t12, rate(0.35), l.phase, o);
    let (s, _) = s_sincos(angle(t12, rate(0.6), l.phase, o), o);
    let el = s_sum(&[(amp(s, 18f64.to_radians(), o), 1)], rad(62f64.to_radians()), o);
    let (sa, ca) = s_sincos(az, o);
    let (se, ce) = s_sincos(el, o);
    [s_div(s_lin(&[(ca, Some(ce))], o), 998, o), s_div(s_lin(&[(sa, Some(ce))], o), 998, o), s_div(s_lin(&[(se, Some(1000))], o), 999, o)]
}

/// A vector made a unit (thousandths).
pub fn unit<O: Ops>(o: &mut O, d: [i64; 3]) -> [i64; 3] {
    let dd: [i64; 3] = std::array::from_fn(|i| s_div(d[i], 10, o));
    let l2 = s_lin(&[(dd[0], Some(dd[0])), (dd[1], Some(dd[1])), (dd[2], Some(dd[2]))], o);
    let l = { let r = s_sqrt(l2, o); s_least1(r, o) };
    std::array::from_fn(|i| s_clamp(s_div(s_lin(&[(dd[i], Some(1000))], o), l, o), -999, 999, o))
}

/// A laser source's beams: each one's way (thousandths) and how lit it is (thousandths): a flat fan over the heads,
/// a fan up into the sky, a rotating tunnel, a chase, by bar (at the crest the fans).
pub fn laser_beams<O: Ops>(o: &mut O, t: &Tables, sh: &Show, l: &Light, q: i64) -> Vec<([i64; 3], i64)> {
    let t12 = sh.t12;
    let n = l.par as usize;
    let crest = !s_lt(t12, CREST_T, o) && s_lt(t12, CREST_T + 24, o);
    let pat = if crest { s_divmod(q, 2, o).1 } else { s_divmod(s_sum(&[(sh.bar, 1)], q, o), 4, o).1 };
    let spread = l.half_spread;
    let wbar = { let a = cycle(t12, C_BAR, o); frac1m(a, o) };         // 2 pi t / BAR
    let mut out = vec![];
    for i in 0..n {
        // s from -1 to 1 across the fan, thousandths
        let s = s_div(s_lin(&[(i as i64, Some(2000))], o), l.gaps, o);
        let s = s_sum(&[(s, 1)], -1000, o);
        let base = -250_000;
        let (az, el) = match pat {
            0 => {
                let az = s_sum(&[(s_div(s_lin(&[(s, Some(spread))], o), 1000, o), 1)], base, o);
                let (sw, _) = s_sincos(wbar, o);
                let deg = s_sum(&[(s_div(s_lin(&[(sw, Some(1500))], o), 999, o), 1)], 3500, o);   // 2 + 3 (0.5 + 0.5 sin), thousandths of a degree
                (az, s_div(s_lin(&[(deg, Some(2778))], o), 1000, o))
            }
            1 => {
                let w2 = { let a = cycle(t12, C_BAR / 2, o); frac1m(a, o) };
                let (sw2, _) = s_sincos(w2, o);
                let (sw, _) = s_sincos(wbar, o);
                let a1 = s_div(s_lin(&[(s, Some(spread))], o), 2000, o);
                let az = s_sum(&[(a1, 1), (amp(sw2, 0.4, o), 1)], base, o);
                let deg = s_sum(&[(s_div(s_lin(&[(sw, Some(10_000))], o), 999, o), 1)], 25_000, o);
                (az, s_div(s_lin(&[(deg, Some(2778))], o), 1000, o))
            }
            2 => {
                let a = s_lin(&[(s, Some(500)), (wbar, None)], o);
                let (sa, _) = s_sincos(a, o);
                let az = s_sum(&[(amp(sa, 0.6, o), 1)], base, o);
                let deg = s_lin(&[(s, Some(12)), (16_000, None)], o);
                (az, s_div(s_lin(&[(deg, Some(2778))], o), 1000, o))
            }
            _ => (s_sum(&[(s_div(s_lin(&[(s, Some(spread))], o), 1000, o), 1)], base, o), rad(6f64.to_radians())),
        };
        let (sa, ca) = s_sincos(az, o);
        let (se, ce) = s_sincos(el, o);
        let d = [s_div(s_lin(&[(ce, Some(ca))], o), 998, o), s_div(s_lin(&[(ce, Some(sa))], o), 998, o), s_div(s_lin(&[(se, Some(1000))], o), 999, o)];
        let on = if pat == 3 {
            // one beam at a time lit, scanning across, strobing on the beat
            let k = s_divmod(t12, n as i64, o).1;
            let back = s_divmod(s_sum(&[(k, -1)], i as i64, o), n as i64, o).1;
            let e = s_tab(&t.half_decay(), back, o);
            if s_lt(sh.beat_m, 500, o) { e } else { s_div(s_lin(&[(e, Some(4))], o), 10, o) }
        } else { 1000 };
        out.push((d, on));
    }
    out
}

impl Tables {
    /// e^(-x / 2) for x 0..99 (thousandths)
    pub fn half_decay(&self) -> Tab { tab("half_decay", (0..100).map(|x| ((-0.5 * x as f64).exp() * 1000.0).round() as i64).collect()) }
}
