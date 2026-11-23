//! Tone, for the mesh kit: the light's two dots made ready for the shade. Deck: the count, then each item's diffuse
//! dot and the lift's change to it, its half-way dot and the change to that (polydot's sums, thousandths; the changes
//! 0 when not lifted). Punches for each the diffuse in hundredths (0 when the dot is not above 0, at most 99) and the
//! half-way dot (0 likewise, at most 999: the highlight table's index). A dot and its change are never added past a
//! card: the change is tested against what is left to the most first.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut diffuse = inp();
        let change = inp();
        if diffuse <= 0 { diffuse = 0; } else if change > 989 - diffuse { diffuse = 989; } else { diffuse += change; }
        out(diffuse / 10);
        let mut half = inp();
        let change = inp();
        if half <= 0 { half = 0; } else if change > 999 - half { half = 999; } else { half += change; }
        out(half);
    }
}
