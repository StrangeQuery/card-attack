//! Span, once per tile row of the Mode X ray tracer (320x240, rows of four 96-pixel tiles). Deck: the frame's row y,
//! the ball's centre across and down (pixels), the tile's place across. The ball, radius 46, covers the pixels with
//! A^2 + B^2 < 46^2, A across and B up from its centre. Punches where its span starts in this row of the tile and
//! how many of the tile's pixels it covers (0 if none), then B and each covered pixel's A.
// cardiac: phase-cards=4
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let y = inp();
    let centre_x = inp();
    let centre_y = inp();
    let tile_x = inp();
    let b = centre_y - y;             // this row's height above the ball's centre
    let distance = abs(b);
    let mut half_width = -1;           // none of the ball in this row
    if distance < 46 {
        // the largest h with h^2 <= 2115 - b^2. That passes a card, so it is kept as hundreds and units: b^2 taken
        // away as the odd numbers below 2b, then the root counted as the odd numbers that can still be taken away
        let mut hundreds = 21;
        let mut units = 15;
        let mut odd = 1;
        for _ in 0..distance {
            units -= odd;
            if units < 0 { units += 100; hundreds -= 1; }
            odd += 2;
        }
        half_width = 0;
        odd = 1;
        while hundreds > 0 || units >= odd {
            units -= odd;
            if units < 0 { units += 100; hundreds -= 1; }
            odd += 2;
            half_width += 1;
        }
    }
    overlay();                         // the span's width is found; the clipping and punching load over that
    let centre = centre_x - tile_x;    // the ball's centre, across the tile
    let mut first = centre - half_width;
    let mut last = centre + half_width;
    if first < 0 { first = 0; }        // only the tile's own pixels
    if last > 95 { last = 95; }
    let mut count = last - first + 1;
    if count < 0 { count = 0; }
    out(first);
    out(count);
    out(b);
    for x in first..=last {            // each covered pixel's offset across
        out(x - centre);
    }
}
