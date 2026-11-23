// What the screens show each frame: the giant screen (384 x 192: a montage of the 8 demos we may show, a different
// one every 2 beats, 4:3 with their captions, then THANK YOU D. W. HAGELBARGER, then the CARDIAC), the LED walls
// (120 x 220: a callout over its title), the banners round the bowl (160 x 100), the airship's screen (160 x 120:
// callouts and DEMOSPLASH 0x10 over its laser grid; 192 x 108 THANK YOU SAUL FINGERMAN in its hold). Which picture
// is worked out on CARDIAC (the bar, the slot's callout, the frame in the clip); a texel's bytes are fetched from the
// picture by its place (a nearest-neighbour blit), and the brightness is the shading's. A callout is one of the 8
// clips (its footage) or one of the 55 parodies: a stand-in effect (plasma, tunnel, stars, rotozoomer, rings) worked
// out on CARDIAC at 80 x 60, only at the texels some screen shows. Included after light.rs.

/// A screen's picture this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Src { Black, Demo(usize, usize), Hag, Cardiac, Fingerman, Grid }

pub struct Shows {
    pub main: Src,
    /// the main screen's caption's demo (for a Demo)
    pub leds: [Src; 2],
    pub banners: Vec<Src>,
    pub airship: Src,
    /// the airship's grid: which of its rows (120) are lit this frame
    pub grid_rows: Vec<bool>,
}

/// The pictures of a frame: which callout each screen shows (on_screen: callouts[(slot 3 + which 7) % 63]); the
/// giant screen's montage only the clips.
pub fn shows<O: Ops>(o: &mut O, sc: &Scene, sh: &Show, nbanners: usize) -> Shows {
    let t12 = sh.t12;
    let nd = sc.demo_clip.len() as i64;
    let nclip = sc.clip_entry.len() as i64;
    let frame = s_divmod(t12, 60, o).1 as usize;
    let slot = sh.slot;
    let on = |o: &mut O, which: i64| -> Src {
        let a = s_lin(&[(slot, Some(3)), (which, Some(7))], o);
        Src::Demo(s_divmod(a, nd, o).1 as usize, frame)
    };
    let main = if !s_lt(t12, CARDIAC_T, o) { Src::Cardiac } else if !s_lt(t12, HAG.0, o) { Src::Hag } else {
        // the montage: the 2-beat chunk on the beat's clock (from BEAT_T0; whole chunks added so it counts up
        // from zero before then), its clip round the 8 in order
        let a = s_lin(&[(t12, Some(C_2BEAT)), (nclip * 1_000_000 - beat_off(C_2BEAT), None)], o);
        let chunk = whole1m(a, o);
        Src::Demo(sc.clip_entry[s_divmod(chunk, nclip, o).1 as usize], frame)
    };
    let leds = [on(o, 1), on(o, 2)];
    let banners = (0..nbanners).map(|q| { let k = s_sum(&[(q as i64, 1)], 3, o); on(o, k) }).collect();
    let in_hold = !s_lt(t12, HOLD.0, o) && s_lt(t12, HOLD.1 + 1, o);
    let mut grid_rows = vec![false; 120];
    let airship = if in_hold { Src::Fingerman } else {
        let (s2, _) = s_divmod(t12, 30, o);
        if s_divmod(s2, 3, o).1 != 1 { let d = s_lin(&[(s2, Some(5))], o); Src::Demo(s_divmod(d, nd, o).1 as usize, frame) } else {
            // the grid's rows: (z + 30 t) mod 6 < 0.6 (thousandths of a metre-ish unit), z per row from the picture
            // (30 t in thousandths is 2,500 t12; the pattern repeats every 12 twelfths, so t12's twelfths are enough)
            let tt = s_lin(&[(s_divmod(t12, 12, o).1, Some(2_500))], o);
            for (y, &z) in sc.airship_z.iter().enumerate() {
                if z < 0 { continue; }
                let a = s_sum(&[(tt, 1)], z, o);
                let (_, r) = s_divmod(s_div(a, 10, o), 600, o);
                grid_rows[y] = s_lt(r, 60, o);
            }
            Src::Grid
        }
    };
    Shows { main, leds, banners, airship, grid_rows }
}

/// The sizes of the pictures (columns, rows).
pub const MAIN: (usize, usize) = (384, 192);
pub const LED: (usize, usize) = (120, 220);
pub const BANNER: (usize, usize) = (160, 100);

/// A texel's bytes (R, G, B): the blit from the picture each screen shows.
/// A callout's picture (160 x 120) at row y, column x: its clip's frame, or its stand-in's texel (80 x 60).
pub fn demo_px(sc: &Scene, sd: &Sd, d: usize, f: usize, y: usize, x: usize) -> [u8; 3] {
    let c = sc.demo_clip[d];
    if c >= 0 { return px3(&sc.demos, c as usize * 60 + f, y, x); }
    sd.get(d, (y / 2) * SD_W + x / 2)
}

pub fn main_texel(sc: &Scene, sd: &Sd, s: Src, x: usize, y: usize) -> [u8; 3] {
    match s {
        Src::Cardiac => px3(&sc.cardiac, 0, y, x),
        Src::Hag => px3(&sc.thanks_h, 0, y, x),
        Src::Demo(d, f) => {
            if (150..190).contains(&y) { return px3(&sc.captions, d, y - 150, x); }
            if !(64..320).contains(&x) { return [0; 3]; }
            // the 160 x 120 frame blown up to 256 x 192 (nearest)
            demo_px(sc, sd, d, f, (y * 120 + 60) / 192, ((x - 64) * 160 + 80) / 256)
        }
        _ => [0; 3],
    }
}

pub fn led_texel(sc: &Scene, sd: &Sd, s: Src, x: usize, y: usize) -> [u8; 3] {
    let Src::Demo(d, f) = s else { return [0; 3] };
    if (10..100).contains(&y) { return demo_px(sc, sd, d, f, ((y - 10) * 120 + 60) / 90, (x * 160 + 80) / 120); }
    if (110..210).contains(&y) { return px3(&sc.vcaptions, d, y - 110, x); }
    [0; 3]
}

pub fn banner_texel(sc: &Scene, sd: &Sd, s: Src, x: usize, y: usize) -> [u8; 3] {
    let Src::Demo(d, f) = s else { return [0; 3] };
    // the prototype's linspace picks
    if y < 75 && (30..130).contains(&x) { return demo_px(sc, sd, d, f, y * 119 / 74, (x - 30) * 159 / 99); }
    if (78..98).contains(&y) { return px3(&sc.captions, d, (y - 78) * 39 / 19, x * 383 / 159); }
    [0; 3]
}

pub fn airship_texel(sc: &Scene, sd: &Sd, s: Src, grid_rows: &[bool], a: usize, b: usize) -> [u8; 3] {
    match s {
        Src::Fingerman => px3(&sc.thanks_f, 0, b.min(107), a.min(191)),
        Src::Demo(d, f) => demo_px(sc, sd, d, f, b.min(119), a.min(159)),
        Src::Grid => { let b = b.min(119); px3(if grid_rows[b] { &sc.airship_b } else { &sc.airship_a }, 0, b, a.min(159)) }
        _ => [0; 3],
    }
}

fn px3(p: &Pic, i: usize, y: usize, x: usize) -> [u8; 3] { let k = ((i * p.h + y) * p.w + x) * p.c; [p.px[k], p.px[k + 1], p.px[k + 2]] }

/// The means of blocks of a picture (linear light, ten-thousandths): its texels' linear values summed along each
/// block's rows (runsum), the rows' sums over the block's width and summed again, over its height. `texel` gives the
/// bytes; blocks bw x bh over a picture w x h, left to right then down.
pub fn means<O: Ops>(o: &mut O, t: &Tables, w: usize, h: usize, bw: usize, bh: usize, texel: &dyn Fn(usize, usize) -> [u8; 3]) -> Vec<[i64; 3]> {
    let (nx, ny) = (w / bw, h / bh);
    let mut out = vec![[0; 3]; nx * ny];
    for c in 0..3 {
        // the linear values, row by row (the host's look-ups by the bytes)
        let bytes: V = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).map(|(x, y)| texel(x, y)[c] as i64).collect();
        let lin = o.tab(&t.srgb_th, v(&bytes));
        let rows: Vec<&[i64]> = lin.chunks(bw).collect();
        let rs = o.runsum(&rows);
        // each block-row's sum is its run's last; over the width
        let last: V = (0..rows.len()).map(|k| rs[k * bw + bw - 1]).collect();
        let rm = o.div(v(&last), K(bw as i64), 0);
        // the block-rows of each block, one after another, summed again, over the height
        let mut lists: Vec<Vec<i64>> = vec![vec![]; nx * ny];
        for y in 0..h { for bx in 0..nx { lists[(y / bh) * nx + bx].push(rm[y * nx + bx]); } }
        let refs: Vec<&[i64]> = lists.iter().map(|l| l.as_slice()).collect();
        let s2 = o.runsum(&refs);
        let tot: V = (0..nx * ny).map(|b| s2[b * bh + bh - 1]).collect();
        let m = o.div(v(&tot), K(bh as i64), 0);
        let m = o.lin(&[(v(&m), Some(K(10)))]);
        for b in 0..nx * ny { out[b][c] = m[b]; }
    }
    out
}

// ---- the parodies' stand-ins ------------------------------------------------------------------------------------

pub const SD_W: usize = 80;
pub const SD_H: usize = 60;

/// The stand-ins' texels worked out this frame: (callout, texel) to its bytes.
/// Or, asking (`asking`), the texels the screens' fetches would reach, noted (and black for now).
#[derive(Default)]
pub struct Sd { pub px: std::collections::HashMap<(usize, usize), [u8; 3]>, pub asking: Option<std::cell::RefCell<Vec<(usize, usize)>>> }

impl Sd {
    pub fn get(&self, d: usize, q: usize) -> [u8; 3] {
        if let Some(a) = &self.asking { a.borrow_mut().push((d, q)); return [0; 3]; }
        *self.px.get(&(d, q)).unwrap_or_else(|| panic!("stand-in {d} texel {q} not worked out"))
    }
    pub fn ask() -> Sd { Sd { px: Default::default(), asking: Some(Default::default()) } }
    pub fn asked(self) -> Vec<(usize, usize)> { self.asking.map(|a| a.into_inner()).unwrap_or_default() }
}

/// A phase at t12 for a rate of rad_s radians a second: 256ths of a turn (millionths of a turn by `angle`, times 4
/// over 15,625).
fn sd_phase<O: Ops>(o: &mut O, t12: i64, rad_s: f64) -> i64 { let a = angle(t12, rate(rad_s), 0, o); s_div(s_lin(&[(a, Some(4))], o), 15_625, o) }

/// Works out the stand-ins' texels asked for (callout, texel) at t12, adding them to `sd`: each one's value 0..255
/// by its effect, then its colour from the palette (by its hue and the value). The effects (the prototype's
/// `standin_values`), a few sums and printed tables a texel:
/// - plasma: four sines (0..63 each) at the texel's places along x, y, the diagonal and out from the middle, each
///   moved by its phase this frame, summed;
/// - tunnel: the rings (half on, half off) at the texel's depth moved by the frame's, the spokes at its angle moved,
///   on where one of them is, faded toward the middle;
/// - stars: 300 stars at their depths this frame (wrapped), each projected (its place over the depth, a quotient),
///   bright by its depth; the texel's star's (0 for none);
/// - rotozoomer: the texel's place turned and zoomed (two products each way, the frame's cosine and sine times the
///   zoom), its cell's parity each way, a checkerboard;
/// - rings: the squared distance to each of two moving centres, the ring's sine at it (one moving), their product.
pub fn standins<O: Ops>(o: &mut O, sc: &Scene, t12: i64, need: &[(usize, usize)], sd: &mut Sd) {
    let mut want: Vec<(usize, usize)> = need.iter().copied().filter(|k| !sd.px.contains_key(k)).collect();
    want.sort();
    want.dedup();
    if want.is_empty() { return; }
    let t = &sc.sd;
    let mut vals: Vec<(usize, usize, i64)> = vec![];
    for kind in 0..5i64 {
        let it: Vec<(usize, usize)> = want.iter().copied().filter(|&(d, _)| sc.standin[d][0] == kind).collect();
        if it.is_empty() { continue; }
        let q: Vec<usize> = it.iter().map(|k| k.1).collect();
        let at = |a: &V| -> V { q.iter().map(|&k| a[k]).collect() };
        let v: V = match kind {
            0 => {
                let ph = [sd_phase(o, t12, 2.0), sd_phase(o, t12, -1.5), sd_phase(o, t12, 1.0), sd_phase(o, t12, -3.0)];
                let s: Vec<V> = (0..4).map(|k| { let i = o.sum(&[(v(&at(&t.pl[k])), 1)], ph[k]); o.tab(&t.sin64, v(&i)) }).collect();
                o.sum(&[(v(&s[0]), 1), (v(&s[1]), 1), (v(&s[2]), 1), (v(&s[3]), 1)], 0)
            }
            1 => {
                let (pd, pa) = (sd_phase(o, t12, 6.0 * std::f64::consts::PI), sd_phase(o, t12, std::f64::consts::PI));
                let bd = { let i = o.sum(&[(v(&at(&t.tu[0])), 1)], pd); o.tab(&t.half, v(&i)) };
                let ba = { let i = o.sum(&[(v(&at(&t.tu[1])), 1)], pa); o.tab(&t.half, v(&i)) };
                let i = o.sum(&[(v(&bd), 1), (v(&ba), 1), (v(&at(&t.tu[2])), 1)], 0);
                o.tab(&t.tun, v(&i))
            }
            2 => {
                // each starfield once (every star), then the texels' values
                let mut grid: std::collections::HashMap<usize, Vec<i64>> = Default::default();
                let m = { let x = s_lin(&[(t12, Some(50))], o); let qq = s_div(x, 2000, o); s_lin(&[(x, None), (qq, Some(-2000))], o) };
                let mut ds: Vec<usize> = it.iter().map(|k| k.0).collect();
                ds.dedup();
                for &d in &ds {
                    let st = &sc.sd_stars[d];
                    let z0: V = st.iter().map(|s| s[2]).collect();
                    let zz = { let c = s_sum(&[(m, -1)], 2000, o); o.sum(&[(v(&z0), 1)], c) };
                    let zq = o.tab(&t.wrap2000, v(&zz));
                    let xs: V = st.iter().map(|s| s[0]).collect();
                    let ys: V = st.iter().map(|s| s[1]).collect();
                    let sx = { let a = o.div(v(&xs), v(&zq), 0); o.sum(&[(v(&a), 1)], 40) };
                    let sy = { let a = o.div(v(&ys), v(&zq), 0); o.sum(&[(v(&a), 1)], 30) };
                    let b = o.tab(&t.vb, v(&zq));
                    let mut g = vec![0i64; SD_W * SD_H];
                    for k in 0..st.len() { if (0..SD_W as i64).contains(&sx[k]) && (0..SD_H as i64).contains(&sy[k]) { g[sy[k] as usize * SD_W + sx[k] as usize] = b[k]; } }
                    grid.insert(d, g);
                }
                it.iter().map(|&(d, qq)| grid[&d][qq]).collect()
            }
            3 => {
                let (sa, ca) = s_sincos(angle(t12, rate(0.7), 0, o), o);
                let (st, _) = s_sincos(angle(t12, rate(1.0), 0, o), o);
                let z = s_sum(&[(st, 1)], 1500, o);
                let cz = s_div(s_lin(&[(ca, Some(z))], o), 1000, o);
                let sz = s_div(s_lin(&[(sa, Some(z))], o), 1000, o);
                let (x50, y50) = (at(&t.xy[0]), at(&t.xy[1]));
                let p = { let nsz = s_neg(sz, o); let a = o.lin(&[(v(&x50), Some(K(cz))), (v(&y50), Some(K(nsz)))]); let a = o.div(v(&a), K(25_000), 0); o.tabo(&t.par, 64, v(&a)) };
                let w = { let a = o.lin(&[(v(&x50), Some(K(sz))), (v(&y50), Some(K(cz)))]); let a = o.div(v(&a), K(25_000), 0); o.tabo(&t.par, 64, v(&a)) };
                let i = o.sum(&[(v(&p), 1), (v(&w), 1)], 0);
                o.tab(&t.rot, v(&i))
            }
            _ => {
                let (st, _) = s_sincos(angle(t12, rate(1.0), 0, o), o);
                let s50 = s_div(s_lin(&[(st, Some(50))], o), 999, o);
                let ph = sd_phase(o, t12, -4.0);
                let x50 = at(&t.xy[0]);
                let ring = |o: &mut O, c: i64, y2: &V, p: Option<i64>| -> V {
                    let dx = o.sum(&[(v(&x50), 1)], c);
                    let d2 = o.tabo(&t.sq, 400, v(&dx));
                    let d2 = o.sum(&[(v(&d2), 1), (v(y2), 1)], 0);
                    let r = o.tab(&t.rsq, v(&d2));
                    let r = match p { Some(p) => o.sum(&[(v(&r), 1)], p), None => r };
                    o.tab(&t.sin64, v(&r))
                };
                let q1 = { let n = s_neg(s50, o); ring(o, n, &at(&t.xy[2]), Some(ph)) };
                let q2 = ring(o, s50, &at(&t.xy[3]), None);
                o.tab2(&t.ring, v(&q1), v(&q2))
            }
        };
        for (k, &(d, qq)) in it.iter().enumerate() { vals.push((d, qq, v[k])); }
    }
    let hue: V = vals.iter().map(|x| sc.standin[x.0][1]).collect();
    let val: V = vals.iter().map(|x| x.2).collect();
    let c: [V; 3] = std::array::from_fn(|c| o.tab2(&t.pal[c], v(&hue), v(&val)));
    for (k, &(d, qq, _)) in vals.iter().enumerate() { sd.px.insert((d, qq), [c[0][k] as u8, c[1][k] as u8, c[2][k] as u8]); }
}
