// The train's arithmetic: every number the renderer works out (render.rs) goes through these operations, on vectors
// (one number an item: a pixel, a corner, a triangle; a frame's own numbers are vectors of one). Each operation has
// an exact meaning in whole numbers, written out here as `Native` does it: the reference runs the renderer with
// `Native`, the host with machines that do the same on CARDIAC (host/src/fx/train.rs), and the frames must match.
//
// Numbers are of three sizes: a card (-999..999), a long (two cards, thousands and rest: -999,999..999,999) and
// three cards (millions, thousands, rest: under 999,000,000 in size). An operation says what it takes and gives; the
// native one stops if a number falls outside.

pub type V = Vec<i64>;

/// An argument: a vector, or one number for every item.
#[derive(Clone, Copy)]
pub enum A<'a> { V(&'a [i64]), K(i64) }
pub use A::K;
pub fn v(x: &V) -> A<'_> { A::V(x) }

impl A<'_> {
    pub fn at(&self, i: usize) -> i64 { match self { A::V(v) => v[i], A::K(k) => *k } }
    pub fn len(&self) -> Option<usize> { match self { A::V(v) => Some(v.len()), A::K(_) => None } }
}

/// A printed table: numbers by an index (a card, a long or two cards), or by two (rows of `w`).
pub struct Tab { pub name: &'static str, pub data: Vec<i64>, pub w: usize }

pub fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
pub fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }

pub const CARD: i64 = 999;
pub const LONG: i64 = 999_999;
pub const WIDE: i64 = 998_999_999;

/// The card sorter the host puts cards in order with, by numbers they hold (host/src/sorter.rs).
pub mod sorter { include!("../../host/src/sorter.rs"); }

/// How long the vectors are (every vector argument the same).
pub fn n_of(args: &[A]) -> usize {
    let mut n = None;
    for a in args { if let Some(l) = a.len() { assert!(n.is_none_or(|m| m == l), "vectors of {} and {}", n.unwrap(), l); n = Some(l); } }
    n.unwrap_or(1)
}

fn within(name: &str, v: i64, lim: i64) -> i64 {
    // a long's thousands are a card too: -999,000 the least
    assert!(v.abs() <= lim && (lim != LONG || v >= -999_000), "{name}: {v} past {lim}");
    v
}

/// The operations. Defaults are written in terms of the others; an implementation gives its own where it has a
/// machine for it.
pub trait Ops {
    /// Counting, for the reports (the reference counts its operations' items).
    fn note(&mut self, _op: &'static str, _n: usize) {}
    /// Which part of the frame the operations are for (the reports).
    fn stage(&mut self, _name: &'static str) {}
    // --- the host's part: tables and picks (no arithmetic) -------------------------------------------------------

    /// The table's entry at each index (a card or a long, 0 to its length less 1).
    fn tab(&mut self, t: &Tab, i: A) -> V;
    /// The table's entry at row i, column j.
    fn tab2(&mut self, t: &Tab, i: A, j: A) -> V;
    /// a where the flag is 1, b where it is 0.
    fn sel(&mut self, flag: &[i64], a: A, b: A) -> V;
    /// The entry of a table printed from -off at each index (from -off to its length less off, less 1).
    fn tabo(&mut self, t: &Tab, off: i64, i: A) -> V {
        let n = n_of(&[i]);
        self.note("tab", n);
        map1(i, n, |k| { assert!(k + off >= 0 && ((k + off) as usize) < t.data.len(), "{}: index {k}", t.name); t.data[(k + off) as usize] })
    }

    // --- cards -------------------------------------------------------------------------------------------------

    /// k1 a1 + k2 a2 + ... + c, the k's -9..9, every value and the sum a long.
    fn sum(&mut self, terms: &[(A, i64)], c: i64) -> V;
    /// a t / 100 cut toward zero, a a card, t -100..100.
    fn pct(&mut self, a: A, t: A) -> V;
    /// a + (b - a) t / 100, cut toward zero; b - a and t as for pct.
    fn mix(&mut self, a: A, b: A, t: A) -> V;
    /// fl(a b, 10^s): cards a, b; the result a long (s 0..6).
    fn mul(&mut self, a: A, b: A, s: u32) -> V;
    /// a held to lo..hi.
    fn clamp(&mut self, a: A, lo: i64, hi: i64) -> V;
    /// 1 where a < b, else 0 (any numbers).
    fn lt(&mut self, a: A, b: A) -> V;
    /// The larger, the smaller (cards).
    fn max(&mut self, a: A, b: A) -> V;
    fn min(&mut self, a: A, b: A) -> V;
    /// fl(a, d) and md(a, d) for a number a and a whole d 1..999 (a quotient under a million in size).
    fn divmod(&mut self, a: A, d: A) -> (V, V);

    // --- longs and three cards (exact) ---------------------------------------------------------------------------

    /// Sums of products, exact: each term a b (or a alone), any numbers whose product is under 999,000,000 in size.
    /// The sum under 999,000,000 in size.
    fn lin(&mut self, terms: &[(A, Option<A>)]) -> V;
    /// fl(x 1000^extra, d): x three cards, d 1..99,999; the quotient under a million in size.
    fn div(&mut self, x: A, d: A, extra: u32) -> V;
    /// 999 sin and 999 cos of an angle in millionths of a turn (the swarm's table, by tenths).
    fn sincos(&mut self, a: A) -> (V, V);
    /// The whole square root of a number 0..9,999,999.
    fn sqrt(&mut self, x: A) -> V;
    /// fl(a b, d): cards a and b, d 1..999; the result a long.
    fn muldiv(&mut self, a: A, b: A, d: i64) -> V {
        let n = n_of(&[a, b]);
        self.note("muldiv", n);
        map2(a, b, n, |x, y| { within("muldiv", x, CARD); within("muldiv", y, CARD); assert!((1..=999).contains(&d)); within("muldiv", fl(x * y, d), LONG) })
    }
    /// md(fl(u, d), 1000): a place's index in a table that repeats every 1000 (u a long or three cards, d 1 or 10).
    fn wrap(&mut self, u: A, d: i64) -> V { let n = n_of(&[u]); self.note("wrap", n); map1(u, n, |x| md(fl(within("wrap", x, WIDE), d), 1000)) }
    /// Where a place falls in a pattern repeating every 10^s / r: t = u r; the pattern's cell md(fl(t, 10^s), 1000)
    /// and how far into it, thousandths: fl(1000 md(t, 10^s), 10^s). u a long, r 1..999, s 3..5.
    fn phase(&mut self, u: A, r: i64, s: u32) -> (V, V) {
        let n = n_of(&[u]);
        self.note("phase", n);
        let p = 10i64.pow(s);
        assert!((1..=999).contains(&r) && (3..=5).contains(&s), "phase rate {r} / 10^{s}");
        let t: Vec<i64> = (0..n).map(|i| within("phase", u.at(i), LONG) * r).collect();
        (t.iter().map(|&t| md(fl(t, p), 1000)).collect(), t.iter().map(|&t| fl(1000 * md(t, p), p)).collect())
    }
    /// md(x, 1,000,000): an angle's whole turns dropped (x three cards).
    fn turns(&mut self, x: A) -> V { let n = n_of(&[x]); self.note("turns", n); map1(x, n, |a| md(within("turns", a, WIDE), 1_000_000)) }

    // --- drawing ---------------------------------------------------------------------------------------------------

    /// Each edge's crossings of the screen's rows. An edge from a to b (x, y: hundredths of a pixel from the middle,
    /// y down, a.y <= b.y) meets the rows from first = fl(a.y + 49, 100) (rows' middles are at 100 row + 50) to short
    /// of fl(b.y + 49, 100); for each of those on the screen (-384..383): the row, the first pixel whose middle is at
    /// or right of the crossing, fl(x + 49, 100) with x = a.x + fl((b.x - a.x) (100 row + 50 - a.y), b.y - a.y),
    /// and the row's place along the edge (0 for its first). With the count of rows it meets.
    fn crossings(&mut self, edges: &[([i64; 2], [i64; 2])], range: (i64, i64)) -> Vec<(i64, Vec<(i64, i64, i64)>)> {
        self.note("crossings", edges.len());
        edges.iter().map(|&(a, b)| {
            let first = within("a row", fl(a[1] + 49, 100), CARD);
            let rows = (within("a row", fl(b[1] + 49, 100), CARD) - first).max(0);
            let mut l = vec![];
            for j in 0..rows {
                let row = first + j;
                if !(range.0..range.1).contains(&row) { continue; }
                let x = a[0] + fl((b[0] - a[0]) * (100 * row + 50 - a[1]), b[1] - a[1]);
                l.push((row, fl(x + 49, 100), j));
            }
            (rows, l)
        }).collect()
    }
    /// Walks: for each (a, b, va, vb), the values at the n = |b - a| places from the lesser on: at the k-th, the
    /// lesser's value plus the change to the other's times (k + 1) / (n + 1), cut toward zero (cards or longs); a walk
    /// of more than 998 places in two (walk_pieces). With a window (lo, hi), only the places in it. All the walks'
    /// values one after another.
    fn walk(&mut self, walks: &[(i64, i64, i64, i64)], window: Option<(i64, i64)>) -> V {
        let places: usize = walks.iter().map(|w| (w.1 - w.0).unsigned_abs() as usize).sum();
        self.note("walk", places);
        self.note("walk_runs", walks.len());
        let pieces = walk_pieces(walks, |m| m.iter().map(|&(vl, vr, h, n)| { let d = vr - vl; vl + d.signum() * (d.abs() * h / n) }).collect());
        let mut out = vec![];
        for &(lo, n, vl, vr) in &pieces {
            within("walk", vl, LONG); within("walk", vr, LONG);
            for k in 0..n {
                if let Some((wl, wh)) = window { if lo + k < wl || lo + k >= wh { continue; } }
                out.push(vl + (vr - vl) * (k + 1) / (n + 1));
            }
        }
        out
    }
    /// Walks by slopes: for each (n, va, a, f0), the values at n places: at the k-th, va plus fl(f0 + a k, 1000)
    /// (a the change a place, thousandths, three cards; f0 0..999; va and the values longs). All one after another.
    fn walka(&mut self, walks: &[(i64, i64, i64, i64)]) -> V {
        let places: usize = walks.iter().map(|w| w.0 as usize).sum();
        self.note("walka", places);
        let s = 0;
        let mut out = Vec::with_capacity(places);
        for &(n, va, a, f0) in walks {
            within("walka", va, LONG); within("walka", a, WIDE);
            assert!(n >= 1 && n <= 99_999 && (0..1000).contains(&f0), "walka of {n} {f0}");
            for k in 0..n { out.push(within("walka", va + fl(f0 + a * (k + s), 1000), LONG)); }
        }
        out
    }
    /// walka over cards: n under 1000, the start and every value a card, the slope's whole part a card.
    fn walkc(&mut self, walks: &[(i64, i64, i64, i64)]) -> V {
        let places: usize = walks.iter().map(|w| w.0 as usize).sum();
        self.note("walkc", places);
        let mut out = Vec::with_capacity(places);
        for &(n, va, a, f0) in walks {
            within("walkc", va, CARD); within("walkc", fl(a, 1000), CARD);
            assert!(n >= 1 && n <= 999 && (0..1000).contains(&f0), "walkc of {n} {f0}");
            for k in 0..n { out.push(within("walkc", va + fl(f0 + a * k, 1000), CARD)); }
        }
        out
    }
    /// K / 0.7 over a depth (K / z: so z / 0.7, centimetres): from the table of K / 0.7 / i for i under 10,000, the
    /// depth's first four digits looked up and the result moved down as many places as were dropped.
    fn recip(&mut self, t: &Tab, iz: A) -> V {
        let n = n_of(&[iz]);
        self.note("recip", n);
        map1(iz, n, |z| {
            assert!((1..=LONG).contains(&z), "recip of {z}");
            if z < 10_000 { t.data[z as usize] } else if z < 100_000 { fl(t.data[(z / 10) as usize], 10) } else { fl(t.data[(z / 100) as usize], 100) }
        })
    }
    /// Values between samples: for each segment (n pixels, the values at its first sample and at the next), each
    /// pixel's: the first's plus k times the step, rounded down, the step (b - a) q thousandths a pixel, q = 1000 / n
    /// rounded down (0 for one pixel); for values that repeat every 1000 (wrap), b - a the shorter way round and each
    /// value md 1000. Cards.
    fn walkseg(&mut self, segs: &[(i64, i64, i64)], wrap: bool) -> V {
        let total: usize = segs.iter().map(|s| s.0 as usize).sum();
        self.note("walkseg", total);
        let mut out = Vec::with_capacity(total);
        for &(n, va, vb) in segs {
            within("walkseg", va, CARD); within("walkseg", vb, CARD);
            let mut d = vb - va;
            if wrap { if d >= 500 { d -= 1000; } if d < -500 { d += 1000; } }
            let q = if n <= 1 { 0 } else { 1000 / n };
            let step = d * q;
            for k in 0..n {
                let v = va + fl(k * step, 1000);
                out.push(if wrap { md(v, 1000) } else { v });
            }
        }
        out
    }
    /// The sun's share at places (percent) from their shadow taps: 100 less 100/16 (rounded up) for each depth of the
    /// map nearer than the place's (longs: each place's need, its 16 depths).
    fn lit16(&mut self, need: &V, taps: &[V]) -> V {
        self.note("lit16", need.len());
        assert_eq!(taps.len(), 16);
        (0..need.len()).map(|i| { let c = taps.iter().filter(|t| within("lit16", t[i], LONG) < within("lit16", need[i], LONG)).count() as i64; 100 + fl(-100 * c, 16) }).collect()
    }
    /// a - b for longs, held to lo 1000 ..= hi 1000 + 999 (lo, hi cards: the thousands' bounds).
    fn ldiff(&mut self, a: &V, b: &V, lo: i64, hi: i64) -> V {
        self.note("ldiff", a.len());
        (0..a.len()).map(|i| within("ldiff's difference", within("ldiff", a[i], LONG) - within("ldiff", b[i], LONG), LONG).clamp(lo * 1000, hi * 1000 + 999)).collect()
    }
    /// The tone curve's index for each channel: the card plus 25 times its tenths past 999 (by the printed table x25)
    /// plus the light the lens adds (a long, 0 or more), under 40,000 (cards and tenths 0..999).
    fn tone_index(&mut self, col: &V, hot: &V, add: &V, x25: &Tab) -> V {
        self.note("tone_index", col.len());
        let _ = x25;
        (0..col.len()).map(|i| { assert!(add[i] >= 0); let t = within("tone", col[i], CARD) + 25 * within("tone", hot[i], CARD) + within("tone", add[i], LONG); assert!(t < 40_000, "a tone index of {t}"); t }).collect()
    }
    /// Motion as table places: x = u + r, y likewise (u at most 400, r 599 in size), each held to -400..400, plus 400.
    fn vel(&mut self, ux: &V, rx: &V, uy: &V, ry: &V) -> (V, V) {
        self.note("vel", ux.len());
        let f = |u: i64, r: i64| { assert!(u.abs() <= 400 && r.abs() <= 599, "vel {u} {r}"); (u + r).clamp(-400, 400) + 400 };
        ((0..ux.len()).map(|i| f(ux[i], rx[i])).collect(), (0..ux.len()).map(|i| f(uy[i], ry[i])).collect())
    }
    /// Light's sum: a + s + e (cards 0..999), as the card it gives (at most 999) and how far past 999 (at most 999).
    fn light_sum(&mut self, a: &V, s: &V, e: &V) -> (V, V) {
        self.note("light_sum", a.len());
        let t: V = (0..a.len()).map(|i| within("light_sum", a[i], CARD) + within("light_sum", s[i], CARD) + within("light_sum", e[i], CARD)).collect();
        assert!(a.iter().chain(s).chain(e).all(|&x| x >= 0), "light_sum below 0");
        (t.iter().map(|&x| x.min(999)).collect(), t.iter().map(|&x| (x - 999).clamp(0, 999)).collect())
    }
    /// Whether each x (a long) is within h of 0 (|x| < h, h a long), and whether it is below 0.
    fn absless(&mut self, x: &V, h: A) -> (V, V) {
        self.note("absless", x.len());
        ((0..x.len()).map(|i| (within("absless", x[i], LONG).abs() < within("absless", h.at(i), LONG)) as i64).collect(), x.iter().map(|&a| (a < 0) as i64).collect())
    }
    /// A streak's ends along its line: from places k (longs, 0 or more), half lengths h (cards 0..998) and the lines'
    /// last places (longs), a = max(k - h - 1, -1), b = min(k + h, last) and the counts b - a.
    fn ends(&mut self, k: &V, h: &V, last: &V) -> (V, V, V) {
        self.note("ends", k.len());
        let a: V = (0..k.len()).map(|i| (within("ends", k[i], LONG) - within("ends", h[i], CARD) - 1).max(-1)).collect();
        let b: V = (0..k.len()).map(|i| (k[i] + h[i]).min(within("ends", last[i], LONG))).collect();
        let n: V = (0..k.len()).map(|i| within("ends", b[i] - a[i], CARD)).collect();
        (a, b, n)
    }
    /// Lines of values (cards 0..999) four times as fine: between each value a and the next, a, a + q, a + h and
    /// a + h + q, h half the change and q half of h, each cut toward zero. All the lines' values one after another.
    fn up4(&mut self, lines: &[&[i64]]) -> V {
        self.note("up4", lines.iter().map(|l| 4 * l.len()).sum());
        let mut out = vec![];
        for l in lines {
            for k in 1..l.len() {
                let (a, b) = (within("up4", l[k - 1], CARD), within("up4", l[k], CARD));
                let h = (b - a) / 2;
                let q = h / 2;
                out.extend_from_slice(&[a, a + q, a + h, a + h + q]);
            }
        }
        out
    }
    /// A little more colour: each of a pixel's three toned channels (0..255) t + (3 t - s) / 20 cut toward zero, s their
    /// sum, held to 0..255.
    fn tone_push(&mut self, t: [&V; 3], div20: &Tab) -> [V; 3] {
        self.note("tone_push", t[0].len());
        let _ = div20;
        std::array::from_fn(|c| (0..t[0].len()).map(|i| { let s = t[0][i] + t[1][i] + t[2][i]; (t[c][i] + (3 * t[c][i] - s) / 20).clamp(0, 255) }).collect())
    }
    /// Running sums: for each list, its values' sums so far (longs), one after another.
    fn runsum(&mut self, lists: &[&[i64]]) -> V {
        self.note("runsum", lists.iter().map(|l| l.len()).sum());
        let mut out = vec![];
        for l in lists { let mut s = 0; for &x in l.iter() { s += within("runsum", x, CARD); out.push(within("runsum", s, LONG)); } }
        out
    }
    /// The nearest of each pixel's pieces: of each list of depths (K / z, longs), the place of the largest, the
    /// first of equals.
    fn nearest(&mut self, groups: &[&[i64]]) -> Vec<usize> {
        self.note("nearest", groups.iter().map(|g| g.len()).sum());
        groups.iter().map(|g| { assert!(g.len() <= 999, "nearest: {} in a group", g.len()); let mut b = 0; for (i, &z) in g.iter().enumerate() { if within("nearest", z, LONG) > g[b] { b = i; } } b }).collect()
    }
}

/// The swarm's sine: 999 sin of an angle (millionths of a turn), its table every thousandth of a turn, between by
/// tenths.
pub fn sin_t(trig: &[i64], a: i64) -> i64 {
    let t = fl(md(a, 1_000_000), 100);
    let q = t / 2500;
    let mut r = t % 2500;
    if q % 2 == 1 { r = 2500 - r; }
    let (k, fr) = ((r / 10) as usize, r % 10);
    let v = if fr == 0 { trig[k] } else { trig[k] + (trig[k + 1] - trig[k]) * fr / 10 };
    if q >= 2 { -v } else { v }
}

pub fn isqrt(n: i64) -> i64 { let mut r = (n as f64).sqrt() as i64; while r * r > n { r -= 1; } while (r + 1) * (r + 1) <= n { r += 1; } r }

/// The operations done natively: what the reference runs, and what the machines must match.
pub struct Native { pub trig: Vec<i64>, pub count: std::collections::BTreeMap<&'static str, u64>, pub st: &'static str, pub by_stage: std::collections::BTreeMap<(&'static str, &'static str), u64> }

impl Native {
    pub fn new(trig: Vec<i64>) -> Self { Native { trig, count: Default::default(), st: "setup", by_stage: Default::default() } }
    fn c(&mut self, op: &'static str, n: usize) { *self.count.entry(op).or_insert(0) += n as u64; *self.by_stage.entry((self.st, op)).or_insert(0) += n as u64; }
}

fn map1(a: A, n: usize, f: impl Fn(i64) -> i64) -> V { (0..n).map(|i| f(a.at(i))).collect() }
fn map2(a: A, b: A, n: usize, f: impl Fn(i64, i64) -> i64) -> V { (0..n).map(|i| f(a.at(i), b.at(i))).collect() }

impl Ops for Native {
    fn note(&mut self, op: &'static str, n: usize) { self.c(op, n); }
    fn stage(&mut self, name: &'static str) { self.st = name; }
    fn tab(&mut self, t: &Tab, i: A) -> V {
        let n = n_of(&[i]);
        self.c("tab", n);
        map1(i, n, |k| { assert!(k >= 0 && (k as usize) < t.data.len(), "{}: index {k} of {}", t.name, t.data.len()); t.data[k as usize] })
    }
    fn tab2(&mut self, t: &Tab, i: A, j: A) -> V {
        let n = n_of(&[i, j]);
        self.c("tab", n);
        map2(i, j, n, |a, b| {
            assert!(a >= 0 && b >= 0 && (b as usize) < t.w && ((a as usize) * t.w + b as usize) < t.data.len(), "{}: index {a},{b}", t.name);
            t.data[a as usize * t.w + b as usize]
        })
    }
    fn sel(&mut self, flag: &[i64], a: A, b: A) -> V {
        let n = n_of(&[A::V(flag), a, b]);
        self.c("sel", n);
        (0..n).map(|i| match flag[i] { 1 => a.at(i), 0 => b.at(i), f => panic!("a flag of {f}") }).collect()
    }
    fn sum(&mut self, terms: &[(A, i64)], c: i64) -> V {
        let n = n_of(&terms.iter().map(|t| t.0).collect::<Vec<_>>());
        self.c("sum", n);
        (0..n).map(|i| {
            let mut s = c;
            for (a, k) in terms { assert!(k.abs() <= 9); s += within("sum term", a.at(i), LONG) * k; }
            within("sum", s, LONG)
        }).collect()
    }
    fn pct(&mut self, a: A, t: A) -> V {
        let n = n_of(&[a, t]);
        self.c("pct", n);
        map2(a, t, n, |a, t| { within("pct", a, CARD); assert!(t.abs() <= 100, "pct t {t}"); a * t / 100 })
    }
    fn mix(&mut self, a: A, b: A, t: A) -> V {
        let n = n_of(&[a, b, t]);
        self.c("mix", n);
        (0..n).map(|i| {
            let (x, y, t) = (within("mix", a.at(i), CARD), within("mix", b.at(i), CARD), t.at(i));
            assert!((0..=100).contains(&t), "mix t {t}");
            within("mix", x + within("mix change", y - x, CARD) * t / 100, CARD)
        }).collect()
    }
    fn mul(&mut self, a: A, b: A, s: u32) -> V {
        let n = n_of(&[a, b]);
        self.c("mul", n);
        map2(a, b, n, |a, b| { within("mul", a, CARD); within("mul", b, CARD); within("mul", fl(a * b, 10i64.pow(s)), LONG) })
    }
    fn clamp(&mut self, a: A, lo: i64, hi: i64) -> V { let n = n_of(&[a]); self.c("clamp", n); map1(a, n, |x| within("clamp", x, WIDE).clamp(lo, hi)) }
    fn lt(&mut self, a: A, b: A) -> V { let n = n_of(&[a, b]); self.c("lt", n); map2(a, b, n, |x, y| (within("lt", x, WIDE) < within("lt", y, WIDE)) as i64) }
    fn max(&mut self, a: A, b: A) -> V { let n = n_of(&[a, b]); self.c("max", n); map2(a, b, n, |x, y| within("max", x, LONG).max(within("max", y, LONG))) }
    fn min(&mut self, a: A, b: A) -> V { let n = n_of(&[a, b]); self.c("min", n); map2(a, b, n, |x, y| within("min", x, LONG).min(within("min", y, LONG))) }
    fn divmod(&mut self, a: A, d: A) -> (V, V) {
        let n = n_of(&[a, d]);
        self.c("divmod", n);
        let q = map2(a, d, n, |x, d| { assert!((1..=999).contains(&d)); within("divmod", fl(within("divmod", x, WIDE), d), LONG) });
        let r = map2(a, d, n, |x, d| md(x, d));
        (q, r)
    }
    fn lin(&mut self, terms: &[(A, Option<A>)]) -> V {
        let mut all = vec![];
        for (a, b) in terms { all.push(*a); if let Some(b) = b { all.push(*b); } }
        let n = n_of(&all);
        self.c("lin", n * terms.len());
        let limbs = |x: i64| if x.abs() < 1000 { 1 } else if x.abs() < 1_000_000 { 2 } else { 3 };
        let mut lp = 0usize;
        for i in 0..n { for (a, b) in terms { if let Some(b) = b { lp += limbs(a.at(i)) * limbs(b.at(i)); } } }
        self.c("lin_limbs", lp);
        (0..n).map(|i| {
            let mut s = 0;
            for (a, b) in terms {
                let x = within("lin", a.at(i), WIDE);
                s += match b { Some(b) => { let y = within("lin", b.at(i), WIDE); within("lin product", x * y, WIDE) } None => x };
            }
            within("lin sum", s, WIDE)
        }).collect()
    }
    fn div(&mut self, x: A, d: A, extra: u32) -> V {
        let n = n_of(&[x, d]);
        self.c("div", n);
        map2(x, d, n, |x, d| { assert!((1..=99_999).contains(&d), "a divisor of {d}"); within("div", fl(within("div", x, WIDE) * 1000i64.pow(extra), d), LONG) })
    }
    fn sincos(&mut self, a: A) -> (V, V) {
        let n = n_of(&[a]);
        self.c("sincos", n);
        (map1(a, n, |x| sin_t(&self.trig, x)), map1(a, n, |x| sin_t(&self.trig, x + 250_000)))
    }
    fn sqrt(&mut self, x: A) -> V { let n = n_of(&[x]); self.c("sqrt", n); map1(x, n, |v| { assert!((0..10_000_000).contains(&v), "sqrt {v}"); isqrt(v) }) }
}

/// Walks as pieces of at most 998 places: (first place, places, value at the start, at the end). A longer one is
/// halved at h = n / 2 places, the value there vl + (vr - vl) h / n cut toward zero (worked out by `mid` for a batch
/// of (vl, vr, h, n)), again until every piece is short enough.
pub fn walk_pieces(walks: &[(i64, i64, i64, i64)], mut mid: impl FnMut(&[(i64, i64, i64, i64)]) -> Vec<i64>) -> Vec<(i64, i64, i64, i64)> {
    let mut list: Vec<(i64, i64, i64, i64)> = walks.iter().map(|&(a, b, va, vb)| if b < a { (b, a - b, vb, va) } else { (a, b - a, va, vb) }).collect();
    loop {
        let long: Vec<usize> = (0..list.len()).filter(|&i| list[i].1 > 998).collect();
        if long.is_empty() { return list; }
        let asks: Vec<(i64, i64, i64, i64)> = long.iter().map(|&i| { let (_, n, vl, vr) = list[i]; (vl, vr, n / 2, n) }).collect();
        let m = mid(&asks);
        let mut next = Vec::with_capacity(list.len() + long.len());
        let mut j = 0;
        for (i, &(lo, n, vl, vr)) in list.iter().enumerate() {
            if j < long.len() && long[j] == i {
                let h = n / 2;
                next.push((lo, h, vl, m[j]));
                next.push((lo + h, n - h, m[j], vr));
                j += 1;
            } else { next.push((lo, n, vl, vr)); }
        }
        list = next;
    }
}
