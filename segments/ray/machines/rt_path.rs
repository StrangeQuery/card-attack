//! Path, once per frame of the ray-traced ball over the plasma. Deck: the frame k, the script's ball x and height
//! for it, and the entrance's two arcs, each its first frame, length L (even) and height H. In an arc the ball is
//! H j (L - j) / (L / 2)^2 over the floor (rounded down), j frames into it: a parabola, so it drops in from above
//! the frame, bounces and lands where the script's own bounce starts; after the arcs it follows the script (14 is
//! on the floor). Punches the ball's x and height, and its shadow's radius, 44 less half the lift, and darkness,
//! 16 less a quarter of it, in 32nds: the higher the ball, the smaller and fainter.
// cardiac: phase-cards=9,0
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let k = inp();
    let ball_x = inp();
    let mut lift = inp() - 14;         // over the floor, from the script
    let mut j = -1;                    // frames into the arc the frame is in
    let mut length = 0;
    let mut height = 0;
    for _ in 0..2 {
        let first = inp();
        let l = inp();
        let h = inp();
        if k >= first && k < first + l {
            j = k - first;
            length = l;
            height = h;
        }
    }
    overlay();                         // the arc is found; the parabola loads over that
    if j >= 0 {
        let mut along = 0;             // j (L - j), by adding
        for _ in 0..j { along += length - j; }
        let (half, _) = divide(length, 2);
        let mut quarter = 0;           // (L / 2)^2, by adding
        for _ in 0..half { quarter += half; }
        // H j (L - j) / (L / 2)^2 without the product: H added j (L - j) times, (L / 2)^2 taken away when it fits
        lift = 0;
        let mut part = 0;
        for _ in 0..along {
            part += height;
            if part >= quarter {
                part -= quarter;
                lift += 1;
            }
        }
    }
    overlay();                         // the lift is found; the punching loads over that
    out(ball_x);
    out(14 + lift);
    let (half, _) = divide(lift, 2);
    let (fourth, _) = divide(lift, 4);
    out(44 - half);
    out(16 - fourth);
}
