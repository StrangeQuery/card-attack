// The stadium's printed tables and the small sums every part uses. Included after scene.rs.
//
// Products and quotients go by logarithms where one side is known not to be negative, as the train's do: the table
// of ln |x| (ten-thousandths, for any long x), the sign's table, and e back (or -e) by the table of e: one sum on
// CARDIAC for a product of any number of factors. Every other curve (the beat's decay, the tone curve, a square root)
// is a printed table looked up at a number a machine punched.

/// ln 0, as the tables give it: far enough down that a product with it comes to 0.
pub const LN0: i64 = -400_000;
/// The table of e covers ln values from EXP_LO up (ten-thousandths).
pub const EXP_LO: i64 = -1_000_000;
pub const EXP_HI: i64 = 210_000;
/// ln 10, ln 100, ln 1000, ln 10,000 (ten-thousandths)
pub const LN10: i64 = 23_026;
pub const LN100: i64 = 46_052;
pub const LN1000: i64 = 69_078;
pub const LN10K: i64 = 92_103;
/// The tone curve's index at most (light in ten-thousandths): past it the byte is 255.
pub const TONE_N: i64 = 60_000;

pub fn lnf(x: f64) -> i64 { (x.ln() * 10_000.0).round() as i64 }

pub struct Tables {
    pub trig: Vec<i64>,
    /// ln |x|, ten-thousandths, x -999,999..999,999 (offset 999,999)
    pub ln: Tab,
    /// 1 where x < 0 (offset 999,999)
    pub neg: Tab,
    /// e and -e of y / 10,000, rounded, at most 999,999 (offset -EXP_LO)
    pub exp: Tab,
    pub expn: Tab,
    /// the same up to three cards (slopes)
    pub expw: Tab,
    pub expwn: Tab,
    /// a byte of a picture as linear light (the prototype's srgb_lin), ten-thousandths, and its logarithm
    pub srgb: Tab,
    pub ln_srgb: Tab,
    /// the tone curve: light (ten-thousandths, 0..TONE_N) to the byte, 255 (1 - e^-1.25 x)^(1 / 2.2)
    pub tone: Tab,
    /// Hermite's four, by u in thousandths (thousandths)
    pub herm: [Tab; 4],
    /// e^(-k x) for x 0..999 thousandths of a cycle: the lights' decays (k 5, 3), thousandths
    pub decay5: Tab,
    pub decay4: Tab,
    pub decay3: Tab,
    /// e^(-1.2 s) for s in twelfths of a second after the crest, 0..999 (thousandths)
    pub crest: Tab,
    /// a square root: of x 0..9,999,999 by its first four digits (see sqrtv)
    pub sqrt: Tab,
    /// cos and sin of atan(r / 1000), thousandths
    pub cos_atan: Tab,
    pub sin_atan: Tab,
    /// a byte of a picture as linear light in thousandths, at most 999 (for means)
    pub srgb_th: Tab,
    pub ln_sq99: Tab,
    pub ln_sq999: Tab,
    pub rim: Tab,
    pub inv_ray: Tab,
    pub sky: [[Tab; 3]; 2],
    pub dissolve: Tab,
    pub mq_wrap: Tab,
    pub dot: Tab,
    pub hue: [Tab; 3],
    pub skin_pulse: Tab,
    pub flick: Tab,
    pub hyp: Tab,
    pub haze_k: Tab,
    pub unit2: Tab,
    pub sq577: Tab,
    pub sq999: Tab,
    pub sqrt_m: Tab,
    pub ball_rim: Tab,
}

fn tab(name: &'static str, data: Vec<i64>) -> Tab { Tab { name, data, w: 0 } }

pub fn tables(dir: &str) -> Tables {
    const L: i64 = 999_999;
    let ln = (-L..=L).map(|x| if x == 0 { LN0 } else { lnf(x.abs() as f64) }).collect();
    let neg = (-L..=L).map(|x| (x < 0) as i64).collect();
    let e = |y: i64| -> i64 { ((y as f64 / 10_000.0).exp().round() as i64).min(L) };
    let exp: V = (EXP_LO..EXP_HI).map(e).collect();
    let expn: V = exp.iter().map(|&x| -x).collect();
    let srgb: V = (0..256).map(|b| ((b as f64 / 255.0).powf(2.2) * 10_000.0).round() as i64).collect();
    let ln_srgb = srgb.iter().map(|&x| if x == 0 { LN0 } else { lnf(x as f64) }).collect();
    let tone = (0..=TONE_N).map(|x| {
        let v = (1.0 - (-1.25 * x as f64 / 10_000.0).exp()).clamp(0.0, 1.0).powf(1.0 / 2.2);
        ((v * 255.0 + 0.5) as i64).min(255)
    }).collect();
    let h = |f: &dyn Fn(f64) -> f64| -> Tab { tab("herm", (0..=1000).map(|u| (f(u as f64 / 1000.0) * 1000.0).round() as i64).collect()) };
    let herm = [h(&|u| 2.0 * u * u * u - 3.0 * u * u + 1.0), h(&|u| u * u * u - 2.0 * u * u + u), h(&|u| -2.0 * u * u * u + 3.0 * u * u), h(&|u| u * u * u - u * u)];
    let dec = |k: f64| -> V { (0..1000).map(|x| ((-k * x as f64 / 1000.0).exp() * 1000.0).round() as i64).collect() };
    let crest = (0..1000).map(|s| ((-1.2 * s as f64 / 12.0).exp() * 1000.0).round() as i64).collect();
    let sqrt = (0..10_000).map(|x| ((x as f64).sqrt() * 1000.0).round() as i64).collect();
    Tables {
        trig: read_trig(dir),
        ln: tab("ln", ln), neg: tab("neg", neg), exp: tab("exp", exp), expn: tab("expn", expn), expw: Tables::expw_tabs().0, expwn: Tables::expw_tabs().1,
        srgb: tab("srgb", srgb), ln_srgb: tab("ln_srgb", ln_srgb), tone: tab("tone", tone), herm,
        decay5: tab("decay5", dec(5.0)), decay4: tab("decay4", dec(4.0)), decay3: tab("decay3", dec(3.0)),
        crest: tab("crest", crest), sqrt: tab("sqrt", sqrt), cos_atan: Tables::atan_tabs().0, sin_atan: Tables::atan_tabs().1,
        ln_sq99: Tables::ln_sq99_tab(), ln_sq999: Tables::ln_sq999_tab(), rim: Tables::rim_tab(), inv_ray: Tables::inv_ray_tab(), sky: Tables::sky_tabs(),
        dissolve: Tables::dissolve_tab(), mq_wrap: Tables::mq_wrap_tab(17_360), dot: Tables::dot_tab(), hue: Tables::hue_tabs(),
        skin_pulse: Tables::skin_pulse_tab(), flick: Tables::flick_tab(), hyp: hyp_tab(), haze_k: Tables::haze_k_tab(), unit2: Tables::unit2_tab(), sq577: Tables::sq577_tab(), sq999: Tables::sq999_tab(), sqrt_m: Tables::sqrt_m_tab(), ball_rim: Tables::ball_rim_tab(),
        srgb_th: tab("srgb_th", (0..256).map(|b| (((b as f64 / 255.0).powf(2.2) * 1000.0).round() as i64).min(999)).collect()),
    }
}

pub fn read_trig(dir: &str) -> Vec<i64> {
    let text = std::fs::read_to_string(format!("{dir}/swarm/data/tables.txt")).unwrap();
    let l = text.lines().find(|l| l.split_whitespace().next() == Some("trig")).unwrap();
    l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect()
}

/// Each vector's one number.
pub fn one(v: V) -> i64 { assert_eq!(v.len(), 1, "one number"); v[0] }

/// ln |x| and whether x < 0, for longs.
pub fn lnv<O: Ops>(o: &mut O, t: &Tables, x: &V) -> (V, V) { (o.tabo(&t.ln, 999_999, v(x)), o.tabo(&t.neg, 999_999, v(x))) }

/// e of ln values (ten-thousandths), negative where neg is 1: the host's look-ups (the sum that made y is CARDIAC's).
pub fn expv<O: Ops>(o: &mut O, t: &Tables, y: &V, neg: Option<&V>) -> V {
    match neg {
        None => o.tabo(&t.exp, -EXP_LO, v(y)),
        Some(ng) => {
            let mut out = vec![0; y.len()];
            for (table, want) in [(&t.exp, 0), (&t.expn, 1)] {
                let at: Vec<usize> = (0..y.len()).filter(|&i| ng[i] == want).collect();
                if at.is_empty() { continue; }
                let yy: V = at.iter().map(|&i| y[i]).collect();
                let e = o.tabo(table, -EXP_LO, v(&yy));
                for (k, &i) in at.iter().enumerate() { out[i] = e[k]; }
            }
            out
        }
    }
}

/// Products by logarithms: e of the sum of the factors' logarithms (each a vector of ln values or a constant) plus c,
/// with the sign of the one signed factor (neg) if any.
pub fn lprod<O: Ops>(o: &mut O, t: &Tables, lns: &[A], c: i64, neg: Option<&V>) -> V {
    let terms: Vec<(A, i64)> = lns.iter().map(|&a| (a, 1)).collect();
    let y = o.sum(&terms, c);
    expv(o, t, &y, neg)
}

/// x times a factor f >= 0 given as its logarithm (a vector or one for all), over 10^s: sign kept.
pub fn lscale<O: Ops>(o: &mut O, t: &Tables, x: &V, lnf_: A, s: i64) -> V {
    let (l, ng) = lnv(o, t, x);
    lprod(o, t, &[v(&l), lnf_], -s * LN10, Some(&ng))
}

/// a b / 10^s for any numbers whose product fits three cards: CARDIAC's exact product (lin), then the shift.
pub fn mulx<O: Ops>(o: &mut O, a: A, b: A, s: u32) -> V {
    let p = o.lin(&[(a, Some(b))]);
    if s == 0 { return p; }
    o.div(v(&p), K(10i64.pow(s)), 0)
}

/// The whole square root of x (0..999,999,999): the op's.
pub fn sqrtv<O: Ops>(o: &mut O, x: &V) -> V { o.sqrt(v(x)) }

/// x / d rounded down for vectors, the cheap way where d divides a power of ten: x times 10^k / d, then the digits
/// moved down (exact: the floor of a floor); else the long division.
pub fn divk<O: Ops>(o: &mut O, x: &V, d: i64) -> V {
    let (k, p) = match d { 2 => (5, 10), 4 => (25, 100), 5 => (2, 10), 20 => (5, 100), 25 => (4, 100), 50 => (2, 100), 250 => (4, 1000), 16 => (625, 10_000), 10_000 => (1, 10_000), _ => return o.div(v(x), K(d), 0) };
    let y = if k == 1 { x.clone() } else if k == 2 { o.sum(&[(v(x), 2)], 0) } else { o.lin(&[(v(x), Some(K(k)))]) };
    if p == 10_000 { let a = o.div(v(&y), K(1000), 0); o.div(v(&a), K(10), 0) } else { o.div(v(&y), K(p), 0) }
}

/// One-number helpers for the frame's own numbers (every argument one number: the operation runs once).
pub fn s_lin<O: Ops>(terms: &[(i64, Option<i64>)], o: &mut O) -> i64 {
    let t: Vec<(A, Option<A>)> = terms.iter().map(|&(a, b)| (K(a), b.map(K))).collect();
    one(o.lin(&t))
}
pub fn s_div<O: Ops>(x: i64, d: i64, o: &mut O) -> i64 { one(o.div(K(x), K(d), 0)) }
pub fn s_sum<O: Ops>(terms: &[(i64, i64)], c: i64, o: &mut O) -> i64 {
    let t: Vec<(A, i64)> = terms.iter().map(|&(a, k)| (K(a), k)).collect();
    one(o.sum(&t, c))
}
pub fn s_clamp<O: Ops>(x: i64, lo: i64, hi: i64, o: &mut O) -> i64 { one(o.clamp(K(x), lo, hi)) }
pub fn s_lt<O: Ops>(a: i64, b: i64, o: &mut O) -> bool { one(o.lt(K(a), K(b))) == 1 }
pub fn s_max<O: Ops>(a: i64, b: i64, o: &mut O) -> i64 { one(o.max(K(a), K(b))) }
/// -x, on the machines (the host's sign flips).
pub fn s_neg<O: Ops>(x: i64, o: &mut O) -> i64 { s_lin(&[(x, Some(-1))], o) }
/// x held to 1 or more (a divisor), on the machines.
pub fn s_least1<O: Ops>(x: i64, o: &mut O) -> i64 { s_clamp(x, 1, 998_999_999, o) }
pub fn s_min<O: Ops>(a: i64, b: i64, o: &mut O) -> i64 { one(o.min(K(a), K(b))) }
pub fn s_tab<O: Ops>(t: &Tab, i: i64, o: &mut O) -> i64 { one(o.tab(t, K(i))) }
pub fn s_tab_o<O: Ops>(t: &Tab, off: i64, i: i64, o: &mut O) -> i64 { one(o.tabo(t, off, K(i))) }
pub fn s_sincos<O: Ops>(a: i64, o: &mut O) -> (i64, i64) { let a = one(o.turns(K(a))); let (s, c) = o.sincos(K(a)); (one(s), one(c)) }
/// |x|: x or its negative (CARDIAC's), by its sign.
pub fn s_abs<O: Ops>(x: i64, o: &mut O) -> i64 { let n = s_sum(&[(x, -1)], 0, o); if s_lt(x, 0, o) { n } else { x } }
pub fn s_sqrt<O: Ops>(x: i64, o: &mut O) -> i64 { one(o.sqrt(K(x))) }
pub fn s_divmod<O: Ops>(x: i64, d: i64, o: &mut O) -> (i64, i64) { let (q, r) = o.divmod(K(x), K(d)); (one(q), one(r)) }
