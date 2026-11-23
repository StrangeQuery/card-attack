//! Mac: the cold open. A Mac Classic's 512x342 one-bit screen: Boom, a shooter in a window, runs down a corridor,
//! a door rises, into a room, three shots; it goes haywire (the hall of mirrors) and crashes (the bomb box); Resume, and
//! behind it THINK C's editor with its source at three times the size, scrolled two lines; the Alarm Clock flashes, the reminder (Demos tonight in Derek's room. Did you check out that new one?), and the game is
//! closed in a hurry, then the two windows behind it, Wikipedia's Gouraud shading page (its picture a Gouraud-shaded
//! sphere) and Learn to Speak Finnish (the flag); the floppy going in and mounting as Card Attack, its window opening, the application launched (the
//! zoom rects to the whole screen), the watch; then the one-bit tunnel at 128x86 blocks of 4x4, Atkinson-dithered, four
//! pictures a second, with STRANGE QUERY PRESENTS / A CARDIAC PRODUCTION / CARD ATTACK dissolving in (HyperCard's
//! dissolve). The desk's frames are described by segments/mac/desk.rs (its places a window's corner and a distance), the
//! game's by segments/mac/boom.rs (the prototype macdoom.py's script).
//!
//! On CARDIAC, each frame of the desk: the place machine (mac_place) adds up every place (a window's corner and the
//! distance from it, the middle of its title bar halved); the text machine (ui_text, the kit's) sets every line of
//! Chicago and Geneva from the atlas's Mac faces (the advances summed, centred or set right), and where the boxes
//! behind the names and the titles go; the ease machine (ui_ease) works out the zoom rects' edges; the glide machine
//! (mac_glide) how much of the pointer's way is left at its step (easing out) and the arc machine (mac_arc) where it
//! is (the bow); the XOR machine (mac_xor) turns over the pixels under each zoom rect's outline.
//!
//! The game, each frame: its numbers on the swarm's arithmetic machines (swarm_calc): the camera on its path (the
//! smoothsteps), the view's way (the sine table), every column's ray, where each ray first meets the lines x = a cell's
//! edge and y = one and how much further at each next, each row-pair's floor and ceiling (where the texels start and
//! how they step, the light); the cross machine (mac_cross) walks each ray across the lines, the across machine
//! (mac_across) turns each crossing into the cell entered and the texel crossed at; the host looks each cell up in the
//! map; the first machine (mac_first) finds each way's first wall and the door before it, the pick machine (mac_pick)
//! the nearer; the swarm's arithmetic each wall's scale, light, first row and texel; the wall machine (mac_wall) walks
//! each wall's column down the rows, its dots by the ordered dither; the floor machine (mac_fidx) the flats' texels, the
//! dots machine (mac_fdot) their dots. The gun's bob and the ammo's digits are the arithmetic's too.
//!
//! The crash (the prototype macboomcrash.py): from frame 32 the game goes haywire, its schedule data (data/haywire.txt);
//! where its bands show the walls behind the player, the game's view again from the same place half a turn round (the
//! arithmetic adds the half turn), only those columns' rays cast; the host moves the pixels (bands of the picture before
//! moved across, a column streaked, the view behind; strips of rows torn across; the junk counter's digits laid). The
//! bomb box and THINK C's editor are the desk's items like the others (the code's lines sprites at three times the
//! size, placed and cut off at their foot by the place machine); the pointer's straight moves are the ease machine's.
//!
//! Once: the sphere, each row's span, its first pixel's height (a whole square root) and light on the arithmetic; the
//! ball machine (mac_ball) keeps the root and the light across the row, the ball dots machine (mac_bdot) dithers it
//! (the ordered dither at twice the size). The flag: its size and the cross's bands on the arithmetic, its pixels on
//! the flag machine (mac_flag), the blue as a dense dither; the host frames it.
//!
//! The tunnel, set up once: the offset machine (mac_offset) gives each block's offset from the middle in half pixels,
//! the slope machine (mac_slope) its slope step (128 shorter / longer, a bit at a time), the angle machine (mac_angle)
//! its angle round the tunnel and the reach machine (mac_reach) its distance, from the slope table. Each picture: the
//! turn machine (mac_turn) its turn and travel from the sine table, the twist machine (mac_twist) how far each depth
//! twists the wall; the tunnel row machine (mac_trow) each block's shade (the checks around and along); the dither
//! machine (mac_dither), row after row, Atkinson's dither, carrying the eighths down. The titles: the ease machine
//! works out how far the dissolve has come (85 a third), the dissolve machine (mac_dissolve) which of the card's
//! pixels show.
//!
//! The host deals and files: it fetches the glyphs' bitmaps and the sprites (data/sprites.txt: the icons, the
//! pointers, the windows' boxes and arrows, the OK button, the gun, the faces, the status bar, the big letters) and lays
//! their pixels where the machines placed them; fills the rectangles from the places punched; fetches the map's cells,
//! the textures' greys and the dither's thresholds (data/boom.txt) by the cells, texels and lights the machines
//! punched, the slope and ring tables' cards by the steps and distances, and the dissolve's turns; copies each block's
//! dot four by four. It files video.rgb (377 frames, 512x342, black and white) and video_hd.rgb (the same at 2x,
//! 1024x684 at 256, 42 of 1536x768, for the cut: the Mac's bezel round it is the turn's).
use crate::*;
use crate::fx::text::{Line, Text};
use super::swarm_calc::{Arg, Calc, Items, N, Tables as CalcTables};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

#[allow(dead_code)]
#[path = "../../../segments/mac/desk.rs"]
pub mod desk;
#[allow(dead_code)]
#[path = "../../../segments/mac/boom.rs"]
pub mod boom;
use desk::*;
use desk::{W, H};

pub const FRAMES: usize = 377;
/// The ordered dither's rows from a phase on: row (phase + j) mod 4 at [phase][j].
const TURNED: [[usize; 4]; 4] = [[0, 1, 2, 3], [1, 2, 3, 0], [2, 3, 0, 1], [3, 0, 1, 2]];
/// Where the screen goes on the 1536x768 frame, at 2x.
pub const HD_X: usize = 256;
pub const HD_Y: usize = 42;

/// The machines: the cold open's, the swarm's arithmetic's (for the game's numbers).
const MACHINES: &[&str] = &["mac_place", "mac_glide", "mac_arc", "mac_xor", "mac_turn", "mac_twist", "mac_trow", "mac_dither", "mac_dissolve", "mac_fill", "mac_thirds",
    "mac_cross", "mac_across", "mac_first", "mac_pick", "mac_wall", "mac_fidx", "mac_fdot", "mac_ball", "mac_bdot", "mac_flag",
    "sw_qis", "sw_qp", "sw_lsum", "sw_term", "sw_mag", "sw_shift", "sw_divd", "sw_join", "sw_floor3", "sw_sign", "sw_ang", "sw_sin", "sw_short", "sw_hund", "sw_isqrt", "sw_isqrt2"];

struct Mac { text: Text, m: BTreeMap<&'static str, Machine>, ease: Machine, sprites: HashMap<String, Sprite>, tables: Tables, cards: Vec<Vec<u8>>, order: Vec<u8>,
    /// Each block of the tunnel: its angle round, and the ring table's cards at its distance (along, twist, light, dark).
    blocks: Vec<[i32; 5]>, calc: CalcTables, data: boom::Data, haywire: boom::Haywire }

/// A rectangle's or a point's numbers, as the place machine punched them, taken in order.
struct Placed { v: Vec<i32>, at: usize }
impl Placed { fn next(&mut self) -> i32 { self.at += 1; self.v[self.at - 1] } }

fn rect_cards(rect: Rect) -> [i32; 4] { rect.map(|v| v as i32) }

/// A number as the arithmetic's cards say (one, a long, or three).
fn value(c: &Calc, n: N) -> i64 { c.cards(n).iter().fold(0i64, |a, &v| a * 1000 + v as i64) }
/// The same, as a card for a machine's deck.
fn card(c: &Calc, n: N) -> i32 { let v = value(c, n); assert!(v.abs() <= 999, "a card of {v}"); v as i32 }
fn flag(c: &Calc, a: Arg) -> bool {
    match a { Arg::Some(n, f, 1) => { let v = c.cards(n)[f as usize]; assert!(v == 0 || v == 1); v == 1 } _ => panic!("not a flag") }
}

/// The smoothstep of x (thousandths, held to 0..1000), in thousandths: fl(fl(x^2, 10) (3000 - 2 x), 100000).
fn smooth(c: &mut Calc, x: N) -> N {
    let x = c.clamp(x, 0, 1000);
    let xx = c.mul(x, x);
    let q = c.divk(xx, 10);
    let q = c.within(q, 100_000);
    let k = c.num(3000);
    let two = c.num(2);
    let w = c.lin(&[(k, None, false), (x, Some(two), true)]);
    let w = c.within(w, 3000);
    let p = c.mul(q, w);
    let p = c.shift(p);
    let s = c.divk(p, 100);
    c.within(s, 1000)
}
/// a + fl((b - a) u, 1000).
fn lerp(c: &mut Calc, a: i64, b: i64, u: N) -> N {
    let (a, b) = (c.num(a), c.num(b));
    let d = c.sub(b, a);
    let p = c.mul(d, u);
    let q = c.divk(p, 1000);
    c.add(a, q)
}
/// fl(n k, d) for a number n and the data's k and d.
fn scale(c: &mut Calc, n: N, k: i64, d: i64) -> N { let p = c.mulk(n, k); c.divk(p, d) }


impl Mac {
    fn new(demo: &Path, host: &mut Host) -> Mac {
        let mut m = BTreeMap::new();
        for &n in MACHINES { m.insert(n, rust(demo, n)); }
        let mut mac = Mac { text: Text::new(demo), ease: rust(demo, "ui_ease"), sprites: sprites(demo), tables: tables(demo), cards: titles(demo), order: dissolve(demo),
            blocks: vec![], calc: super::swarm_calc::load_tables(&read(demo, "swarm/data/tables.txt")), data: boom::data(demo), haywire: boom::haywire(demo), m };
        // the tunnel's blocks, row by row
        let (offset, slope, angle, reach) = (rust(demo, "mac_offset"), rust(demo, "mac_slope"), rust(demo, "mac_angle"), rust(demo, "mac_reach"));
        for iy in 0..TH as i32 {
            let off = host.go1(&offset, &[&[iy]], 3 * TW);
            let pairs: Vec<i32> = (0..TW).flat_map(|x| [off[3 * x], off[3 * x + 1]]).collect();
            let steps = host.go1(&slope, &[&pairs], TW);
            let entry = |x: usize| mac.tables.slope[steps[x] as usize].map(|v| v as i32);
            let deck: Vec<i32> = (0..TW).flat_map(|x| [entry(x)[0], off[3 * x + 2]]).collect();
            let around = host.go1(&angle, &[&deck], TW);
            let deck: Vec<i32> = (0..TW).flat_map(|x| [entry(x)[1], off[3 * x]]).collect();
            let far = host.go1(&reach, &[&deck], TW);
            for x in 0..TW {
                let [along, twist, light, dark] = mac.tables.ring[far[x] as usize].map(|v| v as i32);
                mac.blocks.push([around[x], along, twist, light, dark]);
            }
        }
        mac
    }

    /// Run a machine over items: its deck is the count, then the items, at most 999 a run.
    fn go(&self, host: &mut Host, name: &str, items: &[Vec<i32>]) -> Vec<i32> {
        let mut out = vec![];
        for chunk in items.chunks(999) {
            let mut deck = vec![chunk.len() as i32];
            for it in chunk { deck.extend(it); }
            if let Some(i) = deck.iter().position(|v| v.abs() > 999) { panic!("{name}: card {i} of {} is {}", deck.len(), deck[i]); }
            out.extend(host.gon(&self.m[name], &[&deck]));
        }
        out
    }

    fn eval(&self, host: &mut Host, c: &mut Calc) {
        let mut go = |m: &'static str, items: Items, out: &mut Vec<i32>| {
            for (k, cards) in items.chunks(999) {
                let count = [k as i32];
                if let Some(i) = cards.iter().position(|v| v.abs() > 999) { panic!("{m}: card {} of {} is {}", i + 1, cards.len() + 1, cards[i]); }
                host.gon_into(&self.m[m], &[&count, cards], out);
            }
        };
        c.eval(&mut go, &self.calc);
    }

    /// Atkinson's dither of a picture's rows (each value 256 white), row after row on the dither machine: each block black
    /// (true) or white.
    fn dither(&self, host: &mut Host, rows: &[Vec<i32>]) -> Vec<Vec<bool>> {
        let w = rows[0].len();
        let (mut passed, mut under) = (vec![0; w], vec![0; w]);
        let mut black = vec![];
        for row in rows {
            let deck: Vec<i32> = (0..w).flat_map(|x| [row[x], passed[x], under[x]]).collect();
            let out = host.go1(&self.m["mac_dither"], &[&[w as i32], &deck], 3 * w);
            // each block: black, its eighth; from the second, what goes to the block under the one before; at the end, under the last
            let mut at = 0;
            let mut b = vec![false; w];
            for x in 0..w {
                b[x] = out[at] == 1;
                under[x] = out[at + 1];
                at += 2;
                if x > 0 { passed[x - 1] = out[at]; at += 1; }
            }
            passed[w - 1] = out[at];
            black.push(b);
        }
        black
    }

    /// The tunnel's picture j: each block black (true) or white.
    fn tunnel(&self, host: &mut Host, j: i16) -> Vec<bool> {
        let sine: Vec<i32> = self.tables.sine.iter().map(|&v| v as i32).collect();
        let tw = host.go1(&self.m["mac_turn"], &[&[j as i32], &sine], 3);
        let (turn, weight, travel) = (tw[0], tw[1], tw[2]);
        let twisted = host.go1(&self.m["mac_twist"], &[&[weight]], 61);
        let mut shades = vec![];
        for y in 0..TH {
            let row = &self.blocks[y * TW..(y + 1) * TW];
            let deck: Vec<i32> = row.iter().flat_map(|&[around, along, twist, light, dark]| [around, twisted[twist as usize], along, light, dark]).collect();
            shades.push(host.go1(&self.m["mac_trow"], &[&[turn, travel], &deck], TW));
        }
        self.dither(host, &shades).concat()
    }

    fn sprite(&self, screen: &mut [bool], w: usize, h: usize, name: &str, x: i32, y: i32) {
        let s = &self.sprites[name];
        for j in 0..s.h { for i in 0..s.w {
            let (px, py) = (x + s.ox as i32 + i as i32, y + s.oy as i32 + j as i32);
            if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 { continue }
            match s.px[j * s.w + i] { 1 => screen[py as usize * w + px as usize] = false, 2 => screen[py as usize * w + px as usize] = true, _ => {} }
        } }
    }

    // ---- the game ---------------------------------------------------------------------------------------------------

    /// The game's frame f: the view (448 x 224) with the gun over it, the status bar under it, black true. `behind`:
    /// instead, the view from the same place half a turn round (the hall of mirrors' walls behind the player), never in
    /// a muzzle flash, only its walls in the columns marked (the rest of it is never shown), nothing over it.
    fn game(&self, host: &mut Host, f: usize, behind: Option<&[bool]>) -> Vec<bool> {
        use boom::*;
        let (kick, flash) = kick_flash(f);
        let bright = flash == 2 && behind.is_none();
        let xs: Vec<usize> = (0..VW).filter(|&x| behind.is_none_or(|m| m[x])).collect();
        let mut c = Calc::new();
        let fi = c.num(f as i64);
        // the camera on its path
        let (x, y, angle) = match leg(f) {
            Leg::Run => {
                let t = scale(&mut c, fi, 500, 15);
                let u = c.addk(t, 500);
                let s = smooth(&mut c, u);
                let w = c.mulk(s, 2);
                let w = c.addk(w, -1000);
                let w = c.within(w, 1000);
                let x = lerp(&mut c, X0, XD, w);
                (x, c.num(Y0), c.num(0))
            }
            Leg::Wait => (c.num(XD), c.num(Y0), c.num(0)),
            Leg::Through => {
                let t = c.addk(fi, -21);
                let u = scale(&mut c, t, 1000, 15);
                let u = c.within(u, 1000);
                let h = c.divk(u, 2);
                let s = smooth(&mut c, h);
                let s2 = c.mulk(s, 2);
                let x = lerp(&mut c, XD, XR, s2);
                (x, c.num(Y0), lerp(&mut c, 0, A_ROOM, u))
            }
            Leg::Turn => {
                let t = c.addk(fi, -36);
                let u = scale(&mut c, t, 1000, 24);
                let u = smooth(&mut c, u);
                (lerp(&mut c, XR, XE, u), lerp(&mut c, Y0, YE, u), lerp(&mut c, A_ROOM, A_END, u))
            }
        };
        let (x, y) = (c.within(x, 999), c.within(y, 999));
        let angle = if behind.is_some() { c.addk(angle, 500_000) } else { angle };
        let t = c.addk(fi, -DOOR_AT);
        let t = scale(&mut c, t, 1000, DOOR_FRAMES);
        let door = smooth(&mut c, t);
        // the view's way: 448 times its cosine and sine
        let wide = c.wide(angle);
        let (sn, cs) = c.sin(wide);
        let ux = scale(&mut c, cs, 448, 999);
        let uy = scale(&mut c, sn, 448, 999);
        let (ux, uy) = (c.within(ux, 448), c.within(uy, 448));
        let (mx, fx, my, fy) = (c.divk(x, 64), c.modk(x, 64), c.divk(y, 64), c.modk(y, 64));
        let (mx, my) = (c.within(mx, 23), c.within(my, 16));
        // every column's ray, and where it meets the lines each way
        struct Col { rdx: N, rdy: N, ax: N, ay: N, xneg: Arg, yneg: Arg, ypos: Arg, ways: [Option<[N; 8]>; 2] }
        let mut cols = vec![];
        let one = c.num(1);
        for &col in &xs {
            let n = c.num(col as i64);
            let k = c.mulk(n, 2);
            let k = c.addk(k, -447);
            let k = c.within(k, 447);
            let t = c.lin(&[(uy, Some(k), true)]);
            let t = c.divk(t, 560);
            let rdx = c.add(ux, t);
            let t = c.mul(ux, k);
            let t = c.divk(t, 560);
            let rdy = c.add(uy, t);
            let (rdx, rdy) = (c.within(rdx, 806), c.within(rdy, 806));
            let (ax, ay) = (c.abs(rdx), c.abs(rdy));
            let (ax, ay) = (c.within(ax, 806), c.within(ay, 806));
            let (xneg, yneg) = (c.below0(rdx), c.below0(rdy));
            let zero = c.num(0);
            let ypos = c.lt(zero, rdy);
            let mut ways = [None, None];
            for (w, (a, b, neg, cell, tex)) in [(ax, ay, xneg, mx, fx), (ay, ax, yneg, my, fy)].into_iter().enumerate() {
                // the first line: tex or 64 less it on; the cell entered there; how far across at it and at each next
                let back = c.sub(cell, one);
                let on = c.add(cell, one);
                let cell0 = c.sel(neg, back, on);
                let sixty4 = c.num(64);
                let rest = c.sub(sixty4, tex);
                let first = c.sel(neg, tex, rest);
                let first = c.within(first, 64);
                let den = c.max(a, one);
                let p = c.mul(first, b);
                let (q0, r0) = c.divrem(p, den, 0);
                let q0 = c.within(q0, 99_999);
                let k999 = c.num(999);
                let q0 = c.min(q0, k999);
                let p = c.mulk(b, 64);
                let (q, r) = c.divrem(p, den, 0);
                let q = c.within(q, 99_999);
                let q = c.min(q, k999);
                let _ = w;
                ways[w] = Some([first, cell0, q0, r0, den, q, r, a]);
            }
            cols.push(Col { rdx, rdy, ax, ay, xneg, yneg, ypos, ways });
        }
        // the flats, per row-pair m = 2 j + 1 (rows 112 + j and 111 - j)
        let nx = { let (a, b) = (c.num(560), c.num(447)); c.lin(&[(a, Some(ux), false), (b, Some(uy), false)]) };
        let ny = { let (a, b) = (c.num(560), c.num(447)); c.lin(&[(a, Some(uy), false), (b, Some(ux), true)]) };
        let mut flats = vec![];
        for j in 0..VH / 2 {
            let n = c.num(j as i64);
            let m = c.mulk(n, 2);
            let m = c.addk(m, 1);
            let m = c.within(m, 223);
            let m14 = c.mulk(m, 14);
            let m7 = c.mulk(m, 7);
            let mut parts = vec![];
            for (nn, t, u) in [(nx, fx, uy), (ny, fy, ux)] {
                // the start: n / 14m on from the camera's texel, wrapped at 64 texels, in thousandths
                let (q, r) = c.divrem(nn, m14, 0);
                let q64 = c.modk(q, 64);
                let r = c.long3(r);
                let r1000 = c.mulk(r, 1000);
                let frac = c.div(r1000, m14);
                let (k1, k2) = (c.num(1000), c.num(1000));
                let s = c.lin(&[(k1, Some(t), false), (k2, Some(q64), false), (frac, None, false)]);
                let start = c.modk(s, 64_000);
                // the step a column: -1000 uy / 7m across, 1000 ux / 7m down, wrapped
                let k = c.num(if u == uy { -1000 } else { 1000 });
                let p = c.mul(k, u);
                let step = c.div(p, m7);
                let step = c.modk(step, 64_000);
                parts.extend([c.divk(start, 1000), c.modk(start, 1000), c.divk(step, 1000), c.modk(step, 1000)]);
            }
            // the light: 155 - fl(2545, m), held to 10..100; brighter in a muzzle flash
            let k = c.num(2545);
            let q = c.div(k, m);
            let l = c.neg(q);
            let l = c.addk(l, 155);
            let mut l = c.clamp(l, 10, 100);
            if bright { l = scale(&mut c, l, 27, 20); }
            flats.push((parts, l));
        }
        // the gun's bob and the ammo's digits
        let ph = c.mulk(fi, PHASE);
        let ph = c.wide(ph);
        let (s, cs) = c.sin(ph);
        let b = c.num(bob(f));
        let b12 = c.mulk(b, 12);
        let b7 = c.mulk(b, 7);
        let p = c.mul(b12, s);
        let across = c.divk(p, 9990);
        let acs = c.abs(cs);
        let p = c.mul(b7, acs);
        let down = c.divk(p, 9990);
        let gx = c.addk(across, GUN_X);
        let kickn = c.num(kick);
        let gyk = c.num(GUN_Y);
        let gy = c.lin(&[(gyk, None, false), (down, None, false), (kickn, None, true)]);
        let fifty = c.num(50);
        let mut terms = vec![(fifty, None, false)];
        for _ in 0..fired(f) { terms.push((one, None, true)); }
        let ammo = c.lin(&terms);
        let (tens, ones) = (c.divk(ammo, 10), c.modk(ammo, 10));
        self.eval(host, &mut c);

        // the door: 0 shut, 1 part way up, 2 all the way up
        let dv = value(&c, door);
        let door_mode = if dv == 0 { 0 } else if dv == 1000 { 2 } else { 1 };
        // each ray across the lines, the crossings' cells and texels, the map's cells looked up
        let mut lists = vec![];
        let mut cross = vec![];
        for col in &cols {
            for w in 0..2 {
                let [first, cell0, q0, r0, den, q, r, a] = col.ways[w].unwrap();
                let (neg, way) = if w == 0 { (col.xneg, 1) } else { (col.yneg, 1) };
                let way = if flag(&c, neg) { -way } else { way };
                let cells = if w == 0 { MAP_W } else { MAP_H } as i32;
                let has = value(&c, a) > 0;
                let first_v = card(&c, first);
                // a ray along the lines never meets them: past every wall from the start
                let deck = if has { [first_v, card(&c, cell0), way, cells, card(&c, q0), card(&c, r0), card(&c, den), card(&c, q), card(&c, r)] }
                    else { [999, 0, 1, 1, 999, 0, 1, 0, 0] };
                cross.push(deck.to_vec());
                lists.push((w, first_v));
            }
        }
        let crossed = self.go(host, "mac_cross", &cross);
        let mut deck = vec![];
        let mut at = 0;
        for (i, col) in cols.iter().enumerate() {
            for w in 0..2 {
                let _ = i;
                // the other way: the camera's cell and texel, 1 going back, the map's size
                let (pc, pt, back, cells) = if w == 0 { (my, fy, col.yneg, MAP_H) } else { (mx, fx, col.xneg, MAP_W) };
                let mut item = vec![card(&c, pc), card(&c, pt), flag(&c, back) as i32, cells as i32];
                loop {
                    let v = crossed[at];
                    at += 1;
                    item.push(v);
                    if v < 0 { break; }
                    item.extend(&crossed[at..at + 2]);
                    at += 2;
                }
                deck.push(item);
            }
        }
        let across = self.go(host, "mac_across", &deck);
        let mut deck = vec![];
        let mut at = 0;
        for &(w, first) in &lists {
            let mut items = vec![];
            loop {
                let cell = across[at];
                if cell < 0 { at += 1; break; }
                let (other, tex, gone) = (across[at + 1], across[at + 2], across[at + 3]);
                at += 4;
                let (col, row) = if w == 0 { (cell, other) } else { (other, cell) };
                items.extend([self.data.map[row as usize][col as usize] as i32, gone, tex]);
            }
            let mut d = vec![door_mode, first, items.len() as i32 / 3];
            d.extend(items);
            deck.push(d);
        }
        let firsts = self.go(host, "mac_first", &deck);
        // each list's door (across, along, texel) and wall (texture, across, along, texel)
        let mut found = vec![];
        let mut at = 0;
        for _ in &lists {
            let mut gate = None;
            let mut wall = None;
            loop {
                match firsts[at] {
                    0 => { at += 1; break; }
                    1 => { gate = Some([firsts[at + 1], firsts[at + 2], firsts[at + 3]]); at += 4; }
                    _ => { wall = Some([firsts[at + 1], firsts[at + 2], firsts[at + 3], firsts[at + 4]]); at += 5; break; }
                }
            }
            found.push((gate, wall));
        }
        let deck: Vec<Vec<i32>> = (0..xs.len()).map(|x| {
            let ((xg, xw), (yg, yw)) = (found[2 * x], found[2 * x + 1]);
            let (xw, yw) = (xw.unwrap_or([0; 4]), yw.unwrap_or([0; 4]));
            let (xg, yg) = (xg.map(|g| [1, g[0], g[1]]).unwrap_or([0; 3]), yg.map(|g| [1, g[0], g[1]]).unwrap_or([0; 3]));
            vec![xw[0], xw[1], yw[0], yw[2], xg[0], xg[1], yg[0], yg[2]]
        }).collect();
        let picked = self.go(host, "mac_pick", &deck);

        // each wall's (and door's) column: its scale, light, first row, texel, on the arithmetic
        struct Draw { x: usize, kind: usize, u: N, top: N, phase: N, texel: N, part: N, step: N, step_part: N, light: N }
        let mut draws = vec![];
        let mut c2 = Calc::new();
        let zero = c2.num(0);
        let one = c2.num(1);
        for (i, &x) in xs.iter().enumerate() {
            let (side, gate) = (picked[2 * i] as usize, picked[2 * i + 1]);
            let wall = found[2 * i + side].1.unwrap();
            let mut hits = vec![(side, wall[0] as usize, wall[2], wall[3], 0i64)];
            if gate > 0 {
                let s = gate as usize - 1;
                let g = found[2 * i + s].0.unwrap();
                hits.push((s, DOOR as usize, g[1], g[2], dv));
            }
            let col = &cols[i];
            for (side, kind, along, tex, lift) in hits {
                // the texture's column: 63 less the texel seen from behind
                let behind = if side == 0 { flag(&c, col.xneg) } else { flag(&c, col.ypos) };
                let t = c2.num(tex as i64);
                let u = if behind { let k = c2.num(63); c2.sub(k, t) } else { t };
                let d = c2.num(value(&c, if side == 0 { col.ax } else { col.ay }));
                let a = c2.num(along as i64);
                let p = c2.mulk(a, 400);
                let s = c2.div(p, d);
                let s = c2.within(s, 999);
                // the light: 155 - fl(7 s, 44) held to 10..100, four fifths on a y line's wall, 27/20 in a flash
                let l = scale(&mut c2, s, 7, 44);
                let l = c2.neg(l);
                let l = c2.addk(l, 155);
                let mut l = c2.clamp(l, 10, 100);
                if side == 1 { l = scale(&mut c2, l, 4, 5); }
                if bright { l = scale(&mut c2, l, 27, 20); }
                // the first row: up from the middle while 16000 + (2y - 223) s is not below 0
                let k = c2.num(16_000);
                let k223 = c2.num(223);
                let n = c2.lin(&[(k, None, false), (k223, Some(s), true)]);
                let s2 = c2.mulk(s, 2);
                let s2 = c2.within(s2, 1998);
                let s2d = c2.max(s2, one);
                let q = c2.div(n, s2d);
                let top = c2.neg(q);
                let top = c2.max(top, zero);
                let top = c2.within(top, 223);
                let phase = c2.modk(top, 4);
                let t2 = c2.mulk(top, 2);
                let t2 = c2.addk(t2, -223);
                let k = c2.num(16_000);
                let k32 = c2.num(32);
                let lift = c2.num(lift);
                let wv = c2.lin(&[(k, None, false), (t2, Some(s), false), (k32, Some(lift), false)]);
                let (texel, part) = (c2.divk(wv, 500), c2.modk(wv, 500));
                let (step, step_part) = (c2.divk(s2, 500), c2.modk(s2, 500));
                draws.push(Draw { x, kind, u, top, phase, texel, part, step, step_part, light: l });
            }
        }
        self.eval(host, &mut c2);
        let deck: Vec<Vec<i32>> = draws.iter().map(|d| {
            let (top, u, e) = (card(&c2, d.top), card(&c2, d.u) as usize, card(&c2, d.light) as usize);
            assert!(card(&c2, d.step) >= 0);
            let ph = card(&c2, d.phase) as usize;
            let mut v = vec![top, card(&c2, d.texel), card(&c2, d.part), card(&c2, d.step), card(&c2, d.step_part)];
            for j in 0..4 { v.push(self.data.thresh[e][BAYER[TURNED[ph][j]][d.x % 4]] as i32); }
            let tex = &self.data.tex[d.kind];
            for r in 0..64 { v.push(tex[r * 64 + u] as i32); }
            v
        }).collect();
        let dots = self.go(host, "mac_wall", &deck);

        // the flats: each row-pair's texels, then every row's dots
        let deck: Vec<Vec<i32>> = flats.iter().map(|(p, _)| p.iter().map(|&n| card(&c, n)).collect()).collect();
        let texels = self.go(host, "mac_fidx", &deck);
        let mut deck = vec![];
        for yy in 0..VH {
            let (j, ceiling) = if yy >= VH / 2 { (yy - VH / 2, false) } else { (VH / 2 - 1 - yy, true) };
            let e = card(&c, flats[j].1) as usize;
            let tex = &self.data.tex[if ceiling { 7 } else { 6 }];
            let mut v: Vec<i32> = (0..4).map(|x| self.data.thresh[e][BAYER[yy % 4][x]] as i32).collect();
            let row = &texels[j * 2 * VW..(j + 1) * 2 * VW];
            for xx in 0..VW { v.push(tex[row[2 * xx + 1] as usize * 64 + row[2 * xx] as usize] as i32); }
            deck.push(v);
        }
        let flat_dots = self.go(host, "mac_fdot", &deck);

        // the picture: the flats, the walls over them, the door over its wall, the gun, the status bar
        let (pw, ph) = (VW, VH + SBH);
        let mut px: Vec<bool> = vec![false; pw * ph];
        for i in 0..VW * VH { px[i] = flat_dots[i] == 1; }
        let mut at = 0;
        for d in &draws {
            let mut yy = card(&c2, d.top) as usize;
            while dots[at] >= 0 {
                px[yy * VW + d.x] = dots[at] == 1;
                yy += 1;
                at += 1;
            }
            at += 1;
        }
        let _ = (lists.len(), cols.iter().map(|c| (c.rdx, c.rdy)).count());
        if behind.is_some() { return px }
        self.sprite(&mut px, pw, VH, &format!("gun_{kick}_{flash}"), card(&c, gx), card(&c, gy));
        let digits = [(card(&c, tens), AMMO_AT[0].0 as i32), (card(&c, ones), AMMO_AT[1].0 as i32)];
        self.status(&mut px, &digits, &face(f));
        px
    }

    /// The status bar under a game picture (VW wide): the bar, the ammo's digits (each a digit and its place across),
    /// the fixed counters, the face.
    fn status(&self, px: &mut [bool], digits: &[(i32, i32)], looks: &str) {
        use boom::*;
        let (pw, ph, bar) = (VW, VH + SBH, VH as i32);
        self.sprite(px, pw, ph, "status_base", 0, bar);
        for &(d, x) in digits { self.sprite(px, pw, ph, &format!("big_{d}"), x, bar + AMMO_AT[0].1 as i32); }
        for (name, x, y) in FIXED_DIGITS { self.sprite(px, pw, ph, name, x as i32, bar + y as i32); }
        self.sprite(px, pw, ph, looks, FACE_AT.0 as i32, bar + FACE_AT.1 as i32);
    }

    /// The game's picture as frame f shows it (its view and status bar), worked out the first time: its own to
    /// HAYWIRE, then gone haywire, each from the one before (worked out first); from BOMB the last held.
    fn shown(&self, host: &mut Host, f: usize, worked: &mut HashMap<Pic, (usize, Vec<bool>)>) {
        use boom::*;
        if worked.contains_key(&Pic::Game(f)) { return }
        let fr = self.game(host, f, None);
        let px = if f < HAYWIRE || self.haywire.frames[f - HAYWIRE].clean { fr } else {
            self.shown(host, f - 1, worked);
            let spoil = &self.haywire.frames[f - HAYWIRE];
            // the columns that show the walls behind: the bands spoiled that way
            let mut cols = vec![false; VW];
            for (j, &(x, w)) in self.haywire.bands[..spoil.bands].iter().enumerate() {
                if band(j) == Band::Behind { for c in x..(x + w).min(VW) { cols[c] = true; } }
            }
            let behind = if cols.contains(&true) { self.game(host, f, Some(&cols)) } else { vec![] };
            let last = &worked[&Pic::Game(f - 1)].1;
            // the bands, in order: the picture before moved across, a column streaked, the walls behind
            let mut out = fr;
            for (j, &(x, w)) in self.haywire.bands[..spoil.bands].iter().enumerate() {
                for y in 0..VH {
                    let row = y * VW;
                    for c in x..(x + w).min(VW) {
                        out[row + c] = match band(j) {
                            Band::Smear => last[row + (c + VW - spoil.shift) % VW],
                            Band::Streak => out[row + x],
                            Band::Behind => behind[row + c],
                        };
                    }
                }
            }
            // the torn strips, moved across, wrapping
            for &(y0, dx) in &self.haywire.tears[..spoil.tears] {
                for y in y0..y0 + TEAR {
                    let row: Vec<bool> = out[y * VW..(y + 1) * VW].to_vec();
                    for c in 0..VW { out[y * VW + c] = row[(c + VW - dx.rem_euclid(VW as i64) as usize) % VW]; }
                }
            }
            // the ammo counter's junk
            if let Some(junk) = &spoil.junk {
                let at = if junk.len() == 3 { JUNK_AT[0] } else { JUNK_AT[1] };
                let digits: Vec<(i32, i32)> = junk.iter().zip(at).map(|(&d, x)| (d as i32, x as i32)).collect();
                self.status(&mut out, &digits, JUNK_FACE);
            }
            out
        };
        worked.insert(Pic::Game(f), (VW, px));
    }

    /// The Wikipedia page's sphere ((2R + 1) square): its dots, black true.
    fn sphere(&self, host: &mut Host) -> Vec<bool> {
        use boom::*;
        let r = BALL_R;
        let n = (2 * r + 1) as usize;
        let mut c = Calc::new();
        // the dither's thresholds: ceil((70000 (2k + 1) - 179200) / 288) for each step k, never where not above 0
        let (unit, less, over) = BALL_DITHER;
        let zero = c.num(0);
        let steps: Vec<(N, N, Arg)> = (0..16).map(|k| {
            let kn = { let k = c.num(k); let k2 = c.mulk(k, 2); c.addk(k2, 1) };
            let u = c.num(unit);
            let l = c.num(less);
            let m = c.lin(&[(l, None, false), (u, Some(kn), true)]);
            let q = c.divk(m, over);
            let t = c.neg(q);
            let above = c.lt(zero, t);
            (c.divk(t, 100), c.modk(t, 100), above)
        }).collect();
        // each row: the span |x| <= sqrt(R^2 - y^2), the first pixel's root, what its square is over the root's, its light
        let rr = c.num(r * r);
        let mut rows = vec![];
        for y in -r..=r {
            let yn = c.num(y);
            let d = c.lin(&[(rr, None, false), (yn, Some(yn), true)]);
            let half = c.sqrt(d);
            let half = c.within(half, r);
            let d0 = c.lin(&[(d, None, false), (half, Some(half), true)]);
            let root = c.sqrt(d0);
            let root = c.within(root, r);
            let over = c.lin(&[(d0, None, false), (root, Some(root), true)]);
            let (bx, by, bz) = (c.num(-BALL_X), c.num(BALL_Y), c.num(BALL_Z));
            let light = c.lin(&[(bx, Some(half), false), (by, Some(yn), false), (bz, Some(root), false)]);
            let n = c.mulk(half, 2);
            let n = c.addk(n, 1);
            let first = c.neg(half);
            // the first pixel's threshold: which of the four, and whether its second pixel; the row's step of the dither
            let i0 = c.addk(first, r);
            let column = { let h = c.divk(i0, 2); c.modk(h, 4) };
            let second = c.modk(i0, 2);
            let yr = c.addk(yn, r);
            let brow = { let h = c.divk(yr, 2); c.modk(h, 4) };
            rows.push((n, first, root, over, c.divk(light, 100), c.modk(light, 100), column, second, brow));
        }
        self.eval(host, &mut c);
        let deck: Vec<Vec<i32>> = rows.iter().map(|&(n, first, root, over, h, l, ..)| [n, first, root, over, h, l].iter().map(|&v| card(&c, v)).collect()).collect();
        let lights = self.go(host, "mac_ball", &deck);
        let mut at = 0;
        let mut deck = vec![];
        for &(n, _, _, _, _, _, column, second, brow) in &rows {
            let (n, column, brow) = (card(&c, n) as usize, card(&c, column) as usize, card(&c, brow) as usize);
            let mut v = vec![card(&c, second)];
            let ts: Vec<(i32, i32)> = (0..4).map(|j| {
                let (h, l, above) = steps[BAYER[brow][(column + j) % 4]];
                if flag(&c, above) { (card(&c, h), card(&c, l)) } else { (-99, 0) }
            }).collect();
            v.extend(ts.iter().map(|t| t.0));
            v.extend(ts.iter().map(|t| t.1));
            v.push(n as i32);
            v.extend(&lights[at..at + 2 * n]);
            at += 2 * n;
            deck.push(v);
        }
        let dots = self.go(host, "mac_bdot", &deck);
        let mut px = vec![false; n * n];
        let mut at = 0;
        for (j, &(cnt, first, ..)) in rows.iter().enumerate() {
            let (cnt, first) = (card(&c, cnt), card(&c, first));
            for i in 0..cnt { px[j * n + (first + r as i32 + i) as usize] = dots[at] == 1; at += 1; }
        }
        px
    }

    /// The Finnish flag, cropped to its window: its width and its dots, black true; framed in black.
    fn flag(&self, host: &mut Host) -> (usize, Vec<bool>) {
        use boom::*;
        let mut c = Calc::new();
        let [l, t, r, b] = FINNISH_RECT.map(|v| c.num(v as i64));
        let k = c.num(-1 - FLAG_IN);
        let fw = c.lin(&[(r, None, false), (l, None, true), (k, None, false)]);
        let fh = scale(&mut c, fw, FLAG_RATIO.1, FLAG_RATIO.0);
        let k = c.num(-1 - FLAG_FOOT - FLAG_TOP);
        let room = c.lin(&[(b, None, false), (t, None, true), (k, None, false)]);
        let ch = c.min(fh, room);
        // the bands' edges: ceil(k size / parts)
        let mut edges = vec![];
        for (way, size) in [(0, fw), (1, fh)] {
            let parts = if way == 0 { FLAG_RATIO.0 } else { FLAG_RATIO.1 };
            for k in [FLAG_BANDS[way].0, FLAG_BANDS[way].1] {
                let p = c.mulk(size, -k);
                let q = c.divk(p, parts);
                edges.push(c.neg(q));
            }
        }
        // its last column and row
        let (rn, bn) = (c.addk(fw, -1), c.addk(ch, -1));
        self.eval(host, &mut c);
        let (w, h) = (card(&c, fw), card(&c, ch));
        let (r, b) = (card(&c, rn), card(&c, bn));
        let mut deck = vec![w, h];
        deck.extend(edges.iter().map(|&e| card(&c, e)));
        let dots = host.go1(&self.m["mac_flag"], &[&deck], (w * h) as usize);
        let (w, h) = (w as usize, h as usize);
        let mut px: Vec<bool> = dots.iter().map(|&d| d == 1).collect();
        // its frame, the first and last rows and columns, black from the fill machine
        let sides = [[0, r, 0, 0], [0, r, b, b], [0, 0, 0, b], [r, r, 0, b]];
        let mut deck = vec![4];
        for sd in &sides { deck.push(1); deck.extend(sd); }
        let black = host.gon(&self.m["mac_fill"], &[&deck]);
        let mut at = 0;
        for [l, r, t, b] in sides { for y in t..=b { for x in l..=r { px[y as usize * w + x as usize] = black[at] == 1; at += 1; } } }
        (w, px)
    }

    // ---- the desk ---------------------------------------------------------------------------------------------------

    /// A frame of the desk; `pictures` the game's frames, the sphere and the flag, worked out (each its width and pixels).
    fn desk(&self, host: &mut Host, shot: &Shot, pictures: &HashMap<Pic, (usize, Vec<bool>)>) -> Vec<bool> {
        // every place on CARDIAC: the layers' items, the zoom rects' ends, the pointer's points
        let mut deck = vec![0];
        let mut groups = 0;
        let mut group = |deck: &mut Vec<i32>, rect: Rect, ats: &[At]| {
            deck.extend(rect_cards(rect));
            deck.push(ats.len() as i32);
            for a in ats { let (w, d) = a.cards(); deck.extend([w as i32, d as i32]); }
            groups += 1;
        };
        for layer in &shot.layers {
            let ats: Vec<At> = layer.items.iter().flat_map(|item| match item {
                Item::Fill(l, t, r, b, _) | Item::Frame(l, t, r, b) => vec![*l, *t, *r, *b],
                Item::Sprite(_, x, y) | Item::Picture(_, x, y) => vec![*x, *y],
                Item::Clip(_, x, y, foot) => vec![*x, *y, *foot],
                Item::Text(tx) => vec![tx.x, tx.y],
            }).collect();
            group(&mut deck, layer.rect, &ats);
        }
        for z in &shot.zoom { group(&mut deck, z.from.0, &z.from.1); group(&mut deck, z.to.0, &z.to.1); }
        let points: Vec<Point> = match &shot.cursor {
            Some((_, Mouse::Still(p))) => vec![*p],
            Some((_, Mouse::Glide { from, to, .. })) | Some((_, Mouse::Line { from, to, .. })) => vec![*from, *to],
            None => vec![],
        };
        for p in &points { group(&mut deck, p.rect, &[p.x, p.y]); }
        deck[0] = groups;
        let total = (deck.len() - 1 - 5 * groups as usize) / 2;
        let mut placed = Placed { v: host.go1(&self.m["mac_place"], &[&deck], total), at: 0 };

        // every line of text on the text machine: its glyphs, and the boxes' corners and the caret as glyphs of no size
        let mut lines = vec![];
        let mut texts = vec![];   // per layer item: Some(index into lines)
        let mut places = vec![];  // the items' places, in order
        for layer in &shot.layers {
            for item in &layer.items {
                let n = match item { Item::Fill(..) | Item::Frame(..) => 4, Item::Clip(..) => 3, Item::Sprite(..) | Item::Text(_) | Item::Picture(..) => 2 };
                let p: Vec<i32> = (0..n).map(|_| placed.next()).collect();
                if let Item::Text(tx) = item {
                    let mut glyphs = vec![];
                    if let Some(([l, t, _, _], _)) = tx.back { glyphs.push([0, -l as i32, t as i32]); }
                    for piece in &tx.pieces {
                        match piece {
                            Piece::Str(face, s) => glyphs.extend(s.chars().map(|c| self.text.glyph(*face, code(c)))),
                            Piece::Gap(g) => glyphs.push([*g as i32, 0, 0]),
                            Piece::Sprite(_, adv) => glyphs.push([*adv as i32, 0, 0]),
                        }
                    }
                    if let Some(([_, _, r, b], _)) = tx.back { glyphs.push([0, r as i32, b as i32]); }
                    if tx.caret { glyphs.push([0, 0, 0]); }
                    texts.push(Some(lines.len()));
                    lines.push(Line { x: p[0], y: p[1], across: match tx.align { Align::Start => 0, Align::Centre => 1, Align::End => 2 }, glyphs });
                } else { texts.push(None); }
                places.push(p);
            }
        }
        let set = if lines.is_empty() { vec![] } else { self.text.set(host, &lines) };
        let mut starts = vec![0];
        for l in &lines { starts.push(starts.last().unwrap() + l.glyphs.len()); }

        // the zoom rects' edges, on the ease machine (straight)
        let mut ends = vec![];
        for z in &shot.zoom {
            let from: Vec<i32> = (0..4).map(|_| placed.next()).collect();
            let to: Vec<i32> = (0..4).map(|_| placed.next()).collect();
            ends.push((from, to, z.step as i32, z.steps as i32));
        }
        let edges = if ends.is_empty() { vec![] } else {
            let deck: Vec<i32> = ends.iter().flat_map(|(f, t, j, n)| (0..4).flat_map(move |k| [1, f[k], t[k], *j, *n])).collect();
            host.go1(&self.ease, &[&[4 * ends.len() as i32], &deck], 4 * ends.len())
        };

        // the pointer
        let mouse = match &shot.cursor {
            Some((_, Mouse::Still(_))) => Some((placed.next(), placed.next())),
            Some((_, Mouse::Glide { step, steps, .. })) => {
                let (x0, y0, x1, y1) = (placed.next(), placed.next(), placed.next(), placed.next());
                let (j, n) = (*step as i32, *steps as i32);
                let left = host.go1(&self.m["mac_glide"], &[&[j, n, 2, x0, x1, y0, y1]], 2);
                let left2 = host.go1(&self.m["mac_glide"], &[&[j, n, 2, 0, left[0], 0, left[1]]], 2);
                let at = host.go1(&self.m["mac_arc"], &[&[x0, y0, x1, y1, left[0], left[1], left2[0], left2[1]]], 2);
                Some((at[0], at[1]))
            }
            Some((_, Mouse::Line { step, steps, .. })) => {
                // straight, on the ease machine: from + (to - from) j / n each way
                let (x0, y0, x1, y1) = (placed.next(), placed.next(), placed.next(), placed.next());
                let (j, n) = (*step as i32, *steps as i32);
                let at = host.go1(&self.ease, &[&[2], &[1, x0, x1, j, n, 1, y0, y1, j, n]], 2);
                Some((at[0], at[1]))
            }
            None => None,
        };
        assert_eq!(placed.at, placed.v.len());

        // the boxes of one colour (the filled rectangles, the outlines' four sides, the text boxes' backs), in the
        // items' order, each clipped to the screen: their pixels from the fill machine
        let mut boxes: Vec<(bool, i32, i32, i32, i32)> = vec![];
        let mut k = 0;
        for layer in &shot.layers {
            for item in &layer.items {
                let p = &places[k];
                match item {
                    Item::Fill(.., v) => boxes.push((*v, p[0], p[1], p[2], p[3])),
                    Item::Frame(..) => {
                        let (l, t, r, b) = (p[0], p[1], p[2], p[3]);
                        boxes.extend([(true, l, t, r, t), (true, l, b, r, b), (true, l, t, l, b), (true, r, t, r, b)]);
                    }
                    Item::Text(tx) => if let Some((_, v)) = tx.back {
                        let g = &set[2 * starts[texts[k].unwrap()]..2 * starts[texts[k].unwrap() + 1]];
                        let n = g.len() / 2 - 1 - tx.caret as usize;
                        boxes.push((v, g[0], g[1], g[2 * n], g[2 * n + 1]));
                    },
                    _ => {}
                }
                k += 1;
            }
        }
        let clipped: Vec<[i32; 4]> = boxes.iter().map(|&(_, l, t, r, b)| [l.max(0), r.min(W as i32 - 1), t.max(0), b.min(H as i32 - 1)]).collect();
        let mut deck = vec![boxes.len() as i32];
        for (bx, c) in boxes.iter().zip(&clipped) { deck.push(bx.0 as i32); deck.extend(c); }
        let filled = host.gon(&self.m["mac_fill"], &[&deck]);
        let (mut next_box, mut next_px) = (0, 0);

        // the picture: the desktop's grey, the items in order
        let mut s = vec![false; W * H];
        self.sprite(&mut s, W, H, "desktop", 0, 0);
        let mut fill = |s: &mut Vec<bool>| {
            let [l, r, t, b] = clipped[next_box];
            next_box += 1;
            for y in t..=b { for x in l..=r { s[y as usize * W + x as usize] = filled[next_px] == 1; next_px += 1; } }
        };
        let mut k = 0;
        for layer in &shot.layers {
            for item in &layer.items {
                let p = &places[k];
                match item {
                    Item::Fill(..) => fill(&mut s),
                    Item::Frame(..) => { for _ in 0..4 { fill(&mut s); } }
                    Item::Sprite(name, ..) => self.sprite(&mut s, W, H, name, p[0], p[1]),
                    Item::Clip(name, ..) => self.sprite(&mut s, W, (p[2] + 1) as usize, name, p[0], p[1]),
                    Item::Picture(pic, ..) => {
                        // the picture's pixels copied (the shading's dots only, laid over what is there)
                        let ink = *pic == Pic::Sphere;
                        let (w, px) = (pictures[pic].0, &pictures[pic].1);
                        for j in 0..px.len() / w { for i in 0..w {
                            let (x, y) = (p[0] + i as i32, p[1] + j as i32);
                            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 || (ink && !px[j * w + i]) { continue }
                            s[y as usize * W + x as usize] = px[j * w + i];
                        } }
                    }
                    Item::Text(tx) => {
                        let line = texts[k].unwrap();
                        let g = &set[2 * starts[line]..2 * starts[line + 1]];
                        let mut at = 0;
                        if tx.back.is_some() {
                            fill(&mut s);
                            at = 1;
                        }
                        if tx.glyphs {
                            for piece in &tx.pieces {
                                match piece {
                                    Piece::Str(face, st) => for c in st.chars() { self.text.lay_bits(&mut s, W, *face, code(c), g[2 * at], g[2 * at + 1], tx.black); at += 1; },
                                    Piece::Gap(_) => at += 1,
                                    Piece::Sprite(name, _) => { self.sprite(&mut s, W, H, name, g[2 * at], g[2 * at + 1]); at += 1; }
                                }
                            }
                            if tx.caret { let c = g.len() / 2 - 1; self.sprite(&mut s, W, H, "caret", g[2 * c], g[2 * c + 1]); }
                        }
                    }
                }
                k += 1;
            }
        }

        assert_eq!(next_px, filled.len());

        // the zoom rects: each outline's pixels (its top and bottom rows and its sides, each pixel once) turned over
        // on the XOR machine, one rect after another
        let mut on = vec![false; W * H];
        for z in 0..ends.len() {
            let e = &edges[4 * z..4 * z + 4];
            let (l, t, r, b) = (e[0].max(0), e[1].max(0), e[2].min(W as i32 - 1), e[3].min(H as i32 - 1));
            let mut px: Vec<usize> = vec![];
            let sides = (l..=r).map(|x| t as usize * W + x as usize).chain((l..=r).map(|x| b as usize * W + x as usize))
                .chain((t + 1..b).map(|y| y as usize * W + l as usize)).chain((t + 1..b).map(|y| y as usize * W + r as usize));
            for i in sides { if !on[i] { on[i] = true; px.push(i); } }
            for &i in &px { on[i] = false; }
            for part in px.chunks(999) {
                let deck: Vec<i32> = part.iter().map(|&i| s[i] as i32).collect();
                let flipped = host.go1(&self.m["mac_xor"], &[&[part.len() as i32], &deck], part.len());
                for (&i, &v) in part.iter().zip(&flipped) { s[i] = v == 1; }
            }
        }
        self.corners(&mut s);
        if let (Some((name, _)), Some((x, y))) = (&shot.cursor, mouse) { self.sprite(&mut s, W, H, name, x, y); }
        s
    }

    fn corners(&self, s: &mut [bool]) {
        let (r, b) = (W as i32 - 1, H as i32 - 1);
        for (name, x, y) in [("corner_tl", 0, 0), ("corner_tr", r, 0), ("corner_bl", 0, b), ("corner_br", r, b)] { self.sprite(s, W, H, name, x, y); }
    }
}

pub fn render(demo: &Path, refbin: &str) {
    let mut host = Host::new();
    let shots = script();
    assert_eq!(shots.len(), FRAMES);
    let mac = Mac::new(demo, &mut host);
    // how far the dissolve has come at a third, two, three
    let come = mac.text.steps(&mut host, 255, &[(1, 3), (2, 3), (3, 3)]);
    let mut pictures: HashMap<i16, Vec<bool>> = HashMap::new();
    let mut worked: HashMap<Pic, (usize, Vec<bool>)> = HashMap::new();
    let mut video = cardiac_host::Video::<W, H>::new(FRAMES);
    for f in live::frames(FRAMES) {
        let shot = &shots[f];
        let s = match shot.tunnel {
            None => {
                // the pictures this frame shows, worked out the first time
                for layer in &shot.layers { for item in &layer.items {
                    if let Item::Picture(pic, ..) = item {
                        if !worked.contains_key(pic) {
                            match pic {
                                Pic::Game(g) => mac.shown(&mut host, *g, &mut worked),
                                Pic::Sphere => { let px = (2 * boom::BALL_R as usize + 1, mac.sphere(&mut host)); worked.insert(*pic, px); }
                                Pic::Flag => { let px = mac.flag(&mut host); worked.insert(*pic, px); }
                            }
                        }
                    }
                } }
                mac.desk(&mut host, shot, &worked)
            }
            Some(t) => {
                if !pictures.contains_key(&t.picture) { let p = mac.tunnel(&mut host, t.picture); pictures.insert(t.picture, p); }
                let pic = &pictures[&t.picture];
                let mut s: Vec<bool> = (0..W * H).map(|i| pic[(i / W / BLOCK) * TW + i % W / BLOCK]).collect();
                if let Some((card, k, n)) = t.title {
                    let thirds = host.go1(&mac.m["mac_thirds"], &[&[k as i32, n as i32]], 1)[0];
                    // each screen row's pixels of the card, shown when their turn has come
                    let card = &mac.cards[card];
                    for y in 0..H {
                        let xs: Vec<usize> = (0..W).filter(|&x| card[(y / BLOCK) * TW + x / BLOCK] > 0).collect();
                        if xs.is_empty() { continue }
                        let turns: Vec<i32> = xs.iter().map(|&x| mac.order[y * W + x] as i32).collect();
                        let shown = host.go1(&mac.m["mac_dissolve"], &[&[come[thirds as usize - 1], xs.len() as i32], &turns], xs.len());
                        for (&x, &on) in xs.iter().zip(&shown) { if on == 1 { s[y * W + x] = card[(y / BLOCK) * TW + x / BLOCK] == 1; } }
                    }
                }
                mac.corners(&mut s);
                s
            }
        };
        let frame = &mut video.buf[f * W * H * 3..(f + 1) * W * H * 3];
        for (i, &b) in s.iter().enumerate() { let v = if b { 0 } else { 255 }; frame[3 * i..3 * i + 3].copy_from_slice(&[v, v, v]); }
    }
    host.done();
    // the same at 2x on the 1536x768 frame, for the cut (a blit)
    let blit = |src: &[u8], frames: usize| {
        let mut hd = cardiac_host::Video::<{ text::HW }, { text::HH }>::new(frames);
        let (fw, fh) = (text::HW, text::HH);
        for f in live::range(frames) { for y in 0..2 * H { for x in 0..2 * W {
            let s = (f * W * H + (y / 2) * W + x / 2) * 3;
            let d = (f * fw * fh + (HD_Y + y) * fw + HD_X + x) * 3;
            hd.buf[d..d + 3].copy_from_slice(&src[s..s + 3]);
        } } }
        hd
    };
    let hd = blit(&video.buf, FRAMES);
    let refs = reference(refbin, &[demo]);
    let ref_hd = if refs.is_empty() { vec![] } else { blit(&refs, FRAMES).buf };
    finish("mac (512x342)", &host, &video, &refs, &demo.join("mac/video.rgb"), "");
    finish("mac (1536x768)", &Host::new(), &hd, &ref_hd, &demo.join("mac/video_hd.rgb"), " (the 512x342 frames at 2x, at 256, 42)");
}

use crate::fx::text;
