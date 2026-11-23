//! Palette, for the shaded cube. Deck: for each of the 6 faces, its brightness v (0-16) and its colour (r, g, b) in
//! sixteenths. Punches the lit colour: each channel times v, by adding.
// cardiac: replaces cube/original/palette.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..6 {                     // the faces
        let brightness = inp();         // 0..16
        for _ in 0..3 {                 // red, green, blue, in sixteenths
            let sixteenths = inp();
            // lit = sixteenths x brightness, by adding: CARDIAC has no multiply
            let mut lit = 0;
            for _ in 0..brightness { lit += sixteenths; }
            out(lit);
        }
    }
}
