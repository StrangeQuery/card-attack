// Reference for the tumble: every frame at 192x96 in ordinary Rust, straight from the formulas: the timeline, the
// turning cube and its travels seen in perspective, its edges and lit faces, the rotozoomer behind it and the dissolves. Writes raw RGB for
// all frames to stdout.
#[allow(dead_code)]
#[path = "../roto/roto_data.rs"]
mod roto_data;   // the rotozoomer's heart, palette and sine table
use roto_data::*;
use std::io::Write;
#[allow(unused_attributes)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;
const BAYER: [i32; 16] = demokit::tables::widen(demokit::tables::BAYER4);
const W: i32 = 192; const H: i32 = 96;
const EDGES: [(usize, usize); 12] = [(0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (1, 3), (4, 6), (5, 7), (0, 4), (1, 5), (2, 6), (3, 7)];
const FACE_TRI: [[usize; 3]; 6] = [[4, 6, 2], [1, 3, 7], [1, 5, 4], [2, 6, 7], [2, 3, 1], [4, 5, 7]];
const FACE_EDGES: [[usize; 4]; 6] = [[6, 10, 4, 8], [5, 11, 7, 9], [9, 2, 8, 0], [10, 3, 11, 1], [1, 5, 0, 4], [2, 7, 3, 6]];
const COLOURS: [[i32; 3]; 6] = [[15, 3, 3], [2, 13, 15], [15, 13, 2], [4, 6, 15], [14, 4, 14], [3, 14, 4]];
type Pts = [(i32, i32); 8];

// the timeline: 16 frames flat, 15 tilting, 170 travelling (faces in from 0, the rotozoomer from 120, the cube out
// from 154). Where the cube is, (x, y, distance), at half size: the whole cube is twice it. lens: its size on
// screen is lens / distance; None is flat on, no perspective.
struct Frame { axis: usize, angle: usize, tilt: i32, at: (i32, i32, i32), lens: Option<i32>, flevel: i32, rlevel: i32, rframe: i32, out: i32 }
fn sine(scale: f64, n: usize, k: usize) -> i32 { (scale * (2.0 * std::f64::consts::PI * k as f64 / n as f64).sin() + 0.5).floor() as i32 }
fn fade(k: i32) -> i32 { if k < 0 { 0 } else if k < 16 { 16 * (k + 1) / 17 } else { 16 } }
fn frame(f: i32) -> Frame {
    if f < 16 { return Frame { axis: 2, angle: ((15 * (f + 1) / 4) % 60) as usize, tilt: 0, at: (0, 0, 0), lens: None, flevel: 0, rlevel: 0, rframe: 0, out: 0 }; }
    let k = f - 16;
    let angle = |u: i32| ((3 * (u + 1)) % 60) as usize;
    // across, sin a less a quarter of it; down, a sixth of sin b; away, 180 + sin c and a half (quarters, sixths
    // and halves toward zero)
    let place = |a: i32, b: i32, c: i32| {
        let (sa, sb, sc) = (sine(48.0, 120, (a % 120) as usize), sine(48.0, 120, (b % 120) as usize), sine(48.0, 120, (c % 120) as usize));
        (sa - sa / 4, sb / 6, 180 + sc + sc / 2)
    };
    if k < 15 { return Frame { axis: 0, angle: angle(k), tilt: (k / 4).min(3), at: place(0, 0, 30), lens: Some(504), flevel: 0, rlevel: 0, rframe: 0, out: 0 }; }
    let v = k - 15;
    Frame { axis: if (v / 20) % 2 == 0 { 1 } else { 0 }, angle: angle(v), tilt: 3, at: place(2 * v, 4 * v, 30 + 2 * v), lens: Some((504 - 20 * v).max(205)),
        flevel: (v + 1).min(16), rlevel: fade(v - 120), rframe: (v - 120).max(0), out: fade(v - 154) }
}
fn sign(v: usize, bit: usize) -> i32 { if v & bit != 0 { 1 } else { -1 } }
// the corners turned by the angle about the axis, for a cube of half-width h at (x, y, distance), tilted, seen
// from the eye through the lens (trunc(lens x / (distance + depth)), likewise y), then placed at the centre
fn project(fr: &Frame, h: i32, at: (i32, i32, i32), lens: Option<i32>, centre: (i32, i32)) -> Pts {
    let (s, c) = (sine(h as f64, 60, fr.angle), sine(h as f64, 60, (fr.angle + 15) % 60));
    let mut p = [(0, 0); 8];
    for v in 0..8 {
        let (x, y, z) = (sign(v, 1), sign(v, 2), sign(v, 4));
        let (x, y, d) = match fr.axis {
            0 => (x * c + z * s, h * y, z * c - x * s),
            1 => (h * x, y * c + z * s, z * c - y * s),
            _ => (x * c + y * s, y * c - x * s, h * z),
        };
        let a = (fr.tilt * d) / 10;
        let (x, y) = (at.0 + x + a, at.1 + y - a);
        p[v] = match lens {
            Some(l) => (centre.0 + l * x / (at.2 + d), centre.1 + l * y / (at.2 + d)),
            None => (centre.0 + x, centre.1 + y),
        };
    }
    p
}
// row y's span of an edge: the part of it within half a pixel of the row's centre, its ends rounded to the
// nearest pixel (halves away from the top end): even runs along the edge and half runs at its two ends
fn edge_span(p: &Pts, e: usize, y: i32) -> Option<(i32, i32)> {
    let (a, b) = EDGES[e];
    let ((mut xt, mut yt), (mut xb, mut yb)) = (p[a], p[b]);
    if yb < yt { std::mem::swap(&mut xt, &mut xb); std::mem::swap(&mut yt, &mut yb); }
    if y < yt || y > yb { return None; }
    if yt == yb { return Some((xt.min(xb), xt.max(xb))); }
    let (dx, dy, k) = (xb - xt, yb - yt, y - yt);
    // the edge's x, (half rows) / (2 dy) of the way down
    let at = |half_rows: i32| { let q = (half_rows * dx.abs() + dy) / (2 * dy); if dx < 0 { xt - q } else { xt + q } };
    let (x0, x1) = (at((2 * k - 1).max(0)), at((2 * k + 1).min(2 * dy)));
    Some((x0.min(x1), x0.max(x1)))
}
// the light of each face, from the cube at the long sequence's size: facing us, 8 + its projected area / 72
fn brightness(p: &Pts) -> [i32; 6] {
    let mut v = [0; 6];
    for f in 0..6 {
        let [a, b, c] = FACE_TRI[f].map(|i| p[i]);
        let cross = (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0);
        if cross < 0 { v[f] = (8 + (-cross) / 72).min(16); }
    }
    v
}
// which faces show, from the very corners the edges are drawn from: a face's diagonals, a to c and b to d (d, the
// fourth corner, is the one the other three don't share), cross to twice its projected area with a sign, so a face
// whose corners have been rounded out of flat shows if it shows as a whole. One that shows has its light, at least
// 8 (nearly edge on, the small cube may hide it); one that doesn't, none.
fn shown(p: &Pts, light: [i32; 6]) -> [i32; 6] {
    let mut v = [0; 6];
    for f in 0..6 {
        let [a, b, c] = FACE_TRI[f];
        let (a, b, c, d) = (p[a], p[b], p[c], p[a ^ b ^ c]);
        let cross = (c.0 - a.0) * (d.1 - b.1) - (c.1 - a.1) * (d.0 - b.0);
        if cross < 0 { v[f] = light[f].max(8); }
    }
    v
}
fn roto(f: i32, x: i32, y: i32) -> [u8; 3] {
    let (a, b) = ((f % 48) as usize, ((f * 2 / 3) % 48) as usize);
    let (s, c, z) = (SIN48[a], SIN48[(a + 12) % 48], 21 + (8 * SIN48[b]) / 10);
    let (du, dv) = ((z * c) / 20, (z * s) / 20);
    let u = (450 + (x - 96) * du - (y - 48) * dv).rem_euclid(900);
    let v = (450 + (x - 96) * dv + (y - 48) * du).rem_euclid(900);
    let code = (TEX[(v / 100) as usize].as_bytes()[(u / 100) as usize] - b'0') as usize;
    [PALETTE[3 * code], PALETTE[3 * code + 1], PALETTE[3 * code + 2]]
}
fn main() {
    let mut out = std::io::stdout().lock();
    for f in 0..201 {
        let fr = frame(f);
        let (x, y, d) = fr.at;
        let pts = project(&fr, 24, (2 * x, 2 * y, 2 * d), fr.lens, (96, 48));
        // the light from the cube at the long sequence's size, at half the place and distance, through a lens that
        // shows its near face, face on, at its own size; the faces shown from the big cube's own corners
        let lit = if fr.flevel > 0 { shown(&pts, brightness(&project(&fr, 12, fr.at, Some(d - 12), (48, 24)))) } else { [0; 6] };
        for y in 0..H {
            let spans: Vec<Option<(i32, i32)>> = (0..12).map(|e| edge_span(&pts, e, y)).collect();
            let faces: Vec<(i32, i32)> = (0..6).map(|f| {
                if lit[f] == 0 { return (900, -1); }
                FACE_EDGES[f].iter().filter_map(|&e| spans[e]).fold((900, -1), |(lo, hi), (a, b)| (lo.min(a), hi.max(b)))
            }).collect();
            for x in 0..W {
                let thr = (3 * y + 7 * (x + 1)) % 16;
                let mut code = if spans.iter().any(|s| s.is_some_and(|(lo, hi)| lo <= x && x <= hi)) { 2 } else { 0 };
                if fr.flevel > 0 && thr < fr.flevel { if let Some(f) = (0..6).find(|&f| faces[f].0 <= x && x <= faces[f].1) { code = 3 + f; } }
                let bayer = BAYER[((y % 4) * 4 + x % 4) as usize];
                let behind = if fr.rlevel > bayer { roto(fr.rframe, x, y) } else { [0, 0, 0] };
                let mut px = match code {
                    0 => behind,
                    2 => [120, 240, 240],
                    c => { let f = c - 3; COLOURS[f].map(|ch| (ch * lit[f]) as u8) }
                };
                if fr.out > bayer { px = roto(fr.rframe, x, y); }
                out.write_all(&px).unwrap();
            }
        }
    }
}
