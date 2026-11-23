// The die and the Rubik's cube as data, shared by the host (fx/dierubik.rs, which works every number out on CARDIAC)
// and the reference (reference.rs, which works them out in ordinary Rust): the timeline (which shape each frame shows,
// where the cube is going, how far a turn has got), and the shapes as boxes whose corners are sums of the cube's axes
// at a few sizes. Nothing here is worked out: it says which numbers make which, as a card sorter's plugboard would.
#![allow(dead_code)]

/// Frames: coming toward us, its faces going to ivory as the pips fizz in from the approach's MORPH0th frame (a die
/// from MORPH0 + IVORY_IN on, still coming); the die held; the die splitting into the Rubik's cube (crossfading); the
/// solve (turns.txt); a pop; then roaming on. The die is whole for 3 s: its last 4 frames of the approach and
/// DIE_HOLD after.
pub const APPROACH: usize = 18;
pub const MORPH0: usize = 6;
pub const IVORY_IN: usize = 8;
pub const DIE_HOLD: usize = 32;
pub const SPLIT: usize = 6;
pub const POP: usize = 14;
pub const TAIL: usize = 80;

/// The sizes (sizes.txt, then the pips' own): their indexes.
pub const S100: usize = 0;
pub const S102: usize = 1;
pub const S098: usize = 2;
pub const S105: usize = 3;
pub const S055: usize = 4;
pub const S017: usize = 5;
pub const S068: usize = 6;
pub const S033: usize = 7;
pub const S035: usize = 8;
pub const S028: usize = 9;
pub const S034: usize = 10;
/// The pips' half width as they fizz in (S017 j / 4).
pub const SPIP: usize = 11;
pub const SIZES: usize = 12;

/// colours.txt: the plastic, the stickers by face (1..=6), the plain cube's faces (7..=12), ivory, the pips.
pub const PLASTIC: usize = 0;
pub const STICKERS: usize = 1;
pub const PLAIN: usize = 7;
pub const IVORY: usize = 13;
pub const PIP: usize = 14;

/// The axes after each axis, in turn: (u, w) for a face along a.
pub const AFTER: [(usize, usize); 3] = [(1, 2), (2, 0), (0, 1)];

/// The die: its faces' colour step (j of 8 from the plain cube's colour to ivory), its pips' step (j of 4; None: none).
#[derive(Clone, Copy, Debug)]
pub struct Die { pub colour: i32, pub pips: Option<i32> }

/// How big the Rubik's cube is: as it is; splitting from the die (step j of 7: 1.08 less 0.08 j / 7); in its pop
/// (step k: 1 + 0.06 sin(30 min(k, 6) degrees)).
#[derive(Clone, Copy, Debug)]
pub enum Scale { One, Split(i32), Pop(i32) }

/// The Rubik's cube: the moves of the solve done, the move turning (its index, step j of n), its size.
#[derive(Clone, Copy, Debug)]
pub struct Rubik { pub done: usize, pub turn: Option<(usize, i32, i32)>, pub scale: Scale }

/// Where the cube is: coming toward us (step k of 18), still at the near place, or roaming (frame k of the roam).
#[derive(Clone, Copy, Debug)]
pub enum Place { Approach(i32), Still, Roam(i32) }

/// A frame: its number, the 0.12 s steps the pop has added to the tumble's time, where the cube is, and what it is
/// (the die, the Rubik's cube, or the die crossfading into the Rubik's cube at step j of 7).
#[derive(Clone, Copy, Debug)]
pub enum Shape { Die(Die), Rubik(Rubik), Split(Die, Rubik, i32) }
#[derive(Clone, Copy, Debug)]
pub struct Frame { pub i: i32, pub extra: i32, pub place: Place, pub shape: Shape }

pub fn frames(turns: &[i32]) -> usize { APPROACH + DIE_HOLD + SPLIT + turns.iter().sum::<i32>() as usize + POP + TAIL }

pub fn frame(turns: &[i32], f: usize) -> Frame {
    let full = Die { colour: 8, pips: Some(4) };
    let at = |shape, place, extra| Frame { i: f as i32, extra, place, shape };
    let mut k = f;
    if k < APPROACH {
        let die = if k < MORPH0 { Die { colour: 0, pips: None } } else if k < MORPH0 + IVORY_IN {
            let j = k - MORPH0;
            Die { colour: j as i32 + 1, pips: (j >= 4).then_some(j as i32 - 3) }
        } else { full };
        return at(Shape::Die(die), Place::Approach(k as i32), 0)
    }
    k -= APPROACH;
    if k < DIE_HOLD { return at(Shape::Die(full), Place::Still, 0) }
    k -= DIE_HOLD;
    if k < SPLIT { return at(Shape::Split(full, Rubik { done: 0, turn: None, scale: Scale::Split(k as i32 + 1) }, k as i32 + 1), Place::Still, 0) }
    k -= SPLIT;
    for (m, &n) in turns.iter().enumerate() {
        if k < n as usize { return at(Shape::Rubik(Rubik { done: m, turn: Some((m, k as i32 + 1, n)), scale: Scale::One }), Place::Still, 0) }
        k -= n as usize;
    }
    if k < POP { return at(Shape::Rubik(Rubik { done: turns.len(), turn: None, scale: Scale::Pop(k as i32) }), Place::Still, k as i32) }
    k -= POP;
    at(Shape::Rubik(Rubik { done: turns.len(), turn: None, scale: Scale::One }), Place::Roam(k as i32), POP as i32)
}

/// A term of a sum: a sign and a size (its index).
pub type Term = (i32, usize);

/// What a box's faces are made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Material { Plastic, Sticker(usize), DieFace(usize), Pip }

/// A box in the cube's own axes: along each axis its low and high sides as sums of sizes; the group whose axes it
/// is in (1: the turning layer); its material. Only the part of a plate, sticker or pip outside what it sits on is
/// a box: the rest is hidden however the cube is turned, so its sides are drawn as much as can be seen.
#[derive(Clone, Debug)]
pub struct Bx { pub lo: [Vec<Term>; 3], pub hi: [Vec<Term>; 3], pub group: usize, pub material: Material }

impl Bx {
    /// The face along axis a on side s (-1, 1): its plane's terms, the sum s x (the place on the axis) of the face.
    pub fn plane(&self, a: usize, s: i32) -> Vec<Term> {
        let side = if s > 0 { &self.hi[a] } else { &self.lo[a] };
        side.iter().map(|&(sign, i)| (s * sign, i)).collect()
    }
    /// The face's corners, each per axis the terms of its place, anticlockwise as seen on the screen (y down).
    pub fn corners(&self, a: usize, s: i32) -> Vec<[Vec<Term>; 3]> {
        let (u, w) = AFTER[a];
        let mut order = vec![(-1, -1), (1, -1), (1, 1), (-1, 1)];
        if s > 0 { order.reverse(); }
        order.iter().map(|&(su, sw)| {
            let mut c: [Vec<Term>; 3] = Default::default();
            c[a] = if s > 0 { self.hi[a].clone() } else { self.lo[a].clone() };
            c[u] = if su > 0 { self.hi[u].clone() } else { self.lo[u].clone() };
            c[w] = if sw > 0 { self.hi[w].clone() } else { self.lo[w].clone() };
            c
        }).collect()
    }
}

/// A face: its axis and side; which face of the cube that is (2 axis + 1 if it faces up the axis).
pub fn face_of(a: usize, s: i32) -> usize { 2 * a + (s > 0) as usize }

fn sym(t: &[Term]) -> Vec<Term> { t.to_vec() }

/// The die: the body (its faces in order), then per face the plate and the pips on it. pips: the pips' places per
/// face (pips.txt), drawn when `with_pips`.
pub fn die_boxes(pips: &[Vec<i32>], with_pips: bool) -> (Bx, Vec<(usize, Vec<Bx>)>) {
    let body = Bx { lo: [0, 1, 2].map(|_| vec![(-1, S100)]), hi: [0, 1, 2].map(|_| vec![(1, S100)]), group: 0, material: Material::Plastic };
    let mut on = vec![];
    for f in 0..6 {
        let (a, s) = (f / 2, if f % 2 == 1 { 1 } else { -1 });
        let (u, w) = AFTER[a];
        let mut boxes = vec![];
        let mut plate = Bx { lo: Default::default(), hi: Default::default(), group: 0, material: Material::DieFace(f) };
        let (lo, hi) = if s > 0 { (vec![(1, S100)], vec![(1, S102)]) } else { (vec![(-1, S102)], vec![(-1, S100)]) };
        plate.lo[a] = lo;
        plate.hi[a] = hi;
        for b in [u, w] { plate.lo[b] = vec![(-1, S098)]; plate.hi[b] = vec![(1, S098)]; }
        boxes.push(plate);
        if with_pips {
            // a pip's place on the two axes other than the face's, in order of axis
            let others: Vec<usize> = (0..3).filter(|&b| b != a).collect();
            for p in pips[f].chunks(2) {
                let mut pip = Bx { lo: Default::default(), hi: Default::default(), group: 0, material: Material::Pip };
                let (lo, hi) = if s > 0 { (vec![(1, S102)], vec![(1, S105)]) } else { (vec![(-1, S105)], vec![(-1, S102)]) };
                pip.lo[a] = lo;
                pip.hi[a] = hi;
                for (k, &b) in others.iter().enumerate() {
                    let mut centre = vec![];
                    if p[k] != 0 { centre.push((p[k], S055)); }
                    pip.lo[b] = [sym(&centre), vec![(-1, SPIP)]].concat();
                    pip.hi[b] = [centre, vec![(1, SPIP)]].concat();
                }
                boxes.push(pip);
            }
        }
        on.push((f, boxes));
    }
    (body, on)
}

/// A cubie at p (each -1, 0 or 1) in group g: its body, and a sticker of each colour on its outer faces (by face).
pub fn cubie_boxes(p: [i32; 3], group: usize, sticker: &dyn Fn(usize) -> Option<usize>) -> (Bx, Vec<(usize, Vec<Bx>)>) {
    let centre = |b: usize| if p[b] != 0 { vec![(p[b], S068)] } else { vec![] };
    let mut body = Bx { lo: Default::default(), hi: Default::default(), group, material: Material::Plastic };
    for b in 0..3 { body.lo[b] = [centre(b), vec![(-1, S033)]].concat(); body.hi[b] = [centre(b), vec![(1, S033)]].concat(); }
    let mut on = vec![];
    for f in 0..6 {
        let (a, s) = (f / 2, if f % 2 == 1 { 1 } else { -1 });
        if p[a] != s { continue }
        let Some(colour) = sticker(f) else { continue };
        let (u, w) = AFTER[a];
        let mut st = Bx { lo: Default::default(), hi: Default::default(), group, material: Material::Sticker(colour) };
        let (lo, hi) = if s > 0 { ((1, S033), (1, S035)) } else { ((-1, S035), (-1, S033)) };
        st.lo[a] = [centre(a), vec![lo]].concat();
        st.hi[a] = [centre(a), vec![hi]].concat();
        for b in [u, w] { st.lo[b] = [centre(b), vec![(-1, S028)]].concat(); st.hi[b] = [centre(b), vec![(1, S028)]].concat(); }
        on.push((f, vec![st]));
    }
    (body, on)
}

/// An object's boxes in drawing order, given which of its body's faces show: what sits on hidden faces first (only
/// its sides could show, past the body's edge), then the body, then what sits on the faces that show.
pub fn in_order(body: Bx, on: Vec<(usize, Vec<Bx>)>, shown: &dyn Fn(usize) -> bool) -> Vec<Bx> {
    let mut out = vec![];
    for (f, boxes) in &on { if !shown(*f) { out.extend(boxes.iter().cloned()); } }
    out.push(body);
    for (f, boxes) in on { if shown(f) { out.extend(boxes); } }
    out
}

/// The cubies' places (cubies.txt), less the middle one.
pub fn cubies(list: &[i32]) -> Vec<[i32; 3]> {
    list.chunks(3).map(|c| [c[0], c[1], c[2]]).filter(|c| *c != [0, 0, 0]).collect()
}

/// A cubie's number for its pockets, 9 (x + 1) + 3 (y + 1) + z + 1, as the pocket machine numbers them (times 6,
/// plus the face).
pub fn pocket(p: [i32; 3], f: usize) -> usize { (6 * (9 * (p[0] + 1) + 3 * (p[1] + 1) + p[2] + 1)) as usize + f }
