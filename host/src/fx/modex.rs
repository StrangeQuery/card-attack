//! Mode X: the ray-traced mirror ball over the plasma floor at 320x240 (Mode X's square pixels), shown 4:3 in the
//! middle of the 1536x768 frame, VGA over MODE X in white outlined in black filling the sky behind the ball. The scene is rayplasma's (the ray's ball, bounce and entrance over the plasma's plane tilted right
//! back, its clock running on from the plasma's last frame) at 5 / 3 the size, more sky above and more floor below.
//!
//! Each frame the ray's path machine (rt_path) places the ball on the entrance's arcs or the script and sizes its
//! shadow, and the place machine (mx_place) turns that into pixels at this size. A row is four 96-pixel tiles, at
//! 0, 96, 192 and 224 across (the plasma's machines paint 96 pixels a run; the last tile's first 64 are the third's
//! again, and only its last 32 are filed). Per tile row: the row machine (mx_row) gives the floor's points (with the
//! shadow), the span machine (mx_span) the ball's pixels, and for each the normal (mx_norm), the bounce (mx_bounce)
//! and, for a ray that reaches the floor, the hit (mx_hit) give its look and the floor point it shows; the compose
//! machine (mx_compose) lays the ball's looks, u and v over the floor's, and the plasma's machines paint the row.
//!
//! The sign (in place of the setup's listing; modex/data/sign.txt, made by make_data.py): where a
//! lettered pixel isn't the ball's it is white, and black on its outline, as the titles of an earlier cut had:
//! the pixels within 3 of a letter, found once by the titles' role machine (tt_role) outlining the mask three
//! times over, each pass's outline and letters the next pass's mask. The ball falls
//! from the frame's top at frame 0 (modex/data/entrance.txt), the ray's script carried on from its 22nd frame.
//!
//! It files video.rgb, the 150 frames at 320x240 (the ray's 120-frame script and 30 more, carried on from its start:
//! the cut shows the first 66, the dorm's monitor the rest), and video_hd.rgb, the 150 at 1536x768: each 320x240
//! frame blitted to 1024x768 in the middle, pixel (x, y) of it showing (5 x / 16, 5 y / 16), black at the sides.
use crate::fx::plasma::{third, Plasma, FRAMES as PLASMA};
use crate::fx::ray::{script, SCRIPT};
use crate::fx::degauss::{self, Degauss};
use crate::fx::text::{self, Text};
use crate::*;
use std::path::Path;

/// The ray's script and 36 frames more (its sway and bounces carried on from its start), the degauss's.
pub const FRAMES: usize = SCRIPT + 36;
pub const MW: usize = 320;
pub const MH: usize = 240;
/// The tiles' places across a row.
const TILES: [i32; 4] = [0, 96, 192, 224];
/// Where in the ray's script Mode X carries on from: its hops land as the entrance's last arc does.
const SCRIPT_AT: usize = 22;
/// The sign's outline, in pixels (modex/reference.rs's OUTLINE): the role machine's passes over the mask.
const OUTLINE: usize = 3;
/// The rows the outline is found on: the sign's (6 to 93) and the outline's around them.
const SIGN_ROWS: usize = 100;
/// The role machine's two runs across a row: 192 columns from 0, and from 128.
const ROLE_AT: [usize; 2] = [0, 128];
/// The degauss over the last frames: off here, as it runs at the rotozoom's end instead (fx/rotodg.rs).
const DEGAUSS: bool = false;

/// For each pixel of the 1536x768 frame, the 320x240 pixel shown there (NONE at the sides).
pub fn on_frame() -> Vec<u32> {
    (0..text::HW * text::HH).map(|i| {
        let (x, y) = (i % text::HW, i / text::HW);
        if (256..1280).contains(&x) { ((y * 5 / 16) * MW + (x - 256) * 5 / 16) as u32 } else { text::NONE }
    }).collect()
}

pub fn render(demo: &Path, refbin: &str) {
    let plasma = Plasma::new(demo);
    let r = |n: &str| rust(demo, n);
    let (clock, path, place, row, span, norm, bounce, hit, compose) =
        (r("pl_clock"), r("rt_path"), r("mx_place"), r("mx_row"), r("mx_span"), r("mx_norm"), r("mx_bounce"), r("mx_hit"), r("mx_compose"));
    let script = script(demo);
    crate::fx::plasma::script(demo);   // its length starts the clock
    let arcs = numbers(&demo.join("modex/data/entrance.txt"));
    // the sign: its letters' pixels, then the outline's, the mask outlined OUTLINE times by the role machine
    let mut mask = vec![false; MW * SIGN_ROWS];
    for l in read(demo, "modex/data/sign.txt").lines().filter(|l| !l.starts_with('#')) {
        let p: Vec<usize> = l.split_whitespace().map(|v| v.parse().unwrap()).collect();
        mask[p[1] * MW + p[0]] = true;
    }
    let letters = mask.clone();
    let role = rust(demo, "tt_role");
    let mut host = Host::new();
    for _ in 0..OUTLINE {
        let on = |x: i32, y: i32| x >= 0 && y >= 0 && (x as usize) < MW && (y as usize) < SIGN_ROWS && mask[y as usize * MW + x as usize];
        let mut grown = vec![false; MW * SIGN_ROWS];
        for y in 0..SIGN_ROWS as i32 {
            for &x0 in &ROLE_AT {
                let mut deck = vec![1, 0];   // outlined, no shadow
                for x in x0 as i32 - 1..x0 as i32 + 193 {
                    deck.push(100 * on(x, y - 1) as i32 + 10 * on(x, y) as i32 + on(x, y + 1) as i32);
                    deck.push(7);
                }
                let out = host.go1(&role, &[&deck], 2 * 192);
                for i in 0..192 { if out[2 * i] > 0 { grown[y as usize * MW + x0 + i] = true; } }
            }
        }
        mask = grown;
    }
    let sign: Vec<(usize, usize, u8)> = (0..MW * SIGN_ROWS).filter(|&i| mask[i])
        .map(|i| (i % MW, i / MW, if letters[i] { 255 } else { 0 })).collect();
    let kit = Text::new(demo);
    let mut video = cardiac_host::Video::<MW, MH>::new(FRAMES);
    let mut hd = cardiac_host::Video::<{ text::HW }, { text::HH }>::new(FRAMES);
    let at = on_frame();
    let mut degauss = Degauss::new(demo);
    let mut rays = 0;
    for f in live::frames(FRAMES) {
        let [sx, sy, _, _] = script[(f + SCRIPT_AT) % SCRIPT];
        let times = host.go1(&clock, &[&[(PLASMA + f) as i32]], 5);
        let ball = host.go1(&path, &[&[f as i32, sx, sy], &arcs], 4);
        let (bx, radius, dark) = (ball[0], ball[2], ball[3]);
        let placed = host.go1(&place, &[&[ball[0], ball[1]]], 4);
        let (cx, cy, centre, across) = (placed[0], placed[1], placed[2], placed[3]);
        let mut ball_at: Vec<Option<(i32, i32)>> = vec![None; MH];   // the ball's pixels across each row, from and to
        for y in 0..MH as i32 {
            for &tx in &TILES {
                let floor = host.go1(&row, &[&[y, tx, bx, radius, dark]], 3 * W);
                let sp = host.gon(&span, &[&[y, cx, cy, tx]]);
                let (x0, n) = (sp[0], sp[1]);
                let looks: Vec<Vec<i32>> = if n > 0 {
                    let b = sp[2];
                    let mut seen = [vec![], vec![], vec![]];   // the ball's pixels: looks, u, v
                    for &a in &sp[3..] {
                        let o = host.go1(&norm, &[&[a, b]], 4);
                        let (c, rx, ry, rz) = (o[0], o[1], o[2], o[3]);
                        let mut p = host.go1(&bounce, &[&[rx, ry, b, centre]], 3); rays += 1;
                        if p[0] < 0 { p = host.go1(&hit, &[&[p[1], rx, rz, a, c, across]], 3); }
                        for k in 0..3 { seen[k].push(p[k]); }
                    }
                    // each background card, and after it the ball's where the ball covers it
                    (0..3).map(|k| {
                        let back = third(&floor, k);
                        let mut deck = vec![x0, n];
                        for (x, &v) in back.iter().enumerate() {
                            deck.push(v);
                            let i = x as i32 - x0;
                            if i >= 0 && i < n { deck.push(seen[k][i as usize]); }
                        }
                        host.go1(&compose, &[&deck], W)
                    }).collect()
                } else {
                    (0..3).map(|k| third(&floor, k)).collect()
                };
                let rgb = plasma.paint(&mut host, &times, &looks[0], &looks[1], &looks[2]);
                // the last tile files only what the third didn't
                let skip = if tx == 224 { 64 } else { 0 };
                let to = ((f * MH + y as usize) * MW + tx as usize + skip) * 3;
                for (i, &v) in rgb[skip * 3..].iter().enumerate() { video.buf[to + i] = v as u8; }
                if n > 0 {                                     // the ball's run across the row, over the tiles
                    let (a, b) = (tx + x0, tx + x0 + n);
                    ball_at[y as usize] = Some(match ball_at[y as usize] { Some((a0, b0)) => (a0.min(a), b0.max(b)), None => (a, b) });
                }
            }
        }
        // the sign: white letters, their outline black, where the ball doesn't cover them
        for &(x, y, c) in &sign {
            if let Some((a, b)) = ball_at[y] { if (x as i32) >= a && (x as i32) < b { continue; } }
            let to = ((f * MH + y) * MW + x) * 3;
            video.buf[to..to + 3].fill(c);
        }

        // at 1536x768: the frame (blotched, near the end) blitted, the degauss's sign and jolt
        let blotched = if DEGAUSS { degauss.strength(&mut host, f).map(|s| degauss.blotch(&mut host, s, video.frame(f))) } else { None };
        let pic = blotched.as_deref().unwrap_or(video.frame(f));
        let frame = &mut hd.buf[f * text::HW * text::HH * 3..][..text::HW * text::HH * 3];
        for (i, &p) in at.iter().enumerate() {
            if p != text::NONE { frame[3 * i..3 * i + 3].copy_from_slice(&pic[3 * p as usize..3 * p as usize + 3]); }
        }
        if DEGAUSS && degauss::sign_shown(f) { degauss.lay_sign(&mut host, &kit, frame, &at, pic); }
        if DEGAUSS && (degauss::D0..degauss::D0 + degauss::DN).contains(&f) {
            let jolted = degauss.jolt(&mut host, f - degauss::D0, frame, &at);
            frame.copy_from_slice(&jolted);
        }
    }
    host.done();
    let both = reference(refbin, &[demo]);
    let lo_end = FRAMES * MW * MH * 3;
    let part = |a: usize, b: usize| &both[a.min(both.len())..b.min(both.len())];
    finish("modex (320x240)", &host, &video, part(0, lo_end), &demo.join("modex/video.rgb"), &format!(", {rays} reflected rays"));
    finish("modex (1536x768)", &host, &hd, part(lo_end, both.len()), &demo.join("modex/video_hd.rgb"), "");
}
