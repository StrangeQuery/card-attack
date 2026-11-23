//! Rim, once per frame of the polys: the light catching each covered pixel's edge, and its diffuse light held at 0
//! or above. Deck: the count, then each pixel's diffuse light and depth part (in thousandths, the depth part 0 or
//! below). Punches the diffuse light (0 if below) and the rim, 0.35 (1 - |depth|)^3, from a table (every 0.05) in
//! tenths of a step of light.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let rim: [i16; 21] = load_resource("rim");  // the host punches demokit's table in here
    let count = inp();
    for _ in 0..count {
        let diffuse = inp();
        out(if diffuse < 0 { 0 } else { diffuse });
        let edge = 2 * (-inp() / 10) / 10;
        // SAFETY: the depth part is -999..0, so edge is 0..19
        unsafe { assume(edge >= 0); assume(edge <= 20); }
        out(rim[edge as usize]);
    }
}
