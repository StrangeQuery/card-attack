//! Along, once per lamp and car per frame of the drive. Deck: across s / 400 and along c / 400 (dmul; eighths
//! across and quarters ahead of the camera, c and s the turn's cosine and sine times 400). Punches how far along the
//! view it is, in quarters: half the first, toward zero, plus the second.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let twice = inp();
    let mut rest = abs(twice);
    let mut half = 0;
    while rest >= 2 { rest -= 2; half += 1; }
    out(inp() + if twice < 0 { -half } else { half });
}
