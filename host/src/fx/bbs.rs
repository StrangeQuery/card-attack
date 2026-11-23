//! The BBS: the cube in front of a BBS's message reader, and then of the rotozoomer.
//! The cube's frames are the tumble's (tumble/video.rgb, its frames 14..147) and the die's and the Rubik's cube's
//! (dierubik/video.rgb); the rotozoomer's are its own render (roto/video4.rgb), as the swarm's screens read them.
//!
//! bbs, video_hd.rgb, 1536x768, 272 frames: the BBS page (pages.rs's BBS: 80-column ANSI text in the IBM CGA font,
//! 16 colours) arriving 20 characters a frame, the cursor blinking, the cube in front of it; at the end the page
//! crossfades into the rotozoomer (eight frames) behind the cube. How many characters have arrived, and the
//! crossfade's cover, are the ease machine's (ui_ease, then rb_hold at the page's end); the page is the text kit's
//! (ui_page, umix: fx/text.rs), drawn once for each number of characters and blink it shows.
//!
//! bbsroto, video.rgb, 192x96, 72 frames: the rotozoomer, its second half wobbling (each row pushed sideways, wrapping
//! round: dr_clock gives each row's angle, rb_sine its sine, dr_wobble the push, the wobble growing in over 15
//! frames), the cube roaming in front of it. The cut mixes its last six frames into the EGA plasma.
//!
//! Where the cube is: the key machine (dr_key) says, per pixel of the cube's frames, whether it is black. The host
//! copies each computed pixel where it goes: the cube's (8 x 8 on the 1536x768 frame), the page's, the rotozoomer's
//! (pushed along its row by the punched push).
use crate::*;
use crate::fx::text::{self, pages, Shown, Text};
use crate::fx::train::{imp::{v, Ops, K, V}, Cardiac};
use std::collections::HashMap;
use std::path::Path;

/// Frames of the cube over the page: the tumble's (from its frame TUMBLE0), then the die's and the Rubik's cube's;
/// then over the page crossfading into the rotozoomer (from its frame ROTO_IN).
pub const TUMBLE0: usize = 14;
pub const TUMBLE: usize = 134;
pub const CUBE: usize = 130;
pub const CROSS: usize = 8;
pub const ROTO_IN: usize = 42;
pub const HD_FRAMES: usize = TUMBLE + CUBE + CROSS;
/// The page's first character arrives at frame DELAY, 20 a frame.
pub const DELAY: usize = 6;
/// The rotozoomer's frames after the crossfade, the cube's frame at their first; the wobble from frame WOBBLE.
pub const ROTO0: usize = ROTO_IN + CROSS;
pub const ROTO_FRAMES: usize = 72;
pub const WOBBLE: usize = 36;

/// Where the cube is in each row of one of its frames: 1 where it isn't black (the key machine).
fn keys(host: &mut Host, key: &Machine, frame: &[u8]) -> Vec<i32> {
    let mut out = Vec::with_capacity(W2 * H2);
    for row in frame.chunks(W2 * 3) {
        let deck: Vec<i32> = std::iter::once(W2 as i32).chain(row.iter().map(|&v| v as i32)).collect();
        out.extend(host.go1(key, &[&deck], W2));
    }
    out
}

pub fn render(demo: &Path, refbin: &str) {
    let (tumble, cube, roto) = (live::fetch(&demo.join("tumble/video.rgb")), live::fetch(&demo.join("dierubik/video.rgb")), live::fetch(&demo.join("roto/video4.rgb")));
    let kit = Text::new(demo);
    let (key, hold) = (rust(demo, "dr_key"), rust(demo, "rb_hold"));
    let mut host = Host::new();
    let mut o = Cardiac::new(demo);
    let page = &pages::BBS;
    let (big, on) = (text::big8(), text::on_frame(page));
    // how many characters have arrived at each frame (20 a frame from frame DELAY, held at the page's end): the
    // frame's step from DELAY, held to 0..45, on the train's operations; the crossfade's covers
    let ks: V = (0..HD_FRAMES as i64).collect();
    let since = o.sum(&[(v(&ks), 1)], -(DELAY as i64));
    let since = o.clamp(v(&since), 0, 45);
    let steps: Vec<(i32, i32)> = since.iter().map(|&j| (j as i32, 45)).collect();
    let arrived = kit.steps(&mut host, 900, &steps);
    let mut deck = vec![arrived.len() as i32, 0, page.chars() as i32];
    deck.extend(&arrived);
    deck.push(0);
    let arrived = host.go1(&hold, &[&deck], HD_FRAMES);
    let covers = kit.steps(&mut host, 32, &(1..=CROSS as i32).map(|j| (j, CROSS as i32 + 1)).collect::<Vec<_>>());
    let mut drawn: HashMap<(i32, bool), Vec<u8>> = HashMap::new();
    let mut video = cardiac_host::Video::<{ text::HW }, { text::HH }>::new(HD_FRAMES);
    for k in live::frames(HD_FRAMES) {
        // the cursor on for six frames, off for six
        let (sixes, _) = o.divmod(K(k as i64), K(6));
        let (_, off) = o.divmod(v(&sixes), K(2));
        let cursor = off[0] == 0;
        let px = drawn.entry((arrived[k], cursor)).or_insert_with(|| kit.draw(&mut host, page, arrived[k] as usize, cursor)).clone();
        let front = if k < TUMBLE { &tumble[(TUMBLE0 + k) * FRAME2..][..FRAME2] } else { &cube[(k - TUMBLE) * FRAME2..][..FRAME2] };
        let mut frame = if k < TUMBLE + CUBE {
            Text::blit(&Shown { px: &px, at: &on })
        } else {
            let j = k - TUMBLE - CUBE;
            kit.crossfade(&mut host, &Shown { px: &px, at: &on }, &Shown { px: &roto[(ROTO_IN + j) * FRAME2..][..FRAME2], at: &big }, covers[j])
        };
        // the cube in front, where it is
        let here = keys(&mut host, &key, front);
        for (i, &at) in big.iter().enumerate() { if here[at as usize] == 1 { frame[3 * i..3 * i + 3].copy_from_slice(&front[3 * at as usize..3 * at as usize + 3]); } }
        video.buf[k * frame.len()..(k + 1) * frame.len()].copy_from_slice(&frame);
    }
    host.steps += o.host.steps; host.runs += o.host.runs;
    host.done();
    finish("bbs (1536x768)", &host, &video, &reference(refbin, &[demo, Path::new("hd")]), &demo.join("bbs/video_hd.rgb"), "");
}

pub fn render_roto(demo: &Path, refbin: &str) {
    let (cube, roto) = (live::fetch(&demo.join("dierubik/video.rgb")), live::fetch(&demo.join("roto/video4.rgb")));
    let r = |n: &str| rust(demo, n);
    let (key, clock, sine, wobble, hold) = (r("dr_key"), r("dr_clock"), r("rb_sine"), r("dr_wobble"), r("rb_hold"));
    let table = numbers(&demo.join("rubik/data/sine.txt"));
    let rates = numbers(&demo.join("dierubik/data/rates.txt"));
    let (down, on) = (&rates[30..33], &rates[33..36]);
    let mut host = Host::new();
    let mut o = Cardiac::new(demo);
    let mut video = Video2::new(ROTO_FRAMES);
    for k in live::frames(ROTO_FRAMES) {
        let back = &roto[(ROTO0 + k) * FRAME2..][..FRAME2];
        let front = &cube[(CUBE + CROSS + k) * FRAME2..][..FRAME2];
        // each row's push: its wave's angle (9 degrees a row, -0.5 radians a frame), its sine, the push
        let push = if k < WOBBLE { vec![0; H2] } else {
            // the wobble's frame from its start, on the train's operations
            let w = o.sum(&[(K(k as i64), 1)], -(WOBBLE as i64))[0] as i32;
            let grown = host.go1(&hold, &[&[1, 0, 15, w, 0]], 1)[0];
            let mut deck = vec![H2 as i32];
            for y in 0..H2 as i32 { deck.extend([2, y]); deck.extend(down); deck.push(w); deck.extend(on); }
            let angles = host.go1(&clock, &[&deck], 2 * H2);
            let mut deck = vec![grown, H2 as i32];
            for a in angles.chunks(2) { deck.push(host.go1(&sine, &[a, &table, &table], 2)[0]); }
            host.go1(&wobble, &[&deck], H2)
        };
        let here = keys(&mut host, &key, front);
        let mut img = vec![0u8; FRAME2];
        for y in 0..H2 { for x in 0..W2 {
            let i = y * W2 + x;
            let from = y * W2 + (x as i32 - push[y]).rem_euclid(W2 as i32) as usize;
            let src = if here[i] == 1 { &front[3 * i..3 * i + 3] } else { &back[3 * from..3 * from + 3] };
            img[3 * i..3 * i + 3].copy_from_slice(src);
        } }
        video.buf[k * FRAME2..(k + 1) * FRAME2].copy_from_slice(&img);
    }
    host.steps += o.host.steps; host.runs += o.host.runs;
    host.done();
    finish("bbsroto", &host, &video, &reference(refbin, &[demo, Path::new("roto")]), &demo.join("bbs/video.rgb"), "");
}
