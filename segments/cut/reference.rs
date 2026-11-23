// Cut reference: the same segments, stutters and captions, in ordinary Rust, at 192x96 (96x48 segments doubled);
// then the frames made at 1536x768 (un-pixelated, or under a layer), in order. Args: font.json, the cut file, the demo folder.
use std::io::Write;
const W: usize = 96; const H: usize = 48;    // the segments' frames; the cut is twice that, 192x96
const W2: usize = 2 * W; const H2: usize = 2 * H;
const WHITE: [u8; 3] = [240, 240, 240];
#[allow(unused_attributes)]
#[path = "../../demokit/src/lib.rs"]
mod demokit;   // the tables, shared with the machines and the host
const BAYER: [i32; 16] = demokit::tables::widen(demokit::tables::BAYER4);
fn ints(s: &str) -> Vec<i64> { s.split(|c: char| !c.is_ascii_digit() && c != '-').filter(|t| !t.is_empty()).map(|t| t.parse().unwrap()).collect() }
struct Seg { video: String, start: usize, len: usize, big: bool, hd: bool, reverse: bool, stutter: Vec<usize>, hold: usize, gap: usize, caps: Vec<(usize, usize, String)>,
    freeze: Vec<(usize, Vec<usize>)>, unpix: [usize; 3], overs: Vec<(String, usize, usize, usize, usize, usize, usize)> }
impl Seg {
    // frames shown, with the freezes and repeats; and which of the segment's frames shows at t (after frame a, a
    // freeze shows a n times more, a repeat the frames it lists)
    fn shown(&self) -> usize { self.len + self.freeze.iter().map(|f| f.1.len()).sum::<usize>() }
    fn src(&self, t: usize) -> usize {
        let mut seq: Vec<usize> = vec![];
        for k in 0..self.len { seq.push(k); for (a, more) in &self.freeze { if *a == k { seq.extend(more); } } }
        seq[t]
    }
}
const HW: usize = 1536; const HH: usize = 768;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let js = std::fs::read_to_string(&a[1]).unwrap();
    let mut font = std::collections::HashMap::new();
    let body = &js[js.find('{').unwrap() + 1..];
    let mut rest = &body[body.find('{').unwrap() + 1..];
    while let Some(q) = rest.find('"') {
        let after = &rest[q + 1..]; let e = after.find('"').unwrap(); let ch = after[..e].chars().next().unwrap();
        let lb = after.find('[').unwrap(); let rb = after.find(']').unwrap();
        font.insert(ch, ints(&after[lb + 1..rb])); rest = &after[rb + 1..];
    }
    let mut segs: Vec<Seg> = Vec::new();
    for l in std::fs::read_to_string(&a[2]).unwrap().trim().lines() {
        let p: Vec<&str> = l.split(' ').collect();
        match p[0] {
            "seg" => segs.push(Seg { video: p[2].into(), start: p[3].parse().unwrap(), len: p[4].parse().unwrap(), big: p[5..].contains(&"192x96"), hd: p[5..].contains(&"1536x768"), reverse: p[5..].contains(&"reverse"), stutter: vec![], hold: 0, gap: 0, caps: vec![], freeze: vec![], unpix: [0; 3], overs: vec![] }),
            "freeze" => { let a: usize = p[1].parse().unwrap(); segs.last_mut().unwrap().freeze.push((a, vec![a; p[2].parse().unwrap()])) }
            "repeat" => segs.last_mut().unwrap().freeze.push((p[1].parse().unwrap(), p[2..].iter().map(|v| v.parse().unwrap()).collect())),
            "unpixelate" => segs.last_mut().unwrap().unpix = [p[1].parse().unwrap(), p[2].parse().unwrap(), p[3].parse().unwrap()],
            "over" => { let (w, h) = p[2].split_once('x').unwrap(); let n: Vec<usize> = p[3..7].iter().map(|v| v.parse().unwrap()).collect();
                segs.last_mut().unwrap().overs.push((p[1].into(), w.parse().unwrap(), h.parse().unwrap(), n[0], n[1], n[2], n[3])) }
            "hold" => segs.last_mut().unwrap().hold = p[1].parse().unwrap(),
            "gap" => segs.last_mut().unwrap().gap = p[1].parse().unwrap(),
            "stutter" => segs.last_mut().unwrap().stutter = p[1..].iter().map(|v| v.parse().unwrap()).collect(),
            _ => segs.last_mut().unwrap().caps.push((p[1].parse().unwrap(), p[2].parse().unwrap(), p[3..].join(" "))),
        }
    }
    let fb = W * H * 3;
    let fb2 = W2 * H2 * 3;
    let double = |img: &[u8]| -> Vec<u8> { (0..H2).flat_map(|y| (0..W2).flat_map(move |x| { let i = ((y / 2) * W + x / 2) * 3; [img[i], img[i + 1], img[i + 2]] })).collect() };
    let mut videos: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for s in segs.iter().filter(|s| !s.hd) { if !videos.contains_key(&s.video) { videos.insert(s.video.clone(), std::fs::read(format!("{}/{}", a[3], s.video)).unwrap()); } }
    let at = |s: &Seg, t: usize| { let t = s.src(t); if s.reverse { s.start + s.len - 1 - t } else { s.start + t } };
    let raw = |s: &Seg, t: usize| { let v = &videos[&s.video]; let (f, n) = (at(s, t), if s.big { fb2 } else { fb }); v[f * n..(f + 1) * n].to_vec() };
    let frame = |s: &Seg, t: usize| if s.big { raw(s, t) } else { double(&raw(s, t)) };
    let lit = |line: &Vec<char>, g: usize, x: usize| x % 4 < 3 && { let c = font[&line[x / 4]][g]; [c / 100, c / 10 % 10, c % 10][x % 4] == 1 };
    let black = vec![0u8; fb2];
    let mut frames: Vec<Vec<u8>> = Vec::new();
    let mut whose: Vec<Option<(usize, usize)>> = Vec::new();   // the segment and its frame each frame of the cut shows as itself
    for (i, s) in segs.iter().enumerate() {
        if s.hd { for t in 0..s.shown() + s.hold { frames.push(black.clone()); whose.push(Some((i, t))); } continue; }   // 1536x768: shown in its place, black here
        let head = if i > 0 { segs[i - 1].stutter.iter().sum::<usize>() } else { 0 };
        let mut last = Vec::new();
        let len = s.shown();
        for t in head..len {
            let mut img = raw(s, t);
            if let Some((at, len, text)) = s.caps.iter().find(|c| t >= c.0 && t < c.0 + c.1) {
                let u = (t - at) as i32; let fade = 16.min(u).min(*len as i32 - 1 - u);
                let line: Vec<char> = format!("{:<24}", format!(" {}", text)).chars().collect();
                let x1 = 4 * text.chars().count() + 4;
                for y in 0..7 { for x in 0..=x1.min(W - 1) {
                    if BAYER[(y % 4) * 4 + x % 4] >= fade { continue; }
                    let on = y >= 1 && y < 6 && lit(&line, y - 1, x);
                    let k = (y * W + x) * 3; img[k..k + 3].copy_from_slice(if on { &WHITE } else { &[0, 0, 0] });
                } }
            }
            let img = if s.big { assert!(s.caps.is_empty()); img } else { double(&img) };
            frames.push(img.clone()); last = img; whose.push(Some((i, t)));
        }
        for _ in 0..s.hold { frames.push(last.clone()); whose.push(*whose.last().unwrap()); }
        for _ in 0..s.gap { frames.push(black.clone()); whose.push(None); }
        // a stutter: beats taking turns, this segment's last frame, then the next one's as it runs underneath
        let beats: Vec<usize> = s.stutter.iter().enumerate().flat_map(|(b, &n)| std::iter::repeat_n(b, n)).collect();
        for (k, b) in beats.into_iter().enumerate() { frames.push(if b % 2 == 0 { frame(s, len - 1) } else { frame(&segs[i + 1], k) }); whose.push(None); }
    }
    let mut out = std::io::stdout().lock();
    for f in &frames { out.write_all(f).unwrap(); }
    // at 1536x768: a 1536x768 segment's frame averaged over b x b blocks (rounded half up), or the 192x96 frame eight
    // times each way; then each layer mixed in by its cover (32nds), toward zero
    let hdf = HW * HH * 3;
    for (n, w) in whose.iter().enumerate() {
        let Some((i, t)) = *w else { continue };
        let s = &segs[i];
        let u = s.unpix;
        let b = if !s.hd { 1 } else if t < u[0] { 8 } else if t < u[0] + u[1] { 4 } else if t < u[0] + u[1] + u[2] { 2 } else { 1 };
        let overs: Vec<_> = s.overs.iter().filter(|o| t >= o.5 && t < o.5 + o.6).collect();
        if b == 1 && overs.is_empty() { continue; }
        let mut img: Vec<u8> = if s.hd {
            use std::io::{Read, Seek, SeekFrom};
            let mut fh = std::fs::File::open(format!("{}/{}", a[3], s.video)).unwrap();
            fh.seek(SeekFrom::Start((at(s, t.min(s.shown() - 1)) * hdf) as u64)).unwrap();
            let mut v = vec![0u8; hdf]; fh.read_exact(&mut v).unwrap();
            let mut o = v.clone();
            for by in 0..HH / b { for bx in 0..HW / b { for c in 0..3 {
                let mut sum = 0;
                for y in by * b..(by + 1) * b { for x in bx * b..(bx + 1) * b { sum += v[(y * HW + x) * 3 + c] as usize; } }
                let m = ((2 * sum + b * b) / (2 * b * b)) as u8;
                for y in by * b..(by + 1) * b { for x in bx * b..(bx + 1) * b { o[(y * HW + x) * 3 + c] = m; } }
            } } }
            o
        } else {
            (0..HH).flat_map(|y| (0..HW).flat_map(move |x| (0..3).map(move |c| (y / 8 * W2 + x / 8) * 3 + c))).map(|k| frames[n][k]).collect()
        };
        for (file, w, h, x0, y0, at0, _) in overs {
            let layer = std::fs::read(format!("{}/{}", a[3], file)).unwrap();
            let k = (t - at0).min(layer.len() / (w * h * 4) - 1);
            for y in 0..*h { for x in 0..*w {
                let l = &layer[((k * h + y) * w + x) * 4..][..4];
                let i = ((y0 + y) * HW + x0 + x) * 3;
                for c in 0..3 { let (p, q) = (img[i + c] as i32, l[c] as i32); img[i + c] = (p + (q - p) * l[3] as i32 / 32) as u8; }
            } }
        }
        out.write_all(&img).unwrap();
    }
}
