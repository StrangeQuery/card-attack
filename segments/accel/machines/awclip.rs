//! Wall clip, once per wall a column's ray meets per frame of the accelerated drive: the lowest row the wall can show
//! over the street (below it the street is nearer). Deck: how many walls, then each: its distance (quarters), the
//! lowest row still open, then the street's distances in its column from that row up (999 where there is none),
//! as many as the open row and one. Punches each the lowest row at or above the open one where the street is no
//! nearer than the wall (-1: none). Each wall's cards are dealt as their own run's.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    {
        let far = inp();
        let mut y = inp();
        // (the cards past the row found are left unread)
        while y >= 0 && inp() < far { y -= 1; }
        out(y);
    }
}
