// Card Attack's machine prelude, included by every machine after shared/cardiac.rs: demokit (colour, lookup
// tables, arithmetic, random numbers; a dependency in segments/Cargo.toml) and card I/O for its types. Functions
// here are small enough that rustc inlines them, so they cost nothing on CARDIAC.
#[allow(unused_imports)]
use demokit::{lcg, tables, Pixel};
#[allow(unused_imports)]
use demokit::arith::*;   // isqrt, abs, abs_diff, wrap, remainder, divide (a machine's own function of the same name wins)

/// Read a colour off the deck: three cards, red, green and blue.
#[allow(dead_code)]
#[inline(always)]
fn read_pixel() -> Pixel {
    Pixel { r: inp(), g: inp(), b: inp() }
}

/// Punch a colour: three cards, red, green and blue.
#[allow(dead_code)]
#[inline(always)]
fn punch(p: Pixel) {
    out(p.r);
    out(p.g);
    out(p.b);
}
