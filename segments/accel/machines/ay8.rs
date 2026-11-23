//! Heights, as many as the host has, per frame of the accelerated drive: the street's grade a part of the way along
//! an entry of the plan, raised. Deck: how many, then each: the grade at the entry and the next (eighths), how far
//! into it (quarters, 0-7), eighths to raise it. Punches each 8 g + (g' - g) part (64ths) + 8 raise, as eighths
//! (rounded down) and 64ths over (0-7).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let grade = inp();
        let rise = inp() - grade;
        let mut lean = 0;
        for _ in 0..inp() { lean += rise; }
        let mut eighths = grade + inp();
        while lean < 0 { lean += 8; eighths -= 1; }
        while lean >= 8 { lean -= 8; eighths += 1; }
        out(eighths);
        out(lean);
    }
}
