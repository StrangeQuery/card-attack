//! Mix, for the mesh kit: a colour from its parts. Deck: the count; the ambient colour (three cards); then each item's
//! diffuse part and highlight part (three cards each: red, green, blue, from the material's tables). Punches for each
//! red, green and blue: the ambient plus the two parts, at most 255.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    let ambient = [inp(), inp(), inp()];
    for _ in 0..count {
        let diffuse = [inp(), inp(), inp()];
        for c in 0..3 {
            let v = ambient[c] + diffuse[c] + inp();
            out(if v > 255 { 255 } else { v });
        }
    }
}
