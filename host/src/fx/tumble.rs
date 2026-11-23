//! The tumble, CARD ATTACK's new beginning after the opening: the square already spinning fast, at 192x96; it
//! tilts into a cube that turns faster than the long sequence's and travels about in front of us, across, up and
//! down, and toward and away from us, seen in perspective; its faces fizzle in, the rotozoomer dissolves in behind
//! it, and the cube dissolves away over the rotozoomer. The cut carries on with the rotozoomer's own render from the
//! next frame.
//!
//! Per frame: the vertex machine turns the cube at twice the long sequence's size (a sine table scaled to 24). Once
//! it tilts, the path machine says where the cube is, the projector shifts its corners for the tilt about that
//! place, and the perspective machine sees them from the eye. The faces are lit from the cube at the long
//! sequence's size, at half the place and distance, so it looks the same from the eye (the vertex machine, the
//! projector, the perspective machine, then the gather and shader as they are), since at twice the size a face's
//! area runs past a card. Which faces show is decided from the big cube's own corners, the ones its edges are drawn
//! from: gather4 takes each face's diagonals and facing crosses them, keeping the area as 576s and a rest, so a
//! face is filled exactly when it shows; then the palette. Per tile (a 192x96 frame is four 96x48 tiles): project4,
//! told the tile's place, puts the corners in the tile's pixels, and the edge sorter and raster4 make the spans, each
//! from the edge's own ends; the rotozoomer's rotor runs once its picture is needed. Per tile row: the compositor (no text) and, with faces, the span and solid machines make the cube's codes;
//! the rotozoomer's row and paint machines make the picture behind, dissolved in from black while it arrives; cover
//! lays the cube over it; and the dissolve machine takes the cube away at the end.
use crate::fx::roto4::Roto4;
use crate::tables::cube;
use crate::*;
use std::path::Path;

/// The parts, in frames: the flat spin, the tilt, then the cube travelling.
const FLAT: i32 = 16;
const TILT: i32 = 15;
const TRAVEL: i32 = 170;
/// In the travel: the faces fizzle in from its start, the rotozoomer dissolves in from ROTO_IN, the cube dissolves
/// away from CUBE_OUT; each over 16 frames.
const ROTO_IN: i32 = 120;
const CUBE_OUT: i32 = 154;
/// The lens, as the perspective machine has it: the cube's size on screen is LENS / its distance. The tilt sees
/// the cube from its far end (504) at its own size, and the lens draws back to LENS over the travel's first frames.
const LENS: i32 = 205;
const FAR: i32 = 504;
/// Twice the long sequence's cube: half-width 24.
const HALF: i32 = 24;

/// One frame's numbers, all decided here: the cube's axis, angle and tilt, the path's steps and the lens (none:
/// centred and flat on, no perspective), the faces' fizzle level, the rotozoomer's level and frame, and the cube's
/// dissolve level.
#[derive(Clone, Copy)]
pub struct Frame { pub axis: usize, pub angle: i32, pub tilt: i32, pub path: Option<([i32; 3], i32)>, pub flevel: i32, pub rlevel: i32, pub rframe: i32, pub out: i32 }

pub fn frames() -> usize { (FLAT + TILT + TRAVEL) as usize }

/// One frame's numbers, on CARDIAC (they were the host's): the turn (tm_turn: the angle; tm_axis: the axis and
/// tilt), the path and lens (tm_path), the levels (tm_levels).
pub fn frame(demo: &Path, host: &mut Host, f: i32) -> Frame {
    let (turn, axis, path, levels) = (rust(demo, "tm_turn"), rust(demo, "tm_axis"), rust(demo, "tm_path"), rust(demo, "tm_levels"));
    let a = host.go1(&turn, &[&[f]], 1);
    let x = host.go1(&axis, &[&[f]], 2);
    let p = host.go1(&path, &[&[f]], 5);
    let l = host.go1(&levels, &[&[f]], 4);
    Frame { axis: x[0] as usize, angle: a[0], tilt: x[1], path: if p[0] == 1 { Some(([p[1], p[2], p[3]], p[4])) } else { None },
        flevel: l[0], rlevel: l[1], rframe: l[2], out: l[3] }
}

/// The first rotozoomer frame after the tumble: where the cut picks up the rotozoomer's own render.
pub fn roto_next(demo: &Path, host: &mut Host) -> i32 { frame(demo, host, frames() as i32 - 1).rframe + 1 }

const BASE: [i32; 9] = [0, 0, 0, 240, 240, 240, 120, 240, 240];      // cover codes 0-2: no cube, text (none), wireframe

pub fn render(demo: &Path, refbin: &str) {
    let r = |n: &str| rust(demo, n);
    let (vtx, path, proj, persp, proj4, esort, raster) = (r("vtx"), r("path"), r("project"), r("persp"), r("project4"), r("esort"), r("raster4"));
    let (comp, gather, shade, pal, fspan, solid, cover, mix) = (r("comp"), r("gather"), r("fshade"), r("palette"), r("fspan"), r("solid"), r("cover"), r("dissolve"));
    let (gather4, facing) = (r("gather4"), r("facing"));
    let roto = Roto4::new(demo);
    let sin = |scale: f64, n: usize| -> Vec<i32> { (0..n).map(|k| tables::js_round(scale * (2.0 * std::f64::consts::PI * k as f64 / n as f64).sin())).collect() };
    let (sin60, sin60x2, sin120) = (cube::sin60(), sin(HALF as f64, 60), sin(48.0, 120));
    // the cube's corner cards for each axis, at this size and at the long sequence's: a table made once, before the
    // frames, from the constant corners; each frame takes its axis's
    let corners_at = |axis: usize, half: i32| -> Vec<i32> { cube::corner_cards(axis).chunks(3).flat_map(|c| [c[0], c[1], c[2] / cube::HALF * half]).collect() };
    let corners: Vec<[Vec<i32>; 2]> = (0..3).map(|axis| [corners_at(axis, HALF), corners_at(axis, cube::HALF)]).collect();
    let (edges, faces) = (cube::edge_cards(), cube::faces());
    let face_tri: Vec<i32> = faces.iter().flat_map(|f| f.tri.clone()).collect();
    // each face's diagonals, a to c and b to d, from its three corners a, b, c: d is the corner they don't share
    let face_diag: Vec<i32> = faces.iter().flat_map(|f| { let (a, b, c) = (f.tri[0], f.tri[1], f.tri[2]); [a, c, b, a ^ b ^ c] }).collect();
    let (no_text, black) = (vec![0; W], vec![0; W * 3]);
    let mut host = Host::new();
    let mut video = Video2::new(frames());
    for f in 0..frames() {
        let fr = frame(demo, &mut host, f as i32);
        let raw = host.go(&vtx, &[&[&[fr.angle, fr.axis as i32], &sin60x2], &[&corners[fr.axis][0]]], Some(24));
        // where the cube is: half the place and distance (for the small cube), then the whole, then the small cube's lens
        let place = fr.path.map(|(steps, _)| host.go(&path, &[&[&steps, &sin120], &[]], Some(7)));
        // the corners seen from the eye at a place: shifted for the tilt about it, then the perspective
        let seen = |host: &mut Host, corners: &[i32], at: &[i32], lens: i32| -> Vec<i32> {
            let shifted = host.go1(&proj, &[&[at[0], at[1], fr.tilt], corners], 16);
            let deck: Vec<i32> = (0..8).flat_map(|c| [shifted[2 * c], shifted[2 * c + 1], corners[3 * c + 2]]).collect();
            host.go1(&persp, &[&[at[2], lens], &deck], 24)
        };
        // the big cube's corners about the screen's centre, ready for project4 (at tilt 0 once in perspective)
        let (big, tilt) = match (&place, fr.path) { (Some(p), Some((_, lens))) => (seen(&mut host, &raw, &p[3..], lens), 0), _ => (raw.clone(), fr.tilt) };
        // the faces' light, from the cube at the long sequence's size, at half the place and distance
        let (mut palette, mut lit) = (BASE.to_vec(), vec![]);
        if fr.flevel > 0 {
            let small = host.go(&vtx, &[&[&[fr.angle, fr.axis as i32], &sin60], &[&corners[fr.axis][1]]], Some(24));
            let p = place.as_ref().unwrap();
            let small = seen(&mut host, &small, &p[..3], p[6]);   // its near face, face on, at its own size
            let corners = host.go1(&proj, &[&[48, 24, 0], &small], 16);
            let g = host.go1(&gather, &[&corners, &face_tri], 36);
            let light = host.go1(&shade, &[&g], 6);
            // which faces show, from the big cube's corners as seen, (x, y) without the depth
            let corners: Vec<i32> = big.chunks(3).flat_map(|c| [c[0], c[1]]).collect();
            let diagonals = host.go1(&gather4, &[&corners, &face_diag], 24);
            let deck: Vec<i32> = (0..6).flat_map(|i| { let mut x = vec![light[i]]; x.extend(&diagonals[4 * i..4 * i + 4]); x }).collect();
            lit = host.go1(&facing, &[&deck], 6);
            let deck: Vec<i32> = lit.iter().enumerate().flat_map(|(i, &v)| { let mut x = vec![v]; x.extend(cube::COLOURS[i]); x }).collect();
            palette.extend(host.go1(&pal, &[&deck], 18));
        } else { palette.extend([0; 18]); }                                      // no faces yet: never shown
        let face_deck: Vec<i32> = lit.iter().enumerate().flat_map(|(i, &v)| { let mut x = vec![v]; x.extend(&faces[i].edges); x }).collect();
        for (tx, ty) in [(0, 0), (W, 0), (0, H), (W, H)] {
            let corners = host.go1(&proj4, &[&[W as i32, H as i32, tx as i32, ty as i32, tilt], &big], 16);
            let sorted = host.go1(&esort, &[&corners, &edges], 48);
            let spans = host.gon(&raster, &[&sorted]);
            let behind = if fr.rlevel > 0 { roto.tile(&mut host, fr.rframe, tx, ty) } else { vec![] };
            for y in 0..H {
                let yc = y as i32;
                let mut codes = host.go(&comp, &[&[&[yc, 16], &spans], &[&no_text]], Some(W));
                if fr.flevel > 0 {
                    let fs = host.go(&fspan, &[&[&[yc], &spans], &[&face_deck]], Some(12));
                    codes = host.go(&solid, &[&[&[yc, fr.flevel], &fs], &[&codes]], Some(W));
                }
                let back = match fr.rlevel { 0 => black.clone(), 16 => behind[y].clone(), l => dissolve(&mut host, &mix, &black, &behind[y], l, yc) };
                let deck: Vec<i32> = (0..W).flat_map(|x| [codes[x], back[3 * x], back[3 * x + 1], back[3 * x + 2]]).collect();
                let mut rgb = host.go1(&cover, &[&palette, &deck], W * 3);
                if fr.out > 0 { rgb = dissolve(&mut host, &mix, &rgb, &behind[y], fr.out, yc); }
                let at = ((f * H2 + ty + y) * W2 + tx) * 3;
                for (i, &v) in rgb.iter().enumerate() { video.buf[at + i] = v as u8; }
            }
        }
    }
    host.done();
    let next = roto_next(demo, &mut Host::new());
    finish("tumble", &host, &video, &reference(refbin, &[]), &demo.join("tumble/video.rgb"), &format!("; the rotozoomer carries on at its frame {next}"));
}

/// The dissolve machine on one tile row: a dissolves into b at level (0-16).
fn dissolve(host: &mut Host, mix: &Machine, a: &[i32], b: &[i32], level: i32, y: i32) -> Vec<i32> {
    let mut deck = vec![level, y];
    for x in 0..W { deck.extend_from_slice(&a[3 * x..3 * x + 3]); deck.extend_from_slice(&b[3 * x..3 * x + 3]); }
    host.go1(mix, &[&deck], W * 3)
}
