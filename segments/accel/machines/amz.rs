//! March along, once per sample column per frame of the accelerated drive: the column's ray stepped out over the
//! street from 3 units to 160, each step a 25th further (a quarter at least). Deck: the ray's along (our pixels at
//! 440), where it starts (quarters into the camera's entry of the plan for the plan's place, 0-7; into the camera's
//! block for the dash's, 0-119), the step it counts in (128: entries of the plan; 480: the dash's 30 quarters),
//! then the steps' places (quarters ahead, the march's schedule; 0 after the last). Punches for each step how many
//! steps past the start it is and how far into that one (16ths of a quarter).
// cardiac: phase-cards=3
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let along = inp();
    let start = inp();
    let size = inp();
    // a quarter further along the ray goes 16 along / 440 sixteenths along: a whole and a part, worked out once (16
    // additions, 440 taken away while it fits: the sum stays under 440 + 555, a card)
    let mut step = 0;
    let mut sum = 0;
    for _ in 0..16 {
        sum += along;
        while sum >= 440 { sum -= 440; step += 1; }
    }
    // a part is added by taking away its lack, so that the sum never passes a card
    let lack = 440 - sum;
    sum = 0;
    overlay();                          // phase 2: the walk loads over the part's working out
    let mut whole = 0;
    let mut part = 0;
    for _ in 0..start {
        part += 16;
        if part >= size { part -= size; whole += 1; }
    }
    let mut ahead = 0;
    let mut next = inp();
    while next > 0 {
        part += step;
        sum -= lack;
        if sum < 0 { sum += 440; } else { part += 1; }
        while part >= size { part -= size; whole += 1; }
        ahead += 1;
        if ahead == next {
            out(whole);
            out(part);
            next = inp();
        }
    }
}
