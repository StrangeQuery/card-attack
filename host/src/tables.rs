//! Printed tables and small helpers shared by the segments. These are the host's: the tables a demo crew would
//! print onto cards and hand to the machines.

/// JavaScript's Math.round (halves round up), which the tables were first printed with.
pub fn js_round(x: f64) -> i32 { (x + 0.5).floor() as i32 }

/// A bitmap file: one row per line, one digit per pixel.
pub fn digits(text: &str) -> Vec<Vec<i32>> { text.trim().lines().map(|r| r.trim_end().bytes().map(|b| (b - b'0') as i32).collect()).collect() }

/// Rows of space-separated numbers.
pub fn rows(text: &str) -> Vec<Vec<i32>> { text.trim().lines().map(|l| l.split_whitespace().map(|v| v.parse().unwrap()).collect()).collect() }

/// Flatten a list of RGB triples.
pub fn flat<const N: usize>(v: &[[i32; N]]) -> Vec<i32> { v.iter().flatten().copied().collect() }

/// The cube's data cards.
pub mod cube {
    pub fn sin60() -> Vec<i32> { (0..60).map(|k| super::js_round(12.0 * (k as f64 * std::f64::consts::PI / 30.0).sin())).collect() }
    pub fn sin48() -> Vec<i32> { (0..48).map(|k| super::js_round(24.0 * (k as f64 * std::f64::consts::PI / 24.0).sin())).collect() }
    pub const HALF: i32 = 12;
    pub fn corners() -> Vec<[i32; 3]> { (0..8).map(|v| [if v & 1 != 0 { 1 } else { -1 }, if v & 2 != 0 { 1 } else { -1 }, if v & 4 != 0 { 1 } else { -1 }]).collect() }
    pub const EDGES: [[i32; 2]; 12] = [[0, 1], [2, 3], [4, 5], [6, 7], [0, 2], [1, 3], [4, 6], [5, 7], [0, 4], [1, 5], [2, 6], [3, 7]];
    const PLANES: [[usize; 3]; 3] = [[0, 2, 1], [1, 2, 0], [0, 1, 2]];
    pub fn corner_cards(axis: usize) -> Vec<i32> {
        let [p, q, r] = PLANES[axis];
        corners().iter().flat_map(|c| [c[p], c[q], HALF * c[r]]).collect()
    }
    pub fn square_cards() -> Vec<i32> { corners().iter().flat_map(|c| [c[0], c[1]]).collect() }
    pub struct Face { pub corners: Vec<i32>, pub tri: Vec<i32>, pub edges: Vec<i32> }
    pub fn faces() -> Vec<Face> {
        let mut out = vec![];
        for axis in 0..3 { for side in [-1, 1] {
            let ring: Vec<[i32; 3]> = (0..4).map(|k| { let mut c = [0; 3]; let (u, w) = ((axis + 1) % 3, (axis + 2) % 3);
                c[axis] = side; c[u] = if k == 0 || k == 3 { -1 } else { 1 }; c[w] = if k < 2 { -1 } else { 1 }; c }).collect();
            let order: Vec<[i32; 3]> = if side > 0 { ring } else { ring.into_iter().rev().collect() };
            let id = |c: &[i32; 3]| (c[0] > 0) as i32 + 2 * (c[1] > 0) as i32 + 4 * (c[2] > 0) as i32;
            let vs: Vec<i32> = order.iter().map(id).collect();
            let edges = (0..4).map(|i| { let (a, b) = (vs[i], vs[(i + 1) % 4]);
                EDGES.iter().position(|&[p, q]| (p == a && q == b) || (p == b && q == a)).map(|e| e as i32).unwrap_or(-1) }).collect();
            out.push(Face { tri: vs[..3].to_vec(), corners: vs, edges });
        } }
        out
    }
    pub const COLOURS: [[i32; 3]; 6] = [[15, 3, 3], [2, 13, 15], [15, 13, 2], [4, 6, 15], [14, 4, 14], [3, 14, 4]];
    pub const ROUTE: [[i32; 3]; 7] = [[0, -4, 2], [-4, 0, 6], [0, 4, 4], [4, 0, 12], [0, -4, 4], [-4, 0, 6], [0, 4, 2]];
    pub fn route_jumps() -> i32 { ROUTE.iter().map(|r| r[2]).sum() }
    pub fn edge_cards() -> Vec<i32> { EDGES.iter().flatten().copied().collect() }
    pub fn route_cards() -> Vec<i32> { ROUTE.iter().flatten().copied().collect() }
}
