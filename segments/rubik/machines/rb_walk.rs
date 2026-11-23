//! Walk, per polygon per frame of the Rubik's cube: an edge's crossing of each subsample row it meets. Deck: edges
//! as rb_column punched them, then -1. It meets the rows from the first (included) to its
//! bottom (left out), 8 eighths apart. At each it steps the edge's x by the slope for the eighths since the last,
//! moving the column right while its middle is not right of the edge. Punches for each row: the row (0-191), the
//! bound (0 left, 1 right), and the column bound (0-383).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    loop {
        let bound = inp();
        if bound < 0 { break; }
        let mut row = inp();
        let mut eighths = inp();        // eighths of a row since the top
        let mut left = inp();           // eighths to the bottom
        let whole = inp();
        let part = inp();
        let gap = inp();                // the rest at or past this carries
        let mut column = inp();
        let way = inp();
        let mut ahead = inp();          // the column's middle less the edge's x, whole 8ths
        let mut rest = inp();           // the edge's fraction of an 8th, in parts of down
        while left > 0 {
            for _ in 0..eighths {
                ahead -= whole;
                if rest >= gap { rest -= gap; ahead -= 1; } else { rest += part; }
            }
            while ahead < 1 { ahead += 8; column += way; }
            out(row);
            out(bound);
            out(column);
            row += 1;
            eighths = 8;
            left -= 8;
        }
    }
}
