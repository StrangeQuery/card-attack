//! Lamps, once per sample column per frame of the accelerated drive: at each step of the march, how far along from the
//! block's lamp it is, and how far it came. Deck: how many steps, the first step's entry, part and place in the dash
//! (as below), then each: how many dashes (30 quarters) past the camera's block start and how far into the dash (16ths
//! of a quarter, amz's), how many entries of the plan past the camera's and how far into it (16ths of a quarter).
//! Punches each: how far from the lamp (quarters, at most 36: the lamps stand 60 into each block of 120) and where in
//! the dash it is counted on from the step before (16ths: the step before's place and how far it came since, at most
//! 999; the first step's own place).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let steps = inp();
    // the first step's entry, part and place (its own place is its first)
    let mut entry = inp();
    let mut part = inp();
    let mut last = inp();
    for _ in 0..steps {
        let mut dashes = inp();
        let dash = inp();
        let next_entry = inp();
        let next_part = inp();
        while dashes >= 4 { dashes -= 4; }
        let mut quarter = 30 * dashes;
        let mut sixteenths = dash;
        while sixteenths >= 16 { sixteenths -= 16; quarter += 1; }
        let lamp = abs_diff(quarter, 60);
        out(if lamp > 36 { 36 } else { lamp });
        // how far it came: 128 a plan entry, counted down from the room left under 999 (a card holds it), and no
        // further once that runs out. The room is kept less this step's part, which is added only where it's tested
        // (999 - last - next_part + part may pass a card, and is never stored)
        let mut room = 999 - last - next_part;
        while entry < next_entry { if room + part >= 0 { room -= 128; } entry += 1; }
        out(if room + part < 0 { 999 } else { 999 - (room + part) });
        last = dash;
        part = next_part;
    }
}
