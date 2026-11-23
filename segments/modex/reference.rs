// Mode X reference: the ray-traced mirror ball over the plasma floor (ray/reference_plasma.rs) at 320x240, Mode X's
// square pixels: the same scene over a 4:3 frame, more sky above and more floor below. Every constant tied to
// 192x96 is re-derived at 5 / 3 of it (a 192x96 pixel is 5 / 3 of these across and down; the 192x96 frame's rows
// 0..95 are rows 40..199 here):
// - The floor: the horizon is row 100 (192x96's 36), the camera 200 pixels from the screen (120). A row y meets
//   the floor at v = 120 (200 - y) / (y - 100) plasma units along it (rounded down; row 200 is 192x96's bottom
//   row, below it the floor comes nearer, v under 0), depth z = 120 + v, fog (z - 240) / 7 up to 32 (all haze),
//   across u = (x - 160) z / 200 (rounded down). Rows 0..100 are sky, its bands at rows 60 and 84 (192x96's 12 and
//   26); rows 101..119 haze (192x96's slope under 60). The plasma's units, its shadow and its colours are the
//   plasma's own (plasma/formula.rs).
// - The ball: radius 46 (192x96's 28 is 46 2/3), centred at column 160 + 10 x / 3 and row 107 - 10 L / 3 (rounded
//   down), L its lift over the floor (the script's height less 14, or the entrance's arcs, as reference_plasma's), its
//   centre 46 + 10 L / 3 over the floor. A pixel (A, B) from the centre is on it if A^2 + B^2 < 46^2; its surface
//   faces the camera by C = isqrt(2116 - A^2 - B^2); a ray from the eye bounces off it in the direction
//   (C A / 2, C B / 2, 529 - C^2 / 2) out of 529 (46^2 / 4; each rounded toward zero). The light, the sky high and
//   low are where 192x96's were, the directions out of 529 for its 392: X at most -189 and Y at least 243 the light,
//   Y over 270 the sky high, Y at least 0 the sky low. A downward ray, from height h = 46 + 10 L / 3 + B, meets the
//   floor after t = 100 h / -Y hundredths of 529 pixels (rounded down; 192x96's tenths at 392 / 20 were too
//   coarse for this size, five steps across the floor in the ball); from 25 on (past 130 2/3 pixels, the
//   prototype's 4 twentieths of 392 / 20 at 192x96) the floor is lost in haze. The floor point is (x - 160 + t X / 100, 93 1/3 - C + t Z / 100) in pixels (across from the
//   middle, and deep from the ball's centre's depth, toward zero), in plasma units 9 / 8 of that (rounded down; at
//   the ball's depth a pixel is 9 / 8 plasma units), the depth's from 105: (u, v) = (9 (x - 160 + t X / 100) / 8,
//   105 + 9 (t Z / 100 - C) / 8). It shows the plasma there 4 32nds darker, as reference_plasma's.
// Behind the ball, Mode X's sign (ui/pages.rs's MODEX_SIGN, drawn by ui/text.rs; in place of
// the setup's listing): VGA over MODE X in the VGA's 8x16 font, each page pixel 6 x 4 on the picture at (13, 6), a
// pixel lettered where its page pixel has ink. Where the ball doesn't cover
// them the letters are white and every pixel off them within 3 of one (each way, the diagonals too) black: a thick
// outline, as the titles' (after chrome that didn't read). The ball falls from the frame's top
// at frame 0 (data/entrance.txt's arcs), the ray's script carried on from its 22nd frame. 156 frames: the ray's
// 120-frame script and 36 more, carried on from its start (its sway and its bounces repeat every 120). Over the last
// of them, when DEGAUSS is set, the degauss (degauss.rs) on the 1536x768 frames. Writes video.rgb, the 156 frames at
// 320x240; then video_hd.rgb, the 156 at 1536x768, each 320x240 frame 1024x768 in the middle, pixel (x, y) of it
// showing (5 x / 16, 5 y / 16) (nearest), black at the sides.
// Args: the demo folder.
use std::io::Write;

#[allow(unused_attributes, dead_code)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;
#[allow(dead_code)]
#[path = "../plasma/formula.rs"]
mod formula;
#[path = "../ui/pages.rs"]
#[allow(dead_code)]
mod pages;
#[path = "../ui/text.rs"]
mod text;
#[path = "degauss.rs"]
mod degauss;
use formula::{Look, Shadow};

const W: i32 = 320;
const H: i32 = 240;
const HW: usize = 1536;
const HH: usize = 768;
/// Mode X's own entrance (data/entrance.txt: the ball falling from the frame's top at frame 0), and where in the
/// ray's script it carries on from: 22 frames in, so the script's hops land as the entrance's last arc does.
const SCRIPT_AT: usize = 22;
/// The sign's black outline, in pixels of the 320x240 picture (the letters white; as an earlier
/// cut's titles).
const OUTLINE: i32 = 3;
/// The frames: the ray's script and 36 more, the degauss's.
const FRAMES: usize = 156;
/// The degauss over the last frames: moved to the rotozoom's end (rotodg; the dorm shows these frames
/// small, on its monitor), so none here.
const DEGAUSS: bool = false;

fn isqrt(n: i32) -> i32 { let mut k = 0; while (k + 1) * (k + 1) <= n { k += 1; } k }
/// 9 h / 8 rounded down: a pixel at the ball's depth in plasma units.
fn nine_eighths(h: i32) -> i32 { (9 * h).div_euclid(8) }

/// The floor at 320x240 (formula.rs's plane tilted right back, at 5 / 3 the size).
fn plane(x: i32, y: i32, s: Shadow) -> Look {
    if y <= 100 { return Look::Fixed(if y < 60 { 0 } else if y < 84 { 1 } else { 2 }); }
    if y < 120 { return Look::Fixed(3); }
    let v = (120 * (200 - y)).div_euclid(y - 100);
    let z = 120 + v;
    let fog = ((z - 240) / 7).clamp(0, 32);
    if fog >= 32 { return Look::Fixed(3); }
    let u = ((x - 160) * z).div_euclid(200);
    let (du, dv) = (u - (15 * s.x).div_euclid(4), v - 105);
    let dark = if du * du + dv * dv < s.radius * s.radius { s.dark } else { 0 };
    Look::Plasma { u, v, fog, dark }
}

fn main() {
    let demo = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let path = |p: &str| demo.join(p).to_str().unwrap().to_string();
    let keys = formula::keys(&std::fs::read_to_string(path("plasma/data/ring.txt")).unwrap());
    let script = formula::script(&path("ray/data/script.txt"));
    let arcs = formula::script(&path("modex/data/entrance.txt"));
    // the sign: where its page has ink, or the page pixel left of it (fat strokes; each page pixel 6 x 4 on the picture)
    let sign_page = &pages::MODEX_SIGN;
    let glyphs = text::Glyphs::read(&demo);
    let sign_ink = text::draw(sign_page, &glyphs, sign_page.chars(), false);
    let inked = |x: i32, y: i32| x >= 0 && text::on_page(sign_page, x as usize, y as usize).map_or(false, |i| sign_ink[3 * i] > 0);
    // the outline: the pixels off the letters within OUTLINE of one (each of OUTLINE steps taking in all eight
    // neighbours), black
    let mut outlined = vec![false; (W * H) as usize];
    for y in 0..H { for x in 0..W {
        outlined[(y * W + x) as usize] = !inked(x, y)
            && (-OUTLINE..=OUTLINE).any(|dy| (-OUTLINE..=OUTLINE).any(|dx| inked(x + dx, y + dy)));
    } }
    let start = formula::script(&path("plasma/data/script.txt")).len() as i32;
    let mut out = std::io::stdout().lock();
    let mut frames = vec![];
    for f in 0..FRAMES as i32 {
        // the script's 120 frames, carried on from its start (its sway and its bounces repeat every 120)
        let s = &script[(f as usize + SCRIPT_AT) % script.len()];
        // the ball's lift over the floor: an arc of the entrance, or the script (as reference_plasma's)
        let lift = match arcs.iter().rev().find(|arc| f >= arc[0]) {
            Some(arc) if f < arc[0] + arc[1] => { let (j, l) = (f - arc[0], arc[1]); arc[2] * j * (l - j) / (l * l / 4) }
            _ => s[1] - 14,
        };
        let shadow = Shadow { x: s[0], radius: 44 - lift / 2, dark: 16 - lift / 4 };
        let up = (10 * lift).div_euclid(3);
        let (cx, cy, centre) = (160 + (10 * s[0]).div_euclid(3), 107 - up, 46 + up);
        let mut pic = Vec::with_capacity((W * H * 3) as usize);
        for y in 0..H { for x in 0..W {
            let mut look = plane(x, y, shadow);
            let (a, b) = (x - cx, cy - y);
            let n2 = 2116 - a * a - b * b;
            if n2 > 0 {
                let c = isqrt(n2);
                let (rx, ry, rz) = (c * a / 2, c * b / 2, 529 - c * c / 2);
                look = if rx <= -189 && ry >= 243 { Look::Fixed(7) } else if ry >= 0 { Look::Fixed(if ry > 270 { 4 } else { 5 }) } else {
                    let t = 100 * (centre + b) / -ry;
                    if t >= 25 { Look::Fixed(6) } else {
                        let (dx, dz) = (t * rx / 100, t * rz / 100);
                        Look::Plasma { u: nine_eighths(x - 160 + dx), v: 105 + nine_eighths(dz - c), fog: 0, dark: 4 }
                    }
                };
            }
            if n2 <= 0 && inked(x, y) { pic.extend([255, 255, 255]); continue; }
            if n2 <= 0 && outlined[(y * W + x) as usize] { pic.extend([0, 0, 0]); continue; }
            pic.extend(formula::colour(&keys, look, start + f).map(|v| v as u8));
        } }
        out.write_all(&pic).unwrap();
        frames.push(pic);
    }

    // at 1536x768
    let glyphs_hd = &glyphs;
    // the degauss (degauss.rs): its tables, the sign's pixels, each 320x240 pixel's angle from the middle
    let dg = degauss::read(&demo);
    let word_page = &pages::DEGAUSS;
    let word = text::draw(word_page, glyphs_hd, word_page.chars(), false);
    let sign = degauss::sign_pixels(&dg, &|x, y| text::on_page(word_page, x, y).map_or(false, |i| word[3 * i + 1] > 0));
    let turn: Vec<(i64, i64)> = (0..W as i64 * H as i64).map(|i| degauss::angle(&dg, 2 * (i % W as i64) - 319, 2 * (i / W as i64) - 239)).collect();
    let cell = |x: usize, y: usize| (256..1280).contains(&x).then(|| (y * 5 / 16) * W as usize + (x - 256) * 5 / 16);
    let mut hd = vec![0u8; HW * HH * 3];
    for (k, pic) in frames.iter().enumerate() {
        let s = if DEGAUSS { degauss::strength(k) } else { 0 };
        let blotched;
        let pic = if s > 0 { blotched = degauss::blotch(&dg, s, pic); &blotched } else { pic };
        for y in 0..HH { for x in 256..1280 {
            let i = ((y * 5 / 16) * W as usize + (x - 256) * 5 / 16) * 3;
            let p = [pic[i], pic[i + 1], pic[i + 2]];
            hd[(y * HW + x) * 3..][..3].copy_from_slice(&p);
        } }
        if DEGAUSS && degauss::sign_shown(k) { degauss::lay_sign(&mut hd, &sign, &text::mix); }
        if DEGAUSS && (degauss::D0..degauss::D0 + degauss::DN).contains(&k) {
            out.write_all(&degauss::jolt(&dg, k - degauss::D0, &hd, &cell, &turn)).unwrap();
        } else {
            out.write_all(&hd).unwrap();
        }
    }
}
