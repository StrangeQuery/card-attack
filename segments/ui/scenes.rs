// The UI kit's scenes: what each frame of a layer shows, as data. Shared by the host (which works every frame out on
// CARDIAC: fx/ui.rs) and the reference (which draws it in ordinary Rust): nothing here is worked out, a frame is only
// a list of groups of items, at places in the layer, with the values that move given as eases (from, to, step j of
// n) for the machines to work out. A scene is a layer for the cut's `over` line: W x H at X, Y of the 1536x768 frame.
// The layouts (the notifications, the Wi-Fi gag, the accel segment's menu and brag) are in the layer's own pixels.

/// A number of the scene: a constant, or eased from one to another, step j of n (smooth: smoothstep, else straight).
#[derive(Clone, Copy, Debug)]
pub enum Val { K(i16), Ease { smooth: bool, from: i16, to: i16, j: i16, n: i16 },
    /// Eased out (fast, then slowing: 1 - (1 - u) cubed), a pointer's glide.
    Cube { from: i16, to: i16, j: i16, n: i16 } }
pub use Val::K;

pub const fn ease(from: i16, to: i16, j: usize, n: usize) -> Val { Val::Ease { smooth: true, from, to, j: j as i16, n: n as i16 } }
pub const fn glide(from: i16, to: i16, j: usize, n: usize) -> Val { Val::Cube { from, to, j: j as i16, n: n as i16 } }
pub const fn line(from: i16, to: i16, j: usize, n: usize) -> Val { Val::Ease { smooth: false, from, to, j: j as i16, n: n as i16 } }

/// A colour: red, green, blue (each 0-255).
pub type Rgb = [Val; 3];
pub const fn rgb(r: i16, g: i16, b: i16) -> Rgb { [K(r), K(g), K(b)] }
pub const WHITE: Rgb = rgb(255, 255, 255);
pub const BLACK: Rgb = rgb(0, 0, 0);

/// How a line of text sits on its anchor: across (Left: starts there, Middle: centred, Right: ends there); and
/// whether the anchor is its top (the face's ascent) or its middle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Across { Left, Middle, Right }

/// What a text says: a constant, or a number worked out (0-999) then a constant, or the first `typed` characters of
/// a constant with the face's cursor (code 1) after them if `cursor` (a prompt being typed).
#[derive(Clone, Copy, Debug)]
pub enum Says { Str(&'static str), Num(Val, &'static str), Typed { text: &'static str, typed: usize, cursor: bool } }

#[derive(Clone, Copy, Debug)]
pub enum Item {
    /// A rounded box, solid: x, y its top left, w x h, its corners' radius (0-26; a circle is a box whose radius is
    /// half its size), which corners are round (the top two, the bottom two).
    Panel { x: i16, y: i16, w: Val, h: i16, r: i16, top: bool, bottom: bool, rgb: Rgb },
    /// A soft shadow (black): the box x, y, w x h blurred over `blur` pixels either side of each edge, at most
    /// `level` 32nds dark.
    Shadow { x: i16, y: i16, w: i16, h: i16, blur: i16, level: i16 },
    /// A line of text in face `face` (atlas.txt), anchored at x, y.
    Text { x: i16, y: i16, face: usize, across: Across, middle: bool, rgb: Rgb, says: Says },
    /// Sprite `code` (atlas face 9), its anchor at x, y, laid on at `fade` 32nds.
    Sprite { x: i16, y: i16, code: usize, rgb: Rgb, fade: Val },
    /// The busy spinner: twelve spokes round x, y, spoke `phase` the darkest, each further one round lighter (70 to
    /// 255 grey), laid on at `fade` 32nds.
    Spinner { x: i16, y: i16, phase: i16, fade: Val },
    /// A colour emoji (atlas face 16, EMOJI; CODE: its character), its top left at x, y, laid on at `fade` 32nds.
    Emoji { x: i16, y: i16, code: u32, fade: Val },
}

/// Items moved together by dx, dy, laid on the layer at `fade` 32nds as one, only inside `clip` (x0, y0, x1, y1 of
/// the layer, the far edges not included) if given.
#[derive(Clone, Debug)]
pub struct Group { pub dx: Val, pub dy: Val, pub fade: Val, pub clip: Option<[i16; 4]>, pub items: Vec<Item> }

impl Group {
    pub fn new(items: Vec<Item>) -> Group { Group { dx: K(0), dy: K(0), fade: K(32), clip: None, items } }
}

/// A scene: its layer's size and place on the 1536x768 frame, its frames. Its items' places are in pixels from
/// (-ox, -oy), the layer's top left: a card holds -999..999, so a layer whose items travel more than 999 pixels
/// counts from its middle.
pub struct Scene { pub name: &'static str, pub w: usize, pub h: usize, pub x: usize, pub y: usize, pub ox: i16, pub oy: i16, pub frames: usize, pub frame: fn(usize) -> Vec<Group> }

pub const SCENES: [Scene; 4] = [
    Scene { name: "notify", w: 669, h: 200, x: 867, y: 8, ox: 334, oy: 0, frames: 60, frame: notify },
    Scene { name: "miner", w: 966, h: 700, x: 330, y: 60, ox: 0, oy: 0, frames: 93, frame: miner },
    Scene { name: "videoopts", w: 1197, h: 628, x: 339, y: 140, ox: 0, oy: 0, frames: VIDEO_FRAMES, frame: videoopts },
    Scene { name: "accelbrag", w: 441, h: 187, x: 40, y: 506, ox: 0, oy: 0, frames: 36, frame: accelbrag },
];

pub fn scene(name: &str) -> &'static Scene { SCENES.iter().find(|s| s.name == name).unwrap_or_else(|| panic!("no scene {}", name)) }

/// The faces (atlas.txt).
pub const F22: usize = 0; pub const F22B: usize = 1; pub const F24: usize = 2; pub const F24B: usize = 3; pub const F25: usize = 4;
pub const F26: usize = 5; pub const F26B: usize = 6; pub const F28B: usize = 7; pub const F30B: usize = 8;
/// The terminal's and the chat's: Menlo 30 and its bold, Arial 22 and its bold (with the
/// colour emoji a message carries), the floating colour emoji.
pub const MENLO30: usize = 12; pub const MENLO30B: usize = 13; pub const ARIAL22: usize = 14; pub const ARIAL22B: usize = 15; pub const EMOJI: usize = 16;
/// The cursor's code in a face that has one (the terminal's Menlo).
pub const CURSOR_CODE: u32 = 1;
/// The accelerated drive's (segments/accel): Arial Black 52 and 40, Arial Bold 34.
pub const BLACK52: usize = 20; pub const BOLD34: usize = 21; pub const BLACK40: usize = 22;
/// The sprites (atlas.txt's face 9).
pub const CURSOR_WHITE: usize = 0; pub const CURSOR_BLACK: usize = 1; pub const CURSOR_DOWN_WHITE: usize = 2; pub const CURSOR_DOWN_BLACK: usize = 3;
pub const FACE: usize = 4; pub const QUIT: usize = 5; pub const SPOKE: usize = 6;
/// glide.py's pixel arrow: its white and its black.
pub const ARROW_WHITE: usize = 18; pub const ARROW_BLACK: usize = 19;

fn text(x: i16, y: i16, face: usize, rgb: Rgb, s: &'static str) -> Item { Item::Text { x, y, face, across: Across::Left, middle: false, rgb, says: Says::Str(s) } }
fn text_at(x: i16, y: i16, face: usize, across: Across, middle: bool, rgb: Rgb, says: Says) -> Item { Item::Text { x, y, face, across, middle, rgb, says } }
fn panel(x: i16, y: i16, w: i16, h: i16, r: i16, rgb: Rgb) -> Item { Item::Panel { x, y, w: K(w), h, r, top: true, bottom: true, rgb } }
/// A box with a one-pixel outline: the outline's box, then the fill inside it.
fn framed(x: i16, y: i16, w: i16, h: i16, r: i16, outline: Rgb, fill: Rgb) -> [Item; 2] { [panel(x, y, w, h, r, outline), panel(x + 1, y + 1, w - 2, h - 2, r - 1, fill)] }

// ---- the Discord notification: a banner slides in at the top right, sits 4 s, slides away (60 frames) ----

const SLIDE: i16 = 668;          // from off the layer's right edge to its place
const IN: usize = 6;
const HOLD: usize = 48;
const B: i16 = 1 - 334;          // the banner's picture's top left (notify.py's banner()), at rest

fn notify(t: usize) -> Vec<Group> {
    let dx = if t < IN { ease(SLIDE, 0, t + 1, IN) } else if t < IN + HOLD { K(0) } else { ease(0, SLIDE, t - IN - HOLD + 1, IN) };
    let mut items = vec![Item::Shadow { x: B + 20, y: 26, w: 621, h: 151, blur: 21, level: 11 }];
    items.extend(framed(B + 20, 20, 621, 151, 26, rgb(210, 210, 216), rgb(242, 242, 245)));
    items.push(panel(B + 44, 57, 76, 76, 17, rgb(88, 101, 242)));        // the app's icon (a stand-in)
    items.push(Item::Sprite { x: B + 44, y: 57, code: FACE, rgb: WHITE, fade: K(32) });
    items.push(text(B + 140, 44, F28B, rgb(20, 20, 24), "mom"));
    items.push(text_at(B + 616, 46, F22, Across::Right, false, rgb(130, 130, 138), Says::Str("now")));
    items.push(text(B + 140, 84, F25, rgb(40, 40, 46), "I just got the electric bill. We need to talk."));
    vec![Group { dx, ..Group::new(items) }]
}

// ---- the bitcoin miner: the dive hangs (a spinner), a process monitor comes up, the cursor force quits the miner ----

const LX: i16 = 330; const LY: i16 = 60;                    // the layer's place on the 1536x768 frame
const WX: i16 = 368 - LX; const WY: i16 = 96 - LY;          // the window, 800 x 560 (801 x 561 pixels)
const WW: i16 = 800; const WH: i16 = 560;
const QX: i16 = WX + WW - 210; const QY: i16 = WY + 16;     // the Force Quit button
const HY: i16 = WY + 92;                                    // the column heads
const ROW: i16 = 52;
const BY: i16 = WY + WH - 70;                               // the CPU load bar
const SX: i16 = WX + 130; const SW: i16 = WW - 260; const SH: i16 = 220;   // the sheet, at rest
const SY: i16 = WY + 76;
const SPIN: (i16, i16) = (768 - LX, 384 - LY);
const PROCS: [(&str, &str, &str); 6] = [("BitcoinMiner", "1,187.4", "64"), ("CARD ATTACK", "98.2", "28"), ("WindowServer", "6.1", "12"),
    ("kernel_task", "3.0", "210"), ("Finder", "0.4", "9"), ("Discord", "0.3", "41")];
/// Where the cursor goes: where it starts, the miner's row, the toolbar's Force Quit, the sheet's.
const START: (i16, i16) = (1250 - LX, 700 - LY);
const MINER: (i16, i16) = (WX + 260, WY + 160);
const TOOLBAR: (i16, i16) = (QX + 90, QY + 22);
const SHEET_QUIT: (i16, i16) = (SX + SW / 2 + 20 + 85, SY + 165);
const RED: Rgb = rgb(230, 40, 40);
const GREEN: Rgb = rgb(52, 199, 89);

/// The monitor's state in a frame.
#[derive(Clone, Copy, Default)]
struct Monitor {
    selected: bool,
    /// The sheet's slide (its dy, -220 hidden to 0), if it shows.
    sheet: Option<Val>,
    /// Which button is held down: 1 the toolbar's, 2 the sheet's.
    pressed: u8,
    /// The miner going: None (not yet), Some(None) gone, Some(Some((j, n))) going, step j of n.
    going: Option<Option<(usize, usize)>>,
}

fn monitor(m: Monitor) -> Vec<Group> {
    let mut w = vec![Item::Shadow { x: WX + 6, y: WY + 14, w: WW + 1, h: WH + 1, blur: 30, level: 14 }];
    w.extend(framed(WX, WY, WW + 1, WH + 1, 20, rgb(200, 200, 205), rgb(250, 250, 252)));
    w.push(Item::Panel { x: WX, y: WY, w: K(WW + 1), h: 77, r: 20, top: true, bottom: false, rgb: rgb(232, 232, 236) });   // the title bar
    for (k, c) in [rgb(255, 95, 87), rgb(254, 188, 46), rgb(40, 200, 64)].iter().enumerate() { w.push(panel(WX + 22 + 30 * k as i16, WY + 28, 19, 19, 9, *c)); }
    w.push(text_at(WX + 140, WY + 38, F30B, Across::Left, true, rgb(40, 40, 46), Says::Str("Processes")));
    w.extend(framed(QX, QY, 181, 45, 10, rgb(190, 190, 196), if m.pressed == 1 { rgb(205, 205, 212) } else { rgb(244, 244, 247) }));
    w.push(Item::Sprite { x: QX, y: QY, code: QUIT, rgb: rgb(70, 70, 76), fade: K(32) });
    w.push(text_at(QX + 48, QY + 22, F24, Across::Left, true, rgb(40, 40, 46), Says::Str("Force Quit")));
    let head = rgb(110, 110, 118);
    w.push(text(WX + 40, HY, F22B, head, "Process Name"));
    w.push(text_at(WX + WW - 220, HY, F22B, Across::Right, false, head, Says::Str("% CPU")));
    w.push(text_at(WX + WW - 50, HY, F22B, Across::Right, false, head, Says::Str("Threads")));
    w.push(panel(WX + 30, HY + 35, WW - 59, 2, 0, rgb(220, 220, 224)));
    let gone = matches!(m.going, Some(None));
    for (k, &(name, cpu, threads)) in PROCS.iter().skip(gone as usize).enumerate() {
        let y = HY + 48 + k as i16 * ROW;
        let miner = name == "BitcoinMiner";
        if miner && m.selected { w.push(panel(WX + 24, y - 6, WW - 47, ROW - 4, 0, rgb(40, 122, 246))); }
        let ink = if miner && m.selected { WHITE } else if miner { rgb(220, 30, 40) } else { rgb(40, 40, 46) };
        let face = if miner { F26B } else { F26 };
        w.push(text(WX + 40, y, face, ink, name));
        w.push(text_at(WX + WW - 220, y, face, Across::Right, false, ink, Says::Str(cpu)));
        w.push(text_at(WX + WW - 50, y, F26, Across::Right, false, ink, Says::Str(threads)));
    }
    // the CPU load: 100% in red, falling to 23% in green as the miner goes
    let (load, bar, colour): (Val, Val, Rgb) = match m.going {
        None => (K(100), K(591), RED),
        Some(None) => (K(23), K(137), GREEN),
        Some(Some((j, n))) => (ease(100, 23, j, n), ease(591, 137, j, n), [0, 1, 2].map(|c| match (RED[c], GREEN[c]) { (K(a), K(b)) => ease(a, b, j, n), _ => unreachable!() })),
    };
    w.push(text(WX + 40, BY + 4, F22B, head, "CPU load"));
    w.push(panel(WX + 170, BY, WW - 209, 31, 8, rgb(228, 228, 232)));
    w.push(Item::Panel { x: WX + 170, y: BY, w: bar, h: 31, r: 8, top: true, bottom: true, rgb: colour });
    w.push(text_at(WX + WW - 50, BY + 15, F22B, Across::Right, true, WHITE, Says::Num(load, "%")));
    let mut groups = vec![Group::new(w)];
    if let Some(dy) = m.sheet {
        // the force-quit sheet, sliding out from under the title bar
        let mut s = framed(SX, SY, SW + 1, SH + 1, 16, rgb(190, 190, 196), rgb(252, 252, 253)).to_vec();
        let mid = SX + SW / 2;
        s.push(text_at(mid, SY + 52, F28B, Across::Middle, true, rgb(30, 30, 36), Says::Str("Force quit \u{201c}BitcoinMiner\u{201d}?")));
        s.push(text_at(mid, SY + 96, F22, Across::Middle, true, rgb(110, 110, 118), Says::Str("You will lose any unsaved coins.")));
        let (cx, fx) = (mid - 160, mid + 20);
        s.extend(framed(cx, SY + 140, 141, 51, 12, rgb(200, 200, 205), rgb(236, 236, 240)));
        s.push(text_at(cx + 70, SY + 165, F24, Across::Middle, true, rgb(40, 40, 46), Says::Str("Cancel")));
        s.push(panel(fx, SY + 140, 171, 51, 12, if m.pressed == 2 { rgb(190, 30, 40) } else { rgb(230, 50, 60) }));
        s.push(text_at(fx + 85, SY + 165, F24B, Across::Middle, true, WHITE, Says::Str("Force Quit")));
        groups.push(Group { dy, clip: Some([WX, WY + 77, WX + WW + 1, WY + WH + 1]), ..Group::new(s) });
    }
    groups
}

fn cursor(x: Val, y: Val, down: bool) -> Group {
    let (white, black) = if down { (CURSOR_DOWN_WHITE, CURSOR_DOWN_BLACK) } else { (CURSOR_WHITE, CURSOR_BLACK) };
    Group { dx: x, dy: y, ..Group::new(vec![Item::Sprite { x: 0, y: 0, code: white, rgb: WHITE, fade: K(32) }, Item::Sprite { x: 0, y: 0, code: black, rgb: BLACK, fade: K(32) }]) }
}

fn spinner(phase: usize, fade: Val) -> Group { Group::new(vec![Item::Spinner { x: SPIN.0, y: SPIN.1, phase: phase as i16, fade }]) }

/// The gag's frames, from the dive's first juddering frame (the timing is wifi_gag.py's): 12 juddering with the
/// spinner coming up, 12 hung with it, 107 of the monitor, 12 of the spinner again.
fn miner(f: usize) -> Vec<Group> {
    if f < 12 { return vec![spinner(f, ease(0, 32, (f + 1).min(10), 10))] }
    if f < 24 { return vec![spinner(f, K(32))] }
    if f >= 81 { return vec![spinner(f - 81, K(32))] }
    let mut u = f - 24;
    let at = |p: (i16, i16)| (K(p.0), K(p.1));
    // the steps: (frames, the monitor, where the cursor is (None: not shown), held down)
    type Step = (usize, Monitor, Option<(Val, Val)>, bool);
    let glide = |a: (i16, i16), b: (i16, i16), k: usize, n: usize| Some((ease(a.0, b.0, k + 1, n), ease(a.1, b.1, k + 1, n)));
    let sel = Monitor { selected: true, ..Default::default() };
    let sheet_up = Monitor { sheet: Some(K(0)), ..sel };
    let steps: [&dyn Fn(usize) -> Step; 14] = [
        &|_| (5, Monitor::default(), None, false),                                   // the monitor comes up (faded in)
        &|_| (12, Monitor::default(), Some(at(START)), false),                       // read it
        // in a hurry: from the first move to the last click, 18 frames, 1.5 s
        &|k| (4, Monitor::default(), glide(START, MINER, k, 4), false),              // to the miner
        &|k| (2, sel, Some(at(MINER)), k < 1),                                       // click
        &|_| (1, sel, Some(at(MINER)), false),
        &|k| (3, sel, glide(MINER, TOOLBAR, k, 3), false),                           // to Force Quit
        &|_| (1, Monitor { pressed: 1, ..sel }, Some(at(TOOLBAR)), true),
        &|k| (3, Monitor { sheet: Some(ease(-SH, 0, k + 1, 3)), ..sel }, Some(at(TOOLBAR)), false),   // are you sure?
        &|_| (1, sheet_up, Some(at(TOOLBAR)), false),
        &|k| (2, sheet_up, glide(TOOLBAR, SHEET_QUIT, k, 2), false),
        &|_| (1, Monitor { pressed: 2, ..sheet_up }, Some(at(SHEET_QUIT)), true),
        &|k| (4, Monitor { sheet: Some(ease(0, -SH, k + 1, 4)), going: Some(Some((k + 1, 4))), ..Default::default() }, Some(at(SHEET_QUIT)), false),
        &|_| (14, Monitor { going: Some(None), ..Default::default() }, Some(at(SHEET_QUIT)), false),
        &|_| (4, Monitor { going: Some(None), ..Default::default() }, None, false),   // the window closes
    ];
    for (i, step) in steps.iter().enumerate() {
        let (n, ..) = step(0);
        if u >= n { u -= n; continue }
        let (_, m, cur, down) = step(u);
        let mut groups = monitor(m);
        let fade = match i { 0 => line(0, 32, u + 1, 5), 13 => line(32, 0, u + 1, 4), _ => K(32) };
        for g in &mut groups { g.fade = fade; }
        if let Some((x, y)) = cur { groups.push(cursor(x, y, down)); }
        return groups;
    }
    unreachable!()
}

// ---- the accelerated drive's VIDEO OPTIONS menu (accel.py's menu() and beat()): over the dimmed frame, the pointer
// clicks through the four rows, then APPLY (47 frames); and its brag, held to the end (segments/accel) ----

pub const VIDEO_FRAMES: usize = 47;
const VX: i16 = 339; const VY: i16 = 140;                   // the layer's place on the 1536x768 frame
const PX: i16 = 348 - VX; const PY: i16 = 150 - VY;         // the panel, 841 x 471 (PIL's rectangle, its far edges in)
const PW: i16 = 841; const PH: i16 = 471;
const VAL_X: i16 = 778 - VX;
const ROW_Y: [i16; 4] = [280 - VY, 350 - VY, 420 - VY, 490 - VY];
const APPLY: (i16, i16) = (958 - VX, 540 - VY);             // 191 x 51
const POINTER: (i16, i16) = (1300 - VX, 700 - VY);
const ROWS: [(&str, &str, &str); 4] = [("RENDERER", "< Software >", "< 3D Accelerator >"), ("RESOLUTION", "< 320 x 200 >", "< 640 x 480 >"),
    ("COLOR DEPTH", "< 8-bit >", "< 16-bit >"), ("TEXTURE FILTER", "< Nearest >", "< Bilinear >")];

/// A box with an outline `width` thick inside its edge: the outline as four strips, the fill inside.
fn outlined(x: i16, y: i16, w: i16, h: i16, width: i16, outline: Rgb, fill: Rgb) -> (Vec<Item>, Item) {
    let strips = vec![panel(x, y, w, width, 0, outline), panel(x, y + h - width, w, width, 0, outline),
        panel(x, y + width, width, h - 2 * width, 0, outline), panel(x + w - width, y + width, width, h - 2 * width, 0, outline)];
    (strips, panel(x + width, y + width, w - 2 * width, h - 2 * width, 0, fill))
}

/// The menu's state in a frame: the values shown (switched or not), the row lit, APPLY held, the pointer's place.
fn video(switched: usize, hover: Option<usize>, pressed: bool, at: (Val, Val)) -> Vec<Group> {
    // the panel: its fill a little see-through (215 of 255: 27 32nds), its outline solid
    let (strips, fill) = outlined(PX, PY, PW, PH, 4, rgb(200, 170, 60), rgb(10, 12, 30));
    let mut items = strips;
    items.push(text(PX + 40, PY + 30, BLACK52, rgb(255, 205, 60), "VIDEO OPTIONS"));
    for (i, &(label, before, after)) in ROWS.iter().enumerate() {
        let y = ROW_Y[i];
        items.push(text(PX + 50, y, BOLD34, rgb(170, 180, 210), label));
        let (edge, inside) = outlined(VAL_X - 14, y - 6, 365, 51, 1, rgb(110, 110, 150), if hover == Some(i) { rgb(60, 50, 10) } else { rgb(25, 28, 55) });
        items.extend(edge);
        items.push(inside);
        items.push(text(VAL_X, y, BOLD34, WHITE, if i < switched { after } else { before }));
    }
    let (edge, inside) = outlined(APPLY.0, APPLY.1, 191, 51, 3, rgb(255, 220, 90), if pressed { rgb(200, 170, 60) } else { rgb(90, 75, 20) });
    items.extend(edge);
    items.push(inside);
    items.push(text(APPLY.0 + 38, APPLY.1 + 6, BOLD34, if pressed { rgb(10, 10, 20) } else { rgb(255, 230, 120) }, "APPLY"));
    let pointer = Group { dx: at.0, dy: at.1, ..Group::new(vec![Item::Sprite { x: 0, y: 0, code: ARROW_WHITE, rgb: WHITE, fade: K(32) },
        Item::Sprite { x: 0, y: 0, code: ARROW_BLACK, rgb: BLACK, fade: K(32) }]) };
    vec![Group { fade: K(27), ..Group::new(vec![fill]) }, Group::new(items), pointer]
}

fn videoopts(t: usize) -> Vec<Group> {
    // the pointer's glides, each from where the last ended, eased out over k frames: (to, frames)
    let targets: [((i16, i16), usize); 5] = [((1108 - VX, ROW_Y[0] + 18), 5), ((1108 - VX, ROW_Y[1] + 18), 3), ((1108 - VX, ROW_Y[2] + 18), 3),
        ((1108 - VX, ROW_Y[3] + 18), 3), ((APPLY.0 + 95, APPLY.1 + 25), 4)];
    let mut frames: Vec<Vec<Group>> = vec![];
    let mut from = POINTER;
    let still = |p: (i16, i16)| (K(p.0), K(p.1));
    for _ in 0..8 { frames.push(video(0, None, false, still(from))); }
    for (i, &(to, k)) in targets.iter().enumerate() {
        let row = if i < 4 { Some(i) } else { None };
        for j in 1..=k { frames.push(video(i, row, false, (glide(from.0, to.0, j, k), glide(from.1, to.1, j, k)))); }
        from = to;
        if i < 4 {
            frames.push(video(i, row, false, still(to)));
            for _ in 0..if i == 0 { 5 } else { 3 } { frames.push(video(i + 1, row, false, still(to))); }
        } else {
            frames.push(video(4, None, true, still(to)));
            frames.push(video(4, None, true, still(to)));
            frames.push(video(4, None, false, still(to)));
        }
    }
    assert_eq!(frames.len(), VIDEO_FRAMES);
    frames.swap_remove(t)
}

/// The brag, held over the drive from after the landing: a black box, three lines in Arial Black.
fn accelbrag(_t: usize) -> Vec<Group> {
    let mut items = vec![panel(0, 0, 441, 187, 0, BLACK)];
    for (i, line) in ["640 x 480", "16-BIT  BILINEAR", "3D ACCELERATED"].iter().enumerate() { items.push(text(20, 14 + 56 * i as i16, BLACK40, WHITE, line)); }
    vec![Group::new(items)]
}
