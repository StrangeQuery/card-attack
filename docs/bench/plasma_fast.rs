// The plasma two ways in Rust, for docs/BENCHMARK.md: "exact" keeps the reference's integer square root counted in a
// loop but builds the wave tables once and writes into a frame buffer; "float" takes the square root in floating
// point, as the NumPy version does. Args: ring.txt, mode.
use std::time::Instant;
const W96: [i32; 25] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 19, 21, 23, 24, 25, 27, 28, 29, 30, 30, 31, 31, 32, 32, 32];
const W64: [i32; 17] = [0, 3, 6, 9, 12, 15, 18, 20, 23, 25, 27, 28, 30, 31, 31, 32, 32];
fn wave(q: &[i32], k: i32, n: i32) -> i32 {
    let k = k.rem_euclid(n);
    let (h, sign) = if k < n / 2 { (k, 1) } else { (k - n / 2, -1) };
    let j = if h > n / 4 { n / 2 - h } else { h };
    32 + sign * q[j as usize]
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let rows: Vec<Vec<i32>> = std::fs::read_to_string(&a[1]).unwrap().lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect();
    let keys: Vec<[i32; 3]> = (0..8).map(|k| [rows[6 * k][0], rows[6 * k][1], rows[6 * k][2]]).collect();
    let w96: Vec<i32> = (0..96).map(|k| wave(&W96, k, 96)).collect();
    let w64: Vec<i32> = (0..64).map(|k| wave(&W64, k, 64)).collect();
    let float = a[2] == "float";
    let mut buf = vec![0u8; 192 * 96 * 3];
    let mut sum = 0u64;
    let t = Instant::now();
    let frames = 204;
    for tf in 0..frames {
        for y in 0..96i32 { for x in 0..192i32 {
            let n = x * x + (y - 48) * (y - 48);
            let r = if float { (((4 * n + 1) as f64).sqrt() as i32 + 1) / 2 } else {
                let mut k = 0; while (k + 1) * (k + 1) <= 4 * n + 1 { k += 1; } (k + 1) / 2 };
            let s = w64[(r - 2 * tf).rem_euclid(64) as usize] + w96[(x + 6 * tf).rem_euclid(96) as usize]
                + w96[(y + 2 * tf).rem_euclid(96) as usize] + w96[(x - y + 77 + 2 * tf).rem_euclid(96) as usize];
            let q = (s + 8 * tf).rem_euclid(256);
            let (k, j) = ((q / 32) as usize, q % 32);
            let (c0, c1) = (keys[k], keys[(k + 1) % 8]);
            let i = ((y * 192 + x) * 3) as usize;
            for c in 0..3 { buf[i + c] = (c0[c] + ((c1[c] - c0[c]) * j).div_euclid(32)) as u8; }
        } }
        sum += buf[1000] as u64;
    }
    println!("{}: {:.3} ms a frame ({sum})", a[2], t.elapsed().as_secs_f64() * 1e3 / frames as f64);
}
