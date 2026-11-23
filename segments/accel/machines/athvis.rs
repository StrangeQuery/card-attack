//! Things in view, once per lamp and car per frame of the accelerated drive. Deck: how many, then each: t(dx, c, 400)
//! and t(dz, s, 200) (dmul's) and how far along the view (dalong's, quarters). Punches each how far across the view
//! (their difference, eighths) as its size and side (-1 left, 1 right), and 1 if it is to be drawn (12 to 600
//! quarters along, and its middle less than 600 pixels from the screen's, 220 |across| / along < 600, as
//! 11 |across| / 30 rounded down less than along), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let across = inp() - inp();
        out(abs(across));
        out(if across < 0 { -1 } else { 1 });
        let along = inp();
        let mut left = along;
        let mut sum = 0;
        let mut k = abs(across);
        while k > 0 && left > 0 { sum += 11; if sum >= 30 { sum -= 30; left -= 1; } k -= 1; }
        out(if along >= 12 && along <= 600 && left > 0 { 1 } else { 0 });
    }
}
