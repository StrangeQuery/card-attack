//! Test, once per frame of the polys: the blobs' shadows on each other. Deck: the count, then for each pixel its
//! depth and blob, the shadow map's depth and blob where it falls (999 and 0 if nothing is there), and its diffuse
//! light and highlight (in thousandths). The pixel is in shadow if the other blob is there nearer the light. Punches
//! the diffuse light and the highlight, or 0 and 0 in shadow.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let depth = inp();
        let blob = inp();
        let shadow = inp();
        let caster = inp();
        let diffuse = inp();
        let highlight = inp();
        if caster != blob && shadow < depth {
            out(0);
            out(0);
        } else {
            out(diffuse);
            out(highlight);
        }
    }
}
