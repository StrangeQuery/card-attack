// Opening cards reference, 192x96: the five cards straight from their definition, every pixel on its own, in whole
// numbers. Arg: the data folder (segments/openers/data).
//
// Each card shows a picture for its frames, then three frames of black. The first is a colour picture (photo.bin:
// the CARDIAC's photo and its words in white); the others are bitmaps of words (1 a letter), white. On frame t of a
// card of n frames the picture is at k = min(5, t + 1, n - t) fifths: fading in over the first five frames and out
// over the last five. A colour c is shown as floor(c k / 5); a letter, white (255),
// is 51 k; everything else is black.
use std::io::Write;
const W: usize = 192; const H: usize = 96;
const BEAT: usize = 3;
// each card: its picture and how many frames it shows (four in, held, four out)
const CARDS: [(&str, usize); 5] = [("photo", 48), ("strange", 38), ("couple", 38), ("ghz", 28), ("enjoy", 24)];
fn picture(dir: &str, name: &str) -> Vec<u8> {
    if name == "photo" { return std::fs::read(format!("{dir}/photo.bin")).unwrap() }
    std::fs::read_to_string(format!("{dir}/{name}.txt")).unwrap().lines()
        .flat_map(|l| l.bytes().flat_map(|b| [(b - b'0') * 255; 3]).collect::<Vec<u8>>()).collect()
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    for (name, frames) in CARDS {
        let pic = picture(&a[1], name);
        assert_eq!(pic.len(), W * H * 3);
        for t in 0..frames + BEAT {
            let k = if t < frames { 5.min(t + 1).min(frames - t) } else { 0 };
            let frame: Vec<u8> = pic.iter().map(|&c| (c as usize * k / 5) as u8).collect();
            out.write_all(&frame).unwrap();
        }
    }
}
