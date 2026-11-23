//! Strata, once per screen row per frame of the dive. Deck: how many pixels of the row hit the seabed, then for each:
//! the map row's base (whole units), the hit's height over it (whole units and hundredths 0..99), the face (0 a top,
//! 1..3 a front or a side, 4 the far seabed) and the column's number in the rock's second table, 10 s + j + 5:
//! the strata shifted s tones there and the rock j 32nds lighter. The rock lies in strata of four tones, five units
//! deep. Punches, per pixel: the tone (0..3) and how much lighter, less 4 on a fine dark band every half unit (not on
//! a top).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let base = inp();
        let height = inp();
        let mut fine = inp() / 10;
        let face = inp();
        let vary = inp();
        let mut tone = vary / 10;
        let mut lighter = vary - tone * 10 - 5;
        // the stratum: (base + height) / 5 rounded down, mod 4
        let mut level = base + height + 500;
        while level >= 100 { level -= 100; }
        while level >= 20 { level -= 20; }
        while level >= 5 { level -= 5; tone += 1; }
        while tone >= 4 { tone -= 4; }
        if fine >= 5 { fine -= 5; }
        if face != 0 && fine == 0 { lighter -= 4; }
        out(tone);
        out(lighter);
    }
}
