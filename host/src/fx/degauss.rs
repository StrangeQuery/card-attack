//! The degauss at the rotozoom's end (fx/rotodg.rs; prototypes/degauss.py, made for Mode X's last frames; the
//! reference is segments/modex/degauss.rs): a colour
//! blotch creeps in where the screen is lit, a monitor's DEGAUSS sign comes up, blinking, then the degauss: the
//! picture jolts and wobbles, its three colours land apart, a rainbow swirl washes round it, a brief brightening,
//! all decaying over 16 frames; the sign shakes with the picture and is gone as it settles.
//!
//! The blotch, per frame (at the picture's own size, the rotozoom's 192x96): its strength on the scale machine (dg_scale); on dg_scale too, each row's two
//! blotch strengths (with the falloffs down), each pixel's light over 24 (red + green + blue on the light machine,
//! dg_lum), each pixel's two amounts (the row's strength by the falloff across, by the light), each colour's tints
//! added (70 A1 / 32, then -40 A2 / 32 for red ...); held to 0..255 on the jolt's mix machine (dg_mix, no flash).
//!
//! The sign, on the 1536x768 frame: the text kit's page DEGAUSS (ui_page places its letters, umix lays them on,
//! once), the warning sign's pixels (an asset, modex/data/warning.txt), and the box under them, each picture pixel
//! under it taken toward black once on umix.
//!
//! The jolt, per frame: its sizes on dg_scale (from the decay table); the waves' angles on the clock machine
//! (dr_clock: rows, columns, the swirl's phases), their sines on the sine machine (rb_sine), each row's push across and
//! each column's down on dg_scale and the push machine (dg_push, a push per colour in whole pixels). The swirl: each
//! picture pixel's angle from the middle once (dg_anglew, dg_atan), turned on by each colour's phase (dg_turn), each
//! distinct angle's sine (rb_sine); each pixel's light (dg_lum), its swirl's strength and tint (dg_scale). Then each
//! pair of a colour the jolt brings to a pixel and the tint there, brightened by the flash on dg_mix. The host fetches every pixel's colour from where the pushes
//! say (the pixel's place and its row's and column's punched pushes, held to the frame), as the BBS's wobble does.
//! Every machine computes each distinct item once (the host sorts them out and files the answers back).
use crate::fx::text::{self, pages, Text};
use crate::*;
use std::collections::HashMap;
use std::path::Path;

pub const B0: usize = 114;
pub const D0: usize = 134;
pub const DN: usize = 16;
const CREEP: i32 = 20;
const SIGN_ON: usize = 128;
const SIGN_OFF: usize = 146;
const PHASE0: i32 = 48;
const BOX: (usize, usize, usize, usize) = (489, 72, 1047, 228);
const ICON: (usize, usize) = (519, 96);
const YELLOW: [u8; 3] = [255, 255, 85];
/// The tints: each blotch's push on red, green and blue, over 32.
const TINT: [[i32; 2]; 3] = [[70, -40], [-50, 60], [80, -30]];
const HW: usize = text::HW;
const HH: usize = text::HH;
const W: usize = 320;
const H: usize = 240;

/// Whether the sign shows at frame k: blinking (3 frames on, 3 off) before the jolt, then on.
pub fn sign_shown(k: usize) -> bool { (SIGN_ON..D0).contains(&k) && (k - SIGN_ON) % 6 < 3 || (D0..SIGN_OFF).contains(&k) }
/// Whether frame k is touched at all.
pub fn touched(k: usize) -> bool { k >= B0 && k < D0 + DN }

fn rows(p: &Path) -> Vec<Vec<i32>> {
    std::fs::read_to_string(p).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect()
}

/// Each distinct item in the order first seen, and each item's place among them.
fn distinct<T: Copy + Eq + std::hash::Hash>(items: &[T]) -> (Vec<T>, Vec<usize>) {
    let mut at: HashMap<T, usize> = HashMap::with_capacity(items.len() / 4);
    let mut order = vec![];
    let slot = items.iter().map(|&k| *at.entry(k).or_insert_with(|| { order.push(k); order.len() - 1 })).collect();
    (order, slot)
}

pub struct Degauss { m: HashMap<&'static str, Machine>, gauss: Vec<Vec<i32>>, decay: Vec<i32>, flash: Vec<i32>, atan: Vec<i32>, rates: Vec<Vec<i32>>,
    table: Vec<i32>, warning: Vec<Vec<u8>>,
    /// Each picture pixel's angle from the middle (quarter, tenths), made when first wanted.
    turn: Option<Vec<(i32, i32)>>,
    /// The sign's pixels on the frame (Some colour), made when first wanted.
    sign: Option<Vec<Option<[u8; 3]>>>,
    /// The waves' angles at each row (two) and column, before the frame's phase, made when first wanted.
    waves: Option<Vec<(i32, i32)>>,
    /// The picture's size (the rotozoom's 192x96).
    w: usize, h: usize }

impl Degauss {
    pub fn new(demo: &Path) -> Degauss { Degauss::picture(demo, W, H, "gauss.txt") }
    /// For a w x h picture, its falloffs (modex/data's `gauss`).
    pub fn picture(demo: &Path, w: usize, h: usize, gauss: &str) -> Degauss {
        let m = ["dg_scale", "dg_mix", "dg_push", "dg_angle", "dg_anglew", "dg_atan", "dg_turn", "dg_lum", "dr_clock", "rb_sine"].into_iter().map(|n| (n, rust(demo, n))).collect();
        let d = |n: &str| demo.join("modex/data").join(n);
        let decay = rows(&d("decay.txt"));
        Degauss { m, gauss: rows(&d(gauss)), w, h, decay: decay[0].clone(), flash: decay[1].clone(), atan: rows(&d("atan.txt"))[0].clone(), rates: rows(&d("rates.txt")),
            table: numbers(&demo.join("rubik/data/sine.txt")),
            warning: std::fs::read_to_string(d("warning.txt")).unwrap().lines().filter(|l| !l.starts_with('#')).map(|l| l.bytes().collect()).collect(),
            turn: None, sign: None, waves: None }
    }

    /// The scale machine on many (base, a, b, c): each base + a b / c, the product's part rounded down; each distinct
    /// one once.
    fn scale(&self, host: &mut Host, items: &[[i32; 4]]) -> Vec<i32> {
        let (order, slot) = distinct(items);
        let mut out = Vec::with_capacity(order.len());
        for part in order.chunks(999) {
            let deck: Vec<i32> = part.iter().flatten().copied().collect();
            out.extend(host.go1(&self.m["dg_scale"], &[&[part.len() as i32], &deck], part.len()));
        }
        slot.iter().map(|&s| out[s]).collect()
    }

    /// The mix machine on many (colour, tint) with the flash: each (colour + tint) (32 + flash) / 32, held to 0..255;
    /// each distinct pair once.
    fn mix(&self, host: &mut Host, flash: i32, pairs: &[[i32; 2]]) -> Vec<i32> {
        let (order, slot) = distinct(pairs);
        let mut out = Vec::with_capacity(order.len());
        for part in order.chunks(999) {
            let deck: Vec<i32> = part.iter().flatten().copied().collect();
            out.extend(host.go1(&self.m["dg_mix"], &[&[flash, part.len() as i32], &deck], part.len()));
        }
        slot.iter().map(|&s| out[s]).collect()
    }

    /// The turn machine: each angle turned on by `by`.
    fn turned(&self, host: &mut Host, by: (i32, i32), angles: &[(i32, i32)]) -> Vec<(i32, i32)> {
        let mut out = Vec::with_capacity(angles.len());
        for part in angles.chunks(999) {
            let deck: Vec<i32> = part.iter().flat_map(|&(q, r)| [q, r]).collect();
            out.extend(host.go1(&self.m["dg_turn"], &[&[by.0, by.1, part.len() as i32], &deck], 2 * part.len()).chunks(2).map(|a| (a[0], a[1])));
        }
        out
    }

    /// The clock machine on many angles, each its terms (count, rate): each as (quarter, tenths).
    fn clock(&self, host: &mut Host, angles: &[Vec<(i32, &[i32])>]) -> Vec<(i32, i32)> {
        let mut out = Vec::with_capacity(angles.len());
        for part in angles.chunks(999) {
            let mut deck = vec![part.len() as i32];
            for terms in part {
                deck.push(terms.len() as i32);
                for &(n, r) in terms { deck.push(n); deck.extend(r); }
            }
            out.extend(host.go1(&self.m["dr_clock"], &[&deck], 2 * part.len()).chunks(2).map(|a| (a[0], a[1])));
        }
        out
    }

    /// 999 sin of an angle, on the sine machine.
    fn sine(&self, host: &mut Host, (q, r): (i32, i32)) -> i32 { host.go1(&self.m["rb_sine"], &[&[q, r], &self.table, &self.table], 2)[0] }

    /// The blotch's strength at frame k (32nds), on the scale machine; None where there is none.
    pub fn strength(&self, host: &mut Host, k: usize) -> Option<i32> {
        let s = if (B0..D0).contains(&k) { self.scale(host, &[[0, (k - B0) as i32, 32, CREEP]])[0] }
            else if (D0..D0 + DN).contains(&k) { self.scale(host, &[[32, (k - D0) as i32, -5, 1]])[0] } else { return None };
        (s > 0).then_some(s)
    }

    /// The picture with the blotch at strength s.
    pub fn blotch(&self, host: &mut Host, s: i32, pic: &[u8]) -> Vec<u8> {
        let (w, h) = (self.w, self.h);
        // each blotch's strength at each row; each pixel's light (red + green + blue) / 24
        let down = self.scale(host, &(0..h).flat_map(|v| [[0, s, self.gauss[2][v], 32], [0, s, self.gauss[3][v], 32]]).collect::<Vec<_>>());
        let colours: Vec<[i32; 3]> = pic.chunks(3).map(|p| [p[0] as i32, p[1] as i32, p[2] as i32]).collect();
        let sums = self.lum(host, &colours);
        let light = self.scale(host, &sums.iter().map(|&l| [0, 1, l, 24]).collect::<Vec<_>>());
        // each pixel's two amounts: the row's strength times the falloff across, over 32, times the light over 32
        let mut amounts = [vec![], vec![]];
        for (k, amount) in amounts.iter_mut().enumerate() {
            let near = self.scale(host, &(0..w * h).map(|i| [0, down[2 * (i / w) + k], self.gauss[k][i % w], 32]).collect::<Vec<_>>());
            *amount = self.scale(host, &(0..w * h).map(|i| [0, near[i], light[i], 32]).collect::<Vec<_>>());
        }
        // the tints added, the first blotch's, then the second's, then held to 0..255
        let first = self.scale(host, &(0..w * h * 3).map(|i| [pic[i] as i32, amounts[0][i / 3], TINT[i % 3][0], 32]).collect::<Vec<_>>());
        let second = self.scale(host, &(0..w * h * 3).map(|i| [first[i], amounts[1][i / 3], TINT[i % 3][1], 32]).collect::<Vec<_>>());
        self.mix(host, 0, &second.iter().map(|&v| [v, 0]).collect::<Vec<_>>()).iter().map(|&v| v as u8).collect()
    }

    /// The light machine on many colours: each red + green + blue, each distinct colour once.
    fn lum(&self, host: &mut Host, colours: &[[i32; 3]]) -> Vec<i32> {
        let (order, slot) = distinct(colours);
        let mut out = Vec::with_capacity(order.len());
        for part in order.chunks(999) {
            let deck: Vec<i32> = part.iter().flatten().copied().collect();
            out.extend(host.go1(&self.m["dg_lum"], &[&[part.len() as i32], &deck], part.len()));
        }
        slot.iter().map(|&s| out[s]).collect()
    }

    /// 999 sin of many angles, on the sine machine, each distinct angle once.
    fn sines(&self, host: &mut Host, angles: &[(i32, i32)]) -> Vec<i32> {
        let (order, slot) = distinct(angles);
        let out: Vec<i32> = order.iter().map(|&a| self.sine(host, a)).collect();
        slot.iter().map(|&s| out[s]).collect()
    }

    /// Lay the sign on the 1536x768 frame: the box taken toward black (each picture pixel under it once, `at` saying
    /// which is where), then the warning sign and the word.
    pub fn lay_sign(&mut self, host: &mut Host, kit: &Text, frame: &mut [u8], at: &[u32], pic: &[u8]) {
        if self.sign.is_none() {
            let page = &pages::DEGAUSS;
            let word = kit.draw(host, page, page.chars(), false);
            let on = text::on_frame(page);
            let mut px = vec![None; HW * HH];
            for (r, line) in self.warning.iter().enumerate() { for (c, &ch) in line.iter().enumerate() {
                if ch == b'.' { continue }
                for y in 0..6 { for x in 0..6 { px[(ICON.1 + 6 * r + y) * HW + ICON.0 + 6 * c + x] = Some(if ch == b'Y' { YELLOW } else { [0; 3] }); } }
            } }
            for (i, &p) in on.iter().enumerate() {
                if p != text::NONE && word[3 * p as usize + 1] > 0 { px[i] = Some([word[3 * p as usize], word[3 * p as usize + 1], word[3 * p as usize + 2]]); }
            }
            self.sign = Some(px);
        }
        let (x0, y0, x1, y1) = BOX;
        let mut cells: Vec<u32> = (y0..y1).flat_map(|y| (x0..x1).map(move |x| at[y * HW + x])).collect();
        cells.sort(); cells.dedup();
        let dark = kit.mix(host, &cells.iter().flat_map(|&p| (0..3).map(move |c| [pic[3 * p as usize + c] as i32, 0, 24])).collect::<Vec<_>>());
        let darkened: HashMap<u32, usize> = cells.iter().enumerate().map(|(n, &p)| (p, n)).collect();
        for y in y0..y1 { for x in x0..x1 {
            let i = y * HW + x;
            let n = darkened[&at[i]];
            for c in 0..3 { frame[3 * i + c] = dark[3 * n + c] as u8; }
        } }
        for (i, p) in self.sign.as_ref().unwrap().iter().enumerate() { if let Some(p) = p { frame[3 * i..3 * i + 3].copy_from_slice(p); } }
    }

    /// Each picture pixel's angle from the middle: the quarter and sides on dg_anglew, each distinct ratio's angle on
    /// dg_atan.
    fn turn(&mut self, host: &mut Host) -> &[(i32, i32)] {
        let (w, h) = (self.w, self.h);
        if self.turn.is_none() {
            let mut sides = Vec::with_capacity(w * h);
            for v in 0..h {
                let deck: Vec<i32> = (0..w).flat_map(|u| [u as i32, v as i32]).collect();
                // (Mode X's middle is the machine's own; another picture's on the deck of dg_anglew)
                let o = if (w, h) == (320, 240) { host.go1(&self.m["dg_angle"], &[&[w as i32], &deck], 4 * w) }
                    else { host.go1(&self.m["dg_anglew"], &[&[w as i32 - 1, h as i32 - 1, w as i32], &deck], 4 * w) };
                sides.extend(o.chunks(4).map(|s| [s[0], s[1], s[2], s[3]]));
            }
            let mut order: Vec<[i32; 3]> = sides.iter().map(|s| [s[1], s[2], s[3]]).collect();
            order.sort(); order.dedup();
            let mut within = HashMap::new();
            for part in order.chunks(999) {
                let mut deck = vec![part.len() as i32];
                for s in part { deck.extend(s); deck.extend(&self.atan); }
                for (s, a) in part.iter().zip(host.go1(&self.m["dg_atan"], &[&deck], part.len())) { within.insert(*s, a); }
            }
            self.turn = Some(sides.iter().map(|s| (s[0], within[&[s[1], s[2], s[3]]])).collect());
        }
        self.turn.as_ref().unwrap()
    }

    /// The jolt at step j over `base`, the 1536x768 frame with the blotch and the sign; `at` gives the picture pixel
    /// shown at each pixel of the frame (NONE at the sides).
    pub fn jolt(&mut self, host: &mut Host, j: usize, base: &[u8], at: &[u32]) -> Vec<u8> {
        let p = PHASE0 + j as i32;
        let a10 = self.scale(host, &[[0, 460, self.decay[j], 999]])[0];
        let s = self.scale(host, &[[0, 1, a10, 2], [0, 3, a10, 20], [0, 60, self.decay[j], 999], [0, 8, self.flash[j], 999], [0, 2, a10, 1]]);
        let (half, fringe, swirl, flash, a20) = (s[0], s[1], s[2], s[3], s[4]);
        let r: Vec<&[i32]> = self.rates.iter().map(|v| v.as_slice()).collect();
        // the waves' angles and sines: per row two, per column one, each its place's angle (once) turned on by the
        // frame's phase
        if self.waves.is_none() {
            let mut places: Vec<Vec<(i32, &[i32])>> = (0..HH as i32).flat_map(|y| [vec![(y, r[0])], vec![(y, r[1])]]).collect();
            // (a column past 767 counted in two terms, each a card)
            places.extend((0..HW as i32).map(|x| if x < 768 { vec![(x, r[2])] } else { vec![(768, r[2]), (x - 768, r[2])] }));
            self.waves = Some(self.clock(host, &places));
        }
        let phase = self.clock(host, &[vec![(p, r[3])], vec![(p, r[4])], vec![(p, r[5])]]);
        let places = self.waves.as_ref().unwrap();
        let rows: Vec<(i32, i32)> = places[..2 * HH].to_vec();
        let mut turns: Vec<(i32, i32)> = vec![];
        let (even, odd): (Vec<_>, Vec<_>) = rows.chunks(2).map(|a| (a[0], a[1])).unzip();
        let (first, second) = (self.turned(host, phase[0], &even), self.turned(host, phase[1], &odd));
        for y in 0..HH { turns.push(first[y]); turns.push(second[y]); }
        turns.extend(self.turned(host, phase[2], &places[2 * HH..].to_vec()));
        let sines = self.sines(host, &turns);
        // each row's push across: (s1 / 2 + s2 / 4), its size's share, per colour in pixels
        let halves = self.scale(host, &(0..HH).map(|y| [0, 1, sines[2 * y], 2]).collect::<Vec<_>>());
        let waves = self.scale(host, &(0..HH).map(|y| [halves[y], 1, sines[2 * y + 1], 4]).collect::<Vec<_>>());
        let pushes = self.scale(host, &waves.iter().map(|&q| [0, a20, q, 999]).collect::<Vec<_>>());
        let across = host.go1(&self.m["dg_push"], &[&[half, HH as i32], &pushes], 3 * HH);
        // each column's push down
        let waves = self.scale(host, &(0..HW).map(|x| [0, 7, sines[2 * HH + x], 20]).collect::<Vec<_>>());
        let pushes = self.scale(host, &waves.iter().map(|&q| [0, a10, q, 999]).collect::<Vec<_>>());
        let down: Vec<i32> = pushes.chunks(HW / 2).flat_map(|part| host.go1(&self.m["dg_push"], &[&[fringe, part.len() as i32], part], 3 * part.len())).collect();
        // the swirl: each colour's phase, each picture pixel's angle turned on by it, each distinct angle's sine
        let phases = self.clock(host, &(0..3).map(|c| vec![(p, r[3]), (c, r[3])]).collect::<Vec<_>>());
        let turn = self.turn(host).to_vec();
        let mut angle = vec![[0usize; 3]; self.w * self.h];           // tenths round the turn, per colour
        for (c, &by) in phases.iter().enumerate() {
            for (i, (q, t)) in self.turned(host, by, &turn).into_iter().enumerate() { angle[i][c] = (900 * q + t) as usize; }
        }
        let sine = self.sines(host, &angle.iter().flatten().map(|&a| ((a / 900) as i32, (a % 900) as i32)).collect::<Vec<_>>());
        // each pixel's light, its swirl's strength
        let sums = self.lum(host, &(0..HW * HH).map(|i| [base[3 * i] as i32, base[3 * i + 1] as i32, base[3 * i + 2] as i32]).collect::<Vec<_>>());
        let strength = self.scale(host, &sums.iter().map(|&l| [0, swirl, l, 765]).collect::<Vec<_>>());
        // each pixel's tint per colour (each distinct strength and angle once)
        let mut keys = vec![[0, 0, 0, 999]; HW * HH * 3];
        for i in 0..HW * HH {
            if at[i] == text::NONE { continue }
            for c in 0..3 { keys[3 * i + c] = [0, strength[i], sine[3 * at[i] as usize + c], 999]; }
        }
        let tints = self.scale(host, &keys);
        // each pixel's colour fetched from where its row and column push it, with the tint there, brightened (each
        // distinct pair once)
        let mut pairs = vec![[0; 2]; HW * HH * 3];
        for y in 0..HH { for x in 0..HW { for c in 0..3 {
            let sx = (x as i32 + across[3 * y + c]).clamp(0, HW as i32 - 1) as usize;
            let sy = (y as i32 + down[3 * x + c]).clamp(0, HH as i32 - 1) as usize;
            let i = 3 * (y * HW + x) + c;
            pairs[i] = [base[(sy * HW + sx) * 3 + c] as i32, tints[i]];
        } } }
        self.mix(host, flash, &pairs).iter().map(|&v| v as u8).collect()
    }
}
