//! Across, once per column per frame of the drive, twice: where the column's ray is across the street at the
//! blocks' near ends, then at their far ends. Deck: the ray (across and along, quarters of a pixel at the screen's
//! distance), the first distance ahead (quarters; it may be behind), then nine distances 120 apart. Punches, for each,
//! how far across the ray is there (quarters, toward its side), as far as 600; 0 for one behind.
// cardiac: phase-cards=3,0
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let across = inp();
    let along = inp();
    let mut target = inp();
    let size = abs(across);
    // SAFETY: a ray always looks ahead (the turn is at most 27 degrees, the view 41 either side)
    unsafe { assume(along > 0); }
    // across grows by size / along a quarter along. Ten quarters along: size added ten times, kept as a whole
    // number and a part (division by repeated subtraction; the sum stays under along + size, a card)
    let mut ten = 0;
    let mut part = 0;
    for _ in 0..10 {
        part += size;
        while part >= along { part -= along; ten += 1; }
    }
    // a part is added by taking away its lack (along - part), so that a sum of two never passes a card
    let ten_lack = along - part;
    // 120 quarters along, from one block's end to the next: ten twelve times
    let mut stride = 0;
    part = 0;
    for _ in 0..12 {
        stride += ten;
        part -= ten_lack;
        if part < 0 { part += along; } else { stride += 1; }
    }
    let stride_lack = along - part;
    // the first distance ahead (1-120; the first is more than 120 behind): walked ten quarters, then a quarter, at a
    // time
    let mut first = target;
    if first <= 0 { first += 120; }
    overlay();                          // phase 2: the walk loads over the strides
    let mut gone = 0;
    let mut sum = 0;
    while first >= 10 {
        first -= 10;
        gone += ten;
        sum -= ten_lack;
        if sum < 0 { sum += along; } else { gone += 1; }
    }
    while first > 0 {
        first -= 1;
        sum += size;
        while sum >= along { sum -= along; gone += 1; }
    }
    overlay();                          // phase 3: the distances load over the walk
    for _ in 0..9 {
        if target <= 0 {
            out(0);
        } else {
            if gone > 600 { gone = 600; }
            out(if across < 0 { -gone } else { gone });
        }
        if target < 864 {               // past the furthest block's far end it is never used
            if target > 0 {
                gone += stride;
                sum -= stride_lack;
                if sum < 0 { sum += along; } else { gone += 1; }
            }
            target += 120;
        }
    }
}
