// The Mac cold open as data: the System 6 desk, what each frame of it shows, and the script that runs it (the
// prototype macdoom.py's, then macdesk.py's launch, frame for frame). Shared by the host (fx/mac.rs, which works every
// frame out on CARDIAC) and the reference (which draws them in ordinary Rust). Nothing here is worked out: every place
// is a corner of a window (or of the screen) and a distance from it, as the Mac's own resources gave a window's parts
// in its own coordinates, and the machines add them up; the pointer's glides, the zoom rects and the tunnel are given
// by their ends and steps, the game's frames, the page's sphere and the flag by what boom.rs says, and all of it
// worked out on CARDIAC.
#![allow(dead_code)]

pub const W: usize = 512;
pub const H: usize = 342;

/// The Mac's faces in the UI kit's atlas (segments/ui/data/atlas.txt): Chicago 12, Geneva 9, Geneva 12.
pub const CHICAGO: usize = 17;
pub const GENEVA9: usize = 18;
pub const GENEVA12: usize = 19;

/// A window's frame (left, top, right, bottom, its edges' pixels included); the screen's is all 0, so a place on it
/// is its distance.
pub type Rect = [i16; 4];
pub const SCREEN: Rect = [0, 0, 0, 0];

/// A place across or down: a corner of the window and how far from it (Mid: the middle across, (l + r) / 2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum At { L(i16), T(i16), R(i16), B(i16), Mid(i16) }

impl At {
    /// Which of the window's five numbers (l, t, r, b, the middle) and the distance from it: the place machine's cards.
    pub fn cards(self) -> (i16, i16) {
        match self { At::L(d) => (0, d), At::T(d) => (1, d), At::R(d) => (2, d), At::B(d) => (3, d), At::Mid(d) => (4, d) }
    }
}
use At::*;

/// What a line of text is made of: text in a face, a gap (the menu bar's), a sprite set like a glyph with its advance.
#[derive(Clone, Debug)]
pub enum Piece { Str(usize, String), Gap(i16), Sprite(&'static str, i16) }

/// How a line sits on its place: starting there, centred on it, ending there (ui_text's 0, 1, 2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align { Start, Centre, End }

/// A line of text on its baseline. `black`: its ink (white otherwise). `back`: a box behind it, filled black or
/// white, its edges from the line's ends and baseline (left of the start, above the baseline (negative), right of the
/// end, below it). `glyphs`: whether its glyphs are drawn (a box laid first, the line again later, in the
/// prototype's order). `caret`: the insertion point after its last glyph.
#[derive(Clone, Debug)]
pub struct Text { pub pieces: Vec<Piece>, pub x: At, pub y: At, pub align: Align, pub black: bool, pub back: Option<([i16; 4], bool)>, pub glyphs: bool, pub caret: bool }

/// A picture worked out on CARDIAC: the game's frame (its view and status bar, as frame f shows it: gone haywire from
/// HAYWIRE), the Wikipedia page's shaded sphere (its dots only: laid over what is there), the Finnish flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pic { Game(usize), Sphere, Flag }

/// What is drawn, in order: a rectangle filled black (true) or white; a rectangle's black outline; a sprite at its
/// anchor; a sprite at its anchor cut off below a row (its pixels down to that row); a line of text; a picture worked
/// out, its corner at a place.
#[derive(Clone, Debug)]
pub enum Item { Fill(At, At, At, At, bool), Frame(At, At, At, At), Sprite(&'static str, At, At), Clip(&'static str, At, At, At), Text(Text), Picture(Pic, At, At) }

/// A window's items (or the desk's, the alert's, the menu bar's: on the screen).
pub struct Layer { pub rect: Rect, pub items: Vec<Item> }

/// A point on a window.
#[derive(Clone, Copy, Debug)]
pub struct Point { pub rect: Rect, pub x: At, pub y: At }

/// Where the pointer is: still, or step j of n of a glide (ease out, a slight arc) from one point to another, or of a
/// straight move (as far along as j of n).
#[derive(Clone, Copy, Debug)]
pub enum Mouse { Still(Point), Glide { from: Point, to: Point, step: i16, steps: i16 }, Line { from: Point, to: Point, step: i16, steps: i16 } }

/// One of the Finder's zoom rects (an XOR outline): step j of n from one rectangle to another.
#[derive(Clone, Copy, Debug)]
pub struct Zoom { pub from: (Rect, [At; 4]), pub to: (Rect, [At; 4]), pub step: i16, pub steps: i16 }

/// The one-bit tunnel: its picture j (it changes every third frame, 4 a second), and a title card over it, dissolved in
/// to j thirds (1-3).
#[derive(Clone, Copy, Debug)]
/// A frame of the tunnel: its picture, and the title card over it with the frame's place in the card's showing and how
/// long it shows (0: it stays); how much of the card shows is `thirds`'s (the host's on mac_thirds).
pub struct Tunnel { pub picture: i16, pub title: Option<(usize, i16, i16)> }

/// How many thirds of a title card show, k frames into its n (0: it stays): one more a frame in, one fewer a frame out.
pub fn thirds(k: i16, n: i16) -> i16 { if n > 0 { (k + 1).min(n - k).min(3) } else { (k + 1).min(3) } }

/// A frame: the layers in order over the desktop's grey, the zoom rects, the pointer (a sprite at the mouse); or the
/// tunnel.
pub struct Shot { pub layers: Vec<Layer>, pub zoom: Vec<Zoom>, pub cursor: Option<(String, Mouse)>, pub tunnel: Option<Tunnel> }

// ---- the texts ------------------------------------------------------------------------------------------------------

/// The Apple menu's logo in Chicago.
pub const APPLE: &str = "\u{14}";

/// A character's code in the Mac's fonts (MacRoman): ASCII as it is, the Apple 20, pi 185.
pub fn code(c: char) -> u32 {
    match c { '\u{3c0}' => 185, c if (c as u32) < 128 => c as u32, c => panic!("no MacRoman code for {}", c) }
}

/// The pointer's glides, in frames each, as the prototype times them (1.5 + the distance / 110, rounded, at least
/// 2: about 3 across a window, 5 across the screen): the script's timing.
pub const GLIDES: [i16; 6] = [2, 5, 2, 2, 5, 3];

// ---- the desk -------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum Sel { Plain, Selected, Open }

#[derive(Clone)]
pub enum Content { Icons(Vec<(&'static str, i16, i16, &'static str)>), Floppy(bool), Game(usize), Wiki, Finnish, Code(usize) }

#[derive(Clone)]
pub struct Win { pub title: &'static str, pub rect: Rect, pub info: Option<[&'static str; 3]>, pub scroll: bool, pub zoom: bool, pub content: Content,
    pub folder_at: Option<(i16, i16)>, pub pressed: bool }

fn win(title: &'static str, rect: Rect, content: Content) -> Win {
    Win { title, rect, info: None, scroll: true, zoom: true, content, folder_at: None, pressed: false }
}

/// The disk the demo was written on: a Lightspeed C project (its pi), the sources, the assembly, the build.
pub const FLOPPY: [(&str, i16, i16, &str); 8] = [("doc", 18, 12, "CardAttack.\u{3c0}"), ("doc", 104, 12, "CardAttack.\u{3c0}.rsrc"), ("doc", 190, 12, "main.c"),
    ("doc", 276, 12, "tunnel.c"), ("doc", 18, 74, "dither.c"), ("doc", 104, 74, "CARDIAC.h"), ("doc", 190, 74, "blit.a"), ("card", 276, 74, "Card Attack")];
pub const FLOPPY_WINDOW: Rect = [60, 70, 452, 262];
/// The Card Attack application's icon in the floppy's window, from its contents' corner.
pub const FLOPPY_APP: (i16, i16) = (276, 74);
/// The desk the cold open starts on, back to front: Learn to Speak Finnish, the Wikipedia page, the game in front.
pub fn game_windows() -> Vec<Win> {
    use super::boom::{FINNISH_RECT, GAME_RECT, WIKI_RECT};
    vec![
        Win { scroll: false, zoom: false, ..win("Learn to Speak Finnish", FINNISH_RECT, Content::Finnish) },
        win("Gouraud shading - Wikipedia", WIKI_RECT, Content::Wiki),
        Win { scroll: false, zoom: false, ..win("Boom", GAME_RECT, Content::Game(0)) },
    ]
}

/// The menu bars: the game's, THINK C's editor's, TeachText's (the windows behind it), the Finder's.
pub const GAME_MENUS: [&str; 5] = [APPLE, "File", "Edit", "Game", "Options"];
pub const EDITOR_MENUS: [&str; 6] = [APPLE, "File", "Edit", "Search", "Project", "Source"];
pub const TEACH_MENUS: [&str; 3] = [APPLE, "File", "Edit"];
pub const FINDER_MENUS: [&str; 5] = [APPLE, "File", "Edit", "View", "Special"];

/// A text of one string.
fn text(face: usize, s: &str, x: At, y: At, align: Align, black: bool) -> Text {
    Text { pieces: vec![Piece::Str(face, s.to_string())], x, y, align, black, back: None, glyphs: true, caret: false }
}

/// An icon at (x, y) and its name centred under it (a box behind the name on the desk, or when it's selected).
fn icon(items: &mut Vec<Item>, kind: &str, x: At, y: At, name: &str, sel: Sel, on_desk: bool) {
    let sprite: &'static str = match (kind, sel) {
        ("folder", Sel::Plain) => "icon_folder", ("doc", Sel::Plain) => "icon_doc", ("hd", Sel::Plain) => "icon_hd", ("floppy", Sel::Plain) => "icon_floppy",
        ("trash", Sel::Plain) => "icon_trash", ("card", Sel::Plain) => "icon_card", ("caution", Sel::Plain) => "icon_caution", ("clock", Sel::Plain) => "icon_clock",
        ("floppy", Sel::Selected) => "icon_floppy_sel", ("card", Sel::Selected) => "icon_card_sel", ("floppy", Sel::Open) => "icon_floppy_open",
        _ => panic!("no icon {}", kind),
    };
    items.push(Item::Sprite(sprite, x, y));
    if name.is_empty() { return }
    let selected = sel != Sel::Plain;
    let (cx, base) = match (x, y) { (L(a), T(b)) => (L(a + 16), T(b + 42)), _ => unreachable!() };
    let mut t = text(GENEVA9, name, cx, base, Align::Centre, !selected);
    if on_desk || selected { t.back = Some(([1, -9, 0, 2], selected)); }
    items.push(Item::Text(t));
}

/// A window's items: its frame and shadow, title bar (striped when it's in front, with its close and zoom boxes),
/// its header (items, in disk, available), scroll bars (their arrows and the grow box when in front), its contents.
pub fn window(w: &Win, active: bool) -> Vec<Item> {
    let mut v = vec![
        Item::Fill(L(0), T(0), R(0), B(0), false), Item::Frame(L(0), T(0), R(0), B(0)),
        Item::Fill(L(1), B(1), R(1), B(1), true), Item::Fill(R(1), T(1), R(1), B(1), true),
        Item::Fill(L(0), T(18), R(0), T(18), true),
    ];
    let title = text(CHICAGO, w.title, Mid(0), T(14), Align::Centre, true);
    if active {
        for y in (4..15).step_by(2) { v.push(Item::Fill(L(2), T(y), R(-2), T(y), true)); }
        v.push(Item::Text(Text { back: Some(([6, -11, 5, 1], false)), glyphs: false, ..title.clone() }));
        v.push(Item::Sprite("close_box", L(0), T(0)));
        if w.pressed { v.push(Item::Sprite("star", L(13), T(9))); }
        if w.zoom { v.push(Item::Sprite("zoom_box", R(0), T(0))); }
    }
    v.push(Item::Text(title));
    let mut top = 19;
    if let Some([a, b, c]) = w.info {
        v.push(Item::Text(text(GENEVA9, a, L(10), T(29), Align::Start, true)));
        v.push(Item::Text(text(GENEVA9, b, Mid(0), T(29), Align::Centre, true)));
        v.push(Item::Text(text(GENEVA9, c, R(-26), T(29), Align::End, true)));
        v.push(Item::Fill(L(1), T(32), R(-1), T(32), true));
        v.push(Item::Fill(L(1), T(34), R(-1), T(34), true));
        top = 35;
    }
    if w.scroll {
        v.push(Item::Fill(R(-15), T(18), R(-15), B(0), true));
        v.push(Item::Fill(L(0), B(-15), R(0), B(-15), true));
        if active {
            v.push(Item::Sprite("scroll_up", R(-14), T(19)));
            v.push(Item::Sprite("scroll_down", R(-14), B(-31)));
            v.push(Item::Sprite("scroll_left", L(1), B(-14)));
            v.push(Item::Sprite("scroll_right", R(-31), B(-14)));
            v.push(Item::Sprite("grow_box", R(0), B(0)));
        }
    }
    match &w.content {
        Content::Icons(items) => for &(kind, dx, dy, name) in items { icon(&mut v, kind, L(1 + dx), T(top + dy), name, Sel::Plain, false); },
        Content::Floppy(picked) => for &(kind, dx, dy, name) in FLOPPY.iter() {
            icon(&mut v, kind, L(1 + dx), T(top + dy), name, if *picked && kind == "card" { Sel::Selected } else { Sel::Plain }, false);
        },
        Content::Game(f) => v.push(Item::Picture(Pic::Game(*f), L(1), T(top))),
        Content::Wiki => {
            // the title, a rule, the tagline, the text and contents; the picture box with the sphere and its caption
            v.push(Item::Sprite("wiki_title", L(11), T(top + 8)));
            v.push(Item::Fill(L(11), T(top + 37), R(-41), T(top + 37), true));
            v.push(Item::Text(text(GENEVA9, "From Wikipedia, the free encyclopedia", L(11), T(top + 50), Align::Start, true)));
            for (i, line) in super::boom::WIKI_TEXT.iter().enumerate() {
                if !line.is_empty() { v.push(Item::Text(text(GENEVA9, line, L(11), T(top + 69 + 13 * i as i16), Align::Start, true))); }
            }
            for (l, t, r, b) in [(R(-212), T(top + 59), R(-43), T(top + 59)), (R(-212), T(top + 241), R(-43), T(top + 241)),
                                 (R(-212), T(top + 59), R(-212), T(top + 240)), (R(-42), T(top + 59), R(-42), T(top + 240))] {
                v.push(Item::Fill(l, t, r, b, true));
            }
            v.push(Item::Picture(Pic::Sphere, R(-197), T(top + 67)));
            v.push(Item::Text(text(GENEVA9, "A Gouraud-shaded sphere", R(-204), T(top + 233), Align::Start, true)));
        }
        Content::Code(scroll) => {
            // bsp_tree.c at three times the size, from line `scroll`: a line every 33 rows, its baseline 36 down from
            // the first, cut off at the foot of the 94 rows of 3 that fit (the prototype's canvas, tripled)
            for k in 0..CODE_SHOWN {
                let i = scroll + k;
                if i < CODE_LINES.len() && !CODE_LINES[i].is_empty() {
                    v.push(Item::Clip(CODE_LINES[i], L(1), T(top + 36 + 33 * k as i16), T(top + CODE_FOOT)));
                }
            }
        }
        Content::Finnish => {
            // LEARN TO SPEAK / FINNISH, the flag cropped to the window
            v.push(Item::Sprite("finnish_title", L(76), T(top + 10)));
            v.push(Item::Picture(Pic::Flag, L(31), T(super::boom::FLAG_TOP as i16)));
        }
    }
    v
}

/// THINK C's editor, nearly the whole screen: bsp_tree.c, each line a sprite (data/sprites.txt, `code_I`; the empty
/// line none), as many as show from the first shown and the last row they reach (from the content's top).
pub const EDIT_RECT: Rect = [2, 21, 509, 339];
pub const CODE_LINES: [&str; 19] = ["code_0", "code_1", "code_2", "", "code_4", "code_5", "code_6", "code_7", "code_8", "code_9",
    "code_10", "code_11", "code_12", "code_13", "code_14", "code_15", "code_16", "code_17", "code_18"];
pub const CODE_SHOWN: usize = 9;
pub const CODE_FOOT: i16 = 281;
pub fn editor(scroll: usize) -> Win { win("bsp_tree.c", EDIT_RECT, Content::Code(scroll)) }

/// The system error: its box (double-bordered), the bomb, the words, Restart and Resume (rounded, their labels centred).
pub const BOMB_RECT: Rect = [68, 212, 444, 300];
pub fn bomb_box() -> Vec<Item> {
    let [l, t, r, b] = BOMB_RECT;
    let mut v = vec![];
    for (k, black) in [(0, true), (1, false), (3, true), (5, false)] { v.push(Item::Fill(L(l + k), T(t + k), L(r - k), T(b - k), black)); }
    v.push(Item::Sprite("bomb", L(l + 18), T(t + 18)));
    v.push(Item::Text(text(CHICAGO, "Sorry, a system error occurred.", L(l + 66), T(t + 32), Align::Start, true)));
    v.push(Item::Text(text(CHICAGO, "ID = 02", L(l + 66), T(t + 50), Align::Start, true)));
    for (label, x0) in [("Restart", r - 170), ("Resume", r - 86)] {
        v.push(Item::Sprite("bomb_button", L(x0), T(b - 30)));
        v.push(Item::Text(text(CHICAGO, label, L((x0 + x0 + 70) / 2), T(b - 17), Align::Centre, true)));
    }
    v
}

pub const ALERT: Rect = [96, 96, 416, 196];
pub const OK_BUTTON: Rect = [330, 162, 389, 181];
/// The reminder's words.
pub const REMINDER: [&str; 2] = ["Demos tonight in Derek's room.", "Did you check out that new one?"];

/// The reminder: a System 6 alert, its double border, the Alarm Clock's icon, two lines of Chicago,
/// the OK button ringed.
pub fn alert(pressed: bool) -> Vec<Item> {
    let [l, t, r, b] = ALERT;
    let mut v = vec![];
    for (k, black) in [(0, true), (1, false), (3, true), (5, false)] { v.push(Item::Fill(L(l + k), T(t + k), L(r - k), T(b - k), black)); }
    icon(&mut v, "clock", L(l + 20), T(t + 18), "", Sel::Plain, false);
    for (i, line) in REMINDER.iter().enumerate() { v.push(Item::Text(text(CHICAGO, line, L(l + 72), T(t + 33 + 16 * i as i16), Align::Start, true))); }
    let [bl, bt, br, bb] = OK_BUTTON;
    v.push(Item::Sprite(if pressed { "ok_pressed" } else { "ok" }, L(bl), T(bt)));
    v.push(Item::Text(text(CHICAGO, "OK", L((bl + br) / 2), T(bb - 5), Align::Centre, !pressed)));
    v
}

/// The menu bar: white, its line, the titles from x = 10 (17 after the Apple, 14 after the others); the Alarm Clock
/// flashing in the Apple's place.
pub fn menubar(titles: &[&str], clock: bool) -> Vec<Item> {
    let mut pieces = vec![];
    for (i, &s) in titles.iter().enumerate() {
        if i == 0 && clock { pieces.push(Piece::Sprite("menu_clock", 11)); } else { pieces.push(Piece::Str(CHICAGO, s.to_string())); }
        pieces.push(Piece::Gap(if i == 0 { 17 } else { 14 }));
    }
    vec![Item::Fill(L(0), T(0), L(W as i16 - 1), T(18), false), Item::Fill(L(0), T(19), L(W as i16 - 1), T(19), true),
         Item::Text(Text { pieces, x: L(10), y: T(14), align: Align::Start, black: true, back: None, glyphs: true, caret: false })]
}

// ---- the script -----------------------------------------------------------------------------------------------------

struct Desk { icons: Vec<(&'static str, i16, i16, &'static str, Sel)>, windows: Vec<Win>, bomb: bool, alert: Option<bool>, menus: Vec<&'static str>, clock: bool,
    zoom: Vec<Zoom>, cursor: Option<&'static str>, mouse: Point, glide: Option<(Point, i16, i16)>, line: Option<(Point, i16, i16)>, shots: Vec<Shot>, glides: usize }

fn screen(x: i16, y: i16) -> Point { Point { rect: SCREEN, x: L(x), y: T(y) } }

impl Desk {
    fn snap(&mut self, n: usize) {
        for _ in 0..n {
            let mut layers = vec![];
            let mut desk = vec![];
            for &(kind, x, y, name, sel) in &self.icons { icon(&mut desk, kind, L(x), T(y), name, sel, true); }
            layers.push(Layer { rect: SCREEN, items: desk });
            for (i, w) in self.windows.iter().enumerate() { layers.push(Layer { rect: w.rect, items: window(w, i + 1 == self.windows.len()) }); }
            if self.bomb { layers.push(Layer { rect: SCREEN, items: bomb_box() }); }
            if let Some(pressed) = self.alert { layers.push(Layer { rect: SCREEN, items: alert(pressed) }); }
            layers.push(Layer { rect: SCREEN, items: menubar(&self.menus, self.clock) });
            let mouse = match (self.glide, self.line) {
                (Some((from, step, steps)), _) => Mouse::Glide { from, to: self.mouse, step, steps },
                (None, Some((from, step, steps))) => Mouse::Line { from, to: self.mouse, step, steps },
                (None, None) => Mouse::Still(self.mouse),
            };
            self.shots.push(Shot { layers, zoom: self.zoom.clone(), cursor: self.cursor.map(|c| (c.to_string(), mouse)), tunnel: None });
        }
    }

    /// The pointer to a point, in the script's next glide's frames.
    fn glide(&mut self, to: Point) {
        let steps = GLIDES[self.glides];
        self.glides += 1;
        let from = self.mouse;
        self.mouse = to;
        for step in 1..=steps {
            self.glide = Some((from, step, steps));
            self.snap(1);
        }
        self.glide = None;
    }

    /// The Finder's zoom rects from one rectangle to another: steps + 1 outlines, three drawn a frame.
    fn zoom(&mut self, from: (Rect, [At; 4]), to: (Rect, [At; 4]), steps: i16) {
        for k in (0..=steps).step_by(3) {
            self.zoom = (k..=(k + 2).min(steps)).map(|step| Zoom { from, to, step, steps }).collect();
            self.snap(1);
        }
        self.zoom = vec![];
    }

    /// The front window's close box, clicked in a hurry; the menu bar turns to the next application's.
    /// Step j of n of a straight move of the pointer from one point to another (the mouse there at the last).
    fn line(&mut self, from: Point, to: Point, step: i16, steps: i16) {
        self.mouse = if step == 0 { from } else { to };
        self.line = if step > 0 && step < steps { Some((from, step, steps)) } else { None };
    }

    fn close_front(&mut self, read: usize, menus: &[&'static str]) {
        let w = self.windows.last().unwrap().clone();
        self.glide(Point { rect: w.rect, x: L(13), y: T(9) });
        self.snap(1);
        self.windows.last_mut().unwrap().pressed = true;
        self.snap(2);
        self.windows.pop();
        self.menus = menus.to_vec();
        self.snap(read);
    }
}

/// Every frame of the cold open, in order.
pub fn script() -> Vec<Shot> {
    use super::boom::{clock, ALARM, BOMB, GAME_FRAMES, RESUME, SCROLLS};
    let mut s = Desk { icons: vec![("hd", 464, 28, "Hard Disk", Sel::Plain), ("trash", 464, 292, "Trash", Sel::Plain)], windows: game_windows(), bomb: false, alert: None,
        menus: GAME_MENUS.to_vec(), clock: false, zoom: vec![], cursor: None, mouse: screen(300, 200), glide: None, line: None, shots: vec![], glides: 0 };
    // the game, the pointer hidden, going haywire; the system error, its picture frozen behind the bomb box; after a
    // second the pointer to Resume; back in the editor, the I-beam over the code; to the scroll bar's down arrow, two
    // clicks, a line each; the Alarm Clock flashes over its last second
    let [_, _, br, bb] = BOMB_RECT;
    let (bomb_at, resume) = (screen(320, 150), screen(br - 46, bb - 20));
    let [_, _, er, eb] = EDIT_RECT;
    let (code_at, arrow_at) = (screen(440, 300), screen(er - 7, eb - 24));
    let mut scroll = 0;
    for f in 0..GAME_FRAMES {
        if f < BOMB { s.windows.last_mut().unwrap().content = Content::Game(f); }
        if f == BOMB {
            s.bomb = true;
            s.cursor = Some("arrow");
            s.mouse = bomb_at;
        }
        if (BOMB + 8..=BOMB + 12).contains(&f) { s.line(bomb_at, resume, (f - BOMB - 8) as i16, 4); }
        if f == RESUME {
            s.bomb = false;
            s.windows.pop();
            s.windows.push(editor(0));
            s.menus = EDITOR_MENUS.to_vec();
            s.mouse = code_at;
            s.cursor = Some("ibeam");
        }
        if (SCROLLS[0] - 6..=SCROLLS[0] - 2).contains(&f) {
            s.cursor = Some("arrow");
            s.line(code_at, arrow_at, (f + 6 - SCROLLS[0]) as i16, 4);
        }
        if SCROLLS.contains(&f) {
            scroll += 1;
            s.windows.last_mut().unwrap().content = Content::Code(scroll);
        }
        s.clock = f >= ALARM && clock(f);
        s.snap(1);
    }
    s.clock = false;
    s.cursor = Some("arrow");
    s.mouse = screen(300, 230);
    s.alert = Some(false);
    s.snap(16);
    let [bl, bt, br, bb] = OK_BUTTON;
    s.glide(screen((bl + br) / 2 + 3, (bt + bb) / 2 + 2));
    s.snap(1);
    s.alert = Some(true);
    s.snap(2);
    s.alert = None;
    s.snap(5);
    // and now everything, in a hurry: half a second on the Wikipedia page, eight frames on Learn to Speak Finnish
    s.close_front(6, &TEACH_MENUS);
    s.close_front(8, &TEACH_MENUS);
    s.close_front(4, &FINDER_MENUS);
    s.snap(4);
    // a floppy goes in: the drive grinds
    s.cursor = Some("watch0");
    s.snap(14);
    s.icons.insert(1, ("floppy", 464, 76, "Card Attack", Sel::Plain));
    s.cursor = Some("arrow");
    s.snap(6);
    s.glide(screen(480, 92));
    s.snap(3);
    s.icons[1].4 = Sel::Selected;
    s.snap(3);
    let floppy = Win { info: Some(["8 items", "391K in disk", "9K available"]), ..win("Card Attack", FLOPPY_WINDOW, Content::Floppy(false)) };
    s.zoom((SCREEN, [L(464), T(76), L(495), T(107)]), (FLOPPY_WINDOW, [L(0), T(0), R(0), B(0)]), 10);
    s.icons[1].4 = Sel::Open;
    s.windows.push(floppy);
    s.snap(12);
    // the application, double-clicked
    let (ax, ay) = (1 + FLOPPY_APP.0, 35 + FLOPPY_APP.1);
    s.glide(Point { rect: FLOPPY_WINDOW, x: L(ax + 16), y: T(ay + 14) });
    s.snap(3);
    s.windows[0].content = Content::Floppy(true);
    s.snap(3);
    s.cursor = Some("watch0");
    s.zoom((FLOPPY_WINDOW, [L(ax), T(ay), L(ax + 31), T(ay + 31)]), (SCREEN, [L(0), T(20), L(W as i16 - 1), T(H as i16 - 1)]), 12);
    // the application takes the screen: two seconds of the watch
    s.windows.clear();
    s.icons.clear();
    s.menus = vec![APPLE, "File", "Edit"];
    s.mouse = screen(256, 171);
    const WATCH: [&str; 8] = ["watch0", "watch1", "watch2", "watch3", "watch4", "watch5", "watch6", "watch7"];
    for k in 0..24 {
        s.cursor = Some(WATCH[k / 2 % 8]);
        s.snap(1);
    }
    // the tunnel, in one bit, 4 pictures a second: two seconds alone, then the titles, two seconds, two, three
    let mut timeline = vec![None; 24];
    for (which, n) in [(0usize, 24i16), (1, 24), (2, 36)] {
        for k in 0..n { timeline.push(Some((which, k, if which != 2 { n } else { 0 }))); }
    }
    for (k, title) in timeline.into_iter().enumerate() {
        s.shots.push(Shot { layers: vec![], zoom: vec![], cursor: None, tunnel: Some(Tunnel { picture: k as i16 / 3, title }) });
    }
    s.shots
}

// ---- the data ---------------------------------------------------------------------------------------------------------

/// A sprite (data/sprites.txt): its place from its anchor, its size, each pixel 0 nothing, 1 white, 2 black.
pub struct Sprite { pub ox: i16, pub oy: i16, pub w: usize, pub h: usize, pub px: Vec<u8> }

pub fn sprites(demo: &std::path::Path) -> std::collections::HashMap<String, Sprite> {
    let text = std::fs::read_to_string(demo.join("mac/data/sprites.txt")).unwrap();
    let mut lines = text.lines().filter(|l| !l.starts_with('#'));
    let mut all = std::collections::HashMap::new();
    while let Some(head) = lines.next() {
        let p: Vec<&str> = head.split(' ').collect();
        assert_eq!(p[0], "sprite");
        let n = |i: usize| p[i].parse::<i16>().unwrap();
        let (w, h) = (n(4) as usize, n(5) as usize);
        let px = (0..h).flat_map(|_| lines.next().unwrap().bytes().map(|b| match b { b'.' => 0, b'o' => 1, b'X' => 2, _ => panic!("sprites.txt") }).collect::<Vec<u8>>()).collect::<Vec<u8>>();
        assert_eq!(px.len(), w * h);
        all.insert(p[1].to_string(), Sprite { ox: n(2), oy: n(3), w, h, px });
    }
    all
}

/// The tunnel's blocks: 128 x 86 of them, 4 x 4 pixels each (the last row cut short by the screen).
pub const TW: usize = 128;
pub const TH: usize = 86;
pub const BLOCK: usize = 4;

/// The tunnel's tables (data/tables.txt): per slope step (angle, 200 (sec - 1)); per distance in half pixels (along,
/// twist, light, dark); the sine.
pub struct Tables { pub slope: Vec<[i16; 2]>, pub ring: Vec<[i16; 4]>, pub sine: Vec<i16> }

pub fn tables(demo: &std::path::Path) -> Tables {
    let mut t = Tables { slope: vec![], ring: vec![], sine: vec![] };
    for l in std::fs::read_to_string(demo.join("mac/data/tables.txt")).unwrap().lines().filter(|l| !l.starts_with('#')) {
        let p: Vec<&str> = l.split(' ').collect();
        let n: Vec<i16> = p[2..].iter().map(|v| v.parse().unwrap()).collect();
        match p[0] { "slope" => t.slope.push([n[0], n[1]]), "ring" => t.ring.push([n[0], n[1], n[2], n[3]]), "sine" => t.sine.push(n[0]), o => panic!("tables.txt: {}", o) }
    }
    t
}

/// The title cards at 128x86 (data/titles.txt): each block 0 none, 1 the black rim, 2 the white letters.
pub fn titles(demo: &std::path::Path) -> Vec<Vec<u8>> {
    let mut cards = vec![];
    for l in std::fs::read_to_string(demo.join("mac/data/titles.txt")).unwrap().lines().filter(|l| !l.starts_with('#')) {
        if l.starts_with("title") { cards.push(vec![]); continue }
        cards.last_mut().unwrap().extend(l.bytes().map(|b| match b { b'.' => 0u8, b'X' => 1, b'o' => 2, _ => panic!("titles.txt") }));
    }
    cards
}

/// HyperCard's dissolve (data/dissolve.bin): each screen pixel's turn, 0-254; a title's pixel shows at j thirds when its
/// turn is under 85 j.
pub fn dissolve(demo: &std::path::Path) -> Vec<u8> { std::fs::read(demo.join("mac/data/dissolve.bin")).unwrap() }
