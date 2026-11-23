//! Snow, once per frame of the dive. Deck: the camera's place in the snow's box (up and along, tenths, 0..139), how
//! many specks, then each speck's place in the box (across -200..199, up and along 0..139, tenths). The specks hang
//! still in the water; the box goes round with the camera, 7 units above and below it and from 2 behind to 12 ahead,
//! and it is drawn three times along (out to 40 ahead). Punches, per speck, its three places from the camera
//! (across, up, along, tenths).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let up = inp();
    let along = inp();
    let count = inp();
    for _ in 0..count {
        let x = inp();
        let mut y = inp() - up + 70;
        if y < 0 { y += 140; }
        if y >= 140 { y -= 140; }
        let mut z = inp() - along + 20;
        if z < 0 { z += 140; }
        if z >= 140 { z -= 140; }
        for copy in 0..3 {
            out(x);
            out(y - 70);
            out(z - 20 + copy * 140);
        }
    }
}
