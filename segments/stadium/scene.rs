// The stadium party's scene: its numbers, the data (data/scene.bin and data/pictures.bin, from data/make_data.py) and
// the printed tables. Included after the train's ops.rs.

/// The film's frames: 1536 x 768, the focal length 1000 pixels (75 degrees across).
pub const FULL_W: i64 = 1536;
pub const FULL_H: i64 = 768;
pub const FULL_NPIX: usize = (FULL_W * FULL_H) as usize;

// The screen being drawn: the film's (1536 x 768), or in the party version the world's half (768 x 384, the focal
// length halved: the same view). Set for the thread (`set_res`); everything that draws or walks pixels asks.
thread_local! { static RES: std::cell::Cell<(i64, i64, i64)> = const { std::cell::Cell::new((FULL_W, FULL_H, 1000)) }; }
/// Draw at the film's size (true) or half (false).
pub fn set_res(full: bool) { RES.with(|r| r.set(if full { (FULL_W, FULL_H, 1000) } else { (FULL_W / 2, FULL_H / 2, 500) })); }
pub fn is_full() -> bool { scr_w() == FULL_W }
pub fn scr_w() -> i64 { RES.with(|r| r.get().0) }
pub fn scr_h() -> i64 { RES.with(|r| r.get().1) }
pub fn npix() -> usize { (scr_w() * scr_h()) as usize }
/// The focal length, pixels.
pub fn focal() -> i64 { RES.with(|r| r.get().2) }
/// Depths as K / z (z in centimetres): a long, 999,000 at the near plane (15 cm).
pub const KZ: i64 = 14_985_000;
pub const NEAR: i64 = 15;
/// The night arrival's dissolve into ours (its last frames before the hand-off), then our film from the crest
/// (t = 3 s) to its end (47.25 s; with the weave through the crowd): 44 + 531 frames. The show's clock in twelfths of a second is our frame less 8.
pub const DISSOLVE: i64 = 44;
pub const FRAMES: i64 = DISSOLVE + 531;
/// The picture cuts to black 3 s before the end (t = 44.25 s).
pub const CUT_T12: i64 = 531;

/// Named arrays of whole numbers (or bytes), as make_data.py writes them.
pub struct Bin(pub std::collections::BTreeMap<String, (Vec<usize>, Vec<i64>)>, pub std::collections::BTreeMap<String, (Vec<usize>, Vec<u8>)>);

impl Bin {
    pub fn read(path: &str) -> Bin {
        let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e} (make it with segments/stadium/data/make_data.py)"));
        let mut at = 0;
        let u32_ = |at: &mut usize| -> u32 { let v = u32::from_le_bytes(b[*at..*at + 4].try_into().unwrap()); *at += 4; v };
        let mut m = std::collections::BTreeMap::new();
        let mut bytes = std::collections::BTreeMap::new();
        while at < b.len() {
            let n = u32_(&mut at) as usize;
            let name = String::from_utf8(b[at..at + n].to_vec()).unwrap();
            at += n;
            let kind = u32_(&mut at);
            let nd = u32_(&mut at);
            let shape: Vec<usize> = (0..nd).map(|_| u32_(&mut at) as usize).collect();
            let count: usize = shape.iter().product();
            if kind == 0 {
                let d = (0..count).map(|k| i32::from_le_bytes(b[at + 4 * k..at + 4 * k + 4].try_into().unwrap()) as i64).collect();
                at += 4 * count;
                m.insert(name, (shape, d));
            } else {
                bytes.insert(name, (shape, b[at..at + count].to_vec()));
                at += count;
            }
        }
        Bin(m, bytes)
    }
    pub fn get(&self, name: &str) -> &[i64] { &self.0.get(name).unwrap_or_else(|| panic!("no {name} in the stadium's data")).1 }
    pub fn shape(&self, name: &str) -> &[usize] { &self.0[name].0 }
    pub fn rows<const N: usize>(&self, name: &str) -> Vec<[i64; N]> { self.get(name).chunks(N).map(|c| c.try_into().unwrap()).collect() }
    pub fn take(&mut self, name: &str) -> (Vec<usize>, Vec<u8>) { self.1.remove(name).unwrap_or_else(|| panic!("no {name}")) }
}

/// A picture (bytes: rows of w, c channels).
pub struct Pic { pub w: usize, pub h: usize, pub c: usize, pub n: usize, pub px: Vec<u8> }

impl Pic {
    fn of(b: &mut Bin, name: &str) -> Pic {
        let (s, d) = b.take(name);
        let (c, w, h) = match s.len() { 2 => (1, s[1], s[0]), _ => (s[s.len() - 1], s[s.len() - 2], s[s.len() - 3]) };
        let n = d.len() / (w * h * c);
        Pic { w, h, c, n, px: d }
    }
    /// The byte at picture i, row y, column x, channel ch.
    pub fn at(&self, i: usize, y: usize, x: usize, ch: usize) -> i64 { self.px[((i * self.h + y) * self.w + x) * self.c + ch] as i64 }
    /// The same for a picture of signed numbers (an atlas's normals).
    pub fn ats(&self, i: usize, y: usize, x: usize, ch: usize) -> i64 { self.px[((i * self.h + y) * self.w + x) * self.c + ch] as i8 as i64 }
}

/// A list of triangles over shared corners.
pub struct Tris { pub v: Vec<[i64; 3]>, pub n: Vec<[i64; 3]>, pub t: Vec<[usize; 3]> }

impl Tris {
    fn read(b: &Bin, p: &str) -> Tris {
        Tris { v: b.rows(&format!("{p}_v")), n: b.rows(&format!("{p}_vn")), t: b.rows::<3>(&format!("{p}_t")).iter().map(|t| t.map(|x| x as usize)).collect() }
    }
}

/// A light at a place: colour (ten-thousandths), and its numbers.
#[derive(Clone, Debug)]
/// (`half_spread` and `gaps`, worked out when the scene is read: half of `power` as an angle in degrees, in millionths
/// of a turn (a laser fan's half spread), and `par` less 1 (the gaps between a fan's beams).)
pub struct Light { pub pos: [i64; 3], pub rgb: [i64; 3], pub phase: i64, pub par: i64, pub mode: i64, pub power: i64, pub half_spread: i64, pub gaps: i64 }

/// The dancers' poses: per model the triangles, per (move, phase) the corners, normals, lift.
pub struct Pose { pub t: Vec<[usize; 3]>, pub nv: usize, pub v: Vec<i64>, pub n: Vec<i64>, pub lift: Vec<i64>, pub ndotv: Vec<i64>, pub su: Vec<i64>, pub sv: Vec<i64>, pub sy: Vec<i64> }

pub struct Scene {
    pub stadium: Tris,
    pub st_mat: Vec<usize>,
    pub st_vprobe: Vec<usize>,
    pub mat_kind: Vec<i64>,
    pub sk_v: Vec<[i64; 3]>,
    pub sk_ang: Vec<[i64; 3]>,
    pub mat_alb: Vec<[i64; 3]>,
    pub mat_emit: Vec<[i64; 3]>,
    pub probes: Vec<[i64; 3]>,
    // the crowd
    pub cpos: Vec<[i64; 3]>,
    pub cface: Vec<[i64; 2]>,
    pub cmodel: Vec<i64>,
    pub cmove: Vec<i64>,
    pub cphase: Vec<i64>,
    pub cscale: Vec<i64>,
    pub ctone: Vec<[i64; 3]>,
    pub cglow: Vec<[i64; 3]>,
    pub cshirt: Vec<i64>,
    pub ckind: Vec<i64>,
    pub cang: Vec<i64>,
    pub cphone: Vec<i64>,
    pub cprobe: Vec<usize>,
    pub flag_pid: Vec<usize>,
    pub flag_tex: Vec<usize>,
    pub flag_ph: Vec<i64>,
    // the screens
    pub q_v: Vec<[i64; 3]>,
    pub q_t: Vec<[usize; 3]>,
    pub q_uv: Vec<[i64; 6]>,
    pub q_kind: Vec<i64>,
    pub q_aux: Vec<i64>,
    pub screen: [i64; 5],
    pub screen_parts: Vec<[i64; 3]>,
    pub airship_z: Vec<i64>,
    pub led_walls: Vec<[i64; 5]>,
    // the lights
    pub spots: Vec<Light>,
    pub search: Vec<Light>,
    pub lasers: Vec<Light>,
    pub pyro: Vec<[i64; 3]>,
    // the people
    pub poses: Vec<Pose>,
    pub hand: Vec<[i64; 4]>,
    // the things in the air, the city, the stars
    pub bal: Vec<[i64; 6]>,
    pub bal_c: Vec<[i64; 3]>,
    pub conf: Vec<[i64; 5]>,
    pub conf_c: Vec<[i64; 3]>,
    pub city: Vec<[i64; 3]>,
    pub city_c: Vec<[i64; 3]>,
    pub stars: Vec<[i64; 3]>,
    pub star_b: Vec<i64>,
    pub dstars: Vec<[i64; 3]>,
    pub dstar_l: Vec<i64>,
    pub dstar_big: Vec<i64>,
    // the airship
    pub blimp: Tris,
    pub bl_mat: Vec<i64>,
    pub bl_ab: Vec<[i64; 2]>,
    pub bl_probes: Vec<[i64; 3]>,
    pub bl_vprobe: Vec<usize>,
    pub bl_lights: Vec<[i64; 3]>,
    pub bl_beacons: Vec<[i64; 3]>,
    pub bl_at: [i64; 5],
    pub bl_screen: [i64; 6],
    // the pictures
    pub atlas: Pic,
    pub atlas_hi: Pic,
    pub demos: Pic,
    pub captions: Pic,
    pub vcaptions: Pic,
    pub flags: Pic,
    pub shirts: Pic,
    pub thanks_f: Pic,
    pub thanks_h: Pic,
    pub cardiac: Pic,
    pub airship_txt: Pic,
    pub airship_a: Pic,
    pub airship_b: Pic,
    pub marquee: Pic,
    // the callouts (callouts_final.json): each one's clip (0..7, its footage in `demos`) or -1 (a parody); each clip's
    // callout; the parodies' stand-ins (kind, hue), the starfields' stars, the stand-ins' printed numbers
    // who is in the crowd drawn (32,768 of the 65,536), the light probes (make_data.py) and the maps to them
    pub cparty: Vec<bool>,
    pub probes_hd: Vec<[i64; 3]>,
    pub st_vprobe_hd: Vec<usize>,
    pub cprobe_hd: Vec<usize>,
    pub bl_probes_hd: Vec<[i64; 3]>,
    pub bl_vprobe_hd: Vec<usize>,
    pub demo_clip: Vec<i64>,
    pub clip_entry: Vec<usize>,
    pub standin: Vec<[i64; 2]>,
    pub sd_stars: Vec<Vec<[i64; 3]>>,
    pub sd: SdTabs,
}

/// The stand-ins' numbers (pictures.rs): per texel of 80 x 60 the plasma's places, the tunnel's depth, angle and
/// fade, the place in fiftieths and squares; the printed tables.
pub struct SdTabs { pub pl: [V; 4], pub tu: [V; 3], pub xy: [V; 4], pub sin64: Tab, pub half: Tab, pub tun: Tab, pub par: Tab, pub rot: Tab, pub rsq: Tab, pub ring: Tab, pub sq: Tab, pub pal: [Tab; 3], pub vb: Tab, pub wrap2000: Tab }

pub const MOVES: usize = 8;
pub const NPHASE: i64 = 12;
pub const RAISE: i64 = 2;

pub fn read_scene(dir: &str) -> Scene {
    let b = Bin::read(&format!("{dir}/stadium/data/scene.bin"));
    let mut p = Bin::read(&format!("{dir}/stadium/data/pictures.bin"));
    let light = |r: &[i64], kind: u8| -> Light {
        let (power, phase, mode, par) = match kind { 0 => (1000, r[6], 0, r[7]), 1 => (r[6], r[7], r[8], r[9]), _ => (r[7], 0, 0, r[6]) };
        Light { pos: [r[0], r[1], r[2]], rgb: [r[3], r[4], r[5]], phase, par, mode, power, half_spread: rad((power as f64).to_radians() / 2.0), gaps: par - 1 }
    };
    let rows_of = |name: &str, kind: u8| -> Vec<Light> { let w = b.shape(name)[1]; b.get(name).chunks(w).map(|r| light(r, kind)).collect() };
    let mut poses = vec![];
    for m in 0..5 {
        let t: Vec<[usize; 3]> = b.rows::<3>(&format!("pm{m}_t")).iter().map(|t| t.map(|x| x as usize)).collect();
        let nv = b.shape(&format!("pm{m}_v"))[2];
        let g = |s: &str| b.get(&format!("pm{m}_{s}")).to_vec();
        poses.push(Pose { t, nv, v: g("v"), n: g("n"), lift: g("lift"), ndotv: g("nv"), su: g("su"), sv: g("sv"), sy: g("sy") });
    }
    let us = |name: &str| -> Vec<usize> { b.get(name).iter().map(|&x| x as usize).collect() };
    Scene {
        stadium: Tris::read(&b, "st"),
        st_mat: us("st_mat"),
        st_vprobe: us("st_vprobe"),
        mat_kind: b.get("mat_kind").to_vec(),
        sk_v: b.rows("sk_v"),
        sk_ang: b.rows("sk_ang"),
        mat_alb: b.rows("mat_alb"),
        mat_emit: b.rows("mat_emit"),
        probes: b.rows("probes"),
        cpos: b.rows("cpos"),
        cface: b.rows("cface_cs"),
        cmodel: b.get("cmodel").to_vec(),
        cmove: b.get("cmove").to_vec(),
        cphase: b.get("cphase").to_vec(),
        cscale: b.get("cscale").to_vec(),
        ctone: b.rows("ctone"),
        cglow: b.rows("cglow"),
        cshirt: b.get("cshirt").to_vec(),
        ckind: b.get("ckind").to_vec(),
        cang: b.get("cang").to_vec(),
        cphone: b.get("cphone").to_vec(),
        cprobe: us("cprobe"),
        flag_pid: us("flag_pid"),
        flag_tex: us("flag_tex"),
        flag_ph: b.get("flag_ph").to_vec(),
        q_v: b.rows("q_v"),
        q_t: b.rows::<3>("q_t").iter().map(|t| t.map(|x| x as usize)).collect(),
        q_uv: b.rows("q_uv"),
        q_kind: b.get("q_kind").to_vec(),
        q_aux: b.get("q_aux").to_vec(),
        screen: b.get("screen").try_into().unwrap(),
        screen_parts: b.rows("screen_parts"),
        airship_z: b.get("airship_z").to_vec(),
        led_walls: b.rows("led_walls"),
        spots: rows_of("spots", 0),
        search: rows_of("search", 1),
        lasers: rows_of("lasers", 2),
        pyro: b.rows("pyro"),
        poses,
        hand: b.rows("atlas_hand"),
        bal: b.rows("bal"),
        bal_c: b.rows("bal_c"),
        conf: b.rows("conf"),
        conf_c: b.rows("conf_c"),
        city: b.rows("city"),
        city_c: b.rows("city_c"),
        stars: b.rows("stars"),
        star_b: b.get("star_b").to_vec(),
        dstars: b.rows("dstars"),
        dstar_l: b.get("dstar_l").to_vec(),
        dstar_big: b.get("dstar_big").to_vec(),
        blimp: Tris::read(&b, "bl"),
        bl_mat: b.get("bl_mat").to_vec(),
        bl_ab: b.rows("bl_ab"),
        bl_probes: b.rows("bl_probes"),
        bl_vprobe: us("bl_vprobe"),
        bl_lights: b.rows("bl_lights"),
        bl_beacons: b.rows("bl_beacons"),
        bl_at: b.get("blimp").try_into().unwrap(),
        bl_screen: b.get("bl_screen").try_into().unwrap(),
        atlas: Pic::of(&mut p, "atlas"),
        atlas_hi: Pic::of(&mut p, "atlas_hi"),
        demos: Pic::of(&mut p, "demos"),
        captions: Pic::of(&mut p, "captions"),
        vcaptions: Pic::of(&mut p, "vcaptions"),
        flags: Pic::of(&mut p, "flags"),
        shirts: Pic::of(&mut p, "shirts"),
        thanks_f: Pic::of(&mut p, "thanks_f"),
        thanks_h: Pic::of(&mut p, "thanks_h"),
        cardiac: Pic::of(&mut p, "cardiac"),
        airship_txt: Pic::of(&mut p, "airship_txt"),
        airship_a: Pic::of(&mut p, "airship_a"),
        airship_b: Pic::of(&mut p, "airship_b"),
        marquee: Pic::of(&mut p, "marquee"),
        cparty: b.get("cparty").iter().map(|&x| x == 1).collect(),
        probes_hd: b.rows("probes_hd"),
        st_vprobe_hd: us("st_vprobe_hd"),
        cprobe_hd: us("cprobe_hd"),
        bl_probes_hd: b.rows("bl_probes_hd"),
        bl_vprobe_hd: us("bl_vprobe_hd"),
        demo_clip: b.get("demo_clip").to_vec(),
        clip_entry: (0..b.get("demo_clip").len()).filter(|&d| b.get("demo_clip")[d] >= 0).collect(),
        standin: b.rows("standin"),
        sd_stars: b.get("sd_stars").chunks(900).map(|c| c.chunks(3).map(|r| [r[0], r[1], r[2]]).collect()).collect(),
        sd: {
            let rows = |n: &str, k: usize| -> Vec<V> { b.get(n).chunks(b.get(n).len() / k).map(|c| c.to_vec()).collect() };
            let t = |name: &'static str, w: usize| Tab { name, data: b.get(&format!("sd_{name}")).to_vec(), w };
            let pal = b.get("sd_pal");
            SdTabs {
                pl: rows("sd_pl", 4).try_into().unwrap(), tu: rows("sd_tu", 3).try_into().unwrap(), xy: rows("sd_xy", 4).try_into().unwrap(),
                sin64: t("sin64", 0), half: t("half", 0), tun: t("tun", 0), par: t("par", 0), rot: t("rot", 0), rsq: t("rsq", 0), ring: t("ring", 64), sq: t("sq", 0), vb: t("vb", 0),
                pal: std::array::from_fn(|c| Tab { name: "sd_pal", data: (0..pal.len() / 3).map(|i| pal[3 * i + c]).collect(), w: 256 }),
                wrap2000: Tab { name: "wrap2000", data: (0..4000).map(|i| i % 2000 + 50).collect(), w: 0 },
            }
        },
    }
}

// Which light probes and crowd the frame uses: the full version's or the party version's (coarser probes, 32,768
// people; set for the thread).
thread_local! { static LIGHT_HD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub fn set_light_hd(hd: bool) { LIGHT_HD.with(|l| l.set(hd)); }
pub fn light_hd() -> bool { LIGHT_HD.with(|l| l.get()) }

impl Scene {
    /// The light probes in use, and the stadium's corners', the people's and the airship's corners' maps to them.
    pub fn probes(&self) -> &[[i64; 3]] { if light_hd() { &self.probes_hd } else { &self.probes } }
    pub fn st_vprobe(&self) -> &[usize] { if light_hd() { &self.st_vprobe_hd } else { &self.st_vprobe } }
    pub fn cprobe(&self) -> &[usize] { if light_hd() { &self.cprobe_hd } else { &self.cprobe } }
    pub fn bl_probes(&self) -> &[[i64; 3]] { if light_hd() { &self.bl_probes_hd } else { &self.bl_probes } }
    /// Whether person i is in the crowd drawn (the party version's: 32,768 of them, make_data.py).
    pub fn in_crowd(&self, i: usize) -> bool { !light_hd() || self.cparty[i] }
    pub fn bl_vprobe(&self) -> &[usize] { if light_hd() { &self.bl_vprobe_hd } else { &self.bl_vprobe } }
}
