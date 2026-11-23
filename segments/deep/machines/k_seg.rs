//! Lightning, for the dive's ending at 1536x768: where each 4 x 4 pixels is in a line's glow. Deck: how many; then for
//! each how far along the line it is and how far across (whole pixels), the line's length, whether it starts its
//! stroke and whether it ends it. Punches how far past the line's end it is (none beside the line; -1 past a joint
//! or beyond the table's 63) and how far across (its size).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..inp() {
        let s = inp();
        let t = abs(inp());
        let len = inp();
        let first = inp();
        let last = inp();
        let mut along = 0;
        if s < 0 { if first == 1 { along = -s; } else { along = -1; } }
        if s >= len { if last == 1 { along = s - len; } else { along = -1; } }
        if along > 63 || t > 63 { along = -1; }
        out(along);
        out(t);
    }
}
