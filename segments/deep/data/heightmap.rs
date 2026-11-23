// The dive's height map, printed once from the dive's floating-point seabed: rolling ridges, rocky
// bumps and a canyon; then terraces going down, steeper each time (drop k is 18 + 9k high over 24 units, the shelf
// before it 40 - 4k long, 20 at the least), rough, with the canyon's walls rising either side. One row per unit of
// z, 0..441; on each, x = -100..100 a unit apart. A row is its base (the terraces' height there, whole units), how
// far the base drops to the next row, how far it is past the nearest lip below the canyon's head (z over 140; 99
// when more than 2 units away), then the 201 heights over the base in tenths (within 999 either way), then the
// glowing spots on the seabed between it and the next row: how many, and for each its unit along x (-100..99) and
// its kind and pulse, 10 kind + pulse (kind 1 teal or 2 violet, pulse 0..9). The spots are scattered by a
// hash (one in 2.5 x 2.5 units on average), a unit apart.
//   rustc -O heightmap.rs && ./heightmap > map.txt
use std::fmt::Write as _;
fn smooth(e0: f64, e1: f64, x: f64) -> f64 { let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
fn lips() -> Vec<(f64, f64)> {
    let (mut z, mut h) = (115.0, -4.0); let mut v = vec![];
    for k in 0..40 { v.push((z, h)); h -= 18.0 + 9.0 * k as f64; z += 24.0 + (40.0 - 4.0 * (k + 1) as f64).max(20.0); }
    v
}
fn terraces(l: &[(f64, f64)], z: f64) -> f64 {
    if z < l[0].0 { return -4.0; }
    let k = l.partition_point(|p| p.0 <= z) - 1;
    let drop = l[k].1 - l[k + 1].1;
    l[k].1 - drop * smooth(l[k].0, l[k].0 + 24.0, z)
}
fn height(l: &[(f64, f64)], x: f64, z: f64) -> f64 {
    let ridges = 3.0 * (x * 0.11 + (z * 0.05).sin() * 2.0).sin() + 2.0 * (z * 0.09 + x * 0.04).sin();
    let rock = 1.4 * ((x * 0.25).sin() * (z * 0.21).cos()).abs() + 0.5 * ((x * 0.45 + z * 0.3).sin() * (z * 0.5 - x * 0.2).cos());
    let canyon = -14.0 * smooth(30.0, 120.0, z) * (-(x * x) / 120.0).exp();
    let rough = smooth(140.0, 175.0, z) * (1.6 * ((x * 0.55 + z * 0.2).sin() * (z * 0.62 - x * 0.3).cos()) + 1.1 * ((x * 0.9).sin() * (z * 0.85).sin()).abs() + 0.8 * (z * 1.3 + x * 0.4).sin());
    let walls = smooth(120.0, 160.0, z) * 0.014 * x * x;
    let slope = terraces(l, z) + 0.6 * ridges + rock + rough + 0.01 * x * x + walls;
    let here = ridges + rock + canyon;
    here + (slope - here) * smooth(85.0, 100.0, z) + blocks(x, z)
}
/// Rock: a ragged unit-by-unit roughness and blocky outcrops scattered over the seabed, a few in the sand up top and
/// more, taller and wider, down the canyon (whole units of x and z; the machines snap the seabed to whole units).
fn blocks(x: f64, z: f64) -> f64 {
    let rocky = smooth(70.0, 115.0, z);
    let (xi, zi) = (x.round() as i64, z.round() as i64);
    let mut h = (hash((xi + 100).div_euclid(3) + 300, zi.div_euclid(3) + 500) - 0.5) * 1.2 * rocky;
    for (size, chance, top) in [(7i64, 0.22 + 0.3 * rocky, 1.0 + 2.0 * rocky), (13, 0.15 + 0.35 * rocky, 1.0 + 2.5 * rocky)] {
        let (cx, cz) = ((xi + 100).div_euclid(size), zi.div_euclid(size));
        let key = cx * 7 + size;
        if hash(key, cz + 1000) >= chance { continue; }
        let w = 1 + (hash(key, cz + 2000) * (size as f64 * 0.55)) as i64;
        let d = 1 + (hash(key, cz + 3000) * (size as f64 * 0.45)) as i64;
        let ox = (hash(key, cz + 4000) * (size - w) as f64) as i64;
        let oz = (hash(key, cz + 5000) * (size - d) as f64) as i64;
        let (lx, lz) = ((xi + 100).rem_euclid(size) - ox, zi.rem_euclid(size) - oz);
        if lx >= 0 && lx < w && lz >= 0 && lz < d {
            let step = 1.0 + (hash(key, cz + 6000) * top).floor();
            // a broken top: a corner a unit lower now and then
            let notch = if hash(xi + 7, zi + 9) < 0.25 { 1.0 } else { 0.0 };
            h += step - notch * (step > 1.0) as i32 as f64;
        }
    }
    h
}
fn hash(a: i64, b: i64) -> f64 { (((a.wrapping_mul(73856093) ^ b.wrapping_mul(19349663)).wrapping_mul(2654435761) & 0xffff) as f64) / 65536.0 }
fn main() {
    let l = lips();
    let base: Vec<i32> = (0..443).map(|j| (terraces(&l, j as f64) * smooth(85.0, 100.0, j as f64)).round() as i32).collect();
    let mut s = String::new();
    for j in 0..442 {
        let z = j as f64;
        let row: Vec<i32> = (-100..=100).map(|x| ((height(&l, x as f64, z) - base[j] as f64) * 10.0).round().clamp(-999.0, 999.0) as i32).collect();
        assert!(row.windows(2).all(|w| (w[1] - w[0]).abs() <= 999), "row {j}: neighbours too far apart for the samplers");
        let near = l.iter().filter(|p| p.0 > 140.0).map(|p| j as i32 - p.0 as i32).min_by_key(|d| d.abs()).unwrap();
        write!(s, "{} {} {}", base[j], base[j] - base[j + 1], if near.abs() <= 2 { near } else { 99 }).unwrap();
        for v in &row { write!(s, " {v}").unwrap(); }
        let spots: Vec<(i64, i32)> = (-100..100i64).filter(|&x| hash(x, j as i64) < 0.32 / 6.25)
            .map(|x| (x, 10 * if hash(j as i64, x) < 0.5 { 1 } else { 2 } + (hash(x, j as i64 + 7) * 10.0) as i32)).collect();
        write!(s, " {}", spots.len()).unwrap();
        for (x, code) in spots { write!(s, " {x} {code}").unwrap(); }
        s.push('\n');
    }
    print!("{s}");
}
