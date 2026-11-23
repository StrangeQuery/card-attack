//! Glow sizes, as many lights as the host has, per frame of the accelerated drive. Deck: how many, then each: sc
//! (quarters of a pixel a unit there) and its kind's m, d, least and most, its strength (500ths) and its kind's n.
//! Punches each its radius, sc m / d kept to least..most (quarters of a pixel), and its strength s n / 5
//! (thousandths): a lamp's 9 / 10 at least 12 and 9 / 5; headlights' 6 / 10 at least 12 and 7 / 5; a tail lamp's 7 /
//! 20 at least 8 and 6 / 5; the jet's 0 at least 12 and 10 / 5; our tail lamps' 9 / 55 kept to 8..144 and 10 / 5.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

/// v m / d rounded down (m added v times, d taken away whenever the sum reaches it: the product would pass a card).
fn part(v: i16, m: i16, d: i16) -> i16 {
    let mut q = 0;
    let mut sum = 0;
    for _ in 0..v {
        sum += m;
        while sum >= d { sum -= d; q += 1; }
    }
    q
}

pub fn main() {
    for _ in 0..inp() {
        let sc = inp();
        let m = inp();
        let d = inp();
        let least = inp();
        let most = inp();
        let r = part(sc, m, d);
        out(if r < least { least } else if r > most { most } else { r });
        let s = inp();
        out(part(s, inp(), 5));
    }
}
