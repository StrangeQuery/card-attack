//! The camera, once per frame of the drive. Deck: how far into its road table entry the camera is (0-7 quarters),
//! its place across the street (eighths from the middle), then nine entries of the street's plan from under it (the
//! middle across in eighths, every 2 units). Punches the camera across (eighths) and the street's slope ahead: how
//! far the middle moves across between 2 and 14 units ahead (eighths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The middle a part of the way (0-7 eighths) from entry a to entry b, rounded down: (b - a) part / 8 by adding,
/// lifted by 64 first so it is never below zero.
#[inline(never)]
fn between(a: i16, b: i16, part: i16) -> i16 {
    let mut sum = 64;
    for _ in 0..part { sum += b - a; }
    let mut eighths = 0;
    while sum >= 8 { sum -= 8; eighths += 1; }
    a + eighths - 8
}

pub fn main() {
    let part = inp();
    let lane = inp();
    let mut road = [0; 9];
    for i in 0..9 { road[i] = inp(); }
    out(between(road[0], road[1], part) + lane);
    out(between(road[7], road[8], part) - between(road[1], road[2], part));
}
