// The reference's Boom, Wikipedia sphere and Finnish flag: the pictures the cold open's first windows show, in ordinary Rust from the
// formulas, whole numbers only, every division rounded as the machines round it (fl and md: down, and the remainder
// that goes with it). See reference.rs for the rest of the cold open.
//
// The game (each frame f, 448 x 224 over the 47-row status bar), units texels (64 a cell):
// - The camera: smooth(x) = fl(fl(x^2, 10) (3000 - 2x), 100000) for x thousandths held to 0..1000; lerp from a to b
//   by u thousandths a + fl((b - a) u, 1000). Down the corridor (f <= 15): x = X0 + fl((XD - X0)(2 S - 1000), 1000),
//   S = smooth(500 + fl(500 f, 15)); at the door (to 21) x = XD; through it (to 36), u = fl(1000 (f - 21), 15):
//   x = XD + fl((XR - XD) 2 smooth(fl(u, 2)), 1000), the angle fl(A_ROOM u, 1000); round the room, U =
//   smooth(fl(1000 (f - 36), 24)): x, y and the angle lerped by U. The door up by smooth(fl(1000 (f - 15), 10)).
// - The view's way: 999 cos and sin of the angle (the swarm's table, between by tenths), ux = fl(448 cos, 999), uy =
//   fl(448 sin, 999); column c's ray (448ths) rdx = ux + fl(-uy k, 560), rdy = uy + fl(ux k, 560), k = 2c - 447: the
//   screen 0.8 wide each side.
// - Each ray's walk over the map: across each line x = a cell's edge in turn (the first fx or 64 - fx texels on, then
//   64 at a time), how far it has gone the other way, fl(along |rdy|, |rdx|), and the cell it enters and the texel
//   it crosses at (taking a texel less going up or left when the crossing isn't exactly on one); and the same for
//   the lines y = an edge. The lists stop where a number would pass 998 or the map ends.
// - The wall: the first wall in each list; the x line's is nearer when its way across is less than the y line's along.
//   The door: shut (the door up by 0), a wall; open (up by 1000), not there; between, the first door before the wall,
//   drawn over it. The texture's column: the texel crossed at, 63 less it seen from the other side.
// - A wall's rows: s = fl(400 along, |rd|), 250ths of a texel a row; row y shows the texel fl(W, 500), W = 16000 +
//   (2y - 223) s (+ 32 x the door's height for the door), from the first row W is not below 0 to the row it reaches
//   64. Its light L = 155 - fl(7 s, 44) held to 10..100, four fifths of it on a y line's wall, 27/20 of it in a muzzle
//   flash.
// - The floor and ceiling, rows y and 223 - y alike, m = 2y - 223: across the row the texel's place in thousandths
//   starts at n / 14m texels on from the camera, n = 560 ux + 447 uy, wrapped at 64: md(1000 fx + 1000 md(fl(n, 14m),
//   64) + fl(1000 md(n, 14m), 14m), 64000), and steps md(fl(-1000 uy, 7m), 64000) a column (the same down, from 560 uy
//   - 447 ux, by 1000 ux); its light 155 - fl(2545, m) held to 10..100.
// - A texel of grey g (256 white) under light E is black when g is at most the table's fl((2k + 1) 800, E) for its
//   pixel's step k of the 4x4 ordered dither: the prototype's g E <= the step's middle.
// - The gun's bob: 999 sin and cos of the phase PHASE f; across fl(12 bob sin, 9990), down fl(7 bob |cos|, 9990)
//   (bob in tenths); the gun at (GUN_X + across, GUN_Y + down - kick), kept to the view. The counters: the ammo 50 less
//   the shots, its tens and ones.
// - The walls behind (the hall of mirrors): the same view from the same place, its angle 500000 millionths on, never
//   in a muzzle flash.
// The prototype works in floating point and rounds to the nearest; the walls' places and the textures' texels agree to
// a texel or a row, the dots of the dither to the step.
//
// The Wikipedia page's sphere (radius 70, its picture 141 square, centred): a pixel (x, y) is on it when x^2 + y^2 <=
// 70^2; its light, in 7000ths, -50 x - 60 y + 62 s, s the whole square root of 70^2 - x^2 - y^2 (the prototype's -0.5
// nx - 0.6 ny + 0.62 nz); black where that is under ceil((70000 (2k + 1) - 179200) / 288) (the prototype's light held
// to 0..1, times 0.9, plus 0.08, under the step's middle), k the step of the 4x4 dither at twice the size.
//
// The Finnish flag: 60 narrower than its window's inside, fl(11 w, 18) high, cropped to 8 above the window's foot; the
// cross's bands from ceil(5 w / 18) to before ceil(8 w / 18) across and ceil(4 h / 11) to before ceil(7 h / 11) down;
// its pixels black on its edge, and in the cross on every even row and the odd columns of the odd rows.
#![allow(dead_code)]
use super::boom::*;

pub fn fl(a: i64, b: i64) -> i64 { a.div_euclid(b) }
pub fn md(a: i64, b: i64) -> i64 { a.rem_euclid(b) }

/// The swarm's printed table of 999 sin every thousandth of a turn (a quarter turn), between by tenths.
pub struct Trig { trig: Vec<i64> }
impl Trig {
    pub fn read(demo: &std::path::Path) -> Trig {
        let text = std::fs::read_to_string(demo.join("swarm/data/tables.txt")).unwrap();
        let l = text.lines().find(|l| l.split_whitespace().next() == Some("trig")).unwrap();
        Trig { trig: l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect() }
    }
    pub fn sin(&self, a: i64) -> i64 {
        let t = fl(md(a, 1_000_000), 100);
        let q = t / 2500;
        let mut r = t % 2500;
        if q % 2 == 1 { r = 2500 - r; }
        let (k, fr) = ((r / 10) as usize, r % 10);
        let v = if fr == 0 { self.trig[k] } else { self.trig[k] + (self.trig[k + 1] - self.trig[k]) * fr / 10 };
        if q >= 2 { -v } else { v }
    }
    pub fn cos(&self, a: i64) -> i64 { self.sin(a + 250_000) }
}

pub fn smooth(x: i64) -> i64 {
    let x = x.clamp(0, 1000);
    fl(fl(x * x, 10) * (3000 - 2 * x), 100_000)
}
fn lerp(a: i64, b: i64, u: i64) -> i64 { a + fl((b - a) * u, 1000) }

/// The camera at frame f: its place, its angle (millionths of a turn), how far the door is up (thousandths).
#[derive(Clone, Copy)]
pub struct Cam { pub x: i64, pub y: i64, pub angle: i64, pub door: i64 }
pub fn camera(f: usize) -> Cam {
    let fi = f as i64;
    let (x, y, angle) = match leg(f) {
        Leg::Run => { let s = smooth(500 + fl(500 * fi, 15)); (X0 + fl((XD - X0) * (2 * s - 1000), 1000), Y0, 0) }
        Leg::Wait => (XD, Y0, 0),
        Leg::Through => {
            let u = fl(1000 * (fi - 21), 15);
            let s = smooth(fl(u, 2));
            (XD + fl((XR - XD) * 2 * s, 1000), Y0, fl(A_ROOM * u, 1000))
        }
        Leg::Turn => { let u = smooth(fl(1000 * (fi - 36), 24)); (lerp(XR, XE, u), lerp(Y0, YE, u), lerp(A_ROOM, A_END, u)) }
    };
    Cam { x, y, angle, door: smooth(fl(1000 * (fi - DOOR_AT), DOOR_FRAMES)) }
}

/// One crossing of a ray with a grid line: the map's cell entered, how far the ray has gone the other way, the texel
/// crossed at (along the line), how far along.
#[derive(Clone, Copy, Debug)]
pub struct Cross { pub cell: i64, pub travel: i64, pub tex: i64, pub along: i64 }

/// The crossings with the lines across one way: the first `first` texels on, then 64 at a time; `a` the ray's part
/// this way and `b` the other's (both sizes), `pos` the camera's place the other way and `up` whether the ray goes
/// back that way; `cell(k, other)` the map's cell entered at the k-th line, the other way's cell given.
pub fn crossings(first: i64, a: i64, b: i64, pos: i64, up: bool, cell: &dyn Fn(i64, i64) -> Option<i64>) -> Vec<Cross> {
    let mut v = vec![];
    if a == 0 { return v; }
    for k in 0.. {
        let along = first + 64 * k;
        let (travel, rem) = (fl(along * b, a), md(along * b, a));
        if along >= 999 || travel >= 999 { break; }
        let h = if up { pos - travel - (rem > 0) as i64 } else { pos + travel };
        match cell(k, fl(h, CELL)) {
            Some(m) => v.push(Cross { cell: m, travel, tex: md(h, CELL), along }),
            None => break,
        }
    }
    v
}

/// What a column's ray meets: the wall (its texture, 0 an x line or 1 a y line, how far along, the ray's part that
/// way, the texture's column) and the door in front of it, if drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit { pub kind: i64, pub side: i64, pub along: i64, pub d: i64, pub u: i64 }

pub fn ray(data: &Data, cam: &Cam, rdx: i64, rdy: i64) -> (Hit, Option<Hit>) {
    let (mx, fx, my, fy) = (fl(cam.x, CELL), md(cam.x, CELL), fl(cam.y, CELL), md(cam.y, CELL));
    let (sx, sy) = (if rdx < 0 { -1 } else { 1 }, if rdy < 0 { -1 } else { 1 });
    let (ax, ay) = (rdx.abs(), rdy.abs());
    let map = |col: i64, row: i64| -> Option<i64> {
        if col < 0 || row < 0 || col >= MAP_W as i64 || row >= MAP_H as i64 { None } else { Some(data.map[row as usize][col as usize]) }
    };
    let xs = crossings(if sx > 0 { 64 - fx } else { fx }, ax, ay, cam.y, rdy < 0, &|k, row| map(mx + sx * (k + 1), row));
    let ys = crossings(if sy > 0 { 64 - fy } else { fy }, ay, ax, cam.x, rdx < 0, &|k, col| map(col, my + sy * (k + 1)));
    // the door: shut, a wall; all the way up, not there; between, drawn over what's behind it
    let (shut, open) = (cam.door == 0, cam.door == 1000);
    let wall = |m: i64| m != 0 && (m != DOOR || shut);
    let door = |m: i64| m == DOOR && !shut && !open;
    let first = |v: &[Cross], f: &dyn Fn(i64) -> bool| v.iter().position(|c| f(c.cell));
    let (xw, yw) = (first(&xs, &wall), first(&ys, &wall));
    // the x line's crossing is nearer when its way across is less than the y line's along
    let x_nearer = |x: &Cross, y: &Cross| x.travel < y.along;
    let hit = |side: i64, c: &Cross| {
        let flip = if side == 0 { rdx < 0 } else { rdy > 0 };
        Hit { kind: c.cell, side, along: c.along, d: if side == 0 { ax } else { ay }, u: if flip { 63 - c.tex } else { c.tex } }
    };
    let w = match (xw, yw) {
        (Some(i), Some(j)) => if x_nearer(&xs[i], &ys[j]) { hit(0, &xs[i]) } else { hit(1, &ys[j]) },
        (Some(i), None) => hit(0, &xs[i]),
        (None, Some(j)) => hit(1, &ys[j]),
        (None, None) => panic!("a ray met no wall"),
    };
    let before = |v: &[Cross], w: Option<usize>| v.iter().position(|c| door(c.cell)).filter(|&i| w.is_none_or(|w| i < w));
    let (xd, yd) = (before(&xs, xw), before(&ys, yw));
    let d = match (xd, yd) {
        (Some(i), Some(j)) => Some(if x_nearer(&xs[i], &ys[j]) { (0, i) } else { (1, j) }),
        (Some(i), None) => Some((0, i)),
        (None, Some(j)) => Some((1, j)),
        (None, None) => None,
    };
    // the door is drawn when it is nearer than the wall (across the two lists, the same test)
    let d = d.filter(|&(side, i)| match (side, w.side) {
        (0, 1) => x_nearer(&xs[i], &ys[yw.unwrap()]),
        (1, 0) => !x_nearer(&xs[xw.unwrap()], &ys[i]),
        _ => true,
    });
    (w, d.map(|(side, i)| hit(side, if side == 0 { &xs[i] } else { &ys[i] })))
}

/// A wall's light (hundredths) at its scale s, for its side, in a muzzle flash or not.
pub fn wall_light(s: i64, side: i64, bright: bool) -> i64 {
    let l = (155 - fl(7 * s, 44)).clamp(10, 100);
    let e = if side == 1 { fl(4 * l, 5) } else { l };
    if bright { fl(27 * e, 20) } else { e }
}

/// A wall's column: its rows from the first, each black or white. `lift`: the door's height (thousandths), 0 for a wall.
pub fn wall_rows(data: &Data, h: &Hit, x: usize, lift: i64, bright: bool) -> (usize, Vec<bool>) {
    let s = fl(400 * h.along, h.d);
    let e = wall_light(s, h.side, bright);
    let top = if s == 0 { 0 } else { (-fl(16000 - 223 * s, 2 * s)).max(0) };
    let tex = &data.tex[h.kind as usize];
    let mut rows = vec![];
    let mut y = top;
    while y < VH as i64 {
        let w = 16000 + (2 * y - 223) * s + 32 * lift;
        let texel = fl(w, 500);
        if texel >= 64 { break; }
        let g = tex[texel as usize * 64 + h.u as usize];
        rows.push(g <= data.thresh[e as usize][BAYER[y as usize % 4][x % 4]]);
        y += 1;
    }
    (top as usize, rows)
}

/// The flats' places across a row-pair m = |2y - 223|: where the texel's place (thousandths, across and down) starts
/// and how far it steps a column; the light.
pub struct Flat { pub start: [i64; 2], pub step: [i64; 2], pub light: i64 }
pub fn flat(cam: &Cam, ux: i64, uy: i64, m: i64) -> Flat {
    let (fx, fy) = (md(cam.x, CELL), md(cam.y, CELL));
    // n / 14m in thousandths, wrapped at 64 texels: its whole part's last 64 and its remainder's thousandths
    let start = |f: i64, n: i64| md(1000 * f + 1000 * md(fl(n, 14 * m), 64) + fl(1000 * md(n, 14 * m), 14 * m), 64_000);
    Flat {
        start: [start(fx, 560 * ux + 447 * uy), start(fy, 560 * uy - 447 * ux)],
        step: [md(fl(-1000 * uy, 7 * m), 64_000), md(fl(1000 * ux, 7 * m), 64_000)],
        light: (155 - fl(2545, m)).clamp(10, 100),
    }
}

/// The game's frame f: the view and the status bar, black true; the gun's sprite and the faces from the caller.
pub struct Game { pub px: Vec<bool> }
pub fn game_view(data: &Data, trig: &Trig, f: usize) -> (Vec<bool>, (i64, i64)) {
    let cam = camera(f);
    let (kick, flash) = kick_flash(f);
    let px = view(data, trig, &cam, flash == 2);
    // the gun's place: its bob and kick
    let phase = PHASE * f as i64;
    let (s, c) = (trig.sin(phase), trig.cos(phase));
    let across = fl(12 * bob(f) * s, 9990);
    let down = fl(7 * bob(f) * c.abs(), 9990);
    (px, (GUN_X + across, GUN_Y + down - kick))
}

/// The walls behind the player at frame f, as the hall of mirrors draws them over the view: the view from the same
/// place half a turn round (500000 millionths), never in a muzzle flash.
pub fn behind_view(data: &Data, trig: &Trig, f: usize) -> Vec<bool> {
    let cam = camera(f);
    view(data, trig, &Cam { angle: cam.angle + 500_000, ..cam }, false)
}

/// The view (448 x 224) from the camera: the flats, the walls over them, the door over its wall.
pub fn view(data: &Data, trig: &Trig, cam: &Cam, bright: bool) -> Vec<bool> {
    let cam = *cam;
    let (ux, uy) = (fl(448 * trig.cos(cam.angle), 999), fl(448 * trig.sin(cam.angle), 999));
    let mut px = vec![false; VW * VH];
    // the floor and the ceiling, row by row
    for y in 0..VH {
        let m = (2 * y as i64 - 223).abs();
        let fl_ = flat(&cam, ux, uy, m);
        let e = if bright { fl(27 * fl_.light, 20) } else { fl_.light };
        let tex = &data.tex[if y >= VH / 2 { 6 } else { 7 }];
        let (mut tx, mut ty) = (fl_.start[0], fl_.start[1]);
        for x in 0..VW {
            let g = tex[fl(ty, 1000) as usize * 64 + fl(tx, 1000) as usize];
            px[y * VW + x] = g <= data.thresh[e as usize][BAYER[y % 4][x % 4]];
            tx = md(tx + fl_.step[0], 64_000);
            ty = md(ty + fl_.step[1], 64_000);
        }
    }
    // the walls, column by column, the door over them
    for x in 0..VW {
        let k = 2 * x as i64 - 447;
        let (rdx, rdy) = (ux + fl(-uy * k, 560), uy + fl(ux * k, 560));
        let (wall, door) = ray(data, &cam, rdx, rdy);
        for (h, lift) in [(Some(wall), 0), (door, cam.door)] {
            let Some(h) = h else { continue };
            let (top, rows) = wall_rows(data, &h, x, lift, bright);
            for (i, b) in rows.into_iter().enumerate() { px[(top + i) * VW + x] = b; }
        }
    }
    px
}

/// The whole square root.
fn isqrt(n: i64) -> i64 { let mut s = 0; while (s + 1) * (s + 1) <= n { s += 1; } s }

/// The Wikipedia page's sphere ((2R + 1) square): its dots only.
pub fn sphere() -> Vec<bool> {
    let r = BALL_R;
    let n = (2 * r + 1) as usize;
    let (unit, less, over) = BALL_DITHER;
    let mut px = vec![false; n * n];
    for y in -r..=r {
        let half = isqrt(r * r - y * y);
        for x in -half..=half {
            let s = isqrt(r * r - x * x - y * y);
            let light = BALL_X * x + BALL_Y * y + BALL_Z * s;
            let k = BAYER[((y + r) / 2 % 4) as usize][((x + r) / 2 % 4) as usize] as i64;
            let t = -fl(less - unit * (2 * k + 1), over);
            px[(y + r) as usize * n + (x + r) as usize] = t > 0 && light < t;
        }
    }
    px
}

/// The Finnish flag, cropped to its window: its width and its pixels (black true).
pub fn flag() -> (usize, Vec<bool>) {
    let [l, t, r, b] = FINNISH_RECT.map(|v| v as i64);
    let fw = (r - l - 1) - FLAG_IN;
    let fh = fl(fw * FLAG_RATIO.1, FLAG_RATIO.0);
    let ch = fh.min(b - 1 - FLAG_FOOT - (t + FLAG_TOP));
    // the cross's bands: x * 18 / fw in 5..8, y * 11 / fh in 4..7
    let edge = |k: i64, size: i64, parts: i64| -fl(-k * size, parts);
    let (x0, x1) = (edge(FLAG_BANDS[0].0, fw, FLAG_RATIO.0), edge(FLAG_BANDS[0].1, fw, FLAG_RATIO.0));
    let (y0, y1) = (edge(FLAG_BANDS[1].0, fh, FLAG_RATIO.1), edge(FLAG_BANDS[1].1, fh, FLAG_RATIO.1));
    let mut px = vec![false; (fw * ch) as usize];
    for y in 0..ch { for x in 0..fw {
        let border = x == 0 || y == 0 || x == fw - 1 || y == ch - 1;
        let cross = (x >= x0 && x < x1) || (y >= y0 && y < y1);
        // blue as three dots in four: every even row, and the odd columns of the odd rows
        let blue = y % 2 == 0 || x % 2 == 1;
        px[(y * fw + x) as usize] = border || (cross && blue);
    } }
    (fw as usize, px)
}
