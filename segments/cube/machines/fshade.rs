//! Face shader. Deck: for each of the 6 faces, three consecutive corners a, b, c as projected (x, y). The cross
//! product (b - a) x (c - b) is the face's projected area with a sign: negative faces the viewer, otherwise the face
//! is hidden. A face's projected area is how squarely it faces us, so it is also the light, with the lamp at the
//! eye. Punches 0 for a hidden face, else its brightness 8 + area/72 (at most 16).
// cardiac: replaces cube/original/face_shade.asm
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    for _ in 0..6 {                     // the faces
        let (ax, ay) = (inp(), inp());
        let (bx, by) = (inp(), inp());
        let (cx, cy) = (inp(), inp());
        // the projected area, minus the cross product: (ax - bx)(cy - by) + (by - ay)(cx - bx), positive when the
        // face looks at us. Each product by adding |second| times (CARDIAC has no multiply), both in one loop: a
        // multiply helper called twice costs more cells than this machine has to spare.
        let mut area = 0;
        for product in 0..2 {
            let (mut a, mut b) = if product == 0 { (ax - bx, cy - by) } else { (by - ay, cx - bx) };
            if b < 0 { a = -a; b = -b; }
            while b > 0 { area += a; b -= 1; }
        }
        if area <= 0 {
            out(0);                     // hidden
        } else {
            let mut brightness = 8;     // 8 + area / 72, at most 16, by subtracting
            while area >= 72 && brightness < 16 { area -= 72; brightness += 1; }
            out(brightness);
        }
    }
}
