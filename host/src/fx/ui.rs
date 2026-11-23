//! The UI kit: full-resolution layers for the cut's `over` lines (the drive's dialog and its brag) and the party
//! chat, painted on CARDIAC. A scene (segments/ui/scenes.rs) says what each frame shows: groups of items (boxes,
//! lines of text, sprites) at places in the layer, with the pointer's glide given as an ease. Each frame, on
//! CARDIAC: the cube machine works out every glide (eased out); the text machine places every glyph (its advance
//! from the pen), the panel machine every row of every box. Then the items are laid on in order: the cover machine
//! works out how each pixel of an item covers what is under it, and deep's mix machine (umix) the colour; the fade
//! machine takes an item's or a group's cover down.
//!
//! The host deals and files: it fetches each glyph's and sprite's coverages from the atlas (input data, rasterised
//! by segments/ui/data/make_atlas.py) by the glyph's code and row; it files a box's whole run of a row with the colour
//! (32 32nds: the box's colour, as it is); and it keeps to the layer and a group's clip, as a blit does. Every pixel
//! partly covered goes through the cover and mix machines.
use crate::*;
use std::collections::HashMap;
use std::path::Path;

#[allow(dead_code)]
#[path = "../../../segments/ui/scenes.rs"]
pub mod scenes;
use scenes::*;

/// A glyph or sprite of the atlas: its advance, its bitmap's place from the pen (or anchor), its size, where its
/// coverages start.
pub struct Glyph { pub adv: i16, pub ox: i16, pub oy: i16, pub w: usize, pub h: usize, pub at: usize }
pub struct Atlas { pub mid: Vec<i16>, pub glyphs: HashMap<(usize, u32), Glyph>, pub cover: Vec<u8>,
    /// A colour glyph's colours (red, green, blue a pixel of its bitmap): where they start in the atlas's bytes.
    pub colours: HashMap<(usize, u32), usize> }

impl Atlas {
    pub fn read(demo: &Path) -> Atlas {
        let cover = std::fs::read(demo.join("ui/data/atlas.bin")).expect("ui/data/atlas.bin: run make_atlas.py");
        let (mut mid, mut glyphs, mut colours) = (vec![], HashMap::new(), HashMap::new());
        for l in read(demo, "ui/data/atlas.txt").lines().filter(|l| !l.starts_with('#')) {
            let p: Vec<&str> = l.split(' ').collect();
            let n = |i: usize| p[i].parse::<i64>().unwrap();
            match p[0] {
                "face" => { assert_eq!(n(1) as usize, mid.len()); mid.push(n(4) as i16) }
                "g" => { glyphs.insert((n(1) as usize, n(2) as u32), Glyph { adv: n(3) as i16, ox: n(4) as i16, oy: n(5) as i16, w: n(6) as usize, h: n(7) as usize, at: n(8) as usize }); }
                "rgb" => { colours.insert((n(1) as usize, n(2) as u32), n(3) as usize); }
                o => panic!("atlas.txt: {o}"),
            }
        }
        Atlas { mid, glyphs, cover, colours }
    }
    pub fn glyph(&self, face: usize, code: u32) -> &Glyph { self.glyphs.get(&(face, code)).unwrap_or_else(|| panic!("atlas: no glyph {code} in face {face}")) }
}

/// An item with its numbers worked out.
enum Done { Panel { x: i16, y: i16, w: i16, h: i16, r: i16, top: bool, bottom: bool, rgb: [u8; 3] }, Shadow { x: i16, y: i16, w: i16, h: i16, blur: i16, level: i16 },
    Text { x: i16, y: i16, face: usize, across: Across, middle: bool, rgb: [u8; 3], chars: Vec<u32>, num: Option<(i16, &'static str)> },
    Sprite { x: i16, y: i16, face: usize, code: usize, rgb: [u8; 3], fade: i16 }, Spinner { x: i16, y: i16, phase: i16, fade: i16 } }
struct DoneGroup { dx: i16, dy: i16, fade: i16, clip: Option<[i16; 4]>, items: Vec<Done> }

/// The groups with every number worked out by `val` (each Val in the same order every time).
fn work_out(groups: &[Group], val: &mut dyn FnMut(Val) -> i16) -> Vec<DoneGroup> {
    fn colour(c: Rgb, val: &mut dyn FnMut(Val) -> i16) -> [u8; 3] { c.map(|v| val(v) as u8) }
    groups.iter().map(|g| DoneGroup { dx: val(g.dx), dy: val(g.dy), fade: val(g.fade), clip: g.clip, items: g.items.iter().map(|it| match *it {
        Item::Panel { x, y, w, h, r, top, bottom, rgb } => Done::Panel { x, y, w: val(w), h, r, top, bottom, rgb: colour(rgb, val) },
        Item::Shadow { x, y, w, h, blur, level } => Done::Shadow { x, y, w, h, blur, level },
        Item::Text { x, y, face, across, middle, rgb, says } => {
            let rgb = colour(rgb, val);
            match says {
                Says::Str(s) => Done::Text { x, y, face, across, middle, rgb, chars: s.chars().map(|c| c as u32).collect(), num: None },
                Says::Num(v, suffix) => Done::Text { x, y, face, across, middle, rgb, chars: vec![], num: Some((val(v), suffix)) },
                Says::Typed { text, typed, cursor } => Done::Text { x, y, face, across, middle, rgb, chars: text.chars().take(typed).map(|c| c as u32).chain(cursor.then_some(CURSOR_CODE)).collect(), num: None },
            }
        }
        Item::Sprite { x, y, code, rgb, fade } => Done::Sprite { x, y, face: 9, code, rgb: colour(rgb, val), fade: val(fade) },
        Item::Emoji { x, y, code, fade } => Done::Sprite { x, y, face: EMOJI, code: code as usize, rgb: [0; 3], fade: val(fade) },
        Item::Spinner { x, y, phase, fade } => Done::Spinner { x, y, phase, fade: val(fade) },
    }).collect() }).collect()
}

/// A pixel of an item to lay on: where in the layer (its index), its cover, its colour.
type Px = (usize, u8, [u8; 3]);

/// The kit's machines, the atlas, and what is made once: each radius's corner (per row from the middle out: its
/// whole pixels and its partly covered ones' covers), each shadow's levels down its edge and its ramps across.
pub struct Kit { ease: Machine, cube: Machine, circle: Machine, corner: Machine, panel: Machine, shadow: Machine, text: Machine, digits: Machine, cycle: Machine,
    cover: Machine, fade: Machine, mix: Machine, pub atlas: Atlas, corners: HashMap<i16, Vec<(i16, Vec<u8>)>>, shadows: HashMap<(i16, i16), (Vec<i16>, Vec<Vec<i16>>)> }

/// The spinner: spokes, their darkest and lightest grey.
const SPOKES: i16 = 12;
const GREYS: (i16, i16) = (70, 255);

impl Kit {
    /// The machines, and the corners and shadow ramps every frame of `scene` needs, made on CARDIAC.
    pub fn new(demo: &Path, host: &mut Host, scene: &Scene) -> Kit {
        let groups: Vec<Group> = (0..scene.frames).flat_map(|f| (scene.frame)(f)).collect();
        Kit::with(demo, host, &groups)
    }

    /// The machines, and the corners and shadow ramps that `groups` (any frames' groups) need, made on CARDIAC.
    pub fn with(demo: &Path, host: &mut Host, groups: &[Group]) -> Kit {
        let r = |n: &str| rust(demo, n);
        let mut kit = Kit { ease: r("ui_ease"), cube: r("ui_cube"), circle: r("ui_circle"), corner: r("ui_corner"), panel: r("ui_panel"), shadow: r("ui_shadow"), text: r("ui_text"),
            digits: r("ui_digits"), cycle: r("ui_cycle"), cover: r("ui_cover"), fade: r("ui_fade"), mix: r("umix"), atlas: Atlas::read(demo), corners: HashMap::new(), shadows: HashMap::new() };
        let (mut radii, mut shadows) = (vec![], vec![]);
        for g in groups { for it in &g.items { match *it {
            Item::Panel { r, .. } if r > 0 && !radii.contains(&r) => radii.push(r),
            Item::Shadow { blur, level, .. } if !shadows.contains(&(blur, level)) => shadows.push((blur, level)),
            _ => {}
        } } }
        for r in radii {
            let reach = host.go1(&kit.circle, &[&[r as i32]], 4 * r as usize);
            let out = host.gon(&kit.corner, &[&[r as i32], &reach]);
            let (mut rows, mut i) = (vec![], 0);
            for _ in 0..r { let whole = out[i] as i16; i += 1; let mut covers = vec![]; while out[i] != 0 { covers.push(out[i] as u8); i += 1; } i += 1; rows.push((whole, covers)); }
            kit.corners.insert(r, rows);
        }
        for (blur, level) in shadows {
            // the levels down an edge, then each level's ramp across: smooth, 0 to the level over 2 blur + 1 steps
            let n = 2 * blur + 1;
            let mut deck = vec![];
            for top in std::iter::once(level).chain(0..=level) { for k in 0..2 * blur { deck.extend([3, 0, top, k + 1, n]); } }
            let count = deck.len() / 5;
            let out = host.go1(&kit.ease, &[&[count as i32], &deck.iter().map(|&v| v as i32).collect::<Vec<_>>()], count);
            let ramps: Vec<Vec<i16>> = out.chunks(2 * blur as usize).map(|c| c.iter().map(|&v| v as i16).collect()).collect();
            kit.shadows.insert((blur, level), (ramps[0].clone(), ramps[1..].to_vec()));
        }
        kit
    }

    /// Frame f of the scene: W x H pixels, each red, green, blue and cover (0-32).
    pub fn frame(&self, host: &mut Host, scene: &Scene, f: usize) -> Vec<u8> {
        self.layer(host, scene.w, scene.h, scene.ox, scene.oy, &(scene.frame)(f))
    }

    /// Groups of items on a layer of w x h, their places counted from (-ox, -oy), the layer's top left: w x h
    /// pixels, each red, green, blue and cover (0-32).
    pub fn layer(&self, host: &mut Host, w: usize, h: usize, ox: i16, oy: i16, groups: &[Group]) -> Vec<u8> {
        let scene = Scene { name: "", w, h, x: 0, y: 0, ox, oy, frames: 0, frame: |_| vec![] };
        let scene = &scene;
        // every number that moves, on the ease machine
        let mut deck: Vec<i32> = vec![];
        // every glide (eased out), on the cube machine
        let mut cubes: Vec<i32> = vec![];
        work_out(groups, &mut |v| { match v {
            Val::Ease { smooth, from, to, j, n } => deck.extend([if smooth { 3 } else { 1 }, from as i32, to as i32, j as i32, n as i32]),
            Val::Cube { from, to, j, n } => cubes.extend([from as i32, to as i32, j as i32, n as i32]),
            Val::K(_) => {}
        } 0 });
        let eased = if deck.is_empty() { vec![] } else { host.go1(&self.ease, &[&[(deck.len() / 5) as i32], &deck], deck.len() / 5) };
        let glided = if cubes.is_empty() { vec![] } else { host.go1(&self.cube, &[&[(cubes.len() / 4) as i32], &cubes], cubes.len() / 4) };
        let (mut next, mut next_cube) = (eased.into_iter(), glided.into_iter());
        let mut groups = work_out(groups, &mut |v| match v { Val::K(k) => k, Val::Ease { .. } => next.next().unwrap() as i16, Val::Cube { .. } => next_cube.next().unwrap() as i16 });
        // numbers written out, on the digits machine
        let nums: Vec<i32> = groups.iter().flat_map(|g| g.items.iter()).filter_map(|it| match it { Done::Text { num: Some((v, _)), .. } => Some(*v as i32), _ => None }).collect();
        if !nums.is_empty() {
            let out = host.gon(&self.digits, &[&[nums.len() as i32], &nums]);
            let mut i = 0;
            for it in groups.iter_mut().flat_map(|g| g.items.iter_mut()) {
                if let Done::Text { num: Some((_, suffix)), chars, .. } = it {
                    let n = out[i] as usize;
                    *chars = out[i + 1..i + 1 + n].iter().map(|&c| c as u32).chain(suffix.chars().map(|c| c as u32)).collect();
                    i += 1 + n;
                }
            }
        }
        // where every glyph and sprite goes, on the text machine: (face, code, colour) each, in order. A line's deck:
        // its place and its group's move, 0 (its top is its anchor), 0 (it starts there), its glyphs' advances and
        // offsets; a sprite is a line of one glyph whose advance is 0.
        let mut deck: Vec<i32> = vec![0];
        let mut placed: Vec<(usize, u32, [u8; 3], i16)> = vec![];
        let mut lines = 0;
        for g in &groups { for it in &g.items {
            let mut sprite = |x: i16, y: i16, face: usize, code: usize, rgb: [u8; 3], fade: i16| {
                let gl = self.atlas.glyph(face, code as u32);
                deck.extend([x, g.dx, y, g.dy, 0, 0, 1, 0, gl.ox, gl.oy].map(|v| v as i32));
                placed.push((face, code as u32, rgb, fade));
                lines += 1;
            };
            match it {
                Done::Sprite { x, y, face, code, rgb, fade } => sprite(*x, *y, *face, *code, *rgb, *fade),
                Done::Spinner { x, y, phase, fade } => {
                    let greys = host.go1(&self.cycle, &[&[*phase as i32, SPOKES as i32, GREYS.0 as i32, GREYS.1 as i32]], SPOKES as usize);
                    for (k, &grey) in greys.iter().enumerate() { let v = grey as u8; sprite(*x, *y, 9, SPOKE + k, [v, v, v], *fade); }
                }
                Done::Text { x, y, face, across, middle, rgb, chars, .. } => {
                    let gs: Vec<&Glyph> = chars.iter().map(|&c| self.atlas.glyph(*face, c)).collect();
                    let across = match across { Across::Left => 0, Across::Middle => 1, Across::Right => 2 };
                    deck.extend([*x, g.dx, *y, g.dy, if *middle { self.atlas.mid[*face] } else { 0 }, across, gs.len() as i16].map(|v| v as i32));
                    if across > 0 { deck.extend(gs.iter().map(|gl| gl.adv as i32)); }
                    for gl in &gs { deck.extend([gl.adv, gl.ox, gl.oy].map(|v| v as i32)); }
                    placed.extend(chars.iter().map(|&c| (*face, c, *rgb, 32)));
                    lines += 1;
                }
                _ => {}
            }
        } }
        deck[0] = lines;
        let places = host.go1(&self.text, &[&deck], 2 * placed.len());
        // every row of every box, on the panel machine: its place and its group's move, its size, its corners' radius
        // (0: square), then each row's corner (0, and no partly covered pixels)
        let (mut deck, mut boxes) = (vec![0], 0);
        let (mut sdeck, mut shadows) = (vec![0], 0);
        for g in &groups { for it in &g.items { match *it {
            Done::Panel { x, y, w, h, r, top, bottom, .. } => {
                deck.extend([x, g.dx, y, g.dy, w, h, r].map(|v| v as i32));
                let corner = self.corners.get(&r);
                for i in 0..h {
                    // a corner's row, picked by the box's row: the top r rows from the top, the bottom r from the middle out
                    let row = if top && i < r { Some(r - 1 - i) } else if bottom && i >= h - r { Some(i - (h - r)) } else { None };
                    match row { Some(k) => { let (whole, covers) = &corner.unwrap()[k as usize]; deck.extend([*whole as i32, covers.len() as i32]) } None => deck.extend([r as i32, 0]) }
                }
                boxes += 1;
            }
            Done::Shadow { x, y, w, h, blur, .. } => { sdeck.extend([x, g.dx, y, g.dy, w, h, blur].map(|v| v as i32)); shadows += 1; }
            _ => {}
        } } }
        deck[0] = boxes;
        sdeck[0] = shadows;
        let rows = if boxes > 0 { host.gon(&self.panel, &[&deck]) } else { vec![] };
        let edges = if shadows > 0 { host.go1(&self.shadow, &[&sdeck], 8 * shadows as usize) } else { vec![] };
        // lay the items on, in order
        let (w, h) = (scene.w as i16, scene.h as i16);
        let mut layer = vec![0u8; scene.w * scene.h * 4];
        let (mut glyph, mut row_at, mut edge_at) = (0, 0, 0);
        for g in &groups {
            // the layer's bounds and the group's clip, in the scene's places
            let mut clip = [-scene.ox, -scene.oy, w - scene.ox, h - scene.oy];
            if let Some(c) = g.clip { clip = [clip[0].max(c[0]), clip[1].max(c[1]), clip[2].min(c[2]), clip[3].min(c[3])]; }
            let index = |x: i16, y: i16| -> Option<usize> { (x >= clip[0] && x < clip[2] && y >= clip[1] && y < clip[3]).then(|| ((y + scene.oy) as usize * scene.w + (x + scene.ox) as usize) * 4) };
            // a group laid on faded is made by itself first
            let mut own = if g.fade < 32 { Some(vec![0u8; layer.len()]) } else { None };
            let target: &mut Vec<u8> = own.as_mut().unwrap_or(&mut layer);
            for it in &g.items {
                let mut px: Vec<Px> = vec![];
                let mut fade = 32;
                match *it {
                    Done::Panel { h, r, top, bottom, rgb, .. } => {
                        // each row's edges: with square corners the whole row from its second to its third
                        let y0 = rows[row_at] as i16;
                        row_at += 1;
                        let corner = self.corners.get(&r);
                        for i in 0..h {
                            let e: Vec<i16> = rows[row_at..row_at + 4].iter().map(|&v| v as i16).collect();
                            row_at += 4;
                            let y = y0 + i;
                            let row = if top && i < r { Some(r - 1 - i) } else if bottom && i >= h - r { Some(i - (h - r)) } else { None };
                            let covers: &[u8] = match row { Some(k) => &corner.unwrap()[k as usize].1, None => &[] };
                            // the partly covered pixels at the left are the corner's row backwards, at the right forwards
                            for (k, x) in (e[0]..e[1]).enumerate() { if let Some(i) = index(x, y) { px.push((i, covers[covers.len() - 1 - k], rgb)); } }
                            for x in e[1]..e[2] { if let Some(i) = index(x, y) { target[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 32]); } }
                            for (k, x) in (e[2]..e[3]).enumerate() { if let Some(i) = index(x, y) { px.push((i, covers[k], rgb)); } }
                        }
                    }
                    Done::Shadow { blur, level, .. } => {
                        let e: Vec<i16> = edges[edge_at..edge_at + 8].iter().map(|&v| v as i16).collect();
                        edge_at += 8;
                        let (down, ramps) = &self.shadows[&(blur, level)];
                        for y in e[4]..e[7] {
                            // the row's level: down the top edge, the middle, up the bottom edge (picked by the row)
                            let s = if y < e[5] { down[(y - e[4]) as usize] } else if y < e[6] { level } else { down[(e[7] - 1 - y) as usize] };
                            let ramp = &ramps[s as usize];
                            for x in e[0]..e[3] {
                                let Some(i) = index(x, y) else { continue };
                                let c = if x < e[1] { ramp[(x - e[0]) as usize] } else if x < e[2] { s } else { ramp[(e[3] - 1 - x) as usize] } as u8;
                                if c == 0 { continue }
                                if (e[1]..e[2]).contains(&x) && target[i + 3] == 0 { target[i..i + 4].copy_from_slice(&[0, 0, 0, c]); } else { px.push((i, c, [0, 0, 0])); }
                            }
                        }
                    }
                    Done::Text { ref chars, .. } => {
                        for _ in chars { px.extend(self.glyph_px(&placed[glyph], &places[2 * glyph..2 * glyph + 2], &index)); glyph += 1; }
                    }
                    Done::Sprite { fade: f, .. } => { fade = f; px.extend(self.glyph_px(&placed[glyph], &places[2 * glyph..2 * glyph + 2], &index)); glyph += 1; }
                    Done::Spinner { fade: f, .. } => { fade = f; for _ in 0..SPOKES { px.extend(self.glyph_px(&placed[glyph], &places[2 * glyph..2 * glyph + 2], &index)); glyph += 1; } }
                }
                self.lay(host, target, &px, fade);
            }
            if let Some(own) = own {
                // the group laid on the layer by its fade, each pixel it covers
                let px: Vec<Px> = (0..own.len() / 4).filter(|&k| own[4 * k + 3] > 0).map(|k| (4 * k, own[4 * k + 3], [own[4 * k], own[4 * k + 1], own[4 * k + 2]])).collect();
                self.lay(host, &mut layer, &px, g.fade);
            }
        }
        layer
    }

    /// A glyph's or sprite's pixels, from the atlas, at the place the text machine punched.
    fn glyph_px(&self, &(face, code, rgb, _): &(usize, u32, [u8; 3], i16), at: &[i32], index: &dyn Fn(i16, i16) -> Option<usize>) -> Vec<Px> {
        let gl = self.atlas.glyph(face, code);
        let own = self.atlas.colours.get(&(face, code));
        let mut px = vec![];
        for row in 0..gl.h { for col in 0..gl.w {
            let k = row * gl.w + col;
            let c = self.atlas.cover[gl.at + k];
            if c == 0 { continue }
            // a colour glyph (an emoji) in its own colours
            let rgb = own.map_or(rgb, |&o| [self.atlas.cover[o + 3 * k], self.atlas.cover[o + 3 * k + 1], self.atlas.cover[o + 3 * k + 2]]);
            if let Some(i) = index(at[0] as i16 + col as i16, at[1] as i16 + row as i16) { px.push((i, c, rgb)); }
        } }
        px
    }

    /// Lay pixels on, in order, at `fade` 32nds: the fade machine takes their covers down, the cover machine works
    /// out each one's cover over what is under it and its colour's share, the mix machine its colour. A pixel met
    /// twice (glyphs that touch) is laid on after the ones before it.
    fn lay(&self, host: &mut Host, target: &mut [u8], px: &[Px], fade: i16) {
        let mut from = 0;
        let mut seen = std::collections::HashSet::new();
        for k in 0..=px.len() {
            if k < px.len() && seen.insert(px[k].0) { continue }
            self.lay_some(host, target, &px[from..k], fade);
            from = k;
            seen.clear();
            if k < px.len() { seen.insert(px[k].0); }
        }
    }

    /// Pixels alike (the same cover and colour over the same pixel) are worked out once and the result copied to
    /// each: a panel faded over an empty layer, a glyph's edge over a flat box.
    fn lay_some(&self, host: &mut Host, target: &mut [u8], px: &[Px], fade: i16) {
        let mut seen: HashMap<(u8, [u8; 3], [u8; 4]), usize> = HashMap::new();
        let (mut unique, mut scratch, mut slot): (Vec<Px>, Vec<u8>, Vec<usize>) = (vec![], vec![], vec![]);
        for p in px {
            let under = [target[p.0], target[p.0 + 1], target[p.0 + 2], target[p.0 + 3]];
            let n = *seen.entry((p.1, p.2, under)).or_insert_with(|| { unique.push((4 * unique.len(), p.1, p.2)); scratch.extend(under); unique.len() - 1 });
            slot.push(n);
        }
        self.lay_runs(host, &mut scratch, &unique, fade);
        for (p, &n) in px.iter().zip(&slot) { target[p.0..p.0 + 4].copy_from_slice(&scratch[4 * n..4 * n + 4]); }
    }

    /// At most 333 pixels a run: a deck's count is a card, and the mix machine's counts three a pixel.
    fn lay_runs(&self, host: &mut Host, target: &mut [u8], px: &[Px], fade: i16) {
        if px.len() > 333 { for part in px.chunks(333) { self.lay_runs(host, target, part, fade); } return }
        if px.is_empty() { return }
        let mut covers: Vec<i32> = px.iter().map(|p| p.1 as i32).collect();
        if fade < 32 { covers = host.go1(&self.fade, &[&[fade as i32, px.len() as i32], &covers], px.len()); }
        let deck: Vec<i32> = px.iter().zip(&covers).flat_map(|(p, &c)| [c, target[p.0 + 3] as i32]).collect();
        let both = host.go1(&self.cover, &[&[px.len() as i32], &deck], 2 * px.len());
        let mut deck = vec![(3 * px.len()) as i32];
        for (k, p) in px.iter().enumerate() { for c in 0..3 { deck.extend([target[p.0 + c] as i32, p.2[c] as i32, both[2 * k + 1]]); } }
        let mixed = host.go1(&self.mix, &[&deck], 3 * px.len());
        for (k, p) in px.iter().enumerate() {
            target[p.0..p.0 + 3].copy_from_slice(&[mixed[3 * k] as u8, mixed[3 * k + 1] as u8, mixed[3 * k + 2] as u8]);
            target[p.0 + 3] = both[2 * k] as u8;
        }
    }
}

/// The Discord notification's layer, over the polys.
pub fn notify(demo: &Path, refbin: &str) { render(demo, refbin, "notify") }
/// The bitcoin miner's layer, over the hung dive.
pub fn miner(demo: &Path, refbin: &str) { render(demo, refbin, "miner") }

/// The accelerated drive's VIDEO OPTIONS menu, over the dimmed drive.
pub fn videoopts(demo: &Path, refbin: &str) { render(demo, refbin, "videoopts") }
/// The accelerated drive's brag.
pub fn accelbrag(demo: &Path, refbin: &str) { render(demo, refbin, "accelbrag") }

/// Render scene `name`'s layer (ui/NAME.rgba): every frame, checked against the reference. Each frame is made from
/// its number alone, so the player may split it into blocks.
pub fn render(demo: &Path, refbin: &str, name: &str) {
    let scene = scene(name);
    let mut host = Host::new();
    let kit = Kit::new(demo, &mut host, scene);
    let size = scene.w * scene.h * 4;
    let mut buf = vec![0u8; scene.frames * size];
    for f in live::frames(scene.frames) { buf[f * size..(f + 1) * size].copy_from_slice(&kit.frame(&mut host, scene, f)); }
    host.done();
    let r = reference(refbin, &[Path::new(name), demo]);
    let out = demo.join(format!("ui/{name}.rgba"));
    let extra = format!(" ({}x{} RGB and cover, for the cut's over at {} {})", scene.w, scene.h, scene.x, scene.y);
    // a layer's frame is W x H x 4 bytes: as frames of RGB, W 4/3 wide
    match name {
        "notify" => finish(name, &host, &cardiac_host::Video::<892, 200> { frames: scene.frames, buf }, &r, &out, &extra),
        "miner" => finish(name, &host, &cardiac_host::Video::<1288, 700> { frames: scene.frames, buf }, &r, &out, &extra),
        "videoopts" => finish(name, &host, &cardiac_host::Video::<1596, 628> { frames: scene.frames, buf }, &r, &out, &extra),
        "accelbrag" => finish(name, &host, &cardiac_host::Video::<588, 187> { frames: scene.frames, buf }, &r, &out, &extra),
        n => panic!("no layer size for {n}"),
    }
}
