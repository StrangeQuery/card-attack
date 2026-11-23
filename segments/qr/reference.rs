//! The QR code in ordinary Rust: the reference the host's frame is checked against. One frame at 1536x768, black all
//! over (the black the quiet zone), the code's 25 modules 24 pixels each in the middle, each dark module white:
//! a white code on black.
//!
//!     qr_ref SEGMENTS_DIR        # RGB 1536x768 frames to stdout
use std::io::Write;

const W: usize = 1536;
const H: usize = 768;
const MODULE: usize = 24;

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let rows: Vec<Vec<bool>> = std::fs::read_to_string(format!("{dir}/qr/data/qr.txt")).unwrap().lines()
        .filter(|l| !l.starts_with('#')).map(|l| l.bytes().map(|b| b == b'1').collect()).collect();
    let side = rows.len() * MODULE;
    let (x0, y0) = ((W - side) / 2, (H - side) / 2);
    let mut pic = vec![0u8; W * H * 3];
    for y in y0..y0 + side { for x in x0..x0 + side {
        if rows[(y - y0) / MODULE][(x - x0) / MODULE] { pic[3 * (y * W + x)..][..3].fill(255); }
    } }
    std::io::stdout().lock().write_all(&pic).unwrap();
}
