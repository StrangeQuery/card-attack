//! Wall start, once per wall a column's ray meets per frame of the accelerated drive: where up the wall the lowest row
//! still open is (the wall is drawn from there up: rows below its foot are the street's, nearer, where they meet).
//! Deck: how many walls, then each: Q (twentieths of a texel a half row: its whole, its part over d, and d; awq's),
//! the open row y and the horizon's, F0 (a texel and twentieths; awfoot's). Punches each F = F0 - m Q, m = 2 (y -
//! H0) + 1: its texel, twentieths (0-19) and the part over d, less d (-d to -1).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut whole = inp();
        let mut part = inp();
        let d = inp();
        let m = 2 * (inp() - inp()) + 1;
        let mut texel = inp();
        let mut tw = inp();
        // the part over d kept less d (-d to -1), so that a part added never passes a card; taking Q off is putting
        // on d - part and taking a twentieth off
        let mut rest = -d;
        let mut n = m;
        if m > 0 { whole = -whole - 1; part = d - part; } else { n = -m; }
        for _ in 0..n {
            tw += whole;
            rest += part;
            if rest >= 0 { rest -= d; tw += 1; }
            while tw < 0 { tw += 20; texel -= 1; }
            while tw >= 20 { tw -= 20; texel += 1; }
        }
        out(texel);
        out(tw);
        out(rest);
    }
}
