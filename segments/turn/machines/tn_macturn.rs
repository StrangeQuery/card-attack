//! The Mac's tunnel's turn, for the turn's first frames, once per picture (the cold open's mac_turn, its sine table
//! run on): how it twists and how far it has come at picture j, t = j / 4 seconds. Deck: the picture j (0-48), then the
//! sine table, 100 sin(0.075 i) for i = 0..144. Punches the turn, 30 sin(0.225 j); the depth twist's weight,
//! 16 sin(0.15 j) (each toward zero); and the travel, 11 j mod 40.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let picture = inp();
    let turn_at = picture * 3;
    let weight_at = picture + picture;
    let mut turn = 0;
    let mut weight = 0;
    for i in 0..145 {
        let sine = inp();
        // 30 s / 100 is 3 s / 10, and 16 s / 100 is 4 s / 25: neither passes a card
        if i == turn_at { turn = sine * 3 / 10; }
        if i == weight_at { weight = sine * 4 / 25; }
    }
    let mut travel = 0;
    for _ in 0..picture {
        travel += 11;
        if travel >= 40 { travel -= 40; }
    }
    out(turn);
    out(weight);
    out(travel);
}
