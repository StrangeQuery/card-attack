//! Index, for the dive's ending at 1536x768: where a fall-off is in its table. Deck: the table's last place, how
//! many; then for each two parts (across and down, each at most 100). Punches their sum, or -1 past the table.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let last = inp();
    for _ in 0..inp() {
        let e = inp() + inp();
        if e > last { out(-1); } else { out(e); }
    }
}
