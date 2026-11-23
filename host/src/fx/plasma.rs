//! Plasma at 192x96 in exact colour, as four 96x48 tiles: four waves added up at every pixel, the colours turning
//! round a ring of eight. The plasma lies on a plane that stands facing the camera, and over the last two seconds
//! tilts back about its foot until it is the floor the ray-traced ball bounces on (the ray's plasma floor carries
//! on from here, see rayplasma). Each frame a clock machine sets where every wave starts. Per tile row: a row
//! machine (told the tilt) finds each pixel's point (u, v) on the plane, or sky or haze, and its fog; a ring
//! machine walks to each point to find its distance from the rings' centre; the rings machine, the wave machine
//! twice (across, rolling) and the slant machine add the four waves; a paint machine per red, green and blue turns
//! the total into colour between the ring's colours, and a tone machine per channel fogs it or puts the sky in.
use crate::*;
use std::path::Path;

/// The plasma's frames; the last 48 tilt the plane back.
pub const FRAMES: usize = 204;
const TILT: usize = 48;

/// The machines that paint the plasma on its plane, a tile row at a time.
pub struct Plasma { row: Machine, ring: Machine, rings: Machine, wave: Machine, slant: Machine, paint: Vec<Machine>, tone: Vec<Machine> }

/// Decks interleaved: a card from each in turn.
pub fn weave(decks: &[&[i32]]) -> Vec<i32> { (0..decks[0].len()).flat_map(|i| decks.iter().map(move |d| d[i])).collect() }

/// The fixed colours: the sky's three bands and the haze, the sky high and low and the haze seen in the ball, the light.
const FIXED: [[i32; 3]; 8] = [[40, 60, 150], [80, 120, 200], [150, 180, 230], [170, 180, 200], [110, 150, 220], [170, 195, 235], [150, 160, 180], [255, 255, 255]];

impl Plasma {
    pub fn new(demo: &Path) -> Self {
        let ring = tables::rows(&read(demo, "plasma/data/ring.txt"));
        Self::with_keys(demo, &std::array::from_fn(|k| [ring[6 * k][0], ring[6 * k][1], ring[6 * k][2]]))
    }

    /// The painter on another ring of eight colours (Mode X's sign paints the plasma silver).
    pub fn with_keys(demo: &Path, keys: &[[i32; 3]; 8]) -> Self {
        let r = |n: &str| rust(demo, n);
        // per channel: the ring's eight colours and the first again; the fixed colours
        let paint = (0..3).map(|c| carrying(demo, "pl_paint", "keys", &(0..9).map(|k| keys[k % 8][c]).collect::<Vec<_>>())).collect();
        let tone = (0..3).map(|c| carrying(demo, "pl_tone", "palette", &FIXED.map(|p| p[c]))).collect();
        Plasma { row: r("pl_row"), ring: r("pl_ring"), rings: r("pl_rings"), wave: r("pl_wave"), slant: r("pl_slant"), paint, tone }
    }

    /// One tile row's points: per pixel its look, u and v. Row y of the frame, the tile's place across, the tilt (600
    /// cos, 100 sin), the shadow (ball x, radius, darkness).
    pub fn points(&self, host: &mut Host, y: i32, tile_x: i32, tilt: [i32; 2], shadow: [i32; 3]) -> Vec<i32> {
        host.go1(&self.row, &[&[y, tile_x, tilt[0], tilt[1], shadow[0], shadow[1], shadow[2]]], 3 * W)
    }

    /// One tile row's places on the colour ring, from its points and the frame's clock: the four waves and the
    /// colours' start added up (0..511; the paint machine takes them mod 256).
    pub fn places(&self, host: &mut Host, clock: &[i32], u: &[i32], v: &[i32]) -> Vec<i32> {
        let (across, rolling, slanting, rings, colours) = (clock[0], clock[1], clock[2], clock[3], clock[4]);
        let r = host.go1(&self.ring, &[&weave(&[u, v])], W);
        let sum = host.go1(&self.rings, &[&[rings, colours], &r], W);
        let sum = host.go1(&self.wave, &[&[across], &weave(&[u, &sum])], W);
        let sum = host.go1(&self.wave, &[&[rolling], &weave(&[v, &sum])], W);
        host.go1(&self.slant, &[&[slanting], &weave(&[u, v, &sum])], W)
    }

    /// One tile row's colours, red, green and blue a pixel, from its looks and points and the frame's clock.
    pub fn paint(&self, host: &mut Host, clock: &[i32], looks: &[i32], u: &[i32], v: &[i32]) -> Vec<i32> {
        let sum = self.places(host, clock, u, v);
        let channels: Vec<Vec<i32>> = (0..3).map(|c| {
            let plasma = host.go1(&self.paint[c], &[&sum], W);
            host.go1(&self.tone[c], &[&weave(&[looks, &plasma])], W)
        }).collect();
        weave(&[&channels[0], &channels[1], &channels[2]])
    }

    /// One channel's colours for up to a row of places on the ring (the deck padded to the row).
    pub fn colours(&self, host: &mut Host, c: usize, places: &[i32]) -> Vec<i32> {
        let mut deck = places.to_vec();
        deck.resize(W, 0);   // the paint machine takes a whole row
        let mut out = host.go1(&self.paint[c], &[&deck], W);
        out.truncate(places.len());
        out
    }

    /// One channel's tone of a row: its looks and their colours.
    pub fn tone(&self, host: &mut Host, c: usize, looks: &[i32], colours: &[i32]) -> Vec<i32> {
        host.go1(&self.tone[c], &[&weave(&[looks, colours])], W)
    }
}

/// Every third card from the k-th: one of the looks, u or v.
pub fn third(cards: &[i32], k: usize) -> Vec<i32> { cards.iter().skip(k).step_by(3).copied().collect() }

pub fn render(demo: &Path, refbin: &str) {
    let plasma = Plasma::new(demo);
    let clock = rust(demo, "pl_clock");
    let script = script(demo);
    let mut host = Host::new();
    let mut video = Video2::new(FRAMES);
    for f in live::frames(FRAMES) {
        let tilt = script[f];
        let times = host.go1(&clock, &[&[f as i32]], 5);
        for (tx, ty) in [(0, 0), (W as i32, 0), (0, H as i32), (W as i32, H as i32)] {
            for y in 0..H as i32 {
                let points = plasma.points(&mut host, ty + y, tx, tilt, [0, 0, 0]);
                let rgb = plasma.paint(&mut host, &times, &third(&points, 0), &third(&points, 1), &third(&points, 2));
                let at = ((f * H2 + (ty + y) as usize) * W2 + tx as usize) * 3;
                for (i, &v) in rgb.iter().enumerate() { video.buf[at + i] = v as u8; }
            }
        }
    }
    host.done();
    finish("plasma", &host, &video, &reference(refbin, &[&demo.join("plasma/data/ring.txt"), &demo.join("plasma/data/script.txt")]), &demo.join("plasma/video.rgb"), "");
}

/// The script: per frame the plane's tilt as 600 cos and 100 sin (the references read it from plasma/data/script.txt,
/// which is kept in the repository; a render that checks its frames stops if the machines made another). It stands
/// upright, then over the last 48 frames tilts back a quarter turn, easing in and out (3 s^2 - 2 s^3): how far
/// through on the tilt machine (pl_tilt, thousandths), the eased angle's cos and sin from the printed table
/// (data/tilt.txt) at that line (the ease was the host's, in floats).
pub fn script(demo: &Path) -> Vec<[i32; 2]> {
    let machine = rust(demo, "pl_tilt");
    let table: Vec<[i32; 2]> = read(demo, "plasma/data/tilt.txt").lines().filter(|l| !l.starts_with('#'))
        .map(|l| { let v: Vec<i32> = l.split_whitespace().map(|x| x.parse().unwrap()).collect(); [v[0], v[1]] }).collect();
    let mut host = Host::new();
    let script: Vec<[i32; 2]> = (0..FRAMES).map(|f| {
        let r = host.go1(&machine, &[&[f as i32, (FRAMES - TILT) as i32, (TILT - 1) as i32]], 2);
        table[if r[1] == 1 { 1000 } else { r[0] as usize }]
    }).collect();
    live::count(host.steps);
    std::fs::write(demo.join("plasma/data/script.txt"), script.iter().map(|s| format!("{} {}", s[0], s[1])).collect::<Vec<_>>().join("\n") + "\n").unwrap();
    script
}
