//! demokit: integer-only building blocks for small demos. Colour, lookup tables, arithmetic and random numbers, written
//! once and used everywhere: by the machines (built for CARDIAC through rust-cardiac, or natively), by the native
//! references that check them, and by the host.
//!
//! Nothing here knows about CARDIAC. Every value fits in three decimal digits (±999), so the same code runs on a
//! card computer and on a PC, but that is a property of the numbers, not a dependency.
//!
//! The machines and the host use it as a crate. The native references include this file as a module, so that each
//! builds with rustc alone.
#![no_std]
#![allow(dead_code)]

/// A colour: red, green and blue, 0..255 each.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pixel {
    pub r: i16,
    pub g: i16,
    pub b: i16,
}

impl Pixel {
    pub const BLACK: Pixel = Pixel { r: 0, g: 0, b: 0 };
    pub const WHITE: Pixel = Pixel { r: 255, g: 255, b: 255 };
}

/// Lookup tables. Each is the single definition: change a number here and every machine, reference and host
/// table follows.
pub mod tables {
    /// The plasma's sine: sixteen steps of one turn, 31.5 (1 + sin(22.5 k degrees)), rounded half up: 0..63.
    pub const PLASMA_SINE: [i16; 16] = [32, 44, 54, 61, 63, 61, 54, 44, 32, 19, 9, 2, 0, 2, 9, 19];
    /// The 4 x 4 ordered dither (Bayer), row by row: a pixel at (x, y) shows a level l when
    /// BAYER4[4 (y mod 4) + x mod 4] < l, for l in 0..16.
    pub const BAYER4: [i16; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

    /// The polys' highlight: 1000 h^24 for h = 0.80, 0.81 ... 1.00 (21 entries), rounded half up, the last held to
    /// 999.
    pub const SPECULAR: [i16; 21] = [5, 6, 9, 11, 15, 20, 27, 35, 47, 61, 80, 104, 135, 175, 227, 292, 375, 481, 616, 786, 999];

    /// The polys' rim light, in tenths of a step of light: 70 (1 - d)^3 for d = 0, 0.05 ... 1 (21 entries), rounded
    /// half up.
    pub const RIM: [i16; 21] = [70, 60, 51, 43, 36, 30, 24, 19, 15, 12, 9, 6, 4, 3, 2, 1, 1, 0, 0, 0, 0];
    /// The dive's caustics: a quarter of a 60-step turn, 20 sin(6 k degrees) for k = 0..15, rounded: 0..20. The
    /// rest of the turn is the same quarter turned about.
    pub const QUARTER_SINE60: [i16; 16] = [0, 2, 4, 6, 8, 10, 12, 13, 15, 16, 17, 18, 19, 20, 20, 20];

    /// The dive's glowing spots: how a spot's glow falls off, 32 exp(-k / 4) rounded for k = 0..10, k a fifth of the
    /// squared distance from its middle in tenths of a unit (exp(-5 r^2), r in units).
    pub const GLOW_FALLOFF: [i16; 11] = [32, 25, 19, 15, 12, 9, 7, 6, 4, 3, 3];

    /// How far a point is from a middle (made for the dive's portal, which has since gone; the dive's bells and kelp
    /// use it): 200 (sqrt(1 + (k / 32)^2) - 1) rounded, k = 0..32, for a point k / 32 as far across one way as the
    /// other (the distance is the larger of the two and this much of it).
    pub const PORTAL_RADIUS: [i16; 33] = [0, 0, 0, 1, 2, 2, 3, 5, 6, 8, 10, 11, 14, 16, 18, 21, 24, 26, 29, 33, 36, 39, 43,
        46, 50, 54, 58, 62, 66, 70, 74, 78, 83];

    /// The dive's portal, which way a point is from its middle: atan(k / 32) in tenths of a step of a 60-step turn,
    /// rounded, k = 0..32 (0 to 75, an eighth of the turn).
    pub const PORTAL_ANGLE: [i16; 33] = [0, 3, 6, 9, 12, 15, 18, 21, 23, 26, 29, 32, 34, 37, 39, 42, 44, 47, 49, 51, 53, 55,
        58, 60, 61, 63, 65, 67, 69, 70, 72, 73, 75];

    /// The dive's portal: its light spilling round it, 32 0.35 exp(-k / 4) rounded, k = 0..10 units past the ring.
    pub const PORTAL_SPILL: [i16; 11] = [11, 9, 7, 5, 4, 3, 2, 2, 2, 1, 1];

    /// The dive's jellyfish: how they fade with distance, 64 exp(-2 k / 14) rounded, k = 0..30 half their distance.
    pub const JELLY_FADE: [i16; 31] = [64, 55, 48, 42, 36, 31, 27, 24, 20, 18, 15, 13, 12, 10, 9, 8, 7, 6, 5, 4, 4, 3, 3, 2, 2,
        2, 2, 1, 1, 1, 1];

    /// The dive's jellyfish: their halo, 32 0.35 exp(-q / 10) rounded, q = 0..48 tenths of the squared distance
    /// from the middle over twice the halo's squared width.
    pub const JELLY_HALO: [i16; 49] = [11, 10, 9, 8, 8, 7, 6, 6, 5, 5, 4, 4, 3, 3, 3, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    /// The dive's jellyfish bells: twelve rings from the top down to the rim, 100 sin and 100 cos of
    /// 1.5 (ring + 0.5) / 12 radians, rounded.
    pub const BELL_SIN: [i16; 12] = [6, 19, 31, 42, 53, 63, 73, 81, 87, 93, 97, 99];
    pub const BELL_COS: [i16; 12] = [100, 98, 95, 91, 85, 77, 69, 59, 49, 37, 26, 13];

    /// The dive's jellyfish bells: how a point glows by how side-on it is seen, 8 + 24 (e / 32)^2 rounded, e = 0..31
    /// (32nds of edge-on): 0.25 + 0.75 e^2 in 32nds.
    pub const BELL_GLOW: [i16; 32] = [8, 8, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 12, 13, 13, 14, 15, 16, 16, 17, 18, 19, 20,
        22, 23, 24, 25, 26, 28, 29, 31];

    /// The dive's kelp and seagrass blades: how wide a blade is along its length, 100 sin(pi u)^0.7 rounded, u in
    /// 20ths, tapering to points at both ends.
    pub const LEAF_TAPER: [i16; 21] = [0, 27, 44, 58, 69, 78, 86, 92, 97, 99, 100, 99, 97, 92, 86, 78, 69, 58, 44, 27, 0];

    /// The dive's kelp and fish fade into the water with distance: 32 (1 - exp(-d / 20)) and 32 (1 - exp(-d / 38))
    /// rounded, d = 0..49 whole units.
    pub const KELP_FOG: [i16; 50] = [0, 2, 3, 4, 6, 7, 8, 9, 11, 12, 13, 14, 14, 15, 16, 17, 18, 18, 19, 20, 20, 21, 21, 22, 22,
        23, 23, 24, 24, 24, 25, 25, 26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 28, 28, 29, 29, 29, 29, 29];
    pub const FISH_FOG: [i16; 50] = [0, 1, 2, 2, 3, 4, 5, 5, 6, 7, 7, 8, 9, 9, 10, 10, 11, 12, 12, 13, 13, 14, 14, 15, 15,
        15, 16, 16, 17, 17, 17, 18, 18, 19, 19, 19, 20, 20, 20, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23];

    /// A quarter of a 64-step turn: 100 cos(2 pi k / 64), k = 0..16, rounded.
    pub const QUARTER_COS64: [i16; 17] = [100, 100, 98, 96, 92, 88, 83, 77, 71, 63, 56, 47, 38, 29, 20, 10, 0];

    /// The dive's rock: a column's place in the rock tables, by its unit along x (mod 16); the row (mod 16) is added
    /// to it (mod 16), so neighbouring columns and rows land in different places.
    pub const U_ROCK_PLACE: [i16; 16] = [7, 12, 3, 9, 0, 14, 5, 10, 2, 15, 8, 1, 11, 6, 13, 4];
    /// The dive's rock, per place: 10 s + j + 5, the strata shifted s tones there and the rock j 32nds lighter.
    pub const U_ROCK_VARY: [i16; 16] = [5, 6, 4, 5, 14, 5, 7, 3, 6, 5, 4, 6, 34, 5, 6, 5];
    /// The dive's rock up close: a grain of chips and cracks, 32nds lighter, 4 x 4 across a face.
    pub const U_ROCK_GRAIN: [i16; 16] = [0, 2, -3, 1, -1, 3, 0, -4, 2, -2, 1, 0, -3, 2, -1, 1];

    /// The plasma's waves at 192x96, a quarter of a turn each: 32 sin(2 pi k / 96), k = 0..24, for the waves across,
    /// rolling and slanting, and 32 sin(2 pi k / 64), k = 0..16, for the rings; rounded half up. A wave is 32 plus or
    /// minus it: 0..64.
    pub const PL_WAVE96: [i16; 25] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 19, 21, 23, 24, 25, 27, 28, 29, 30, 30, 31, 31, 32, 32, 32];
    pub const PL_WAVE64: [i16; 17] = [0, 3, 6, 9, 12, 15, 18, 20, 23, 25, 27, 28, 30, 31, 31, 32, 32];

    /// A table as i32, for code that works in i32 (the native references): `const T: [i32; 16] = widen(BAYER4);`
    pub const fn widen<const N: usize>(t: [i16; N]) -> [i32; N] {
        let mut out = [0i32; N];
        let mut i = 0;
        while i < N { out[i] = t[i] as i32; i += 1; }
        out
    }
}

/// Arithmetic with adding and subtracting only, for a computer with no multiply or divide. Each loop runs about
/// as many times as its answer, so these suit small answers.
pub mod arith {
    /// The size of v, |v|.
    pub fn abs(v: i16) -> i16 {
        if v < 0 { -v } else { v }
    }

    /// How far apart a and b are, |a - b|.
    pub fn abs_diff(a: i16, b: i16) -> i16 {
        if a < b { b - a } else { a - b }
    }

    /// The integer square root of n (0 for n < 1): count the odd numbers 1, 3, 5, ... that can be taken away,
    /// since their sums are the squares.
    pub fn isqrt(mut n: i16) -> i16 {
        let mut root = 0;
        let mut odd = 1;
        while n >= odd {
            n -= odd;
            odd += 2;
            root += 1;
        }
        root
    }

    /// v brought into 0..n-1 by one step round, for v in -n..2n-1: an angle or a place on a map that wraps.
    pub fn wrap(v: i16, n: i16) -> i16 {
        if v < 0 { v + n } else if v >= n { v - n } else { v }
    }

    /// v mod m, for v >= 0 and m >= 1: m taken away while it fits.
    pub fn remainder(v: i16, m: i16) -> i16 {
        let mut left = v;
        while left >= m {
            left -= m;
        }
        left
    }

    /// a b / c rounded down, for a, b >= 0 and c >= 1, without the product, which would pass a card: b added a
    /// times, c taken away whenever the sum reaches it. The sum stays below b + c, so that must fit a card, and so
    /// must the answer.
    #[inline(never)]
    pub fn muldiv(a: i16, b: i16, c: i16) -> i16 {
        let mut sum = 0;
        let mut q = 0;
        for _ in 0..a {
            sum += b;
            while sum >= c {
                sum -= c;
                q += 1;
            }
        }
        q
    }

    /// n / d and n mod d, for n >= 0 and d >= 1: d taken away while it fits, counting.
    pub fn divide(n: i16, d: i16) -> (i16, i16) {
        let mut rest = n;
        let mut quotient = 0;
        while rest >= d {
            rest -= d;
            quotient += 1;
        }
        (quotient, rest)
    }
}

/// Multiplying without a multiplier, and without a product that leaves three digits.
pub mod mul {
    /// a d, for a small d >= 0 (a digit, say): a added d times. Not inlined: a machine's multiplications share one
    /// copy.
    #[inline(never)]
    pub fn times(a: i16, d: i16) -> i16 {
        let mut sum = 0;
        let mut left = d;
        while left > 0 {
            sum += a;
            left -= 1;
        }
        sum
    }

    /// c v / 100, toward zero, for -99 <= c <= 99 and -999 <= v <= 999: c times v's digits, units first, the sum
    /// so far cut to a tenth before each: ((c u / 10 + c t) / 10) + c h. Every part has the product's sign, so cutting
    /// each toward zero cuts the whole the same, and nothing on the way passes 989.
    #[inline(never)]
    pub fn by100(c: i16, v: i16) -> i16 {
        let (mut add, mut rest) = (c, v);
        if rest < 0 { add = -add; rest = -rest; }
        let mut sum = 0;
        for _digit in 0..3 {
            sum /= 10;
            let mut d = rest % 10;
            rest /= 10;
            while d > 0 {
                sum += add;
                d -= 1;
            }
        }
        sum
    }

    /// v f / 100 rounded down, for v, f >= 0, without the product, which would pass a card: f times, v's hundreds
    /// added to the answer and its rest to a running remainder, a hundred of which carries one more. Nothing on the
    /// way passes 199 or the answer. Not inlined: one copy a machine.
    #[inline(never)]
    pub fn percent(v: i16, f: i16) -> i16 {
        let hundreds = v / 100;
        let rest = v % 100;
        let mut whole = 0;
        let mut part = 0;
        for _ in 0..f {
            whole += hundreds;
            part += rest;
            if part >= 100 {
                part -= 100;
                whole += 1;
            }
        }
        whole
    }
}

/// Random numbers: the step s -> (21 s + 17) mod 1000, which visits all 1000 seeds.
pub mod lcg {
    /// The next seed after s, for s in 0..999. Worked out so that nothing on the way leaves three digits:
    /// 20 s mod 1000 is 10 (2 s mod 100), and a + s + 17 reaches 1000 just when a > 982 - s (never 2000).
    /// Not inlined: a machine that rolls several numbers keeps one copy and calls it.
    #[inline(never)]
    pub fn next(s: i16) -> i16 {
        let a = s * 2 % 100 * 10;
        let room = 982 - s;
        if a > room { a - room - 1 } else { a + s + 17 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_is_the_definition() {
        for s in 0..1000i32 {
            assert_eq!(lcg::next(s as i16) as i32, (21 * s + 17) % 1000, "seed {s}");
        }
    }

    #[test]
    fn lcg_visits_every_seed() {
        let (mut s, mut seen) = (0i16, [false; 1000]);
        for _ in 0..1000 { seen[s as usize] = true; s = lcg::next(s); }
        assert!(seen.iter().all(|&v| v));
    }

    #[test]
    fn sines_are_what_they_say() {
        for k in 0..16 {
            let a = (k as f64 * 22.5).to_radians().sin();
            assert_eq!(tables::PLASMA_SINE[k] as f64, (31.5 * (1.0 + a)).round(), "plasma {k}");
        }
    }

    #[test]
    fn dive_sine_is_what_it_says() {
        for k in 0..16 {
            let a = (k as f64 * 6.0).to_radians().sin();
            assert_eq!(tables::QUARTER_SINE60[k] as f64, (20.0 * a).round(), "dive sine {k}");
        }
    }

    #[test]
    fn glow_falloff_is_what_it_says() {
        for k in 0..11 {
            assert_eq!(tables::GLOW_FALLOFF[k] as f64, (32.0 * (-(k as f64) / 4.0).exp()).round(), "glow {k}");
        }
    }

    #[test]
    fn portal_tables_are_what_they_say() {
        for k in 0..33 {
            let t = k as f64 / 32.0;
            assert_eq!(tables::PORTAL_RADIUS[k] as f64, (200.0 * ((1.0 + t * t).sqrt() - 1.0)).round(), "radius {k}");
            assert_eq!(tables::PORTAL_ANGLE[k] as f64, (10.0 * t.atan() * 60.0 / (2.0 * core::f64::consts::PI)).round(), "angle {k}");
        }
    }

    #[test]
    fn portal_spill_is_what_it_says() {
        for k in 0..11 {
            assert_eq!(tables::PORTAL_SPILL[k] as f64, (32.0 * 0.35 * (-(k as f64) / 4.0).exp()).round(), "spill {k}");
        }
    }

    #[test]
    fn jelly_tables_are_what_they_say() {
        for k in 0..31 { assert_eq!(tables::JELLY_FADE[k] as f64, (64.0 * (-2.0 * k as f64 / 14.0).exp()).round(), "fade {k}"); }
        for q in 0..49 { assert_eq!(tables::JELLY_HALO[q] as f64, (32.0 * 0.35 * (-(q as f64) / 10.0).exp()).round(), "halo {q}"); }
        for r in 0..12 {
            let th = 1.5 * (r as f64 + 0.5) / 12.0;
            assert_eq!(tables::BELL_SIN[r] as f64, (100.0 * th.sin()).round(), "bell sin {r}");
            assert_eq!(tables::BELL_COS[r] as f64, (100.0 * th.cos()).round(), "bell cos {r}");
        }
        for u in 0..21 { assert_eq!(tables::LEAF_TAPER[u] as f64, (100.0 * (core::f64::consts::PI * u as f64 / 20.0).sin().powf(0.7)).round(), "taper {u}"); }
        for d in 0..50 {
            assert_eq!(tables::KELP_FOG[d] as f64, (32.0 * (1.0 - (-(d as f64) / 20.0).exp())).round(), "kelp fog {d}");
            assert_eq!(tables::FISH_FOG[d] as f64, (32.0 * (1.0 - (-(d as f64) / 38.0).exp())).round(), "fish fog {d}");
        }
        for e in 0..32 { assert_eq!(tables::BELL_GLOW[e] as f64, 8.0 + (24.0 * (e as f64 / 32.0).powi(2)).round(), "glow {e}"); }
        for k in 0..17 { assert_eq!(tables::QUARTER_COS64[k] as f64, (100.0 * (core::f64::consts::TAU * k as f64 / 64.0).cos()).round(), "cos64 {k}"); }
    }

    #[test]
    fn abs_and_abs_diff_are_the_definition() {
        for v in -999..=999i16 {
            assert_eq!(arith::abs(v), v.abs(), "abs {v}");
        }
        for a in -500..=500i16 {
            for b in -499..=499i16 {
                assert_eq!(arith::abs_diff(a, b), (a - b).abs(), "abs_diff {a} {b}");
            }
        }
    }

    #[test]
    fn isqrt_is_the_definition() {
        for n in -999..=999i16 {
            let root = if n < 1 { 0 } else { (n as f64).sqrt().floor() as i16 };
            assert_eq!(arith::isqrt(n), root, "isqrt {n}");
        }
    }

    #[test]
    fn wrap_is_the_definition() {
        for n in 1..=480i16 {
            for v in -n..2 * n {
                assert_eq!(arith::wrap(v, n), v.rem_euclid(n), "wrap {v} {n}");
            }
        }
    }

    #[test]
    fn remainder_and_divide_are_the_definition() {
        for d in 1..=999i16 {
            for n in 0..=999i16 {
                assert_eq!(arith::remainder(n, d), n % d, "remainder {n} {d}");
                assert_eq!(arith::divide(n, d), (n / d, n % d), "divide {n} {d}");
            }
        }
    }

    #[test]
    fn mul_is_the_definition() {
        for c in -99..=99i16 {
            for v in -999..=999i16 {
                assert_eq!(mul::by100(c, v) as i32, c as i32 * v as i32 / 100, "by100 {c} {v}");
            }
        }
    }

    #[test]
    fn polys_tables_are_what_they_say() {
        for i in 0..21 {
            let h = (800 + 10 * i) as f64 / 1000.0;
            assert_eq!(tables::SPECULAR[i] as f64, (1000.0 * libm_pow(h, 24) + 0.5).floor().min(999.0), "specular {i}");
        }
        for i in 0..21 {
            let d = 1.0 - i as f64 / 20.0;
            assert_eq!(tables::RIM[i] as f64, (70.0 * d * d * d + 0.5).floor(), "rim {i}");
        }
    }

    fn libm_pow(x: f64, n: u32) -> f64 { (0..n).fold(1.0, |p, _| p * x) }

    #[test]
    fn muldiv_is_the_definition() {
        for a in 0..=40i16 { for b in 0..=600i16 { for c in 1..=(999 - b).min(399) {
            if a as i32 * b as i32 / c as i32 <= 999 { assert_eq!(arith::muldiv(a, b, c) as i32, a as i32 * b as i32 / c as i32, "{a} {b} {c}"); }
        } } }
    }

    #[test]
    fn percent_is_the_definition() {
        for v in 0..=999i32 { for f in 0..=400i32 {
            if v * f / 100 <= 999 { assert_eq!(mul::percent(v as i16, f as i16) as i32, v * f / 100, "percent {v} {f}"); }
        } }
    }

    #[test]
    fn plasma_waves_are_what_they_say() {
        let q = |n: f64, k: usize| (32.0 * (2.0 * core::f64::consts::PI * k as f64 / n).sin() + 0.5).floor();
        for k in 0..25 { assert_eq!(tables::PL_WAVE96[k] as f64, q(96.0, k), "wave96 {k}"); }
        for k in 0..17 { assert_eq!(tables::PL_WAVE64[k] as f64, q(64.0, k), "wave64 {k}"); }
    }

    #[test]
    fn bayer_is_a_permutation() {
        let mut v = tables::BAYER4; v.sort();
        assert_eq!(v, core::array::from_fn(|i| i as i16));
    }
}
