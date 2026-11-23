//! The opening cards, before everything else, at 192x96: THIS IS A CARDIAC. / IT RUNS AT 0.2 HZ. beside the photo of
//! one; STRANGE QUERY / WANTED TO SEE / WHAT IT COULD / REALLY DO; SO WE GOT / A COUPLE OF THEM / AND OVERCLOCKED /
//! THEM...; ...TO 1 GHZ.; ENJOY! White Futura on black, each card fading in and out by fifths, with three frames of
//! black after it. The pictures are data (data/photo.bin, a colour picture; data/*.txt, bitmaps of words). Per frame
//! the fade machine works out the steps of the fade and the white's level from the frame's place in its card; per
//! screen row the tint machine shows the photo card's colours at those steps, or the ink machine colours a bitmap's
//! row at the level.
use crate::*;
use std::path::Path;

/// The cards: their pictures and how many frames each shows (four in, held, four out).
const CARDS: [(&str, usize); 5] = [("photo", 48), ("strange", 38), ("couple", 38), ("ghz", 28), ("enjoy", 24)];
/// Frames of black after each card.
const BEAT: usize = 3;
pub const FRAMES: usize = CARDS[0].1 + CARDS[1].1 + CARDS[2].1 + CARDS[3].1 + CARDS[4].1 + 5 * BEAT;

/// A card's rows: the photo card's colours (red, green, blue a pixel), or a bitmap's (1 a letter).
fn rows(demo: &Path, name: &str) -> Vec<Vec<i32>> {
    if name == "photo" {
        let b = std::fs::read(demo.join("openers/data/photo.bin")).unwrap();
        return b.chunks(W2 * 3).map(|r| r.iter().map(|&c| c as i32).collect()).collect();
    }
    read(demo, &format!("openers/data/{name}.txt")).lines().map(|l| l.bytes().map(|b| (b - b'0') as i32).collect()).collect()
}

pub fn render(demo: &Path, refbin: &str) {
    let (fade, ink, tint) = (rust(demo, "op_fade"), rust(demo, "op_ink"), rust(demo, "op_tint"));
    let mut host = Host::new();
    let mut video = Video2::new(FRAMES);
    let mut f = 0;
    for (name, frames) in CARDS {
        let picture = rows(demo, name);
        for t in 0..frames + BEAT {
            let fifths = host.go1(&fade, &[&[t as i32, frames as i32]], 2);
            let (steps, level) = (fifths[0], fifths[1]);
            for y in 0..H2 {
                let rgb = if name == "photo" { host.go1(&tint, &[&[steps], &picture[y]], W2 * 3) }
                          else { host.go1(&ink, &[&[level], &picture[y]], W2 * 3) };
                video.row(f, y, &rgb);
            }
            f += 1;
        }
    }
    host.done();
    finish("openers", &host, &video, &reference(refbin, &[&demo.join("openers/data")]), &demo.join("openers/video.rgb"), "");
}
