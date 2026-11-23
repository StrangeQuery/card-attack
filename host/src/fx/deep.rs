//! The dive, at 192x96: down a canyon of terraces in the spirit of Subnautica (from the prototype sea2/deep12.rs).
//! The seabed is a stored height map (segments/deep/data/map.txt), dealt to the machines a row at a time, and traced
//! per pixel through a camera that pitches, yaws and rolls; everything else is points and lines through the same
//! camera; light, fog and glow are worked out as colours, each channel exactly, 0..255.
//!
//! Each frame's numbers (the camera from the prototype's path: where it is, its forward, right and up vectors; the
//! light; phases; where the jellyfish, kelp, seagrass and fish are) are the script's (segments/deep/script.rs), worked
//! out on CARDIAC by the train's machines once, in the setup; the host carries cards from machine to machine and
//! files the frames. Per frame:
//! - per screen row, ray machines find each pixel's ray, its steps per slice of the map, its first step and its last
//!   slice;
//! - the march: a slice is a row of the map, a unit of z apart. Each screen column is marched from its bottom pixel
//!   up, slice after slice; a pixel whose ray is under the seabed (or too far, or above everything) is settled and the
//!   next pixel up starts at the same slice (walk machines put its ray there), since a pixel above hits no nearer.
//!   The slice machine is dealt the slice's map row and reads it as far as the ray needs;
//! - near (the first 30 slices) the seabed is blocks a unit square, two units a step: per hit, walk and block
//!   machines step back a slice and read the blocks about it; per screen row, machines find the face the ray went in
//!   by and where, the light on it (crevices, edges catching the light), the rock's strata, grain, rubble and coral.
//!   Further out the seabed is smooth: sample machines read it, others find where the ray crossed and the slope's
//!   light, and the same strata as a texture. Then how far it is and its fog, the diver's lamp, the caustics, the
//!   glowing spots and the lips' bands, and mix machines make the colour; open water gets its colour for how far up
//!   it looks, and the god rays;
//! - then, as the prototype draws them: the kelp, seagrass and fish (leaves rasterized row by row, painted over what is
//!   further), the marine snow, the jellyfish (halo, bell, tentacles);
//! - the channels the machines punched are the frame's colours as they are (no palette, no dithering).
//!
//! It files a second video, video_hd.rgb: the same frames at 1536x768 with the creatures drawn sharp over the water
//! (deep_hd.rs). DEEP_FRAMES=a,b,... renders and checks only those frames (DEEP_DUMP=dir writes their 1536x768
//! frames there).
use crate::*;
use super::deep_hd;
use std::path::Path;

pub const FRAMES: usize = 353;
/// The slices of the map near enough to be drawn as blocks; past them the seabed is smooth, textured with the strata.
const NEAR: i32 = 30;
/// The rock's tones in full light (four strata and coral) and the sand's.
const ROCKS: [[i32; 3]; 5] = [[124, 119, 130], [134, 116, 98], [108, 124, 112], [98, 106, 126], [230, 120, 100]];
const SAND: [i32; 3] = [202, 186, 148];
/// Specks of marine snow.
const SPECKS: usize = 450;
/// The map: per row (a unit of z) its base, its drop to the next row, how far past the nearest lip it is, 201
/// heights over the base (tenths), and its glowing spots (how many, then each one's unit along x and kind).
struct Map { base: Vec<i32>, drop: Vec<i32>, lip: Vec<i32>, rows: Vec<Vec<i32>>, spots: Vec<Vec<i32>> }

fn load_map(demo: &Path) -> Map {
    let (mut base, mut drop, mut lip, mut rows, mut spots) = (vec![], vec![], vec![], vec![], vec![]);
    for r in tables::rows(&read(demo, "deep/data/map.txt")) {
        base.push(r[0]); drop.push(r[1]); lip.push(r[2]); rows.push(r[3..204].to_vec()); spots.push(r[204..].to_vec());
    }
    Map { base, drop, lip, rows, spots }
}

#[allow(dead_code, clippy::all, unused_variables, unused_mut, unused_imports)]
pub mod whole {
    pub use super::super::train::imp::{fl, md, n_of, v, Native, Ops, Tab, A, CARD, K, LONG, V, WIDE};
    include!("../../../segments/deep/script.rs");
}


/// A jellyfish this frame: its place from the camera (tenths), how far its bell is turned (64ths of a turn), its kind
/// (0 blue, 1 violet), its bell's radius and height (hundredths, pulsing), its eight tentacles' lengths (tenths) and
/// its number (0..59, the same every frame).
#[derive(Clone)]
pub struct Jelly { pub at: [i32; 3], pub turn: i32, pub kind: i32, pub rad: i32, pub ht: i32, pub lengths: [i32; 8], pub id: i32 }

/// A clump of kelp this frame: three stalks from one root on the seabed. The root's place on the map (x as whole
/// units and hundredths, the map row), its place from the camera across and along (tenths); per stalk how far its
/// lean carries the top across (tenths), how tall it is (tenths) and its sway's phase (a step of 60); per blade on
/// it which way it points on the screen (a step of 64) and its length (50ths of a unit per pixel a unit makes).
#[derive(Clone)]
pub struct Clump { pub map_x: [i32; 2], pub row: i32, pub at: [i32; 2], pub stalks: Vec<Stalk> }
#[derive(Clone)]
pub struct Stalk { pub lean: i32, pub tall: i32, pub phase: i32, pub blades: Vec<[i32; 2]> }

/// A tuft of seagrass: its root on the map and from the camera (as a clump's), its sway's phase (a step of 60) and
/// its five blades' heights (tenths).
#[derive(Clone)]
pub struct Tuft { pub map_x: [i32; 2], pub row: i32, pub at: [i32; 2], pub phase: i32, pub heights: [i32; 5] }

/// A fish: its head and tail from the camera (tenths), how much it flashes silver (32nds) and its number (in its
/// school, the second school's from 100).
#[derive(Clone)]
pub struct Fish { pub head: [i32; 3], pub tail: [i32; 3], pub flash: i32, pub id: i32 }


/// One frame's numbers.
#[derive(Clone)]
pub struct Frame {
    pub white: bool,
    /// the camera: x in hundredths, y as whole units and hundredths, z as whole units and tenths
    pub x: i32, pub y: i32, pub y_part: i32, pub z: i32, pub z_part: i32,
    /// forward, right and up, 300 times the unit vectors
    pub f: [i32; 3], pub r: [i32; 3], pub u: [i32; 3],
    /// the light in 32nds and in tenths
    pub light: i32, pub light10: i32,
    /// the caustics' three phases (0..59)
    pub phase: [i32; 3],
    /// the god rays' two phases, hundredths of a step of 60
    pub shaft: [i32; 2],
    /// the glowing spots' pulse (a step 0..59) and how dark it is for them, 32 - 7 light / 10
    pub pulse: i32, pub dark: i32,
    /// forward, right and up in hundredths, for the things drawn over the seabed
    pub f100: [i32; 3], pub r100: [i32; 3], pub u100: [i32; 3],
    /// the camera's place in the snow's box (up and along, tenths, 0..139) and how much the light lets the snow show
    pub snow: [i32; 2], pub show: i32,
    /// how far the caption has faded in (0..16; -1 when it isn't up)
    pub caption: i32,
    /// the frame's number
    pub number: usize,
    /// the portal: how far it has come on (32nds; 0 before it shows), how far ahead its plane is (tenths), the camera's
    /// place from its middle across and up (hundredths), its swirl's and colour's phases (tenths of a step of 60), and
    /// how far everything has gone white (32nds)
    pub on: i32, pub plane: i32, pub from: [i32; 2], pub swirl: [i32; 2], pub white32: i32,
    /// the jellyfish: how much the light lets them glow (32nds; 0 before they come), their tentacles' wave this frame
    /// (a step of 60), and those near the camera, by number
    pub jelly_light: i32, pub wave: i32, pub jellies: Vec<Jelly>,
    /// the kelp (and whether its blades' tips glow, in the dark), the seagrass and the fish; the camera's height in
    /// tenths, for the roots on the map; how the light lets the fish shine (32nds)
    /// where the diver's torch points: its beam's middle from the screen's middle, half pixels right and up
    pub aim: [i32; 2],
    pub glow_tips: i32, pub kelp: Vec<Clump>, pub grass: Vec<Tuft>, pub fish: Vec<Fish>, pub height10: i32, pub fish_light: i32,
}

/// Each frame's numbers: the dive's script (segments/deep/script.rs) on CARDIAC, by the train's machines, its
/// lines read back as frames.
pub fn script(demo: &Path) -> Vec<Frame> {
    let d = whole::ScriptData::parse(&read(demo, "deep/data/script.txt"));
    let mut o = super::train::Cardiac::new(demo);
    let lines = whole::script(&mut o, &d);
    live::count(o.host.steps);
    lines.iter().enumerate().map(|(n, l)| frame_of(n, &l.iter().map(|&v| v as i32).collect::<Vec<_>>())).collect()
}

/// A frame from its line of the script (as the reference reads it).
fn frame_of(number: usize, v: &[i32]) -> Frame {
    let mut i = 0;
    let mut take = |n: usize| { let s = v[i..i + n].to_vec(); i += n; s };
    let h = take(44);
    let a3 = |s: &[i32]| [s[0], s[1], s[2]];
    let j = take(3);
    let jellies = (0..j[2]).map(|_| { let s = take(16); Jelly { at: a3(&s), turn: s[3], kind: s[4], rad: s[5], ht: s[6],
        lengths: [s[7], s[8], s[9], s[10], s[11], s[12], s[13], s[14]], id: s[15] } }).collect();
    let g = take(4);
    let kelp = (0..g[3]).map(|_| { let s = take(5); Clump { map_x: [s[0], s[1]], row: s[2], at: [s[3], s[4]],
        stalks: (0..3).map(|_| { let t = take(33); Stalk { lean: t[0], tall: t[1], phase: t[2], blades: t[3..].chunks(2).map(|b| [b[0], b[1]]).collect() } }).collect() } }).collect();
    let n = take(1)[0];
    let grass = (0..n).map(|_| { let s = take(11); Tuft { map_x: [s[0], s[1]], row: s[2], at: [s[3], s[4]], phase: s[5], heights: [s[6], s[7], s[8], s[9], s[10]] } }).collect();
    let n = take(1)[0];
    let fish = (0..n).map(|_| { let s = take(8); Fish { head: a3(&s), tail: a3(&s[3..]), flash: s[6], id: s[7] } }).collect();
    let aim = take(2);
    assert_eq!(i, v.len(), "frame {number}'s line");
    Frame { white: h[0] == 1, x: h[1], y: h[2], y_part: h[3], z: h[4], z_part: h[5], f: a3(&h[6..]), r: a3(&h[9..]), u: a3(&h[12..]),
        light: h[15], light10: h[16], phase: a3(&h[17..]), shaft: [h[20], h[21]], pulse: h[22], dark: h[23],
        f100: a3(&h[24..]), r100: a3(&h[27..]), u100: a3(&h[30..]), snow: [h[33], h[34]], show: h[35], caption: h[36], number,
        on: h[37], plane: h[38], from: [h[39], h[40]], swirl: [h[41], h[42]], white32: h[43],
        jelly_light: j[0], wave: j[1], jellies, aim: [aim[0], aim[1]], glow_tips: g[0], kelp, grass, fish, height10: g[1], fish_light: g[2] }
}

struct Machines { rays: Machine, slope: Machine, tenth: Machine, limit: Machine, slice: Machine, sample: Machine, cross: Machine,
    face: Machine, far: Machine, phase: Machine, phase2: Machine, sine: Machine, glint: Machine, mix: Machine, water: Machine,
    norm: Machine, root: Machine, shaft: Machine, wave: Machine, pow: Machine, ray: Machine,
    spot: Machine, fall: Machine, glow: Machine, lip: Machine, dot: Machine, cull: Machine, seed: Machine, snow: Machine, speck: Machine, caption: Machine,
    walks: Machine, neg: Machine, twice: Machine, tenths: Machine, behind: Machine, order: Machine, ratio: Machine, look: Machine, scale: Machine, turn: Machine, swirl: Machine,
    inner: Machine, ring: Machine, spill: Machine, spill_mix: Machine,
    jfade: Machine, fine: Machine, bx: Machine, lin: Machine, gauss: Machine, shade: Machine, part: Machine, add: Machine, cos: Machine,
    edge: Machine, bell: Machine, span: Machine, inside: Machine, wiggle: Machine, amp: Machine, hang: Machine,
    steps: Machine, step: Machine, pix: Machine,
    base: Machine, index: Machine, sway: Machine, swing: Machine, sum: Machine, leafv: Machine, bbox: Machine, reach: Machine, strip: Machine,
    spanrow: Machine, cap: Machine, width: Machine, wide: Machine, lit: Machine, along: Machine, put: Machine, tip: Machine,
    blocks: Machine, column: Machine, descend: Machine, side: Machine, pick: Machine, top: Machine, corner: Machine, riser: Machine,
    crest: Machine, place: Machine, strata: Machine, far2: Machine, grain: Machine, coral: Machine, rocky: Machine, lamp: Machine, beam: Machine }

/// A pixel's ray: its steps per slice across and up (hundredths), its forward part, first steps and last slice.
#[derive(Clone, Copy, Default)]
struct Ray { qx: i32, qy: i32, nqx: i32, nqy: i32, dz: i32, fx: i32, fy: i32, last: i32 }

/// A settled pixel: water, or a hit at slice k with the ray's x and height there (whole, hundredths).
#[derive(Clone, Copy)]
enum Settled { Water, Hit { k: i32, x: [i32; 2], y: [i32; 2] } }

impl Machines {
    /// a + (b - a) k / 32 for each triple, on the mix machine.
    fn mix(&self, host: &mut Host, triples: &[[i32; 3]]) -> Vec<i32> {
        let flat: Vec<i32> = triples.iter().flatten().copied().collect();
        many(host, &self.mix, &[], &flat, 3, 1)
    }
}

/// The machines, by name.
fn machines(demo: &Path) -> Machines {
    let r = |n: &str| rust(demo, n);
    let quarter: Vec<i32> = demokit::tables::QUARTER_SINE60.iter().map(|&v| v as i32).collect();
    let sine = carrying(demo, "dsine", "sine", &quarter);
    let wave = carrying(demo, "dwave", "sine", &quarter);
    Machines { rays: r("drays"), slope: r("dslope"), tenth: r("dtenth"), limit: r("dlimit"), slice: r("dslice"), sample: r("dsample"),
        cross: r("dcross"), face: r("uface"), far: r("dfar"), phase: r("dphase"), phase2: r("dphase2"), sine, glint: r("dglint"), mix: r("umix"),
        water: r("dwater"), norm: r("dnorm"), root: r("droot"),
        shaft: r("dshaft"), wave, pow: r("dpow"), ray: r("uray"), spot: r("dspot"), fall: r("dfall"), glow: r("uglow"), lip: r("dlip"),
        dot: r("ddot"), cull: r("dcull"), seed: r("dseed"), snow: r("dsnow"), speck: r("dspeck"), caption: r("dcaption"),
        walks: r("dwalks"), neg: r("dneg"), twice: r("dtwice"), tenths: r("dtenths"), behind: r("dbehind"), order: r("dorder"), ratio: r("dratio"), look: r("dlook"), scale: r("dscale"),
        turn: r("uturn"), swirl: r("dswirl"), inner: r("dinner"), ring: r("dring"), spill: r("dspill"), spill_mix: r("dspillmix"),
        jfade: r("djfade"), fine: r("dfine"), bx: r("dbox"), lin: r("dlin"), gauss: r("dgauss"), shade: r("ushade"), part: r("dpart"), add: r("dadd"),
        cos: r("dcos"), edge: r("uedge"), bell: r("dbell"), span: r("uspan"), inside: r("dinside"), wiggle: r("dwiggle"),
        amp: r("damp"), hang: r("dhang"), steps: r("dsteps"), step: r("ustep"), pix: r("dpix"),
        base: r("dbase"), index: r("dindex"), sway: r("dsway"), swing: r("dswing"), sum: r("dsum"), leafv: r("dleafv"), bbox: r("dbbox"),
        reach: r("ureach"), strip: r("dstrip"), spanrow: r("dspanrow"), cap: r("ucap"), width: r("dwidth"), wide: r("dwide"), lit: r("ulit"),
        along: r("ualong"), put: r("dput"), tip: r("dtip"),
        blocks: r("ublocks"), column: r("ucolumn"), descend: r("udescend"), side: r("uside"), pick: r("upick"), top: r("utop"),
        corner: r("ucorner"), riser: r("uriser"), crest: r("ucrest"), place: r("uplace"), strata: r("ustrata"), far2: r("ufar"),
        grain: r("ugrain"), coral: r("ucoral"), rocky: r("urocky"), lamp: r("ulamp"), beam: r("ubeam") }
}

/// What every frame takes, made once: each pixel's ray length, the marine snow's specks, and (for the creatures at
/// 1536x768) the first frame each jellyfish shows: the first with it near enough and its fade more than 0 (as the
/// dive draws it), by its number (-1 never).
fn once(host: &mut Host, m: &Machines, sc: &[Frame]) -> (Vec<i32>, Vec<i32>, Vec<i32>) {
    let (w, h) = (W2, H2);
    let norm: Vec<i32> = (0..w * h).map(|i| {
        let big = host.go1(&m.norm, &[&[(i % w) as i32, (i / w) as i32]], 2);
        host.go1(&m.root, &[&big], 1)[0]
    }).collect();
    let specks = host.go1(&m.seed, &[&[SPECKS as i32, 1]], 3 * SPECKS);
    let mut first = vec![-1; 60];
    for (n, fr) in sc.iter().enumerate() {
        if fr.white || fr.jellies.is_empty() || fr.jelly_light == 0 { continue; }
        let centres: Vec<i32> = fr.jellies.iter().flat_map(|j| j.at).collect();
        let ahead = many(host, &m.dot, &fr.f100, &centres, 3, 1);
        let fades = many(host, &m.jfade, &[fr.jelly_light], &ahead, 1, 1);
        for (e, j) in fr.jellies.iter().enumerate() { if fades[e] > 0 && first[j.id as usize] < 0 { first[j.id as usize] = n as i32; } }
    }
    (norm, specks, first)
}

/// The player's job before the deep's blocks: the script and `once`, kept for them (live::keep).
pub fn setup(demo: &Path) {
    let mut host = Host::new();
    let sc = script(demo);
    let kept = once(&mut host, &machines(demo), &sc);
    live::count(host.steps);
    live::keep("deep", kept);
    live::keep("deep_script", sc);
}

pub fn render(demo: &Path, refbin: &str) {
    // the script (by the player, once for all the blocks: see setup)
    let sc = match live::kept::<Vec<Frame>>("deep_script") { Some(s) => (*s).clone(), None => script(demo) };
    let map = load_map(demo);
    let m = machines(demo);
    let mut host = Host::new();
    let (w, h) = (W2, H2);
    // once (by the player, once for all the blocks it split the deep into: see setup)
    let kept = live::kept::<(Vec<i32>, Vec<i32>, Vec<i32>)>("deep");
    let (norm, specks, first) = match kept.as_deref() { Some((n, s, f)) => (n.clone(), s.clone(), f.clone()), None => once(&mut host, &m, &sc) };
    // the caption set in the 3 x 5 font: 7 rows of 69 dots
    let caption = caption_dots(demo);
    // the creatures at 1536x768, a host of their own so each video says what it took
    let hd = Hd { m: deep_hd::machines(demo), d: deep_hd::data(demo), first };
    let mut host_hd = Host::new();
    // DEEP_FRAMES=a,b,...: only those frames, each checked against the reference's (for working on it)
    if let Ok(list) = std::env::var("DEEP_FRAMES") {
        let both = reference(refbin, &[&demo.join("deep/data/map.txt"), &demo.join("deep/data/script.txt"), &demo.join("deep/data/font3x5.txt"),
            &demo.join("deep/data/hd.txt"), &demo.join("swarm/data/tables.txt")]);
        for fi in list.split(',').map(|v| v.trim().parse::<usize>().unwrap()) {
            let fr = &sc[fi];
            if fr.white { continue; }
            let (b0, h0) = (host.steps, host_hd.steps);
            let (rgb, big) = frame(&mut host, &mut host_hd, &m, &map, fr, &norm, &specks, &caption, Some(&hd));
            let big = big.unwrap();
            let lo: Vec<u8> = rgb.iter().flat_map(|c| c.map(|v| v as u8)).collect();
            let want_lo = &both[fi * FRAME2..(fi + 1) * FRAME2];
            let want = &both[FRAMES * FRAME2 + fi * HD_FRAME..FRAMES * FRAME2 + (fi + 1) * HD_FRAME];
            let bad = big.iter().zip(want).position(|(a, b)| a != b);
            let diff = big.iter().zip(want).filter(|(a, b)| a != b).count();
            println!("frame {fi}: 192x96 {}, 1536x768 {} ({diff} channels differ{}); {}M + {}M instructions", if lo == want_lo { "same" } else { "DIFFERS" },
                if bad.is_none() { "same" } else { "DIFFERS" }, bad.map(|i| format!(", first at pixel {} {}: {} for {}", i / 3 % deep_hd::HW, i / 3 / deep_hd::HW, big[i], want[i])).unwrap_or_default(),
                (host.steps - b0) / 1_000_000, (host_hd.steps - h0) / 1_000_000);
            if let Ok(dir) = std::env::var("DEEP_DUMP") { std::fs::write(format!("{dir}/hd_{fi}.rgb"), &big).unwrap(); }
        }
        return;
    }
    let mut video = Video2::new(FRAMES);
    let frames = live::range(FRAMES);
    let mut hd_frames: Vec<u8> = Vec::with_capacity(frames.len() * HD_FRAME);
    let (mut per_frame, mut per_hd) = (0, 0);
    for fi in live::frames(FRAMES) {
        let fr = &sc[fi];
        let (before, before_hd) = (host.steps, host_hd.steps);
        // every channel as the machines punched it (0..255; the host stops if not)
        let (rgb, big) = if fr.white { (vec![[255; 3]; w * h], vec![255u8; HD_FRAME]) } else {
            // the dive's machines on its own host, the creatures' on theirs
            let (rgb, big) = frame(&mut host, &mut host_hd, &m, &map, fr, &norm, &specks, &caption, Some(&hd));
            (rgb, big.unwrap())
        };
        for (i, c) in rgb.iter().enumerate() {
            for (k, &v) in c.iter().enumerate() {
                assert!((0..=255).contains(&v), "deep: a channel out of 0..255");
                video.buf[(fi * H2 * W2 + i) * 3 + k] = v as u8;
            }
        }
        hd_frames.extend_from_slice(&big);
        per_frame = per_frame.max(host.steps - before);
        per_hd = per_hd.max(host_hd.steps - before_hd);
    }
    host.done();
    host_hd.done();
    let both = reference(refbin, &[&demo.join("deep/data/map.txt"), &demo.join("deep/data/script.txt"), &demo.join("deep/data/font3x5.txt"),
        &demo.join("deep/data/hd.txt"), &demo.join("swarm/data/tables.txt")]);
    let lo_end = FRAMES * FRAME2;
    let part = |a: usize, b: usize| &both[a.min(both.len())..b.min(both.len())];
    finish("deep", &host, &video, part(0, lo_end), &demo.join("deep/video.rgb"), &format!(", up to {}M instructions a frame", per_frame / 1_000_000));
    // the creatures at 1536x768: live, only this block's frames, as a part
    let path = demo.join("deep/video_hd.rgb");
    if live::on() {
        live::count(host_hd.steps);
        match live::block() { Some((a, _)) => live::put_part(&path, a.min(FRAMES), HD_FRAME, hd_frames), None => live::put(&path, hd_frames) }
        return;
    }
    let mut video_hd = deep_hd::VideoHd::new(FRAMES);
    video_hd.buf = hd_frames;
    finish("deep (the creatures, 1536x768)", &host_hd, &video_hd, part(lo_end, both.len()), &path, &format!(", up to {}M instructions a frame", per_hd / 1_000_000));
}

/// A frame of the creatures at 1536x768: RGB.
const HD_FRAME: usize = deep_hd::HW * deep_hd::HH * 3;


/// Points seen from the camera (places in tenths, three cards each): for each, Some(pixel) and how far ahead
/// (tenths), or None.
fn project(host: &mut Host, m: &Machines, fr: &Frame, points: &[i32]) -> Vec<Option<(i32, i32, i32)>> {
    // a deck says how many points it holds on one card: at most 999, so 300 at a time
    if points.len() > 900 { return points.chunks(900).flat_map(|c| project(host, m, fr, c)).collect(); }
    let n = points.len() / 3;
    let along = |host: &mut Host, v: &[i32; 3]| host.go1(&m.dot, &[v, &[n as i32], points], n);
    let (ahead, across, up) = (along(host, &fr.f100), along(host, &fr.r100), along(host, &fr.u100));
    let deck: Vec<i32> = (0..n).flat_map(|i| [ahead[i], across[i], up[i]]).collect();
    let seen = host.go1(&m.cull, &[&[n as i32], &deck], n);
    let shown: Vec<usize> = (0..n).filter(|&i| seen[i] == 1).collect();
    let mut out = vec![None; n];
    if shown.is_empty() { return out; }
    let k = shown.len() as i32;
    let deck: Vec<i32> = shown.iter().flat_map(|&i| [across[i], ahead[i]]).collect();
    let sx = host.go1(&m.slope, &[&[2, k], &deck], shown.len());
    let deck: Vec<i32> = shown.iter().flat_map(|&i| [up[i], ahead[i]]).collect();
    let sy = host.go1(&m.slope, &[&[2, k], &deck], shown.len());
    for (e, &i) in shown.iter().enumerate() { out[i] = Some((96 + sx[e], 48 - sy[e], ahead[i])); }
    out
}

/// Mix one colour into pixels by amounts (32nds), in order: where a pixel comes again, what came before is mixed
/// first (the mix machine takes a batch with each pixel once).
fn blend(host: &mut Host, m: &Machines, rgb: &mut [[i32; 3]], items: &[(usize, [i32; 3], i32)]) { blend_by(host, &m.mix, &[], rgb, items) }

/// The same with another mix machine and the cards it takes first (the spill's, in 256ths, and the portal's on).
fn blend_by(host: &mut Host, mix: &Machine, head: &[i32], rgb: &mut [[i32; 3]], items: &[(usize, [i32; 3], i32)]) {
    let mut start = 0;
    while start < items.len() {
        let mut seen = std::collections::HashSet::new();
        let mut end = start;
        while end < items.len() && end - start < 300 && seen.insert(items[end].0) { end += 1; }
        let batch = &items[start..end];
        for c in 0..3 {
            let t: Vec<[i32; 3]> = batch.iter().map(|&(i, col, k)| [rgb[i][c], col[c], k]).collect();
            let flat: Vec<i32> = t.iter().flatten().copied().collect();
            for (e, v) in many(host, mix, head, &flat, 3, 1).into_iter().enumerate() { rgb[batch[e].0][c] = v; }
        }
        start = end;
    }
}

/// A machine over a list of items, each `width` cards, punching `each` cards per item: in decks of at most 999 items
/// (a deck says how many it holds on one card), with `head` cards before the count.
fn many(host: &mut Host, machine: &Machine, head: &[i32], items: &[i32], width: usize, each: usize) -> Vec<i32> {
    let mut out = vec![];
    for chunk in items.chunks(300 * width) {
        let mut deck = head.to_vec();
        deck.push((chunk.len() / width) as i32);
        deck.extend_from_slice(chunk);
        out.extend(host.go1(machine, &[&deck], chunk.len() / width * each));
    }
    out
}

/// Pairs zipped into one list.
fn zip2(a: &[i32], b: &[i32]) -> Vec<i32> { a.iter().zip(b).flat_map(|(&x, &y)| [x, y]).collect() }

/// The places of points from the camera, seen to a tenth of a pixel: for each, Some((across, up from the middle in
/// tenths of a pixel, how far ahead in tenths)) when it is within 100 pixels of the middle, else None.
fn see_fine(host: &mut Host, m: &Machines, fr: &Frame, points: &[i32]) -> Vec<Option<(i32, i32, i32)>> {
    let n = points.len() / 3;
    let ahead = many(host, &m.dot, &fr.f100, points, 3, 1);
    let across = many(host, &m.dot, &fr.r100, points, 3, 1);
    let up = many(host, &m.dot, &fr.u100, points, 3, 1);
    let deck: Vec<i32> = (0..n).flat_map(|i| [ahead[i], across[i], up[i]]).collect();
    let seen = many(host, &m.fine, &[], &deck, 3, 1);
    let shown: Vec<usize> = (0..n).filter(|&i| seen[i] == 1).collect();
    let mut out = vec![None; n];
    let sx = many(host, &m.slope, &[3], &shown.iter().flat_map(|&i| [across[i], ahead[i]]).collect::<Vec<_>>(), 2, 1);
    let sy = many(host, &m.slope, &[3], &shown.iter().flat_map(|&i| [up[i], ahead[i]]).collect::<Vec<_>>(), 2, 1);
    for (e, &i) in shown.iter().enumerate() { out[i] = Some((sx[e], sy[e], ahead[i])); }
    out
}

/// How long each pair (a, b) is: the order machine takes the larger and smaller part, the ratio machine 32 times
/// their ratio, and the scale machine adds the radius table's share of the larger.
fn magnitude(host: &mut Host, m: &Machines, radius: &[i32], pairs: &[i32]) -> Vec<i32> {
    let n = pairs.len() / 2;
    let ord = many(host, &m.order, &[], pairs, 2, 5);
    let ratio = many(host, &m.ratio, &[], &(0..n).flat_map(|i| [ord[5 * i], ord[5 * i + 1]]).collect::<Vec<_>>(), 2, 1);
    let mut head = vec![radius.len() as i32];
    head.extend(radius);
    let share = many(host, &m.look, &head, &ratio, 1, 1);
    many(host, &m.scale, &[], &(0..n).flat_map(|i| [ord[5 * i], share[i]]).collect::<Vec<_>>(), 2, 1)
}

/// The jellyfish, far ones first: each a halo round its middle, a bell of 12 rings of 32 points (brighter at the
/// edge, seen side on, and along its canals; dimmer seen from inside) and eight waving tentacles, all through the
/// camera, each point mixing the jellyfish's glow into the pixels it covers where the seabed is not nearer.
fn jellies(host: &mut Host, m: &Machines, fr: &Frame, rgb: &mut [[i32; 3]], depth: &[i32]) -> Vec<(i32, i32, usize)> {
    if fr.jellies.is_empty() || fr.jelly_light == 0 { return vec![]; }
    let (w, h) = (W2 as i32, H2 as i32);
    let tab = |t: &[i16]| -> Vec<i32> { t.iter().map(|&v| v as i32).collect() };
    let (bell_sin, bell_cos, radius, halo) = (tab(&demokit::tables::BELL_SIN), tab(&demokit::tables::BELL_COS), tab(&demokit::tables::PORTAL_RADIUS), tab(&demokit::tables::JELLY_HALO));
    // printed tables: each tentacle's phase (19 tentacle / 2 steps of 60) and each step down's strength, (15 - k) 29 / 15
    let phases: Vec<i32> = (0..8).map(|t| 19 * t / 2).collect();
    let strength: Vec<i32> = (0..13).map(|k| (15 - k) * 29 / 15).collect();
    let centres: Vec<i32> = fr.jellies.iter().flat_map(|j| j.at).collect();
    let ahead = many(host, &m.dot, &fr.f100, &centres, 3, 1);
    let fades = many(host, &m.jfade, &[fr.jelly_light], &ahead, 1, 1);
    let mut order: Vec<usize> = (0..fr.jellies.len()).filter(|&e| fades[e] > 0).collect();
    order.sort_by(|&a, &b| ahead[b].cmp(&ahead[a]));
    let drawn: Vec<(i32, i32, usize)> = order.iter().map(|&e| (ahead[e], fades[e], e)).collect();
    let pixel = |c: i32, r: i32| -> Option<usize> { let (x, y) = (c + 96, r + 48); (x >= 0 && y >= 0 && x < w && y < h).then(|| (y * w + x) as usize) };
    for e in order {
        let jelly = &fr.jellies[e];
        let (ahead, fade, c) = (ahead[e], fades[e], jelly.at);
        let glow = if jelly.kind == 1 { [220, 150, 255] } else { [140, 200, 255] };
        let centre = see_fine(host, m, fr, &c)[0];
        // the halo
        if let Some((cx, cy, _)) = centre {
            let sizes = many(host, &m.slope, &[2], &[140, ahead, 30, ahead], 2, 1);
            let b = host.go1(&m.bx, &[&[cx, cy, sizes[0], sizes[1]]], 5);
            let sigma = b[0];
            for row in b[3].max(-48)..b[4].min(48) {
                let cols: Vec<i32> = (b[1].max(-96)..b[2].min(96)).collect();
                if cols.is_empty() { continue; }
                let dy = many(host, &m.lin, &[cy, 1], &[row], 1, 1)[0];
                let v = many(host, &m.slope, &[2], &[dy, sigma], 2, 1)[0];
                let dx = many(host, &m.lin, &[cx, 0], &cols, 1, 1);
                let u = many(host, &m.slope, &[2], &dx.iter().flat_map(|&d| [d, sigma]).collect::<Vec<_>>(), 2, 1);
                let q = many(host, &m.gauss, &[v], &u, 1, 1);
                let mut head = vec![halo.len() as i32];
                head.extend(&halo);
                let fall = many(host, &m.look, &head, &q, 1, 1);
                let deck: Vec<i32> = cols.iter().enumerate().flat_map(|(k, &col)| [fall[k], fade, ahead, depth[pixel(col, row).unwrap()]]).collect();
                let amount = many(host, &m.shade, &[32], &deck, 4, 1);
                let items: Vec<(usize, [i32; 3], i32)> = cols.iter().enumerate().map(|(k, &col)| (pixel(col, row).unwrap(), glow, amount[k])).collect();
                blend(host, m, rgb, &items);
            }
        }
        // which way the camera is from it, over its length: across with along, then with up
        let level = magnitude(host, m, &radius, &[c[0], c[2]])[0];
        let length = magnitude(host, m, &radius, &[level, c[1]])[0];
        let view = many(host, &m.slope, &[2], &c.iter().flat_map(|&v| [v, length]).collect::<Vec<_>>(), 2, 1);
        // the rings: their radius and height, and their surface's direction
        let rings: Vec<usize> = (0..12).collect();
        let rs = many(host, &m.part, &[0], &rings.iter().flat_map(|&i| [jelly.rad, bell_sin[i]]).collect::<Vec<_>>(), 2, 1);
        let hc = many(host, &m.part, &[0], &rings.iter().flat_map(|&i| [jelly.ht, bell_cos[i]]).collect::<Vec<_>>(), 2, 1);
        let hc10 = many(host, &m.part, &[0], &hc.iter().flat_map(|&v| [v, 10]).collect::<Vec<_>>(), 2, 1);
        let nx = bell_sin.clone();
        let xc = many(host, &m.part, &[0], &rings.iter().flat_map(|&i| [jelly.rad, bell_cos[i]]).collect::<Vec<_>>(), 2, 1);
        let ny = many(host, &m.slope, &[2], &xc.iter().flat_map(|&v| [v, jelly.ht]).collect::<Vec<_>>(), 2, 1);
        let lengths = magnitude(host, m, &radius, &zip2(&nx, &ny));
        let na = many(host, &m.slope, &[2], &zip2(&nx, &lengths), 2, 1);
        let nb = many(host, &m.slope, &[2], &zip2(&ny, &lengths), 2, 1);
        // the bell's points
        let cs = many(host, &m.cos, &[jelly.turn], &(0..32).map(|s| 2 * s).collect::<Vec<_>>(), 1, 2);
        let pts: Vec<(usize, usize)> = (0..12).flat_map(|i| (0..32).map(move |s| (i, s))).collect();
        let ox = many(host, &m.part, &[1], &pts.iter().flat_map(|&(i, s)| [rs[i], cs[2 * s]]).collect::<Vec<_>>(), 2, 1);
        let oz = many(host, &m.part, &[1], &pts.iter().flat_map(|&(i, s)| [rs[i], cs[2 * s + 1]]).collect::<Vec<_>>(), 2, 1);
        let px = many(host, &m.add, &[0], &ox.iter().flat_map(|&o| [c[0], o]).collect::<Vec<_>>(), 2, 1);
        let py = many(host, &m.add, &[0], &pts.iter().flat_map(|&(i, _)| [c[1], hc10[i]]).collect::<Vec<_>>(), 2, 1);
        let pz = many(host, &m.add, &[0], &oz.iter().flat_map(|&o| [c[2], o]).collect::<Vec<_>>(), 2, 1);
        let s1 = many(host, &m.part, &[0], &pts.iter().flat_map(|&(_, s)| [cs[2 * s], view[0]]).collect::<Vec<_>>(), 2, 1);
        let s2 = many(host, &m.part, &[0], &pts.iter().flat_map(|&(_, s)| [cs[2 * s + 1], view[2]]).collect::<Vec<_>>(), 2, 1);
        let side = many(host, &m.add, &[0], &zip2(&s1, &s2), 2, 1);
        let f1 = many(host, &m.part, &[0], &pts.iter().zip(&side).flat_map(|(&(i, _), &sd)| [sd, na[i]]).collect::<Vec<_>>(), 2, 1);
        let f2 = many(host, &m.part, &[0], &pts.iter().flat_map(|&(i, _)| [view[1], nb[i]]).collect::<Vec<_>>(), 2, 1);
        let facing = many(host, &m.add, &[0], &zip2(&f1, &f2), 2, 1);
        let edge = many(host, &m.edge, &[], &facing, 1, 2);
        let deck: Vec<i32> = pts.iter().enumerate().flat_map(|(k, &(_, s))| [edge[2 * k], (s % 4 == 0) as i32, edge[2 * k + 1]]).collect();
        let lit = many(host, &m.bell, &[], &deck, 3, 2);
        let places: Vec<i32> = (0..pts.len()).flat_map(|k| [px[k], py[k], pz[k]]).collect();
        let seen = see_fine(host, m, fr, &places);
        let shown: Vec<usize> = (0..pts.len()).filter(|&k| seen[k].is_some()).collect();
        let rads = many(host, &m.slope, &[2], &shown.iter().flat_map(|&k| [9, seen[k].unwrap().2]).collect::<Vec<_>>(), 2, 1);
        let deck: Vec<i32> = shown.iter().enumerate().flat_map(|(e, &k)| { let (sx, sy, _) = seen[k].unwrap(); [sx, sy, rads[e]] }).collect();
        let across = many(host, &m.span, &[0], &deck, 3, 3);
        let down = many(host, &m.span, &[1], &deck, 3, 3);
        let spans: Vec<i32> = (0..shown.len()).flat_map(|e| [across[3 * e], across[3 * e + 1], across[3 * e + 2], down[3 * e + 1], down[3 * e + 2]]).collect();
        let mut items = vec![];
        for (e, &k) in shown.iter().enumerate() {
            let (sx, sy, pd) = seen[k].unwrap();
            let sp = &spans[5 * e..5 * e + 5];
            let cells: Vec<(i32, i32)> = (sp[3]..=sp[4]).flat_map(|r| (sp[1]..=sp[2]).map(move |c| (c, r))).filter(|&(c, r)| pixel(c, r).is_some()).collect();
            if cells.is_empty() { continue; }
            let deck: Vec<i32> = cells.iter().flat_map(|&(c, r)| [c, r]).collect();
            let inside = many(host, &m.inside, &[sx, sy, sp[0]], &deck, 2, 1);
            let colour: Vec<i32> = (0..3).map(|ch| m.mix(host, &[[glow[ch], [230, 245, 255][ch], lit[2 * k]]])[0]).collect();
            let covered: Vec<usize> = cells.iter().zip(&inside).filter(|(_, &i)| i == 1).map(|(&(c, r), _)| pixel(c, r).unwrap()).collect();
            if covered.is_empty() { continue; }
            let deck: Vec<i32> = covered.iter().flat_map(|&i| [lit[2 * k + 1], fade, pd, depth[i]]).collect();
            let amount = many(host, &m.shade, &[29], &deck, 4, 1);
            for (j, &i) in covered.iter().enumerate() { items.push((i, [colour[0], colour[1], colour[2]], amount[j])); }
        }
        blend(host, m, rgb, &items);
        // the tentacles
        let rr = host.go1(&m.part, &[&[0, 1, jelly.rad, 90]], 1)[0];
        let cs = many(host, &m.cos, &[jelly.turn], &(0..8).map(|t| 8 * t).collect::<Vec<_>>(), 1, 2);
        let tps: Vec<(usize, i32)> = (0..8).flat_map(|t| (0..=12).map(move |s| (t, s))).collect();
        let steps = many(host, &m.wiggle, &[fr.wave], &tps.iter().flat_map(|&(t, s)| [phases[t], s]).collect::<Vec<_>>(), 2, 1);
        let wave = many(host, &m.wave, &[], &steps, 1, 1);
        let wv = many(host, &m.amp, &[], &tps.iter().zip(&wave).flat_map(|(&(_, s), &v)| [v, s]).collect::<Vec<_>>(), 2, 1);
        let a1 = many(host, &m.part, &[1], &tps.iter().flat_map(|&(t, _)| [rr, cs[2 * t]]).collect::<Vec<_>>(), 2, 1);
        let a2 = many(host, &m.part, &[0], &tps.iter().zip(&wv).flat_map(|(&(t, _), &v)| [cs[2 * t + 1], v]).collect::<Vec<_>>(), 2, 1);
        let b1 = many(host, &m.part, &[1], &tps.iter().flat_map(|&(t, _)| [rr, cs[2 * t + 1]]).collect::<Vec<_>>(), 2, 1);
        let b2 = many(host, &m.part, &[0], &tps.iter().zip(&wv).flat_map(|(&(t, _), &v)| [cs[2 * t], v]).collect::<Vec<_>>(), 2, 1);
        let ox = many(host, &m.add, &[0], &zip2(&a1, &a2), 2, 1);
        let oz = many(host, &m.add, &[1], &zip2(&b1, &b2), 2, 1);
        let px = many(host, &m.add, &[0], &ox.iter().flat_map(|&o| [c[0], o]).collect::<Vec<_>>(), 2, 1);
        let py = many(host, &m.hang, &[c[1]], &tps.iter().flat_map(|&(t, s)| [s, jelly.lengths[t]]).collect::<Vec<_>>(), 2, 1);
        let pz = many(host, &m.add, &[0], &oz.iter().flat_map(|&o| [c[2], o]).collect::<Vec<_>>(), 2, 1);
        let places: Vec<i32> = (0..tps.len()).flat_map(|k| [px[k], py[k], pz[k]]).collect();
        let seen = see_fine(host, m, fr, &places);
        let mut items = vec![];
        for t in 0..8 {
            for s in 1..=12usize {
                let (Some(a), Some(b)) = (seen[t * 13 + s - 1], seen[t * 13 + s]) else { continue };
                let n = host.go1(&m.steps, &[&[1, a.0, a.1, b.0, b.1]], 1)[0];
                let xs = host.go1(&m.step, &[&[1, a.0, b.0, n]], n as usize + 1);
                let ys = host.go1(&m.step, &[&[1, a.1, b.1, n]], n as usize + 1);
                let ds = host.go1(&m.step, &[&[1, a.2, b.2, n]], n as usize + 1);
                let cols = many(host, &m.pix, &[1], &xs, 1, 1);
                let rows = many(host, &m.pix, &[-1], &ys, 1, 1);
                let dots: Vec<(usize, i32)> = (0..=n as usize).filter_map(|d| pixel(cols[d], rows[d]).map(|i| (i, ds[d]))).collect();
                if dots.is_empty() { continue; }
                let deck: Vec<i32> = dots.iter().flat_map(|&(i, pd)| [strength[s], fade, pd, depth[i]]).collect();
                let amount = many(host, &m.shade, &[26], &deck, 4, 1);
                for (j, &(i, _)) in dots.iter().enumerate() { items.push((i, glow, amount[j])); }
            }
        }
        blend(host, m, rgb, &items);
    }
    drawn
}

/// A leaf to draw: its ends on the screen (tenths of a pixel from the middle, y up), half width (tenths), whether a
/// blade, its colours at its start and end (lit), its fog (32nds) and how far it is ahead (tenths).
struct Leaf { a: (i32, i32), b: (i32, i32), w: i32, blade: bool, c0: [i32; 3], c1: [i32; 3], fog: i32, d: i32 }

/// The leaves' pixels and colours, each leaf by itself (its pixels in rows), against the depths as they are: for each
/// leaf, the pixels to paint, their colours and the depth to keep in them.
fn leaves(host: &mut Host, m: &Machines, list: &[Leaf], rim: [i32; 3], far: [i32; 3], radius: &[i32], depth: &[i32]) -> Vec<Vec<(usize, [i32; 3], i32)>> {
    let mut out = vec![];
    if list.is_empty() { return out; }
    let ends = many(host, &m.leafv, &[], &list.iter().flat_map(|l| [l.a.0, l.a.1, l.b.0, l.b.1]).collect::<Vec<_>>(), 4, 6);
    let lengths = magnitude(host, m, radius, &(0..list.len()).flat_map(|e| [ends[6 * e + 4], ends[6 * e + 5]]).collect::<Vec<_>>());
    let cols = many(host, &m.bbox, &[], &(0..list.len()).flat_map(|e| [ends[6 * e], ends[6 * e + 2], list[e].w]).collect::<Vec<_>>(), 3, 2);
    let rows = many(host, &m.bbox, &[], &(0..list.len()).flat_map(|e| [ends[6 * e + 1], ends[6 * e + 3], list[e].w]).collect::<Vec<_>>(), 3, 2);
    for (e, leaf) in list.iter().enumerate() {
        let (ax, ay, vx, vy, length) = (ends[6 * e], ends[6 * e + 1], ends[6 * e + 4], ends[6 * e + 5], lengths[e]);
        let mut paint = vec![];
        if length == 0 { out.push(paint); continue; }
        let unit = many(host, &m.slope, &[2], &[vx, length, vy, length], 2, 1);
        let reach = host.go1(&m.reach, &[&[1, leaf.w, length, vy]], 2);
        let (first, last) = (rows[2 * e].max(-48), rows[2 * e + 1].min(47));
        if first > last { out.push(paint); continue; }
        // the columns to look at on each row: within the strip round the middle line when steep, else all
        let spans: Vec<(i32, i32)> = if reach[0] == 1 {
            let line = host.go1(&m.strip, &[&[ay, vx, vy, first, last]], (last - first + 1) as usize);
            let s = many(host, &m.spanrow, &[ax, reach[1], cols[2 * e], cols[2 * e + 1]], &line, 1, 2);
            (0..line.len()).map(|k| (s[2 * k], s[2 * k + 1])).collect()
        } else { (first..=last).map(|_| (cols[2 * e], cols[2 * e + 1])).collect() };
        let cells: Vec<(i32, i32)> = spans.iter().enumerate().flat_map(|(k, &(l, r))| (l.max(-96)..=r.min(95)).map(move |c| (c, first + k as i32))).collect();
        if cells.is_empty() { out.push(paint); continue; }
        let px = many(host, &m.lin, &[ax, 0], &cells.iter().map(|c| c.0).collect::<Vec<_>>(), 1, 1);
        let py = many(host, &m.lin, &[ay, 0], &cells.iter().map(|c| c.1).collect::<Vec<_>>(), 1, 1);
        let a1 = many(host, &m.part, &[0], &px.iter().flat_map(|&x| [x, unit[0]]).collect::<Vec<_>>(), 2, 1);
        let a2 = many(host, &m.part, &[0], &py.iter().flat_map(|&y| [y, unit[1]]).collect::<Vec<_>>(), 2, 1);
        let b1 = many(host, &m.part, &[0], &px.iter().flat_map(|&x| [x, unit[1]]).collect::<Vec<_>>(), 2, 1);
        let b2 = many(host, &m.part, &[0], &py.iter().flat_map(|&y| [y, unit[0]]).collect::<Vec<_>>(), 2, 1);
        let along = many(host, &m.add, &[0], &zip2(&a1, &a2), 2, 1);
        let across = many(host, &m.add, &[1], &zip2(&b1, &b2), 2, 1);
        let cap = many(host, &m.cap, &[length], &along, 1, 2);
        let dist = magnitude(host, m, radius, &(0..cells.len()).flat_map(|k| [cap[2 * k], across[k]]).collect::<Vec<_>>());
        let u = many(host, &m.slope, &[2], &(0..cells.len()).flat_map(|k| [cap[2 * k + 1], length]).collect::<Vec<_>>(), 2, 1);
        let ww = many(host, &m.width, &[leaf.w, leaf.blade as i32], &u, 1, 1);
        let wide = many(host, &m.wide, &[], &zip2(&dist, &ww), 2, 2);
        let inside: Vec<usize> = (0..cells.len()).filter(|&k| wide[2 * k] == 1).collect();
        if inside.is_empty() { out.push(paint); continue; }
        let side = many(host, &m.slope, &[2], &inside.iter().flat_map(|&k| [across[k], wide[2 * k + 1]]).collect::<Vec<_>>(), 2, 1);
        let lit = many(host, &m.lit, &[leaf.blade as i32], &inside.iter().enumerate().flat_map(|(e, &k)| [side[e], ww[k], dist[k], across[k]]).collect::<Vec<_>>(), 4, 2);
        let u32s = many(host, &m.along, &[], &inside.iter().map(|&k| u[k]).collect::<Vec<_>>(), 1, 1);
        let mut colour = vec![[0; 3]; inside.len()];
        for c in 0..3 {
            let t: Vec<[i32; 3]> = (0..inside.len()).map(|e| [leaf.c0[c], leaf.c1[c], u32s[e]]).collect();
            let v = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..inside.len()).map(|e| [0, v[e], lit[2 * e]]).collect();
            let v = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..inside.len()).map(|e| [v[e], rim[c], lit[2 * e + 1]]).collect();
            let v = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..inside.len()).map(|e| [v[e], far[c], leaf.fog]).collect();
            for (e, v) in m.mix(host, &t).into_iter().enumerate() { colour[e][c] = v; }
        }
        let pixels: Vec<usize> = inside.iter().map(|&k| { let (c, r) = cells[k]; ((r + 48) * W2 as i32 + c + 96) as usize }).collect();
        let put = many(host, &m.put, &[leaf.d], &pixels.iter().map(|&i| depth[i]).collect::<Vec<_>>(), 1, 2);
        for (e, &i) in pixels.iter().enumerate() { if put[2 * e] == 1 { paint.push((i, colour[e], put[2 * e + 1])); } }
        out.push(paint);
    }
    out
}

/// A disc of one colour at p (tenths of a pixel from the middle, y up), radius r (tenths, kept within 5..31), ahead
/// d: its pixels, colour and depth to keep, against the depths as they are.
fn dot(host: &mut Host, m: &Machines, p: (i32, i32), r: i32, d: i32, colour: [i32; 3], depth: &[i32]) -> Vec<(usize, [i32; 3], i32)> {
    let across = host.go1(&m.span, &[&[0, 1, p.0, p.1, r]], 3);
    let down = host.go1(&m.span, &[&[1, 1, p.0, p.1, r]], 3);
    let cells: Vec<(i32, i32)> = (down[1].max(-48)..=down[2].min(47)).flat_map(|row| (across[1].max(-96)..=across[2].min(95)).map(move |c| (c, row))).collect();
    if cells.is_empty() { return vec![]; }
    let inside = many(host, &m.inside, &[p.0, p.1, across[0]], &cells.iter().flat_map(|&(c, r)| [c, r]).collect::<Vec<_>>(), 2, 1);
    let pixels: Vec<usize> = cells.iter().zip(&inside).filter(|(_, &i)| i == 1).map(|(&(c, r), _)| ((r + 48) * W2 as i32 + c + 96) as usize).collect();
    if pixels.is_empty() { return vec![]; }
    let put = many(host, &m.put, &[d], &pixels.iter().map(|&i| depth[i]).collect::<Vec<_>>(), 1, 2);
    pixels.iter().enumerate().filter(|(e, _)| put[2 * e] == 1).map(|(e, &i)| (i, colour, put[2 * e + 1])).collect()
}

/// Paint pixels in order: colour and depth.
fn apply(rgb: &mut [[i32; 3]], depth: &mut [i32], paint: &[(usize, [i32; 3], i32)]) {
    for &(i, c, d) in paint { rgb[i] = c; depth[i] = d; }
}

/// The kelp (clumps of three swaying stalks with blades on alternate sides, a bladder at each blade's base, their
/// tips glowing in the dark) and the seagrass, each painted over what is further.
fn plants(host: &mut Host, m: &Machines, map: &Map, fr: &Frame, far: [i32; 3], rgb: &mut [[i32; 3]], depth: &mut [i32]) {
    let tab = |t: &[i16]| -> Vec<i32> { t.iter().map(|&v| v as i32).collect() };
    let (radius, kelp_fog) = (tab(&demokit::tables::PORTAL_RADIUS), tab(&demokit::tables::KELP_FOG));
    let mut fog_head = vec![kelp_fog.len() as i32];
    fog_head.extend(&kelp_fog);
    // the plants' colours at this frame's light
    let pairs: [([i32; 3], [i32; 3]); 11] = [([4, 4, 2], [70, 62, 30]), ([5, 5, 2], [90, 80, 36]), ([6, 6, 2], [96, 92, 34]), ([8, 9, 3], [132, 150, 56]),
        ([10, 10, 4], [170, 160, 70]), ([10, 12, 6], [170, 200, 100]), ([4, 12, 8], [60, 200, 140]), ([8, 15, 11], [140, 255, 180]),
        ([4, 5, 2], [60, 90, 40]), ([7, 9, 4], [110, 150, 60]), ([0; 3], [0; 3])];
    let triples: Vec<[i32; 3]> = pairs.iter().flat_map(|(lo, hi)| (0..3).map(move |c| [lo[c], hi[c], fr.light])).collect();
    let lit = m.mix(host, &triples);
    let colour = |k: usize| [lit[3 * k], lit[3 * k + 1], lit[3 * k + 2]];
    let (stalk0, stalk1, blade0, blade1, bladder, rim, tip0, tip1, grass0, grass1) =
        (colour(0), colour(1), colour(2), colour(3), colour(4), colour(5), colour(6), colour(7), colour(8), colour(9));
    // printed tables: the stalk's 23 points and its 15 blades' bases, parts of 990 of the way up
    let points: Vec<i32> = (0..=22).map(|i| 45 * i).chain((1..=15).map(|i| 66 * i - 20)).collect();
    for clump in &fr.kelp {
        let row = clump.row as usize;
        let base = root(host, m, map, fr, clump.map_x, row);
        let root = [clump.at[0], base, clump.at[1]];
        let d = many(host, &m.dot, &fr.f100, &root, 3, 1)[0];
        if d < 20 { continue; }
        let scale = many(host, &m.slope, &[2], &[100, d], 2, 1)[0];
        let widths = many(host, &m.part, &[0], &[scale, 7, scale, 22, scale, 9], 2, 1);
        let fog_at = many(host, &m.index, &[], &[d], 1, 1);
        let fog = many(host, &m.look, &fog_head, &fog_at, 1, 1)[0];
        let mut list = vec![];
        let mut dots = vec![];
        for st in &clump.stalks {
            // the points up the stalk: height, lean and sway
            let hs = many(host, &m.part, &[0], &points.iter().flat_map(|&p| [st.tall, p / 10]).collect::<Vec<_>>(), 2, 1);
            let steps = many(host, &m.sway, &[], &hs.iter().flat_map(|&h| [st.phase, h]).collect::<Vec<_>>(), 2, 1);
            let waves = many(host, &m.wave, &[], &steps, 1, 1);
            let swing = many(host, &m.swing, &[], &zip2(&hs, &waves), 2, 1);
            let lean = many(host, &m.part, &[1], &points.iter().flat_map(|&p| [p, st.lean]).collect::<Vec<_>>(), 2, 1);
            let xs = many(host, &m.sum, &[], &(0..points.len()).flat_map(|k| [root[0], lean[k], swing[k]]).collect::<Vec<_>>(), 3, 1);
            let ys = many(host, &m.add, &[0], &hs.iter().flat_map(|&h| [root[1], h]).collect::<Vec<_>>(), 2, 1);
            let places: Vec<i32> = (0..points.len()).flat_map(|k| [xs[k], ys[k], root[2]]).collect();
            let seen = see_fine(host, m, fr, &places);
            for i in 0..22 {
                if let (Some(a), Some(b)) = (seen[i], seen[i + 1]) { list.push(Leaf { a: (a.0, a.1), b: (b.0, b.1), w: widths[0], blade: false, c0: stalk0, c1: stalk1, fog, d }); }
            }
            for (i, bl) in st.blades.iter().enumerate() {
                let Some(p) = seen[23 + i] else { continue };
                let half = many(host, &m.part, &[0], &[scale, bl[1]], 2, 1)[0];
                let length = host.go1(&m.twice, &[&[1, half]], 1)[0];
                let cs = many(host, &m.cos, &[bl[0]], &[0], 1, 2);
                let off = many(host, &m.part, &[0], &[length, cs[0], length, cs[1]], 2, 1);
                let end = host.go1(&m.tip, &[&[p.0, p.1, off[0], off[1]]], 3);
                if end[0] == 0 { continue; }
                let (tip, tip_up) = (end[1], end[2]);
                list.push(Leaf { a: (p.0, p.1), b: (tip, tip_up), w: widths[1], blade: true, c0: blade0, c1: blade1, fog, d });
                dots.push((list.len(), (p.0, p.1)));
                if fr.glow_tips == 1 && i == 14 { list.push(Leaf { a: (p.0, p.1), b: (tip, tip_up), w: widths[1], blade: true, c0: tip0, c1: tip1, fog, d }); }
            }
        }
        // paint the clump in order: leaves, with each blade's bladder after it
        let painted = leaves(host, m, &list, rim, far, &radius, depth);
        let bead: Vec<i32> = (0..3).map(|c| m.mix(host, &[[bladder[c], far[c], fog]])[0]).collect();
        let bead = [bead[0], bead[1], bead[2]];
        let r = widths[2];
        let beads: Vec<(usize, Vec<(usize, [i32; 3], i32)>)> = dots.iter().map(|&(after, p)| (after, dot(host, m, p, r, d, bead, depth))).collect();
        for (k, leaf) in painted.iter().enumerate() {
            apply(rgb, depth, leaf);
            for (after, b) in &beads { if *after == k + 1 { apply(rgb, depth, b); } }
        }
    }
    // seagrass, blade by blade
    for tuft in &fr.grass {
        let row = tuft.row as usize;
        let base = root(host, m, map, fr, tuft.map_x, row);
        // each blade's sway a sixth of a turn on from the last (a printed table of 0, 10, ..., 40 steps)
        let steps = many(host, &m.wiggle, &[tuft.phase], &(0..5).flat_map(|b| [10 * b, 0]).collect::<Vec<_>>(), 2, 1);
        let waves = many(host, &m.wave, &[], &steps, 1, 1);
        let sines = many(host, &m.add, &[1], &waves.iter().flat_map(|&v| [v, 20]).collect::<Vec<_>>(), 2, 1);
        let sway = many(host, &m.part, &[0], &sines.iter().flat_map(|&v| [v, 8]).collect::<Vec<_>>(), 2, 1);
        for b in 0..5 {
            let off = b as i32 - 2;
            let x0 = many(host, &m.add, &[0], &[tuft.at[0], off], 2, 1)[0];
            let x1 = many(host, &m.sum, &[], &[tuft.at[0], 2 * off, sway[b]], 3, 1)[0];
            let y1 = many(host, &m.add, &[0], &[base, tuft.heights[b]], 2, 1)[0];
            let seen = see_fine(host, m, fr, &[x0, base, tuft.at[1], x1, y1, tuft.at[1]]);
            let (Some(a), Some(t)) = (seen[0], seen[1]) else { continue };
            if a.2 > 220 { continue; }
            let scale = many(host, &m.slope, &[2], &[100, a.2], 2, 1)[0];
            let w = many(host, &m.part, &[0], &[scale, 5], 2, 1)[0];
            let fog_at = many(host, &m.index, &[], &[a.2], 1, 1);
            let fog = many(host, &m.look, &fog_head, &fog_at, 1, 1)[0];
            let painted = leaves(host, m, &[Leaf { a: (a.0, a.1), b: (t.0, t.1), w, blade: true, c0: grass0, c1: grass1, fog, d: a.2 }], rim, far, &radius, depth);
            apply(rgb, depth, &painted[0]);
        }
    }
}

/// The schools of fish: a short line each, silver, flashing as the school turns, painted over what is further.
fn fish(host: &mut Host, m: &Machines, fr: &Frame, far: [i32; 3], rgb: &mut [[i32; 3]], depth: &mut [i32]) {
    let fish_fog: Vec<i32> = demokit::tables::FISH_FOG.iter().map(|&v| v as i32).collect();
    let mut fish_head = vec![fish_fog.len() as i32];
    fish_head.extend(&fish_fog);
    for fish in &fr.fish {
        let seen = see_fine(host, m, fr, &[fish.head, fish.tail].concat());
        let (Some(a), Some(b)) = (seen[0], seen[1]) else { continue };
        if a.2 < 10 { continue; }
        let fog_at = many(host, &m.index, &[], &[a.2], 1, 1);
        let fog = many(host, &m.look, &fish_head, &fog_at, 1, 1)[0];
        let colour: Vec<i32> = (0..3).map(|c| {
            let dim = m.mix(host, &[[[70, 82, 88][c], [140, 165, 175][c], fr.fish_light]])[0];
            let bright = m.mix(host, &[[[120, 124, 125][c], [240, 248, 250][c], fr.fish_light]])[0];
            let silver = m.mix(host, &[[dim, bright, fish.flash]])[0];
            m.mix(host, &[[silver, far[c], fog]])[0]
        }).collect();
        let n = host.go1(&m.steps, &[&[1, a.0, a.1, b.0, b.1]], 1)[0];
        let xs = host.go1(&m.step, &[&[1, a.0, b.0, n]], n as usize + 1);
        let ys = host.go1(&m.step, &[&[1, a.1, b.1, n]], n as usize + 1);
        let cols = many(host, &m.pix, &[1], &xs, 1, 1);
        let rows = many(host, &m.pix, &[-1], &ys, 1, 1);
        let thick = if a.2 < 140 { 1 } else { 0 };
        let mut paint = vec![];
        for k in 0..=n as usize {
            for r in rows[k]..=rows[k] + thick {
                let (x, y) = (cols[k] + 96, r + 48);
                if x < 0 || y < 0 || x >= W2 as i32 || y >= H2 as i32 { continue; }
                let i = (y * W2 as i32 + x) as usize;
                let put = many(host, &m.put, &[a.2], &[depth[i]], 1, 2);
                if put[0] == 1 { paint.push((i, [colour[0], colour[1], colour[2]], put[1])); depth[i] = put[1]; rgb[i] = [colour[0], colour[1], colour[2]]; }
            }
        }
    }
}

/// The marine snow: specks hanging in the water in a box that goes round with the camera, drawn three times along;
/// each mixes the snow's colour into its pixel, fading with distance, where the seabed is not nearer.
fn snow(host: &mut Host, m: &Machines, fr: &Frame, specks: &[i32], rgb: &mut [[i32; 3]], depth: &[i32]) {
    let points: Vec<i32> = specks.chunks(300).flat_map(|c| host.go1(&m.snow, &[&[fr.snow[0], fr.snow[1], (c.len() / 3) as i32], c], 3 * c.len())).collect();
    let seen: Vec<(usize, i32)> = project(host, m, fr, &points).into_iter().flatten()
        .filter(|&(x, y, _)| (0..W2 as i32).contains(&x) && (0..H2 as i32).contains(&y)).map(|(x, y, a)| (y as usize * W2 + x as usize, a)).collect();
    if seen.is_empty() { return; }
    let deck: Vec<i32> = seen.iter().flat_map(|&(i, a)| [a, depth[i]]).collect();
    let amount = many(host, &m.speck, &[fr.show], &deck, 2, 1);
    let items: Vec<(usize, [i32; 3], i32)> = seen.iter().zip(&amount).filter(|(_, &k)| k > 0).map(|(&(i, _), &k)| (i, [170, 200, 200], k)).collect();
    blend(host, m, rgb, &items);
}

/// The portal: a ring of light standing on the diver's path, its middle a slow cyan and violet swirl, its light
/// spilling round it. Per screen row: where each ray crosses the portal's plane (a walk machine adds up its steps
/// to the plane), whether the seabed is nearer, how far and which way that is from the middle (a ratio machine and
/// two tables), the swirl's waves there, and how much of the swirl, the ring and the spill to mix in.
#[allow(clippy::too_many_arguments)]
fn portal(host: &mut Host, m: &Machines, fr: &Frame, rays: &[Ray], settled: &[Settled], crossed: &[i32], start: i32) -> Vec<[Mixes; 3]> {
    let (w, h) = (W2, H2);
    let mut rows = vec![];
    let (whole, tenth) = (fr.plane / 10, fr.plane % 10);
    let radius: Vec<i32> = demokit::tables::PORTAL_RADIUS.iter().map(|&v| v as i32).collect();
    let angle: Vec<i32> = demokit::tables::PORTAL_ANGLE.iter().map(|&v| v as i32).collect();
    for y in 0..h {
        let row = &rays[y * w..(y + 1) * w];
        // where each ray crosses the plane, across and up, in tenths from the middle
        let mut l = vec![];
        for (c, from) in fr.from.iter().enumerate() {
            let q = |r: &Ray| if c == 0 { r.qx } else { r.qy };
            let deck: Vec<i32> = row.iter().flat_map(|r| [tenth, q(r)]).collect();
            let first = host.go1(&m.tenth, &[&[w as i32], &deck], w);
            let deck: Vec<i32> = row.iter().zip(&first).flat_map(|(r, &o)| [from.div_euclid(100), 0, from.rem_euclid(100), o, q(r), whole]).collect();
            let pairs = host.go1(&m.walks, &[&[w as i32], &deck], 2 * w);
            l.push(host.go1(&m.tenths, &[&[w as i32], &pairs], w));
        }
        let near: Vec<usize> = (0..w).filter(|&x| l[0][x] != 400 && l[1][x] != 400).collect();
        if near.is_empty() { continue; }
        let n = near.len() as i32;
        let deck: Vec<i32> = near.iter().flat_map(|&x| match settled[y * w + x] { Settled::Hit { k, .. } => [k, crossed[y * w + x]], Settled::Water => [-1, 0] }).collect();
        let seen = host.go1(&m.behind, &[&[fr.plane, start, n], &deck], near.len());
        let near: Vec<usize> = near.into_iter().zip(&seen).filter(|(_, &s)| s == 1).map(|(x, _)| x).collect();
        if near.is_empty() { continue; }
        let n = near.len() as i32;
        // how far and which way from the middle
        let deck: Vec<i32> = near.iter().flat_map(|&x| [l[0][x], l[1][x]]).collect();
        let order = host.go1(&m.order, &[&[n], &deck], 5 * near.len());
        let deck: Vec<i32> = (0..near.len()).flat_map(|e| [order[5 * e], order[5 * e + 1]]).collect();
        let ratio = host.go1(&m.ratio, &[&[n], &deck], near.len());
        let share = host.go1(&m.look, &[&[radius.len() as i32], &radius, &[n], &ratio], near.len());
        let eighth = host.go1(&m.look, &[&[angle.len() as i32], &angle, &[n], &ratio], near.len());
        let deck: Vec<i32> = (0..near.len()).flat_map(|e| [order[5 * e], share[e]]).collect();
        let far = host.go1(&m.scale, &[&[n], &deck], near.len());
        let deck: Vec<i32> = (0..near.len()).flat_map(|e| [eighth[e], order[5 * e + 2], order[5 * e + 3], order[5 * e + 4]]).collect();
        let way = host.go1(&m.turn, &[&[n], &deck], near.len());
        // the swirl and its colour
        let deck: Vec<i32> = (0..near.len()).flat_map(|e| [way[e], far[e]]).collect();
        let steps = host.go1(&m.swirl, &[&[fr.swirl[0], fr.swirl[1], n], &deck], 2 * near.len());
        let waves: Vec<Vec<i32>> = (0..2).map(|k| { let d: Vec<i32> = (0..near.len()).map(|e| steps[2 * e + k]).collect(); host.go1(&m.wave, &[&[n], &d], near.len()) }).collect();
        let deck: Vec<i32> = (0..near.len()).flat_map(|e| [far[e], waves[0][e], waves[1][e]]).collect();
        let inner = host.go1(&m.inner, &[&[n], &deck], 3 * near.len());
        let deck: Vec<i32> = (0..near.len()).flat_map(|e| [far[e], inner[3 * e + 2]]).collect();
        let ring = host.go1(&m.ring, &[&[fr.on, n], &deck], 2 * near.len());
        let spill = host.go1(&m.spill, &[&[n], &far], near.len());
        let mut swirl = vec![[0; 3]; near.len()];
        for c in 0..3 {
            let t: Vec<[i32; 3]> = (0..near.len()).map(|e| [[90, 240, 255][c], [200, 120, 255][c], inner[3 * e + 1]]).collect();
            let hue = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..near.len()).map(|e| [[20, 60, 120][c], hue[e], inner[3 * e]]).collect();
            for (e, v) in m.mix(host, &t).into_iter().enumerate() { swirl[e][c] = v; }
        }
        let i = |e: usize| y * w + near[e];
        rows.push([(0..near.len()).map(|e| (i(e), swirl[e], ring[2 * e])).collect::<Vec<_>>(),
            (0..near.len()).map(|e| (i(e), [220, 250, 255], ring[2 * e + 1])).collect::<Vec<_>>(),
            (0..near.len()).map(|e| (i(e), [80, 200, 240], spill[e])).collect::<Vec<_>>()]);
    }
    rows
}

/// Mixes to lay on: (pixel, colour, amount).
type Mixes = Vec<(usize, [i32; 3], i32)>;

/// The portal's mixes laid on, row by row: the swirl and the ring (32nds), the spill (tenths of 32nds, mixed in
/// 256ths by the portal's on).
fn lay_portal(host: &mut Host, m: &Machines, fr: &Frame, rows: &[[Mixes; 3]], rgb: &mut [[i32; 3]]) {
    for r in rows {
        blend(host, m, rgb, &r[0]);
        blend(host, m, rgb, &r[1]);
        blend_by(host, &m.spill_mix, &[fr.on], rgb, &r[2]);
    }
}

/// The caption: a black strip at the top left, the words in white, faded in and out.
fn lay_caption(host: &mut Host, m: &Machines, fr: &Frame, caption: &[Vec<i32>], rgb: &mut [[i32; 3]]) {
    let w = W2;
    for (y, dots) in caption.iter().enumerate() {
        let mut deck = vec![fr.caption];
        for x in 0..dots.len() { deck.push(dots[x]); deck.extend_from_slice(&rgb[y * w + x]); }
        let o = host.go1(&m.caption, &[&deck], 3 * dots.len());
        for x in 0..dots.len() { rgb[y * w + x] = [o[3 * x], o[3 * x + 1], o[3 * x + 2]]; }
    }
}

/// A kelp or seagrass root's height from the camera (tenths): the seabed under it on its map row, near (slices
/// before 30) the top of its block, further the smooth seabed, as the march sees it.
fn root(host: &mut Host, m: &Machines, map: &Map, fr: &Frame, map_x: [i32; 2], row: usize) -> i32 {
    if (row as i32) < fr.z + 1 + NEAR {
        let block = host.go(&m.blocks, &[&[&[map_x[0], 0, 1], &map.rows[row]]], Some(1))[0];
        let top = many(host, &m.add, &[0], &[map.base[row], block], 2, 1)[0];
        host.go1(&m.base, &[&[fr.height10, 1, top, 0]], 1)[0]
    } else {
        let h = host.go(&m.sample, &[&[&map_x, &map.rows[row]]], Some(1))[0];
        host.go1(&m.base, &[&[fr.height10, 1, map.base[row], h]], 1)[0]
    }
}

/// TOTALLY WORTH IT in the listing's 3 x 5 font, 4 dots a letter, a blank row above and below: 7 rows of
/// 69 dots, 1 lit.
fn caption_dots(demo: &Path) -> Vec<Vec<i32>> {
    let mut font = std::collections::HashMap::new();
    for l in read(demo, "deep/data/font3x5.txt").lines().filter(|l| !l.starts_with('#')) {
        let p: Vec<&str> = l.split(' ').collect();
        font.insert(if p[0] == "space" { ' ' } else { p[0].chars().next().unwrap() }, p[1..6].iter().map(|r| r.to_string()).collect::<Vec<String>>());
    }
    let text: Vec<char> = " TOTALLY WORTH IT".chars().collect();
    (0..7).map(|y| (0..69).map(|x| {
        let (ch, cx) = (text.get(x / 4).copied().unwrap_or(' '), x % 4);
        (y >= 1 && y <= 5 && cx < 3 && font[&ch][y - 1].as_bytes()[cx] == b'1') as i32
    }).collect()).collect()
}

/// The creatures at 1536x768: their machines, atlases and tables, and the first frame each jellyfish showed.
pub struct Hd { m: deep_hd::Machines, d: std::sync::Arc<deep_hd::Data>, first: Vec<i32> }

/// One frame's colours, and its creatures at 1536x768 when asked for.
#[allow(clippy::too_many_arguments)]
fn frame(host: &mut Host, host_hd: &mut Host, m: &Machines, map: &Map, fr: &Frame, norm: &[i32], specks: &[i32], caption: &[Vec<i32>], hd: Option<&Hd>) -> (Vec<[i32; 3]>, Option<Vec<u8>>) {
    let (w, h) = (W2, H2);
    let j0 = (fr.z + 1) as usize;           // the first slice is the map row just ahead of the camera
    let start = 10 - fr.z_part;             // tenths of a unit from the camera to it
    // the frame's colours at its light
    let pairs: [([i32; 3], [i32; 3]); 11] = [([1, 2, 6], [4, 30, 50]), ([2, 6, 14], [40, 160, 170]), ([1, 3, 8], [6, 40, 62]),
        ([12, 11, 9], SAND), ([13, 15, 14], [210, 250, 240]), ([16, 16, 16], [200, 220, 215]),
        ([7, 7, 8], ROCKS[0]), ([7, 7, 8], ROCKS[1]), ([7, 7, 8], ROCKS[2]), ([7, 7, 8], ROCKS[3]), ([7, 7, 8], ROCKS[4])];
    let triples: Vec<[i32; 3]> = pairs.iter().flat_map(|(lo, hi)| (0..3).map(move |c| [lo[c], hi[c], fr.light])).collect();
    let lit = m.mix(host, &triples);
    let colour = |k: usize| [lit[3 * k], lit[3 * k + 1], lit[3 * k + 2]];
    let (water_down, water_up, far, sand, caustics, pale) = (colour(0), colour(1), colour(2), colour(3), colour(4), colour(5));
    let rocks: Vec<[i32; 3]> = (6..11).map(colour).collect();
    // the rays, a screen row at a time
    let mut rays = vec![Ray::default(); w * h];
    let mut dir = vec![[0i32; 3]; w * h];
    for y in 0..h {
        let b = 95 - 2 * y as i32;
        let comp: Vec<Vec<i32>> = (0..3).map(|c| host.go1(&m.rays, &[&[fr.f[c], fr.r[c], fr.u[c], b]], w)).collect();
        let pairs = |n: &Vec<i32>| -> Vec<i32> { (0..w).flat_map(|x| [n[x], comp[2][x]]).collect() };
        let mut deck = vec![];
        let qx = { deck.clear(); deck.extend(pairs(&comp[0])); host.go1(&m.slope, &[&[2, w as i32], &deck], w) };
        let qy = { deck.clear(); deck.extend(pairs(&comp[1])); host.go1(&m.slope, &[&[2, w as i32], &deck], w) };
        let firsts = |q: &Vec<i32>| -> Vec<i32> { (0..w).flat_map(|x| [start, q[x]]).collect() };
        let fx = host.go1(&m.tenth, &[&[w as i32], &firsts(&qx)], w);
        let fy = host.go1(&m.tenth, &[&[w as i32], &firsts(&qy)], w);
        let last = host.go1(&m.limit, &[&comp[2]], w);
        // the slopes the other way, for the walks back
        let (nqx, nqy) = (host.go1(&m.neg, &[&[w as i32], &qx], w), host.go1(&m.neg, &[&[w as i32], &qy], w));
        for x in 0..w {
            rays[y * w + x] = Ray { qx: qx[x], qy: qy[x], nqx: nqx[x], nqy: nqy[x], dz: comp[2][x], fx: fx[x], fy: fy[x], last: last[x] };
            dir[y * w + x] = [comp[0][x], comp[1][x], comp[2][x]];
        }
    }
    // the march
    // walks, many to a run: each (whole, less, part, first, step, k)
    let walks = |host: &mut Host, items: &[[i32; 6]]| -> Vec<[i32; 2]> {
        let flat: Vec<i32> = items.iter().flatten().copied().collect();
        many(host, &m.walks, &[], &flat, 6, 2).chunks(2).map(|o| [o[0], o[1]]).collect()
    };
    // pixels' rays entered at slice k: x and height, for each pixel
    let enter = |host: &mut Host, pixels: &[usize], k: i32| -> Vec<([i32; 2], [i32; 2])> {
        let items: Vec<[i32; 6]> = pixels.iter().flat_map(|&i| { let ray = &rays[i]; [
            [fr.x.div_euclid(100), 0, fr.x.rem_euclid(100), ray.fx, ray.qx, k],
            [fr.y, map.base[j0 + k as usize], fr.y_part, ray.fy, ray.qy, k]] }).collect();
        walks(host, &items).chunks(2).map(|o| (o[0], o[1])).collect()
    };
    let mut settled = vec![Settled::Water; w * h];
    // per column: the pixel being marched (row, counting up from the bottom), its ray's x and height
    let mut pixel: Vec<i32> = vec![h as i32 - 1; w];
    let mut state: Vec<([i32; 2], [i32; 2])> = enter(host, &(0..w).map(|x| (h - 1) * w + x).collect::<Vec<_>>(), 0);
    let mut k = 0;
    while pixel.iter().any(|&p| p >= 0) {
        let j = j0 + k as usize;
        let row = &map.rows[j];
        let mut pending: Vec<usize> = (0..w).filter(|&x| pixel[x] >= 0).collect();
        while !pending.is_empty() {
            let mut again = vec![];
            for &x in &pending {
                let i = pixel[x] as usize * w + x;
                let ray = &rays[i];
                let (sx, sy) = state[x];
                let o = host.go(&m.slice, &[&[&[k, ray.last, sx[0], sx[1], sy[0], sy[1], ray.qx, ray.qy, map.drop[j]], row]], Some(5));
                match o[0] {
                    0 => { state[x] = ([o[1], o[2]], [o[3], o[4]]); continue; }
                    1 => settled[i] = Settled::Hit { k, x: sx, y: sy },
                    2 => {}
                    _ => { pixel[x] = -1; continue; }       // above everything: the rest of the column is water
                }
                pixel[x] -= 1;
                if pixel[x] >= 0 { again.push(x); }
            }
            // the next pixels up, entered at this slice
            let entered = enter(host, &again.iter().map(|&x| pixel[x] as usize * w + x).collect::<Vec<_>>(), k);
            for (&x, e) in again.iter().zip(entered) { state[x] = e; }
            pending = again;
        }
        k += 1;
    }
    // the colours, a screen row at a time
    let mut rgb = vec![[0i32; 3]; w * h];
    let mut depth = vec![999i32; w * h];
    let mut crossed = vec![0i32; w * h];
    for y in 0..h {
        let hits: Vec<usize> = (0..w).filter(|&x| matches!(settled[y * w + x], Settled::Hit { .. })).collect();
        let waters: Vec<usize> = (0..w).filter(|&x| matches!(settled[y * w + x], Settled::Water)).collect();
        // water: how far up each ray looks, its colour, and the god rays: two waves across the screen, sharpened
        if !waters.is_empty() {
            let n = waters.len();
            let deck: Vec<i32> = waters.iter().flat_map(|&x| [dir[y * w + x][1], norm[y * w + x]]).collect();
            let up = host.go1(&m.slope, &[&[2, n as i32], &deck], n);
            let level = host.go1(&m.water, &[&[n as i32], &up], n);
            let steps = host.go1(&m.shaft, &[&[y as i32, fr.shaft[0] / 100, fr.shaft[0] % 100, fr.shaft[1] / 100, fr.shaft[1] % 100]], 2 * w);
            let mut sharp = vec![];
            for wave in 0..2 {
                let deck: Vec<i32> = waters.iter().map(|&x| steps[2 * x + wave]).collect();
                let v = host.go1(&m.wave, &[&[n as i32], &deck], n);
                sharp.push(host.go1(&m.pow, &[&[wave as i32, n as i32], &v], n));
            }
            let deck: Vec<i32> = (0..n).flat_map(|e| [sharp[0][e], sharp[1][e], up[e]]).collect();
            let shine = host.go1(&m.ray, &[&[fr.light, n as i32], &deck], n);
            for c in 0..3 {
                let t: Vec<[i32; 3]> = level.iter().map(|&l| [water_down[c], water_up[c], l]).collect();
                let water = m.mix(host, &t);
                let t: Vec<[i32; 3]> = (0..n).map(|e| [water[e], [170, 240, 230][c], shine[e]]).collect();
                for (e, v) in m.mix(host, &t).into_iter().enumerate() { rgb[y * w + waters[e]][c] = v; }
            }
        }
        if hits.is_empty() { continue; }
        // per hit: near (slices before 30) the block the ray went into and its face, far the smooth seabed; both give
        // where it crossed (tenths of the slice), the rock's place in the tables, its tone, the light on it, the
        // light its edge catches and how rocky it is
        let n = hits.len();
        let hit = |e: usize| -> (i32, usize, [i32; 2], [i32; 2], Ray) {
            let i = y * w + hits[e];
            let Settled::Hit { k, x: sx, y: sy } = settled[i] else { unreachable!() };
            (k, j0 + k as usize, sx, sy, rays[i])
        };
        let near: Vec<usize> = (0..n).filter(|&e| hit(e).0 < NEAR).collect();
        let far_hits: Vec<usize> = (0..n).filter(|&e| hit(e).0 >= NEAR).collect();
        let mut tenths = vec![0; n];
        let mut xp = vec![[0; 2]; n];
        let mut tone = vec![0; n];
        let mut light = vec![0; n];
        let mut rim = vec![0; n];
        let mut rocky = vec![0; n];
        let col = |v: &Vec<i32>, k: usize, width: usize| -> Vec<i32> { (0..v.len() / width).map(|e| v[width * e + k]).collect() };
        if !near.is_empty() {
            let c = near.len() as i32;
            // a slice back (over the hit's row base), and the blocks about the columns there and at the hit
            let back = walks(host, &near.iter().flat_map(|&e| { let (_, _, sx, sy, ray) = hit(e);
                [[sy[0], 0, sy[1], ray.nqy, 0, 0], [sx[0], 0, sx[1], ray.nqx, 0, 0]] }).collect::<Vec<_>>());
            let ye: Vec<[i32; 2]> = (0..near.len()).map(|q| back[2 * q]).collect();
            for (q, &e) in near.iter().enumerate() { xp[e] = back[2 * q + 1]; }
            let around = |host: &mut Host, x: i32, j: usize| -> Vec<i32> { host.go(&m.blocks, &[&[&[x, -1, 3], &map.rows[j]]], Some(3)) };
            let at_before: Vec<Vec<i32>> = near.iter().map(|&e| around(host, xp[e][0], hit(e).1)).collect();
            let at_hit: Vec<Vec<i32>> = near.iter().map(|&e| { let (_, j, sx, _, _) = hit(e); around(host, sx[0], j) }).collect();
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| { let (_, _, sx, _, ray) = hit(e); [ye[q][0], xp[e][0], sx[0], xp[e][1], ray.qx, at_before[q][1]] }).collect();
            let how = many(host, &m.column, &[], &deck, 6, 2);
            let descend = |host: &mut Host, level: &dyn Fn(usize) -> i32| -> Vec<i32> {
                let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| { let sy = hit(e).3; [ye[q][0], ye[q][1], sy[0], sy[1], level(q)] }).collect();
                many(host, &m.descend, &[], &deck, 5, 1)
            };
            let first = descend(host, &|q| at_before[q][1]);
            let last = descend(host, &|q| at_hit[q][1]);
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| [how[2 * q], how[2 * q + 1], first[q], last[q], hit(e).4.qx]).collect();
            let sides = many(host, &m.side, &[], &deck, 5, 3);
            let face = col(&sides, 0, 3);
            for (q, &e) in near.iter().enumerate() { tenths[e] = sides[3 * q + 1]; }
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| {
                let (b, a) = (&at_before[q], &at_hit[q]);
                [sides[3 * q + 2], xp[e][0], hit(e).2[0], b[0], b[1], b[2], a[0], a[1], a[2]] }).collect();
            let picked = many(host, &m.pick, &[], &deck, 9, 4);
            let (column, top, left, right) = (col(&picked, 0, 4), col(&picked, 1, 4), col(&picked, 2, 4), col(&picked, 3, 4));
            // the blocks in front and behind, over the hit's row base
            let front: Vec<i32> = near.iter().enumerate().map(|(q, &e)| host.go(&m.blocks, &[&[&[column[q], 0, 1], &map.rows[hit(e).1 - 1]]], Some(1))[0]).collect();
            let behind: Vec<i32> = near.iter().enumerate().map(|(q, &e)| host.go(&m.blocks, &[&[&[column[q], 0, 1], &map.rows[hit(e).1 + 1]]], Some(1))[0]).collect();
            let front = many(host, &m.add, &[0], &near.iter().enumerate().flat_map(|(q, &e)| [front[q], map.drop[hit(e).1 - 1]]).collect::<Vec<_>>(), 2, 1);
            let behind = many(host, &m.add, &[1], &near.iter().enumerate().flat_map(|(q, &e)| [behind[q], map.drop[hit(e).1]]).collect::<Vec<_>>(), 2, 1);
            // the hit's place across and its height
            let shift_x = host.go1(&m.tenth, &[&[c], &near.iter().flat_map(|&e| [tenths[e], hit(e).4.qx]).collect::<Vec<_>>()], near.len());
            let shift_y = host.go1(&m.tenth, &[&[c], &near.iter().flat_map(|&e| [tenths[e], hit(e).4.qy]).collect::<Vec<_>>()], near.len());
            let on = walks(host, &near.iter().enumerate().flat_map(|(q, &e)|
                [[xp[e][0], 0, xp[e][1], shift_x[q], 0, 0], [ye[q][0], 0, ye[q][1], shift_y[q], 0, 0]]).collect::<Vec<_>>());
            let across: Vec<i32> = (0..near.len()).map(|q| on[2 * q][1]).collect();
            let height: Vec<[i32; 2]> = (0..near.len()).map(|q| on[2 * q + 1]).collect();
            // the light: the top's crevices and lit front edge, its edges across, a face's foot and crest
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| [face[q], tenths[e], top[q], front[q], behind[q]]).collect();
            let tops = many(host, &m.top, &[], &deck, 5, 3);
            let deck: Vec<i32> = (0..near.len()).flat_map(|q| [face[q], across[q], top[q], left[q], right[q], tops[3 * q], tops[3 * q + 1]]).collect();
            let corner = many(host, &m.corner, &[], &deck, 7, 2);
            let deck: Vec<i32> = (0..near.len()).flat_map(|q| [face[q], front[q], at_before[q][1], height[q][0], height[q][1], corner[2 * q]]).collect();
            let lit = many(host, &m.riser, &[], &deck, 6, 1);
            let deck: Vec<i32> = (0..near.len()).flat_map(|q| [face[q], top[q], height[q][0], height[q][1], corner[2 * q + 1]]).collect();
            let crest = many(host, &m.crest, &[], &deck, 5, 3);
            // the rock: its place in the tables, its stratum, its grain, coral on a lip, how rocky
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| [column[q], 0, hit(e).1 as i32]).collect();
            let places = many(host, &m.place, &[], &deck, 3, 2);
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| [map.base[hit(e).1], crest[3 * q + 1], crest[3 * q + 2], face[q], places[2 * q + 1]]).collect();
            let strata = many(host, &m.strata, &[], &deck, 5, 2);
            let deck: Vec<i32> = near.iter().enumerate().flat_map(|(q, &e)| [places[2 * q], across[q], tenths[e], face[q], crest[3 * q + 2]]).collect();
            let grain = many(host, &m.grain, &[], &deck, 5, 1);
            let deck: Vec<i32> = (0..near.len()).flat_map(|q| [places[2 * q], strata[2 * q], grain[q], lit[q], strata[2 * q + 1], tops[3 * q + 2]]).collect();
            let coral = many(host, &m.coral, &[], &deck, 6, 2);
            let deck: Vec<i32> = (0..near.len()).flat_map(|q| [face[q], coral[2 * q], top[q]]).collect();
            let rock = many(host, &m.rocky, &[], &deck, 3, 1);
            for (q, &e) in near.iter().enumerate() {
                tone[e] = coral[2 * q]; light[e] = coral[2 * q + 1]; rim[e] = crest[3 * q]; rocky[e] = rock[q];
            }
        }
        if !far_hits.is_empty() {
            let c = far_hits.len() as i32;
            // back one slice, to the row before: the seabed there and at the hit, where the ray crossed, the slope
            struct Far { yp: [i32; 2], hp: i32, hs: i32, hf: i32 }
            let back = walks(host, &far_hits.iter().flat_map(|&e| { let (_, j, sx, sy, ray) = hit(e);
                [[sx[0], 0, sx[1], ray.nqx, 0, 0], [sy[0], map.drop[j - 1], sy[1], ray.nqy, 0, 0]] }).collect::<Vec<_>>());
            let fs: Vec<Far> = far_hits.iter().enumerate().map(|(q, &e)| {
                let (_, j, sx, _, _) = hit(e);
                xp[e] = back[2 * q];
                let yp = back[2 * q + 1];
                let hp = host.go(&m.sample, &[&[&xp[e], &map.rows[j - 1]]], Some(1))[0];
                let hs = host.go(&m.sample, &[&[&sx, &map.rows[j]]], Some(1))[0];
                let hf = host.go(&m.sample, &[&[&sx, &map.rows[j - 1]]], Some(1))[0];
                Far { yp, hp, hs, hf }
            }).collect();
            let deck: Vec<i32> = far_hits.iter().zip(&fs).flat_map(|(&e, f)| { let sy = hit(e).3; [f.yp[0], f.yp[1], f.hp, sy[0], sy[1], f.hs] }).collect();
            let crossed_at = host.go1(&m.cross, &[&[c], &deck], far_hits.len());
            let deck: Vec<i32> = far_hits.iter().zip(&fs).enumerate().flat_map(|(q, (&e, f))| [f.hp, f.hs, f.hf, map.drop[hit(e).1 - 1], crossed_at[q]]).collect();
            let face = host.go1(&m.face, &[&[c], &deck], 2 * far_hits.len());
            // the strata as a texture: the column's place in the tables, the stratum and band at the seabed's height
            let deck: Vec<i32> = far_hits.iter().flat_map(|&e| { let (_, j, sx, _, _) = hit(e); [sx[0], 0, j as i32] }).collect();
            let places = many(host, &m.place, &[], &deck, 3, 2);
            let height = many(host, &m.far2, &[], &fs.iter().map(|f| f.hs).collect::<Vec<_>>(), 1, 2);
            let deck: Vec<i32> = far_hits.iter().enumerate().flat_map(|(q, &e)| [map.base[hit(e).1], height[2 * q], height[2 * q + 1], 4, places[2 * q + 1]]).collect();
            let strata = many(host, &m.strata, &[], &deck, 5, 2);
            let deck: Vec<i32> = (0..far_hits.len()).flat_map(|q| [places[2 * q], strata[2 * q], 0, face[2 * q], strata[2 * q + 1], 0]).collect();
            let coral = many(host, &m.coral, &[], &deck, 6, 2);
            for (q, &e) in far_hits.iter().enumerate() {
                tenths[e] = crossed_at[q]; tone[e] = coral[2 * q]; light[e] = coral[2 * q + 1]; rocky[e] = face[2 * q + 1];
            }
        }
                let deck: Vec<i32> = (0..n).flat_map(|e| [tenths[e], hit(e).0, hit(e).4.dz]).collect();
        let far_fog = host.go1(&m.far, &[&[start, n as i32], &deck], 2 * n);
        let beam = host.go1(&m.beam, &[&[y as i32, fr.aim[0], fr.aim[1], n as i32], &hits.iter().map(|&x| x as i32).collect::<Vec<_>>()], n);
        let lamp = host.go1(&m.lamp, &[&[fr.light, n as i32], &(0..n).flat_map(|e| [beam[e], far_fog[2 * e]]).collect::<Vec<_>>()], n);
        // the caustics: where the ray crossed, the waves there, their glint
        let deck: Vec<i32> = (0..n).flat_map(|e| [tenths[e], hit(e).4.qx]).collect();
        let shift = host.go1(&m.tenth, &[&[n as i32], &deck], n);
        let cross: Vec<[i32; 2]> = walks(host, &(0..n).map(|e| [xp[e][0], 0, xp[e][1], shift[e], 0, 0]).collect::<Vec<_>>());
        let deck: Vec<i32> = (0..n).flat_map(|e| [cross[e][0], cross[e][1], tenths[e], hit(e).1 as i32 - 1]).collect();
        let p12 = host.go1(&m.phase, &[&[fr.phase[0], fr.phase[1], n as i32], &deck], 2 * n);
        let p3 = host.go1(&m.phase2, &[&[fr.phase[2], n as i32], &deck], n);
        let deck: Vec<i32> = (0..n).flat_map(|e| [p12[2 * e], p12[2 * e + 1], p3[e]]).collect();
        let sums = host.go1(&m.sine, &[&[n as i32], &deck], n);
        let deck: Vec<i32> = (0..n).flat_map(|e| [sums[e], far_fog[2 * e]]).collect();
        let glint = host.go1(&m.glint, &[&[fr.light10, n as i32], &deck], n);
        // the glowing spots: the spot under the crossing (if any), its falloff and pulse; the lips' bands
        let kinds: Vec<[i32; 2]> = (0..n).map(|e| { let o = host.go(&m.spot, &[&[&[cross[e][0]], &map.spots[hit(e).1 - 1]]], Some(2)); [o[0], o[1]] }).collect();
        let glowing: Vec<usize> = (0..n).filter(|&e| kinds[e][0] > 0).collect();
        let mut glow = vec![0; n];
        if !glowing.is_empty() {
            let g = glowing.len() as i32;
            let deck: Vec<i32> = glowing.iter().flat_map(|&e| [cross[e][1], tenths[e], kinds[e][1]]).collect();
            let fall = host.go1(&m.fall, &[&[fr.pulse, g], &deck], 2 * glowing.len());
            let steps: Vec<i32> = (0..glowing.len()).map(|e| fall[2 * e + 1]).collect();
            let pulse = host.go1(&m.wave, &[&[g], &steps], glowing.len());
            let deck: Vec<i32> = (0..glowing.len()).flat_map(|e| [fall[2 * e], pulse[e]]).collect();
            for (e, v) in host.go1(&m.glow, &[&[fr.dark, g], &deck], glowing.len()).into_iter().enumerate() { glow[glowing[e]] = v; }
        }
        let deck: Vec<i32> = (0..n).flat_map(|e| [map.lip[hit(e).1 - 1], tenths[e]]).collect();
        let band = host.go1(&m.lip, &[&[fr.light, n as i32], &deck], n);
        // the colour: the rock's tone into sand, the light on it, the diver's lamp toward the same in full light, the
        // edge's light, the caustics, the glows, the fog
        for c in 0..3 {
            let mut head = vec![5];
            head.extend(rocks.iter().map(|r| r[c]));
            let stone = many(host, &m.look, &head, &tone, 1, 1);
            let mut head = vec![5];
            head.extend(ROCKS.iter().map(|r| r[c]));
            let full = many(host, &m.look, &head, &tone, 1, 1);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [sand[c], stone[e], rocky[e]]).collect();
            let base = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [0, base[e], light[e]]).collect();
            let shaded = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [SAND[c], full[e], rocky[e]]).collect();
            let base = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [0, base[e], light[e]]).collect();
            let lamplit = m.mix(host, &t);                     // the rock in full light, its own shading kept
            let t: Vec<[i32; 3]> = (0..n).map(|e| [shaded[e], pale[c], rim[e]]).collect();
            let edged = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [edged[e], caustics[c], glint[e]]).collect();
            let shone = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [shone[e], if kinds[e][0] == 2 { [160, 90, 200][c] } else { [60, 200, 160][c] }, glow[e]]).collect();
            let glowed = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [glowed[e], [70, 170, 160][c], band[e]]).collect();
            let banded = m.mix(host, &t);
            let t: Vec<[i32; 3]> = (0..n).map(|e| [banded[e], far[c], far_fog[2 * e + 1]]).collect();
            let fogged = m.mix(host, &t);
            // the diver's lamp last, so it cuts through the fog
            let t: Vec<[i32; 3]> = (0..n).map(|e| [fogged[e], lamplit[e], lamp[e]]).collect();
            for (e, v) in m.mix(host, &t).into_iter().enumerate() { rgb[y * w + hits[e]][c] = v; }
        }
        for (e, &x) in hits.iter().enumerate() { depth[y * w + x] = far_fog[2 * e]; crossed[y * w + x] = tenths[e]; }
    }
    plants(host, m, map, fr, far, &mut rgb, &mut depth);
    // the picture so far is the water the creatures are drawn over at 1536x768, its depths what hides them
    let (under_raw, mut under, under_depth) = (rgb.clone(), rgb.clone(), depth.clone());
    fish(host, m, fr, far, &mut rgb, &mut depth);
    snow(host, m, fr, specks, &mut rgb, &depth);
    let drawn = jellies(host, m, fr, &mut rgb, &depth);
    // the caption, the portal and the whiteout: on the picture, and on the water (the creatures' at 1536x768 where
    // nothing is drawn over it)
    let bufs: Vec<&mut Vec<[i32; 3]>> = if hd.is_some() { vec![&mut rgb, &mut under] } else { vec![&mut rgb] };
    let portal_rows = if fr.on > 0 { portal(host, m, fr, &rays, &settled, &crossed, start) } else { vec![] };
    for buf in bufs {
        if fr.caption >= 0 { lay_caption(host, m, fr, caption, buf); }
        lay_portal(host, m, fr, &portal_rows, buf);
        // the whiteout as the diver passes through
        if fr.white32 > 0 {
            let items: Vec<(usize, [i32; 3], i32)> = (0..w * h).map(|i| (i, [255; 3], fr.white32)).collect();
            for chunk in items.chunks(300) { blend(host, m, buf, chunk); }
        }
    }
    let Some(hd) = hd else { return (rgb, None) };
    // the creatures at 1536x768, over the water before the caption, portal and whiteout (under_raw)
    let mut portal_at: std::collections::HashMap<usize, Vec<(bool, [i32; 3], i32)>> = std::collections::HashMap::new();
    for r in &portal_rows {
        for (k, list) in r.iter().enumerate() { for &(i, c, a) in list { portal_at.entry(i).or_default().push((k == 2, c, a)); } }
    }
    // the creatures at 1536x768, over the water
    let quarter = &demokit::tables::QUARTER_SINE60;
    let sin60: Vec<i32> = (0..60).map(|k: usize| (match k { 0..=15 => quarter[k], 16..=29 => quarter[30 - k], 30..=45 => -quarter[k - 30], _ => -quarter[60 - k] }) as i32).collect();
    let scene = deep_hd::Scene { n: fr.number as i32, cam: [fr.x, 100 * fr.y + fr.y_part, 100 * fr.z + 10 * fr.z_part], r3: fr.r, u3: fr.u, f3: fr.f, f: fr.f100, r: fr.r100, u: fr.u100,
        fish: fr.fish.iter().map(|f| (f.head, f.tail, f.flash, f.id)).collect(), fish_light: fr.fish_light, far, show: fr.show, snow: fr.snow,
        specks, jellies: drawn.iter().map(|&(a, fd, e)| { let j = &fr.jellies[e]; (a, fd, j.at, j.turn, j.kind, j.rad, j.ht, j.lengths, hd.first[j.id as usize]) }).collect(),
        wave: fr.wave, sin60: &sin60, under: &under_raw, depth: &under_depth, base: &under,
        caption: fr.caption, dots: caption, portal: &portal_at, on: fr.on, white32: fr.white32 };
    let frame_hd = deep_hd::frame(host_hd, &hd.m, &hd.d, &scene);
    (rgb, Some(frame_hd))
}
