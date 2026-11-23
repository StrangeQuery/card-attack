//! Multiply, per lamp and car per frame of the drive, four times, to turn them to the camera's view. Deck: a, b and
//! per. Punches a b / per toward zero (|b| + per at most 800).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let a = inp();
    let b = inp();
    let size = muldiv(abs(a), abs(b), inp());
    out(if (a < 0) == (b < 0) { size } else { -size });
}
