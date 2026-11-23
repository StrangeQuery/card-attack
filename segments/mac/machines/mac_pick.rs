//! Pick, once per frame of Boom: for each column, the nearer of its ray's two first walls (mac_first's, the x lines'
//! and the y lines'), and the door if it is in front of it. The x line's crossing is the nearer when the way the ray had
//! gone across there is less than how far along the y line's is: both are the ray's distances one way. Deck: the
//! count; each column: the x lines' wall (its texture, 0 for none, and how far across), the y lines' (its texture and
//! how far along), the x lines' door (1 if found, how far across), the y lines' (1 if found, how far along). Punches
//! the wall's side (0 the x lines', 1 the y lines') and the door's (0 none, 1 the x lines', 2 the y lines').
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let x_kind = inp();
        let x_across = inp();
        let y_kind = inp();
        let y_along = inp();
        let x_gate = inp();
        let x_gate_across = inp();
        let y_gate = inp();
        let y_gate_along = inp();
        let x_wall = x_kind != 0 && (y_kind == 0 || x_across < y_along);
        if x_wall { out(0); } else { out(1); }
        let mut gate = 0;
        if x_gate == 1 && (y_gate == 0 || x_gate_across < y_gate_along) {
            // the x lines' door, in front of a y line's wall only when it had gone less across than the wall is along
            if x_wall || x_gate_across < y_along { gate = 1; }
        } else if y_gate == 1 {
            if !x_wall || x_across >= y_gate_along { gate = 2; }
        }
        out(gate);
    }
}
