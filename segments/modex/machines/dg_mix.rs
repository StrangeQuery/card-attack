//! Mix, once per frame of the jolt: each colour the jolt brings to a pixel with the swirl's tint there, brightened
//! by the flash. Deck: the flash (32nds over 32), how many; then each pair, the colour (0..255) and the tint.
//! Punches each (colour + tint) (32 + flash) / 32 rounded down, held to 0..255.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let flash = inp();
    let count = inp();
    for _ in 0..count {
        let v = inp() + inp();
        if v <= 0 {
            out(0);
        } else {
            // v flash / 32 as v's 32s times the flash and the rest's share
            let (whole, rest) = divide(v, 32);
            let mut lit = v + muldiv(rest, flash, 32);
            for _ in 0..flash { lit += whole; }
            if lit > 255 { lit = 255; }
            out(lit);
        }
    }
}
