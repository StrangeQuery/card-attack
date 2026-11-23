//! The swarm's sums, written for the swarm (no longer in the film) and used now by most of the film's 3D: numbers as
//! the steps the machines take to work them out. Each number is a node: cards dealt (the frame's data, the
//! reference's constants), a machine's run on the cards of the nodes before it, a printed table's entry at an index the machines worked out, or one of two numbers as a machine's
//! flag says. The host builds the nodes as the reference's formulas say, then works them out a level at a time: each
//! level's runs of one machine in one deck (the sorter's work: it only carries cards). A number is one card, or three
//! (millions, thousands, rest; the last two 0..999), or for a few a long (thousands, rest).
//!
//! The host's bookkeeping is kept in flat arrays (every node's arguments in one, every node's cards in another) and
//! reused: by the calculator between evaluations, and between calculators on one thread (a calculator dropped gives
//! its buffers, cleared, to the next one made). None of that changes what the machines are dealt or in what order.
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// A node: a number (or a machine's cards) of the sums.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct N(u32);

/// Cards for a machine's deck: a node's, some of a node's (from, how many), or one written in (a count, a place, a
/// zero: the deck's own bookkeeping).
#[derive(Clone, Copy, Debug)]
pub enum Arg { All(N), Some(N, u16, u16), Lit(i32) }

/// The printed tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tab { Quarter, Trig, Atan, Rsq, Spec12, Spec30 }

/// A node's step. Arguments are a range of `Calc::args` (start, count); dealt cards a range of `Calc::lits`.
#[derive(Clone, Copy)]
enum Op {
    Cards(u32, u32),
    /// a machine (its place in `Calc::names`), its arguments, how many cards it punches for the item
    Run(u16, u32, u32, u32),
    Table(Tab, u32, u32),
    /// a flag's card 1 picks the first, else the second
    Sel(Arg, N, N),
    /// cards arranged: a node's cards picked out, a zero card put before a long that is never below 0
    Cat(u32, u32),
}

/// The printed tables, as the host looks them up: by their indices' cards.
pub struct Tables {
    /// n^2 / 4 for n = 0..1998, by n's thousands and rest: a long
    pub quarter: Vec<Vec<[i32; 2]>>,
    /// 999 sin every thousandth of a turn, a quarter turn, each entry printed with the next
    pub trig: Vec<[i32; 2]>,
    /// atan every 2000th, millionths of a turn, by the index's thousands and rest: a long, printed with the next
    pub atan: Vec<Vec<[i32; 4]>>,
    /// the highlight's scale, by the index's thousands and rest: a long
    pub rsq: Vec<Vec<[i32; 2]>>,
    pub spec12: Vec<i32>,
    pub spec30: Vec<i32>,
}

/// The printed tables (swarm/data/tables.txt), by their indices' cards.
pub(crate) fn load_tables(text: &str) -> Tables {
    let get = |name: &str| -> Vec<i32> {
        let l = text.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap_or_else(|| panic!("no table {name}"));
        l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect()
    };
    // a number's cards: thousands and rest, as the data prints it
    let long = |v: i32| -> [i32; 2] { [v.div_euclid(1000), v.rem_euclid(1000)] };
    let by_cards = |t: &[i32], n: usize| -> Vec<Vec<[i32; 2]>> { (0..n).map(|h| t.iter().skip(1000 * h).take(1000).map(|&v| long(v)).collect()).collect() };
    let trig = get("trig");
    let atan = get("atan");
    let quarter = get("quarter");
    let rsq = get("rsq");
    Tables {
        quarter: by_cards(&quarter, 2),
        trig: (0..trig.len()).map(|k| [trig[k], trig[(k + 1).min(trig.len() - 1)]]).collect(),
        atan: (0..3).map(|h| (0..1000).filter(|&l| 1000 * h + l < atan.len()).map(|l| { let k = 1000 * h + l; let (a, b) = (long(atan[k]), long(atan[(k + 1).min(atan.len() - 1)])); [a[0], a[1], b[0], b[1]] }).collect()).collect(),
        rsq: by_cards(&rsq, 3),
        spec12: get("spec12"),
        spec30: get("spec30"),
    }
}

pub struct Calc {
    ops: Vec<Op>,
    /// every op's arguments, one after another
    args: Vec<Arg>,
    /// the dealt numbers' cards, one after another
    lits: Vec<i32>,
    /// the machines named, each once
    names: Vec<&'static str>,
    level: Vec<u32>,
    /// how many cards a number has (1 or 3; 2 for a long; 0 for a machine's cards that are not one number)
    width: Vec<u8>,
    /// the largest size the number can have, as the reference's ranges say
    bound: Vec<i64>,
    /// every worked-out node's cards, one after another; a node's start and count in `at` (0 cards until worked out)
    vals: Vec<i32>,
    at: Vec<(u32, u32)>,
    done: usize,
    consts: HashMap<i64, N, BuildHasherDefault<Fx>>,
    /// a number's sign and size (`mag`), once built (NONE until then)
    mags: Vec<u32>,
    // scratch, kept between evaluations: the order the nodes are worked out in, the decks dealt
    order: Vec<(u32, u16, u32)>,
    locals: Vec<(u32, u32)>,
    rank: Vec<u16>,
    deck: Vec<i32>,
    ends: Vec<u32>,
    scratch: Vec<Arg>,
}

const NONE: u32 = u32::MAX;

/// A machine's items for one evaluation: their cards one after another, and where each item's end.
pub struct Items<'a> { pub cards: &'a [i32], pub ends: &'a [u32] }

impl<'a> Items<'a> {
    pub fn len(&self) -> usize { self.ends.len() }
    pub fn is_empty(&self) -> bool { self.ends.is_empty() }
    /// The items, at most `k` at a time: how many, and their cards.
    pub fn chunks(&self, k: usize) -> impl Iterator<Item = (usize, &'a [i32])> + 'a {
        let (cards, ends) = (self.cards, self.ends);
        (0..ends.len()).step_by(k).map(move |a| {
            let b = (a + k).min(ends.len());
            let from = if a == 0 { 0 } else { ends[a - 1] as usize };
            (b - a, &cards[from..ends[b - 1] as usize])
        })
    }
}

/// A plain multiplicative hash for the dealt numbers (the standard one is slow for a key this small).
#[derive(Default)]
pub struct Fx(u64);
impl Hasher for Fx {
    fn finish(&self) -> u64 { self.0 }
    fn write(&mut self, b: &[u8]) { for &x in b { self.write_u64(x as u64) } }
    fn write_u64(&mut self, v: u64) { self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95); }
    fn write_i64(&mut self, v: i64) { self.write_u64(v as u64) }
}

thread_local! {
    /// Calculators' buffers, cleared, for the next calculator made on this thread (most effects make one a frame).
    static POOL: RefCell<Vec<Calc>> = const { RefCell::new(Vec::new()) };
}

/// A calculator's buffers this large (cards) are let go, not kept for the next.
const POOL_MAX: usize = 1 << 24;

impl Drop for Calc {
    fn drop(&mut self) {
        if self.ops.capacity() == 0 || self.vals.capacity() > POOL_MAX || self.args.capacity() > POOL_MAX { return; }
        if !POOL.try_with(|p| p.try_borrow().is_ok_and(|p| p.len() < 4)).unwrap_or(false) { return; }
        let mut c = Calc::empty();
        std::mem::swap(self, &mut c);
        c.clear();
        let _ = POOL.try_with(|p| if let Ok(mut p) = p.try_borrow_mut() { p.push(c) });
    }
}

/// How many of a size's cards can be other than 0.
fn limbs(bound: i64) -> usize { if bound < 1000 { 1 } else if bound < 1_000_000 { 2 } else { 3 } }

/// The cards an op's arguments deal, onto `out`.
fn deal(args: &[Arg], vals: &[i32], at: &[(u32, u32)], out: &mut Vec<i32>) {
    for a in args {
        match *a {
            Arg::All(n) => { let (s, l) = at[n.0 as usize]; out.extend_from_slice(&vals[s as usize..(s + l) as usize]) }
            Arg::Some(n, f, l) => {
                let (s, len) = at[n.0 as usize];
                let (f, l) = (f as usize, l as usize);
                out.extend_from_slice(&vals[s as usize..(s + len) as usize][f..f + l])
            }
            Arg::Lit(c) => out.push(c),
        }
    }
}

impl Default for Calc { fn default() -> Self { Self::new() } }

impl Calc {
    fn empty() -> Self {
        Calc { ops: vec![], args: vec![], lits: vec![], names: vec![], level: vec![], width: vec![], bound: vec![], vals: vec![], at: vec![],
            done: 0, consts: HashMap::default(), mags: vec![], order: vec![], locals: vec![], rank: vec![], deck: vec![], ends: vec![], scratch: vec![] }
    }
    fn clear(&mut self) {
        self.ops.clear(); self.args.clear(); self.lits.clear(); self.names.clear(); self.level.clear(); self.width.clear(); self.bound.clear();
        self.vals.clear(); self.at.clear(); self.done = 0; self.consts.clear(); self.mags.clear();
        self.order.clear(); self.locals.clear(); self.rank.clear(); self.deck.clear(); self.ends.clear(); self.scratch.clear();
    }
    pub fn new() -> Self {
        POOL.try_with(|p| p.try_borrow_mut().ok().and_then(|mut p| p.pop())).ok().flatten().unwrap_or_else(Calc::empty)
    }

    fn level_of(&self, a: &Arg) -> u32 { match a { Arg::All(n) | Arg::Some(n, _, _) => self.level[n.0 as usize] + 1, Arg::Lit(_) => 0 } }

    fn push(&mut self, op: Op, width: u8, bound: i64) -> N {
        let lv = match op {
            Op::Cards(..) => 0,
            Op::Run(_, s, l, _) | Op::Table(_, s, l) | Op::Cat(s, l) =>
                self.args[s as usize..(s + l) as usize].iter().map(|a| self.level_of(a)).max().unwrap_or(0),
            Op::Sel(f, a, b) => self.level_of(&f).max(self.level[a.0 as usize] + 1).max(self.level[b.0 as usize] + 1),
        };
        self.ops.push(op);
        self.level.push(lv);
        self.width.push(width);
        self.bound.push(bound);
        self.at.push((0, 0));
        self.mags.push(NONE);
        N(self.ops.len() as u32 - 1)
    }
    fn put_args(&mut self, a: &[Arg]) -> (u32, u32) {
        let s = self.args.len() as u32;
        self.args.extend_from_slice(a);
        (s, a.len() as u32)
    }

    pub fn width(&self, n: N) -> u8 { self.width[n.0 as usize] }
    pub fn bound(&self, n: N) -> i64 { self.bound[n.0 as usize] }
    /// A number's cards, once worked out.
    pub fn cards(&self, n: N) -> &[i32] { let (s, l) = self.at[n.0 as usize]; &self.vals[s as usize..(s + l) as usize] }
    /// Say a number is never larger than this (the reference's range for it).
    pub fn within(&mut self, n: N, bound: i64) -> N { let b = &mut self.bound[n.0 as usize]; *b = (*b).min(bound); n }

    // --- dealt ---------------------------------------------------------------------------------------------------

    /// A number from the data, dealt as cards: one card, or three.
    pub fn num(&mut self, v: i64) -> N {
        if let Some(&n) = self.consts.get(&v) { return n; }
        let s = self.lits.len() as u32;
        let n = if v.abs() <= 999 { self.lits.push(v as i32); self.push(Op::Cards(s, 1), 1, v.abs()) } else {
            assert!(v.abs() < 999_000_000, "a number past three cards: {v}");
            self.lits.extend_from_slice(&[v.div_euclid(1_000_000) as i32, v.div_euclid(1000).rem_euclid(1000) as i32, v.rem_euclid(1000) as i32]);
            self.push(Op::Cards(s, 3), 3, v.abs())
        };
        self.consts.insert(v, n);
        n
    }

    /// Cards a machine punches for one item, as a node; `width` 0 when they are not one number.
    pub fn run(&mut self, machine: &'static str, args: impl AsRef<[Arg]>, outs: usize, width: u8, bound: i64) -> N {
        let m = match self.names.iter().position(|&n| n == machine) {
            Some(m) => m,
            None => { self.names.push(machine); self.names.len() - 1 }
        } as u16;
        let (s, l) = self.put_args(args.as_ref());
        self.push(Op::Run(m, s, l, outs as u32), width, bound)
    }
    pub fn cat(&mut self, args: impl AsRef<[Arg]>, width: u8, bound: i64) -> N { let (s, l) = self.put_args(args.as_ref()); self.push(Op::Cat(s, l), width, bound) }
    pub fn part(&mut self, n: N, from: u16, len: u16, width: u8, bound: i64) -> N { self.cat([Arg::Some(n, from, len)], width, bound) }
    pub fn table(&mut self, t: Tab, index: impl AsRef<[Arg]>, width: u8, bound: i64) -> N { let (s, l) = self.put_args(index.as_ref()); self.push(Op::Table(t, s, l), width, bound) }

    /// a if the flag's card is 1, else b.
    pub fn sel(&mut self, flag: Arg, a: N, b: N) -> N {
        let (wa, wb) = (self.width(a), self.width(b));
        let (a, b) = if wa == wb { (a, b) } else { (self.wide(a), self.wide(b)) };
        let w = self.width(a);
        let bound = self.bound(a).max(self.bound(b));
        self.push(Op::Sel(flag, a, b), w, bound)
    }

    // --- arithmetic ------------------------------------------------------------------------------------------------

    /// A number as three cards.
    pub fn wide(&mut self, n: N) -> N {
        match self.width(n) {
            3 => n,
            1 => { let b = self.bound(n); self.run("sw_term", [Arg::Lit(0), Arg::Lit(1), Arg::All(n)], 3, 3, b) }
            w => panic!("not a number of one card or three: {w}"),
        }
    }

    /// A number's sign (1 below 0) and its size's three cards.
    pub fn mag(&mut self, n: N) -> N {
        if self.mags[n.0 as usize] != NONE { return N(self.mags[n.0 as usize]); }
        let w = self.width(n);
        assert!(w == 1 || w == 3);
        let b = self.bound(n);
        let m = self.run("sw_mag", [Arg::Lit(w as i32), Arg::All(n)], 4, 0, b);
        self.mags[n.0 as usize] = m.0;
        m
    }
    /// A number's size, as three cards.
    pub fn abs(&mut self, n: N) -> N { let m = self.mag(n); let b = self.bound(n); self.part(m, 1, 3, 3, b) }

    /// A sum of terms: each a product of two numbers or a number alone, added or taken away; exact, as three cards.
    pub fn lin(&mut self, terms: &[(N, Option<N>, bool)]) -> N { self.lin_within(terms, None) }
    /// The same, each product and the sum known (the reference's ranges) never to pass `known` in size.
    pub fn lin_within(&mut self, terms: &[(N, Option<N>, bool)], known: Option<i64>) -> N {
        // the sum's deck: its count, then the pieces (built in the scratch, kept for the next)
        let mut pieces = std::mem::take(&mut self.scratch);
        pieces.clear();
        pieces.push(Arg::Lit(0));
        let mut bound = 0i64;
        for &(a, b, neg) in terms {
            match b {
                Some(b) => {
                    let (ba, bb) = (self.bound(a), self.bound(b));
                    let pb = ((ba as i128) * (bb as i128)).min(known.unwrap_or(i64::MAX) as i128);
                    assert!(pb < 999_000_000, "a product past three cards: {ba} {bb}");
                    let pb = pb as i64;
                    bound += pb;
                    let (ma, mb) = (self.mag(a), self.mag(b));
                    for i in 0..limbs(ba) {
                        for j in 0..limbs(bb) {
                            if i + j > 2 { continue; }
                            let idx = self.run("sw_qis", [Arg::Some(ma, 3 - i as u16, 1), Arg::Some(mb, 3 - j as u16, 1),
                                Arg::Some(ma, 0, 1), Arg::Some(mb, 0, 1), Arg::Lit(neg as i32)], 4, 0, 0);
                            let q1 = self.table(Tab::Quarter, [Arg::Some(idx, 0, 2)], 0, 0);
                            let q2 = self.table(Tab::Quarter, [Arg::Some(idx, 2, 2)], 0, 0);
                            let p = self.run("sw_qp", [Arg::Lit((i + j) as i32), Arg::All(q1), Arg::All(q2)], 3, 3, pb);
                            pieces.push(Arg::All(p));
                        }
                    }
                }
                None => {
                    bound += self.bound(a);
                    let w = self.width(a);
                    let ba = self.bound(a);
                    if w == 3 && !neg { pieces.push(Arg::All(a)); } else {
                        let p = self.run("sw_term", [Arg::Lit(neg as i32), Arg::Lit(w as i32), Arg::All(a)], 3, 3, ba);
                        pieces.push(Arg::All(p));
                    }
                }
            }
        }
        let bound = bound.min(known.unwrap_or(i64::MAX));
        assert!(bound < 999_000_000, "a sum past three cards: {bound}");
        let r = if pieces.len() == 2 {
            let Arg::All(p) = pieces[1] else { unreachable!() };
            self.within(p, bound)
        } else {
            pieces[0] = Arg::Lit(pieces.len() as i32 - 1);
            self.run("sw_lsum", &pieces, 3, 3, bound)
        };
        self.scratch = pieces;
        r
    }
    /// a + b, a - b, a b, a b + c d, ...
    pub fn add(&mut self, a: N, b: N) -> N { self.lin(&[(a, None, false), (b, None, false)]) }
    pub fn sub(&mut self, a: N, b: N) -> N { self.lin(&[(a, None, false), (b, None, true)]) }
    pub fn neg(&mut self, a: N) -> N { self.lin(&[(a, None, true)]) }
    pub fn mul(&mut self, a: N, b: N) -> N { self.lin(&[(a, Some(b), false)]) }
    pub fn mulk(&mut self, a: N, k: i64) -> N { let k = self.num(k); self.mul(a, k) }
    pub fn addk(&mut self, a: N, k: i64) -> N { let k = self.num(k); self.add(a, k) }

    /// fl(x, 1000).
    pub fn shift(&mut self, x: N) -> N {
        let x = self.wide(x);
        let b = self.bound(x) / 1000 + 1;
        self.run("sw_shift", [Arg::All(x)], 3, 3, b)
    }
    /// fl(a b + ..., 1000): a sum of products in thousandths.
    pub fn lin_k(&mut self, terms: &[(N, Option<N>, bool)]) -> N { let s = self.lin(terms); self.shift(s) }

    /// fl(x 1000^extra, d) and the remainder (a long): d a number 1..99999, never below 0. The quotient under a
    /// million, as three cards.
    pub fn divrem(&mut self, x: N, d: N, extra: usize) -> (N, N) {
        let bd = self.bound(d);
        assert!(bd <= 99_999, "a divisor past 99999: {bd}");
        let (dl, dn) = match self.width(d) { 1 => ([Arg::Lit(0), Arg::All(d)], 2), 3 => ([Arg::Some(d, 1, 2), Arg::Lit(0)], 1), w => panic!("divisor of {w} cards") };
        let dl = &dl[..dn];
        let m = self.mag(x);
        // the size's cards that can be other than 0 (a number under a million has no millions)
        let cards = if self.bound(x) < 1_000_000 { 2 } else { 3 };
        let n = 3 * (cards + extra);
        let mut a = std::mem::take(&mut self.scratch);
        a.clear();
        a.extend_from_slice(dl);
        a.push(Arg::Lit((cards + extra) as i32));
        a.push(Arg::Some(m, 4 - cards as u16, cards as u16));
        for _ in 0..extra { a.push(Arg::Lit(0)); }
        let digits = self.run("sw_divd", &a, n + 2, 0, 0);
        let q = self.run("sw_join", [Arg::Lit(n as i32), Arg::Some(digits, 0, n as u16)], 2, 0, 0);
        a.clear();
        a.extend_from_slice(&[Arg::Some(m, 0, 1), Arg::All(q), Arg::Some(digits, n as u16, 2)]);
        a.extend_from_slice(dl);
        let mut qb = self.bound(x);
        for _ in 0..extra { qb *= 1000; }
        let f = self.run("sw_floor3", &a, 5, 0, 0);
        self.scratch = a;
        let quot = self.part(f, 0, 3, 3, qb.min(999_999));
        let rem = self.part(f, 3, 2, 2, bd);
        (quot, rem)
    }
    pub fn div(&mut self, x: N, d: N) -> N { self.divrem(x, d, 0).0 }
    pub fn divk(&mut self, x: N, d: i64) -> N { let d = self.num(d); self.div(x, d) }
    /// A long that is never below 0 as three cards.
    pub fn long3(&mut self, l: N) -> N { let b = self.bound(l); self.cat([Arg::Lit(0), Arg::All(l)], 3, b) }
    /// md(x, d): the remainder, as three cards.
    pub fn modk(&mut self, x: N, d: i64) -> N { let d = self.num(d); let r = self.divrem(x, d, 0).1; self.long3(r) }

    /// The flags of a number: below 0, and 0 (cards 0 or 1).
    pub fn sign(&mut self, x: N) -> N { let x = self.wide(x); self.run("sw_sign", [Arg::All(x)], 2, 0, 0) }
    /// 1 if a < b.
    pub fn lt(&mut self, a: N, b: N) -> Arg { let d = self.sub(a, b); let s = self.sign(d); Arg::Some(s, 0, 1) }
    pub fn below0(&mut self, a: N) -> Arg { let s = self.sign(a); Arg::Some(s, 0, 1) }
    pub fn is0(&mut self, a: N) -> Arg { let s = self.sign(a); Arg::Some(s, 1, 1) }
    /// 1 if the flag is 0, else 0.
    pub fn not(&mut self, f: Arg) -> Arg { let (z, o) = (self.num(0), self.num(1)); let v = self.sel(f, z, o); Arg::Some(v, 0, 1) }
    pub fn max(&mut self, a: N, b: N) -> N { let f = self.lt(a, b); self.sel(f, b, a) }
    pub fn min(&mut self, a: N, b: N) -> N { let f = self.lt(a, b); self.sel(f, a, b) }
    pub fn clamp(&mut self, x: N, lo: i64, hi: i64) -> N {
        let (l, h) = (self.num(lo), self.num(hi));
        let x = self.max(x, l);
        let x = self.min(x, h);
        let b = lo.abs().max(hi.abs());
        self.within(x, b)
    }

    /// 999 sin and cos of an angle (millionths of a turn, three cards: its turns ignored): s, c, -s, -c (cards).
    pub fn sincos(&mut self, a: N) -> N {
        let a = self.wide(a);
        let at = self.run("sw_ang", [Arg::Some(a, 1, 2)], 6, 0, 0);
        let t0 = self.table(Tab::Trig, [Arg::Some(at, 0, 1)], 0, 0);
        let t1 = self.table(Tab::Trig, [Arg::Some(at, 3, 1)], 0, 0);
        self.run("sw_sin", [Arg::All(t0), Arg::Some(at, 1, 2), Arg::All(t1), Arg::Some(at, 4, 2)], 4, 0, 0)
    }
    pub fn sin(&mut self, a: N) -> (N, N) {
        let s = self.sincos(a);
        (self.part(s, 0, 1, 1, 999), self.part(s, 1, 1, 1, 999))
    }

    /// The angle of (x, y) from the x axis, millionths of a turn (0 for the origin): both under a million.
    pub fn atan2(&mut self, y: N, x: N) -> N {
        let (y, x) = (self.wide(y), self.wide(x));
        assert!(self.bound(y) < 1_000_000 && self.bound(x) < 1_000_000);
        let (my, mx) = (self.mag(y), self.mag(x));
        let o = self.run("sw_order", [Arg::Some(my, 2, 2), Arg::Some(mx, 2, 2)], 6, 0, 0);
        // the smaller over the larger in hundred-thousandths (over 1 when both are 0)
        let small = self.cat([Arg::Lit(0), Arg::Some(o, 0, 2)], 3, 999_999);
        let large = self.cat([Arg::Some(o, 2, 2)], 2, 99_999);
        let one = self.cat([Arg::Lit(0), Arg::Lit(1)], 2, 1);
        let large = self.sel(Arg::Some(o, 5, 1), one, large);
        let large = self.long3(large);
        let num = self.mulk(small, 100);
        let ratio = self.divrem(num, large, 1).0;
        let a1 = self.run("sw_atan1", [Arg::Some(ratio, 1, 2)], 3, 0, 0);
        let e = self.table(Tab::Atan, [Arg::Some(a1, 0, 2)], 0, 0);
        let a2 = self.run("sw_atan2", [Arg::All(e), Arg::Some(a1, 2, 1)], 2, 0, 0);
        let q = self.run("sw_quad", [Arg::All(a2), Arg::Some(o, 4, 1), Arg::Some(mx, 0, 1), Arg::Some(my, 0, 1), Arg::Some(o, 5, 1)], 2, 2, 999_999);
        self.long3(q)
    }

    /// The whole square root of a number under ten million (three cards), as three cards.
    pub fn sqrt(&mut self, x: N) -> N {
        let x = self.wide(x);
        let b = (self.bound(x) as f64).sqrt() as i64 + 1;
        let h = self.run("sw_hund", [Arg::All(x)], 3, 0, 0);
        let a = self.run("sw_isqrt", [Arg::All(h)], 6, 0, 0);
        let r = self.run("sw_isqrt2", [Arg::All(a)], 2, 2, b);
        self.long3(r)
    }

    /// A number under a million in size as a long (thousands, rest).
    pub fn long(&mut self, x: N) -> N {
        let x = self.wide(x);
        let b = self.bound(x);
        self.run("sw_short", [Arg::All(x)], 2, 2, b)
    }

    // --- working out -----------------------------------------------------------------------------------------------

    /// Work out every node not yet worked out: a level at a time, each machine's items of the level in one deck (the
    /// machines in the order of their names, as before). `go` runs machine `name` over the items and punches each
    /// item's cards onto `out`, the items' one after another.
    pub fn eval(&mut self, go: &mut dyn FnMut(&'static str, Items, &mut Vec<i32>), t: &Tables) {
        let from = self.done;
        let to = self.ops.len();
        if from == to { return; }
        // each machine's place among the names in order
        self.rank.clear();
        self.rank.resize(self.names.len(), 0);
        {
            let mut by: Vec<u16> = (0..self.names.len() as u16).collect();
            by.sort_unstable_by_key(|&m| self.names[m as usize]);
            for (r, &m) in by.iter().enumerate() { self.rank[m as usize] = r as u16; }
        }
        self.order.clear();
        self.locals.clear();
        for i in from..to {
            match self.ops[i] {
                Op::Run(m, ..) => self.order.push((self.level[i], self.rank[m as usize], i as u32)),
                _ => self.locals.push((self.level[i], i as u32)),
            }
        }
        self.order.sort_unstable();
        self.locals.sort_unstable();
        let order = std::mem::take(&mut self.order);
        let locals = std::mem::take(&mut self.locals);
        let (mut lp, mut rp) = (0, 0);
        while lp < locals.len() || rp < order.len() {
            let lv = locals.get(lp).map_or(u32::MAX, |l| l.0).min(order.get(rp).map_or(u32::MAX, |r| r.0));
            while lp < locals.len() && locals[lp].0 == lv { self.local(locals[lp].1 as usize, t); lp += 1; }
            while rp < order.len() && order[rp].0 == lv {
                let rank = order[rp].1;
                let first = rp;
                self.deck.clear();
                self.ends.clear();
                while rp < order.len() && order[rp].0 == lv && order[rp].1 == rank {
                    let Op::Run(_, s, l, _) = self.ops[order[rp].2 as usize] else { unreachable!() };
                    deal(&self.args[s as usize..(s + l) as usize], &self.vals, &self.at, &mut self.deck);
                    self.ends.push(self.deck.len() as u32);
                    rp += 1;
                }
                let Op::Run(mi, ..) = self.ops[order[first].2 as usize] else { unreachable!() };
                let m = self.names[mi as usize];
                let base = self.vals.len();
                go(m, Items { cards: &self.deck, ends: &self.ends }, &mut self.vals);
                let mut at = base;
                for &(_, _, i) in &order[first..rp] {
                    let Op::Run(_, _, _, n) = self.ops[i as usize] else { unreachable!() };
                    self.at[i as usize] = (at as u32, n);
                    at += n as usize;
                }
                assert_eq!(at, self.vals.len(), "{m} punched {} cards for {} items", self.vals.len() - base, rp - first);
            }
        }
        self.order = order;
        self.locals = locals;
        self.done = to;
    }

    /// The nodes the host works out itself: dealt cards, cards arranged, a table looked up, a flag's pick.
    fn local(&mut self, i: usize, t: &Tables) {
        let start = self.vals.len();
        let range = |at: &[(u32, u32)], n: N| { let (s, l) = at[n.0 as usize]; s as usize..(s + l) as usize };
        match self.ops[i] {
            Op::Cards(s, l) => self.vals.extend_from_slice(&self.lits[s as usize..(s + l) as usize]),
            Op::Cat(s, l) => {
                for k in s..s + l {
                    match self.args[k as usize] {
                        Arg::All(n) => { let r = range(&self.at, n); self.vals.extend_from_within(r) }
                        Arg::Some(n, f, l) => {
                            let r = range(&self.at, n);
                            let (f, l) = (f as usize, l as usize);
                            assert!(f + l <= r.len(), "range end index {} out of range for slice of length {}", f + l, r.len());
                            self.vals.extend_from_within(r.start + f..r.start + f + l)
                        }
                        Arg::Lit(c) => self.vals.push(c),
                    }
                }
            }
            Op::Sel(f, a, b) => {
                let flag = match f {
                    Arg::Lit(c) => c,
                    Arg::All(n) => self.vals[range(&self.at, n)][0],
                    Arg::Some(n, f, l) => self.vals[range(&self.at, n)][f as usize..f as usize + l as usize][0],
                };
                assert!(flag == 0 || flag == 1, "a flag of {flag}");
                let r = range(&self.at, if flag == 1 { a } else { b });
                self.vals.extend_from_within(r)
            }
            Op::Table(tab, s, l) => {
                self.deck.clear();
                deal(&self.args[s as usize..(s + l) as usize], &self.vals, &self.at, &mut self.deck);
                let ix = &self.deck;
                let u = |v: i32| v as usize;
                match tab {
                    Tab::Quarter => self.vals.extend_from_slice(&t.quarter[u(ix[0])][u(ix[1])]),
                    Tab::Trig => self.vals.extend_from_slice(&t.trig[u(ix[0])]),
                    Tab::Atan => self.vals.extend_from_slice(&t.atan[u(ix[0])][u(ix[1])]),
                    Tab::Rsq => self.vals.extend_from_slice(&t.rsq[u(ix[0])][u(ix[1])]),
                    Tab::Spec12 => self.vals.push(t.spec12[u(ix[0])]),
                    Tab::Spec30 => self.vals.push(t.spec30[u(ix[0])]),
                }
            }
            Op::Run(..) => unreachable!(),
        }
        self.at[i] = (start as u32, (self.vals.len() - start) as u32);
    }
}
