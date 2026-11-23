// The degauss (prototypes/degauss.py, made for Mode X's last frames; the film has it at the rotozoom's end), in whole
// numbers: a colour blotch creeps in where the
// screen is lit, a monitor's DEGAUSS sign comes up, blinking, then the degauss: the picture jolts and wobbles, its
// three colours land apart, a rainbow swirl washes round it, a brief brightening, all decaying over 16 frames; the sign
// shakes with the picture (it is laid on before the jolt) and is gone as the picture settles; then it is clean.
// Included by reference.rs and by the rotozoom's degauss reference (segments/rotodg/reference.rs) with #[path]. Every
// a b / c below is rounded down (the scale machine's). Mode X's picture is 320x240 (its falloffs data/gauss.txt); the
// rotozoom's is 192x96, its own falloffs (data/gauss_roto.txt) and its middle on the angle's deck.
//
// Timing (frames of Mode X): the blotch from B0 = 114, its strength s = 32 (k - 114) / 20 32nds at frame k (the
// prototype's creep over 24 frames in 20); the jolt at D0 = 134, step j = k - 134 of 16, s = 32 - 5 j there (none
// from j = 7: the prototype's 1 - 2.5 t). The sign from 128: shown 128..130, not 131..133, then 134..145.
//
// The blotch, per picture pixel (u, v) of colour (r, g, b) (the blotches are smooth enough to take at the picture's
// size): the falloffs G1x[u] G1y[v] and G2x[u] G2y[v] in 32nds (data/gauss.txt or data/gauss_roto.txt), each blotch's amount A = ((s G y / 32) G x / 32) l / 32, l = (r + g + b) / 24 the light (a blotch
// only shows where the screen is lit), and to each colour 70 A1 / 32 - 40 A2 / 32 (red), -50 A1 / 32 + 60 A2 / 32
// (green), 80 A1 / 32 - 30 A2 / 32 (blue) added, held to 0..255. (The prototype adds them in floating point, and holds
// them to 0..255 only after the jolt.)
//
// The sign, on the 1536x768 frame over the blotched picture: the box (489..1046 across, 72..227 down) taken toward
// black by 24 32nds (a + (0 - a) 24 / 32 toward zero, umix), the warning sign (data/warning.txt, each pixel 6 x 6
// from (519, 96)) yellow and black, the word (ui/pages.rs's DEGAUSS) bright green.
//
// The jolt, at step j (E = decay.txt's 999 exp(-j / 4), Fl its 999 exp(-j / 2)): its size a10 = 460 E / 999 tenths
// of a pixel (the prototype's 46 exp(-4 t)), a10 / 2, 3 a10 / 20 (0.15 a); the swirl's strength 60 E / 999, the flash
// 8 Fl / 999 32nds. The angles are the clock machine's (dr_clock: counts times rates, data/rates.txt, rounded to a
// tenth of a degree), each place's angle and the frame's phase taken apart and added (the turn machine's), the phase
// the prototype's at its frame p = 48 + j; their sines are the sine machine's (rb_sine). Per row y: the two waves'
// sines s1 (y / 37, then 2.1 p) and s2 (y / 11, then -3.57 p), q = s1 / 2 + s2 / 4, the row's push 2 a10 q / 999
// tenths, the three colours' pushes across it the push plus a10 / 2, plus 0, less a10 / 2, over 10 (whole pixels).
// Per column x: s3 (x / 90, then 2.73 p), its push a10 (7 s3 / 20) / 999 tenths, the colours' pushes down it that
// plus 3 a10 / 20, 0, less it, over 10. Colour c of pixel (x, y) is colour c of the picture (blotched, with the sign)
// at (x + its row's push, y + its column's push), each held to the frame. The swirl: each picture pixel's angle from
// the frame's middle (atan2 of its middle's offset, 2 v - (h - 1) down and 2 u - (w - 1) across: the table
// atan(k / 256) at 256 near / far, or a quarter less it past half a quarter, in tenths of a degree), turned on by
// 2.1 (p + c) radians for colour c (the clock's), its sine; per pixel of the frame the picture's light there
// L = r + g + b, w = (60 E / 999) L / 765, and w sin / 999 added to the colour (none at the sides). Then the flash:
// v + (v / 32) F + (v mod 32) F / 32 for v the sum, held to 0..255 (0 where v is not above 0).
#![allow(dead_code)]
use std::path::Path;

pub const B0: usize = 114;
pub const D0: usize = 134;
pub const DN: usize = 16;
pub const CREEP: usize = 20;
pub const SIGN_ON: usize = 128;
pub const SIGN_OFF: usize = 146;
pub const PHASE0: i64 = 48;
pub const BOX: (usize, usize, usize, usize) = (489, 72, 1047, 228);
pub const ICON: (usize, usize) = (519, 96);
pub const GREEN: [u8; 3] = [85, 255, 85];
pub const YELLOW: [u8; 3] = [255, 255, 85];
const HW: usize = 1536;
const HH: usize = 768;

pub struct Tables { pub gauss: Vec<Vec<i64>>, pub decay: Vec<i64>, pub flash: Vec<i64>, pub atan: Vec<i64>, pub rates: Vec<[i64; 3]>, pub sine: Vec<i64>, pub sign: Vec<Vec<u8>> }

fn rows(p: &Path) -> Vec<Vec<i64>> {
    std::fs::read_to_string(p).unwrap().lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect()
}

pub fn read(demo: &Path) -> Tables {
    let d = |n: &str| demo.join("modex/data").join(n);
    let decay = rows(&d("decay.txt"));
    let sign = std::fs::read_to_string(d("warning.txt")).unwrap().lines().filter(|l| !l.starts_with('#')).map(|l| l.bytes().collect()).collect();
    Tables { gauss: rows(&d("gauss.txt")), decay: decay[0].clone(), flash: decay[1].clone(), atan: rows(&d("atan.txt"))[0].clone(),
        rates: rows(&d("rates.txt")).iter().map(|r| [r[0], r[1], r[2]]).collect(),
        sine: std::fs::read_to_string(demo.join("rubik/data/sine.txt")).unwrap().split_whitespace().map(|v| v.parse().unwrap()).collect(), sign }
}

/// The scale machine: base + a b / c, the product's part rounded down.
pub fn scale(base: i64, a: i64, b: i64, c: i64) -> i64 { base + (a * b).div_euclid(c) }

/// The blotch's strength at frame k, in 32nds (0: none).
pub fn strength(k: usize) -> i64 {
    if (B0..D0).contains(&k) { scale(0, (k - B0) as i64, 32, CREEP as i64) } else if (D0..D0 + DN).contains(&k) { scale(32, (k - D0) as i64, -5, 1).max(0) } else { 0 }
}

/// Whether the sign shows at frame k.
pub fn sign_shown(k: usize) -> bool { (SIGN_ON..D0).contains(&k) && (k - SIGN_ON) % 6 < 3 || (D0..SIGN_OFF).contains(&k) }

/// The 320x240 picture with the blotch at strength s.
pub fn blotch(t: &Tables, s: i64, pic: &[u8]) -> Vec<u8> { blotch_wh(&t.gauss, s, pic, 320, 240) }

/// The falloffs of another picture's size (data/gauss_roto.txt: the rotozoom's 192x96).
pub fn read_gauss(demo: &Path, name: &str) -> Vec<Vec<i64>> { rows(&demo.join("modex/data").join(name)) }

/// A w x h picture with the blotch at strength s, its falloffs `gauss`.
pub fn blotch_wh(gauss: &[Vec<i64>], s: i64, pic: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut out = pic.to_vec();
    const TINT: [[i64; 2]; 3] = [[70, -40], [-50, 60], [80, -30]];
    let t = Tg { gauss };
    for v in 0..h {
        let (r1, r2) = (scale(0, s, t.gauss[2][v], 32), scale(0, s, t.gauss[3][v], 32));
        for u in 0..w {
            let i = (v * w + u) * 3;
            let p = [pic[i] as i64, pic[i + 1] as i64, pic[i + 2] as i64];
            let light = (p[0] + p[1] + p[2]) / 24;
            let a1 = r1 * t.gauss[0][u] / 32 * light / 32;
            let a2 = r2 * t.gauss[1][u] / 32 * light / 32;
            for c in 0..3 { out[i + c] = scale(scale(p[c], a1, TINT[c][0], 32), a2, TINT[c][1], 32).clamp(0, 255) as u8; }
        }
    }
    out
}

struct Tg<'a> { gauss: &'a [Vec<i64>] }

/// The clock machine: counts times rates, rounded to the tenth, as a quarter turn and the rest.
pub fn clock(terms: &[(i64, [i64; 3])]) -> (i64, i64) {
    let total: i64 = terms.iter().map(|&(n, r)| n * (900_000 * r[0] + 1000 * r[1] + r[2])).sum::<i64>() + 500;
    let tenths = (total / 1000).rem_euclid(3600);
    (tenths / 900, tenths % 900)
}

/// The turn machine: an angle turned on by another (quarter turns and tenths).
pub fn turned((q, r): (i64, i64), (pq, pr): (i64, i64)) -> (i64, i64) {
    if r + pr >= 900 { ((q + pq + 1) % 4, r + pr - 900) } else { ((q + pq) % 4, r + pr) }
}

/// The sine machine: 999 sin of a quarter turn and rest.
pub fn sine(t: &Tables, (q, r): (i64, i64)) -> i64 {
    let read = |at: i64| { let (d, f) = ((at / 10) as usize, at % 10); t.sine[d] + (t.sine[d + 1] - t.sine[d]) * f / 10 };
    let (mut s, mut c) = (read(r), read(900 - r));
    for _ in 0..q { let o = s; s = c; c = -o; }
    s
}

/// Which way (x, y) is from the middle (y down), as a quarter turn and tenths of a degree: atan(near / far) from the
/// table at 256 near / far (rounded down), or a quarter less it past half the quarter.
pub fn angle(t: &Tables, x: i64, y: i64) -> (i64, i64) {
    let (ax, ay) = (x.abs(), y.abs());
    let (quarter, num, den) = if x > 0 && y > 0 { (0, ay, ax) } else if x < 0 && y > 0 { (1, ax, ay) } else if x < 0 && y < 0 { (2, ay, ax) } else { (3, ax, ay) };
    (quarter, if num <= den { t.atan[(256 * num / den) as usize] } else { 900 - t.atan[(256 * den / num) as usize] })
}

/// The sign's pixels on the 1536x768 frame: Some(colour) on the warning sign or a letter.
pub fn sign_pixels(t: &Tables, word: &dyn Fn(usize, usize) -> bool) -> Vec<Option<[u8; 3]>> {
    let mut px = vec![None; HW * HH];
    for (r, line) in t.sign.iter().enumerate() { for (c, &ch) in line.iter().enumerate() {
        if ch == b'.' { continue }
        for y in 0..6 { for x in 0..6 { px[(ICON.1 + 6 * r + y) * HW + ICON.0 + 6 * c + x] = Some(if ch == b'Y' { YELLOW } else { [0; 3] }); } }
    } }
    for y in 0..HH { for x in 0..HW { if word(x, y) { px[y * HW + x] = Some(GREEN); } } }
    px
}

/// Lay the sign on a 1536x768 frame: the box taken toward black, then its pixels.
pub fn lay_sign(frame: &mut [u8], sign: &[Option<[u8; 3]>], mix: &dyn Fn(u8, u8, u8) -> u8) {
    let (x0, y0, x1, y1) = BOX;
    for y in y0..y1 { for x in x0..x1 { for c in 0..3 { let i = (y * HW + x) * 3 + c; frame[i] = mix(frame[i], 0, 24); } } }
    for (i, p) in sign.iter().enumerate() { if let Some(p) = p { frame[3 * i..3 * i + 3].copy_from_slice(p); } }
}

/// The jolt at step j over the 1536x768 frame `base` (the blotched picture with the sign); `cell` gives the picture's
/// pixel shown at each pixel of the frame (None at the sides), `turn` each picture pixel's angle.
pub fn jolt(t: &Tables, j: usize, base: &[u8], cell: &dyn Fn(usize, usize) -> Option<usize>, turn: &[(i64, i64)]) -> Vec<u8> {
    let (e, fl, p) = (t.decay[j], t.flash[j], PHASE0 + j as i64);
    let a10 = scale(0, 460, e, 999);
    let (half, fringe, swirl, flash) = (scale(0, 1, a10, 2), scale(0, 3, a10, 20), scale(0, 60, e, 999), scale(0, 8, fl, 999));
    let r = &t.rates;
    let sigma = [1i64, 0, -1];
    let across: Vec<[i64; 3]> = (0..HH as i64).map(|y| {
        let s1 = sine(t, turned(clock(&[(y, r[0])]), clock(&[(p, r[3])])));
        let s2 = sine(t, turned(clock(&[(y, r[1])]), clock(&[(p, r[4])])));
        let push = scale(0, scale(0, 2, a10, 1), scale(scale(0, 1, s1, 2), 1, s2, 4), 999);
        sigma.map(|s| (push + s * half).div_euclid(10))
    }).collect();
    let down: Vec<[i64; 3]> = (0..HW as i64).map(|x| {
        let s3 = sine(t, turned(clock(&[(x, r[2])]), clock(&[(p, r[5])])));
        let push = scale(0, a10, scale(0, 7, s3, 20), 999);
        sigma.map(|s| (push + s * fringe).div_euclid(10))
    }).collect();
    let phase: Vec<(i64, i64)> = (0..3).map(|c| clock(&[(p, r[3]), (c, r[3])])).collect();
    let mut out = vec![0u8; HW * HH * 3];
    for y in 0..HH { for x in 0..HW {
        let i = y * HW + x;
        let light: i64 = (0..3).map(|c| base[3 * i + c] as i64).sum();
        let w = scale(0, swirl, light, 765);
        for c in 0..3 {
            let sx = (x as i64 + across[y][c]).clamp(0, HW as i64 - 1) as usize;
            let sy = (y as i64 + down[x][c]).clamp(0, HH as i64 - 1) as usize;
            let b = base[(sy * HW + sx) * 3 + c] as i64;
            let s = match cell(x, y) {
                Some(q) => scale(0, w, sine(t, turned(turn[q], phase[c])), 999),
                None => 0,
            };
            let v = b + s;
            out[3 * i + c] = if v <= 0 { 0 } else { (v + v / 32 * flash + v % 32 * flash / 32).min(255) as u8 };
        }
    } }
    out
}
