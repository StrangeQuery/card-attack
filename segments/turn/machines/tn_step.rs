//! The screen's walk's step, for the turn: between two of the screen's grid points 20 pixels apart (places on its
//! picture, thousandths of a texel, longs), a twentieth of the change: the change is 20 a + b (b 0..19), a as a long.
//! Deck: the count; then each pair of points. Punches for each a (thousands, rest) and b.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let whole = inp();
        let part = inp();
        let mut thousands = inp() - whole;
        let mut rest = inp() - part;
        if rest < 0 { rest += 999; rest += 1; thousands -= 1; }
        // the change over 20: its thousands make 50 each (so many whole, 50 a twentieth of a thousand), the rest's
        // twentieths one each, by repeated subtraction
        let mut step_whole = 0;
        while thousands < 0 { thousands += 20; step_whole -= 1; }
        while thousands > 19 { thousands -= 20; step_whole += 1; }
        let mut step_part = thousands * 50;
        while rest > 19 { rest -= 20; step_part += 1; }
        out(step_whole);
        out(step_part);
        out(rest);
    }
}
