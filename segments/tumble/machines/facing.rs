//! Facing: which faces of the tumble's cube are shown, once per frame. Deck: for each of the 6 faces, its light
//! from the small cube's shader (0 if hidden there, else 8-16), then its two diagonals as gather4 punched them from
//! the big cube's corners, a to c then b to d (x, y). Their cross product is twice the face's projected area with
//! a sign: negative faces the viewer, otherwise the face is hidden. These are the very corners the edges are drawn
//! from, and taking all four of them, a face whose corners have been rounded out of flat is shown if it shows as a
//! whole. Punches 0 for a hidden face, else its light, at least 8: a face nearly edge on that the small cube hides
//! still shows, as dim as a face can be.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..6 {                     // the faces
        let mut light = inp();
        let (across, down) = (inp(), inp());      // a to c
        let (across2, down2) = (inp(), inp());    // b to d
        // twice the area, minus the cross product: down across2 - across down2, positive when the face looks at
        // us. It runs past a card, so twice the area less 1 is kept as 576s and a rest 0-575, starting at -1 as -1
        // and 575: each product by adding |second| times (CARDIAC has no multiply), carrying a 576 whenever the
        // rest leaves 0-575. The face faces us when the 576s end 0 or more.
        let mut lots = -1;
        let mut rest = 575;
        let (mut a, mut b) = (down, across2);
        for _ in 0..2 {                 // the two products
            if b < 0 { a = -a; b = -b; }
            while b > 0 {
                rest += a;
                if rest >= 576 { rest -= 576; lots += 1; }
                if rest < 0 { rest += 576; lots -= 1; }
                b -= 1;
            }
            a = -across;                // then the second
            b = down2;
        }
        if light < 8 { light = 8; }
        if lots < 0 { light = 0; }      // hidden: it faces away, or has no area
        out(light);
    }
}
