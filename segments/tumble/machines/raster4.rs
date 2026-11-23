//! Edge rasterizer for the tumble's cube, once per tile per frame. Deck: 12 edges as ytop, ybottom, xtop, xbottom.
//! For each edge punches ytop, the number of rows n, then a (left, right) span for each of its rows: the part of
//! the edge within half a pixel of the row's centre, its ends rounded to the nearest pixel. So the edge's runs are
//! even along it, half runs at its two ends, and each span comes from the edge's own ends, wherever the tile is.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..12 {                    // the edges
        let y_top = inp();
        let y_bottom = inp();
        let x_top = inp();
        let x_bottom = inp();
        out(y_top);
        let height = y_bottom - y_top;
        let rows = height + 1;
        out(rows);
        let dx = x_bottom - x_top;
        let (width, step) = if dx < 0 { (-dx, -1) } else { (dx, 1) };
        if height < 1 {                 // a flat edge is one span
            if step < 0 { out(x_bottom); out(x_top); } else { out(x_top); out(x_bottom); }
        } else {
            // Row k's span ends where the edge crosses the line half a row below its centre, (2k + 1) / 2 of the
            // height down: x moves round((2k + 1) width / (2 height)) from the top. An error term keeps that
            // fraction's top less the bottom times x, so it needs only additions and comparisons: it starts at
            // width + height (the half for rounding) and grows by twice the width each row.
            let twice_height = height + height;
            let mut x = x_top;
            let mut error = width + height;
            for rows_left in (0..rows).rev() {   // the rows still to come after this one
                let row_start = x;
                if rows_left >= 1 {
                    while error >= twice_height { error -= twice_height; x += step; }
                    error += width + width;
                } else {
                    x = x_bottom;       // the last row's half run ends at the edge's end
                }
                if x < row_start { out(x); out(row_start); } else { out(row_start); out(x); }
            }
        }
    }
}
