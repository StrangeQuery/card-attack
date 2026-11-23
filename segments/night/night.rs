// The night arrival (the train's ending): the ride carries on past its last frame into the night, every number
// through the train's operations (segments/train/ops.rs), on its renderer. This file: the timeline, the night's
// world (data/scene.bin), the light of each frame, the beat. Included after the train's
// files; path.rs, nframe.rs, nmat.rs and npost.rs after it.

/// The night's material kinds (from 61; its platform is the train's).
pub mod nkind {
    pub const LED: i64 = 61; pub const NAV: i64 = 62; pub const DRONE: i64 = 63; pub const TUBE: i64 = 64; pub const SCREEN: i64 = 65;
    pub const TIER: i64 = 66; pub const PITCH: i64 = 67; pub const FENCE: i64 = 68; pub const STADWALL: i64 = 69; pub const EMITX: i64 = 70;
    pub const BHULL: i64 = 71; pub const BSCREEN: i64 = 72; pub const BFRAME: i64 = 73; pub const DSCREEN: i64 = 74; pub const STADSKIN: i64 = 75;
}

// ---- the timeline (frames of the night; its 0 is the frame after the train's last) --------------------------------

/// The hand-off to the stadium (its camera at its show clock 3 s): our frames are 0 .. F_HAND - 1.
pub const F_HAND: i64 = 373;
/// The design's timeline (the flight, the light, the airship, the stadium's show clock), which ran to its hand-off at
/// F_END: the film shows its frames up to the click to VOL 11 (VOL_OFF, the path's f_read + 20) as they are, then
/// skips CUT_N of them (the orbit round the airship: 7 bars at 137.14 BPM, so the beat runs on) straight into the
/// dive, the camera moving from the screen's read onto the dive's over JOIN frames.
pub const F_END: i64 = 520;
pub const CUT_N: i64 = 147;
pub const JOIN: i64 = 24;
pub const CRUISE: i64 = 24;
pub const DECEL: i64 = 96;
/// Stopped in the station.
pub const F_STOP: i64 = 120;
/// The dissolve from the cab to the platform, F_GO .. F_VIEW.
pub const F_GO: i64 = 130;
pub const F_VIEW: i64 = 142;
/// The drone swooping in, from F_BTN + 1.
pub const F_BTN: i64 = 134;
/// The chase: the flight from the platform, F_CHASE .. F_END (NCH frames, the design's).
pub const F_CHASE: i64 = 152;
pub const NCH: i64 = 368;
/// The first frames dissolve in from the ride's own world; the last are the stadium's dissolve (our layer of it).
pub const XFADE: i64 = 24;
pub const DISSOLVE: i64 = 44;

// ---- the layout (centimetres; the film's frame, unbent unless said) ------------------------------------------

/// The ride's last frame's cab, plus a frame; the S-curve's shift (everything here is 70 m over).
pub const Z0: i64 = 120_400;
pub const BX: i64 = 7000;
/// The cab's eye stopped (the braking's end, rounded), our nose's tip, the platforms, the town's
/// edge.
pub const ZS: i64 = 168_456;
pub const TIP: i64 = ZS + 640;
pub const BOARD_Z: i64 = ZS + 1400;
pub const PL0: i64 = ZS + 4200 - 20_000;
pub const PL1: i64 = ZS + 4200;
pub const TOWN0: i64 = PL0 - 23_000;

/// The flight's places (data's "places", film unbent, cm), by name.
pub mod place { pub const VIEW: usize = 0; pub const DRONE_HOVER: usize = 1; pub const DRONE_FROM: usize = 2; pub const HANDOFF: usize = 3; pub const HAND_LOOK: usize = 4;
    pub const READ: usize = 5; pub const ORBIT0: usize = 6; pub const ORBIT1: usize = 7; pub const TURN: usize = 8; pub const DRONE_AWAY: usize = 9; pub const BLIMP_AU: usize = 10; pub const BLIMP_SCREEN_AU: usize = 11; }

/// The footage the screens show: which demos, and their 60 frames of 160 x 120 RGB each.
pub struct Clips { pub demos: Vec<i64>, pub bytes: Vec<u8> }

impl Clips {
    pub fn read(path: &str) -> Clips {
        let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e} (make it with segments/night/data/make_data.py)"));
        let u = |k: usize| u32::from_le_bytes(b[4 * k..4 * k + 4].try_into().unwrap()) as i64;
        let n = u(0) as usize;
        let demos: Vec<i64> = (1..=n).map(u).collect();
        Clips { demos, bytes: b[4 * (n + 1)..].to_vec() }
    }
    /// Demo d's frame k (0..59) as a printed table of its 120 rows of 160 pixels' RGB (0..255).
    pub fn frame(&self, d: i64, k: i64) -> Tab {
        let i = self.demos.iter().position(|&x| x == d).unwrap_or_else(|| panic!("no footage of demo {d}"));
        let sz = 160 * 120 * 3;
        let at = (i * 60 + k as usize) * sz;
        Tab { name: "clip", data: self.bytes[at..at + sz].iter().map(|&x| x as i64).collect(), w: 480 }
    }
}

/// The night's world: the train's World with the night's scene (both worlds' triangles, flagged), what moves, what's
/// lit, the pictures, and the flight worked out at the start.
pub struct NightWorld {
    pub w: World,
    pub old: Vec<bool>, pub new: Vec<bool>, pub small_old: Vec<bool>,
    /// by material: drawn from both sides
    pub nocull: Vec<bool>,
    pub own: Mesh, pub drone: Mesh, pub blimp: Mesh,
    pub lamps: Vec<[i64; 3]>,
    /// the lamps in decimetres, and each one's tables of its parts times a normal's part (-127..127 hundredths)
    pub lamps_dm: Vec<[i64; 3]>, pub lamp_qn: Vec<[V; 3]>,
    /// the stadium's glow in the sky: its middle 45 m up (bent, mm)
    pub glow_at: [i64; 3],
    pub screens: Vec<[i64; 13]>,
    pub search: Vec<[i64; 8]>, pub spots: Vec<[i64; 7]>, pub lasers: Vec<[i64; 8]>, pub aud_c: [i64; 3],
    pub blights: Vec<[i64; 4]>, pub uplights: Vec<[i64; 4]>,
    /// each laser's beams' places across its fan (thousandths, -1000 to 1000), and each column's or row's middle in
    /// hundredths of a pixel (100 k + 50); worked out at the start
    pub laser_fan: Vec<V>, pub mid: Tab,
    pub balloons: Vec<[i64; 8]>, pub fw_shells: Vec<[i64; 7]>, pub fw_ways: Vec<[i64; 3]>, pub flashes: Vec<[i64; 7]>, pub stars: Vec<[i64; 4]>,
    pub path_keys: Vec<[i64; 3]>, pub places: Vec<[i64; 3]>,
    /// worked out at the start: the places in mm; the airship's centre at rest and its screen from it (film, mm); the
    /// hand-off's way (thousandths); the cab's way
    pub places_mm: Vec<[i64; 3]>, pub blimp0: [i64; 3], pub screen_off_mm: [i64; 3], pub hand_dir: [i64; 3], pub cab_dir: [i64; 3],
    pub pics: Bin,
    pub clips: Clips,
    pub path: Path,
    /// the screen's numbers for the lens's focal lengths (twice f: 1900 the ride's, then on to 2000)
    pub scrs: Vec<(i64, Screen)>,
    /// the same at half the resolution (the party version's world: twice f in its pixels, 950 on to 1000), and its
    /// blur's lines
    pub scrs_h: Vec<(i64, Screen)>, pub lines_h: BlurLines,
    /// the airship's hull: its albedo's factor by grime and seams ((700 + 0.6 noise) (100 - 0.4 seams) / 100), and for
    /// each of its materials its albedo and their mean by that factor (worked out at the start)
    pub hull_f: Tab, pub hull_mats: Vec<usize>, pub hull_a: Vec<[Tab; 3]>, pub hull_mean: Vec<Tab>,
    /// the tubes' tables: each line's x (dm) times a normal's part, and 7.6 m times one
    pub tube_qn: Vec<Tab>, pub tube_y: Tab,
    /// the dusk's shadows of the ride's own world (the frames the night dissolves in from)
    pub dusk_old: SMap,
    /// what the frames' triangles are picked by (nframe.rs)
    pub cull: NightCull,
}

pub fn night_load<O: Ops>(o: &mut O, dir: &str) -> NightWorld {
    let b = Bin::read(&format!("{dir}/night/data/scene.bin"));
    let kinds = b.get("m_kind");
    let alb: Vec<[i64; 3]> = b.rows("m_alb");
    let par: Vec<[i64; 6]> = b.rows("m_par");
    let mats = (0..kinds.len()).map(|i| Mat { kind: kinds[i], alb: alb[i], par: par[i] }).collect();
    let bend = bend_table(o);
    let mut t = Tables { trig: read_trig(dir), recip: recip_table(), bend, bend_cm: Tab { name: "bend_cm", data: vec![], w: 0 } };
    t.bend_cm = bend_cm_table(o, &t);
    let flag = |name: &str| -> Vec<bool> { b.get(name).iter().map(|&x| x != 0).collect() };
    let empty = Mesh { corners: vec![], tris: vec![], vn: vec![], fnorm: vec![], fd: vec![], mat: vec![], smooth: vec![] };
    let sc = Scene {
        world: Mesh::read(&b, "w_"), train: empty, mats,
        zlo: b.get("w_zlo").to_vec(), zhi: b.get("w_zhi").to_vec(), xmid: b.get("w_xmid").to_vec(),
        small: flag("w_small_new"), lod: b.get("w_lod").to_vec(), iz: b.get("w_iz").to_vec(),
        poles: b.get("poles").to_vec(), city_wires: vec![], t, ride: 0,
    };
    // the train's printed tables and the night's
    let mut tb = Tb::read(&format!("{dir}/train/data/tables.bin"));
    for (k, v) in Tb::read(&format!("{dir}/night/data/tables.bin")).0 { tb.0.insert(k, v); }
    let scr = screen(o, &tb);
    let lines = blur_lines(&tb);
    let wires = Wires { seg: b.rows("wires") };
    let ghosts = ghost_sprites(o);
    let none = |o: &mut O| -> SMap { let (r, u, f) = sun_frame(o, DUSK.sun); SMap { r, u, f, c: [0; 3], u0: 0, v1: 0, du: 1, dv: 1, w: 2, h: 2, d: vec![999_000; 4] } };
    let day = none(o);
    let depart = none(o);
    let blank = || SkyConst { hx: vec![], hz: vec![], lvl: vec![], base: [vec![], vec![], vec![]], a: vec![], b: vec![], c: vec![] };
    let skyk = SkyConsts { day: [blank(), blank()], dusk: [blank(), blank()] };
    let tun = TunnelMaps { lamps: Tab { name: "none", data: vec![], w: 0 }, head: Tab { name: "none", data: vec![], w: 0 } };
    let mut w = World { sc, tb, scr, tun, lines, wires, sh: Shadows { day, dusk: none(o), depart }, ghosts, skyk, scr_h: Screen { vlev: vec![], nlq7: vec![], lq: vec![] }, lines_h: BlurLines { dirs: vec![] }, cull: Cull::default() };
    w.cull.mirror = mirror_bounds(o, &w.sc);
    // the dusk's shadows: what stands past the tunnel in the night's world (the sun's way as the ride's)
    w.sh.dusk = dusk_map(o, &w, &flag("w_new"));
    let dusk_old = dusk_map(o, &w, &flag("w_old"));
    let pics = Bin(b.0.iter().filter(|(k, _)| is_picture(k)).map(|(k, v)| (k.clone(), v.clone())).collect());
    // the printed pictures as tables too (the marquee, the airship's words, the knob's parts)
    for (k, (shape, data)) in &pics.0 {
        let leaked: &'static str = Box::leak(k.clone().into_boxed_str());
        w.tb.0.insert(k.clone(), Tab { name: leaked, data: data.clone(), w: if shape.len() > 1 { shape[1] } else { 1 } });
    }
    let mut nw = NightWorld {
        old: flag("w_old"), new: flag("w_new"), small_old: flag("w_small_old"),
        nocull: flag("m_nocull"),
        own: Mesh::read(&b, "o_"), drone: Mesh::read(&b, "d_"), blimp: Mesh::read(&b, "b_"),
        lamps: b.rows("lamps"), screens: b.rows("screens"),
        search: b.rows("search"), spots: b.rows("spots"), lasers: b.rows("lasers"), aud_c: b.get("aud_c").try_into().unwrap(),
        blights: b.rows("blimp_lights"), uplights: b.rows("uplights"),
        balloons: b.rows("balloons"), fw_shells: b.rows("fw_shells"), fw_ways: b.rows("fw_ways"), flashes: b.rows("flashes"), stars: b.rows("stars"),
        path_keys: b.rows("path"), places: b.rows("places"),
        pics, clips: Clips::read(&format!("{dir}/night/data/clips.bin")),
        path: Path::default(), scrs: vec![], scrs_h: vec![], lines_h: BlurLines { dirs: vec![] },
        places_mm: vec![], blimp0: [0; 3], screen_off_mm: [0; 3], hand_dir: [0; 3], cab_dir: [0; 3],
        lamps_dm: vec![], lamp_qn: vec![], glow_at: [0; 3], laser_fan: vec![], mid: Tab { name: "mid", data: vec![], w: 0 },
        hull_f: Tab { name: "hull_f", data: vec![], w: 0 }, hull_mats: vec![], hull_a: vec![], hull_mean: vec![],
        tube_qn: vec![], tube_y: Tab { name: "tube_y", data: vec![], w: 0 }, dusk_old, cull: NightCull::default(),
        w,
    };
    nw.cull = night_cull(o, &nw);
    let ks: V = (0..1536).collect();
    nw.mid = Tab { name: "mid", data: o.lin(&[(v(&ks), Some(K(100))), (K(50), None)]), w: 0 };
    // the lasers' fans: beam j of n at -1000 + 2000 j / (n - 1)
    nw.laser_fan = nw.lasers.iter().map(|s_| {
        let nb = s_[6];
        let js: V = (0..nb).collect();
        let d = kl(o, &[(nb, None), (-1, None)]);
        let t = o.lin(&[(v(&js), Some(K(2000)))]);
        let t = o.div(v(&t), K(d), 0);
        o.lin(&[(v(&t), None), (K(-1000), None)])
    }).collect();
    // the tubes' tables
    {
        let parts: V = (-127..=127).collect();
        for xl in [-830, -395, 825, 1260] { let qx = (xl + BX) / 10; nw.tube_qn.push(Tab { name: "tube_qn", data: o.lin(&[(v(&parts), Some(K(qx)))]), w: 0 }); }
        nw.tube_y = Tab { name: "tube_y", data: o.lin(&[(v(&parts), Some(K(76)))]), w: 0 };
    }
    // the hull's tables
    {
        let nz: V = (0..1000).flat_map(|_| std::iter::empty()).collect::<V>();
        let _ = nz;
        let gk: V = (0..1000i64).flat_map(|i| std::iter::repeat_n(i, 101)).collect();
        let pk: V = (0..1000).flat_map(|_| 0..=100i64).collect();
        let g = o.lin(&[(v(&gk), Some(K(600))), (K(700_000), None)]);
        let g = o.div(v(&g), K(1000), 0);
        let pp = o.lin(&[(v(&pk), Some(K(-4))), (K(1000), None)]);
        let f = o.lin(&[(v(&g), Some(v(&pp)))]);
        let f = o.div(v(&f), K(1000), 0);
        nw.hull_f = Tab { name: "hull_f", data: f, w: 101 };
        let mut mats: Vec<usize> = nw.blimp.mat.iter().copied().filter(|&m| nw.w.sc.mats[m].kind == nkind::BHULL).collect();
        mats.sort(); mats.dedup();
        let fs: V = (0..1400).collect();
        for &m in &mats {
            let al = nw.w.sc.mats[m].alb;
            let a: [Tab; 3] = std::array::from_fn(|c| { let t = o.lin(&[(v(&fs), Some(K(al[c])))]); Tab { name: "hull_a", data: o.div(v(&t), K(1000), 0), w: 0 } });
            let s3 = o.sum(&[(v(&a[0].data), 1), (v(&a[1].data), 1), (v(&a[2].data), 1)], 0);
            let mean = o.div(v(&s3), K(3), 0);
            nw.hull_a.push(a);
            nw.hull_mean.push(Tab { name: "hull_mean", data: mean, w: 0 });
        }
        nw.hull_mats = mats;
    }
    let ld: [V; 3] = std::array::from_fn(|c| { let a: V = nw.lamps.iter().map(|p| p[c]).collect(); o.div(v(&a), K(10), 0) });
    nw.lamps_dm = (0..nw.lamps.len()).map(|k| [ld[0][k], ld[1][k], ld[2][k]]).collect();
    let parts: V = (-127..=127).collect();
    nw.lamp_qn = nw.lamps_dm.iter().map(|q| std::array::from_fn(|c| o.lin(&[(v(&parts), Some(K(q[c])))]))).collect();
    nw.glow_at = { let ac = nw.aud_c; [one(o.lin(&[(K(ac[0]), Some(K(10))), (K(10 * BX), None)])), one(o.lin(&[(K(ac[1]), Some(K(10))), (K(45_000), None)])), one(o.lin(&[(K(ac[2]), Some(K(10)))]))] };
    let pm: [V; 3] = std::array::from_fn(|c| { let a: V = nw.places.iter().map(|p| p[c]).collect(); o.lin(&[(v(&a), Some(K(10)))]) });
    nw.places_mm = (0..nw.places.len()).map(|i| [pm[0][i], pm[1][i], pm[2][i]]).collect();
    let (ac, bau, bsc) = (nw.aud_c, nw.places[place::BLIMP_AU], nw.places[place::BLIMP_SCREEN_AU]);
    // to_film: x = C.x + X, y = C.y + Z, z = C.z + Y
    for (c, a) in [(0, 0), (1, 2), (2, 1)] {
        nw.blimp0[c] = one(o.lin(&[(K(ac[c]), Some(K(10))), (K(bau[a]), Some(K(10)))]));
        nw.screen_off_mm[c] = one(o.lin(&[(K(bsc[a]), Some(K(10))), (K(bau[a]), Some(K(-10)))]));
    }
    let (hl, ho) = (nw.places_mm[place::HAND_LOOK], nw.places_mm[place::HANDOFF]);
    let hd: [V; 3] = std::array::from_fn(|c| o.lin(&[(K(hl[c]), None), (K(ho[c]), Some(K(-1)))]));
    let hd = nrm3(o, &hd[0], &hd[1], &hd[2]);
    nw.hand_dir = [hd[0][0], hd[1][0], hd[2][0]];
    let cd = nrm3(o, &vec![0], &vec![-50], &vec![1000]);
    nw.cab_dir = [cd[0][0], cd[1][0], cd[2][0]];
    nw.path = flight(o, &nw);
    // the lens's screens: the ride's (1900) and every 25 on to 2000
    nw.scrs = vec![(1900, screen_f(o, &nw.w.tb, &nw.w.scr, 1900))];
    for f2 in [1925, 1950, 1975, 2000] { let s = screen_f(o, &nw.w.tb, &nw.w.scr, f2); nw.scrs.push((f2, s)); }
    // at half the resolution (the screen's base by the full screen's distances, so its vignette is the same)
    set_res(2);
    let base = screen(o, &nw.w.tb);
    for f2 in [950, 962, 975, 987, 1000] { let s = screen_f(o, &nw.w.tb, &base, f2); nw.scrs_h.push((f2, s)); }
    nw.lines_h = blur_lines(&nw.w.tb);
    set_res(1);
    nw
}

fn is_picture(k: &str) -> bool {
    ["led", "marquee", "blimp_text"].contains(&k) || k.starts_with("banner_") || k.starts_with("cap") || k.starts_with("vol_") || k.starts_with("si_")
}

/// The dusk's shadow map of the night's world (the train's map: its place, size and sun), from what stands past the
/// tunnel.
fn dusk_map<O: Ops>(o: &mut O, w: &World, new: &[bool]) -> SMap {
    let sc = &w.sc;
    let (r, u, f) = sun_frame(o, DUSK.sun);
    let (u0, u1, v0, v1, du, dv) = (-45_000, 55_000, -8_000, 7_500, 20, 20);
    let (wd, ht) = ((u1 - u0) / du, (v1 - v0) / dv);
    let (wd, ht) = (wd + wd % 2, ht + ht % 2);
    let mut m = SMap { r, u, f, c: [MID + 4000, 0, 105_000], u0, v1, du, dv, w: wd, h: ht, d: vec![999_000; (wd * ht) as usize] };
    let far: Vec<[usize; 3]> = (0..sc.world.tris.len()).filter(|&t| new[t] && sc.world.tris[t].iter().any(|&q| sc.world.corners[q][2] > TUN1 - 1000)).map(|t| sc.world.tris[t]).collect();
    smap_raster(o, &w.tb, &mut m, &sc.world.corners, &far);
    m
}

/// The screen's numbers for a lens of focal length f2 / 2 (the ride's are for 1900): the ray's length at each pixel
/// over the forward part q = 1000 f2 / |(x, y, f2)| (half pixels), worked out for one quarter of the screen (the
/// others mirror it), and its logarithms from the printed tables by q; the vignette's level as the ride's.
pub fn screen_f<O: Ops>(o: &mut O, tb: &Tb, base: &Screen, f2: i64) -> Screen {
    if f2 == 1900 { return Screen { vlev: base.vlev.clone(), nlq7: base.nlq7.clone(), lq: base.lq.clone() }; }
    let (hw, hh) = ((W() / 2) as usize, (H() / 2) as usize);
    // the bottom right quarter's pixels: x 768.., y 384..: half-pixel offsets 2 i + 1
    let s2: V = (0..hh).flat_map(|j| (0..hw).map(move |i| { let (x, y) = (2 * i as i64 + 1, 2 * j as i64 + 1); x * x + y * y })).collect();
    let r2 = o.lin(&[(v(&s2), None), (K(f2), Some(K(f2)))]);
    let r = o.sqrt(v(&r2));
    let q = o.div(K(1000 * f2), v(&r), 0);
    let q = o.min(v(&q), K(999));
    let nl = o.tab(tb.t("scr_nlq7_q"), v(&q));
    let lq = o.tab(tb.t("scr_lq_q"), v(&q));
    let mut out = Screen { vlev: base.vlev.clone(), nlq7: vec![0; N_PIX()], lq: vec![0; N_PIX()] };
    for j in 0..hh { for i in 0..hw {
        let k = j * hw + i;
        for (x, y) in [(hw + i, hh + j), (hw - 1 - i, hh + j), (hw + i, hh - 1 - j), (hw - 1 - i, hh - 1 - j)] {
            let p = y * W() as usize + x;
            out.nlq7[p] = nl[k];
            out.lq[p] = lq[k];
        }
    } }
    out
}

// ---- small sums ----------------------------------------------------------------------------------------------------

/// The smoothstep of x from a to b (thousandths): t = (x - a) 1000 / (b - a) held to 0..1000, then t^2 (3000 - 2t)
/// over a million (in two steps). b - a under 100,000 in size.
pub fn smoothv<O: Ops>(o: &mut O, x: &V, a: i64, b: i64) -> V {
    if b >= a { return smoothk(o, x, -1000 * a, b - a); }
    let num = o.lin(&[(v(x), Some(K(-1000))), (K(1000 * a), None)]);
    let t = o.div(v(&num), K(a - b), 0);
    let t = o.clamp(v(&t), 0, 1000);
    ease_t(o, &t)
}

/// The smoothstep of x rising from a over den (thousandths), given its constant c = -1000 a.
pub fn smoothk<O: Ops>(o: &mut O, x: &V, c: i64, den: i64) -> V {
    let num = o.lin(&[(v(x), Some(K(1000))), (K(c), None)]);
    let t = o.div(v(&num), K(den), 0);
    let t = o.clamp(v(&t), 0, 1000);
    ease_t(o, &t)
}

/// The smoothstep of t (thousandths, 0..1000).
pub fn ease_t<O: Ops>(o: &mut O, t: &V) -> V {
    let t2 = o.lin(&[(v(t), Some(v(t)))]);
    let t2 = o.div(v(&t2), K(1000), 0);
    let k = o.lin(&[(v(t), Some(K(-2))), (K(3000), None)]);
    let s = o.lin(&[(v(&t2), Some(v(&k)))]);
    o.div(v(&s), K(1000), 0)
}

/// Thousandths of a radian to millionths of a turn, fl(x 159,155 / 1000) for any long x: x 159 + fl(x 155 / 1000).
pub fn mturns<O: Ops>(o: &mut O, x: &V) -> V {
    let t = o.lin(&[(v(x), Some(K(155)))]);
    let t = o.div(v(&t), K(1000), 0);
    o.lin(&[(v(x), Some(K(159))), (v(&t), None)])
}

pub fn smooth1<O: Ops>(o: &mut O, x: i64, a: i64, b: i64) -> i64 { one(smoothv(o, &vec![x], a, b)) }

/// a + (b - a) t / 1000 for longs and t thousandths (0..1000), rounded down.
pub fn lerpv<O: Ops>(o: &mut O, a: A, b: A, t: A) -> V {
    let d = o.lin(&[(b, None), (a, Some(K(-1)))]);
    let dd = if a.len().is_none() && b.len().is_none() { K(d[0]) } else { v(&d) };
    let m = o.lin(&[(dd, Some(t))]);
    let m = o.div(v(&m), K(1000), 0);
    o.lin(&[(a, None), (v(&m), None)])
}

/// e^-x for x thousandths (0 or more), thousandths (the printed table, to x = 20).
pub fn expn<O: Ops>(o: &mut O, tb: &Tb, x: &V) -> V {
    let x = o.clamp(v(x), 0, 19_999);
    o.tab(tb.t("expn"), v(&x))
}

// ---- the light: the sun going down, the dark coming -----------------------------------------------------------------

/// The sun's elevation at frame 0 (hundred-thousandths of a radian: asin 0.0945) and its fall a frame (stopped,
/// slower).
pub const EL0: i64 = 9468;

/// A frame's light: the sun's elevation (1e-5 rad), how far night has come, the light's going (k, kr), how dark it is
/// for the windows, the sky's deepening to violet (the chase), the exposure (thousandths), the beat (thousandths:
/// 1000 on beats 1 and 3, decaying); and the Env the train's shading takes (its colours in hundredths of R).
pub struct NightLight { pub f: i64, pub el: i64, pub night: i64, pub k: i64, pub win: i64, pub black: i64, pub e: i64, pub beat: i64, pub env: Env }

pub fn night_light<O: Ops>(o: &mut O, nw: &NightWorld, f: i64) -> NightLight {
    let tb = &nw.w.tb;
    // (the frame up to the stop and past it, on the machines)
    let (upto, past) = (one(o.clamp(K(f), -999_999, F_STOP)), { let d = kl(o, &[(f, None), (-F_STOP, None)]); one(o.clamp(K(d), 0, 999_999)) });
    let el = one(o.lin(&[(K(EL0), None), (K(upto), Some(K(-280))), (K(past), Some(K(-200)))]));
    let night = smooth1(o, el, 0, -16_000);
    let k = smooth1(o, el, -1500, 5000);
    let kr = smooth1(o, el, 0, 10_000);
    let win = smooth1(o, el, 2000, -13_000);
    // the sun's way: level along the dusk's (AZ, thousandths), raised by the elevation (its turns: el 10 / 2 pi)
    let ang = one({ let t = o.lin(&[(K(el), Some(K(1592)))]); o.div(v(&t), K(1000), 0) });
    let (s, c) = o.sincos(K(ang));
    let (s, c) = (s[0], c[0]);
    let sun = [one({ let t = o.lin(&[(K(-313), Some(K(c)))]); o.div(v(&t), K(999), 0) }), s, one({ let t = o.lin(&[(K(950), Some(K(c)))]); o.div(v(&t), K(999), 0) })];
    // its colour (hundredths of R): 2.6, 1.05 (0.45 + 0.55 kr), 0.35 kr, times k
    let c1 = one({ let t = o.lin(&[(K(kr), Some(K(550))), (K(450_000), None)]); o.div(v(&t), K(1000), 0) });
    let sc0 = [65_000, one({ let t = o.lin(&[(K(26_250), Some(K(c1)))]); o.div(v(&t), K(1000), 0) }), one({ let t = o.lin(&[(K(8750), Some(K(kr)))]); o.div(v(&t), K(1000), 0) })];
    let sun_col: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K(sc0[i]), Some(K(k)))]); o.div(v(&t), K(1000), 0) }));
    // the sky's light and the ground's, to the night's (0.010, 0.014, 0.032), the ground's half of it
    const NA: [i64; 3] = [250, 350, 800];
    let amb: [i64; 3] = std::array::from_fn(|i| one(lerpv(o, K([3500, 2750, 4250][i]), K(NA[i]), K(night))));
    let bounce: [i64; 3] = std::array::from_fn(|i| one(lerpv(o, K([2000, 1250, 1000][i]), K(NA[i] / 2), K(night))));
    // the haze: the dusk's, fading with the light, to the night's
    let hk = one({ let t = o.lin(&[(K(k), Some(K(700))), (K(300_000), None)]); o.div(v(&t), K(1000), 0) });
    let mut haze: [i64; 3] = std::array::from_fn(|i| {
        let a = one({ let t = o.lin(&[(K([27_500, 12_000, 5500][i]), Some(K(hk)))]); o.div(v(&t), K(1000), 0) });
        one(lerpv(o, K(a), K([625, 550, 1000][i]), K(night)))
    });
    // the sky deepening to violet over the chase (by the camera's way along it)
    let black = if f >= F_CHASE {
        let d = { let c = kl(o, &[(f, None), (-F_CHASE, None)]); let c = one(o.clamp(K(c), -999_999, NCH)); let t = kl(o, &[(c, Some(10))]); cam_dist(o, &nw.path, t) };
        let a = one(o.lin(&[(K(nw.path.d_orb0), None), (K(40_000), None)]));
        smooth_c(o, d, a, nw.path.d_orb1)
    } else { 0 };
    if black > 0 { for i in 0..3 { haze[i] = one(lerpv(o, K(haze[i]), K([625, 225, 750][i]), K(black))); } }
    let lk = one({ let t = o.lin(&[(K(black), Some(K(-700))), (K(1_000_000), None)]); o.div(v(&t), K(1000), 0) });
    let low: [i64; 3] = std::array::from_fn(|i| one({ let t = o.lin(&[(K([10_500, 6500, 10_000][i]), Some(K(lk)))]); o.div(v(&t), K(1000), 0) }));
    let e = one({ let t = o.lin(&[(K(night), Some(K(1100))), (K(900_000), None)]); o.div(v(&t), K(1000), 0) });
    let beat = beat_at(o, tb, f);
    // the windows' light: on the beat once it's dark (55% between)
    let win_k = if night > 300 { pulse(o, night, beat, 550) } else { 1000 };
    let env = Env { day: false, sun, sun_col, amb, bounce, haze, fog: "fog_dusk", low, night: None, unit: 100, win, win_k };
    NightLight { f, el, night, k, win, black, e, beat, env }
}

/// smooth(a, b, x) for numbers worked out (distances along the flight, mm: in tenths of a metre), either way round.
pub fn smooth_c<O: Ops>(o: &mut O, x: i64, a: i64, b: i64) -> i64 {
    let q = o.div(v(&vec![x, a, b]), K(100), 0);
    let den = one(o.lin(&[(K(q[2]), None), (K(q[1]), Some(K(-1)))]));
    if den == 0 { let f = o.lt(K(q[1]), K(q[0])); return one(o.sel(&f, K(1000), K(0))); }
    let neg = one(o.lt(K(den), K(0)));
    let (num, den) = if neg == 1 {
        (one(o.lin(&[(K(q[1]), Some(K(1000))), (K(q[0]), Some(K(-1000)))])), one(o.lin(&[(K(den), Some(K(-1)))])))
    } else { (one(o.lin(&[(K(q[0]), Some(K(1000))), (K(q[1]), Some(K(-1000)))])), den) };
    let t = o.div(K(num), K(den), 0);
    let t = o.clamp(v(&t), 0, 1000);
    one(ease_t(o, &t))
}

/// The design's frame for the film's frame f: f up to the click (VOL_OFF), then CUT_N on (on the machines).
pub fn design_f<O: Ops>(o: &mut O, nw: &NightWorld, f: i64) -> i64 { if f <= nw.path.f_cut { f } else { kl(o, &[(f, None), (CUT_N, None)]) } }

/// The soundtrack's beat is 0.378 s after the stadium's show clock's (the soundtrack's mix): thousandths of a frame (4.536).
pub const BEAT_T0: i64 = 4536;

/// The beat: 1000 on beats 1 and 3 of the soundtrack's 137.14 BPM (every other beat: 10.5 frames apart), on the
/// stadium's show clock (36 frames before the hand-off is its 0), decaying as e^-3 its phase.
pub fn beat_at<O: Ops>(o: &mut O, tb: &Tb, f: i64) -> i64 {
    let m = one(o.lin(&[(K(f), Some(K(1000))), (K(-1000 * (F_END - 36) - BEAT_T0), None)]));
    let q = one(o.div(K(m), K(10_500), 0));
    let r = one(o.lin(&[(K(m), None), (K(q), Some(K(-10_500)))]));
    let ph = one({ let t = o.lin(&[(K(r), Some(K(1000)))]); o.div(v(&t), K(10_500), 0) });
    let x = one(o.lin(&[(K(ph), Some(K(3)))]));
    one(expn(o, tb, &vec![x]))
}

/// The main lights' level (thousandths): from lo between beats to 1000 on them, once it's dark (pulse()).
pub fn pulse<O: Ops>(o: &mut O, night: i64, beat: i64, lo: i64) -> i64 {
    let a = one(o.lin(&[(K(night), Some(K(1000 - lo)))]));
    let a = one(o.div(K(a), K(1000), 0));
    let nb = one(o.lin(&[(K(beat), Some(K(-1))), (K(1000), None)]));
    let b = one(o.lin(&[(K(a), Some(K(nb)))]));
    let b = one(o.div(K(b), K(1000), 0));
    one(o.lin(&[(K(b), Some(K(-1))), (K(1000), None)]))
}

/// fl(a b, 10^s) for any numbers whose product is under 999,000,000 in size (lin, then over 10^s).
pub fn mulw<O: Ops>(o: &mut O, a: A, b: A, s: u32) -> V {
    let t = o.lin(&[(a, Some(b))]);
    if s == 0 { t } else { o.div(v(&t), K(10i64.pow(s)), 0) }
}
