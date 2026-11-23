//! Clock, a few times per frame of the die and the Rubik's cube (the tumble's three angles, the roam's four, the
//! turning layer's, the pop's), per frame of the wobble (each row's angle) and of the BBS (characters arrived):
//! angles that grow by a rate a step. Deck: how many angles; then each: how many terms, and per term its count of
//! steps and its rate (quarter turns, tenths of a degree up to 900, thousandths of a tenth). Punches each angle, the
//! sum of the counts times the rates rounded to the tenth, as its quarter turn (0-3) and the rest (0-899).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let angles = inp();
    for _ in 0..angles {
        let terms = inp();
        let mut quarter = 0;
        let mut rest = 0;
        let mut part = 500;             // thousandths of a tenth, half of one to round
        for _ in 0..terms {
            let count = inp();
            let quarters = inp();
            let tenths = inp();
            let thousandths = inp();
            for _ in 0..count {
                // the thousandths carried as they fill, without passing a card
                if part > 999 - thousandths { part -= 999 - thousandths; part -= 1; rest += 1; } else { part += thousandths; }
                if rest >= 900 { rest -= 900; quarter += 1; }
                if rest >= 900 - tenths { rest -= 900 - tenths; quarter += 1; } else { rest += tenths; }
                quarter += quarters;
                while quarter >= 4 { quarter -= 4; }
            }
        }
        out(quarter);
        out(rest);
    }
}
