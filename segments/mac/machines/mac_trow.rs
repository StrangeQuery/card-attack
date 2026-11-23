//! Tunnel row, once per row of the one-bit tunnel's blocks per picture: each block's shade, the checkerboard down the
//! tunnel. Deck: the picture's turn and travel (mac_turn's); then for each of the row's 128 blocks its angle around
//! (mac_angle's), how far along it is (the ring table's at its distance), how far its depth twists the wall this
//! picture (mac_turn's at its ring's twist), and its light and dark shades (the ring table's: 256 white, the fog in
//! them). Punches each block's shade: light where the check around (50 of the angle a check) and the check along (20
//! a check) are both even or both odd, else dark.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let turn = inp();
    let travel = inp();
    for _ in 0..128 {
        let mut around = inp() + turn + inp();
        let along = inp() + travel;
        let light = inp();
        let dark = inp();
        if around < 0 { around += 800; }
        if around >= 800 { around -= 800; }
        // odd checks around: the tens digit 5 or more (50 a check, 100 a pair)
        let ring = digit(around, 1) >= 5;
        // odd checks along: 20 a check, the count's last bit
        let count = along / 20;
        let half = count / 2;
        let check = count - half - half > 0;
        if ring == check { out(light); } else { out(dark); }
    }
}
