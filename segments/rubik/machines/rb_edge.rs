//! Edge, per polygon per frame of the Rubik's cube (a face of a cubie, or its sticker). Deck: the number of corners
//! n, then each corner's x and y on the screen (8ths of a subsample from the middle, y down), anticlockwise as
//! seen. Each edge, from one corner to the next and from the last back to the first, bounds the polygon on the left
//! where it runs down and on the right where it runs up; one that runs level bounds nothing. Punches for each
//! other edge: its bound (0 left, 1 right); 1 if it runs down to the left, when it is mirrored (x taken to -x) so
//! it runs down to the right, else 0; its top's x (mirrored) and y; how far it runs across (0 or more) and down
//! (1 or more). Then -1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let n = inp();
    let first_x = inp();
    let first_y = inp();
    let (mut x0, mut y0) = (first_x, first_y);
    for k in 1..=n {
        let (mut x1, mut y1) = (first_x, first_y);
        if k < n { x1 = inp(); y1 = inp(); }
        if y1 != y0 {
            // top to bottom
            let mut bound = 0;
            let (mut x_top, mut y_top, mut across, mut down) = (x0, y0, x1 - x0, y1 - y0);
            if down < 0 { bound = 1; x_top = x1; y_top = y1; across = -across; down = -down; }
            let mut mirrored = 0;
            if across < 0 { mirrored = 1; x_top = -x_top; across = -across; }
            out(bound);
            out(mirrored);
            out(x_top);
            out(y_top);
            out(across);
            out(down);
        }
        x0 = x1;
        y0 = y1;
    }
    out(-1);
}
