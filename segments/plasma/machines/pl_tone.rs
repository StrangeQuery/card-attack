//! Tone, three times per tile row of the plasma at 192x96 and of the ray's plasma floor: once each for red, green
//! and blue. Deck: for each of the 96 pixels its look and its plasma colour's red, green or blue. A look under 100
//! is a fixed colour (0..2 the sky's bands, 3 haze, 4 and 5 sky and 6 haze seen in the ball, 7 the light): this
//! one's red, green or blue of each is punched into the program. A look of 100 + 32 D + F is the plasma, taken D
//! 32nds toward black (seen in the ball, or in its shadow) and then F 32nds toward the haze (the fog). Each step is
//! a + (b - a) L / 32 rounded down. Punches each pixel's value.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// a + (b - a) L / 32 rounded down, for 0..255 and L 0..32: (b - a) added L times to a part kept in 0..31, a
/// thirty-second carried to a at a time, since (b - a) L can pass three digits.
#[inline(never)]
fn blend(from: i16, to: i16, level: i16) -> i16 {
    let gap = to - from;
    let mut value = from;
    let mut part = 0;
    for _ in 0..level {
        part += gap;
        while part >= 32 { part -= 32; value += 1; }
        while part < 0 { part += 32; value -= 1; }
    }
    value
}

pub fn main() {
    let fixed: [i16; 8] = load_resource("palette");   // the host punches this channel of the palette in here
    for _ in 0..96 {
        let look = inp();
        let mut value = inp();
        if look < 100 {
            // SAFETY: the fixed looks are 0..7
            unsafe { assume(look >= 0 && look <= 7); }
            value = fixed[look as usize];
        } else {
            let mut fog = look - 100;
            let mut dark = 0;
            while fog >= 32 {          // D and F, by taking away 32s
                fog -= 32;
                dark += 1;
            }
            value = blend(value, 0, dark);
            value = blend(value, fixed[3], fog);   // toward the haze
        }
        out(value);
    }
}
