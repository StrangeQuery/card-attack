//! The cut: every effect in running order (cut.txt), from the effects' own verified videos, joined on CARDIAC.
//! "mix N" dissolves a segment into the next over N frames (they overlap); "in N" / "out N" dissolve from and to
//! black at the very start and end; "black N" holds N black frames at the end; "hold N" holds a segment's last frame
//! N frames more; "gap N" puts N black frames after it. A dissolve machine per row dithers the incoming picture over
//! the outgoing one. "word FILE" after a "mix" covers that dissolve with a big line: the big word machine dithers it
//! in over 6 frames, holds it across the dissolve and out over 6. "cap at len TEXT" puts a caption up:
//! a caption machine per box row dithers it in at the top left, in the listing's font. "stutter N1 N2 ..." cuts
//! back and forth into the next segment instead: beats of N1, N2, ... frames, the first holding this segment's last
//! frame, the next showing the next segment's own frames as it runs on underneath, and so on; the two overlap by
//! the beats' total, as for a mix, and the next segment carries on from there.
//! The cut is 192x96. A segment drawn at 96x48 (most) has each pixel filed four times, after its caption; one drawn
//! at 192x96 says so after its frame count ("seg ... 192x96") and is filed as it is. The dissolves and big words are
//! dithered at 192x96, each row in two halves of 96 pixels. "reverse" on a segment line plays its frames backwards.
//! A segment drawn at 1536x768 (the climax) says "1536x768": the cut files a black frame in its place at 192x96 and
//! lists which of its frames goes there in cut/hd.txt ("frame video f" a line), so the film is shown at 1536x768 with
//! every 192x96 frame's pixels each filed 64 times (the live player); no mix, stutter, caption or
//! word may touch it.
//! A segment can switch size at a frame: two segment lines, the same effect's two videos, the second starting where
//! the first ends. "freeze AT N" holds the segment's frame AT for N frames more, and it carries on after; "repeat AT
//! F1 F2 ..." shows its frames F1, F2, ... after its frame AT (a stutter, a judder), then carries on from AT + 1.
//! "unpixelate N8 N4 N2" on a 1536x768 segment shows its first N8 frames averaged over 8 x 8 blocks (192x96's
//! pixels), the next N4 over 4 x 4, the next N2 over 2 x 2, then sharp: a block mean machine per row of blocks per
//! channel. "over FILE WxH X Y AT LEN" lays a full-resolution layer over the segment's own frames AT..AT+LEN (not its
//! mixes): FILE is frames of W x H pixels, red, green, blue and cover (0-32, in 32nds), one a frame (the last held),
//! a file on disk or what an effect files (the UI kit's layers, ui/NAME.rgba: read through the live runtime, so the
//! player takes it from memory),
//! placed at X, Y of the 1536x768 frame; the segment's frame is filed 64 times a pixel (8 x 8) beneath it, and a mix
//! machine per row run mixes each covered pixel toward the layer by its cover. A frame un-pixelated or under a layer
//! is made at 1536x768: the cut files it in video_hd.rgb beside cut.txt and lists it in hd.txt like the others; at
//! 192x96 it files the frame without the layer (black for a 1536x768 segment's).
use crate::fx::listing::Listing;
use crate::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub const HW: usize = 1536;
pub const HH: usize = 768;
const HD_FRAME: usize = HW * HH * 3;

pub struct Cap { at: usize, len: usize, text: String }

/// A layer over a segment: its video (frames of w x h, RGB and cover), its place, and the segment's frames it covers.
pub struct Over { pub video: String, w: usize, h: usize, x: usize, y: usize, at: usize, len: usize }
pub struct Seg { pub video: String, start: usize, pub len: usize, pub big: bool, pub hd: bool, reverse: bool, mix: usize, stutter: Vec<usize>, hold: usize, gap: usize, word: Option<Vec<Vec<i32>>>, caps: Vec<Cap>,
    /// After the segment's frame AT, its frames F1, F2, ... (a freeze: AT N times).
    repeats: Vec<(usize, Vec<usize>)>, unpixelate: [usize; 3], pub overs: Vec<Over> }

impl Seg {
    /// Frames shown, with its freezes and repeats.
    fn shown(&self) -> usize { self.len + self.repeats.iter().map(|r| r.1.len()).sum::<usize>() }
    /// The video's frame shown at t (of `shown`): its frames in order, and after frame AT of each freeze or repeat
    /// the frames it shows there.
    fn frame_at(&self, t: usize) -> usize {
        let (mut t, mut next) = (t, 0);
        let mut k = None;
        for (at, shows) in &self.repeats {
            if t <= at - next { k = Some(next + t); break }
            t -= at + 1 - next;
            next = at + 1;
            if t < shows.len() { k = Some(shows[t]); break }
            t -= shows.len();
        }
        let k = k.unwrap_or(next + t);
        if self.reverse { self.start + self.len - 1 - k } else { self.start + k }
    }
}

/// How one frame of the cut is made. The frames are made from the segments' videos by the composer (on CARDIAC, the
/// dissolves), one at a time, in any order: by `render` here, and by the live player.
#[derive(Clone)]
pub enum Step {
    /// Segment s's frame t at 192x96; dissolved in from black at a level, or out to black.
    Pic { s: usize, t: usize, fade_in: Option<i32>, fade_out: Option<i32> },
    /// Segment s's frame t at 192x96, as it is (a stutter's beat).
    Plain { s: usize, t: usize },
    /// Segment a's frame ta dissolving into segment b's frame tb, at a level (0-16).
    Mix { a: usize, ta: usize, b: usize, tb: usize, level: i32 },
    Black,
    /// Frame t of a 1536x768 segment (its video's frame f), a black frame in the 192x96 cut.
    Hd { s: usize, t: usize, f: usize },
}

/// What is done at 1536x768 to a frame: averaged over blocks (1: not), layers laid over it (layer, its frame).
#[derive(Clone, Default)]
pub struct Sharp { pub block: usize, pub overs: Vec<(usize, usize, usize)> }

/// The cut, planned from cut.txt: its segments, and how each frame is made.
pub struct Cut { pub segs: Vec<Seg>, pub frames: Vec<Step>, pub sharp: Vec<Sharp>, words: Vec<(usize, usize, usize)> }

impl Cut {
    pub fn read(demo: &Path) -> Cut { Cut::parse(demo, &read(demo, "cut/cut.txt")) }

    pub fn parse(demo: &Path, text: &str) -> Cut {
        let (mut segs, mut in_n, mut out_n, mut black_n) = (Vec::<Seg>::new(), 0, 0, 0);
        for l in text.trim().lines() {
            let p: Vec<&str> = l.split(' ').collect();
            let n = || p[1].parse::<usize>().unwrap();
            let num = |i: usize| p[i].parse::<usize>().unwrap_or_else(|_| panic!("cut.txt: {l}"));
            match p[0] {
                "seg" => segs.push(Seg { video: p[2].into(), start: p[3].parse().unwrap(), len: p[4].parse().unwrap(), big: p[5..].iter().any(|o| match *o { "192x96" => true, "reverse" | "1536x768" => false, o => panic!("cut.txt: {o}") }), hd: p[5..].contains(&"1536x768"), reverse: p[5..].contains(&"reverse"), mix: 0, stutter: vec![], hold: 0, gap: 0, word: None, caps: vec![],
                    repeats: vec![], unpixelate: [0; 3], overs: vec![] }),
                "hold" => segs.last_mut().unwrap().hold = n(),
                "gap" => segs.last_mut().unwrap().gap = n(),
                "mix" => segs.last_mut().unwrap().mix = n(),
                "stutter" => segs.last_mut().unwrap().stutter = p[1..].iter().map(|v| v.parse().unwrap()).collect(),
                "word" => segs.last_mut().unwrap().word = Some(tables::digits(&read(demo, p[1]))),
                "in" => in_n = n(), "out" => out_n = n(), "black" => black_n = n(),
                "cap" => segs.last_mut().unwrap().caps.push(Cap { at: n(), len: p[2].parse().unwrap(), text: p[3..].join(" ") }),
                "freeze" | "repeat" => {
                    let s = segs.last_mut().unwrap();
                    let at = num(1);
                    let shows: Vec<usize> = if p[0] == "freeze" { vec![at; num(2)] } else { (2..p.len()).map(num).collect() };
                    assert!(s.repeats.last().is_none_or(|f| f.0 < at) && at < s.len && shows.iter().all(|&f| f < s.len), "cut.txt: {l}: freezes and repeats in order, inside the segment");
                    s.repeats.push((at, shows));
                }
                "unpixelate" => { let s = segs.last_mut().unwrap(); assert!(s.hd && p.len() == 4, "cut.txt: {l}: unpixelate N8 N4 N2, on a 1536x768 segment"); s.unpixelate = [num(1), num(2), num(3)]; }
                "over" => {
                    let (w, h) = p[2].split_once('x').unwrap_or_else(|| panic!("cut.txt: {l}"));
                    let o = Over { video: p[1].into(), w: w.parse().unwrap(), h: h.parse().unwrap(), x: num(3), y: num(4), at: num(5), len: num(6) };
                    assert!(o.x + o.w <= HW && o.y + o.h <= HH && o.w > 0 && o.h > 0, "cut.txt: {l}: the layer must lie inside 1536x768");
                    segs.last_mut().unwrap().overs.push(o);
                }
                o => panic!("cut.txt: {o}"),
            }
        }
        let level = |k: usize, n: usize| (16 * (k + 1) / (n + 1)) as i32;
        let mut frames: Vec<Step> = vec![];
        let mut words: Vec<(usize, usize, usize)> = vec![];   // (first frame, frames, segment whose word)
        let last = segs.len() - 1;
        for (i, s) in segs.iter().enumerate() {
            let len = s.shown();
            if s.hd {   // a black frame in its place, and where its frame goes
                let into = i > 0 && (segs[i - 1].mix > 0 || !segs[i - 1].stutter.is_empty());
                assert!(!into && s.mix == 0 && s.stutter.is_empty() && s.word.is_none() && s.caps.is_empty() && s.gap == 0, "{}: a 1536x768 segment is cut to and from, nothing over it", s.video);
                assert!(!(i == 0 && in_n > 0) && !(i == last && out_n > 0), "{}: no fade on a 1536x768 segment", s.video);
                for t in 0..len + s.hold { frames.push(Step::Hd { s: i, t, f: s.frame_at(t.min(len - 1)) }); }
                continue;
            }
            assert!(s.unpixelate == [0; 3], "{}: unpixelate is for a 1536x768 segment", s.video);
            assert!(s.mix == 0 || !segs[i + 1].hd, "{}: no mix into a 1536x768 segment", s.video);
            let (head, tail) = (if i > 0 { segs[i - 1].mix + segs[i - 1].stutter.iter().sum::<usize>() } else { 0 }, s.mix);
            assert!(s.stutter.is_empty() || (tail == 0 && i < last), "{}: a stutter cuts to a next segment, with no mix", s.video);
            for t in head..len - tail {
                if s.big { assert!(s.caps.is_empty(), "{}: captions are drawn at 96x48 only", s.video) }
                let fade_in = (i == 0 && t < in_n).then(|| level(t, in_n));
                let fade_out = (i == last && t + out_n >= len).then(|| (16 * (t + out_n - len + 1) / out_n) as i32);
                frames.push(Step::Pic { s: i, t, fade_in, fade_out });
            }
            for _ in 0..s.hold { let l = frames.last().unwrap().clone(); frames.push(l); }
            for _ in 0..s.gap { frames.push(Step::Black); }
            if s.word.is_some() { words.push((frames.len() - 16, tail + 32, i)); }   // in 6, held over the dissolve, out 6
            for k in 0..tail { frames.push(Step::Mix { a: i, ta: len - tail + k, b: i + 1, tb: k, level: level(k, tail) }); }
            let mut k = 0;
            for (beat, &n) in s.stutter.iter().enumerate() {
                for _ in 0..n { frames.push(if beat % 2 == 0 { Step::Plain { s: i, t: len - 1 } } else { Step::Plain { s: i + 1, t: k } }); k += 1; }
            }
        }
        for _ in 0..black_n { frames.push(Step::Black); }
        // what is done at 1536x768: the un-pixelate steps, and the layers over the segments' own frames
        let sharp: Vec<Sharp> = frames.iter().map(|st| {
            let (s, t) = match *st { Step::Pic { s, t, .. } | Step::Hd { s, t, .. } => (s, t), _ => return Sharp { block: 1, overs: vec![] } };
            let seg = &segs[s];
            let u = seg.unpixelate;
            let block = if !seg.hd { 1 } else if t < u[0] { 8 } else if t < u[0] + u[1] { 4 } else if t < u[0] + u[1] + u[2] { 2 } else { 1 };
            let overs = seg.overs.iter().enumerate().filter(|(_, o)| t >= o.at && t < o.at + o.len).map(|(k, o)| (s, k, t - o.at)).collect();
            Sharp { block, overs }
        }).collect();
        let hd: Vec<usize> = (0..frames.len()).filter(|&n| matches!(frames[n], Step::Hd { .. }) || sharp[n].block > 1 || !sharp[n].overs.is_empty()).collect();
        for &(at, len, _) in &words { assert!(hd.iter().all(|&f| f < at || f >= at + len), "a big word over a 1536x768 frame"); }
        Cut { segs, frames, sharp, words }
    }

    /// Whether frame n is made at 1536x768 by the composer (un-pixelated, or under a layer).
    pub fn composed_hd(&self, n: usize) -> bool { self.sharp[n].block > 1 || !self.sharp[n].overs.is_empty() }

    /// Where the 1536x768 frames go: (frame of the cut, video, its frame). A frame the composer makes at 1536x768 is
    /// the next of `composed` (video_hd.rgb beside the cut file).
    pub fn hd(&self, composed: &str) -> Vec<(usize, String, usize)> {
        let mut k = 0;
        self.frames.iter().enumerate().filter_map(|(n, st)| {
            if self.composed_hd(n) { k += 1; return Some((n, composed.to_string(), k - 1)) }
            match *st { Step::Hd { s, f, .. } => Some((n, self.segs[s].video.clone(), f)), _ => None }
        }).collect()
    }

    /// The videos a frame needs: its segments', and its layers'.
    pub fn uses(&self, n: usize) -> Vec<&str> {
        let mut v: Vec<&str> = match self.frames[n] { Step::Pic { s, .. } | Step::Plain { s, .. } | Step::Hd { s, .. } => vec![&self.segs[s].video], Step::Mix { a, b, .. } => vec![&self.segs[a].video, &self.segs[b].video], Step::Black => vec![] };
        v.extend(self.sharp[n].overs.iter().map(|&(s, k, _)| self.segs[s].overs[k].video.as_str()));
        v
    }

    /// The videos a frame needs and which of each one's frames: its segments', and its layers'.
    pub fn uses_at(&self, n: usize) -> Vec<(&str, usize)> {
        let at = |s: usize, t: usize| (self.segs[s].video.as_str(), self.segs[s].frame_at(t));
        let mut v = match self.frames[n] {
            Step::Pic { s, t, .. } | Step::Plain { s, t } => vec![at(s, t)],
            Step::Hd { s, f, .. } => vec![(self.segs[s].video.as_str(), f)],
            Step::Mix { a, ta, b, tb, .. } => vec![at(a, ta), at(b, tb)],
            Step::Black => vec![],
        };
        v.extend(self.sharp[n].overs.iter().map(|&(s, k, f)| (self.segs[s].overs[k].video.as_str(), f)));
        v
    }

    /// The segments a frame shows.
    pub fn shows(&self, n: usize) -> Vec<usize> {
        match self.frames[n] { Step::Pic { s, .. } | Step::Plain { s, .. } | Step::Hd { s, .. } => vec![s], Step::Mix { a, b, .. } => vec![a, b], Step::Black => vec![] }
    }

    /// The big words over frame n, in the order they are laid on: (segment whose word, fade).
    fn words_at(&self, n: usize) -> Vec<(usize, i32)> {
        self.words.iter().filter(|&&(at, len, _)| n >= at && n < at + len).map(|&(at, len, s)| {
            let u = n - at;
            (s, if u < 6 { 16 * (u + 1) / 6 } else if u < len - 6 { 16 } else { 16 * (len - u) / 6 } as i32)
        }).collect()
}
}

/// The cut's machines: the dissolve, the big word, the caption's; the block mean and the layer's mix.
pub struct Composer { mix: Machine, big: Machine, cwid: Machine, cap: Machine, li: Listing, mean: Machine, over: Machine }

impl Composer {
    pub fn new(demo: &Path) -> Self {
        Composer { mix: one(demo, "cut/original/mix.asm"), big: one(demo, "overlays/original/big.asm"), cwid: one(demo, "overlays/original/cwid.asm"), cap: one(demo, "overlays/original/cap.asm"), li: Listing::new(demo),
            mean: rust(demo, "blockmean"), over: rust(demo, "umix") }
    }

    /// Frame n of the cut at 192x96 (black for a 1536x768 one), from the segments' videos (`video` gives each).
    pub fn frame(&self, host: &mut Host, cut: &Cut, n: usize, video: &dyn Fn(&str) -> Arc<Vec<u8>>) -> Vec<u8> {
        let frame_of = |s: &Seg, t: usize| -> Vec<u8> { let (f, n) = (s.frame_at(t), if s.big { FRAME2 } else { FRAME }); video(&s.video)[f * n..(f + 1) * n].to_vec() };
        let at2 = |s: &Seg, t: usize| if s.big { frame_of(s, t) } else { doubled(&frame_of(s, t)) };
        let black = vec![0u8; FRAME2];
        let mut img = match cut.frames[n] {
            Step::Pic { s, t, fade_in, fade_out } => {
                let s = &cut.segs[s];
                let mut img = frame_of(s, t);
                if !s.big { self.caption(host, &mut img, s, t); }
                let mut img = if s.big { img } else { doubled(&img) };
                if let Some(l) = fade_in { img = dissolve2(host, &self.mix, &black, &img, l); }
                if let Some(l) = fade_out { img = dissolve2(host, &self.mix, &img, &black, l); }
                img
            }
            Step::Plain { s, t } => at2(&cut.segs[s], t),
            Step::Mix { a, ta, b, tb, level } => dissolve2(host, &self.mix, &at2(&cut.segs[a], ta), &at2(&cut.segs[b], tb), level),
            Step::Black | Step::Hd { .. } => black,
        };
        for (s, fade) in cut.words_at(n) { big_word2(host, &self.big, &mut img, cut.segs[s].word.as_ref().unwrap(), fade); }
        img
    }

    /// Frame n at 1536x768, for a frame `composed_hd` says is made so: a 1536x768 segment's frame averaged over its
    /// blocks, or the 192x96 frame filed 64 times a pixel; then its layers laid over it.
    pub fn sharp(&self, host: &mut Host, cut: &Cut, n: usize, video: &dyn Fn(&str) -> Arc<Vec<u8>>) -> Vec<u8> {
        let sh = &cut.sharp[n];
        let mut img = match cut.frames[n] {
            Step::Hd { s, f, .. } => {
                let v = video(&cut.segs[s].video);
                let img = &v[f * HD_FRAME..(f + 1) * HD_FRAME];
                if sh.block > 1 { block_mean(host, &self.mean, img, sh.block) } else { img.to_vec() }
            }
            _ => octupled(&self.frame(host, cut, n, video)),
        };
        for &(s, k, u) in &sh.overs {
            let o = &cut.segs[s].overs[k];
            let layer = video(&o.video);
            let size = o.w * o.h * 4;
            let frames = layer.len() / size;
            assert!(frames > 0 && layer.len() % size == 0, "{}: frames of {}x{} RGB and cover", o.video, o.w, o.h);
            let lf = &layer[u.min(frames - 1) * size..][..size];
            lay_over(host, &self.over, &mut img, lf, o);
        }
        img
    }

    fn caption(&self, host: &mut Host, img: &mut [u8], s: &Seg, t: usize) {
        let Some(c) = s.caps.iter().find(|c| t >= c.at && t < c.at + c.len) else { return };
        let li = &self.li;
        let u = t - c.at;
        let fade = 16.min(u).min(c.len - 1 - u) as i32;
        let padded = format!("{:<24}", format!(" {}", c.text));
        let cards = li.cards(&padded);
        let text: Vec<Vec<i32>> = (0..5).map(|g| { let r = host.go1(&li.ltex, &[&[g, 24], &cards], 24); host.go1(&li.row_machine(&r), &[&[1]], W) }).collect();
        let x1 = host.go1(&self.cwid, &[&[c.text.len() as i32]], 1)[0];
        let blank = vec![0; W];
        for y in 0..7 {
            let codes = if (1..6).contains(&y) { &text[y - 1] } else { &blank };
            let row = &mut img[y * W * 3..(y + 1) * W * 3];
            let mut deck = vec![fade, y as i32, x1];
            for x in 0..W { deck.extend([codes[x], row[3 * x] as i32, row[3 * x + 1] as i32, row[3 * x + 2] as i32]); }
            for (i, v) in host.go1(&self.cap, &[&deck], W * 3).into_iter().enumerate() { row[i] = v as u8; }
        }
    }
}

/// The cut file `cut` (under segments/) and the folder its frames are filed in.
pub fn cut_dir(cut: &str) -> String { Path::new(cut).parent().unwrap().to_string_lossy().into_owned() }

pub fn render(demo: &Path, refbin: &str, file: Option<&str>) {
    let file = file.unwrap_or("cut/cut.txt");
    let dir = cut_dir(file);
    let cut = Cut::parse(demo, &read(demo, file));
    let composer = Composer::new(demo);
    let mut videos: HashMap<String, Arc<Vec<u8>>> = HashMap::new();
    for v in cut.segs.iter().filter(|s| !s.hd).map(|s| s.video.clone()) {
        if !videos.contains_key(&v) { videos.insert(v.clone(), Arc::new(std::fs::read(demo.join(&v)).unwrap_or_else(|e| panic!("{v}: {e}")))); }
    }
    // the layers: what an effect filed (the UI kit's), or a file
    for v in cut.segs.iter().flat_map(|s| s.overs.iter().map(|o| o.video.clone())) {
        if !videos.contains_key(&v) { videos.insert(v.clone(), live::fetch(&demo.join(&v))); }
    }
    for s in cut.segs.iter().filter(|s| s.hd) {
        let n = std::fs::metadata(demo.join(&s.video)).unwrap_or_else(|e| panic!("{}: {e}", s.video)).len() as usize;
        assert!(n >= (s.start + s.len) * HD_FRAME, "{}: {} frames of 1536x768 needed", s.video, s.start + s.len);
        if cut.sharp.iter().zip(&cut.frames).any(|(sh, st)| matches!(st, Step::Hd { s: k, .. } if cut.segs[*k].video == s.video) && (sh.block > 1 || !sh.overs.is_empty())) {
            videos.entry(s.video.clone()).or_insert_with(|| Arc::new(std::fs::read(demo.join(&s.video)).unwrap()));   // read only to be un-pixelated or laid over
        }
    }
    let mut host = Host::new();
    let get = |v: &str| videos.get(v).unwrap_or_else(|| panic!("{v} not read")).clone();
    let frames: Vec<Vec<u8>> = (0..cut.frames.len()).map(|n| composer.frame(&mut host, &cut, n, &get)).collect();
    let sharp: Vec<Vec<u8>> = (0..cut.frames.len()).filter(|&n| cut.composed_hd(n)).map(|n| composer.sharp(&mut host, &cut, n, &get)).collect();
    let video = Video2 { frames: frames.len(), buf: frames.concat() };
    let list: String = cut.hd(&format!("{dir}/video_hd.rgb")).iter().map(|(at, v, f)| format!("{at} {v} {f}\n")).collect();
    std::fs::write(demo.join(&dir).join("hd.txt"), list).unwrap();
    host.done();
    let r = reference(refbin, &[&demo.join("listing/data/font.json"), &demo.join(file), demo]);
    let secs = video.frames / 12;
    let lo = (video.frames * FRAME2).min(r.len());
    if !sharp.is_empty() {
        let hd = cardiac_host::Video::<HW, HH> { frames: sharp.len(), buf: sharp.concat() };
        finish("cut (1536x768)", &host, &hd, &r[lo..], &demo.join(&dir).join("video_hd.rgb"), " un-pixelated or under a layer");
    }
    finish("cut", &host, &video, &r[..lo], &demo.join(&dir).join("video.rgb"), &format!(" ({}:{:02}) over {} segments", secs / 60, (video.frames as f64 / 12.0).round() as usize % 60, cut.segs.len()));
}

/// A 192x96 frame at 1536x768: each pixel filed 64 times.
fn octupled(img: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HD_FRAME);
    for y in 0..HH { for x in 0..HW { let i = ((y / 8) * W2 + x / 8) * 3; out.extend_from_slice(&img[i..i + 3]); } }
    out
}

/// The block mean machine per row of blocks per channel: the 1536x768 frame averaged over b x b blocks, each pixel
/// of a block filed with its block's mean.
fn block_mean(host: &mut Host, mean: &Machine, img: &[u8], b: usize) -> Vec<u8> {
    let mut out = vec![0u8; HD_FRAME];
    let across = HW / b;
    for by in 0..HH / b { for c in 0..3 {
        let mut deck = vec![across as i32, (b * b) as i32];
        for bx in 0..across { for y in by * b..(by + 1) * b { for x in bx * b..(bx + 1) * b { deck.push(img[(y * HW + x) * 3 + c] as i32); } } }
        let means = host.go1(mean, &[&deck], across);
        for (bx, &m) in means.iter().enumerate() { for y in by * b..(by + 1) * b { for x in bx * b..(bx + 1) * b { out[(y * HW + x) * 3 + c] = m as u8; } } }
    } }
    out
}

/// The layer's mix machine, a run of up to 256 pixels at a time: each channel of each pixel the layer covers (cover
/// 1-32) taken from the frame's toward the layer's by cover 32nds. Pixels it doesn't cover are left. Pixels alike
/// (the same layer pixel over the same frame pixel: a panel over a 192x96 frame's 8 x 8 blocks) are mixed once and
/// the result copied to each.
fn lay_over(host: &mut Host, mix: &Machine, img: &mut [u8], layer: &[u8], o: &Over) {
    let mut seen: std::collections::HashMap<[u8; 7], usize> = std::collections::HashMap::new();
    let (mut unique, mut slot): (Vec<[u8; 7]>, Vec<(usize, usize)>) = (vec![], vec![]);
    for ly in 0..o.h { for lx in 0..o.w {
        let l = &layer[(ly * o.w + lx) * 4..][..4];
        if l[3] == 0 { continue }
        assert!(l[3] <= 32, "{}: cover is 0-32", o.video);
        let at = ((o.y + ly) * HW + o.x + lx) * 3;
        let key = [img[at], img[at + 1], img[at + 2], l[0], l[1], l[2], l[3]];
        let n = *seen.entry(key).or_insert_with(|| { unique.push(key); unique.len() - 1 });
        slot.push((at, n));
    } }
    let mut mixed = Vec::with_capacity(3 * unique.len());
    for part in unique.chunks(256) {
        let mut deck = vec![(3 * part.len()) as i32];
        for k in part { for c in 0..3 { deck.extend([k[c] as i32, k[3 + c] as i32, k[6] as i32]); } }
        mixed.extend(host.go1(mix, &[&deck], 3 * part.len()));
    }
    for (at, n) in slot { for c in 0..3 { img[at + c] = mixed[3 * n + c] as u8; } }
}

/// A 96x48 frame at 192x96: each pixel filed four times.
fn doubled(img: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(FRAME2);
    for y in 0..H2 { for x in 0..W2 { let i = ((y / 2) * W + x / 2) * 3; out.extend_from_slice(&img[i..i + 3]); } }
    out
}

/// The dissolve machine per half row at 192x96: a dissolves into b at level (0-16).
fn dissolve2(host: &mut Host, mix: &Machine, a: &[u8], b: &[u8], level: i32) -> Vec<u8> {
    let mut img = vec![0u8; FRAME2];
    for y in 0..H2 { for half in [0, W] {
        let at = (y * W2 + half) * 3;
        let mut deck = vec![level, y as i32];
        for i in (at..at + W * 3).step_by(3) { deck.extend([a[i], a[i + 1], a[i + 2], b[i], b[i + 1], b[i + 2]].map(|v| v as i32)); }
        for (i, v) in host.go1(mix, &[&deck], W * 3).into_iter().enumerate() { img[at + i] = v as u8; }
    } }
    img
}

/// The big word machine per half row at 192x96: the 96x48 bitmap, each bit over four pixels, laid over at fade.
fn big_word2(host: &mut Host, big: &Machine, img: &mut [u8], bits: &[Vec<i32>], fade: i32) {
    for y in 0..H2 { for half in [0, W] {
        let at = (y * W2 + half) * 3;
        let row = &mut img[at..at + W * 3];
        let mut deck = vec![fade, y as i32];
        for x in 0..W { deck.extend([bits[y / 2][(half + x) / 2], row[3 * x] as i32, row[3 * x + 1] as i32, row[3 * x + 2] as i32]); }
        for (i, v) in host.go1(big, &[&deck], W * 3).into_iter().enumerate() { row[i] = v as u8; }
    } }
}
