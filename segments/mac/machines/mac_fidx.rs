//! Floor, once per frame of Boom: for each row of the floor (and the ceiling's row as far above the middle, which sees
//! the same places), the texel under each of the 448 columns, across and down: a start and a step a column, each in
//! texels and thousandths, wrapping at 64 (a texture a cell). Deck: the count; each row: across, the start (texel,
//! thousandths) and the step (texels, thousandths); the same down. Punches each column's texel across and down.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut across = inp();
        let mut across_part = inp();
        let step = inp();
        let step_part = inp();
        let mut down = inp();
        let mut down_part = inp();
        let down_step = inp();
        let down_step_part = inp();
        for _ in 0..448 {
            out(across);
            out(down);
            // thousandths: the sum passes 999 just when the part is more than 999 less the step's
            let short = 999 - step_part;
            if across_part > short {
                across_part -= short;
                across_part -= 1;
                across += 1;
            } else {
                across_part += step_part;
            }
            across += step;
            if across >= 64 { across -= 64; }
            let short = 999 - down_step_part;
            if down_part > short {
                down_part -= short;
                down_part -= 1;
                down += 1;
            } else {
                down_part += down_step_part;
            }
            down += down_step;
            if down >= 64 { down -= 64; }
        }
    }
}
