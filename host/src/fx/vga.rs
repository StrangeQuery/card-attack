//! VGA: the climb from the EGA to the VGA. The plasma in sixteen colours, as a 16-colour plasma's palette had them:
//! each pixel's place on the colour ring picks the colours in their order (black, blue, green, ... white);
//! CompuServe's page with the mode 13h code, in 40-column text, arriving over half a second as that plasma
//! crossfades into it, held two seconds with its cursor blinking; a hard cut back to the plasma, still in sixteen
//! colours; then the 256 colours wiping down over it, a white line at the edge, and the VGA's brag as it lands (a
//! layer for the cut's over line).
//!
//! Per frame of the 16-colour plasma, per tile row, the plasma's own machines work out each pixel's place on the
//! ring (fx/plasma.rs `places`: the row, ring, rings, wave and slant machines); the host fetches its colour's code
//! from the printed table (vga/data/ansi.txt) by that place, and paint16 colours them from the EGA's palette
//! (vga/data/ega.txt). The wipe: the host takes the rows above the edge from the plasma's own frame
//! (plasma/video.rgb), the edge's row white, the rest the 16-colour frame's (vga/data/wipe.txt: the edge a frame).
//! The page, the crossfade and the brag are the text kit's (fx/text.rs).
//!
//! It files three: video.rgb, 44 frames at 192x96 (the 16-colour plasma of the plasma's frames 48..77, the cut
//! mixing the rotozoomer into its first six; then of 90..97; then the wipe over 98..103); video_hd.rgb, 30 frames at
//! 1536x768 (six of the 16-colour plasma's frames 24..29 crossfading into the page as it arrives, 24 of the page);
//! brag.rgba, the brag's layer (704x192 at 64, 528), for the plasma's frames 104..145.
use crate::*;
use crate::fx::plasma::{third, Plasma};
use crate::fx::text::{self, pages, Shown, Text};
use std::path::Path;

/// The 16-colour plasma's frames: the plasma's frame each shows (the intro, back after the page, the wipe).
pub const INTRO: usize = 30;
pub const BACK0: usize = 90;
pub const BACK: usize = 8;
pub const WIPE: usize = 6;
pub const EGA_FRAMES: usize = INTRO + BACK + WIPE;
pub fn plasma_frame(f: usize) -> usize { if f < INTRO { 48 + f } else { BACK0 + f - INTRO } }
/// Frames of a crossfade (its steps 1..=6 of 7), frames of the page held.
pub const X: usize = 6;
pub const HOLD: usize = 24;
pub const HD_FRAMES: usize = X + HOLD;

pub fn render(demo: &Path, refbin: &str) {
    let plasma = live::fetch(&demo.join("plasma/video.rgb"));
    let palette: Vec<i32> = read(demo, "vga/data/ega.txt").lines().filter(|l| !l.starts_with('#')).flat_map(|l| l.split_whitespace().map(|v| v.parse::<i32>().unwrap()).collect::<Vec<_>>()).collect();
    let printed = |f: &str| -> Vec<i32> { read(demo, f).lines().filter(|l| !l.starts_with('#')).flat_map(|l| l.split_whitespace().map(|v| v.parse::<i32>().unwrap()).collect::<Vec<_>>()).collect() };
    let (ansi, wipe) = (printed("vga/data/ansi.txt"), printed("vga/data/wipe.txt"));
    let paint = rust(demo, "paint16");
    let (pl, clock, script) = (Plasma::new(demo), rust(demo, "pl_clock"), fx::plasma::script(demo));
    let kit = Text::new(demo);
    // a host each for the three it files, so each says what it took
    let (mut host, mut lo, mut once) = (Host::new(), Host::new(), Host::new());
    // the 16-colour plasma: each pixel's place on the ring, its code by the printed table, painted
    let mut ega = Video2::new(EGA_FRAMES);
    for f in 0..EGA_FRAMES {
        let pf = plasma_frame(f);
        let times = lo.go1(&clock, &[&[pf as i32]], 5);
        for (tx, ty) in [(0, 0), (W as i32, 0), (0, H as i32), (W as i32, H as i32)] {
            for y in 0..H as i32 {
                let points = pl.points(&mut lo, ty + y, tx, script[pf], [0, 0, 0]);
                let places = pl.places(&mut lo, &times, &third(&points, 1), &third(&points, 2));
                let codes: Vec<i32> = places.iter().map(|&q| ansi[q as usize]).collect();
                let rgb = lo.go1(&paint, &[&palette, &codes], 3 * W);
                let at = ((f * H2 + (ty + y) as usize) * W2 + tx as usize) * 3;
                ega.buf[at..at + 3 * W].iter_mut().zip(&rgb).for_each(|(p, &v)| *p = v as u8);
            }
        }
        // the wipe: the rows above the edge in 256 colours, the edge's row white
        if f >= INTRO + BACK {
            let edge = wipe[f - INTRO - BACK] as usize;
            let row = W2 * 3;
            for y in 0..edge.min(H2) { ega.buf[(f * H2 + y) * row..][..row].copy_from_slice(&plasma[(pf * H2 + y) * row..][..row]); }
            if edge < H2 { ega.buf[(f * H2 + edge) * row..][..row].fill(255); }
        }
    }
    // the page and the crossfade into it, at 1536x768
    let page = &pages::COMPUSERVE;
    let total = page.chars() as i32;
    let (big, on) = (text::big8(), text::on_frame(page));
    // how many characters have arrived at each step of the crossfade in, and each step's cover
    let arrived = kit.steps(&mut host, total, &(1..=X as i32).map(|j| (j, X as i32)).collect::<Vec<_>>());
    let covers = kit.steps(&mut host, 32, &(1..=X as i32).map(|j| (j, X as i32 + 1)).collect::<Vec<_>>());
    let mut hd = cardiac_host::Video::<{ text::HW }, { text::HH }>::new(HD_FRAMES);
    for f in live::frames(HD_FRAMES) {
        let frame = if f < X {
            let px = kit.draw(&mut host, page, arrived[f] as usize, true);
            kit.crossfade(&mut host, &Shown { px: ega.frame(INTRO - X + f), at: &big }, &Shown { px: &px, at: &on }, covers[f])
        } else {
            Text::blit(&Shown { px: &kit.draw(&mut host, page, total as usize, (f - X) / 6 % 2 == 0), at: &on })
        };
        hd.buf[f * frame.len()..(f + 1) * frame.len()].copy_from_slice(&frame);
    }
    // the brag's layer: the page's pixels four times over, all of it covering (its box is black)
    let brag = &pages::VGA_BRAG;
    let px = kit.draw(&mut once, brag, brag.chars(), false);
    let at = text::on_frame(brag);
    let mut layer = cardiac_host::Video::<704, 256>::new(1);
    let (bw, bh) = (brag.w * brag.sx, brag.h * brag.sy);
    assert_eq!(bw * bh * 4, layer.buf.len());
    for y in 0..bh { for x in 0..bw {
        let i = at[(brag.y + y) * text::HW + brag.x + x] as usize;
        layer.buf[(y * bw + x) * 4..][..4].copy_from_slice(&[px[3 * i], px[3 * i + 1], px[3 * i + 2], 32]);
    } }
    for h in [&mut host, &mut lo, &mut once] { h.done(); }

    let both = reference(refbin, &[demo]);
    let (lo_end, hd_end) = (EGA_FRAMES * FRAME2, EGA_FRAMES * FRAME2 + HD_FRAMES * text::HW * text::HH * 3);
    let part = |a: usize, b: usize| &both[a.min(both.len())..b.min(both.len())];
    finish("vga (EGA, 192x96)", &lo, &ega, part(0, lo_end), &demo.join("vga/video.rgb"), "");
    finish("vga (1536x768)", &host, &hd, part(lo_end, hd_end), &demo.join("vga/video_hd.rgb"), "");
    finish("vga (the brag's layer)", &once, &layer, part(hd_end, both.len()), &demo.join("vga/brag.rgba"), " (704x192 RGB and cover, for the cut's over at 64 528)");
}
