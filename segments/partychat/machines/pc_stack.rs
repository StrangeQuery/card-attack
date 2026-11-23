//! Stack, once per frame of the party chat. Deck: the frame, how many messages, then each one's frame it posts at
//! and its height (pc_wrap's), the newest first. The newest posted sits at the bottom, sliding up into place over its first four
//! frames (pushing the others up), each before it above it, until one would be wholly above the head. Punches how
//! many have posted, then the top of each shown, the newest first.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

const BOTTOM: i16 = 746;
const TOP: i16 = 62;
const SLIDE: i16 = 4;

pub fn main() {
    let frame = inp();
    let messages = inp();
    // the ones not posted yet (the newest first): passed over
    let mut left = messages;
    let mut at = inp();
    let mut h = inp();
    while at > frame {
        left -= 1;
        at = inp();
        h = inp();
    }
    out(left);
    // the newest: its height, how far it still has to slide (its height times the frames left of the slide, a
    // quarter of it each)
    let since = frame - at;
    let mut y = BOTTOM - h;
    if since < SLIDE - 1 {
        let mut push = 0;
        for _ in since..SLIDE - 1 { push += h; }
        let (q, _) = divide(push, SLIDE);
        y += q;
    }
    out(y);
    // the ones before it, each its height above the last, while any of it is below the head
    for _ in 1..left {
        inp();
        let h = inp();
        y -= h;
        if y + h < TOP { break; }
        out(y);
    }
}
