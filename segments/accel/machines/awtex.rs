//! Wall texels, as many as the host has, per frame of the accelerated drive: where each texel row's colour is. A
//! floor of the window texture is 12 texels up; a block's top four rows are its wall and ledge. Deck: how many, then
//! each: the texel up, the block's height (half units), the texture's column (awbay's). Punches each: -1 for the wall
//! under the ledge, -2 the ledge, else the window kinds' place (data/kinds.txt: the column plus the floor, 0-31, at
//! most 959); then the texel's place in its floor (0-11).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let mut up = inp();
        let roof = up - 2 * inp();
        let mut place = inp();
        let mut floors = 0;
        while up >= 12 { up -= 12; if floors < 31 { floors += 1; } }
        while up < 0 { up += 12; }
        place += floors;
        if place > 959 { place = 959; }
        if roof >= -4 { place = -1; }
        if roof >= -2 { place = -2; }
        out(place);
        out(up);
    }
}
