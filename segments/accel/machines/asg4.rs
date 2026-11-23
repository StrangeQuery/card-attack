//! Street dash spans, the software half's, once per fourth row of the street per frame of the drive: asgap's spans
//! made ready for each 4 x 4 block's middle only. Deck: asgap's 24 spans. Punches each with its start 5 times (pixel
//! 2's 5 / 64 of the gap) and its step 4 times (a block's), each a whole and 64ths.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..24 {
        out(inp());
        out(inp());
        let (w0, p0, s0, h0) = (inp(), inp(), inp(), inp());
        let (mut whole, mut part, mut step, mut half) = (0, 0, 0, 0);
        for k in 0..5 {
            whole += w0;
            part += p0;
            if part >= 64 { part -= 64; whole += 1; }
            if k < 4 { step += s0; half += h0; if half >= 64 { half -= 64; step += 1; } }
        }
        out(whole);
        out(part);
        out(step);
        out(half);
    }
}
