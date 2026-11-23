// The party chat's reference, in ordinary Rust (the prototype: partychat.py, without its emoji). Args: the demo
// folder. The layer over the stadium party, 512x768 at 1024, 0 of the frame (RGB and cover, a frame each, 495
// frames): the panel (half over the party), its head, the messages (data/messages.txt: each posting at its frame, the
// newest at the bottom sliding up into place over its first four frames, the older pushed up; each wrapped to the
// panel by its words' advances, its handle in a colour by its codes' sum mod 8), the text shadowed. Drawn by the UI
// kit (ui/draw.rs) from layout.rs.
use std::io::Write;
use std::path::Path;

#[allow(dead_code)]
#[path = "../ui/scenes.rs"]
mod scenes;
#[path = "../ui/draw.rs"]
mod draw;
#[path = "layout.rs"]
mod layout;
use layout::*;
use scenes::*;

/// The messages (data/messages.txt): the frame each posts at, its handle, its text.
fn messages(demo: &Path) -> Vec<(i16, String, String)> {
    std::fs::read_to_string(demo.join("partychat/data/messages.txt")).unwrap().lines().filter(|l| !l.starts_with('#'))
        .map(|l| { let p: Vec<&str> = l.splitn(3, '|').collect(); (p[0].parse().unwrap(), p[1].to_string(), p[2].to_string()) }).collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let demo = Path::new(&a[1]);
    let atlas = draw::Atlas::read(demo);
    let adv = |face: usize, s: &str| -> i64 { s.chars().map(|c| atlas.glyphs[&(face, c as u32)].adv).sum() };
    let ms = messages(demo);
    // each message laid out: its handle's colour, where its text starts, its words wrapped, its height
    let mut heights = vec![];
    let laid: Vec<Laid> = ms.iter().map(|(_, name, msg)| {
        let colour = (name.chars().map(|c| c as u32).sum::<u32>() % 8) as usize;
        let tx = TEXT_GAP as i64 + adv(ARIAL22B, &format!("{name}:"));
        let space = adv(ARIAL22, " ");
        let (mut x, mut started, mut line, mut line_of) = (tx, false, 0, vec![]);
        for w in msg.split_whitespace() {
            let ww = adv(ARIAL22, w);
            if started && x + ww > RIGHT as i64 { line += 1; x = WRAP_X as i64; }
            started = true;
            x += ww + space;
            line_of.push(line);
        }
        heights.push(height(line + 1));
        Laid { handle: Box::leak(format!("{name}:").into_boxed_str()), colour, lines: lines(msg, &line_of, tx as i16) }
    }).collect();
    let mut out = std::io::stdout().lock();
    for k in 0..FRAMES as i16 {
        // the messages posted, the newest at the bottom (sliding up over its first frames), each before it above it
        let shown = ms.iter().filter(|m| m.0 <= k).count();
        let newest = shown - 1;
        let since = k - ms[newest].0;
        let h = heights[newest];
        let mut y = BOTTOM - h + if since < SLIDE - 1 { h * (SLIDE - 1 - since) / SLIDE } else { 0 };
        let mut tops = vec![(newest, y)];
        for m in (0..newest).rev() {
            y -= heights[m];
            if y + heights[m] < TOP { break }
            tops.push((m, y));
        }
        out.write_all(&draw::layer(&atlas, CW, CH, 0, 0, &chat_frame(&laid, &tops))).unwrap();
    }
}
