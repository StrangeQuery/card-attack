// Boom, the shooter the cold open starts on, and the two windows behind it, as data: the game's window, its view and
// status bar, its script frame by frame (the prototype macdoom.py's, and macboomcrash.py's crash: the hall of mirrors'
// schedule, data/haywire.txt), the map, the textures, the dither's thresholds,
// the Wikipedia page's sphere and the Finnish flag. Shared by the host (fx/mac.rs, which works every frame out on
// CARDIAC) and the reference (which draws them in ordinary Rust). Nothing here is worked out: the camera's keyframes,
// which shot is fired when and which face shows are the script; where the camera is between them, every ray, every
// wall and every texel's dot are worked out from them.
#![allow(dead_code)]
use std::path::Path;

/// The game's window and its picture: the view (448 x 224) over the status bar (47 rows).
pub const GAME_RECT: [i16; 4] = [31, 34, 480, 325];
pub const VW: usize = 448;
pub const VH: usize = 224;
pub const SBH: usize = 47;
/// The game's frames: Boom plays on its own to HAYWIRE, then goes haywire (the hall of mirrors, worse and worse over
/// three seconds) to BOMB, the system error, its picture frozen behind the bomb box; at RESUME, back in the editor;
/// GAME_FRAMES (118) before the reminder. The Alarm Clock goes off at ALARM and the code scrolls a line at each SCROLLS.
pub const HAYWIRE: usize = 32;
pub const BOMB: usize = 68;
pub const RESUME: usize = 82;
pub const GAME_FRAMES: usize = 118;
pub const ALARM: usize = 106;
pub const SCROLLS: [usize; 2] = [106, 110];
/// The windows behind it: Wikipedia's Gouraud shading page, and under it Learn to Speak Finnish.
pub const WIKI_RECT: [i16; 4] = [24, 26, 496, 318];
pub const FINNISH_RECT: [i16; 4] = [70, 64, 420, 296];

/// A cell of the map is 64 texels; every place and distance of the game is in texels.
pub const CELL: i64 = 64;
pub const MAP_W: usize = 24;
pub const MAP_H: usize = 17;
/// The map's cells: 0 empty; 1 brick, 2 steel panels, 3 stone, 4 the door's jambs, 5 the door.
pub const DOOR: i64 = 5;

// ---- the script -------------------------------------------------------------------------------------------------

/// The camera's keyframes (texels; angles in millionths of a turn): down the corridor from X0 to the door at XD, through
/// it to XR turning to A_ROOM, on to XE, YE turning to A_END.
pub const X0: i64 = 115;
pub const XD: i64 = 493;
pub const XR: i64 = 832;
pub const XE: i64 = 934;
pub const Y0: i64 = 544;
pub const YE: i64 = 614;
pub const A_ROOM: i64 = 23_873;
pub const A_END: i64 = 159_155;
/// The run's phase a frame (0.6 radian), for the gun's bob; the door goes up from DOOR_AT over DOOR_FRAMES.
pub const PHASE: i64 = 95_493;
pub const DOOR_AT: i64 = 15;
pub const DOOR_FRAMES: i64 = 10;
/// The shots: a frame each.
pub const SHOTS: [usize; 3] = [7, 44, 50];
/// The gun's place in the view, before its bob and kick.
pub const GUN_X: i64 = 176;
pub const GUN_Y: i64 = 122;

/// Which leg of the camera's path a frame is on: down the corridor, at the door, through it, round in the room.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Leg { Run, Wait, Through, Turn }
pub fn leg(f: usize) -> Leg {
    match f { 0..=15 => Leg::Run, 16..=21 => Leg::Wait, 22..=36 => Leg::Through, _ => Leg::Turn }
}
/// How hard the gun bobs on each leg, in tenths.
pub fn bob(f: usize) -> i64 {
    match leg(f) { Leg::Run | Leg::Through => 10, Leg::Wait => 3, Leg::Turn => 7 }
}
/// The gun's kick (pixels) and its muzzle flash (0, 1 small, 2 big): the three frames after a shot.
pub fn kick_flash(f: usize) -> (i64, i64) {
    for s in SHOTS {
        if f == s { return (8, 2); }
        if f == s + 1 { return (5, 1); }
        if f == s + 2 { return (2, 0); }
    }
    (0, 0)
}
/// How many shots have been fired by frame f.
pub fn fired(f: usize) -> i64 { SHOTS.iter().filter(|&&s| f >= s).count() as i64 }
/// The face in the status bar: looking ahead, left, ahead, right, three quarters of a second each; grinning for four
/// frames from a shot.
pub fn face(f: usize) -> String {
    let look = [0, -1, 0, 1][(f / 9) % 4];
    let mood = if SHOTS.iter().any(|&s| s <= f && f < s + 4) { "grin" } else { "calm" };
    format!("face_{look}_{mood}")
}
/// The Alarm Clock flashing in the menu bar over the editor's last second.
pub fn clock(f: usize) -> bool { f >= ALARM && ((f - ALARM) / 3) % 2 == 0 }

/// The ammo counter gone to junk: its digits' places (from the bar's corner), three digits or one.
pub const JUNK_AT: [[i64; 3]; 2] = [[19, 37, 55], [37, 0, 0]];
/// The status bar in the junk frames: the face looks ahead, calm.
pub const JUNK_FACE: &str = "face_0_calm";

/// The hall of mirrors (data/haywire.txt, the prototype's schedule): the bands across the view (place, width), the torn
/// strips (their first row, how far they move across; TEAR rows each), and for each frame HAYWIRE..BOMB whether it comes
/// out clean, how many bands are spoiled, how far the smeared ones move, how many strips tear, the ammo's junk digits.
pub const TEAR: usize = 22;
pub struct Spoil { pub clean: bool, pub bands: usize, pub shift: usize, pub tears: usize, pub junk: Option<Vec<u8>> }
pub struct Haywire { pub bands: Vec<(usize, usize)>, pub tears: Vec<(usize, i64)>, pub frames: Vec<Spoil> }

pub fn haywire(demo: &Path) -> Haywire {
    let mut h = Haywire { bands: vec![], tears: vec![], frames: vec![] };
    for l in std::fs::read_to_string(demo.join("mac/data/haywire.txt")).unwrap().lines().filter(|l| !l.starts_with('#')) {
        let p: Vec<&str> = l.split(' ').collect();
        let n = |i: usize| p[i].parse::<i64>().unwrap();
        match p[0] {
            "band" => h.bands.push((n(2) as usize, n(3) as usize)),
            "tear" => h.tears.push((n(1) as usize, n(2))),
            "frame" => {
                assert_eq!(n(1) as usize, HAYWIRE + h.frames.len());
                let junk = if p[6] == "-" { None } else { Some(p[6].bytes().map(|b| b - b'0').collect()) };
                h.frames.push(Spoil { clean: n(2) == 1, bands: n(3) as usize, shift: n(4) as usize, tears: n(5) as usize, junk });
            }
            o => panic!("haywire.txt: {}", o),
        }
    }
    assert_eq!(h.frames.len(), BOMB - HAYWIRE);
    h
}

/// A band of the view spoiled in the hall of mirrors, by its number: kept from the picture before and smeared across,
/// one column streaked across it, or the walls behind the player drawn over it.
#[derive(Clone, Copy, PartialEq)]
pub enum Band { Smear, Streak, Behind }
pub fn band(j: usize) -> Band { match j % 3 { 0 => Band::Smear, 1 => Band::Streak, _ => Band::Behind } }

/// The status bar's counters: each digit's place (from the bar's corner) and what it shows; the ammo's two digits are
/// worked out (50 less the shots).
pub const AMMO_AT: [(i64, i64); 2] = [(28, 7), (46, 7)];
pub const FIXED_DIGITS: [(&str, i64, i64); 7] = [("big_1", 106, 7), ("big_0", 124, 7), ("big_0", 142, 7), ("big_pct", 160, 7),
    ("big_5", 323, 7), ("big_0", 341, 7), ("big_pct", 359, 7)];
pub const FACE_AT: (i64, i64) = (258, 8);

/// The 4x4 ordered dither (QuickDraw's greys): the step 0-15 of each place.
pub const BAYER: [[usize; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

// ---- the windows behind --------------------------------------------------------------------------------------

/// The page's text: a paragraph and the contents, a line every 13 rows.
pub const WIKI_TEXT: [&str; 10] = ["Gouraud shading, named after", "Henri Gouraud, is a way to", "shade a polygon mesh smoothly:",
    "light each vertex, then blend", "the colours across each face.", "", "Contents", " 1 Description", " 2 Comparison with Phong", " 3 History"];
/// The page's picture: a sphere of radius BALL_R (its picture 2 BALL_R + 1 square), lit from the upper left: the light
/// -0.5 x - 0.6 y + 0.62 z of a unit normal, held to 0..1, times 0.9, plus 0.08, as 7000ths of a unit (x, y and z the
/// pixel's place and its height over the radius, z its whole square root): BALL_X x + BALL_Y y + BALL_Z z.
pub const BALL_R: i64 = 70;
pub const BALL_X: i64 = -50;
pub const BALL_Y: i64 = -60;
pub const BALL_Z: i64 = 62;
/// Its dither: the light held to 0..7000, times 0.9, plus 0.08, under the step's middle (2k + 1) / 32 is c 288 < 70000
/// (2k + 1) - 179200: black where the light is under ceil((70000 (2k + 1) - 179200) / 288), never where that is not
/// above 0 (the light's hold at 7000 is above every step).
pub const BALL_DITHER: (i64, i64, i64) = (70_000, 179_200, 288);

/// The Finnish flag: 60 narrower than the window's inside, 18:11, the blue cross's bands in 18ths across (5-8) and
/// 11ths down (4-7), its top 14 under the title, cropped 8 above the window's foot.
pub const FLAG_IN: i64 = 60;
pub const FLAG_RATIO: (i64, i64) = (18, 11);
pub const FLAG_BANDS: [(i64, i64); 2] = [(5, 8), (4, 7)];
pub const FLAG_TOP: i64 = 91;
pub const FLAG_FOOT: i64 = 8;

// ---- the data ---------------------------------------------------------------------------------------------------

/// data/boom.txt: the map (rows of cells), the seven textures (`tex KIND ROW`: 64 rows of 64 greys, 256 white: the walls 1-5, the
/// floor 6, the ceiling 7), the thresholds (for each light 1-135, the 16 greys under which a texel is black at each
/// step of the dither).
pub struct Data { pub map: Vec<Vec<i64>>, pub tex: Vec<Vec<i64>>, pub thresh: Vec<[i64; 16]> }

pub fn data(demo: &Path) -> Data {
    let mut d = Data { map: vec![], tex: vec![vec![]; 8], thresh: vec![[0; 16]; 136] };
    for l in std::fs::read_to_string(demo.join("mac/data/boom.txt")).unwrap().lines().filter(|l| !l.starts_with('#')) {
        let p: Vec<&str> = l.split(' ').collect();
        let n: Vec<i64> = p[2..].iter().map(|v| v.parse().unwrap()).collect();
        match p[0] {
            "map" => d.map.push(n),
            "tex" => { let k: usize = p[1].parse().unwrap(); d.tex[k].extend(&n[1..]); }
            "thresh" => { let e: usize = p[1].parse().unwrap(); d.thresh[e].copy_from_slice(&n); }
            o => panic!("boom.txt: {}", o),
        }
    }
    assert!(d.map.len() == MAP_H && d.tex[1..].iter().all(|t| t.len() == 64 * 64));
    d
}
