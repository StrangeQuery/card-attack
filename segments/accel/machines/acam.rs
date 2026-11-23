//! Camera, once per frame of the accelerated drive: how high it is. Deck: the street's grade at the camera's entry
//! of the plan and the next (eighths), how far into it (quarters, 0-7). Punches the camera's height, 2 units over the
//! street, 8 g + (g' - g) into + 128 (64ths), as eighths (rounded down) and the 64ths over (0-7).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let grade = inp();
    let next = inp();
    let into = inp();
    let mut height = 8 * grade + 128;
    for _ in 0..into { height += next - grade; }
    // eighths rounded down: halved three times (below zero, by the size and back, one more if any is over)
    let mut eighths = abs(height) / 2 / 2 / 2;
    if height < 0 { if 8 * eighths < -height { eighths += 1; } eighths = -eighths; }
    out(eighths);
    out(height - 8 * eighths);
}
