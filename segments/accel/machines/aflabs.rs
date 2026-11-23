//! Spark streaks' lengths, per spark per frame of the accelerated drive while sparks fly. Deck: the way from head to
//! tail across and down (pixels). Punches the greater of their sizes, at least 1.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let a = abs(inp());
    let b = abs(inp());
    let m = if a > b { a } else { b };
    out(if m < 1 { 1 } else { m });
}
