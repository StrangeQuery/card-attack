//! The party chat: the stream chat over the stadium party, carrying the film's credits (segments/partychat), a layer
//! for the cut's over line (512x768 at 1024, 0 of the frame), 495 frames. Once a layer, the wrap machine (pc_wrap)
//! lays out each message (its handle's colour, where its text starts, each word's line, its height); per frame the
//! stack machine (pc_stack) how many have posted (data/messages.txt: the frame each posts at) and where each shown
//! goes, the newest sliding up into place; the UI kit lays the layer on (the panel, the head, the shadowed text).
use crate::*;
use crate::fx::ui::{Atlas, Kit};
use std::path::Path;

pub use crate::fx::ui::scenes;
#[allow(dead_code)]
#[path = "../../../segments/partychat/layout.rs"]
pub mod layout;
use layout::*;
use scenes::*;

/// The messages (data/messages.txt): the frame each posts at, its handle, its text.
fn messages(demo: &Path) -> Vec<(i32, String, String)> {
    read(demo, "partychat/data/messages.txt").lines().filter(|l| !l.starts_with('#'))
        .map(|l| { let p: Vec<&str> = l.splitn(3, '|').collect(); (p[0].parse().unwrap(), p[1].to_string(), p[2].to_string()) }).collect()
}

/// A word's advances in a face, as a deck: how many, then each.
fn advances(atlas: &Atlas, face: usize, s: &str) -> Vec<i32> {
    std::iter::once(s.chars().count() as i32).chain(s.chars().map(|c| atlas.glyph(face, c as u32).adv as i32)).collect()
}

/// The layer, every frame checked against the reference.
pub fn render(demo: &Path, refbin: &str) {
    let (wrap, stack, place) = (rust(demo, "pc_wrap"), rust(demo, "pc_stack"), rust(demo, "pc_place"));
    let mut host = Host::new();
    let kit = Kit::with(demo, &mut host, &[]);
    let ms = messages(demo);
    // each message laid out, on the wrap machine
    let mut heights = vec![];
    let laid: Vec<Laid> = ms.iter().map(|(_, name, msg)| {
        let words: Vec<&str> = msg.split_whitespace().collect();
        let mut deck = vec![name.chars().count() as i32];
        for ch in name.chars() { deck.extend([ch as i32, kit.atlas.glyph(ARIAL22B, ch as u32).adv as i32]); }
        deck.push(kit.atlas.glyph(ARIAL22B, ':' as u32).adv as i32);
        deck.extend([kit.atlas.glyph(ARIAL22, ' ' as u32).adv as i32, words.len() as i32]);
        for w in &words { deck.extend(advances(&kit.atlas, ARIAL22, w)); }
        let out = host.go1(&wrap, &[&deck], 3 + words.len());
        let line_of: Vec<usize> = out[2..2 + words.len()].iter().map(|&l| l as usize).collect();
        heights.push(out[2 + words.len()]);
        Laid { handle: Box::leak(format!("{name}:").into_boxed_str()), colour: out[0] as usize, lines: lines(msg, &line_of, out[1] as i16) }
    }).collect();
    // the stack's deck after the frame: how many, then each one's frame and height, the newest first
    let mut tail = vec![ms.len() as i32];
    for (m, h) in ms.iter().zip(&heights).rev() { tail.extend([m.0, *h]); }
    let size = CW * CH * 4;
    let mut buf = vec![0u8; FRAMES * size];
    for k in live::frames(FRAMES) {
        let s = host.gon(&stack, &[&[k as i32], &tail]);
        let newest = s[0] as usize - 1;
        let tops: Vec<(usize, i16)> = s[1..].iter().enumerate().map(|(i, &y)| (newest - i, y as i16)).collect();
        // each line's places on the place machine, taken in the order the frame draws them
        let shown = lines_shown(&laid, &tops);
        let deck: Vec<i32> = std::iter::once(shown.len() as i32).chain(shown.iter().flat_map(|l| l.map(|v| v as i32))).collect();
        let at = host.go1(&place, &[&deck], 8 * shown.len());
        let mut next = at.chunks(8);
        let frame = chat_frame_with(&laid, &tops, &mut |_, _, _| { let p = next.next().unwrap(); [(p[0] as i16, p[1] as i16), (p[2] as i16, p[3] as i16), (p[4] as i16, p[5] as i16), (p[6] as i16, p[7] as i16)] });
        buf[k * size..(k + 1) * size].copy_from_slice(&kit.layer(&mut host, CW, CH, 0, 0, &frame));
    }
    host.done();
    let r = reference(refbin, &[demo]);
    // a layer's frame is W x H x 4 bytes: as frames of RGB, 1024 x 512
    finish("partychat", &host, &cardiac_host::Video::<1024, 512> { frames: FRAMES, buf }, &r, &demo.join("partychat/chat.rgba"),
        &format!(" ({CW}x{CH} RGB and cover, for the cut's over at {CX} 0)"));
}
