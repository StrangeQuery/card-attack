//! Slice, once per screen column per slice per frame of the dive, for the column's lowest pixel not yet settled.
//! Deck: the slice k, the pixel's last slice, its ray's x and height over the slice's row base (each a whole part
//! and hundredths 0..99), its steps per slice across and up (hundredths), how far the row base drops to the next
//! slice (whole units), then the slice's row of the height map: 201 heights in tenths over its base, x = -100..100.
//! Near (slices 0..29) the seabed is made of blocks: the column's height snapped down to an even number of units, a
//! unit square; further out it is smooth, the two heights either side, the step between them in tenths of a unit.
//! Punches 2 when the ray has gone past its last slice (too far to see), 3 when it is 90 or more over the base
//! (above anything there is to hit), 1 when it is under the seabed (a hit), else 0; then the ray one slice on
//! (x and height over the next row's base, as dealt).
// cardiac: phase-cards=9,0
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let slice = inp();
    let last = inp();
    let mut x = inp();
    let mut x_part = inp();
    let mut height = inp();
    let mut height_part = inp();
    let across = inp();
    let up = inp();
    let drop = inp();
    let mut status = 1;                 // to be settled by the row: a hit or not
    if height >= 90 { status = 3; }
    if slice > last { status = 2; }
    // where the ray is in the row, and its height in tenths
    let mut index = x + 100;
    let mut part = x_part / 10;
    if index < 0 { index = 0; part = 0; }
    if index > 199 { index = 199; part = 9; }
    let mut tenths = 0;                 // only needed below 90 (ten times 90 or more could pass a card)
    if height < 90 {
        tenths = height * 10 + height_part / 10;
        if slice < 30 {
            // under a block when the even number of units above the ray's whole units is no higher than its
            // column's height: as under a smooth seabed one tenth below that (the ray is never 100 under its
            // row's base, so the halving rounds down)
            tenths = ((height + 100) / 2 - 49) * 20 - 1;
        }
    }
    overlay();
    if slice < 30 {
        index = x + 100;            // near, the block's column
        part = 0;
        if index < 0 { index = 0; }
        if index > 200 { index = 200; }
    }
    // one slice on
    x_part += across;
    while x_part >= 100 { x_part -= 100; x += 1; }
    while x_part < 0 { x_part += 100; x -= 1; }
    height_part += up;
    while height_part >= 100 { height_part -= 100; height += 1; }
    while height_part < 0 { height_part += 100; height -= 1; }
    height += drop;
    overlay();                          // the row is read last: only as far as the ray needs
    if status == 1 {
        // the cards before x's are read and dropped, ten at a time while there are ten
        while index >= 10 {
            index -= 10;
            inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp(); inp();
        }
        for _ in 0..index { inp(); }
        let left = inp();
        let mut right = left;
        if part > 0 { right = inp(); }
        let mut rise = 0;
        for _ in 0..part { rise += right - left; }
        if tenths >= left + rise / 10 { status = 0; }
    }
    out(status);
    out(x);
    out(x_part);
    out(height);
    out(height_part);
}
