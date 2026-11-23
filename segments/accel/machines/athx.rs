//! Thing places, once per lamp and car drawn per frame of the accelerated drive: its middle on the screen. Deck: how
//! many, then each: 220 |across| / a and 16 times what that leaves over a (amul's), the side (-1 left, 1 right).
//! Punches each 6136 + 3520 across / a 16ths of a pixel (toward zero), as the pixel and 16ths (0-15).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let q = inp();
        let sixteenths = inp();
        // 6136 = 383 pixels and 8 16ths, plus or less q pixels and those 16ths
        let mut pixel = 383;
        let mut frac = 8;
        if inp() < 0 { pixel -= q; frac -= sixteenths; } else { pixel += q; frac += sixteenths; }
        while frac < 0 { frac += 16; pixel -= 1; }
        while frac >= 16 { frac -= 16; pixel += 1; }
        out(pixel);
        out(frac);
    }
}
