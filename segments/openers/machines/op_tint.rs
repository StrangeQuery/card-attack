//! Opening cards tint, once per screen row per frame of a colour card. Deck: the fade's steps (op_fade, 0-5), then
//! the row's 192 colours of the card's picture, red, green and blue (0-255). Shows each at steps fifths, rounded
//! down: floor(colour steps / 5). Punches the row's 192 colours, red, green and blue.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// floor(colour steps / 5). colour times steps runs past a card (255 x 5), so the colour is split into fifths and
/// what is left over, and each is added steps times: CARDIAC has no multiply.
fn faded(colour: i16, steps: i16) -> i16 {
    let fifths = colour / 5;
    let over = colour - fifths * 5;
    let mut v = 0;
    let mut spare = 0;
    for _ in 0..steps {
        v += fifths;
        spare += over;
    }
    v + spare / 5
}

pub fn main() {
    let steps = inp();
    // held at full, the colours as they are
    if steps == 5 {
        for _ in 0..576 { out(inp()); }
        return;
    }
    for _ in 0..576 { out(faded(inp(), steps)); }
}
