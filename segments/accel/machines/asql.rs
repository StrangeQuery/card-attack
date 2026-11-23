//! Software colours' levels, once per channel per frame of the software half: which of the drive's ten levels each
//! block's colour is at or past. Deck: where levels 1-8 start (tenths), how many, then each a channel (tenths).
//! Punches each the last level whose start it reaches (0-8).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let (s1, s2, s3, s4, s5, s6, s7, s8) = (inp(), inp(), inp(), inp(), inp(), inp(), inp(), inp());
    for _ in 0..inp() {
        let t = inp();
        out(if t >= s8 { 8 } else if t >= s7 { 7 } else if t >= s6 { 6 } else if t >= s5 { 5 } else if t >= s4 { 4 } else if t >= s3 { 3 } else if t >= s2 { 2 } else if t >= s1 { 1 } else { 0 });
    }
}
