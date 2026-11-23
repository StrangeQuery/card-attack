//! Caption, once per row of the caption's strip (7 rows at the top left, 69 pixels across) per frame of the dive
//! while it shows. Deck: how far it has faded in (0..16), then for each of the row's 69 pixels its dot of the caption
//! set in the 3 x 5 font (1 lit, else 0) and its colour as it is (red, green, blue). The strip is black with the lit
//! dots white; it fades in and out: each channel goes from what it is toward black (or white) by the fade in
//! sixteenths, v + (t - v) fade / 16 toward zero. Punches the row's pixels.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let fade = inp();
    for _ in 0..69 {
        let mut to = 0;
        if inp() == 1 { to = 255; }
        for _ in 0..3 {
            let v = inp();
            // (t - v) fade passes a card: the difference's sixteens and the rest, each added fade times, the
            // sixteens carried out of the rest as they fill
            let size = abs(to - v);
            let mut rest = size;
            let mut sixteens = 0;
            while rest >= 16 { rest -= 16; sixteens += 1; }
            let mut whole = 0;
            let mut part = 0;
            for _ in 0..fade {
                whole += sixteens;
                part += rest;
                if part >= 16 { part -= 16; whole += 1; }
            }
            if to < v { whole = -whole; }
            out(v + whole);
        }
    }
}
