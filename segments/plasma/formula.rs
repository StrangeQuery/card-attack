// The plasma at 192x96 and the plane it is painted on, for the native references (the plasma's and the ray's):
// each pixel straight from the formula, in ordinary Rust. Included with #[path] next to demokit.
//
// Units: a plasma unit is a pixel of the flat plasma, 192x96. The plasma is a function of a point (u, v) on its
// plane and the time tf in frames. The plane stands on the floor (the ray's floor) with its foot on the line the
// bottom of the screen shows, and tilts back about it: tilted right back it is the floor.

/// Rows 0..95 of the frame: the horizon of the floor is row 36, the camera 120 pixels from the screen.
pub const HZ: i32 = 36;

/// One wave: 32 + 32 sin(2 pi k / n), 0..64, from a quarter table (n = 96 or 64).
pub fn wave(k: i32, n: i32) -> i32 {
    let q: Vec<i32> = if n == 96 { super::demokit::tables::PL_WAVE96.to_vec() } else { super::demokit::tables::PL_WAVE64.to_vec() }
        .iter().map(|&v| v as i32).collect();
    let k = k.rem_euclid(n);
    let (h, sign) = if k < n / 2 { (k, 1) } else { (k - n / 2, -1) };
    let j = if h > n / 4 { n / 2 - h } else { h };
    32 + sign * q[j as usize]
}

/// The distance from the rings' centre (0, 48), rounded to the nearest whole unit, halves up: the r with
/// r^2 - r <= n < r^2 + r, floor((sqrt(4 n + 1) + 1) / 2) (1 at the centre itself).
pub fn ring_distance(u: i32, v: i32) -> i32 {
    let n = u * u + (v - 48) * (v - 48);
    let mut k = 0;
    while (k + 1) * (k + 1) <= 4 * n + 1 { k += 1; }   // isqrt(4 n + 1)
    (k + 1) / 2
}

/// The plasma's place on the colour ring, 0..255, at (u, v) at time tf: rings spreading from the centre (64 units
/// apart), a wave sliding across, one rolling up the plane and a slanting one (96 units long), and the colours
/// turning with the clock.
pub fn plasma(u: i32, v: i32, tf: i32) -> i32 {
    let sum = wave(ring_distance(u, v) - 2 * tf, 64) + wave(u + 6 * tf, 96) + wave(v + 2 * tf, 96) + wave(u - v + 77 + 2 * tf, 96);
    (sum + 8 * tf).rem_euclid(256)
}

/// The colour at place q on the ring: eight colours (the ring's every sixth), 32 steps between each two, straight
/// lines between them, rounded down.
pub fn ring_colour(keys: &[[i32; 3]; 8], q: i32) -> [i32; 3] {
    let (k, j) = ((q / 32) as usize, q % 32);
    let (a, b) = (keys[k], keys[(k + 1) % 8]);
    [0, 1, 2].map(|c| a[c] + ((b[c] - a[c]) * j).div_euclid(32))
}

/// The fixed colours: 0..2 the sky's bands and 3 the haze, as the ray's; 4, 5 the sky high and low and 6 the haze
/// seen in the ball; 7 the light.
pub fn fixed(code: i32) -> [i32; 3] {
    [[40, 60, 150], [80, 120, 200], [150, 180, 230], [170, 180, 200], [110, 150, 220], [170, 195, 235], [150, 160, 180], [255, 255, 255]][code as usize]
}
/// The sky's band by the row, as the ray's sky: y / 2 under 6, 13, 19, else haze.
pub fn band(y: i32) -> i32 { let y = y / 2; if y < 6 { 0 } else if y < 13 { 1 } else if y < 19 { 2 } else { 3 } }

/// What a pixel shows: a fixed colour, or the plasma at (u, v) taken dark 32nds toward black (in the ball's
/// shadow, or seen in the ball) and then fog 32nds toward the haze.
#[derive(Clone, Copy)]
pub enum Look { Fixed(i32), Plasma { u: i32, v: i32, fog: i32, dark: i32 } }

/// The ball's shadow on the floor: centred under the ball's x (floor units, at 15 x / 4 plasma units across and
/// 105 along), its radius and how dark (32nds).
#[derive(Clone, Copy)]
pub struct Shadow { pub x: i32, pub radius: i32, pub dark: i32 }

/// The plane at the frame's tilt: cos 600 c and sin 100 s, rounded, from the script. Pixel (x, y): the row's
/// slope k5 = 600 c - 5 (36 - y) s / 100 (the second part cut toward zero), > 0 where the row meets the plane. At
/// or below 0 the row sees sky; under 60 the plane is lost in haze. Otherwise along the plane v = 600 (96 - y) / k5,
/// depth z = 120 + v s / 100, across u = (x - 96) z / 120 (all rounded down), fog (z - 240) / 7 up to 32, where it
/// is all haze. In the shadow if (u - 15 x / 4)^2 + (v - 105)^2 < radius^2.
pub fn plane(x: i32, y: i32, c600: i32, s100: i32, shadow: Option<Shadow>) -> Look {
    let g = HZ - y;
    let lean = if g >= 0 { 5 * g * s100 / 100 } else { -(5 * -g * s100 / 100) };
    let k5 = c600 - lean;
    if k5 <= 0 { return Look::Fixed(band(y)); }
    if k5 < 60 { return Look::Fixed(3); }
    let v = 600 * (96 - y) / k5;
    let z = 120 + v * s100 / 100;
    let fog = ((z - 240) / 7).clamp(0, 32);
    if fog >= 32 { return Look::Fixed(3); }
    let u = ((x - 96) * z).div_euclid(120);
    let dark = match shadow {
        Some(s) => { let (du, dv) = (u - (15 * s.x).div_euclid(4), v - 105); if du * du + dv * dv < s.radius * s.radius { s.dark } else { 0 } }
        None => 0,
    };
    Look::Plasma { u, v, fog, dark }
}

/// A pixel's colour: the plasma, taken toward black, then toward the haze (each a + (b - a) L / 32 rounded down).
pub fn colour(keys: &[[i32; 3]; 8], look: Look, tf: i32) -> [i32; 3] {
    match look {
        Look::Fixed(code) => fixed(code),
        Look::Plasma { u, v, fog, dark } => {
            let mut c = ring_colour(keys, plasma(u, v, tf));
            let haze = fixed(3);
            for i in 0..3 {
                c[i] += (-c[i] * dark).div_euclid(32);
                c[i] += ((haze[i] - c[i]) * fog).div_euclid(32);
            }
            c
        }
    }
}

/// The ring's eight colours from ring.txt (every sixth line).
pub fn keys(ring: &str) -> [[i32; 3]; 8] {
    let rows: Vec<Vec<i32>> = ring.lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect();
    std::array::from_fn(|k| [rows[6 * k][0], rows[6 * k][1], rows[6 * k][2]])
}

/// Numbers from a script file, a row per frame.
pub fn script(path: &str) -> Vec<Vec<i32>> {
    std::fs::read_to_string(path).unwrap().lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect()
}
