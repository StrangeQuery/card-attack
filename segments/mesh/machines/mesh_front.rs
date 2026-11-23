//! Front, for the mesh kit: which triangles face the camera. Deck: the count, then for each triangle its plane's
//! distance less the margin, less its normal's dot with the camera (polydot's sums). Punches for each 1 if that is
//! below 0 (the camera is beyond the plane, or nearly: the face is seen), else 0.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        if inp() < 0 { out(1); } else { out(0); }
    }
}
