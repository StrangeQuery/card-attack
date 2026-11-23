//! The turn round the tunnel, for the turn's CGA tunnel, once a frame: down the tunnel the checks turn by the depth
//! times K (K is six times the frame's sine, thousandths), in fifth-hundredths of a check, and all of them by the
//! frame's turn: for each depth (hundredths) d, fl(d K, 100000) and the turn, wrapped to 0..999 (two checks). Deck: the
//! first depth's turn (0..999), its d K's remainder by 100000 as thousands (0..99) and rest (0..999); K's thousands
//! (-6..5) and rest (0..999); how many depths. Punches each depth's turn, K added a depth at a time.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut turn = inp();
    let mut thousands = inp();
    let mut rest = inp();
    let step_thousands = inp();
    let step_rest = inp();
    let count = inp();
    // rest + step_rest would pass a card: past what the step leaves, it carries
    let room = 999 - step_rest;
    for _ in 0..count {
        out(turn);
        if rest > room {
            rest -= room;
            rest -= 1;
            thousands += 1;
        } else {
            rest += step_rest;
        }
        // the thousands, a hundred of them a fifth-hundredth, carry into the turn
        thousands += step_thousands;
        if thousands > 99 {
            thousands -= 100;
            if turn == 999 { turn = 0; } else { turn += 1; }
        }
        if thousands < 0 {
            thousands += 100;
            if turn == 0 { turn = 999; } else { turn -= 1; }
        }
    }
}
