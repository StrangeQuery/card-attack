// Accelerated drive reference: the beat of the drive that goes 3D accelerated, in ordinary Rust. Arg: the demo's
// segments folder. Writes, as the host files them:
//
// 1. video.rgb, 192x96: the drive's first 113 frames as the software renderer draws them (hd.rs's frame_soft), the
//    last 47 dimmed (each level times 0.55, rounded down) under the VIDEO OPTIONS menu (the menu itself is the UI
//    kit's layer, ui/videoopts.rgba, laid on by the cut).
// 2. video_hd.rgb, 1536x768: the accelerated half, drive frames 66-183 (hd.rs) at 768x384 through the 16-bit
//    dither, each pixel shown 2 x 2; over its first 36 frames the calls (ui/pages.rs's ACCEL, drawn by ui/text.rs):
//    their letters white, their shadow 3 pixels down and right black, each pixel taken toward them by 32 32nds for
//    frames 0..24, then 32 (36 - k) / 12 (rounded down) at frame k, a + (b - a) c / 32 toward zero, the shadow
//    first. (The brag is the UI kit's layer, ui/accelbrag.rgba, laid on by the cut.)
use std::io::Write;

#[allow(dead_code)]
#[path = "../drive/reference.rs"]
mod drive;
#[path = "hd.rs"]
mod hd;
#[allow(dead_code)]
#[path = "../ui/pages.rs"]
mod pages;
#[allow(dead_code)]
#[path = "../ui/text.rs"]
mod text;

/// The software drive's frames; the menu's.
const SOFT: i32 = 66;
const MENU: usize = 47;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() > 2 && a[2] == "dump" {
        // the march's rows at each sample column, for checking the machines: s y a4 off pool fk zd
        let d = hd::load(&a[1]);
        let cam = hd::Cam::new(&d, a[3].parse().unwrap());
        for s in 0..=24 { for (y, g) in hd::march(&d, &cam, 32 * s, std::env::var("ACCEL_SOFT").is_ok()).iter().enumerate() {
            if let Some(g) = g { println!("{} {} {} {} {} {} {}", s, y, g.a4, g.off, g.pool, g.fk, g.zd); }
        } }
        return;
    }
    if a.len() > 2 && a[2] == "soft" {
        // the software half's frames a[3] to a[4] at 192x96, undimmed
        let d = hd::load(&a[1]);
        let mut out = std::io::stdout().lock();
        for f in a[3].parse::<i64>().unwrap()..a[4].parse::<i64>().unwrap() { out.write_all(&hd::frame_soft(&d, f)).unwrap(); }
        return;
    }
    if a.len() > 2 && a[2] == "hd" {
        // a look at the accelerated half: its frames at 768x384, from frame a[3] to a[4]
        let d = hd::load(&a[1]);
        let mut out = std::io::stdout().lock();
        for f in a[3].parse::<i64>().unwrap()..a[4].parse::<i64>().unwrap() {
            let px = hd::frame(&d, f);
            let mut rgb = vec![];
            hd::rgb565(&d, &px, &mut rgb);
            out.write_all(&rgb).unwrap();
        }
        return;
    }
    let d = hd::load(&a[1]);
    let mut out = std::io::stdout().lock();
    // the software half: the drive at 192x96 as the software renderer draws it (hd.rs's frame_soft), frames 0..113;
    // from frame 66 under the menu, each level times 0.55 (11 v / 20, rounded down)
    let soft_frames = std::env::var("ACCEL_SOFT_FRAMES").ok().and_then(|v| v.parse::<i64>().ok()).unwrap_or(SOFT as i64 + MENU as i64);
    for f in 0..soft_frames {
        let mut rgb = hd::frame_soft(&d, f);
        if f >= SOFT as i64 { for v in rgb.iter_mut() { *v = (*v as i32 * 11 / 20) as u8; } }
        out.write_all(&rgb).unwrap();
    }
    let page = &pages::ACCEL;
    let glyphs = text::Glyphs::read(std::path::Path::new(&a[1]));
    let letters = text::draw(page, &glyphs, page.chars(), false);
    let letter = |x: usize, y: usize| text::on_page(page, x, y).map_or(false, |i| letters[3 * i] > 0);
    let last = std::env::var("ACCEL_FRAMES").ok().and_then(|v| v.parse::<i64>().ok()).map_or(hd::END, |n| hd::SOFT + n);
    for f in hd::SOFT..last {
        let mut rgb = vec![];
        hd::rgb565(&d, &hd::frame(&d, f), &mut rgb);
        let k = (f - hd::SOFT) as usize;
        let level = if k <= 24 { 32 } else if k < 36 { (32 * (36 - k) / 12) as u8 } else { 0 };
        let mut big = vec![0u8; 1536 * 768 * 3];
        for y in 0..768 { for x in 0..1536 {
            let i = ((y / 2) * 768 + x / 2) * 3;
            let mut p = [rgb[i], rgb[i + 1], rgb[i + 2]];
            if level > 0 {
                if x >= 3 && y >= 3 && letter(x - 3, y - 3) { p = p.map(|v| text::mix(v, 0, level)); }
                if letter(x, y) { p = p.map(|v| text::mix(v, 255, level)); }
            }
            big[(y * 1536 + x) * 3..(y * 1536 + x) * 3 + 3].copy_from_slice(&p);
        } }
        out.write_all(&big).unwrap();
    }
}
