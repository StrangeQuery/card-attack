//! Wrap, once per message of the party chat. Deck: the handle's characters, how many, then each's code and its
//! advance (bold), and the colon's advance; the space's advance; how many words, then each word's characters, how
//! many and their advances. Picks the handle's colour (its codes' sum, mod 8), starts the text 6 past the handle's
//! colon, and lays the words along the line, a space apart; a word that would reach past the panel's edge starts a
//! new line, at the handle's start (unless it is the line's first). Punches: the colour (0-7), where the text starts
//! across the layer, each word's line (0 the first), and the message's height (29 a line and 12 after).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// The text's start past the handle's start, before the handle's width (the panel's edge 132 on the layer, plus 22);
/// a wrapped line's start; how far a word may reach; a line's step and the gap after a message.
const START: i16 = 154;
const WRAP: i16 = 148;
const RIGHT: i16 = 494;
const LINE: i16 = 29;
const GAP: i16 = 12;

pub fn main() {
    let characters = inp();
    let mut colour = 0;
    let mut text_x = START;
    for _ in 0..characters {
        // the codes' sum mod 8, kept as it goes so it never passes a card
        colour = remainder(colour + inp(), 8);
        text_x += inp();
    }
    text_x += inp();
    let space = inp();
    let words = inp();
    out(colour);
    out(text_x);
    // where the next word starts; whether the line has a word yet
    let mut x = text_x;
    let mut started = false;
    let mut line = 0;
    for _ in 0..words {
        let glyphs = inp();
        let mut word = 0;
        for _ in 0..glyphs { word += inp(); }
        if started && x + word > RIGHT {
            line += 1;
            x = WRAP;
        }
        started = true;
        x += word + space;
        out(line);
    }
    let mut h = GAP + LINE;
    for _ in 0..line { h += LINE; }
    out(h);
}
