// The rooms: the dorm (halfway through Mode X's mirror ball the camera pulls back out of a 17-inch
// CRT into a cramped dorm room, four on a couch watching; Dope starts on the monitor and we dive back in) and the
// screening room (halfway through the accelerated drive we pull back out of the projected picture into a hall, a
// loose crowd standing, the projector on its cart; the drive fades, THE END, the prompt it ran from, `seaplusplus`
// typed, Unseen Planets presents SEA++ and LOADING while we zoom slowly back in; the dive starts on the screen while
// we're still in the room). The designs are prototypes/dorm.py and prototypes/screening.py.
//
// Written once against the train's operations (segments/train/ops.rs). Per frame CARDIAC works out: the screen's
// mean colour (from a grid of its picture's pixels, through the bytes-to-linear table); each palette entry's light,
// A + B m faded, and its sRGB byte; the zoom's scale, centre and the walks from each column and row of the frame to
// the art's and the picture's; the fades' and THE END's byte tables; the desktop's typed count, cursor, dots and
// bar; the brag's mix where it lies on the projected drive (we zoom in over the people's heads: none of them covers the
// screen).
// The host fetches each pixel from the printed tables and the pictures by those indices and picks by the masks.

pub const W: i64 = 1536;
pub const H: i64 = 768;
pub const DORM_N: i64 = 118;          // modex's frames 66..149 (84), Dope's 0..33 (34)
pub const HALL_N: i64 = 231;          // the drive's 59..117 (59), THE END (24), the desktop (100), the dive's 0..47 (48)
pub const FRAMES: i64 = DORM_N + HALL_N;
pub const MODEX0: i64 = 66;           // the dorm's first frame is modex's 66th
pub const MODEX_N: i64 = 84;
pub const POLYS0: i64 = 200;          // the cut's Dope starts at the polys' 200th frame
pub const POLYS_N: i64 = 34;
pub const DRIVE0: i64 = 59;           // the screening room's first frame is the accelerated drive's 59th
pub const DRIVE_N: i64 = 59;
pub const BRAG_AT: i64 = 82;          // the brag lies over the drive from its 82nd frame (the cut's over line)
pub const END_N: i64 = 24;
pub const TYPE0: i64 = 24;            // the prompt waits 2 s before the typing
pub const TYPED: &str = "seaplusplus";
pub const LOAD_N: i64 = 60;           // LOADING takes 5 s
pub const FULL_N: i64 = 3;            // the full bar holds a quarter second before the dive starts
pub const DESK_N: i64 = TYPE0 + 11 + 2 + LOAD_N + FULL_N;   // 100: the desktop's frames before the dive starts
pub const ZIN: i64 = 108;             // the screening room's zoom in, from LOADING's start
// 48: the dive's frames 0..47 on the screen, so that the cut, which carries the dive on from its frame 48, shows
// none of them twice
pub const DIVE_N: i64 = HALL_N - DRIVE_N - END_N - DESK_N;
pub const BACK_D: i64 = 24;           // the dorm's pull-back
pub const HOLD_D: i64 = 18;           // Dope on the monitor before the dive
pub const DIVE_D: i64 = 16;
pub const BACK_H: i64 = 26;           // the screening room's pull-back
pub const FADE_H: i64 = 12;           // the drive's fade to black

/// A room's picture: a palette index a pixel, the palette (A then B, three each, ten-thousandths), the screen's mask,
/// its box on the frame (hundredths of a frame pixel) and how many art pixels a frame
/// pixel.
pub struct Pic { pub idx: Tab, pub pal: Vec<[i64; 6]>, pub mask: Tab, pub bx: [i64; 4], pub art: i64 }

pub struct World {
    pub dorm: Pic, pub dormin: Pic, pub hall: Pic,
    pub dorm_screen: [i64; 4], pub hall_screen: [i64; 4], pub dorm_mid: [i64; 2], pub hall_mid: [i64; 2],
    pub srgb: Tab, pub lin8: Tab, pub ease: Tab, pub easep: Tab, pub dorm_s0: Tab, pub dorm_sd: Tab, pub hall_s0: Tab,
    pub end_cover: Tab, pub desk_base: Vec<u8>, pub glyphs: Vec<i64>,
}

/// The pictures the screens show, as bytes: modex's frames 66..149 (1536 x 768), Dope's 200..233 (192 x 96), the
/// drive's 59..117 (1536 x 768), the dive's 0..47 (1536 x 768), the brag's layer (441 x 187, RGB and cover 0-32).
pub struct Srcs { pub modex: Vec<u8>, pub polys: Vec<u8>, pub drive: Vec<u8>, pub dive: Vec<u8>, pub brag: Vec<u8> }

/// A number of a data record, stored in 1, 2 or 4 bytes (segments/shared/records.py).
fn stored_number(b: &[u8]) -> i64 {
    match b.len() { 1 => b[0] as i8 as i64, 2 => i16::from_le_bytes([b[0], b[1]]) as i64, _ => i32::from_le_bytes(b.try_into().unwrap()) as i64 }
}

fn records(path: &str) -> std::collections::BTreeMap<String, (Vec<usize>, Vec<i64>)> {
    let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut out = std::collections::BTreeMap::new();
    let mut at = 0;
    let u32_at = |b: &[u8], at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    while at < b.len() {
        let n = u32_at(&b, at); at += 4;
        let name = String::from_utf8(b[at..at + n].to_vec()).unwrap(); at += n;
        // the count of dimensions, with the bytes per number in its second byte (0 for 4)
        let nd = u32_at(&b, at); at += 4;
        let w = match nd >> 8 { 0 => 4, w => w };
        let nd = nd & 255;
        let dims: Vec<usize> = (0..nd).map(|i| u32_at(&b, at + 4 * i)).collect(); at += 4 * nd;
        let len: usize = dims.iter().product();
        let v: Vec<i64> = (0..len).map(|i| stored_number(&b[at + w * i..at + w * (i + 1)])).collect();
        at += w * len;
        out.insert(name, (dims, v));
    }
    out
}

pub fn load(dir: &str) -> World {
    let mut r = records(&format!("{dir}/rooms/data/rooms.bin"));
    let mut take = |n: &str| r.remove(n).unwrap_or_else(|| panic!("rooms.bin: no {n}"));
    let tab = |name: &'static str, (d, v): (Vec<usize>, Vec<i64>)| Tab { name, w: if d.len() > 1 { d[1] } else { v.len() }, data: v };
    let four = |(_, v): (Vec<usize>, Vec<i64>)| [v[0], v[1], v[2], v[3]];
    let mut pic = |n: &'static str, bx: [i64; 4], art: i64, take: &mut dyn FnMut(&str) -> (Vec<usize>, Vec<i64>)| {
        let pal = take(&format!("{n}_pal")).1.chunks(6).map(|c| [c[0], c[1], c[2], c[3], c[4], c[5]]).collect();
        Pic { idx: tab("idx", take(&format!("{n}_idx"))), pal, mask: tab("mask", take(&format!("{n}_mask"))), bx, art }
    };
    let inset = four(take("dorm_inset"));
    let two = |(_, v): (Vec<usize>, Vec<i64>)| [v[0], v[1]];
    let dorm = pic("dorm", [0, 0, W * 100, H * 100], 2, &mut take);
    let dormin = pic("dormin", inset, 8, &mut take);
    let hall = pic("hall", [0, 0, W * 100, H * 100], 2, &mut take);
    World {
        dorm, dormin, hall,
        dorm_screen: four(take("dorm_screen")), hall_screen: four(take("hall_screen")),
        dorm_mid: two(take("dorm_mid")), hall_mid: two(take("hall_mid")),
        srgb: tab("srgb", take("srgb")), lin8: tab("lin8", take("lin8")), ease: tab("ease", take("ease")),
        easep: tab("easep", take("easep")), dorm_s0: tab("dorm_s0", take("dorm_s0")), dorm_sd: tab("dorm_sd", take("dorm_sd")),
        hall_s0: tab("hall_s0", take("hall_s0")), end_cover: tab("end_cover", take("end_cover")),
        desk_base: take("desk_base").1.iter().map(|&v| v as u8).collect(), glyphs: take("glyphs").1,
    }
}

fn one(v: V) -> i64 { v[0] }

/// The schedule: what frame k shows (the picture, its frame) and how (how far the zoom is out of the screen, 0..1000
/// thousandths of the way: 1000 the screen filling the frame; which zoom; the room's light).
#[derive(Clone, Copy, PartialEq)]
pub enum Show { Modex(i64), Polys(i64), Drive(i64), End(i64), Desk(i64), Dive(i64) }

/// How far along a ramp of n frames frame f is, thousandths (0..1000), on CARDIAC.
fn along<O: Ops>(o: &mut O, f: i64, n: i64) -> i64 { along_at(o, f, 1, 0, n) }

/// `along` for s f - at frames (s 1 or -1): the frame's place taken in its sum on the machines, not worked out first.
fn along_at<O: Ops>(o: &mut O, f: i64, s: i64, at: i64, n: i64) -> i64 {
    let x = one(o.lin(&[(K(f), Some(K(1000 * s))), (K(-1000 * at), None)]));
    let x = one(o.clamp(K(x), 0, n * 1000));
    one(o.div(K(x), K(n), 0))
}

/// The frame: its picture, the zoom (the scale table and how far along it, thousandths: 0 the screen filling the
/// frame), the room's light (thousandths), the room.
struct Plan { show: Show, u: i64, s_tab: u8, light: i64, hall: bool }

fn plan<O: Ops>(o: &mut O, w: &World, k: i64) -> Plan {
    let e = |o: &mut O, t: &Tab, a: i64| one(o.tab(t, K(a)));
    if k < DORM_N {
        if k < MODEX_N {
            // out of the screen over BACK_D frames (the zoom's u: 0 in, 1000 out), the room's light up over 8
            let a = along(o, k, BACK_D);
            let u = e(o, &w.ease, a);
            let light = along(o, k, 8);
            Plan { show: Show::Modex(k), u, s_tab: 0, light, hall: false }
        } else {
            // Dope on the monitor, then the dive: in over DIVE_D, the room's light going with it
            let g = k - MODEX_N;
            let a = along_at(o, k, 1, MODEX_N + HOLD_D, DIVE_D);
            let t = e(o, &w.ease, a);
            let u = one(o.sum(&[(K(t), -1)], 1000));
            Plan { show: Show::Polys(g), u, s_tab: 1, light: u, hall: false }
        }
    } else {
        let h = k - DORM_N;
        if h < DRIVE_N + END_N {
            let a = along_at(o, k, 1, DORM_N, BACK_H);
            let u = e(o, &w.ease, a);
            let light = along_at(o, k, 1, DORM_N, 8);
            let show = if h < DRIVE_N { Show::Drive(h) } else { Show::End(h - DRIVE_N) };
            Plan { show, u, s_tab: 2, light, hall: true }
        } else {
            // the desktop, the dive: the zoom in from LOADING's start (z0), slow at first
            let f = h - DRIVE_N - END_N;
            let z0 = TYPE0 + 11 + 2;
            let a = along_at(o, k, 1, DORM_N + DRIVE_N + END_N + z0, ZIN);
            let t = e(o, &w.easep, a);
            let u = one(o.sum(&[(K(t), -1)], 1000));
            let show = if f < DESK_N { Show::Desk(f) } else { Show::Dive(f - DESK_N) };
            Plan { show, u, s_tab: 2, light: u, hall: true }
        }
    }
}

/// The zoom's scale (thousandths) and centre (hundredths of a frame pixel): s = s0^(1 - u) from the printed table,
/// the centre from the screen's middle (all the way in) to the frame's (out), by (s - 1) / (s0 - 1).
fn zoom<O: Ops>(o: &mut O, w: &World, p: &Plan) -> (i64, [i64; 2]) {
    let t = [&w.dorm_s0, &w.dorm_sd, &w.hall_s0][p.s_tab as usize];
    let s = one(o.tab(t, K(p.u)));
    // wt = (s - 1000) 1000 / (s0 - 1000)
    let num = one(o.sum(&[(K(s), 1)], -1000));
    let num = one(o.lin(&[(K(num), Some(K(1000)))]));
    let den = one(o.sum(&[(K(t.data[0]), 1)], -1000));
    let wt = one(o.div(K(num), K(den), 0));
    let rest = one(o.sum(&[(K(wt), -1)], 1000));
    let m = if p.hall { w.hall_mid } else { w.dorm_mid };
    let c: [i64; 2] = std::array::from_fn(|i| {
        let frm = if i == 0 { W * 50 } else { H * 50 };
        let x = one(o.lin(&[(K(m[i]), Some(K(wt))), (K(frm), Some(K(rest)))]));
        one(o.div(K(x), K(1000), 0))
    });
    (s, c)
}

/// What a walk goes to: a picture laid on the screen's box (its pixels across the box, the box's size in hundredths),
/// or a room picture's art (art pixels a frame pixel).
#[derive(Clone, Copy)]
enum To { Picture { pw: i64, size: i64 }, Art(i64) }

/// The walk from the frame's n columns (or rows) to a picture's: each frame pixel's place there, clamped to 0..len - 1,
/// and whether it lies in it. u = c + (x - n / 2) / S (hundredths of a frame pixel, S = s / 1000), from lo on.
fn walk_to<O: Ops>(o: &mut O, n: i64, s: i64, c: i64, lo: i64, to: To, len: i64) -> (V, V) {
    // the slope: the picture's thousandths a frame pixel
    let a = match to {
        To::Picture { pw, size } => { let q = one(o.div(K(100_000_000), K(s), 0)); let x = one(o.lin(&[(K(q), Some(K(pw)))])); one(o.div(K(x), K(size), 0)) }
        To::Art(art) => one(o.div(K(1_000_000 * art), K(s), 0)),
    };
    // the start: u0 - lo (hundredths), u0 = c - (n / 2) 100000 / s
    let half = one(o.div(K(n / 2 * 100_000), K(s), 0));
    let d = one(o.sum(&[(K(c), 1), (K(half), -1), (K(lo), -1)], 0));
    let (va, f0) = match to {
        To::Picture { pw, size } => {
            // (d pw / size): its whole pixels and thousandths
            let x = one(o.lin(&[(K(d), Some(K(pw)))]));
            let q = one(o.div(K(x), K(size), 0));
            let p = one(o.lin(&[(K(q), Some(K(size)))]));
            let r = one(o.lin(&[(K(x), None), (K(p), Some(K(-1)))]));
            (q, one(o.div(K(r), K(size), 1)))
        }
        To::Art(art) => {
            let x = one(o.lin(&[(K(d), Some(K(10 * art)))]));
            let q = one(o.div(K(x), K(1000), 0));
            (q, one(o.lin(&[(K(x), None), (K(q), Some(K(-1000)))])))
        }
    };
    let wv = o.walka(&[(n, va, a, f0)]);
    let i1 = o.lt(K(-1), v(&wv));
    let i2 = o.lt(v(&wv), K(len));
    let both = o.sum(&[(v(&i1), 1), (v(&i2), 1)], 0);
    let inside = o.lt(K(1), v(&both));
    (o.clamp(v(&wv), 0, len - 1), inside)
}

/// The room's light this frame: each palette entry's lit value, (A + B m) light, as sRGB bytes.
fn light_pal<O: Ops>(o: &mut O, w: &World, pal: &[[i64; 6]], m: [i64; 3], light: i64) -> [V; 3] {
    std::array::from_fn(|c| {
        let a: V = pal.iter().map(|p| p[c]).collect();
        let b: V = pal.iter().map(|p| p[3 + c]).collect();
        // (A 1000 + B m) / 1000, then times the light / 1000: ten-thousandths
        let x = o.lin(&[(v(&a), Some(K(1000))), (v(&b), Some(K(m[c])))]);
        let x = o.div(v(&x), K(1000), 0);
        let x = o.lin(&[(v(&x), Some(K(light)))]);
        let x = o.div(v(&x), K(1000), 0);
        let x = o.clamp(v(&x), 0, w.srgb.data.len() as i64 - 1);
        o.tab(&w.srgb, v(&x))
    })
}

/// The screen's mean colour (linear thousandths) from a picture's pixels: a grid of 32 x 16 over (x0..x1, y0..y1) of
/// an image of width iw, RGB, each scaled by `scale` (thousandths).
fn mean_of<O: Ops>(o: &mut O, w: &World, img: &[u8], iw: usize, x0: usize, x1: usize, y0: usize, y1: usize, scale: i64) -> [i64; 3] {
    std::array::from_fn(|c| {
        let bytes: V = (0..16).flat_map(|j| (0..32).map(move |i| (i, j))).map(|(i, j)| {
            let x = x0 + (x1 - x0) * (2 * i + 1) / 64;
            let y = y0 + (y1 - y0) * (2 * j + 1) / 32;
            img[(y * iw + x) * 3 + c] as i64
        }).collect();
        let mut l = o.tab(&w.lin8, v(&bytes));
        while l.len() > 1 {
            let h = l.len() / 2;
            let (a, b) = (l[..h].to_vec(), l[h..2 * h].to_vec());
            l = o.sum(&[(v(&a), 1), (v(&b), 1)], 0);
        }
        let mean = one(o.div(K(l[0]), K(512), 0));
        let x = one(o.lin(&[(K(mean), Some(K(scale)))]));
        one(o.div(K(x), K(1000), 0))
    })
}

/// A byte table: b x num / den for b 0..255 (the fades), on CARDIAC.
fn scale_tab<O: Ops>(o: &mut O, num: i64, den: i64) -> Tab {
    let b: V = (0..256).collect();
    let x = o.lin(&[(v(&b), Some(K(num)))]);
    Tab { name: "fade", w: 256, data: o.div(v(&x), K(den), 0) }
}

/// The desktop at frame f (640 x 480 RGB): the base, and in the prompt's text area the lines (the VGA face, light
/// grey): drive.exe's line, the prompt and what's been typed, then the banner, LOADING and its bar; the cursor.
fn desktop<O: Ops>(o: &mut O, w: &World, f: i64) -> Vec<u8> {
    let mut img = w.desk_base.clone();
    let typed = one(o.sum(&[(K(f), 1)], -TYPE0));
    let typed = one(o.clamp(K(typed), 0, TYPED.len() as i64)) as usize;
    let type_end = TYPE0 + TYPED.len() as i64;
    let loading = one(o.lt(K(type_end + 1), K(f))) == 1;
    let mut lines: Vec<String> = vec!["C:\\DEMOS>drive.exe".into(), "".into(), format!("C:\\DEMOS>{}", &TYPED[..typed])];
    let mut cursor = false;
    if loading {
        let since = one(o.sum(&[(K(f), 1)], -type_end));
        let (q, _) = o.divmod(K(since), K(4));
        let (_, dots) = o.divmod(K(q[0]), K(4));
        let n = one(o.lin(&[(K(since - 0), Some(K(30)))]));
        let n = one(o.sum(&[(K(n), 1)], -60));
        let n = one(o.div(K(n), K(LOAD_N), 0));
        let n = one(o.clamp(K(n), 0, 30)) as usize;
        lines.push("".into());
        lines.extend(banner());
        lines.push("".into());
        lines.push(format!("   ><(((o>   ~   ~   ~   LOADING{}", ".".repeat(dots[0] as usize)));
        lines.push("".into());
        lines.push(format!("   [{}{}]", "#".repeat(n), " ".repeat(30 - n)));
    } else {
        let (q, _) = o.divmod(K(f), K(3));
        let (_, on) = o.divmod(K(q[0]), K(2));
        cursor = on[0] == 0;
    }
    let (tx, ty) = (91usize, 52usize);
    for (i, l) in lines.iter().enumerate() {
        for (j, ch) in l.chars().enumerate() { glyph(&mut img, w, cp437(ch), tx + 8 * j, ty + 16 * i); }
    }
    if cursor {
        let (cx, cy) = (tx + 8 * lines[lines.len() - 1].chars().count(), ty + 16 * (lines.len() - 1));
        for y in [cy + 13, cy + 14] { for x in cx..cx + 8 { img[(y * 640 + x) * 3..][..3].copy_from_slice(&[192, 192, 192]); } }
    }
    img
}

/// SEA++ in solid blocks two characters a pixel, "Unseen Planets presents" over it, the water line under it.
fn banner() -> Vec<String> {
    let g = |c: char| -> [&'static str; 5] { match c {
        'S' => [" ####", "#    ", " ### ", "    #", "#### "], 'E' => ["#####", "#    ", "#### ", "#    ", "#####"],
        'A' => [" ### ", "#   #", "#####", "#   #", "#   #"], _ => ["     ", "  #  ", " ### ", "  #  ", "     "] } };
    let mut out = vec!["Unseen Planets presents".to_string(), "".into()];
    for r in 0..5 {
        out.push("SEA++".chars().map(|c| g(c)[r].replace('#', "\u{2588}\u{2588}").replace(' ', "  ") + "  ").collect());
    }
    out.push("".into());
    out.push("\u{2591}\u{2592}\u{2593}\u{2592}".repeat(15));
    out
}

fn cp437(c: char) -> usize {
    match c { '\u{2588}' => 219, '\u{2591}' => 176, '\u{2592}' => 177, '\u{2593}' => 178, c => c as usize }
}

/// A glyph's cell (8 x 16) laid on the image (640 wide) at x, y, light grey where it's set.
fn glyph(img: &mut [u8], w: &World, code: usize, x: usize, y: usize) {
    for r in 0..16 { for b in 0..8 {
        if w.glyphs[code * 128 + r * 8 + b] != 0 { img[((y + r) * 640 + x + b) * 3..][..3].copy_from_slice(&[192, 192, 192]); }
    } }
}

/// The picture on the screen this frame, as an image (its width, height) and its fade table; and the mean colour.
struct Picture { img: Vec<u8>, iw: i64, ih: i64 }

/// Frame k of the rooms: RGB 1536 x 768.
pub fn render<O: Ops>(o: &mut O, w: &World, s: &Srcs, k: i64) -> Vec<u8> {
    let p = plan(o, w, k);
    let (sc, c) = zoom(o, w, &p);
    let screen = if p.hall { w.hall_screen } else { w.dorm_screen };
    // the picture the screen shows, as an image, and the room's light from it
    let fr = (W * H * 3) as usize;
    let (pic, mean) = match p.show {
        Show::Modex(f) => {
            // the 4:3 picture in modex's frame (columns 256..1279)
            let src = &s.modex[f as usize * fr..][..fr];
            let img: Vec<u8> = (0..768).flat_map(|y| src[(y * 1536 + 256) * 3..(y * 1536 + 1280) * 3].to_vec()).collect();
            let m = mean_of(o, w, &img, 1024, 0, 1024, 0, 768, 1000);
            (Picture { img, iw: 1024, ih: 768 }, m)
        }
        Show::Polys(f) => {
            // Dope's 2:1 frame letterboxed on the 4:3 screen: 192 x 144, its 96 rows from the 24th
            let src = &s.polys[f as usize * 192 * 96 * 3..][..192 * 96 * 3];
            let mut img = vec![0u8; 192 * 144 * 3];
            img[24 * 192 * 3..120 * 192 * 3].copy_from_slice(src);
            let m = mean_of(o, w, &img, 192, 0, 192, 24, 120, 667);
            (Picture { img, iw: 192, ih: 144 }, m)
        }
        Show::Drive(f) => {
            let mut img = s.drive[f as usize * fr..][..fr].to_vec();
            if DRIVE0 + f >= BRAG_AT { brag(o, &mut img, &s.brag, DRIVE0 + f - BRAG_AT); }
            // the last FADE_H frames fade to black
            let lv = along_at(o, f, -1, 1 - DRIVE_N, FADE_H);
            if lv < 1000 { let t = scale_tab(o, lv, 1000); for b in img.iter_mut() { *b = t.data[*b as usize] as u8; } }
            let m = mean_of(o, w, &img, 1536, 0, 1536, 0, 768, 1000);
            (Picture { img, iw: 1536, ih: 768 }, m)
        }
        Show::End(f) => {
            // THE END on black, fading in over 6 frames
            let a = along_at(o, f, 1, -1, 6);
            let a = one(o.lin(&[(K(a), Some(K(255)))]));
            let a = one(o.div(K(a), K(1000), 0));
            let t = scale_tab(o, a, 255);
            let img: Vec<u8> = w.end_cover.data.iter().flat_map(|&cv| { let b = t.data[cv as usize] as u8; [b, b, b] }).collect();
            let m = mean_of(o, w, &img, 1536, 0, 1536, 0, 768, 1000);
            (Picture { img, iw: 1536, ih: 768 }, m)
        }
        Show::Desk(f) => {
            // the desktop (640 x 480) pillarboxed on the 2:1 screen: 960 x 480, its columns from 160
            let d = desktop(o, w, f);
            let mut img = vec![0u8; 960 * 480 * 3];
            for y in 0..480 { img[(y * 960 + 160) * 3..(y * 960 + 800) * 3].copy_from_slice(&d[y * 640 * 3..(y + 1) * 640 * 3]); }
            let m = mean_of(o, w, &img, 960, 160, 800, 0, 480, 1000);
            (Picture { img, iw: 960, ih: 480 }, m)
        }
        Show::Dive(f) => {
            let img = s.dive[f as usize * fr..][..fr].to_vec();
            let m = mean_of(o, w, &img, 1536, 0, 1536, 0, 768, 1000);
            (Picture { img, iw: 1536, ih: 768 }, m)
        }
    };
    // the walks: the frame's columns and rows to the picture's (over the screen) and to each room picture's art
    let sw = one(o.sum(&[(K(screen[2]), 1), (K(screen[0]), -1)], 0));
    let sh = one(o.sum(&[(K(screen[3]), 1), (K(screen[1]), -1)], 0));
    let (pic_cols, _) = walk_to(o, W, sc, c[0], screen[0], To::Picture { pw: pic.iw, size: sw }, pic.iw);
    let (pic_rows, _) = walk_to(o, H, sc, c[1], screen[1], To::Picture { pw: pic.ih, size: sh }, pic.ih);
    let rooms: Vec<&Pic> = if p.hall { vec![&w.hall] } else { vec![&w.dorm, &w.dormin] };
    let mut layers = vec![];
    for r in &rooms {
        let (aw, ah) = (r.idx.w as i64, (r.idx.data.len() / r.idx.w) as i64);
        let (cols, ci) = walk_to(o, W, sc, c[0], r.bx[0], To::Art(r.art), aw);
        let (rows, ri) = walk_to(o, H, sc, c[1], r.bx[1], To::Art(r.art), ah);
        let lit = light_pal(o, w, &r.pal, mean, p.light);
        layers.push((cols, rows, ci, ri, lit));
    }
    // the pixels: the room's (the last room picture whose box holds it), or the picture's where the screen shows
    let mut out = vec![0u8; fr];
    for y in 0..H as usize {
        for x in 0..W as usize {
            let mut room = [0u8; 3];
            let mut on_screen = false;
            for (r, (cols, rows, ci, ri, lit)) in rooms.iter().zip(&layers) {
                if ci[x] != 1 || ri[y] != 1 { continue; }
                let (ax, ay) = (cols[x] as usize, rows[y] as usize);
                let i = r.idx.data[ay * r.idx.w + ax] as usize;
                room = [lit[0][i] as u8, lit[1][i] as u8, lit[2][i] as u8];
                on_screen = r.mask.data[ay * r.mask.w + ax] == 1;
            }
            let at = (y * W as usize + x) * 3;
            if on_screen {
                let q = ((pic_rows[y] * pic.iw + pic_cols[x]) * 3) as usize;
                out[at..at + 3].copy_from_slice(&pic.img[q..q + 3]);
            } else {
                out[at..at + 3].copy_from_slice(&room);
            }
        }
    }
    out
}

/// The brag's layer (its frame f, the last held) laid over the drive's frame at (40, 506): each covered pixel mixed
/// toward the layer's by its cover (32nds), as the cut's over line does.
fn brag<O: Ops>(o: &mut O, img: &mut [u8], layer: &[u8], f: i64) {
    let (lw, lh) = (441usize, 187usize);
    let size = lw * lh * 4;
    let frames = layer.len() / size;
    let f = (f as usize).min(frames - 1);
    let l = &layer[f * size..][..size];
    let mut at = vec![]; let (mut a, mut b, mut cv) = (vec![], vec![], vec![]);
    for y in 0..lh { for x in 0..lw {
        let p = &l[(y * lw + x) * 4..][..4];
        if p[3] == 0 { continue; }
        let q = ((506 + y) * 1536 + 40 + x) * 3;
        for c in 0..3 { at.push(q + c); a.push(img[q + c] as i64); b.push(p[c] as i64); cv.push(p[3] as i64); }
    } }
    if at.is_empty() { return; }
    // a + (b - a) c / 32: (a (32 - c) + b c) / 32
    let nc = o.sum(&[(v(&cv), -1)], 32);
    let x = o.lin(&[(v(&a), Some(v(&nc))), (v(&b), Some(v(&cv)))]);
    let x = o.div(v(&x), K(32), 0);
    for (j, &q) in at.iter().enumerate() { img[q] = x[j] as u8; }
}
