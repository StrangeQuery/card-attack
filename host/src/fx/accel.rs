//! The accelerated drive: the drive at 192x96 as the software renderer draws it (accel_hd.rs's frame_soft) to its
//! frame 112, going on under the VIDEO OPTIONS menu (the UI kit's layer, ui/videoopts.rgba, laid on by the cut) from
//! frame 66, each level there dimmed by the dim machine (adim: 11 v / 20); then the accelerated half (accel_hd.rs).
//!
//! It files video.rgb, the 113 frames at 192x96.
use crate::fx::accel_hd::{Hd, RH, RW};
use crate::fx::text::{self, pages, Text};
use crate::*;
use std::collections::HashMap;
use std::path::Path;

/// The software drive's frames, then the menu's.
pub const SOFT: usize = 66;
pub const MENU: usize = 47;
pub const LOW: usize = SOFT + MENU;
/// The accelerated half: drive frames HD0..END (the drive goes on under the menu and the blank); the calls held to
/// its frame HELD, gone at GONE.
pub const HD0: usize = 118;
pub const END: usize = 236;
pub const HD: usize = END - HD0;
const HELD: usize = 24;
const GONE: usize = 36;

pub fn render(demo: &Path, refbin: &str) {
    let dim = rust(demo, "adim");
    let mut host = Host::new();
    let mut video = Video2::new(LOW);
    let hd = Hd::new(demo);
    // the software half: the drive at 192x96 to frame 112, from frame 66 under the menu each level dimmed (adim:
    // 11 v / 20)
    let soft_frames = std::env::var("ACCEL_SOFT_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(LOW);
    for f in live::frames(soft_frames) {
        let mut pic = hd.frame_soft(&mut host, f as i32);
        if f >= SOFT {
            // a row at a time (a card counts them)
            pic = pic.chunks(W2 * 3).flat_map(|row| {
                let deck: Vec<i32> = row.iter().map(|&v| v as i32).collect();
                host.go1(&dim, &[&[deck.len() as i32], &deck], deck.len()).into_iter().map(|v| v as u8).collect::<Vec<u8>>()
            }).collect();
        }
        video.buf[f * FRAME2..(f + 1) * FRAME2].copy_from_slice(&pic);
    }
    // the accelerated half: each frame at 768x384, 16-bit, shown 2 x 2; the calls laid over its first 36
    let frames = std::env::var("ACCEL_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(HD);
    let mut big = cardiac_host::Video::<{ text::HW }, { text::HH }>::new(frames);
    let kit = Text::new(demo);
    let page = &pages::ACCEL;
    // (the calls' letters only for a block that shows them)
    let calls = live::range(frames).start < GONE;
    let letters = if calls { kit.draw(&mut host, page, page.chars(), false) } else { vec![] };
    let page_at = text::on_frame(page);
    let ink = |x: usize, y: usize| { let i = page_at[y * text::HW + x]; calls && i != text::NONE && letters[3 * i as usize] > 0 };
    let shadowed: Vec<usize> = (0..text::HW * text::HH).filter(|&i| { let (x, y) = (i % text::HW, i / text::HW); x >= 3 && y >= 3 && ink(x - 3, y - 3) }).collect();
    let lettered: Vec<usize> = (0..text::HW * text::HH).filter(|&i| ink(i % text::HW, i / text::HW)).collect();
    let at = |i: usize| ((i / text::HW / 2) * RW + i % text::HW / 2) as u32;
    let shadow_at: std::collections::HashSet<usize> = shadowed.iter().copied().collect();
    let keys: Vec<(u32, Option<()>)> = lettered.iter().map(|&i| (at(i), shadow_at.contains(&i).then_some(()))).collect();
    let fading = if calls { kit.steps(&mut host, 32, &(HELD + 1..GONE).map(|k| ((GONE - k) as i32, 12)).collect::<Vec<_>>()) } else { vec![] };
    for k in live::frames(frames) {
        let px = hd.frame(&mut host, (HD0 + k) as i32);
        let pic = hd.rgb565(&mut host, &px);
        let frame = &mut big.buf[k * text::HW * text::HH * 3..][..text::HW * text::HH * 3];
        for i in 0..text::HW * text::HH { let p = at(i) as usize; frame[3 * i..3 * i + 3].copy_from_slice(&pic[3 * p..3 * p + 3]); }
        let cover = if k <= HELD { 32 } else if k < GONE { fading[k - HELD - 1] } else { 0 };
        if cover > 0 {
            let dark = lay(&kit, &mut host, &shadowed.iter().map(|&i| (at(i), None)).collect::<Vec<_>>(), &pic, &HashMap::new(), 0, cover);
            let light = lay(&kit, &mut host, &keys, &pic, &dark, 255, cover);
            for &i in &shadowed { frame[3 * i..3 * i + 3].copy_from_slice(&dark[&(at(i), None)]); }
            for (&i, key) in lettered.iter().zip(&keys) { frame[3 * i..3 * i + 3].copy_from_slice(&light[key]); }
        }
    }
    let _ = RH;
    host.done();
    let both = reference(refbin, &[demo]);
    let lo_end = LOW * FRAME2;
    let part = |a: usize, b: usize| &both[a.min(both.len())..b.min(both.len())];
    finish("accel (192x96)", &host, &video, part(0, lo_end), &demo.join("accel/video.rgb"), "");
    finish("accel (1536x768)", &host, &big, part(lo_end, both.len()), &demo.join("accel/video_hd.rgb"), "");
}

/// Mix each distinct pixel toward `to` by `cover` 32nds once on umix (Mode X's): a pixel of the 768x384 frame, as it
/// is, or (Some) as the shadow left it (`under`'s). Returns each one's colour.
fn lay(kit: &Text, host: &mut Host, keys: &[(u32, Option<()>)], pic: &[u8], under: &HashMap<(u32, Option<()>), [u8; 3]>, to: i32, cover: i32) -> HashMap<(u32, Option<()>), [u8; 3]> {
    let mut order: Vec<(u32, Option<()>)> = vec![];
    let mut seen = std::collections::HashSet::new();
    for &k in keys { if seen.insert(k) { order.push(k); } }
    let abc: Vec<[i32; 3]> = order.iter().flat_map(|&(p, s)| {
        let from = if s.is_some() { under[&(p, None)] } else { [pic[3 * p as usize], pic[3 * p as usize + 1], pic[3 * p as usize + 2]] };
        (0..3).map(move |c| [from[c] as i32, to, cover])
    }).collect();
    let mixed = kit.mix(host, &abc);
    order.iter().enumerate().map(|(n, &k)| (k, [mixed[3 * n] as u8, mixed[3 * n + 1] as u8, mixed[3 * n + 2] as u8])).collect()
}
