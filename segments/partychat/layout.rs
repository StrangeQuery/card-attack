// The party chat: the stream chat over the stadium party carrying the credits (without emoji), as data, and the UI kit's groups of items each frame shows, given its numbers. Shared
// by the host (which works the numbers out on CARDIAC: fx/partychat.rs) and the reference (ordinary Rust): nothing
// here is worked out, only placed where the design draws it. Include with #[path] beside ui/scenes.rs (as
// `scenes`).
#![allow(dead_code)]
use super::scenes::*;

/// The layer: 512x768 at 1024, 0 of the frame, over the stadium's frames 44 (the party) to 538 (the cut to black).
pub const CW: usize = 512;
pub const CH: usize = 768;
pub const CX: usize = 1024;
pub const FRAMES: usize = 495;
/// The panel's left edge (1156 on the frame: partychat.py's 380 wide), its colour, how much it covers (0.5, 32nds).
pub const PX: i16 = 1156 - CX as i16;
pub const PANEL: Rgb = rgb(14, 12, 20);
pub const PANEL_COVER: i16 = 16;
/// A message's handle (16 in from the panel's edge), its text (6 past the colon), a wrapped line's start, how far a
/// word may reach (18 short of the frame's right edge).
pub const HANDLE_X: i16 = PX + 16;
pub const TEXT_GAP: i16 = PX + 22;
pub const WRAP_X: i16 = PX + 16;
pub const RIGHT: i16 = (1536 - 18 - CX) as i16;
/// The newest message's bottom (22 up from the layer's), a line's step, the gap between messages, the head's bottom
/// (nothing of a message shows above it), the frames a new message takes to push the others up.
pub const BOTTOM: i16 = 768 - 22;
pub const LINE: i16 = 29;
pub const GAP: i16 = 12;
pub const TOP: i16 = 62;
pub const SLIDE: i16 = 4;
/// The handles' colours (partychat.py's), picked by the sum of the handle's character codes, mod 8.
pub const COLOURS: [[u8; 3]; 8] = [[255, 105, 97], [119, 221, 119], [132, 182, 244], [253, 253, 150], [255, 179, 71], [203, 153, 201], [174, 198, 207], [255, 209, 220]];
pub const INK: Rgb = rgb(240, 240, 245);
/// The text's shadow: drawn black at these offsets first (partychat.py's shadowed()).
pub const SHADOW: [(i16, i16); 3] = [(2, 2), (1, 2), (2, 1)];

/// A message as laid out (worked out once): its handle with its colon, its colour, its lines (each where it starts
/// across and what it says).
pub struct Laid { pub handle: &'static str, pub colour: usize, pub lines: Vec<(i16, &'static str)> }

/// A message's height: its lines and the gap after it.
pub fn height(lines: usize) -> i16 { LINE * lines as i16 + GAP }

/// The lines of a message, given each word's line and where the first starts: words joined by spaces.
pub fn lines(text: &str, line_of: &[usize], tx: i16) -> Vec<(i16, &'static str)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let n = line_of.last().map_or(1, |&l| l + 1);
    (0..n).map(|l| {
        let s = words.iter().zip(line_of).filter(|w| *w.1 == l).map(|w| *w.0).collect::<Vec<_>>().join(" ");
        // a handful of strings, made once a render
        (if l == 0 { tx } else { WRAP_X }, &*Box::leak(s.into_boxed_str()))
    }).collect()
}

fn text(x: i16, y: i16, face: usize, rgb: Rgb, s: &'static str) -> Item { Item::Text { x, y, face, across: Across::Left, middle: false, rgb, says: Says::Str(s) } }

/// Text with its shadow: the shadow's three black copies, then the text.
fn shadowed(items: &mut Vec<Item>, x: i16, y: i16, face: usize, rgb: Rgb, s: &'static str) {
    for (dx, dy) in SHADOW { items.push(text(x + dx, y + dy, face, BLACK, s)); }
    items.push(text(x, y, face, rgb, s));
}

/// The chat's frame: the panel over the party, its head, the messages shown (each with its top; the newest first),
/// nothing of them above the head. Each line's places worked out here (for the reference; the host has the place
/// machine work them out, chat_frame_with).
pub fn chat_frame(laid: &[Laid], tops: &[(usize, i16)]) -> Vec<Group> {
    chat_frame_with(laid, tops, &mut |x, y, j| {
        let y = y + LINE * j;
        [(x + SHADOW[0].0, y + SHADOW[0].1), (x + SHADOW[1].0, y + SHADOW[1].1), (x + SHADOW[2].0, y + SHADOW[2].1), (x, y)]
    })
}

/// Each line drawn, in order, as the place machine (pc_place) is dealt them: its left x, its message's top, which
/// line of the message it is.
pub fn lines_shown(laid: &[Laid], tops: &[(usize, i16)]) -> Vec<[i16; 3]> {
    let mut v = vec![];
    for &(m, y) in tops {
        v.push([HANDLE_X, y, 0]);
        for (j, &(x, _)) in laid[m].lines.iter().enumerate() { v.push([x, y, j as i16]); }
    }
    v
}

/// The chat's frame, each shown line's four places (its shadow's three, then its own) from `place`, called for the
/// lines in lines_shown's order.
pub fn chat_frame_with(laid: &[Laid], tops: &[(usize, i16)], place: &mut dyn FnMut(i16, i16, i16) -> [(i16, i16); 4]) -> Vec<Group> {
    let panel = Item::Panel { x: PX, y: 0, w: K(CW as i16 - PX), h: CH as i16, r: 0, top: true, bottom: true, rgb: PANEL };
    let mut groups = vec![Group { fade: K(PANEL_COVER), ..Group::new(vec![panel]) }];
    let mut head = vec![];
    shadowed(&mut head, PX + 16, 18, ARIAL22B, rgb(205, 205, 215), "STREAM CHAT");
    head.push(Item::Panel { x: PX, y: 51, w: K(CW as i16 - PX), h: 2, r: 0, top: true, bottom: true, rgb: rgb(90, 90, 100) });
    groups.push(Group::new(head));
    let mut items = vec![];
    let mut placed = |items: &mut Vec<Item>, at: [(i16, i16); 4], face: usize, rgb: Rgb, s: &'static str| {
        for &(x, y) in &at[..3] { items.push(text(x, y, face, BLACK, s)); }
        items.push(text(at[3].0, at[3].1, face, rgb, s));
    };
    for &(m, y) in tops {
        let l = &laid[m];
        let c = COLOURS[l.colour];
        placed(&mut items, place(HANDLE_X, y, 0), ARIAL22B, rgb(c[0] as i16, c[1] as i16, c[2] as i16), l.handle);
        for (j, &(x, s)) in l.lines.iter().enumerate() { placed(&mut items, place(x, y, j as i16), ARIAL22, INK, s); }
    }
    groups.push(Group { clip: Some([PX, TOP, CW as i16, CH as i16]), ..Group::new(items) });
    groups
}
