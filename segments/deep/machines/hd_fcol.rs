//! Fish colour, for the dive's creatures at 1536x768: a channel of a fish's texels lit and fogged (and the flash's
//! light first, the same way). Deck: the fish's light and fog g (hundredths, at most 111; or its flash, at most 55),
//! the fog's colour taken in for this channel (or 0), how many pixels, then for each the texel's channel and the
//! flash's light there (or the texel's flash and 0). Punches, per pixel, (c + flash) g / 100 + fog (the product by the
//! digits of c + flash, each part rounded down), at most 255.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let g = inp();
    let fog = inp();
    for _ in 0..inp() {
        let v = inp() + inp();
        // each digit's product by adding g that many times (at most 9: a product by a variable would add the
        // larger number of times)
        let mut hundreds = 0;
        for _ in 0..digit(v, 0) { hundreds += g; }
        let mut tens = 0;
        for _ in 0..digit(v, 1) { tens += g; }
        let mut units = 0;
        for _ in 0..digit(v, 2) { units += g; }
        let mut c = hundreds + tens / 10 + units / 100 + fog;
        if c > 255 { c = 255; }
        out(c);
    }
}
