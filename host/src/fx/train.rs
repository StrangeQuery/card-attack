//! The bullet train: the ride at 1536x768, every number worked out on CARDIAC. The renderer is the
//! segment's (segments/train/*.rs), written against its operations; here each operation is done by machines, the host
//! dealing the cards, carrying what they punch, fetching from the printed tables by the indices they punch, and
//! picking between values by the flags they punch. `card-attack train refs/train_ref` renders it and checks every
//! frame against the reference (the same renderer with the operations done natively).
use crate::*;
use std::collections::BTreeMap;
use std::path::Path;

#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
pub mod imp {
    include!("../../../segments/train/ops.rs");
    include!("../../../segments/train/scene.rs");
    include!("../../../segments/train/render.rs");
    include!("../../../segments/train/frame.rs");
    include!("../../../segments/train/sky.rs");
    include!("../../../segments/train/shade.rs");
    include!("../../../segments/train/materials.rs");
    include!("../../../segments/train/tunnel.rs");
    include!("../../../segments/train/blur.rs");
    include!("../../../segments/train/wires.rs");
    include!("../../../segments/train/lens.rs");
    include!("../../../segments/train/mirror.rs");
    include!("../../../segments/train/shadow.rs");
    include!("../../../segments/train/overlays.rs");
    include!("../../../segments/train/party.rs");
    // the night arrival's parts (segments/night), on the same renderer and operations
    include!("../../../segments/night/night.rs");
    include!("../../../segments/night/path.rs");
    include!("../../../segments/night/nframe.rs");
    include!("../../../segments/night/nmat.rs");
    include!("../../../segments/night/npost.rs");
    include!("../../../segments/night/nseg.rs");
    include!("../../../segments/night/nparty.rs");
}
use imp::{fl, md, Ops, Tab, A, V, CARD, LONG};

const MACHINES: &[&str] = &["tr_tone1", "tr_seg", "tr_dwrap", "tr_qa", "tr_qb", "tr_mixa", "tr_pctb", "tr_addc", "tr_shift1", "tr_shift2", "tr_shift3", "tr_comb", "tr_sum", "tr_lt", "tr_min", "tr_max", "tr_pct", "tr_mix", "tr_qp", "tr_walk_up", "tr_walk_down", "tr_runsum",
    "tr_near", "tr_row", "tr_carry", "tr_cross", "tr_rcp", "tr_wrap", "tr_phase4", "tr_phase5", "tr_ldiff", "tr_lit16", "tr_up4", "tr_ends", "tr_vel", "tr_lsum", "tr_absl", "tr_push1", "tr_push2", "tr_walka", "tr_add", "tr_walkc",
    "sw_qi", "sw_qis", "sw_qp", "sw_lsum", "sw_term", "sw_mag", "sw_abs", "sw_divd", "sw_join", "sw_floor", "sw_sign",
    "sw_ang", "sw_sin", "sw_hund", "sw_isqrt", "sw_isqrt2"];
/// Machines written by hand for one case each of adding longs: one long and a constant; a second added; a second
/// taken away (the deck: the count, how many longs, the second's sign, the constant as a long, then the longs). 41
/// instructions an item, as one compiled machine for the three, down to 23 over the film, and right where a compiled
/// partial high of -1000 wrapped on a card. (A hand machine for the whole of a Rust
/// machine's deck, PART/hand/NAME.asm, `rust` loads in its place.)
const HAND: &[(&str, &str)] = &[("tr_add1", "train/hand/tr_add1.asm"), ("tr_add2p", "train/hand/tr_add2p.asm"), ("tr_add2m", "train/hand/tr_add2m.asm")];

fn long(v: i64) -> [i32; 2] { assert!(v.abs() <= LONG, "not a long: {v}"); [fl(v, 1000) as i32, md(v, 1000) as i32] }
fn three(v: i64) -> [i32; 3] { assert!(v.abs() < 999_000_000, "past three cards: {v}"); [fl(v, 1_000_000) as i32, md(fl(v, 1000), 1000) as i32, md(v, 1000) as i32] }
fn of_long(c: &[i32]) -> i64 { c[0] as i64 * 1000 + c[1] as i64 }
fn of_three(c: &[i32]) -> i64 { c[0] as i64 * 1_000_000 + c[1] as i64 * 1000 + c[2] as i64 }
fn limbs(v: i64) -> usize { if v.abs() < 1000 { 1 } else if v.abs() < 1_000_000 { 2 } else { 3 } }

/// The operations on CARDIAC.
pub struct Cardiac {
    pub host: Host,
    /// the machines and the printed tables, loaded once a process and shared by every job
    s: std::sync::Arc<Loaded>,
    /// instructions by stage, for the report
    pub stage: &'static str,
    pub by_stage: BTreeMap<&'static str, u64>,
    mark: u64,
    pub by_machine: BTreeMap<(&'static str, &'static str), (u64, u64)>,
}

impl Cardiac {
    pub fn new(demo: &Path) -> Self {
        static LOADED: std::sync::OnceLock<std::sync::Arc<Loaded>> = std::sync::OnceLock::new();
        let s = LOADED.get_or_init(|| std::sync::Arc::new(Loaded::new(demo))).clone();
        Cardiac { host: Host::new(), s, stage: "setup", by_stage: BTreeMap::new(), mark: 0, by_machine: BTreeMap::new() }
    }
}

/// What every Cardiac reads and never changes: the machines, the quarter squares, the trig table.
struct Loaded { m: BTreeMap<&'static str, Machine>, quarter: Vec<Vec<[i32; 2]>>, trig: Vec<[i32; 2]> }

impl Loaded {
    fn new(demo: &Path) -> Self {
        let mut m = BTreeMap::new();
        for &n in MACHINES { m.insert(n, rust(demo, n)); }
        for &(n, f) in HAND { let mut h = hand(demo, f); h.name = format!("{n} (hand)"); m.insert(n, h); }
        let t = super::swarm_calc::load_tables(&read(demo, "swarm/data/tables.txt"));
        Loaded { m, quarter: t.quarter, trig: t.trig }
    }
}

impl Cardiac {

    pub fn tally(&mut self) { let s = self.host.steps; *self.by_stage.entry(self.stage).or_insert(0) += s - self.mark; self.mark = s; }

    /// Run a machine over items of `per` cards each (any number of them, in decks of at most 999), the header after
    /// each deck's count.
    fn go(&mut self, name: &'static str, header: &[i32], items: &[i32], per: usize) -> Vec<i32> {
        if per == 0 || items.is_empty() { return vec![]; }
        let n = items.len() / per;
        let mut out = Vec::new();
        for c in (0..n).step_by(999) {
            let k = (n - c).min(999);
            self.run_deck(name, header, &items[c * per..(c + k) * per], k, &mut out);
        }
        out
    }

    /// One deck: the count, the header, the items' cards (dealt as they are, not copied), punched onto `out`.
    fn run_deck(&mut self, name: &'static str, header: &[i32], cards: &[i32], k: usize, out: &mut Vec<i32>) {
        let count = [k as i32];
        // (two plain scans, which vectorise; where, only when one fails)
        if header.iter().any(|v| v.abs() > 999) || cards.iter().any(|v| v.abs() > 999) {
            let i = header.iter().chain(cards).position(|v| v.abs() > 999).unwrap();
            panic!("{name}: card {} is {}", i + 1, header.iter().chain(cards).nth(i).unwrap());
        }
        let s0 = self.host.steps;
        self.host.gon_into(&self.s.m[name], &[&count, header, cards], out);
        let e = self.by_machine.entry((self.stage, name)).or_insert((0, 0)); e.0 += self.host.steps - s0; e.1 += k as u64;
    }

    /// The same for items of differing lengths.
    fn go_var(&mut self, name: &'static str, header: &[i32], items: &[Vec<i32>]) -> Vec<i32> {
        let mut cards = vec![];
        let mut ends = Vec::with_capacity(items.len());
        for it in items { cards.extend_from_slice(it); ends.push(cards.len()); }
        self.go_flat(name, header, &cards, &ends)
    }
    /// The same, the items' cards one after another and where each ends.
    fn go_flat(&mut self, name: &'static str, header: &[i32], cards: &[i32], ends: &[usize]) -> Vec<i32> {
        let mut out = Vec::new();
        for a in (0..ends.len()).step_by(999) {
            let b = (a + 999).min(ends.len());
            let from = if a == 0 { 0 } else { ends[a - 1] };
            self.run_deck(name, header, &cards[from..ends[b - 1]], b - a, &mut out);
        }
        out
    }

    /// Products of cards, exact, as longs over 10^s rounded down: quarter squares (sw_qi), the host's look-ups, tr_qp.
    fn products(&mut self, a: &[i64], b: &[i64], s: u32) -> V {
        let mut it = Vec::with_capacity(2 * a.len());
        for i in 0..a.len() { it.push(a[i] as i32); it.push(b[i] as i32); }
        let idx = self.go("tr_qa", &[], &it, 2);
        let q = self.quarters(&idx);
        let r = if s == 0 || s == 3 { self.go("tr_qb", &[s as i32], &q, 4) } else { self.go("tr_qp", &[s as i32], &q, 4) };
        r.chunks(2).map(of_long).collect()
    }

    /// The quarter squares at the sizes a machine punched (pairs of cards: thousands, rest), as longs.
    fn quarters(&self, idx: &[i32]) -> Vec<i32> {
        let mut q = Vec::with_capacity(idx.len());
        for c in idx.chunks(4) {
            q.extend_from_slice(&self.s.quarter[c[0] as usize][c[1] as usize]);
            q.extend_from_slice(&self.s.quarter[c[2] as usize][c[3] as usize]);
        }
        q
    }

    /// Longs over 10 or 100 rounded down, and the remainders (tr_shift1, tr_shift2).
    fn shifts(&mut self, x: &[i64], d: i64) -> (V, V) {
        let name = match d { 10 => "tr_shift1", 100 => "tr_shift2", _ => "tr_shift3" };
        let mut it = Vec::with_capacity(2 * x.len());
        for &v in x { it.extend_from_slice(&long(v)); }
        let r = self.go(name, &[], &it, 2);
        (r.chunks(3).map(|c| of_long(&c[0..2])).collect(), r.chunks(3).map(|c| c[2] as i64).collect())
    }

    /// x 1000^extra over d, rounded down, and the remainder: x under 999,000,000 in size, d 1..99,999. The swarm's long
    /// division: sizes (sw_abs), digits (sw_divd, leaving out the size's leading cards that are 0), the quotient
    /// (sw_join), rounded down for x below 0 (sw_floor).
    fn divrem(&mut self, x: &[i64], d: &[i64], extra: u32) -> (V, V) {
        let n = x.len();
        if n == 0 { return (vec![], vec![]); }
        // over 1000: the cards moved down; over 10 or 100 of a long: digits
        if extra == 0 && n > 0 && d.iter().all(|&v| v == d[0]) && [10, 100, 1000].contains(&d[0]) {
            if d[0] == 1000 && x.iter().all(|&v| fl(v, 1000).abs() <= 999_000 && fl(v, 1000) >= -999_000) {
                return (x.iter().map(|&v| fl(v, 1000)).collect(), x.iter().map(|&v| md(v, 1000)).collect());
            }
            if x.iter().all(|&v| v.abs() <= LONG && v >= -999_000) { return self.shifts(x, d[0]); }
        }
        let mut it = Vec::with_capacity(3 * n);
        for &v in x { it.extend_from_slice(&three(v)); }
        let sizes = self.go("sw_abs", &[], &it, 3);
        let lead = |k: usize| if sizes[4 * k + 1] != 0 { 0 } else if sizes[4 * k + 2] != 0 { 1 } else { 2 };
        let (mut items, mut ends) = (Vec::with_capacity(n * (6 + extra as usize)), Vec::with_capacity(n));
        let mut cards = Vec::with_capacity(n);
        for k in 0..n {
            assert!((1..=99_999).contains(&d[k]), "a divisor of {}", d[k]);
            let c = 3 - lead(k) + extra as usize;
            items.extend_from_slice(&long(d[k]));
            items.push(c as i32);
            items.extend_from_slice(&sizes[4 * k + 1 + lead(k)..4 * k + 4]);
            items.extend(std::iter::repeat_n(0, extra as usize));
            ends.push(items.len());
            cards.push(c);
        }
        let digits = self.go_flat("sw_divd", &[], &items, &ends);
        items.clear();
        ends.clear();
        let mut rems = Vec::with_capacity(n);
        let mut at = 0;
        for k in 0..n {
            let nd = 3 * cards[k];
            items.push(nd as i32);
            items.extend_from_slice(&digits[at..at + nd]);
            ends.push(items.len());
            rems.push([digits[at + nd], digits[at + nd + 1]]);
            at += nd + 2;
        }
        let q = self.go_flat("sw_join", &[], &items, &ends);
        let mut f = Vec::with_capacity(7 * n);
        for k in 0..n {
            f.push(sizes[4 * k]);
            f.extend_from_slice(&q[2 * k..2 * k + 2]);
            f.extend_from_slice(&rems[k]);
            f.extend_from_slice(&long(d[k]));
        }
        let r = self.go("sw_floor", &[], &f, 7);
        (r.chunks(4).map(|c| of_long(&c[0..2])).collect(), r.chunks(4).map(|c| of_long(&c[2..4])).collect())
    }

    /// Longs added or taken away (tr_sum): each item's `k` terms (sign, value), the items one after another.
    fn sums(&mut self, terms: &[(i32, i64)], k: usize) -> V {
        if terms.is_empty() || k == 0 { return vec![]; }
        let mut it = Vec::with_capacity(3 * terms.len());
        for &(s, v) in terms { it.push(s); it.extend_from_slice(&long(v)); }
        let r = self.go("tr_sum", &[k as i32], &it, 3 * k);
        r.chunks(2).map(of_long).collect()
    }

    /// Pairs of longs through a comparing machine.
    fn pairs(&mut self, name: &'static str, a: A, b: A, n: usize) -> Vec<i32> {
        let mut it = Vec::with_capacity(4 * n);
        for i in 0..n { it.extend_from_slice(&long(a.at(i))); it.extend_from_slice(&long(b.at(i))); }
        self.go(name, &[], &it, 4)
    }

    /// Where a < b for numbers of any size: the difference's sign (sw_sign) when past a long.
    fn less(&mut self, a: A, b: A, n: usize) -> V {
        // (a long's thousands card goes down to -999: below -999,000 it takes the three-card way)
        let wide = (0..n).any(|i| a.at(i).abs() > LONG || b.at(i).abs() > LONG || a.at(i) < -999_000 || b.at(i) < -999_000);
        if !wide { return self.pairs("tr_lt", a, b, n).into_iter().map(|v| v as i64).collect(); }
        let d = self.lin(&[(a, None), (b, Some(A::K(-1)))]);
        let mut it = Vec::with_capacity(3 * n);
        for &v in &d { it.extend_from_slice(&three(v)); }
        self.go("sw_sign", &[], &it, 3).chunks(2).map(|c| c[0] as i64).collect()
    }
}

fn n_of(args: &[A]) -> usize { imp::n_of(args) }

impl Ops for Cardiac {
    fn stage(&mut self, name: &'static str) { self.tally(); self.stage = name; }

    fn tab(&mut self, t: &Tab, i: A) -> V {
        let n = n_of(&[i]);
        (0..n).map(|k| { let x = i.at(k); assert!(x >= 0 && (x as usize) < t.data.len(), "{}: index {x}", t.name); t.data[x as usize] }).collect()
    }
    fn tab2(&mut self, t: &Tab, i: A, j: A) -> V {
        let n = n_of(&[i, j]);
        (0..n).map(|k| { let (a, b) = (i.at(k), j.at(k)); assert!(a >= 0 && b >= 0 && (b as usize) < t.w, "{}: index {a},{b}", t.name); t.data[a as usize * t.w + b as usize] }).collect()
    }
    fn sel(&mut self, flag: &[i64], a: A, b: A) -> V {
        let n = n_of(&[A::V(flag), a, b]);
        (0..n).map(|i| match flag[i] { 1 => a.at(i), 0 => b.at(i), f => panic!("a flag of {f}") }).collect()
    }

    fn sum(&mut self, terms: &[(A, i64)], c: i64) -> V {
        let mut n = None;
        for (a, _) in terms { if let Some(l) = a.len() { assert!(n.is_none_or(|m| m == l), "vectors of {} and {}", n.unwrap(), l); n = Some(l); } }
        let n = n.unwrap_or(1);
        // each term dealt |k| times, added or taken away; the constant as a term: every item's `w` terms, one after
        // another (sign, value)
        let w = terms.iter().map(|(_, k)| k.unsigned_abs() as usize).sum::<usize>() + (c != 0) as usize;
        if n == 0 || w == 0 { return vec![c; n]; }
        // the w terms as (sign, source), read straight from their vectors: no list of every item's terms built up
        // front and walked three times; the decks dealt are the same cards
        let mut tl: Vec<(i32, A)> = Vec::with_capacity(w);
        for &(a, k) in terms { for _ in 0..k.abs() { tl.push((if k > 0 { 1 } else { -1 }, a)); } }
        if c != 0 { tl.push((1, A::K(c))); }
        // cards whose every partial sum is a card: tr_addc
        let k = w - (c != 0) as usize;
        let cardy = c.abs() <= CARD && k <= 6 && (0..n).all(|i| {
            let mut s = c;
            tl[..k].iter().all(|&(sg, a)| { let x = a.at(i); s += sg as i64 * x; x.abs() <= CARD && s.abs() <= CARD })
        });
        if cardy {
            let mut header = [0i32; 8];
            header[0] = c as i32; header[1] = k as i32;
            for (j, t) in tl[..k].iter().enumerate() { header[2 + j] = t.0; }
            let mut it: Vec<i32> = Vec::with_capacity(n * k);
            for i in 0..n { for &(_, a) in &tl[..k] { it.push(a.at(i) as i32); } }
            return self.go("tr_addc", &header[..2 + k], &it, k).into_iter().map(|v| v as i64).collect();
        }
        // one or two longs and a constant, the first added, every partial sum a long: tr_add
        let okl = |x: i64| x.abs() <= LONG && x >= -999_000;
        if (k == 1 || k == 2) && okl(c) {
            let (s1, s2) = (tl[0].0, if k == 2 { tl[1].0 } else { 1 });
            // the added one first
            let order: Option<[usize; 2]> = if s1 > 0 { Some([0, 1]) } else if k == 2 && s2 > 0 { Some([1, 0]) } else { None };
            if let Some(ord) = order {
                let sb = if k == 2 { tl[ord[1]].0 } else { 1 };
                let (ta, tb) = (tl[ord[0]].1, tl[ord[(k == 2) as usize]].1);
                let fits = (0..n).all(|i| {
                    let a = ta.at(i);
                    let mut sum = a + c;
                    let ok = okl(a) && okl(sum);
                    if k == 2 { let b = tb.at(i); sum += sb as i64 * b; ok && okl(b) && okl(sum) } else { ok }
                });
                if fits {
                    let cc = long(c);
                    let mut it = Vec::with_capacity(2 * k * n);
                    for i in 0..n { it.extend_from_slice(&long(ta.at(i))); if k == 2 { it.extend_from_slice(&long(tb.at(i))); } }
                    // by hand, a machine per case
                    let name = if k == 1 { "tr_add1" } else if sb > 0 { "tr_add2p" } else { "tr_add2m" };
                    return self.go(name, &[k as i32, sb, cc[0], cc[1]], &it, 2 * k).chunks(2).map(of_long).collect();
                }
            }
        }
        let mut items: Vec<(i32, i64)> = Vec::with_capacity(n * w);
        for i in 0..n { for &(sg, a) in &tl { items.push((sg, a.at(i))); } }
        self.sums(&items, w)
    }
    fn pct(&mut self, a: A, t: A) -> V {
        // the product by quarter squares (tr_qa), over 100 toward zero (tr_pctb)
        let n = n_of(&[a, t]);
        let mut it = Vec::with_capacity(2 * n);
        for i in 0..n { it.push(a.at(i) as i32); it.push(t.at(i) as i32); }
        let idx = self.go("tr_qa", &[], &it, 2);
        let q = self.quarters(&idx);
        let mut it = Vec::with_capacity(5 * n);
        for c in q.chunks(4) { it.extend_from_slice(c); it.push(0); }
        self.go("tr_pctb", &[], &it, 5).into_iter().map(|v| v as i64).collect()
    }
    fn mix(&mut self, a: A, b: A, t: A) -> V {
        let n = n_of(&[a, b, t]);
        let mut it = Vec::with_capacity(3 * n);
        for i in 0..n { it.push(a.at(i) as i32); it.push(b.at(i) as i32); it.push(t.at(i) as i32); }
        let idx = self.go("tr_mixa", &[], &it, 3);
        let q = self.quarters(&idx);
        let mut it = Vec::with_capacity(5 * n);
        for (i, c) in q.chunks(4).enumerate() { it.extend_from_slice(c); it.push(a.at(i) as i32); }
        self.go("tr_pctb", &[], &it, 5).into_iter().map(|v| v as i64).collect()
    }
    fn mul(&mut self, a: A, b: A, s: u32) -> V {
        let n = n_of(&[a, b]);
        let (x, y): (V, V) = ((0..n).map(|i| a.at(i)).collect(), (0..n).map(|i| b.at(i)).collect());
        self.products(&x, &y, s)
    }
    fn clamp(&mut self, a: A, lo: i64, hi: i64) -> V {
        let n = n_of(&[a]);
        if (0..n).any(|i| a.at(i).abs() > LONG || a.at(i) < -999_000) || lo < -999_000 || hi > LONG {
            let f = self.less(a, A::K(lo), n);
            let x = self.sel(&f, A::K(lo), a);
            let f = self.less(A::K(hi), A::V(&x), n);
            return self.sel(&f, A::K(hi), A::V(&x));
        }
        let x: V = self.pairs("tr_max", a, A::K(lo), n).chunks(2).map(of_long).collect();
        self.pairs("tr_min", A::V(&x), A::K(hi), n).chunks(2).map(of_long).collect()
    }
    fn lt(&mut self, a: A, b: A) -> V { let n = n_of(&[a, b]); self.less(a, b, n) }
    fn max(&mut self, a: A, b: A) -> V { let n = n_of(&[a, b]); self.pairs("tr_max", a, b, n).chunks(2).map(of_long).collect() }
    fn min(&mut self, a: A, b: A) -> V { let n = n_of(&[a, b]); self.pairs("tr_min", a, b, n).chunks(2).map(of_long).collect() }
    fn divmod(&mut self, a: A, d: A) -> (V, V) {
        let n = n_of(&[a, d]);
        let (x, y): (V, V) = ((0..n).map(|i| a.at(i)).collect(), (0..n).map(|i| d.at(i)).collect());
        self.divrem(&x, &y, 0)
    }
    fn muldiv(&mut self, a: A, b: A, d: i64) -> V {
        // over 1, 10, 100 or 1000: the product's digits moved down (tr_qp, tr_qb)
        if let Some(s) = [1, 10, 100, 1000].iter().position(|&x| x == d) { return self.mul(a, b, s as u32); }
        let p = self.mul(a, b, 0);
        let n = p.len();
        self.divrem(&p, &vec![d; n], 0).0
    }

    fn lin(&mut self, terms: &[(A, Option<A>)]) -> V {
        let mut all = vec![];
        for (a, b) in terms { all.push(*a); if let Some(b) = b { all.push(*b); } }
        let n = n_of(&all);
        // the common shapes, by the lean machines: one product of cards; one of a card and a long; sums of longs
        if terms.len() == 1 {
            if let (a, Some(b)) = terms[0] {
                let small = |x: A| (0..n).all(|i| x.at(i).abs() <= CARD);
                let longish = |x: A| (0..n).all(|i| x.at(i).abs() <= LONG && x.at(i) >= -999_000);
                if small(a) && small(b) {
                    let (x, y): (V, V) = ((0..n).map(|i| a.at(i)).collect(), (0..n).map(|i| b.at(i)).collect());
                    return self.products(&x, &y, 0);
                }
                let (c, l) = if small(a) && longish(b) { (a, b) } else if small(b) && longish(a) { (b, a) } else { (A::K(0), A::K(0)) };
                if (0..n).any(|i| l.at(i) != 0) || (small(a) && longish(b)) || (small(b) && longish(a)) {
                    if (small(a) && longish(b)) || (small(b) && longish(a)) {
                        let cv: V = (0..n).map(|i| c.at(i)).collect();
                        let hi: V = (0..n).map(|i| fl(l.at(i), 1000)).collect();
                        let lo: V = (0..n).map(|i| md(l.at(i), 1000)).collect();
                        let p = self.products(&cv, &hi, 0);
                        let q = self.products(&cv, &lo, 0);
                        let mut it = Vec::with_capacity(4 * n);
                        for i in 0..n { it.extend_from_slice(&long(p[i])); it.extend_from_slice(&long(q[i])); }
                        return self.go("tr_comb", &[], &it, 4).chunks(3).map(of_three).collect();
                    }
                }
            }
        }
        let addends = terms.iter().all(|(_, b)| matches!(b, None) || matches!(b, Some(A::K(k)) if k.abs() <= 9));
        if addends && (0..n).all(|i| {
            let mut s = 0i64;
            terms.iter().all(|(a, b)| { let k = match b { Some(A::K(k)) => *k, _ => 1 }; let x = a.at(i); s += k * x; x.abs() <= LONG && x >= -999_000 && s.abs() <= LONG && s >= -999_000 })
        }) {
            let ks: Vec<(A, i64)> = terms.iter().map(|(a, b)| (*a, match b { Some(A::K(k)) => *k, _ => 1 })).collect();
            return self.sum(&ks, 0);
        }
        // each item's pieces (three cards each): addends as they are (sw_term to take away or widen), products by limbs
        // (each piece with its item, in the order made; grouped by item, in that order, for the sums)
        let mut pieces: Vec<(usize, [i32; 3])> = vec![];
        let mut addends: Vec<(usize, i32, i64)> = vec![];      // item, take away, value
        let mut prods: Vec<(usize, i64, i64)> = vec![];        // item, a, b
        for (a, b) in terms {
            match b {
                None => for i in 0..n { addends.push((i, 0, a.at(i))); },
                Some(A::K(k)) if k.abs() <= 9 => for i in 0..n { for _ in 0..k.abs() { addends.push((i, (*k < 0) as i32, a.at(i))); } },
                Some(b) => for i in 0..n { prods.push((i, a.at(i), b.at(i))); },
            }
        }
        // addends: three cards as they stand, else through sw_term
        let (mut to_term, mut ends) = (vec![], vec![]);
        let mut whose = vec![];
        for &(i, neg, v) in &addends {
            if v == 0 { continue; }
            if neg == 0 && v.abs() > CARD { pieces.push((i, three(v))); } else {
                if v.abs() <= CARD { to_term.extend_from_slice(&[neg, 1, v as i32]) } else { let t = three(v); to_term.extend_from_slice(&[neg, 3, t[0], t[1], t[2]]) }
                ends.push(to_term.len());
                whose.push(i);
            }
        }
        let t = self.go_flat("sw_term", &[], &to_term, &ends);
        for (k, &i) in whose.iter().enumerate() { pieces.push((i, [t[3 * k], t[3 * k + 1], t[3 * k + 2]])); }
        // products: signs and sizes (sw_mag), each pair of limbs (sw_qis, the quarter squares, sw_qp)
        let prods: Vec<(usize, i64, i64)> = prods.into_iter().filter(|p| p.1 != 0 && p.2 != 0).collect();
        if !prods.is_empty() {
            let mag = |s: &mut Self, vals: &[i64]| -> Vec<[i32; 4]> {
                let (mut cards, mut ends) = (Vec::with_capacity(4 * vals.len()), Vec::with_capacity(vals.len()));
                for &v in vals { if v.abs() <= CARD { cards.extend_from_slice(&[1, v as i32]) } else { let t = three(v); cards.extend_from_slice(&[3, t[0], t[1], t[2]]) } ends.push(cards.len()); }
                s.go_flat("sw_mag", &[], &cards, &ends).chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
            };
            let av: V = prods.iter().map(|p| p.1).collect();
            let bv: V = prods.iter().map(|p| p.2).collect();
            let ma = mag(self, &av);
            let mb = mag(self, &bv);
            let mut qis = vec![];
            let mut owner = vec![];
            for (k, &(i, a, b)) in prods.iter().enumerate() {
                assert!((a as i128 * b as i128).abs() < 999_000_000, "a product past three cards: {a} {b}");
                for li in 0..limbs(a) {
                    for lj in 0..limbs(b) {
                        if li + lj > 2 { continue; }
                        qis.extend_from_slice(&[ma[k][3 - li], mb[k][3 - lj], ma[k][0], mb[k][0], 0]);
                        owner.push((i, li + lj));
                    }
                }
            }
            let idx = self.go("sw_qis", &[], &qis, 5);
            let mut qp = Vec::with_capacity(5 * owner.len());
            for (k, &(_, place)) in owner.iter().enumerate() {
                let c = &idx[4 * k..4 * k + 4];
                qp.push(place as i32);
                qp.extend_from_slice(&self.s.quarter[c[0] as usize][c[1] as usize]);
                qp.extend_from_slice(&self.s.quarter[c[2] as usize][c[3] as usize]);
            }
            let p = self.go("sw_qp", &[], &qp, 5);
            for (k, &(i, _)) in owner.iter().enumerate() { pieces.push((i, [p[3 * k], p[3 * k + 1], p[3 * k + 2]])); }
        }
        // the sums (sw_lsum) where there is more than one piece
        pieces.sort_by_key(|p| p.0);
        let mut out = vec![0; n];
        let (mut items, mut ends) = (Vec::with_capacity(4 * pieces.len()), vec![]);
        let mut at = vec![];
        let mut g = 0;
        while g < pieces.len() {
            let i = pieces[g].0;
            let e = g + pieces[g..].iter().take_while(|p| p.0 == i).count();
            match e - g {
                1 => out[i] = of_three(&pieces[g].1),
                k => { items.push(k as i32); for p in &pieces[g..e] { items.extend_from_slice(&p.1); } ends.push(items.len()); at.push(i); }
            }
            g = e;
        }
        let s = self.go_flat("sw_lsum", &[], &items, &ends);
        for (k, &i) in at.iter().enumerate() { out[i] = of_three(&s[3 * k..3 * k + 3]); }
        out
    }
    fn div(&mut self, x: A, d: A, extra: u32) -> V {
        let n = n_of(&[x, d]);
        let (a, b): (V, V) = ((0..n).map(|i| x.at(i)).collect(), (0..n).map(|i| d.at(i)).collect());
        self.divrem(&a, &b, extra).0
    }
    fn sincos(&mut self, a: A) -> (V, V) {
        let n = n_of(&[a]);
        let mut it = Vec::with_capacity(2 * n);
        for i in 0..n { it.extend_from_slice(&long(a.at(i))); }
        let at = self.go("sw_ang", &[], &it, 2);
        let mut d = Vec::with_capacity(8 * n);
        for c in at.chunks(6) {
            d.extend_from_slice(&self.s.trig[c[0] as usize]);
            d.extend_from_slice(&c[1..3]);
            d.extend_from_slice(&self.s.trig[c[3] as usize]);
            d.extend_from_slice(&c[4..6]);
        }
        let s = self.go("sw_sin", &[], &d, 8);
        (s.chunks(4).map(|c| c[0] as i64).collect(), s.chunks(4).map(|c| c[1] as i64).collect())
    }
    fn sqrt(&mut self, x: A) -> V {
        let n = n_of(&[x]);
        let mut it = Vec::with_capacity(3 * n);
        for i in 0..n { it.extend_from_slice(&three(x.at(i))); }
        let h = self.go("sw_hund", &[], &it, 3);
        let a = self.go("sw_isqrt", &[], &h, 3);
        let r = self.go("sw_isqrt2", &[], &a, 6);
        r.chunks(2).map(of_long).collect()
    }
    fn turns(&mut self, x: A) -> V {
        // a number's last two cards (a long): its whole turns dropped
        let n = n_of(&[x]);
        (0..n).map(|i| { let t = three(x.at(i)); t[1] as i64 * 1000 + t[2] as i64 }).collect()
    }
    fn wrap(&mut self, u: A, d: i64) -> V {
        let n = n_of(&[u]);
        if d == 1 { return (0..n).map(|i| md(u.at(i), 1000)).collect(); }
        assert_eq!(d, 10);
        let mut it = Vec::with_capacity(2 * n);
        for i in 0..n { it.extend_from_slice(&long(u.at(i))); }
        self.go("tr_wrap", &[], &it, 2).into_iter().map(|v| v as i64).collect()
    }
    fn phase(&mut self, u: A, r: i64, s: u32) -> (V, V) {
        let n = n_of(&[u]);
        assert!((1..=999).contains(&r) && (s == 4 || s == 5), "phase {r} {s}");
        let hi: V = (0..n).map(|i| fl(u.at(i), 1000)).collect();
        let lo: V = (0..n).map(|i| md(u.at(i), 1000)).collect();
        let p = self.products(&hi, &vec![r; n], 0);
        let q = self.products(&lo, &vec![r; n], 0);
        let mut it = Vec::with_capacity(4 * n);
        for i in 0..n { it.extend_from_slice(&long(p[i])); it.extend_from_slice(&long(q[i])); }
        let o = self.go(if s == 4 { "tr_phase4" } else { "tr_phase5" }, &[], &it, 4);
        (o.chunks(2).map(|c| c[0] as i64).collect(), o.chunks(2).map(|c| c[1] as i64).collect())
    }
    fn recip(&mut self, t: &Tab, iz: A) -> V {
        let n = n_of(&[iz]);
        let mut it = Vec::with_capacity(2 * n);
        for i in 0..n { assert!(iz.at(i) >= 1); it.extend_from_slice(&long(iz.at(i))); }
        let r = self.go("tr_rcp", &[], &it, 2);
        // the table's entry by the index punched, then moved down as many places as were dropped (tr_qp, by places)
        let mut out = vec![0; n];
        for e in 0..3 {
            let at: Vec<usize> = (0..n).filter(|&i| r[3 * i + 2] == e).collect();
            if at.is_empty() { continue; }
            let mut q = Vec::with_capacity(4 * at.len());
            for &i in &at { let k = of_long(&r[3 * i..3 * i + 2]) as usize; q.extend_from_slice(&long(t.data[k])); q.extend_from_slice(&[0, 0]); }
            let v = self.go("tr_qp", &[e], &q, 4);
            for (j, &i) in at.iter().enumerate() { out[i] = of_long(&v[2 * j..2 * j + 2]); }
        }
        out
    }
    fn walkseg(&mut self, segs: &[(i64, i64, i64)], wrap: bool) -> V {
        let n = segs.len();
        if n == 0 { return vec![]; }
        // the change (tr_dwrap), the step by n's thousandths over it (a product), the values (tr_seg)
        let mut it = Vec::with_capacity(2 * n);
        for s in segs { it.push(s.1 as i32); it.push(s.2 as i32); }
        let d: V = self.go("tr_dwrap", &[wrap as i32], &it, 2).into_iter().map(|v| v as i64).collect();
        let q: V = segs.iter().map(|s| if s.0 <= 1 { 0 } else { 1000 / s.0 }).collect();
        let step = self.products(&d, &q, 0);
        let mut it = Vec::with_capacity(4 * n);
        for (i, s) in segs.iter().enumerate() { it.push(s.0 as i32); it.push(s.1 as i32); it.extend_from_slice(&long(step[i])); }
        self.go("tr_seg", &[wrap as i32], &it, 4).into_iter().map(|v| v as i64).collect()
    }
    fn ldiff(&mut self, a: &V, b: &V, lo: i64, hi: i64) -> V {
        let it: Vec<i32> = (0..a.len()).flat_map(|i| { let (x, y) = (long(a[i]), long(b[i])); [x[0], x[1], y[0], y[1]] }).collect();
        self.go("tr_ldiff", &[lo as i32, hi as i32], &it, 4).chunks(2).map(of_long).collect()
    }
    fn lit16(&mut self, need: &V, taps: &[V]) -> V {
        let mut it = Vec::with_capacity(34 * need.len());
        for i in 0..need.len() { it.extend_from_slice(&long(need[i])); for t in taps { it.extend_from_slice(&long(t[i])); } }
        self.go("tr_lit16", &[], &it, 34).into_iter().map(|v| v as i64).collect()
    }
    fn tone_index(&mut self, col: &V, hot: &V, add: &V, x25: &Tab) -> V {
        // 25 h from the printed table, the three added
        let mut it = Vec::with_capacity(5 * col.len());
        for i in 0..col.len() { it.push(col[i] as i32); it.extend_from_slice(&long(x25.data[hot[i] as usize])); it.extend_from_slice(&long(add[i])); }
        self.go("tr_tone1", &[], &it, 5).chunks(2).map(of_long).collect()
    }
    fn vel(&mut self, ux: &V, rx: &V, uy: &V, ry: &V) -> (V, V) {
        let mut it = Vec::with_capacity(4 * ux.len());
        for i in 0..ux.len() { it.extend_from_slice(&[ux[i] as i32, rx[i] as i32, uy[i] as i32, ry[i] as i32]); }
        let r = self.go("tr_vel", &[], &it, 4);
        (r.iter().step_by(2).map(|&v| v as i64).collect(), r.iter().skip(1).step_by(2).map(|&v| v as i64).collect())
    }
    fn light_sum(&mut self, a: &V, s: &V, e: &V) -> (V, V) {
        let mut it = Vec::with_capacity(3 * a.len());
        for i in 0..a.len() { it.extend_from_slice(&[a[i] as i32, s[i] as i32, e[i] as i32]); }
        let r = self.go("tr_lsum", &[], &it, 3);
        (r.iter().step_by(2).map(|&v| v as i64).collect(), r.iter().skip(1).step_by(2).map(|&v| v as i64).collect())
    }
    fn absless(&mut self, x: &V, h: A) -> (V, V) {
        let mut it = Vec::with_capacity(4 * x.len());
        for i in 0..x.len() { it.extend_from_slice(&long(x[i])); it.extend_from_slice(&long(h.at(i))); }
        let r = self.go("tr_absl", &[], &it, 4);
        (r.iter().step_by(2).map(|&v| v as i64).collect(), r.iter().skip(1).step_by(2).map(|&v| v as i64).collect())
    }
    fn walka(&mut self, walks: &[(i64, i64, i64, i64)]) -> V {
        let mut it = Vec::with_capacity(8 * walks.len());
        for &(n, va, a, f0) in walks {
            it.extend_from_slice(&long(n)); it.extend_from_slice(&long(va)); it.extend_from_slice(&long(fl(a, 1000))); it.push(md(a, 1000) as i32); it.push(f0 as i32);
        }
        self.go("tr_walka", &[], &it, 8).chunks(2).map(of_long).collect()
    }
    fn walkc(&mut self, walks: &[(i64, i64, i64, i64)]) -> V {
        let mut it = Vec::with_capacity(5 * walks.len());
        for &(n, va, a, f0) in walks { it.extend_from_slice(&[n as i32, va as i32, fl(a, 1000) as i32, md(a, 1000) as i32, f0 as i32]); }
        self.go("tr_walkc", &[], &it, 5).into_iter().map(|v| v as i64).collect()
    }
    fn ends(&mut self, k: &V, h: &V, last: &V) -> (V, V, V) {
        let mut it = Vec::with_capacity(5 * k.len());
        for i in 0..k.len() { it.extend_from_slice(&long(k[i])); it.push(h[i] as i32); it.extend_from_slice(&long(last[i])); }
        let r = self.go("tr_ends", &[], &it, 5);
        (r.chunks(5).map(|c| of_long(&c[0..2])).collect(), r.chunks(5).map(|c| of_long(&c[2..4])).collect(), r.chunks(5).map(|c| c[4] as i64).collect())
    }
    fn up4(&mut self, lines: &[&[i64]]) -> V {
        let (mut cards, mut ends) = (vec![], Vec::with_capacity(lines.len()));
        for l in lines { cards.push(l.len() as i32); cards.extend(l.iter().map(|&x| x as i32)); ends.push(cards.len()); }
        self.go_flat("tr_up4", &[], &cards, &ends).into_iter().map(|v| v as i64).collect()
    }
    fn tone_push(&mut self, t: [&V; 3], div20: &Tab) -> [V; 3] {
        // 3 t - s each (tr_push1), over 20 from the printed table, added and held (tr_push2)
        let n = t[0].len();
        let mut it = Vec::with_capacity(3 * n);
        for i in 0..n { for c in 0..3 { it.push(t[c][i] as i32); } }
        let d = self.go("tr_push1", &[], &it, 3);
        let mut it2 = Vec::with_capacity(6 * n);
        for i in 0..n { for c in 0..3 { it2.push(t[c][i] as i32); } for c in 0..3 { it2.push(div20.data[(d[3 * i + c] + 510) as usize] as i32); } }
        let r = self.go("tr_push2", &[], &it2, 6);
        std::array::from_fn(|c| r.iter().skip(c).step_by(3).map(|&v| v as i64).collect())
    }
    fn runsum(&mut self, lists: &[&[i64]]) -> V {
        let (mut cards, mut ends) = (vec![], Vec::with_capacity(lists.len()));
        for l in lists { cards.extend_from_slice(&long(l.len() as i64)); cards.extend(l.iter().map(|&x| x as i32)); ends.push(cards.len()); }
        self.go_flat("tr_runsum", &[], &cards, &ends).chunks(2).map(of_long).collect()
    }
    fn nearest(&mut self, groups: &[&[i64]]) -> Vec<usize> {
        let (mut cards, mut ends) = (Vec::with_capacity(groups.iter().map(|g| 1 + 2 * g.len()).sum()), Vec::with_capacity(groups.len()));
        for g in groups { cards.push(g.len() as i32); for &z in g.iter() { cards.extend_from_slice(&long(z)); } ends.push(cards.len()); }
        self.go_flat("tr_near", &[], &cards, &ends).into_iter().map(|v| v as usize).collect()
    }

    fn walk(&mut self, walks: &[(i64, i64, i64, i64)], window: Option<(i64, i64)>) -> V {
        // the pieces (a walk over 998 places in halves, as the definition says), each one's change over n + 1
        let pieces = imp::walk_pieces(walks, |m| {
            // the value where a long walk is halved: |change| h over n (tr: lin and the long division), its sign back
            let size: V = m.iter().map(|q| (q.1 - q.0).abs()).collect();
            let h: V = m.iter().map(|q| q.2).collect();
            let nn: V = m.iter().map(|q| q.3).collect();
            let p = self.lin(&[(A::V(&size), Some(A::V(&h)))]);
            let (q, _) = self.divrem(&p, &nn, 0);
            let up: V = m.iter().map(|q| (q.1 >= q.0) as i64).collect();
            let a: V = m.iter().map(|q| q.0).collect();
            let plus = self.sum(&[(A::V(&a), 1), (A::V(&q), 1)], 0);
            let minus = self.sum(&[(A::V(&a), 1), (A::V(&q), -1)], 0);
            self.sel(&up, A::V(&plus), A::V(&minus))
        });
        let n = pieces.len();
        let mut out = vec![];
        if n == 0 { return out; }
        let size: V = pieces.iter().map(|p| (p.3 - p.2).abs()).collect();
        let steps: V = pieces.iter().map(|p| p.1 + 1).collect();
        let (whole, rem) = self.divrem(&size, &steps, 0);
        let mut vals: Vec<V> = vec![vec![]; n];
        for up in [true, false] {
            let at: Vec<usize> = (0..n).filter(|&i| (pieces[i].3 >= pieces[i].2) == up && pieces[i].1 > 0).collect();
            let mut it = Vec::with_capacity(7 * at.len());
            for &i in &at { it.push(pieces[i].1 as i32); it.extend_from_slice(&long(pieces[i].2)); it.extend_from_slice(&long(whole[i])); it.push(rem[i] as i32); it.push(steps[i] as i32); }
            let v = self.go(if up { "tr_walk_up" } else { "tr_walk_down" }, &[], &it, 7);
            let mut k = 0;
            for &i in &at { let m = pieces[i].1 as usize; vals[i] = v[2 * k..2 * (k + m)].chunks(2).map(of_long).collect(); k += m; }
        }
        for (i, p) in pieces.iter().enumerate() {
            for (j, &x) in vals[i].iter().enumerate() {
                let place = p.0 + j as i64;
                if let Some((lo, hi)) = window { if place < lo || place >= hi { continue; } }
                out.push(x);
            }
        }
        out
    }

    fn crossings(&mut self, edges: &[([i64; 2], [i64; 2])], range: (i64, i64)) -> Vec<(i64, Vec<(i64, i64, i64)>)> {
        let n = edges.len();
        if n == 0 { return vec![]; }
        // the rows each meets (tr_row), and the asked-for ones
        let mut it = Vec::with_capacity(4 * n);
        for e in edges { it.extend_from_slice(&long(e.0[1])); it.extend_from_slice(&long(e.1[1])); }
        let rw = self.go("tr_row", &[], &it, 2);
        let first: V = (0..n).map(|i| rw[2 * i] as i64).collect();
        let last: V = (0..n).map(|i| rw[2 * i + 1] as i64).collect();
        let rows = self.sum(&[(A::V(&last), 1), (A::V(&first), -1)], 0);
        let rows = self.max(A::V(&rows), A::K(0));
        let j0 = self.sum(&[(A::V(&first), -1)], range.0);
        let j0 = self.max(A::V(&j0), A::K(0));
        let j1 = self.sum(&[(A::V(&first), -1)], range.1);
        let j1 = self.min(A::V(&j1), A::V(&rows));
        let cnt = self.sum(&[(A::V(&j1), 1), (A::V(&j0), -1)], 0);
        let cnt = self.max(A::V(&cnt), A::K(0));
        let mut found: Vec<(i64, Vec<(i64, i64, i64)>)> = (0..n).map(|i| (rows[i], vec![])).collect();
        // pieces of at most 998 rows: each piece's first row's offset below the top, 100 row + 50 - a.y
        let mut pc = vec![];      // (edge, first place along it, rows)
        for i in 0..n { let mut j = j0[i]; while j < j0[i] + cnt[i] { let m = (j0[i] + cnt[i] - j).min(998); pc.push((i, j, m)); j += m; } }
        if pc.is_empty() { return found; }
        let np = pc.len();
        // per edge: the run times the first row's offset (1..100) and times 100, each over the height with its
        // remainder; per piece, its first place j: the crossing q0 + j qs + fl(r0 + j rs, h) and that remainder
        let r100 = self.mul(A::V(&first), A::K(100), 0);
        let ayv: V = edges.iter().map(|e| e.0[1]).collect();
        let offf = self.sum(&[(A::V(&r100), 1), (A::V(&ayv), -1)], 50);
        let runv: V = edges.iter().map(|e| e.1[0] - e.0[0]).collect();
        let hv: V = edges.iter().map(|e| e.1[1] - e.0[1]).collect();
        let runv = self.sum(&[(A::V(&runv), 1)], 0);
        let hv = self.sum(&[(A::V(&hv), 1)], 0);
        let hv1 = self.max(A::V(&hv), A::K(1));
        let a = self.lin(&[(A::V(&runv), Some(A::V(&offf)))]);
        let (q0, r0e) = self.divrem(&a, &hv1, 0);
        let b = self.lin(&[(A::V(&runv), Some(A::K(100)))]);
        let (qse, rse) = self.divrem(&b, &hv1, 0);
        let g = |v: &V| -> V { pc.iter().map(|p| v[p.0]).collect() };
        let jv: V = pc.iter().map(|p| p.1).collect();
        let hgt = g(&hv1);
        let (qs, rs) = (g(&qse), g(&rse));
        let t = self.lin(&[(A::V(&g(&r0e)), None), (A::V(&jv), Some(A::V(&rs)))]);
        let (c, r0) = self.divrem(&t, &hgt, 0);
        let x0 = self.lin(&[(A::V(&g(&q0)), None), (A::V(&jv), Some(A::V(&qs))), (A::V(&c), None)]);
        let rh = self.sum(&[(A::V(&r0), 1), (A::V(&hgt), -1)], 0);
        let mut it = vec![];
        for k in 0..np { let mut v = vec![pc[k].2 as i32]; v.extend_from_slice(&long(rh[k])); v.extend_from_slice(&long(rs[k])); v.extend_from_slice(&long(hgt[k])); it.push(v); }
        let carries = self.go_var("tr_carry", &[], &it);
        let mut it = vec![];
        let mut at = 0;
        for k in 0..np {
            let m = pc[k].2 as usize;
            let mut v = vec![m as i32];
            v.extend_from_slice(&long(x0[k]));
            v.extend_from_slice(&long(qs[k]));
            v.extend_from_slice(&carries[at..at + m - 1]);
            at += m - 1;
            it.push(v);
        }
        let xs: V = self.go_var("tr_cross", &[], &it).chunks(2).map(of_long).collect();
        // x the top's plus the crossing; the pixel fl(x + 49, 100)
        let mut ax = vec![];
        for k in 0..np { for _ in 0..pc[k].2 { ax.push(edges[pc[k].0].0[0]); } }
        let x = self.sum(&[(A::V(&ax), 1), (A::V(&xs), 1)], 0);
        let mut it = Vec::with_capacity(2 * x.len());
        for &v in &x { it.extend_from_slice(&long(v)); }
        let px = self.go("tr_row", &[], &it, 2);
        let mut at = 0;
        for k in 0..np {
            let (i, j, m) = pc[k];
            for q in 0..m { found[i].1.push((first[i] + j + q, px[at] as i64, j + q)); at += 1; }
        }
        found
    }
}

/// The ride, rendered on CARDIAC and checked against the reference.
/// The ride's frames.
pub const FRAMES: usize = imp::FRAMES as usize;

/// The player's job before the train's blocks: the start (the tables it works out on CARDIAC: the screen's, the
/// sky's constant parts, the shadow maps, the tunnel's light), kept for every block (live::keep).
pub fn setup(demo: &Path) {
    let mut o = Cardiac::new(demo);
    let world = imp::load(&mut o, demo.to_str().unwrap());
    live::count(o.host.steps);
    live::keep("train", world);
}

pub fn render(demo: &Path, refbin: &str) {
    // live: this block's frames, from the start the player kept (or worked out here), as a part of the video
    if live::on() {
        let mut o = Cardiac::new(demo);
        let kept = live::kept::<imp::World>("train");
        let own;
        let world: &imp::World = match kept.as_deref() { Some(w) => w, None => { own = imp::load(&mut o, demo.to_str().unwrap()); &own } };
        const FRAME: usize = 1536 * 768 * 3;
        let mut bytes = Vec::with_capacity(live::range(FRAMES).len() * FRAME);
        // the standing frames are one picture: drawn once (copying it is bookkeeping)
        let mut last: Option<(i64, Vec<u8>)> = None;
        for k in live::frames(FRAMES) {
            let k = k as i64;
            if last.as_ref().is_none_or(|(t, _)| *t != imp::ride_t(k)) { last = Some((imp::ride_t(k), imp::render_v(&mut o, world, k, true))); }
            bytes.extend_from_slice(&last.as_ref().unwrap().1);
        }
        live::count(o.host.steps);
        let path = demo.join("train/video_hd.rgb");
        match live::block() { Some((a, _)) => live::put_part(&path, a.min(FRAMES), FRAME, bytes), None => live::put(&path, bytes) }
        return;
    }
    if std::env::var("TRAIN_BOTH").is_ok() {
        let dir = demo.to_str().unwrap();
        let mut o = Both { n: imp::Native::new(imp::read_trig(dir)), c: Cardiac::new(demo), op: 0, last: 0, cost: Default::default() };
        // the start worked out natively (it's checked whole by the ordinary render)
        // TRAIN_BOTH_SETUP: the start too, both ways (for its costs)
        let world = if std::env::var("TRAIN_BOTH_SETUP").is_ok() { imp::load(&mut o, dir) } else { imp::load(&mut o.n, dir) };
        o.last = o.c.host.steps;
        for k in std::env::var("TRAIN_FRAMES").unwrap().split(',').map(|v| v.parse::<i64>().unwrap()) {
            imp::render(&mut o, &world, k);
            eprintln!("frame {k}: the same, {} operations", o.op);
        }
        if std::env::var("TRAIN_OPS").is_ok() {
            let mut v: Vec<_> = o.cost.iter().collect();
            v.sort_by_key(|(_, c)| std::cmp::Reverse(c.0));
            for ((s, w), (c, n)) in v.iter().take(std::env::var("TRAIN_OPS").unwrap().parse().unwrap_or(120)) { eprintln!("  {s:10} {w:28} {:>8.1}M {n:>5}", *c as f64 / 1e6); }
        }
        return;
    }
    const W: usize = 1536;
    const H: usize = 768;
    let mut o = Cardiac::new(demo);
    let dir = demo.to_str().unwrap();
    // TRAIN_SETUP_NATIVE: the start worked out natively (quicker, for measuring frames; not a checked render)
    let world = if std::env::var("TRAIN_SETUP_NATIVE").is_ok() { imp::load(&mut imp::Native::new(imp::read_trig(dir)), dir) } else { imp::load(&mut o, dir) };
    // the party version (train/video_hd.rgb, the live cut's) unless TRAIN_FULL (train/video_full.rgb, for --hires);
    // the reference (this process's environment) draws the same
    let party = std::env::var("TRAIN_FULL").is_err();
    let only: Option<Vec<i64>> = std::env::var("TRAIN_FRAMES").ok().map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let frames: Vec<i64> = (0..imp::FRAMES).filter(|k| only.as_ref().is_none_or(|l| l.contains(k))).collect();
    let mut video = cardiac_host::Video::<1536, 768>::new(frames.len());
    let mut last: Option<(i64, Vec<u8>)> = None;
    for (i, &k) in frames.iter().enumerate() {
        // the standing frames are one picture: drawn once
        if last.as_ref().is_none_or(|(t, _)| *t != imp::ride_t(k)) { last = Some((imp::ride_t(k), imp::render_v(&mut o, &world, k, party))); }
        video.buf[i * W * H * 3..(i + 1) * W * H * 3].copy_from_slice(&last.as_ref().unwrap().1);
        eprintln!("frame {k}: {:.1}G so far", o.host.steps as f64 / 1e9);
    }
    o.tally();
    o.host.done();
    if std::env::var("TRAIN_STAGES").is_ok() {
        for (s, c) in &o.by_stage { eprintln!("  {s:10} {:>9.1}M", *c as f64 / 1e6); }
        let mut v: Vec<_> = o.by_machine.iter().collect();
        v.sort_by_key(|(_, c)| std::cmp::Reverse(c.0));
        for ((s, m), (c, n)) in v.iter().take(60) { eprintln!("    {s:10} {m:12} {:>9.1}M {:>10} items {:>6.1} each", *c as f64 / 1e6, n, *c as f64 / (*n).max(1) as f64); }
    }
    let list = frames.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(",");
    let reference = cardiac_host::reference(refbin, &[demo, Path::new(&list)]);
    let out = std::env::var("TRAIN_OUT").map(std::path::PathBuf::from).unwrap_or_else(|_| demo.join(if party { "train/video_hd.rgb" } else { "train/video_full.rgb" }));
    finish("train", &o.host, &video, &reference, &out, if party { " (party: 768x384 world, 1536x768 board and words)" } else { " (full, 1536x768)" });
}

/// Every operation done both natively and on CARDIAC, stopping at the first that differs (TRAIN_BOTH=1): for finding
/// where a frame parts from the reference.
pub struct Both { pub n: imp::Native, pub c: Cardiac, pub op: u64, pub last: u64, pub cost: std::collections::BTreeMap<(&'static str, String), (u64, u64)> }

impl Both {
    #[track_caller]
    fn check<T: PartialEq + std::fmt::Debug>(&mut self, what: &str, a: T, b: T) -> T {
        self.op += 1;
        let l = std::panic::Location::caller();
        let e = self.cost.entry((self.c.stage, format!("{what} {}:{}", l.file().rsplit('/').next().unwrap(), l.line()))).or_insert((0, 0));
        e.0 += self.c.host.steps - self.last; e.1 += 1;
        self.last = self.c.host.steps;
        if a != b {
            let (sa, sb) = (format!("{a:?}"), format!("{b:?}"));
            let at = sa.chars().zip(sb.chars()).position(|(x, y)| x != y).unwrap_or(0);
            panic!("{what} (operation {}, stage {}) differs at character {at}:\n native  ..{}\n CARDIAC ..{}", self.op, self.c.stage, &sa[at.saturating_sub(80)..(at + 80).min(sa.len())], &sb[at.saturating_sub(80)..(at + 80).min(sb.len())]);
        }
        a
    }
}

impl Ops for Both {
    fn stage(&mut self, name: &'static str) { self.n.stage(name); self.c.stage(name); }
    #[track_caller] fn tab(&mut self, t: &Tab, i: A) -> V { let (a, b) = (self.n.tab(t, i), self.c.tab(t, i)); self.check("tab", a, b) }
    #[track_caller] fn tab2(&mut self, t: &Tab, i: A, j: A) -> V { let (a, b) = (self.n.tab2(t, i, j), self.c.tab2(t, i, j)); self.check("tab2", a, b) }
    #[track_caller] fn sel(&mut self, f: &[i64], x: A, y: A) -> V { let (a, b) = (self.n.sel(f, x, y), self.c.sel(f, x, y)); self.check("sel", a, b) }
    #[track_caller] fn sum(&mut self, t: &[(A, i64)], k: i64) -> V { let (a, b) = (self.n.sum(t, k), self.c.sum(t, k)); self.check("sum", a, b) }
    #[track_caller] fn pct(&mut self, x: A, t: A) -> V { let (a, b) = (self.n.pct(x, t), self.c.pct(x, t)); self.check("pct", a, b) }
    #[track_caller] fn mix(&mut self, x: A, y: A, t: A) -> V { let (a, b) = (self.n.mix(x, y, t), self.c.mix(x, y, t)); self.check("mix", a, b) }
    #[track_caller] fn mul(&mut self, x: A, y: A, s: u32) -> V { let (a, b) = (self.n.mul(x, y, s), self.c.mul(x, y, s)); self.check("mul", a, b) }
    #[track_caller] fn clamp(&mut self, x: A, lo: i64, hi: i64) -> V { let (a, b) = (self.n.clamp(x, lo, hi), self.c.clamp(x, lo, hi)); self.check("clamp", a, b) }
    #[track_caller] fn lt(&mut self, x: A, y: A) -> V { let (a, b) = (self.n.lt(x, y), self.c.lt(x, y)); self.check("lt", a, b) }
    #[track_caller] fn max(&mut self, x: A, y: A) -> V { let (a, b) = (self.n.max(x, y), self.c.max(x, y)); self.check("max", a, b) }
    #[track_caller] fn min(&mut self, x: A, y: A) -> V { let (a, b) = (self.n.min(x, y), self.c.min(x, y)); self.check("min", a, b) }
    #[track_caller] fn divmod(&mut self, x: A, d: A) -> (V, V) { let (a, b) = (self.n.divmod(x, d), self.c.divmod(x, d)); self.check("divmod", a, b) }
    #[track_caller] fn lin(&mut self, t: &[(A, Option<A>)]) -> V {
        let (a, b) = (self.n.lin(t), self.c.lin(t));
        if a != b { let i = (0..a.len()).find(|&i| a[i] != b[i]).unwrap(); eprintln!("lin terms at {i}: {:?}", t.iter().map(|(x, y)| (x.at(i), y.map(|y| y.at(i)))).collect::<Vec<_>>()); }
        self.check("lin", a, b)
    }
    #[track_caller] fn div(&mut self, x: A, d: A, e: u32) -> V { let (a, b) = (self.n.div(x, d, e), self.c.div(x, d, e)); self.check("div", a, b) }
    #[track_caller] fn sincos(&mut self, x: A) -> (V, V) { let (a, b) = (self.n.sincos(x), self.c.sincos(x)); self.check("sincos", a, b) }
    #[track_caller] fn sqrt(&mut self, x: A) -> V { let (a, b) = (self.n.sqrt(x), self.c.sqrt(x)); self.check("sqrt", a, b) }
    #[track_caller] fn muldiv(&mut self, x: A, y: A, d: i64) -> V { let (a, b) = (self.n.muldiv(x, y, d), self.c.muldiv(x, y, d)); self.check("muldiv", a, b) }
    #[track_caller] fn wrap(&mut self, u: A, d: i64) -> V { let (a, b) = (self.n.wrap(u, d), self.c.wrap(u, d)); self.check("wrap", a, b) }
    #[track_caller] fn phase(&mut self, u: A, r: i64, s: u32) -> (V, V) { let (a, b) = (self.n.phase(u, r, s), self.c.phase(u, r, s)); self.check("phase", a, b) }
    #[track_caller] fn turns(&mut self, x: A) -> V { let (a, b) = (self.n.turns(x), self.c.turns(x)); self.check("turns", a, b) }
    #[track_caller] fn crossings(&mut self, e: &[([i64; 2], [i64; 2])], r: (i64, i64)) -> Vec<(i64, Vec<(i64, i64, i64)>)> { let (a, b) = (self.n.crossings(e, r), self.c.crossings(e, r)); self.check("crossings", a, b) }
    #[track_caller] fn walk(&mut self, w: &[(i64, i64, i64, i64)], win: Option<(i64, i64)>) -> V { let (a, b) = (self.n.walk(w, win), self.c.walk(w, win)); self.check("walk", a, b) }
    #[track_caller] fn recip(&mut self, t: &Tab, iz: A) -> V { let (a, b) = (self.n.recip(t, iz), self.c.recip(t, iz)); self.check("recip", a, b) }
    #[track_caller] fn runsum(&mut self, l: &[&[i64]]) -> V { let (a, b) = (self.n.runsum(l), self.c.runsum(l)); self.check("runsum", a, b) }
    #[track_caller] fn ldiff(&mut self, x: &V, y: &V, lo: i64, hi: i64) -> V { let (a, b) = (self.n.ldiff(x, y, lo, hi), self.c.ldiff(x, y, lo, hi)); self.check("ldiff", a, b) }
    #[track_caller] fn lit16(&mut self, x: &V, t: &[V]) -> V { let (a, b) = (self.n.lit16(x, t), self.c.lit16(x, t)); self.check("lit16", a, b) }
    #[track_caller] fn light_sum(&mut self, a: &V, b: &V, c: &V) -> (V, V) { let (x, y) = (self.n.light_sum(a, b, c), self.c.light_sum(a, b, c)); self.check("light_sum", x, y) }
    #[track_caller] fn absless(&mut self, x: &V, h: A) -> (V, V) { let (a, b) = (self.n.absless(x, h), self.c.absless(x, h)); self.check("absless", a, b) }
    #[track_caller] fn walka(&mut self, w: &[(i64, i64, i64, i64)]) -> V { let (a, b) = (self.n.walka(w), self.c.walka(w)); self.check("walka", a, b) }
    #[track_caller] fn walkc(&mut self, w: &[(i64, i64, i64, i64)]) -> V { let (a, b) = (self.n.walkc(w), self.c.walkc(w)); self.check("walkc", a, b) }
    #[track_caller] fn vel(&mut self, a: &V, b: &V, c: &V, d: &V) -> (V, V) { let (x, y) = (self.n.vel(a, b, c, d), self.c.vel(a, b, c, d)); self.check("vel", x, y) }
    #[track_caller] fn ends(&mut self, k: &V, h: &V, l: &V) -> (V, V, V) { let (a, b) = (self.n.ends(k, h, l), self.c.ends(k, h, l)); self.check("ends", a, b) }
    #[track_caller] fn up4(&mut self, l: &[&[i64]]) -> V { let (a, b) = (self.n.up4(l), self.c.up4(l)); self.check("up4", a, b) }
    #[track_caller] fn tone_index(&mut self, c: &V, h: &V, x: &V, t: &Tab) -> V { let (a, b) = (self.n.tone_index(c, h, x, t), self.c.tone_index(c, h, x, t)); self.check("tone_index", a, b) }
    #[track_caller] fn tone_push(&mut self, t: [&V; 3], d: &Tab) -> [V; 3] { let (a, b) = (self.n.tone_push(t, d), self.c.tone_push(t, d)); self.check("tone_push", a, b) }
    #[track_caller] fn walkseg(&mut self, s: &[(i64, i64, i64)], w: bool) -> V { let (a, b) = (self.n.walkseg(s, w), self.c.walkseg(s, w)); self.check("walkseg", a, b) }
    #[track_caller] fn nearest(&mut self, g: &[&[i64]]) -> Vec<usize> { let (a, b) = (self.n.nearest(g), self.c.nearest(g)); self.check("nearest", a, b) }
}
