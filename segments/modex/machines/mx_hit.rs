//! Hit, once per ball pixel of the Mode X ray tracer (320x240) whose ray reaches the floor. Deck: t (hundredths of
//! 529 pixels), the ray's X and Z (out of 529), the pixel's A and C, the ball's offset across from the middle. The
//! floor point, in pixels, is t X / 100 across from the pixel and t Z / 100 deeper than the ball's surface there
//! (C nearer than its centre), toward zero; in plasma units 9 / 8 of that (at the ball's depth a pixel is 9 / 8
//! plasma units), rounded down, the ball's centre at 105 along. Punches the look of plasma seen in the ball, 100 + 4
//! 32nds toward black, and the point (u, v). In two phases, to fit: the first reads the six cards and finds u.
// cardiac: phase-cards=6
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let t = inp();
    let ray_x = inp();
    let ray_z = inp();
    let a = inp();
    let c = inp();
    let across = inp();
    let u = plasma_units(across + a + along(ray_x, t));
    overlay();                         // u is found; v loads over it
    out(228);
    out(u);
    out(105 + plasma_units(along(ray_z, t) - c));
}

/// How far t hundredths of a ray goes along one direction, t v / 100 toward zero. t v can be past a card, so v is
/// added t times into a part kept under 100 either way, the hundreds carried out.
fn along(v: i16, t: i16) -> i16 {
    let mut hundreds = 0;
    let mut part = 0;
    for _ in 0..t {
        part += v;
        while part >= 100 { part -= 100; hundreds += 1; }
        while part <= -100 { part += 100; hundreds -= 1; }
    }
    hundreds
}

/// 9 h / 8 rounded down, as h and h / 8 rounded down (9 h can pass a card): eighths counted by taking 8s away.
fn plasma_units(h: i16) -> i16 {
    let mut rest = h;
    let mut eighths = 0;
    while rest >= 8 { rest -= 8; eighths += 1; }
    while rest < 0 { rest += 8; eighths -= 1; }
    h + eighths
}
