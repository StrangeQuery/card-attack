//! The QR code: one frame between the stadium and the SQ, a big white code on black filling
//! the screen for the blooper reel's hidden page (segments/qr/data/qr.txt, made by make_data.py). Per module row the
//! row machine (qr_line) colours the row's pixels, white for a dark module; the host files each machine row 24 pixel
//! rows high in the middle of the black frame (replication, no math on it).
//! `card-attack qr refs/qr_ref` renders it and checks the frame against the reference.
use crate::*;
use std::path::Path;

pub const FRAMES: usize = 1;
const W: usize = 1536;
const H: usize = 768;
const FRAME: usize = W * H * 3;
pub const VIDEO: &str = "qr/video_hd.rgb";
/// Pixels a module (reference.rs's): the code 600 pixels a side, the frame's black its quiet zone.
const MODULE: usize = 24;

fn frame(demo: &Path, host: &mut Host) -> Vec<u8> {
    let line = rust(demo, "qr_line");
    let rows: Vec<Vec<i32>> = read(demo, "qr/data/qr.txt").lines().filter(|l| !l.starts_with('#'))
        .map(|l| l.bytes().map(|b| (b == b'1') as i32).collect()).collect();
    let side = rows.len() * MODULE;
    let (x0, y0) = ((W - side) / 2, (H - side) / 2);
    let mut pic = vec![0u8; FRAME];
    // black all over, then each module row's pixels as the machine coloured them, a module's rows high
    for (v, row) in rows.iter().enumerate() {
        let rgb: Vec<u8> = host.go1(&line, &[&[row.len() as i32, MODULE as i32], row], side * 3).iter().map(|&c| c as u8).collect();
        for y in y0 + v * MODULE..y0 + (v + 1) * MODULE { pic[3 * (y * W + x0)..][..3 * side].copy_from_slice(&rgb); }
    }
    pic
}

pub fn render(demo: &Path, refbin: &str) {
    let mut host = Host::new();
    let mut video = cardiac_host::Video::<W, H>::new(FRAMES);
    for _ in live::frames(FRAMES) { video.buf.copy_from_slice(&frame(demo, &mut host)); }
    host.done();
    finish("qr (1536x768)", &host, &video, &reference(refbin, &[demo]), &demo.join(VIDEO), "");
}
