// The dive's reference: the whole picture from the height map, the script's numbers and the font, in plain Rust,
// with the same whole-number arithmetic the machines are given, but none of their card handling. Every frame the
// host's machines file must be this, byte for byte.
//
// The seabed is the height map (map.txt: per row of z, its base, its drop, how far past a lip it is, 201 heights
// in tenths over the base, its glowing spots). Each pixel's ray, 300 (f + a r + b u), is marched up its screen
// column a map row at a time, from where the pixel below stopped. Near (the first 30 rows) the seabed is blocks a
// unit square, each its column's height snapped down to an even number of units; where a ray goes into one it is
// shaded by the face it hit (a top, a front, a side), crevices and lit edges where blocks meet, strata of rock tones
// and a grain of chips, rubble and coral; further out the seabed is smooth, lit by the slope ahead and textured with
// the same strata. Then the diver's lamp near the camera, caustics, glowing spots and lip bands, the fog. Open water is
// dark below and bright above, with god rays. Then, in the prototype's order, the kelp, seagrass and fish (painted
// over what is further), the marine snow, the jellyfish (far ones first), the caption, the portal and the whiteout;
// each channel exactly as worked out, 0..255. Then the dive's second video, its creatures at 1536x768
// (reference_hd.rs). Arguments: map.txt script.txt font3x5.txt hd.txt (the HD creatures' index; its .bin files beside
// it) and the swarm's tables.txt; on standard output the 192x96 frames (RGB), then the 1536x768 frames.
use std::io::Write;
const W: usize = 192;
const H: usize = 96;
#[allow(unused_attributes, dead_code)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;   // the tables, shared with the machines and the host
use demokit::tables::{self, widen};
#[path = "reference_hd.rs"]
mod hd;
#[path = "reference_orca.rs"]
mod orca;
#[path = "reference_knob.rs"]
mod knob;
/// The script: every frame's numbers, worked out here natively (script.rs, against the train's operations).
#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
mod ops { include!("../train/ops.rs"); }
#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
mod whole { pub use super::ops::*; include!("script.rs"); }
const GLOW_FALLOFF: [i32; 11] = widen(tables::GLOW_FALLOFF);
const PORTAL_RADIUS: [i32; 33] = widen(tables::PORTAL_RADIUS);
const PORTAL_ANGLE: [i32; 33] = widen(tables::PORTAL_ANGLE);
const PORTAL_SPILL: [i32; 11] = widen(tables::PORTAL_SPILL);
const BELL_SIN: [i32; 12] = widen(tables::BELL_SIN);
const BELL_COS: [i32; 12] = widen(tables::BELL_COS);
const BELL_GLOW: [i32; 32] = widen(tables::BELL_GLOW);
const QUARTER_COS64: [i32; 17] = widen(tables::QUARTER_COS64);
const QUARTER_SINE60: [i32; 16] = widen(tables::QUARTER_SINE60);
const JELLY_FADE: [i32; 31] = widen(tables::JELLY_FADE);
const JELLY_HALO: [i32; 49] = widen(tables::JELLY_HALO);
const LEAF_TAPER: [i32; 21] = widen(tables::LEAF_TAPER);
const KELP_FOG: [i32; 50] = widen(tables::KELP_FOG);
const FISH_FOG: [i32; 50] = widen(tables::FISH_FOG);
struct Map { base: Vec<i32>, lip: Vec<i32>, rows: Vec<Vec<i32>>, spots: Vec<Vec<(i32, i32)>> }

fn load_map(path: &str) -> Map {
    let (mut base, mut lip, mut rows, mut spots) = (vec![], vec![], vec![], vec![]);
    for l in std::fs::read_to_string(path).unwrap().lines() {
        let v: Vec<i32> = l.split_whitespace().map(|s| s.parse().unwrap()).collect();
        base.push(v[0]); lip.push(v[2]); rows.push(v[3..204].to_vec());
        spots.push(v[205..].chunks(2).map(|c| (c[0], c[1])).collect());
    }
    Map { base, lip, rows, spots }
}

/// The map's column under x (hundredths of a unit): its whole unit, 0..200 for x = -100..100.
fn column(x: i32) -> i32 { (x.div_euclid(100) + 100).clamp(0, 200) }
/// The seabed's block at a column: its height over the row's base in tenths, snapped down to a whole unit.
fn block(row: &[i32], c: i32) -> i32 { P_STEP * row[c as usize].div_euclid(10 * P_STEP) }
/// The height (tenths over the row's base) under x (hundredths of a unit): the block's top.
fn sample(row: &[i32], x: i32, k: i32) -> i32 { if k < P_FAR { 10 * block(row, column(x)) } else { smooth(row, x) } }
/// The smooth seabed (tenths over the row's base) under x: the two heights either side, the step between them in
/// tenths of a unit.
fn smooth(row: &[i32], x: i32) -> i32 {
    let mut i = x.div_euclid(100) + 100;
    let mut part = x.rem_euclid(100) / 10;
    if i < 0 { i = 0; part = 0; }
    if i > 199 { i = 199; part = 9; }
    let left = row[i as usize];
    let right = row[i as usize + 1];
    left + (right - left) * part / 10
}
/// The rock's tone at a column c and row j where the seabed is hy (hundredths over the row's base b): strata of four
/// tones, five units deep, each column shifting them a tone now and then; and how much lighter or darker it is (32nds),
/// with a fine dark band every half unit down a face.
fn strata(c: i32, j: i32, b: i32, hy: i32, face: bool) -> (i32, i32) {
    let hidx = (U_PERM[c.rem_euclid(16) as usize] + j % 16) % 16;
    let (shift, jit) = (U_VARY[hidx as usize] / 10, U_VARY[hidx as usize] % 10 - 5);
    let tone = ((b + hy.div_euclid(100)).div_euclid(5) + shift).rem_euclid(4);
    let band = if face && hy.rem_euclid(100) / 10 % 5 == 0 { P_BANDK } else { 0 };
    (tone, jit - band)
}
/// How many tenths of the slice from the ray's entry (height ye, hundredths) to its exit (y) it reaches a level
/// (whole units): 10 a / (a + b), a the tenths over it at entry (at most 99), b the tenths under it after; 0 when the
/// ray is not over it at entry, 10 when it never goes under.
fn reach(ye: i32, y: i32, level: i32) -> i32 {
    if y >= 100 * level { return 10; }
    let above = (ye.div_euclid(10) - 10 * level).min(99);
    let under = 10 * level - y.div_euclid(10);
    if above <= 0 { 0 } else { 10 * above / (above + under) }
}

/// a + (b - a) k / 32, k 0..32
fn mix32(a: i32, b: i32, k: i32) -> i32 { a + (b - a) * k / 32 }
fn mix32c(a: [i32; 3], b: [i32; 3], k: i32) -> [i32; 3] { [0, 1, 2].map(|c| mix32(a[c], b[c], k)) }
fn mix256(a: i32, b: i32, k: i32) -> i32 { a + (b - a) * k / 256 }
/// The fog, in 32nds, at d along the view: as 0.85 (d / 70)^0.8 up to 1, in two straight pieces.
fn fog32(d: i32) -> i32 { if d < 20 { d / 2 } else { (10 + (d - 20) / 3).min(32) } }
/// 100 cos(2 pi j / 64), from the quarter table.
fn cos64(j: i32) -> i32 {
    let j = j.rem_euclid(64);
    match j { 0..=16 => QUARTER_COS64[j as usize], 17..=32 => -QUARTER_COS64[(32 - j) as usize], 33..=48 => -QUARTER_COS64[(j - 32) as usize], _ => QUARTER_COS64[(64 - j) as usize] }
}

/// v t / 100 for v and t within 999 either way, by v's digits 100 a + 10 b + c: ((c t / 10 + b t) / 10 + a t),
/// toward zero, with v's sign.
fn hundredth(v: i32, t: i32) -> i32 { let s = v.abs(); let h = ((s % 10) * t / 10 + (s / 10 % 10) * t) / 10 + (s / 100) * t; if v < 0 { -h } else { h } }

/// v t / 1000, as v t / 100 then a tenth of it.
fn thousandth(v: i32, t: i32) -> i32 { hundredth(v, t) / 10 }

/// A jellyfish's place on the screen in tenths of a pixel from the middle, if it is no further across or up than
/// ahead: (across, up, ahead), toward zero.
fn see_fine(p: [i32; 3], f: [i32; 3], r: [i32; 3], u: [i32; 3]) -> Option<(i32, i32, i32)> {
    let (ahead, across, up) = (along(p, f), along(p, r), along(p, u));
    if ahead < 3 || across.abs() >= ahead || up.abs() >= ahead { return None; }
    Some((1000 * across / ahead, 1000 * up / ahead, ahead))
}

/// How long (a, b) is: the larger part and a table's share of it, M + M T[32 m / M] / 200.
fn magnitude(a: i32, b: i32) -> i32 {
    let (big, small) = (a.max(b), a.min(b));
    if big == 0 { return 0; }
    big + hundredth(big, PORTAL_RADIUS[(32 * small / big) as usize]) / 2
}

/// The screen pixel (index) at a column and row counted from the middle's, if it is on the screen.
fn pixel(c: i32, r: i32) -> Option<usize> { let (x, y) = (c + 96, r + 48); (x >= 0 && y >= 0 && x < W as i32 && y < H as i32).then(|| y as usize * W + x as usize) }

/// A leaf (a blade, tapering, or a stalk's piece) from a to b on the screen (tenths of a pixel from the middle, y
/// up), half as wide as w (tenths), at d (tenths) ahead: each pixel near it whose middle is within its width is
/// painted, lit across its width, a darker rib down a wide blade and light at its rim, faded by the fog; nearer
/// things already there stay.
#[allow(clippy::too_many_arguments)]
fn leaf(rgb: &mut [[i32; 3]], depth: &mut [i32], a: (i32, i32), b: (i32, i32), w: i32, blade: bool, d: i32, c0: [i32; 3], c1: [i32; 3], rim: [i32; 3], far: [i32; 3], fog: i32) {
    let (ax, ay, bx, by) = (a.0, -a.1, b.0, -b.1);       // y down
    let (vx, vy) = (bx - ax, by - ay);
    let length = magnitude(vx.abs(), vy.abs());
    if length == 0 { return; }
    let (ux, uy) = (100 * vx / length, 100 * vy / length);
    let floor10 = |v: i32| v.div_euclid(10);
    let (c0b, c1b) = (floor10(ax.min(bx) - w - 10), floor10(ax.max(bx) + w + 19));
    let (r0, r1) = (floor10(ay.min(by) - w - 10), floor10(ay.max(by) + w + 19));
    let steep = vy.abs() > 2 * (w + 10);
    let reach = if steep { (w + 10) * length / vy.abs() } else { 0 };
    for r in r0.max(-48)..=r1.min(47) {
        let (mut cl, mut cr) = (c0b, c1b);
        if steep {
            // the strip within the width of the line, on this row
            let (n, dd) = if vy < 0 { (-vx * (10 * r + 5 - ay), -vy) } else { (vx * (10 * r + 5 - ay), vy) };
            let centre = ax + n.div_euclid(dd);
            cl = cl.max(floor10(centre - reach - 10));
            cr = cr.min(floor10(centre + reach + 10));
        }
        for c in cl.max(-96)..=cr.min(95) {
            let Some(i) = pixel(c, r) else { continue };
            let (px, py) = (10 * c + 5 - ax, 10 * r + 5 - ay);
            let along = hundredth(px, ux) + hundredth(py, uy);
            let across = hundredth(px, uy) - hundredth(py, ux);
            let cap = if along < 0 { -along } else if along > length { along - length } else { 0 };
            let dist = magnitude(cap, across.abs());
            let u = 100 * along.clamp(0, length) / length;
            let ww = if blade { hundredth(w, LEAF_TAPER[(u / 5) as usize]) } else { w };
            let wide = ww.max(5);
            if dist > wide { continue; }
            let mut lit = 20 + (100 * across / wide).clamp(-100, 100) * 12 / 100;
            if blade && ww > 12 && dist < 5 { lit = lit * 72 / 100; }
            let shine = if ww > 12 && dist > ww * 8 / 10 && across > 0 { 11 } else { 0 };
            let colour = mix32c(mix32c(mix32c([0; 3], mix32c(c0, c1, u * 8 / 25), lit), rim, shine), far, fog);
            if depth[i].min(99) * 10 >= d { rgb[i] = colour; depth[i] = (d + 9) / 10; }
        }
    }
}

/// A disc at p on the screen (tenths of a pixel from the middle, y up), radius r (tenths, 5..31): the pixels whose
/// middles lie within it.
fn disc(p: (i32, i32), r: i32) -> Vec<usize> {
    let r = r.clamp(5, 31);
    let (x, y) = (p.0, -p.1);
    let mut v = vec![];
    for row in (y - r).div_euclid(10)..=(y + r + 9).div_euclid(10) {
        for col in (x - r).div_euclid(10)..=(x + r + 9).div_euclid(10) {
            let (dx, dy) = (10 * col + 5 - x, 10 * row + 5 - y);
            if dx.abs() <= r && dy.abs() <= r && dx * dx <= r * r - dy * dy { if let Some(i) = pixel(col, row) { v.push(i); } }
        }
    }
    v
}

/// The random sequence: s -> (21 s + 17) mod 1000.
fn lcg(s: i32) -> i32 { (21 * s + 17) % 1000 }

/// How far a place (tenths) lies along a vector (hundredths), in tenths: each product split by the place's digits,
/// ((c v / 10 + b v) / 10 + a v) for a place 100 a + 10 b + c, toward zero.
fn along(p: [i32; 3], v: [i32; 3]) -> i32 {
    (0..3).map(|c| { let s = p[c].abs(); let t = ((s % 10) * v[c] / 10 + (s / 10 % 10) * v[c]) / 10 + (s / 100) * v[c]; if p[c] < 0 { -t } else { t } }).sum()
}

/// A place from the camera (tenths) on the screen: the pixel and how far ahead (tenths), if seen.
fn see(p: [i32; 3], f: [i32; 3], r: [i32; 3], u: [i32; 3]) -> Option<(i32, i32, i32)> {
    let (ahead, across, up) = (along(p, f), along(p, r), along(p, u));
    if ahead < 3 || across.abs() > 2 * ahead || up.abs() > ahead { return None; }
    Some((96 + 100 * across / ahead, 48 - 100 * up / ahead, ahead))
}

/// The faces a ray can go into a block by.
const TOP: i32 = 0; const FRONT: i32 = 1; const SIDE_L: i32 = 2; const SIDE_R: i32 = 3;
/// The blocks' height steps (units); the map rows ahead that are blocks (further, the smooth seabed).
const P_STEP: i32 = 2; const P_FAR: i32 = 30;
/// The light on each face (32nds); what an edge catching the light adds and a crevice takes (32nds); the fine bands down
/// a face; rubble's lighter and darker stones; the colour an edge catches.
const P_TOP: i32 = 29; const P_FRONT: i32 = 16; const P_SIDE_R: i32 = 22; const P_SIDE_L: i32 = 12;
const P_RIM: i32 = 14; const P_DEEP: i32 = 6; const P_AO: i32 = 5; const P_BANDK: i32 = 4; const P_PEBBLE: i32 = 5;
const P_PALE: [i32; 3] = [200, 220, 215];
/// The diver's lamp: as far as it reaches (the fog's distance) and at most how far toward full light (32nds); its
/// beam is full within 7 pixels of row 61.5 in the middle (aimed at the floor ahead) and gone by 23; what it lights it takes toward the rock's
/// colour in full light, its own shading kept; laid on after the fog.
const P_LAMP: i32 = 70; const P_LAMPMAX: i32 = 26;
/// The rock's four strata and coral, and the sand, in full light; the rock's tables.
const U_ROCKS: [[i32; 3]; 5] = [[124, 119, 130], [134, 116, 98], [108, 124, 112], [98, 106, 126], [230, 120, 100]];
const U_SAND_LIT: [i32; 3] = [202, 186, 148];
const U_PERM: [i32; 16] = widen(tables::U_ROCK_PLACE);
const U_VARY: [i32; 16] = widen(tables::U_ROCK_VARY);
const U_GRAIN: [i32; 16] = widen(tables::U_ROCK_GRAIN);
#[derive(Clone, Copy)]
enum Px { Water, Hit(i32, i32, i32) }   // hit at slice k with x and height over the row base

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let map = load_map(&args[1]);
    let data = whole::ScriptData::parse(&std::fs::read_to_string(&args[2]).unwrap());
    let script: Vec<Vec<i32>> = whole::script(&mut ops::Native::new(vec![]), &data).into_iter().map(|l| l.into_iter().map(|v| v as i32).collect()).collect();
    // 20 sin(6 k degrees) round the turn, from the quarter
    let sin60: Vec<i32> = (0..60).map(|k: usize| match k { 0..=15 => QUARTER_SINE60[k], 16..=29 => QUARTER_SINE60[30 - k], 30..=45 => -QUARTER_SINE60[k - 30], _ => -QUARTER_SINE60[60 - k] }).collect();
    // per pixel, once: |(a, b, 1)| in 300ths
    let norm: Vec<i32> = (0..W * H).map(|i| { let (a, b) = ((2 * (i % W) as i32 - 191) as f64, (95 - 2 * (i / W) as i32) as f64);
        let t = 360000 + 9 * (a as i32 * a as i32 + b as i32 * b as i32); let mut g = 0; while 4 * (g + 1) * (g + 1) <= t { g += 1; } g }).collect();
    // the marine snow's specks: places in a box 40 across, 14 up and 14 along (tenths)
    let mut seed = 1;
    let specks: Vec<[i32; 3]> = (0..450).map(|_| {
        seed = lcg(seed); let x = seed / 10 * 4 + seed % 10 * 4 / 10 - 200;
        seed = lcg(seed); let y = seed / 100 * 14 + seed % 100 * 7 / 50;
        seed = lcg(seed); let z = seed / 100 * 14 + seed % 100 * 7 / 50;
        [x, y, z] }).collect();
    // the listing's 3 x 5 font: a character's five rows of three dots
    let mut font = std::collections::HashMap::new();
    for l in std::fs::read_to_string(&args[3]).unwrap().lines().filter(|l| !l.starts_with('#')) {
        let p: Vec<&str> = l.split(' ').collect();
        font.insert(if p[0] == "space" { ' ' } else { p[0].chars().next().unwrap() }, p[1..6].iter().map(|r| i32::from_str_radix(r, 2).unwrap()).collect::<Vec<i32>>());
    }
    let hd_data = hd::Data::load(&args[4]);
    let trig = hd::Trig::load(&args[5]);
    let knob_data = knob::Data::load(std::path::Path::new(&args[4]).parent().unwrap());
    // the first frame each jellyfish shows (by its number), for its fade-in at 1536x768
    let mut first: std::collections::HashMap<i32, i64> = std::collections::HashMap::new();
    for (n, sc) in script.iter().enumerate() {
        if sc[0] == 1 { continue; }
        let f100 = [sc[24], sc[25], sc[26]];
        for k in 0..sc[46] as usize {
            let j = &sc[47 + 16 * k..47 + 16 * (k + 1)];
            let ahead = along([j[0], j[1], j[2]], f100);
            if ahead <= 10 || ahead >= 600 || JELLY_FADE[(ahead / 20) as usize] * sc[44] / 32 == 0 { continue; }
            first.entry(j[15]).or_insert(n as i64);
        }
    }
    let mut out = std::io::stdout().lock();
    let mut hd_frames: Vec<u8> = Vec::with_capacity(script.len() * hd::HW * hd::HH * 3);
    let mut hd_rgb = vec![[0i64; 3]; hd::HW * hd::HH];
    // DEEP_REF_FRAMES=a,b,...: only those frames (the rest black), for working on a few
    let only: Option<Vec<usize>> = std::env::var("DEEP_REF_FRAMES").ok().map(|l| l.split(',').map(|v| v.trim().parse().unwrap()).collect());
    for (n, sc) in script.iter().enumerate() {
        if only.as_ref().is_some_and(|o| !o.contains(&n)) { out.write_all(&vec![0u8; W * H * 3]).unwrap(); hd_frames.extend(std::iter::repeat(0u8).take(hd::HW * hd::HH * 3)); continue; }
        let (white, cxh, cyu, cyf, czu, czf) = (sc[0], sc[1], sc[2], sc[3], sc[4], sc[5]);
        let (f, r, u) = ([sc[6], sc[7], sc[8]], [sc[9], sc[10], sc[11]], [sc[12], sc[13], sc[14]]);
        let (light, light10) = (sc[15], sc[16]);
        if white == 1 { out.write_all(&vec![255u8; W * H * 3]).unwrap(); hd_frames.extend(std::iter::repeat(255u8).take(hd::HW * hd::HH * 3)); continue; }
        let (t1, t2, t3) = (sc[17], sc[18], sc[19]);
        let shaft = [sc[20], sc[21]];
        let (pulse, dark) = (sc[22], sc[23]);
        let (f100, r100, u100) = ([sc[24], sc[25], sc[26]], [sc[27], sc[28], sc[29]], [sc[30], sc[31], sc[32]]);
        let (snow_up, snow_along, show) = (sc[33], sc[34], sc[35]);
        let fade = sc[36];
        let (on, plane, from, swirl, white32) = (sc[37], sc[38], [sc[39], sc[40]], [sc[41], sc[42]], sc[43]);
        let aim = [sc[sc.len() - 2], sc[sc.len() - 1]];      // the torch's beam's middle, half pixels right and up
        // the frame's colours at its light
        let lit = |lo: [i32; 3], hi: [i32; 3]| mix32c(lo, hi, light);
        let water_up = lit([2, 6, 14], [40, 160, 170]);
        let water_down = lit([1, 2, 6], [4, 30, 50]);
        let far = lit([1, 3, 8], [6, 40, 62]);
        let (sand, cc) = (lit([12, 11, 9], [202, 186, 148]), lit([13, 15, 14], [210, 250, 240]));
        let rocks = U_ROCKS.map(|t| lit([7, 7, 8], t));
        let pale = lit([16, 16, 16], P_PALE);
        let j0 = (czu + 1) as usize;
        let s0 = 10 - czf;
        // the rays
        let mut d3 = vec![[0i32; 3]; W * H];
        let mut qx = vec![0i32; W * H]; let mut qy = vec![0i32; W * H];
        let mut x0 = vec![0i32; W * H]; let mut y0 = vec![0i32; W * H]; let mut lim = vec![0i32; W * H];
        for py in 0..H { for px in 0..W {
            let i = py * W + px;
            let (a, b) = (2 * px as i32 - 191, 95 - 2 * py as i32);
            for c in 0..3 { d3[i][c] = (200 * f[c] + a * r[c] + b * u[c]).div_euclid(200); }
            let dz = d3[i][2];
            assert!(dz > 0);
            qx[i] = 100 * d3[i][0] / dz;
            qy[i] = 100 * d3[i][1] / dz;
            x0[i] = cxh + (s0 * qx[i]).div_euclid(10);
            y0[i] = 100 * (cyu - map.base[j0]) + cyf + (s0 * qy[i]).div_euclid(10);
            lim[i] = 11 * (dz / 30) + 11 * (dz % 30) / 30 - 1;
        } }
        // the march, a column at a time from the bottom: each pixel starts where the one below it stopped
        let mut res = vec![Px::Water; W * H];
        for px in 0..W {
            let mut p = H as i32 - 1;
            let mut k = 0i32;
            let start = |p: i32, k: i32| { let i = p as usize * W + px; (x0[i] + k * qx[i], y0[i] + k * qy[i] - 100 * (map.base[j0 + k as usize] - map.base[j0])) };
            let (mut x, mut y) = start(p, k);
            loop {
                let i = p as usize * W + px;
                if k > lim[i] {
                    p -= 1; if p < 0 { break; }
                    (x, y) = start(p, k); continue;
                }
                if y >= 9000 { break; }
                let j = j0 + k as usize;
                let h = sample(&map.rows[j], x, k);
                if y.div_euclid(10) < h {
                    res[i] = Px::Hit(k, x, y);
                    p -= 1; if p < 0 { break; }
                    (x, y) = start(p, k); continue;
                }
                x += qx[i];
                y += qy[i] + 100 * (map.base[j] - map.base[j + 1]);
                k += 1;
            }
        }
        // shade
        let mut rgb = vec![[0i32; 3]; W * H];
        let mut depth = vec![999i32; W * H];
        let mut crossed = vec![0i32; W * H];
        for py in 0..H { for px in 0..W {
            let i = py * W + px;
            rgb[i] = match res[i] {
                Px::Water => {
                    let up = 100 * d3[i][1] / norm[i];
                    let c = mix32c(water_down, water_up, (16 + up * 3 / 10).clamp(0, 32));
                    // god rays: two waves across the screen along column + 0.45 row, sharpened to shafts
                    let s1 = ((shaft[0] + 39 * py as i32 + 86 * px as i32) / 100) % 60;
                    let s2 = ((shaft[1] + 18 * py as i32 + 41 * px as i32) / 100) % 60;
                    let by = |a: i32, b: i32| a * b / 40;
                    let (v1, v2) = (20 + sin60[s1 as usize], 20 + sin60[s2 as usize]);
                    let (q1, q2) = (by(v1, v1), by(v2, v2));
                    let (f1, f2) = (by(q1, q1), by(q2, q2));
                    let g1 = by(f1, q1);
                    let g2 = by(by(f2, q2), f2);
                    let u = ((up + 20) / 10).max(0);
                    let amount = ((g1 + g2) * u / 28).min(16) * light / 32;
                    mix32c(c, [170, 240, 230], amount)
                }
                Px::Hit(k, x, y) => {
                    let j = j0 + k as usize;
                    let drop = map.base[j - 1] - map.base[j];
                    let row = &map.rows[j];
                    // near: the block it went into; far: the smooth seabed with the rock's strata as a texture.
                    // Either way: where it crossed (tenths of the slice), the place across, how rocky, the rock's
                    // tone, the light on it and the light its edge catches
                    let (r, xh, m, tone, bf, rim) = if k < P_FAR {
                        // back one slice (over this row's base), the columns there and here
                        let (xp, ye) = (x - qx[i], y - qy[i]);
                        assert!(xp.abs() < 9000 && x.abs() < 9000, "near rays stay well on the map");
                        let (c0, c1) = (column(xp), column(x));
                        let (h0, h1) = (block(row, c0), block(row, c1));
                        // which face: the front of the row's block, a side of the next column's, or a top
                        let (face, r, c) = if ye < 100 * h0 { (FRONT, 0, c0) }
                            else if c0 == c1 { (TOP, reach(ye, y, h1), c1) }
                            else if (c1 - c0).abs() == 1 {
                                // crossing one edge: the first column's top if it goes under it before the edge,
                                // else the next column's side if it is under that top at the edge, else its top
                                let gap = (if qx[i] > 0 { 100 - xp.rem_euclid(100) } else { xp.rem_euclid(100) }).min(99);
                                let t = (10 * gap / qx[i].abs()).min(9);
                                let r0 = reach(ye, y, h0);
                                let r1 = reach(ye, y, h1);
                                if r0 < t { (TOP, r0, c0) } else if r1 < t { (if qx[i] > 0 { SIDE_L } else { SIDE_R }, t, c1) } else { (TOP, r1, c1) }
                            } else {
                                // crossing more: the last column's top if it was over it, else its side halfway
                                let r1 = reach(ye, y, h1);
                                if r1 > 0 { (TOP, r1, c1) } else { (if qx[i] > 0 { SIDE_L } else { SIDE_R }, 5, c1) }
                            };
                        let ht = block(row, c);                                   // the block's top (units)
                        let hy = if face == TOP { 100 * ht } else { ye + (qy[i] * r).div_euclid(10) };   // the point's height
                        let xh = xp + (qx[i] * r).div_euclid(10);
                        let front = block(&map.rows[j - 1], c) + drop;           // the block in front, over this base
                        let back = block(&map.rows[j + 1], c) - (map.base[j] - map.base[j + 1]);
                        let left = block(row, (c - 1).max(0));
                        let right = block(row, (c + 1).min(200));
                        let xpart = xh.rem_euclid(100);
                        // light by the face, edges catching it, crevices where blocks meet
                        let mut bf = match face { TOP => P_TOP, FRONT => P_FRONT, SIDE_R => P_SIDE_R, _ => P_SIDE_L };
                        let mut rim = 0;
                        let mut shadow = 0;
                        if face == TOP {
                            if r <= 1 && front < ht { rim += P_RIM; }
                            if r >= 8 && back > ht { shadow += P_AO * (r - 7); }
                            if back >= ht + 4 { shadow += P_DEEP; }
                            if xpart < 20 { if left > ht { shadow += P_AO; } else if left < ht { rim += P_RIM / 2; } }
                            if xpart >= 80 { if right > ht { shadow += P_AO; } else if right < ht { rim += P_RIM / 2; } }
                        } else {
                            let low = if face == FRONT { front } else { h0 };      // the top the face rises from
                            if hy - 100 * low < 35 { shadow += P_AO; }
                            if 100 * ht - hy < 15 { rim += P_RIM / 2; }
                        }
                        let (mut tone, vary) = strata(c, j as i32, map.base[j], hy, face != TOP);
                        // close up, the rock's grain: a 4 x 4 pattern of chips and cracks across each face, shifted
                        // block by block; rubble where a top runs into the wall behind it; and coral on some ledges
                        let hidx = (U_PERM[c.rem_euclid(16) as usize] + j as i32 % 16) % 16;
                        let across = xpart / 25;
                        let down = if face == TOP { r * 4 / 10 } else { hy.rem_euclid(100) / 25 };
                        let grain = U_GRAIN[((hidx + across + 4 * down) % 16) as usize];
                        bf += grain;
                        if face == TOP && back > ht && r >= 6 { bf += if grain >= 0 { P_PEBBLE } else { -P_PEBBLE }; }
                        if face == TOP && front < ht && r <= 2 && hidx % 4 == 0 && grain >= 0 { tone = 4; }
                        let m = if face == TOP && tone < 4 { (10 * ht + 20).clamp(0, 64) * 5 / 10 } else { 32 };
                        (r, xh, m, tone, (bf + vary - shadow).clamp(0, 32), rim.min(32))
                    } else {
                        // back one slice, to the row before: how far along the ray crossed the seabed
                        let (xp, yp) = (x - qx[i], y - qy[i] - 100 * drop);
                        let hp = smooth(&map.rows[j - 1], xp);
                        let hs = smooth(row, x);
                        let ep = (yp.div_euclid(10) - hp).min(99);
                        let ec = y.div_euclid(10) - hs;
                        assert!(ec < 0);
                        let r = if ep <= 0 { 0 } else { 10 * ep / (ep - ec) };
                        let xh = xp + (qx[i] * r).div_euclid(10);
                        // the slope ahead, at the hit's x: the row before against the hit's row
                        let hf = smooth(&map.rows[j - 1], x);
                        let slope = (hf - hs + 10 * drop).clamp(-10, 10);
                        let m = ((if r < 5 { hp } else { hs }) + 20).clamp(0, 64) * 5 / 10;
                        let (tone, vary) = strata(x.div_euclid(100) + 100, j as i32, map.base[j], 10 * hs, true);
                        (r, xh, m, tone, (20 + slope * 12 / 10 + vary).clamp(0, 32), 0)
                    };
                    let n = 10 * (k - 1) + s0 + r;
                    let d = if n <= 0 { 0 } else { 30 * n / d3[i][2] };
                    let mut g = mix32c([0; 3], mix32c(sand, rocks[tone as usize], m), bf);
                    // the diver's lamp, a beam along the view: near rock toward its colour in full light, the more the darker the water
                    let (ax, ay) = ((2 * px as i32 - 191 - aim[0]).abs(), (2 * py as i32 - 95 + aim[1]).abs());
                    let beam = (40 - ax.max(ay) - ax.min(ay) / 2).clamp(0, 32);
                    let lamp = (P_LAMP - d).clamp(0, P_LAMPMAX) * beam / 32 * (32 - light) / 32;
                    g = mix32c(g, pale, rim);
                    // caustics: three waves across the seabed, bright where they nearly cancel
                    let (xu, xt) = (xh.div_euclid(100), xh.rem_euclid(100) / 10);
                    let jm = j as i32 - 1;
                    let i1 = ((10 * xu.rem_euclid(5) + xt) * 12 / 10 + t1) % 60;
                    let i2 = ((10 * jm.rem_euclid(6) + r) % 60 + t2) % 60;
                    let i3 = ((10 * (xu + jm).rem_euclid(15) + xt + r) % 75 * 8 / 10 + t3) % 60;
                    let s = (sin60[i1 as usize] + sin60[i2 as usize] + sin60[i3 as usize]).abs();
                    let glint = if s < 16 { (16 - s) * (16 - s) / 10 * 8 / 10 * light10 / 10 } else { 0 };
                    let ck = glint * (25 - (d / 2).min(25)) * 4 / 100;
                    g = mix32c(g, cc, ck);
                    // a glowing spot in the middle of the unit square under the crossing
                    let code = map.spots[j - 1].iter().filter(|s| s.0 == xu).map(|s| s.1).last().unwrap_or(0);
                    if code > 0 {
                        let (ax, az) = ((xt - 5).abs(), (r - 5).abs());
                        let fall = GLOW_FALLOFF[((ax * ax + az * az) / 5) as usize];
                        let step = (pulse + 6 * (code % 10)) % 60;
                        let by = |a: i32, b: i32, d: i32| a * b / d;
                        let amount = by(by(fall, 20 + sin60[step as usize] + 10, 50), dark * 3, 128).min(19);
                        g = mix32c(g, if code / 10 == 2 { [160, 90, 200] } else { [60, 200, 160] }, amount);
                    }
                    // the band of glow on a lip
                    let away = (10 * map.lip[j - 1] + r).abs();
                    if away < 15 { g = mix32c(g, [70, 170, 160], (15 - away) * (32 - light) / 27); }
                    depth[i] = d;
                    crossed[i] = r;
                    // the diver's lamp last, so it cuts through the fog
                    mix32c(mix32c(g, far, fog32(d)), mix32c([0; 3], mix32c(U_SAND_LIT, U_ROCKS[tone as usize], m), bf), lamp)
                }
            };
        } }
        // kelp, seagrass, fish (their numbers follow the jellyfish's in the script)
        let mut at = 47 + 16 * (sc[46] as usize);
        let mut next = || { let v = sc[at]; at += 1; v };
        let (glow_tips, height10, fish_light, nk) = (next(), next(), next(), next());
        let lit = |lo: [i32; 3], hi: [i32; 3]| mix32c(lo, hi, light);
        let (stalk0, stalk1) = (lit([4, 4, 2], [70, 62, 30]), lit([5, 5, 2], [90, 80, 36]));
        let (blade0, blade1) = (lit([6, 6, 2], [96, 92, 34]), lit([8, 9, 3], [132, 150, 56]));
        let (bladder, rimc) = (lit([10, 10, 4], [170, 160, 70]), lit([10, 12, 6], [170, 200, 100]));
        let (tip0, tip1) = (lit([4, 12, 8], [60, 200, 140]), lit([8, 15, 11], [140, 255, 180]));
        let (grass0, grass1) = (lit([4, 5, 2], [60, 90, 40]), lit([7, 9, 4], [110, 150, 60]));
        for _ in 0..nk {
            let (mx, mxp, row, rx, rz) = (next(), next(), next(), next(), next());
            let stalks: Vec<(i32, i32, i32, Vec<[i32; 2]>)> = (0..3).map(|_| { let (l, t, ph) = (next(), next(), next()); (l, t, ph, (0..15).map(|_| [next(), next()]).collect()) }).collect();
            let base = 10 * map.base[row as usize] + sample(&map.rows[row as usize], 100 * mx + mxp, row - j0 as i32) - height10;
            let root = [rx, base, rz];
            let d = along(root, f100);
            if d < 20 { continue; }
            let scale = 100 * 100 / d;
            let fog = KELP_FOG[(d / 10).min(49) as usize];
            for (lean, tall, phase, blades) in &stalks {
                // a point up the stalk, part p of 990 of the way: its height, lean and sway
                let point = |p: i32| -> Option<(i32, i32)> {
                    let h = hundredth(*tall, p / 10);
                    let step = (phase + 120 - hundredth(h, 33)) % 60;
                    let sway = hundredth(h, sin60[step as usize]) * 7 / 10;
                    see_fine([root[0] + hundredth(p, *lean) / 10 + sway, root[1] + h, root[2]], f100, r100, u100).map(|q| (q.0, q.1))
                };
                let pts: Vec<Option<(i32, i32)>> = (0..=22).map(|i| point(45 * i)).collect();
                for i in 0..22 {
                    if let (Some(a), Some(b)) = (pts[i], pts[i + 1]) { leaf(&mut rgb, &mut depth, a, b, hundredth(scale, 7), false, d, stalk0, stalk1, rimc, far, fog); }
                }
                for (i, bl) in blades.iter().enumerate() {
                    let Some(p) = point(66 * (i as i32 + 1) - 20) else { continue };
                    let length = 2 * hundredth(scale, bl[1]);
                    let tip = (p.0 + hundredth(length, cos64(bl[0])), p.1 - hundredth(length, cos64(bl[0] - 16)));
                    if tip.0.abs() > 999 || tip.1.abs() > 999 { continue; }
                    leaf(&mut rgb, &mut depth, p, tip, hundredth(scale, 22), true, d, blade0, blade1, rimc, far, fog);
                    let colour = mix32c(bladder, far, fog);
                    for k in disc(p, hundredth(scale, 9).max(5)) { if depth[k].min(99) * 10 >= d { rgb[k] = colour; depth[k] = (d + 9) / 10; } }
                    if glow_tips == 1 && i == 14 { leaf(&mut rgb, &mut depth, p, tip, hundredth(scale, 22), true, d, tip0, tip1, rimc, far, fog); }
                }
            }
        }
        let ng = next();
        for _ in 0..ng {
            let (mx, mxp, row, rx, rz, phase) = (next(), next(), next(), next(), next(), next());
            let heights: Vec<i32> = (0..5).map(|_| next()).collect();
            let base = 10 * map.base[row as usize] + sample(&map.rows[row as usize], 100 * mx + mxp, row - j0 as i32) - height10;
            for b in 0..5 {
                let off = b - 2;
                let sway = hundredth(sin60[((phase + 10 * b) % 60) as usize], 8);
                let (Some(a), Some(t)) = (see_fine([rx + off, base, rz], f100, r100, u100), see_fine([rx + 2 * off + sway, base + heights[b as usize], rz], f100, r100, u100)) else { continue };
                if a.2 > 220 { continue; }
                let scale = 100 * 100 / a.2;
                leaf(&mut rgb, &mut depth, (a.0, a.1), (t.0, t.1), hundredth(scale, 5), true, a.2, grass0, grass1, rimc, far, KELP_FOG[(a.2 / 10).min(49) as usize]);
            }
        }
        // the picture so far and its depths: the water the creatures are drawn over at 1536x768
        let (under, under_depth) = (rgb.clone(), depth.clone());
        let nf = next();
        let mut fish_hd = vec![];
        let (dim, bright) = (mix32c([70, 82, 88], [140, 165, 175], fish_light), mix32c([120, 124, 125], [240, 248, 250], fish_light));
        for _ in 0..nf {
            let head = [next(), next(), next()];
            let tail = [next(), next(), next()];
            let flash = next();
            let id = next();
            fish_hd.push((head.map(|v| v as i64), tail.map(|v| v as i64), flash as i64, id as i64));
            let (Some(a), Some(b)) = (see_fine(head, f100, r100, u100), see_fine(tail, f100, r100, u100)) else { continue };
            let d = a.2;
            if d < 10 { continue; }
            let colour = mix32c(mix32c(dim, bright, flash), far, FISH_FOG[(d / 10).min(49) as usize]);
            let steps = ((2 * (b.0 - a.0).abs().max((b.1 - a.1).abs()) + 9) / 10).max(1);
            for st in 0..=steps {
                let (x, y) = (a.0 + (b.0 - a.0) * st / steps, a.1 + (b.1 - a.1) * st / steps);
                let (c, r) = (x.div_euclid(10), (-y).div_euclid(10));
                for rr in r..=r + if d < 140 { 1 } else { 0 } {
                    if let Some(i) = pixel(c, rr) { if depth[i].min(99) * 10 >= d { rgb[i] = colour; depth[i] = (d + 9) / 10; } }
                }
            }
        }
        // marine snow: three copies of the box along, each speck mixed into its pixel, fading out by 40 ahead
        for sp in &specks {
            let y = (sp[1] - snow_up + 70).rem_euclid(140) - 70;
            let z = (sp[2] - snow_along + 20).rem_euclid(140) - 20;
            for copy in 0..3 {
                let Some((sx, sy, ahead)) = see([sp[0], y, z + 140 * copy], f100, r100, u100) else { continue };
                if sx < 0 || sy < 0 || sx >= W as i32 || sy >= H as i32 { continue; }
                let i = sy as usize * W + sx as usize;
                let a = ahead / 10;
                if a < 40 && a <= depth[i] { rgb[i] = mix32c(rgb[i], [170, 200, 200], (40 - a) * show / 40); }
            }
        }
        // jellyfish, far ones first: a halo, a bell of points (brighter at the edge, dimmer seen from inside) and
        // eight waving tentacles
        let (jlight, wave, nj) = (sc[44], sc[45], sc[46] as usize);
        let mut list: Vec<(i32, i32, Vec<i32>)> = (0..nj).filter_map(|n| {
            let j = sc[47 + 16 * n..47 + 16 * (n + 1)].to_vec();
            let ahead = along([j[0], j[1], j[2]], f100);
            if ahead <= 10 || ahead >= 600 { return None; }
            let fade = JELLY_FADE[(ahead / 20) as usize] * jlight / 32;
            if fade == 0 { return None; }
            Some((ahead, fade, j))
        }).collect();
        list.sort_by(|a, b| b.0.cmp(&a.0));
        let put = |rgb: &mut Vec<[i32; 3]>, x: i32, y: i32, ahead: i32, c: [i32; 3], k: i32| {
            if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 { return; }
            let i = y as usize * W + x as usize;
            if depth[i].min(99) * 10 < ahead - 20 { return; }
            rgb[i] = mix32c(rgb[i], c, k);
        };
        for (ahead, fade, j) in &list {
            let (ahead, fade) = (*ahead, *fade);
            let c = [j[0], j[1], j[2]];
            let (turn, rad, ht) = (j[3], j[5], j[6]);
            let glow = if j[4] == 1 { [220, 150, 255] } else { [140, 200, 255] };
            let centre = see_fine(c, f100, r100, u100);
            // the halo round the middle's place on the screen, a fall off by the squared distance
            if let Some((cx, cy, _)) = centre {
                let sigma = (100 * 140 / ahead).max(25);
                let reach = (100 * 30 / ahead).max(5);
                let (cx10, cy10) = (960 + cx, 480 - cy);
                for y in (cy10.div_euclid(10) - reach).max(0)..(cy10.div_euclid(10) + reach).min(H as i32) {
                    for x in (cx10.div_euclid(10) - reach).max(0)..(cx10.div_euclid(10) + reach).min(W as i32) {
                        let i = y as usize * W + x as usize;
                        let (u, v) = (100 * (10 * x + 5 - cx10) / sigma / 10, 100 * (10 * y + 5 - cy10) / sigma / 10);
                        let q = (u * u + v * v) / 20;
                        if q < 49 && depth[i].min(99) * 10 >= ahead - 20 { rgb[i] = mix32c(rgb[i], glow, JELLY_HALO[q as usize] * fade / 64); }
                    }
                }
            }
            // which way the camera is from the jellyfish, in hundredths: its place over its length, the length the
            // larger of two parts and a table's share of it, across with along and then with up
            let length = magnitude(magnitude(c[0].abs(), c[2].abs()), c[1].abs());
            let view: Vec<i32> = c.iter().map(|&v| -100 * v / length).collect();
            // the bell: twelve rings of 32 points over a flattened dome
            for ring in 0..12 {
                let rs = hundredth(rad, BELL_SIN[ring]);
                let hc = hundredth(ht, BELL_COS[ring]);
                // the ring's surface direction, (sin, cos rad / ht) over its length
                let (nx, ny) = (BELL_SIN[ring], 100 * hundredth(rad, BELL_COS[ring]) / ht);
                let length = magnitude(nx, ny);
                let (a, b) = (100 * nx / length, 100 * ny / length);
                for seg in 0..32 {
                    let ph = 2 * seg + turn;
                    let (cs, sn) = (cos64(ph), cos64(ph - 16));
                    let p = [c[0] + thousandth(rs, cs), c[1] + hundredth(hc, 10), c[2] + thousandth(rs, sn)];
                    let side = hundredth(cs, view[0]) + hundredth(sn, view[2]);
                    let facing = hundredth(side, a) + hundredth(view[1], b);
                    let side_on = (100 - facing.abs()).max(0);
                    let edge = ((side_on / 10 * 32 + side_on % 10 * 32 / 10) / 10).min(31);
                    let mut bright = BELL_GLOW[edge as usize] + if seg % 4 == 0 { 11 } else { 0 };
                    if facing < 0 { bright = bright * 5 / 10 + bright * 5 / 100; }
                    let colour = mix32c(glow, [230, 245, 255], edge * 6 / 10);
                    let alpha = ((bright + bright * 5 / 10) * fade / 64).min(29);
                    let Some((sx, sy, pd)) = see_fine(p, f100, r100, u100) else { continue };
                    let r = (100 * 9 / pd).clamp(5, 31);
                    let (x10, y10) = (960 + sx, 480 - sy);
                    for yy in (y10 - r).div_euclid(10)..=(y10 + r + 9).div_euclid(10) {
                        for xx in (x10 - r).div_euclid(10)..=(x10 + r + 9).div_euclid(10) {
                            let (dx, dy) = (10 * xx + 5 - x10, 10 * yy + 5 - y10);
                            if dx.abs() > r || dy.abs() > r || dx * dx > r * r - dy * dy { continue; }
                            put(&mut rgb, xx, yy, pd, colour, alpha);
                        }
                    }
                }
            }
            // tentacles: from the rim, waving, drawn in 3D as lines of dots
            for tn in 0..8 {
                let ph = 8 * tn + turn;
                let (cs, sn) = (cos64(ph), cos64(ph - 16));
                let rr = hundredth(rad, 90);
                let mut prev: Option<(i32, i32, i32)> = None;
                for seg in 0..=12 {
                    let w = sin60[((4 * seg + 19 * tn / 2 + wave) % 60) as usize] * seg / 100;
                    let p = [c[0] + thousandth(rr, cs) + hundredth(sn, w), c[1] - seg * j[7 + tn as usize] / 12, c[2] + thousandth(rr, sn) - hundredth(cs, w)];
                    let q = see_fine(p, f100, r100, u100);
                    if let (Some(a0), Some(b0)) = (prev, q) {
                        let (ax, ay, bx, by) = (960 + a0.0, 480 - a0.1, 960 + b0.0, 480 - b0.1);
                        let steps = ((2 * (bx - ax).abs().max((by - ay).abs()) + 9) / 10).max(1);
                        let alpha = ((15 - seg) * 29 / 15 * fade / 64).min(26);
                        for st in 0..=steps {
                            let (x, y, pd) = (ax + (bx - ax) * st / steps, ay + (by - ay) * st / steps, a0.2 + (b0.2 - a0.2) * st / steps);
                            put(&mut rgb, x.div_euclid(10), y.div_euclid(10), pd, glow, alpha);
                        }
                    }
                    prev = q;
                }
            }
        }
        // what is laid over everything, per pixel and in order: the caption, the portal and the whiteout, mixes of a
        // colour by 32nds (or 256ths); on this picture, and on the creatures' at 1536x768
        let mut fin: Vec<Vec<(bool, [i32; 3], i32)>> = vec![vec![]; W * H];
        // the caption: TOTALLY WORTH IT, small at the top left on a black strip, faded in and out (sixteenths)
        if fade >= 0 {
            let text: Vec<char> = " TOTALLY WORTH IT".chars().collect();
            for y in 0..7 { for x in 0..69usize {
                let (ch, cx) = (text.get(x / 4).copied().unwrap_or(' '), x % 4);
                let on = y >= 1 && y <= 5 && cx < 3 && font[&ch][y - 1] >> (2 - cx) & 1 == 1;
                fin[y * W + x].push((false, if on { [255; 3] } else { [0; 3] }, 2 * fade));
            } }
        }
        // the portal: where each ray crosses its plane, how far and which way from its middle
        if on > 0 {
            for i in 0..W * H {
                let l: Vec<i32> = (0..2).map(|c| {
                    let q = if c == 0 { qx[i] } else { qy[i] };
                    let v = from[c] + (plane % 10 * q).div_euclid(10) + plane / 10 * q;
                    if v.div_euclid(100).abs() > 30 { 400 } else { v.div_euclid(10) }
                }).collect();
                if l[0] == 400 || l[1] == 400 { continue; }
                if let Px::Hit(k, _, _) = res[i] { if k <= 50 && plane > 10 * k - 10 + s0 + crossed[i] { continue; } }
                let (large, small) = (l[0].abs().max(l[1].abs()), l[0].abs().min(l[1].abs()));
                let ratio = if large == 0 { 0 } else { 32 * small / large };
                let sh = PORTAL_RADIUS[ratio as usize];
                let far = large + (((large % 10) * sh / 10 + (large / 10 % 10) * sh) / 10 + (large / 100) * sh) / 2;
                let mut way = PORTAL_ANGLE[ratio as usize];
                if l[1].abs() > l[0].abs() { way = 150 - way; }
                if l[0] < 0 { way = 300 - way; }
                if l[1] < 0 && way > 0 { way = 600 - way; }
                let reach = far.min(60);
                let s1 = (3 * way + reach * 12 + reach * 4 / 10 + swirl[0]) % 600 / 10;
                let s2 = (2 * way + swirl[1]) % 600 / 10;
                let (v1, v2) = (20 + sin60[s1 as usize], 20 + sin60[s2 as usize]);
                let turn = v1 * (90 + reach) / 188;
                let colour = mix32c([20, 60, 120], mix32c([90, 240, 255], [200, 120, 255], v2 * 8 / 10), turn);
                let f = far - 60;
                if f < 0 { fin[i].push((false, colour, on * (18 + turn * 11 / 32) / 32)); }
                if f.abs() < 5 { fin[i].push((false, [220, 250, 255], on * (9 - 2 * f.abs()) / 9)); }
                if (0..=109).contains(&f) {
                    // the spill between the table's units by a straight line (to nothing past the last), in tenths of
                    // 32nds; mixed in 256ths
                    let (at, next) = (PORTAL_SPILL[(f / 10) as usize], if f < 100 { PORTAL_SPILL[(f / 10 + 1) as usize] } else { 0 });
                    let tenths = 10 * at + (next - at) * (f % 10);
                    fin[i].push((true, [80, 200, 240], on * tenths / 40));
                }
            }
        }
        // the whiteout, as the diver passes through
        if white32 > 0 { for f in fin.iter_mut() { f.push((false, [255; 3], white32)); } }
        let finish = |c: [i64; 3], ops: &[(bool, [i32; 3], i32)]| -> [i64; 3] {
            let mut c = c;
            for &(fine, to, k) in ops { for ch in 0..3 { c[ch] = if fine { mix256(c[ch] as i32, to[ch], k) as i64 } else { hd::mix32(c[ch], to[ch] as i64, k as i64) }; } }
            c
        };
        for i in 0..W * H { let c = finish(rgb[i].map(|v| v as i64), &fin[i]); rgb[i] = c.map(|v| v as i32); }
        // every channel exactly as worked out
        for c in &rgb { for &v in c { assert!((0..=255).contains(&v), "a channel out of 0..255: {}", v); } }
        out.write_all(&rgb.iter().flat_map(|c| c.map(|v| v as u8)).collect::<Vec<u8>>()).unwrap();
        // the creatures at 1536x768
        let scene = hd::Scene { n: n as i64, cam: [cxh as i64, 100 * cyu as i64 + cyf as i64, 100 * czu as i64 + 10 * czf as i64], f: f100.map(|v| v as i64), r: r100.map(|v| v as i64), u: u100.map(|v| v as i64),
            fish: fish_hd, fish_light: fish_light as i64, far: far.map(|v| v as i64), show: show as i64, snow: [snow_up as i64, snow_along as i64],
            specks: &specks, jellies: list.iter().map(|(a, fd, j)| (*a as i64, *fd as i64, j.iter().map(|&v| v as i64).collect(), first[&j[15]])).collect(),
            wave: wave as i64, sin60: &sin60, under: &under, depth: &under_depth };
        hd::frame(&scene, &hd_data, &trig, &mut hd_rgb);
        for y in 0..hd::HH { for x in 0..hd::HW { let i = y * hd::HW + x; hd_rgb[i] = finish(hd_rgb[i], &fin[(y / 8) * W + x / 8]); } }
        // the ending: the knob
        let ending = knob::Scene { n: n as i64, cam: [cxh as i64, 100 * cyu as i64 + cyf as i64, 100 * czu as i64 + 10 * czf as i64],
            f: f.map(|v| v as i64), r: r.map(|v| v as i64), u: u.map(|v| v as i64), far: far.map(|v| v as i64), depth: &under_depth, haze70: &hd_data.orca_haze };
        knob::frame(&ending, &knob_data, &trig, &hd_data.orcas, &mut hd_rgb);
        for c in &hd_rgb { for &v in c { assert!((0..=255).contains(&v), "an HD channel out of 0..255: {}", v); hd_frames.push(v as u8); } }
    }
    out.write_all(&hd_frames).unwrap();
}
