//! Car across, once per car per frame of the drive. Deck: how far into its road table entry the car is (0-7
//! quarters), the entry and the next (the street's middle across, eighths), the car's lane (eighths from the
//! middle) and the camera across. Punches how far across from the camera the car is (eighths): the middle there,
//! rounded down, plus the lane, less the camera.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let part = inp();
    let before = inp();
    let rise = inp() - before;
    let mut sum = 64;
    for _ in 0..part { sum += rise; }
    let mut middle = before - 8;
    while sum >= 8 { sum -= 8; middle += 1; }
    out(middle + inp() - inp());
}
