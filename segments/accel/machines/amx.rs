//! March across, once per sample column per frame of the accelerated drive: how far across the column's ray is at
//! each step of the march (amz's: from 3 units to 160, each a 25th further, a quarter at least). Deck: the ray's
//! across (our pixels at 440), then the steps' places (quarters ahead; 0 after the last). Punches for each step how far across it is toward its side: eighths (at most 400)
//! and 64ths over (0-7).
// cardiac: phase-cards=1
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let across = abs(inp());
    // a quarter further along goes 16 across / 440 64ths across: a whole and a part, worked out once
    let mut step = 0;
    let mut sum = 0;
    for _ in 0..16 {
        sum += across;
        while sum >= 440 { sum -= 440; step += 1; }
    }
    let lack = 440 - sum;
    sum = 0;
    overlay();                          // phase 2: the walk loads over the part's working out
    let mut eighths = 0;
    let mut sixty4 = 0;
    let mut ahead = 0;
    let mut next = inp();
    while next > 0 {
        sixty4 += step;
        sum -= lack;
        if sum < 0 { sum += 440; } else { sixty4 += 1; }
        while sixty4 >= 8 { sixty4 -= 8; if eighths < 400 { eighths += 1; } }
        ahead += 1;
        if ahead == next {
            out(eighths);
            out(sixty4);
            next = inp();
        }
    }
}
