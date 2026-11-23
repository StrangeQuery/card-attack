// The train's scene: the data (data/scene.bin, from data/make_data.py) and the numbers of the ride.
// Included by render.rs's users after ops.rs.

// The resolution the renderer draws at: 1536x768 over RES (1, or 2 for the night's party version's world). Set by
// the caller between renders (one render at a time). Per thread: the live player's workers run the train's and the
// night's jobs side by side, so one worker's half resolution must not reach another's.
thread_local! { static RES_: std::cell::Cell<i64> = const { std::cell::Cell::new(1) }; }
pub fn set_res(r: i64) { RES_.with(|c| c.set(r)) }
pub fn res() -> i64 { RES_.with(|c| c.get()) }
#[allow(non_snake_case)]
pub fn W() -> i64 { 1536 / res() }
#[allow(non_snake_case)]
pub fn H() -> i64 { 768 / res() }
pub const FRAMES: i64 = 256;
/// The departure (standing in a station, then pulling away) before the ride: the ride's frame 0 is our DEPART.
pub const DEPART: i64 = 84;
/// Standing this long (frames), then pulling away for the rest of the departure.
pub const HOLD: i64 = 12;
/// The ride's time (its frames) at our frame k: before 0 while departing, held while standing.
pub fn ride_t(k: i64) -> i64 { (k - DEPART).max(HOLD - DEPART) }
/// The focal length over 10 (950 pixels: 78 degrees across).
pub const F10: i64 = 95;
/// Depths as K / z (z in centimetres): a long, 999,000 at the nearest (30 cm).
pub const KZ: i64 = 29_970_000;
/// The same over 0.7 (ray directions are kept at 0.7 of their length, so each part fits a card).
pub const KZ7: i64 = 42_814_286;
/// Metres to centimetres, and the ride's places (centimetres).
pub const SPEED: i64 = 700;
pub const TUN0: i64 = 26_000;
pub const TUN1: i64 = 50_000;
pub const BR0: i64 = 92_000;
pub const BR1: i64 = 104_000;
pub const RIV0: i64 = 92_800;
pub const RIV1: i64 = 103_200;
pub const BANK: i64 = 1000;
pub const X2: i64 = 430;
pub const MID: i64 = 215;
pub const MEET: i64 = 55_000;
pub const CARS: i64 = 16;
pub const CAR: i64 = 2500;
pub const EYE_Y: i64 = 300;
pub const LAND: i64 = -150;
pub const RIVER: i64 = -450;
pub const STREET: i64 = -650;
/// The near plane, millimetres.
pub const NEAR_MM: i64 = 300;
pub const GUARD_X: i64 = 960;
pub const GUARD_Y: i64 = 480;

/// Material kinds, as the prototype numbers them.
pub mod kind {
    pub const SKY: i64 = 0; pub const DECK: i64 = 1; pub const PADDY: i64 = 2; pub const WATER: i64 = 3; pub const BANK: i64 = 4;
    pub const STREET: i64 = 5; pub const EMBANK: i64 = 6;
    pub const BUILD: i64 = 10; pub const CONC: i64 = 11; pub const HILL: i64 = 12; pub const TUNNEL: i64 = 13; pub const LAMP: i64 = 14;
    pub const METAL: i64 = 15; pub const TRUSS: i64 = 16; pub const TRAIN: i64 = 17; pub const TREE: i64 = 18; pub const HOUSE: i64 = 19;
    pub const TILE: i64 = 20; pub const SIGN: i64 = 21; pub const WOOD: i64 = 22; pub const SHED: i64 = 23; pub const DARK: i64 = 24;
    pub const ROOFTOP: i64 = 25; pub const CHOUSE: i64 = 26; pub const SOLAR: i64 = 27; pub const ASPHALT: i64 = 28;
    // the departure's station
    pub const PLATFORM: i64 = 40; pub const LED: i64 = 41; pub const TUBE: i64 = 42; pub const FENCE: i64 = 43;
}

/// A number of a data record, stored in 1, 2 or 4 bytes (segments/shared/records.py).
fn stored_number(b: &[u8]) -> i64 {
    match b.len() { 1 => b[0] as i8 as i64, 2 => i16::from_le_bytes([b[0], b[1]]) as i64, _ => i32::from_le_bytes(b.try_into().unwrap()) as i64 }
}

/// Named arrays of whole numbers, as make_data.py writes them.
pub struct Bin(pub std::collections::BTreeMap<String, (Vec<usize>, Vec<i64>)>);

impl Bin {
    pub fn read(path: &str) -> Bin {
        let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e} (make it with segments/train/data/make_data.py)"));
        let mut at = 0;
        let u32_ = |at: &mut usize| -> u32 { let v = u32::from_le_bytes(b[*at..*at + 4].try_into().unwrap()); *at += 4; v };
        let mut m = std::collections::BTreeMap::new();
        while at < b.len() {
            let n = u32_(&mut at) as usize;
            let name = String::from_utf8(b[at..at + n].to_vec()).unwrap();
            at += n;
            // the count of dimensions, with the bytes per number in its second byte (0 for 4)
            let nd = u32_(&mut at);
            let w = match nd >> 8 { 0 => 4, w => w as usize };
            let shape: Vec<usize> = (0..nd & 255).map(|_| u32_(&mut at) as usize).collect();
            let count: usize = shape.iter().product();
            let data: Vec<i64> = (0..count).map(|k| stored_number(&b[at + w * k..at + w * (k + 1)])).collect();
            at += w * count;
            m.insert(name, (shape, data));
        }
        Bin(m)
    }
    pub fn get(&self, name: &str) -> &[i64] { &self.0.get(name).unwrap_or_else(|| panic!("no {name} in scene.bin")).1 }
    pub fn rows<const N: usize>(&self, name: &str) -> Vec<[i64; N]> {
        self.get(name).chunks(N).map(|c| c.try_into().unwrap()).collect()
    }
}

/// A list of triangles: corners shared by place (centimetres), each triangle's corners, corner normals and face
/// normal (hundredths), its plane's distance (hundredths of a centimetre) and material.
pub struct Mesh {
    pub corners: Vec<[i64; 3]>,
    pub tris: Vec<[usize; 3]>,
    pub vn: Vec<[i64; 9]>,
    pub fnorm: Vec<[i64; 3]>,
    pub fd: Vec<i64>,
    pub mat: Vec<usize>,
    /// whether its corners' normals differ (carried across it) or are its face's
    pub smooth: Vec<bool>,
}

impl Mesh {
    pub fn read(b: &Bin, p: &str) -> Mesh {
        let vn: Vec<[i64; 9]> = b.rows(&format!("{p}vn"));
        let smooth = vn.iter().map(|n| n[0..3] != n[3..6] || n[0..3] != n[6..9]).collect();
        Mesh {
            corners: b.rows(&format!("{p}corners")),
            tris: b.rows::<3>(&format!("{p}tris")).iter().map(|t| [t[0] as usize, t[1] as usize, t[2] as usize]).collect(),
            vn,
            fnorm: b.rows(&format!("{p}fn")),
            fd: b.get(&format!("{p}fd")).to_vec(),
            mat: b.get(&format!("{p}mat")).iter().map(|&m| m as usize).collect(),
            smooth,
        }
    }
}

pub struct Mat { pub kind: i64, pub alb: [i64; 3], pub par: [i64; 6] }

pub struct Scene {
    pub world: Mesh,
    pub train: Mesh,
    pub mats: Vec<Mat>,
    /// what the prototype selects the world's triangles by (metres)
    pub zlo: Vec<i64>, pub zhi: Vec<i64>, pub xmid: Vec<i64>, pub small: Vec<bool>, pub lod: Vec<i64>, pub iz: Vec<i64>,
    pub poles: Vec<i64>,
    pub city_wires: Vec<Vec<i64>>,
    pub t: Tables,
    /// how many of the world's triangles are the ride's (the departure's come after)
    pub ride: usize,
}

/// The printed tables, and the ones worked out at the start (on CARDIAC: the S-curve's bend).
pub struct Tables {
    pub trig: Vec<i64>,
    /// K / 0.7 / i for i 1..9999 (centimetres, at most 999,999)
    pub recip: Tab,
    /// the S-curve's bend at each metre 0..2600 (centimetres), worked out at the start
    pub bend: Tab,
    /// the bend at every centimetre from -100,000 (to 1,000,000)
    pub bend_cm: Tab,
}

pub fn read_trig(dir: &str) -> Vec<i64> {
    let text = std::fs::read_to_string(format!("{dir}/swarm/data/tables.txt")).unwrap();
    let l = text.lines().find(|l| l.split_whitespace().next() == Some("trig")).unwrap();
    l.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect()
}

pub fn recip_table() -> Tab {
    let mut d = vec![999_999; 10_000];
    for (i, e) in d.iter_mut().enumerate().skip(1) { *e = (KZ7 / i as i64).min(999_999); }
    Tab { name: "recip", data: d, w: 0 }
}
