// Drive reference: the night drive at 192x96 in ordinary Rust, from the formulas and the data cards (data/), then
// the white flash and the comic-book cover. Arg: the drive's data folder. Units: along the street quarters of a
// unit (q), across it eighths (e); a colour is three digits, a step of each channel's levels. Divisions round
// down, or toward zero where they say t().
use std::io::Write;
#[allow(unused_attributes, dead_code)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;
use self::demokit::tables;
const W: i32 = 192;
const H: i32 = 96;
const BAYER: [i32; 16] = tables::widen(tables::BAYER4);
const POOL_ALONG: [i32; 11] = tables::widen(tables::POOL_ALONG);
const POOL_ACROSS: [i32; 19] = tables::widen(tables::POOL_ACROSS);
/// Frames of the drive; then one white, then the cover.
const DRIVE: i32 = 144;
const COVER: i32 = 36;

fn nums(s: &str) -> Vec<i32> { s.split_whitespace().filter_map(|t| t.parse().ok()).collect() }
fn sgn(v: i32) -> i32 { if v < 0 { -1 } else { 1 } }
/// a b / c toward zero.
fn t(a: i32, b: i32, c: i32) -> i32 { a * b / c }
fn thr(x: i32, y: i32) -> i32 { BAYER[((y & 3) * 4 + (x & 3)) as usize] }
/// The haze over a thing of scale v (k = 188; sixteenths of a pixel a unit) or a row of ground 2 y - 75 = v half
/// rows below the horizon (k = 47): about 8 (distance - 60) / 75 sixteenths, as k / v - 6, 0-16.
fn fog(k: i32, v: i32) -> i32 { (k / v - 6).clamp(0, 16) }
/// Colour a toward b by k sixteenths, each channel's step rounded by the threshold.
fn mix(a: i32, b: i32, k: i32, th: i32) -> i32 {
    let mut out = 0;
    for p in [100, 10, 1] {
        let (da, db) = (a / p % 10, b / p % 10);
        out += p * (da + sgn(db - da) * (((db - da).abs() * k + th) / 16));
    }
    out
}

pub(crate) struct Data { road: Vec<i32>, turn: Vec<i32>, blocks: Vec<i32>, windows: Vec<i32>, cars: Vec<i32>, script: Vec<i32>, art: [Vec<i32>; 3],
    pub(crate) levels: Vec<i32>, mats: Vec<i32>, col: std::collections::HashMap<String, i32>, glyphs: Vec<[i32; 5]> }

impl Data {
    fn c(&self, name: &str) -> i32 { self.col[name] }
}

pub(crate) fn load(dir: &str) -> Data {
    let rd = |f: &str| std::fs::read_to_string(format!("{dir}/{f}")).unwrap();
    let n = |f: &str| nums(&rd(f));
    let mut col = std::collections::HashMap::new();
    let mut mats = vec![];
    for l in rd("colours.txt").lines() {
        let (name, rest) = l.split_once(' ').unwrap();
        if name == "materials" { mats = nums(rest) } else { col.insert(name.to_string(), nums(rest)[0]); }
    }
    let car = n("car.txt");
    // the caption's glyphs, from the listing's font: five rows of three bits, written as digits
    let font = std::fs::read_to_string(format!("{dir}/../../listing/data/font.json")).unwrap();
    let glyph = |c: char| -> [i32; 5] {
        let key = format!("\"{c}\": [");
        let at = font.find(&key).unwrap() + key.len();
        let v = nums(&font[at..at + font[at..].find(']').unwrap()].replace(',', " "));
        [v[0], v[1], v[2], v[3], v[4]]
    };
    Data { road: n("road.txt"), turn: n("turn.txt"), blocks: n("blocks.txt"), windows: n("windows.txt"), cars: n("cars.txt"), script: n("script.txt"),
        art: [car[..168].to_vec(), car[168..].to_vec(), n("jet.txt")], levels: n("levels.txt"), mats, col,
        glyphs: " ENJOY THIS EARLY PREVIEW ".chars().map(glyph).collect() }
}

/// Something drawn over the street: a box (quarters of a pixel) and a picture (or a post: its middle and half
/// width), its depth for the walls (sixteenths of a pixel a unit, bigger nearer) and its fog.
struct Sprite { post: bool, art: usize, colours: [i32; 7], left: i32, width: i32, top: i32, bottom: i32, depth: i32, fog: i32 }
/// A light: its middle and reach (quarters of a pixel), its depth for the walls, the row it stops above,
/// brightness and colour.
struct Light { x: i32, y: i32, reach: i32, depth: i32, above: i32, bright: i32, colour: i32 }

/// Frame f's colours; the caption ENJOY THIS EARLY PREVIEW over frames 24-83 if `caption`.
pub(crate) fn render(d: &Data, f: i32, caption: bool) -> Vec<i32> {
    let road = &d.road;
    // where the camera is: 9 quarters a frame, along the street's plan (an entry every 2 units)
    let z = 9 * f;
    let (entry, into, o, block) = (z / 8, z % 8, z % 120, z / 120);
    let (lane, red_left, red_top2) = (d.script[3 * f as usize], d.script[3 * f as usize + 1], d.script[3 * f as usize + 2]);
    let at = |i: i32, part: i32| road[i as usize] + ((road[i as usize + 1] - road[i as usize]) * part).div_euclid(8);
    let camera = at(entry, into) + lane;
    let slope = at(entry + 7, into) - at(entry + 1, into);
    let (c, s) = (d.turn[2 * slope.unsigned_abs() as usize], sgn(slope) * d.turn[2 * slope.unsigned_abs() as usize + 1]);
    // each column's ray, across and along (quarters of a pixel at the screen's distance)
    let ray_x = |x: i32| (440 * s + (2 * x - 191) * 2 * c).div_euclid(400);
    let ray_z = |x: i32| (440 * c - (2 * x - 191) * 2 * s).div_euclid(400);
    // the blocks' faces ahead: across from the camera (eighths and quarters), height in bands, tone, window texture place
    let face = |k: i32, side: i32| -> [i32; 5] {
        let b = &d.blocks[(8 * (block + k) + 4 * side) as usize..];
        let e = road[(15 * (block + k) + 6) as usize] - camera + if side == 0 { -b[0] } else { b[0] };
        [e, e.div_euclid(2), b[1], b[2], b[3]]
    };

    // the walls, column by column: material + 32 fog, -2 ground, -1 sky
    let mut mat = vec![0; (W * H) as usize];
    let mut wall_depth = vec![0; W as usize];
    for x in 0..W {
        let (rx, rz) = (ray_x(x), ray_z(x));
        // how far across the ray is where it is z ahead (quarters, toward its side, at most 600)
        let lat = |z: i32| sgn(rx) * (z.max(0) * rx.abs() / rz).min(600);
        let ends = |first: i32| { let mut v = vec![]; let mut z = first; for _ in 0..9 { v.push(lat(z)); if z < 864 { z += 120; } } v };
        let (near, far) = (ends(-o), ends(104 - o));
        // the walls the ray meets, nearest first: an end wall where the ray is already behind a face at the block's
        // start, a face where it crosses one along the block
        let mut hits = vec![];
        for k in 0..9 {
            let start = 120 * k - o;
            for side in 0..2 {
                let q = face(k, side)[1];
                let (pn, pf) = if side == 0 { (q - near[k as usize], q - far[k as usize]) } else { (near[k as usize] - q, far[k as usize] - q) };
                if start < 880 && start + 104 > 0 {
                    let end = start > 0 && pn >= 0;
                    if (end && pn < 120) || (pn < 0 && pf >= 0) { hits.push((face(k, side), start, if end { pn } else { -1 })); }
                }
            }
        }
        let mut column = vec![];   // bottom up
        let fill = |column: &mut Vec<i32>, from: i32, stop: i32| for y in (stop + 1..=from).rev() { column.push(if y > 39 { -2 } else if y > 37 { 537 } else { -1 }); };
        let mut open = 95;
        for (i, &(fc, start, met)) in hits.iter().enumerate() {
            let [e, q, top, tone, tex] = fc;
            let end = met >= 0;
            let scale = if end { (rz * 16 / start).min(467) } else { (rx.abs() * 32 / e.abs()).min(467) };
            let place = if end { met } else { (q.abs() * rz / rx.abs() - start).clamp(0, 103) };
            let fg = fog(188, scale);
            // the foot's row; up from it, half-unit bands, each scale 32nds of a row
            let (lift, mut row) = { let l = 16 + 4 * (scale % 8); if l >= 32 { (l - 32, 38 + scale / 8) } else { (l, 37 + scale / 8) } };
            if i == 0 { wall_depth[x as usize] = scale; fill(&mut column, 95, row); }
            let mut band = 0;
            let mut to_next = scale - lift;
            while to_next <= 0 { band += 1; to_next += scale; }
            // the window texture's column for where along the wall the ray meets it
            let tcol = (tex + if end { 480 } else { 0 } + 32 * (place / 8)) % 960;
            let pillar = place % 8 < 2;
            while band < top && row >= 0 {
                if row <= open {
                    let m = if band >= top - 2 { tone + 2 * (band - (top - 2)) } else if pillar || band % 6 < 2 { tone } else { d.windows[(tcol + band / 6) as usize] };
                    column.push(32 * fg + if end { 9 } else { 0 } + m);
                    open = row - 1;
                }
                row -= 1;
                to_next -= 32;
                while to_next <= 0 { band += 1; to_next += scale; }
            }
        }
        fill(&mut column, open, -1);
        assert_eq!(column.len(), 96, "frame {f} column {x}");
        for y in 0..H { mat[(y * W + x) as usize] = column[(95 - y) as usize]; }
    }

    // the street: where each row meets it, how far from its middle, the lines and the lamplight
    let lane32 = (into * (road[entry as usize + 1] - road[entry as usize])).div_euclid(2) + 4 * lane;
    for y in 40..H {
        let eta = 2 * y - 75;
        let (near, sparse) = (y >= 58, y < 48);
        let fg = fog(47, eta);
        for x in 0..W {
            let i = (y * W + x) as usize;
            if mat[i] != -2 { continue; }
            let (rx, rz) = (ray_x(x), ray_z(x));
            // along (eighths of a quarter near by, else quarters); the street's middle there, in 32nds
            let w = rz * if near { 32 } else { 4 } / eta;
            let (step, per_entry, part, div) = if near { (1, 64, 1, 16) } else if sparse { (2, 16, 1, 4) } else { (1, 8, 8, 16) };
            let place = w + if near { 8 * into } else { into };
            let (e, inside) = (entry + step * (place / per_entry), part * (place % per_entry));
            let (lo, hi) = (road[e as usize], road[(e + step) as usize]);
            let middle = 4 * (lo - road[entry as usize]) + sgn(hi - lo) * ((hi - lo).abs() * inside / div);
            // across from it: 32nds near by, else eighths (at most 200) times 4
            let off = if near { (sgn(rx) * (32 * rx.abs() / eta) + lane32 - middle).abs() }
                else { let o32 = lane32 - middle; 4 * (sgn(rx) * (8 * rx.abs() / eta) + sgn(o32) * (o32.abs() / 4)).abs().min(200) };
            let run = (o + w / if near { 8 } else { 1 }) % 120;
            let mut m = if off >= 288 { 25 } else if off >= 192 { 24 } else if (182..190).contains(&off) { 23 } else if off <= 4 && run % 30 < 15 { 22 } else { 18 };
            if m == 18 {
                let (across, along) = (((off - 160).abs() / 8).min(19), (run - 60).abs() / 4);
                let light = if across < 19 && along < 11 { POOL_ALONG[along as usize] * POOL_ACROSS[across as usize] } else { 0 };
                m += ((light + 3 * thr(x, y)) / 48).min(3);
            }
            mat[i] = 32 * fg + m;
        }
    }

    // colours, fog and depth (sky: nothing behind it)
    let mut code: Vec<i32> = mat.iter().map(|&m| if m < 0 { -1 } else { d.mats[(m % 32) as usize] }).collect();
    let mut fogs: Vec<i32> = mat.iter().map(|&m| if m < 0 { 0 } else { m / 32 }).collect();
    let mut depth: Vec<i32> = (0..W * H).map(|i| if mat[i as usize] < 0 { 0 } else { wall_depth[(i % W) as usize] }).collect();

    // what stands in the street, from the camera (eighths across, quarters ahead): the lamps (each block, left then
    // right), then the traffic
    let mut things: Vec<(i32, i32)> = vec![];
    for k in 0..7 {
        let (a, b) = (road[(15 * (block + k) + 7) as usize], road[(15 * (block + k) + 8) as usize]);
        let middle = a + (b - a).div_euclid(2);
        things.push((middle - 53 - camera, 60 - o + 120 * k));
        things.push((middle + 53 - camera, 60 - o + 120 * k));
    }
    for car in d.cars.chunks(6) {
        let ahead = if car[0] > 0 { if (car[1] - f).abs() > 50 { 828 } else { 28 + 16 * (car[1] - f) } } else { car[1] - 4 * f };
        let place = into + ahead.max(0);
        things.push((at(entry + place / 8, place % 8) + car[2] - camera, ahead));
    }
    let head = if f % 2 == 0 { d.c("head") } else { d.c("head2") };
    let mut sprites: Vec<Sprite> = vec![];
    let mut lights: Vec<Light> = vec![];
    for (i, &(dx, dz)) in things.iter().enumerate() {
        // turned to the view: across in eighths, along in quarters; then on the screen
        let across = t(dx, c, 400) - t(dz, s, 200);
        let along = t(t(dx, s, 400), 1, 2) + t(dz, c, 400);
        let mut sight = (12..=559).contains(&along);
        let mut cx = 0;
        if sight { cx = 220 * across.abs() / along; if cx > 480 { sight = false; cx = 0; } else { cx *= sgn(across); } }
        let along = if sight { along } else { 12 };
        let (scale, s16) = (1760 / along, 7040 / along);   // quarters of a pixel a unit; sixteenths
        let fg = fog(188, s16);
        if i < 14 {
            let side = if i % 2 == 0 { -1 } else { 1 };
            let (top, foot) = if sight { (153 - 3 * scale, 152 + 2 * scale) } else { (999, 999) };
            sprites.push(Sprite { post: true, art: 0, colours: [0; 7], left: 384 + cx, width: (scale / 8).max(2), top, bottom: foot, depth: s16, fog: fg });
            let arm = 4 * scale / 5;
            let reach = if sight { (scale / 2).max(5) } else { 0 };
            lights.push(Light { x: 384 + cx + if side < 0 { arm } else { -arm }, y: 152 - 3 * scale, reach, depth: s16, above: 999, bright: 16 - fg, colour: d.c("lamp") });
        } else {
            let car = &d.cars[6 * (i - 14)..6 * (i - 14) + 6];
            let half = 9 * scale / 8;
            let (left, width, height) = (384 + cx - half, 2 * half, 7 * scale / 8);
            let (top, bottom) = if sight { (152 + half, 152 + half + height) } else { (999, 999) };
            let colours = [0, car[4], car[5], d.c("glass"), if car[3] == 0 { d.c("tail") } else { head }, d.c("tyre"), d.c("plate")];
            sprites.push(Sprite { post: false, art: car[3] as usize, colours, left, width, top, bottom, depth: s16, fog: fg });
            if car[3] == 1 {
                let gy = if sight { top + height / 2 } else { 999 };
                let reach = if sight { (6 * scale / 5).max(8) } else { 0 };
                for lx in [left + 3 * width / 20, left + 17 * width / 20] {
                    lights.push(Light { x: lx, y: gy, reach, depth: s16, above: bottom, bright: (16 - fg) * 13 / 16, colour: d.c("glare") });
                }
            }
        }
    }
    // the jet, coming in down the strip of sky: a picture too, behind everything but the sky
    if (104..140).contains(&f) {
        let k = f - 103;
        let size = 4 + k * k / 36;
        let (jx, jy) = (368 + 10 * k / 9, 120 - 10 * k * k / 81);
        let top = jy - 9 * size / 16;
        let (grey, lit) = (d.c("jet"), f % 4 < 2);
        let colours = [0, grey, if f % 6 == 0 { d.c("strobe") } else { grey }, if lit { d.c("tip_red") } else { grey }, if lit { d.c("tip_green") } else { grey }, grey, grey];
        sprites.push(Sprite { post: false, art: 2, colours, left: jx - size, width: 2 * size, top, bottom: top + 7 * size / 8, depth: 1, fog: 0 });
    }
    // each drawn where it is at least as near as what is there (and becomes what is there)
    let draw = |sp: &Sprite, code: &mut Vec<i32>, fogs: &mut Vec<i32>, depth: &mut Vec<i32>| {
        for y in 0..H {
            let py = 4 * y + 2;
            if py < sp.top || py >= sp.bottom { continue; }
            for x in 0..W {
                let i = (y * W + x) as usize;
                let px = 4 * x + 2;
                if sp.depth < depth[i] { continue; }
                let colour = if sp.post { if (px - sp.left).abs() < sp.width { d.c("post") } else { continue } } else {
                    let into = px - sp.left;
                    if into < 0 || into >= sp.width { continue; }
                    let (u, v) = (into * 36 / sp.width, (py - sp.top) * 14 / (sp.bottom - sp.top));
                    let card = d.art[sp.art][(12 * v + u / 3) as usize];
                    let what = [card / 100, card / 10 % 10, card % 10][(u % 3) as usize];
                    if what == 0 { continue; }
                    sp.colours[what as usize]
                };
                code[i] = colour;
                fogs[i] = sp.fog;
                depth[i] = sp.depth;
            }
        }
    };
    for sp in &sprites { draw(sp, &mut code, &mut fogs, &mut depth); }

    // the sky, then the haze over everything at a distance
    for y in 0..H { for x in 0..W {
        let i = (y * W + x) as usize;
        code[i] = if code[i] < 0 { mix(d.c("sky_low"), d.c("sky_top"), if y < 38 { (75 - 2 * y) * 16 / 76 } else { 0 }, thr(x, y)) }
            else { mix(code[i], d.c("sky_low"), fogs[i], thr(x, y)) };
    } }
    // the lights, each over the walls it is in front of, falling off over its reach: the distance an octagon's
    // (the larger offset and half the smaller), counted twice over
    for l in &lights {
        for y in 0..H { for x in 0..W {
            let (ax, ay) = ((4 * x + 2 - l.x).abs(), (4 * y + 2 - l.y).abs());
            let inside = 2 * l.reach - if ax > ay { 2 * ax + ay } else { 2 * ay + ax };
            if l.depth < wall_depth[x as usize] || 4 * y + 2 >= l.above || inside <= 0 { continue; }
            let i = (y * W + x) as usize;
            code[i] = mix(code[i], l.colour, inside * l.bright / (2 * l.reach), thr(x, y));
        } }
    }
    // our car, a beat ahead of the camera, and its shadow
    let top = 2 * red_top2;
    for y in 0..H { for x in 0..W {
        let (px, py) = (4 * x + 2, 4 * y + 2);
        if py >= top + 80 && py < top + 90 && (11..=205).contains(&(px - red_left)) { let i = (y * W + x) as usize; code[i] = mix(code[i], d.c("black"), 8, thr(x, y)); }
    } }
    let red = Sprite { post: false, art: 0, colours: [0, d.c("red"), d.c("red_dark"), d.c("glass"), d.c("tail"), d.c("tyre"), d.c("plate")],
        left: red_left, width: 216, top, bottom: top + 84, depth: 999, fog: 0 };
    let mut ignored = vec![0; (W * H) as usize];
    draw(&red, &mut code, &mut ignored.clone(), &mut ignored);
    // the caption, from 2 s for 5 s, dithered in and out
    let cu = f - 24;
    if caption && (0..60).contains(&cu) {
        let fade = (2 * cu.min(59 - cu)).min(16);
        for y in 0..7 { for x in 0..=100 {
            if thr(x, y) >= fade { continue; }
            let g = d.glyphs[(x / 4) as usize];
            let on = (1..6).contains(&y) && x % 4 < 3 && { let r = g[(y - 1) as usize]; [r / 100, r / 10 % 10, r % 10][(x % 4) as usize] == 1 };
            code[(y * W + x) as usize] = if on { d.c("caption") } else { d.c("black") };
        } }
    }
    code
}

pub(crate) fn paint(levels: &[i32], codes: &[i32], out: &mut impl Write) {
    for &c in codes { out.write_all(&[levels[(c / 100) as usize] as u8, levels[10 + (c / 10 % 10) as usize] as u8, levels[20 + (c % 10) as usize] as u8]).unwrap(); }
}

/// The cover's panels: twice as fine (1) or not, crop (first column and row, width, height, in that grid), the
/// picture's size, the shears (times 500, rows then columns: a turn by three shears) and where the canvas goes.
const PANELS: [(i32, [i32; 4], [i32; 2], [i32; 2], [i32; 2]); 3] = [(0, [20, 6, 152, 90], [118, 70], [-17, 35], [-3, 1]),
    (1, [0, 0, 120, 70], [58, 34], [-26, 52], [122, -2]), (1, [150, 150, 100, 40], [58, 30], [22, -44], [124, 39])];
const MARGIN: i32 = 8;

/// The last frame as a comic-book cover: poster colours (each channel lifted, pushed from grey, cut to four steps),
/// black lines where the step sum jumps, three panels cropped, resized to the nearest, framed, turned and laid down.
fn cover(d: &Data, last: &[i32], lift: &[i32]) -> Vec<i32> {
    let mut colour = vec![0; (W * H) as usize];
    let mut sum = vec![0; (W * H) as usize];
    for i in 0..(W * H) as usize {
        let c = last[i];
        let l = [lift[(c / 100) as usize], lift[10 + (c / 10 % 10) as usize], lift[20 + (c % 10) as usize]];
        let grey = l[0] + l[1] + l[2];
        for &v in &l {
            let dd = 3 * v - grey;
            let pushed = sgn(dd) * (dd.abs() * 17 / 10);
            let step = [192, 384, 576].iter().filter(|&&t| pushed >= t - grey).count() as i32;
            colour[i] = 10 * colour[i] + step;
            sum[i] += step;
        }
    }
    let lines = |x: i32, y: i32| -> i32 { let i = (y * W + x) as usize;
        (if x > 0 && (sum[i] - sum[i - 1]).abs() >= 2 { 1 } else { 0 }) + if y > 0 && (sum[i] - sum[i - W as usize]).abs() >= 2 { 2 } else { 0 } };
    let mut out = vec![666; (W * H) as usize];
    for &(fine, [x0, y0, cw, ch], [sw, sh], [a, b], [ax, ay]) in &PANELS {
        let crop = |xc: i32, yc: i32| -> i32 {
            let (column, row) = (x0 + xc, y0 + yc);
            let (fx, fy) = if fine > 0 { (column / 2, row / 2) } else { (column, row) };
            let l = lines(fx, fy);
            let started = fine == 0 || column % 2 == 0;
            let above = row > y0 && (fine == 0 || row % 2 == 0);
            if (xc > 0 && started && l % 2 == 1) || (above && l >= 2) { 444 } else { colour[(fy * W + fx) as usize] }
        };
        let near = |x: i32, n_in: i32, n_out: i32| (2 * x + 1) * n_in / (2 * n_out);
        let (fw, fh, wc, hc) = (sw + 4, sh + 4, sw + 4 + 2 * MARGIN, sh + 4 + 2 * MARGIN);
        let framed = |x: i32, y: i32| -> i32 { if x < 0 || y < 0 || x >= fw || y >= fh { -1 } else if x < 2 || y < 2 || x >= fw - 2 || y >= fh - 2 { 555 }
            else { crop(near(x - 2, cw, sw), near(y - 2, ch, sh)) } };
        let shift = |s: i32, i: i32, n: i32| (s * (2 * i + 1 - n) + 500).div_euclid(1000);
        let turned = |x: i32, y: i32| -> i32 {
            // the last shear's row, then the column's, then the first's
            let x2 = x - shift(a, y, hc);
            if x2 < 0 || x2 >= wc { return -1; }
            let y1 = y - shift(b, x2, wc);
            if y1 < 0 || y1 >= hc { return -1; }
            let x1 = x2 - shift(a, y1, hc);
            if x1 < 0 || x1 >= wc { return -1; }
            framed(x1 - MARGIN, y1 - MARGIN)
        };
        for y in 0..H { for x in 0..W {
            let (xc, yc) = (x - ax, y - ay);
            if xc < 0 || yc < 0 || xc >= wc || yc >= hc { continue; }
            let v = turned(xc, yc);
            if v >= 0 { out[(y * W + x) as usize] = v; }
        } }
    }
    let _ = d;
    out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = load(&a[1]);
    let lift = nums(&std::fs::read_to_string(format!("{}/lift.txt", a[1])).unwrap());
    let cover_levels = nums(&std::fs::read_to_string(format!("{}/cover.txt", a[1])).unwrap());
    let mut out = std::io::stdout().lock();
    let mut last = vec![];
    for f in 0..DRIVE { last = render(&d, f, true); paint(&d.levels, &last, &mut out); }
    paint(&cover_levels, &vec![777; (W * H) as usize], &mut out);
    let page = cover(&d, &last, &lift);
    for _ in 0..COVER { paint(&cover_levels, &page, &mut out); }
}
