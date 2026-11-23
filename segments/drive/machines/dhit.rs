//! Hits, once per column per frame of the drive: which wall the column's ray meets in each block. Deck: how far the
//! camera is into its block (quarters), then for each of the nine blocks from there: the ray across at the block's
//! near end and at its far end, and its left and right faces across (all quarters from the camera). Blocks start
//! every 120 quarters and are 104 long; one 880 or more ahead is past seeing. A ray already behind a face where a
//! block starts meets its end wall there, if within the building's 120 quarters; one that crosses a face along the
//! block meets that face. Punches for each wall met, nearest first: where its face's record starts in dface's cards
//! (5 a face, 2 faces a block, left first), the block's start (quarters ahead), and for an end wall how far along
//! it the ray meets it (quarters), else -1; then 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let mut start = -inp();
    let mut face = 0;
    for _ in 0..9 {
        let near = inp();
        let far = inp();
        for side in 0..2 {
            let quarters = inp();
            // how far behind the face the ray is at the block's near end, and at its far end
            let (past_near, past_far) = if side == 0 { (quarters - near, quarters - far) } else { (near - quarters, far - quarters) };
            if start < 880 && start + 104 > 0 {
                // along the end wall where the ray meets it, if it does; -1 for none
                let mut met = -1;
                if start > 0 && past_near >= 0 { met = past_near; }
                if (met >= 0 && met < 120) || (past_near < 0 && past_far >= 0) {
                    out(face);
                    out(start);
                    out(met);
                }
            }
            face += 5;
        }
        start = if start < 760 { start + 120 } else { 880 };   // past seeing, where it stays
    }
    out(0);
}
