//! Star, once per star of the polys' sky: a star laid on its row. Deck: its x and brightness, then the row's 192
//! colours (red, green and blue). Punches the row, a grey of the star's brightness at its x.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let at = inp();
    let light = inp();
    for x in 0..192 {
        let p = read_pixel();
        if x == at { punch(Pixel { r: light, g: light, b: light }); } else { punch(p); }
    }
}
