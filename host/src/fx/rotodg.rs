//! The degauss at the rotozoom's end (segments/rotodg; moved from Mode X's end, where the dorm now
//! shows the monitor small; the transition into the VGA plasma mid-degauss): 36 frames at 1536x768, the rotozoom's
//! frames 36..63 (bbs/video.rgb) and then the plasma's 0..7 (vga/video.rgb), each 192x96 shown 8 x 8 a pixel, under
//! Mode X's degauss (fx/degauss.rs, at the picture's size: its falloffs modex/data/gauss_roto.txt) as if its frames
//! 114..149: the blotch, the DEGAUSS sign blinking, the jolt from frame 20, the cut to the plasma at 28.
use crate::fx::degauss::{self, Degauss};
use crate::fx::text::{self, Text};
use crate::*;
use std::path::Path;

pub const FRAMES: usize = 36;
const W: usize = 192;
const H: usize = 96;
const ROTO0: usize = 36;
const CUT: usize = 28;
const K0: usize = 114;
pub const READS: [&str; 2] = ["bbs/video.rgb", "vga/video.rgb"];

pub fn render(demo: &Path, refbin: &str) {
    let roto = live::fetch(&demo.join(READS[0]));
    let vga = live::fetch(&demo.join(READS[1]));
    let size = W * H * 3;
    let kit = Text::new(demo);
    let mut host = Host::new();
    let mut hd = cardiac_host::Video::<{ text::HW }, { text::HH }>::new(FRAMES);
    // each frame pixel's picture pixel: 8 x 8 a pixel
    let at: Vec<u32> = (0..text::HW * text::HH).map(|i| (((i / text::HW) / 8) * W + (i % text::HW) / 8) as u32).collect();
    let mut dg = Degauss::picture(demo, W, H, "gauss_roto.txt");
    for j in live::frames(FRAMES) {
        let k = K0 + j;
        let pic = if j < CUT { &roto[(ROTO0 + j) * size..][..size] } else { &vga[(j - CUT) * size..][..size] };
        let blotched = dg.strength(&mut host, k).map(|s| dg.blotch(&mut host, s, pic));
        let pic = blotched.as_deref().unwrap_or(pic);
        let frame = &mut hd.buf[j * text::HW * text::HH * 3..][..text::HW * text::HH * 3];
        for (i, &p) in at.iter().enumerate() { frame[3 * i..3 * i + 3].copy_from_slice(&pic[3 * p as usize..3 * p as usize + 3]); }
        if degauss::sign_shown(k) { dg.lay_sign(&mut host, &kit, frame, &at, pic); }
        if (degauss::D0..degauss::D0 + degauss::DN).contains(&k) {
            let jolted = dg.jolt(&mut host, k - degauss::D0, frame, &at);
            frame.copy_from_slice(&jolted);
        }
    }
    host.done();
    finish("rotodg (1536x768)", &host, &hd, &reference(refbin, &[demo]), &demo.join("rotodg/video_hd.rgb"), "");
}
