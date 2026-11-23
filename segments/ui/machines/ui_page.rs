//! Page, once a frame of a text page: where each character that has arrived goes, and the cursor after the last.
//! Deck: the advance (a character's width), the line step, the first line's top, how many characters show (the rest
//! are still to come), whether the cursor shows (1) or not (0), how many lines; then each line: how it sits (0
//! starting at its x, 1 centred on it), its x, how many characters. Punches each character shown's place, x and y,
//! in order; then, if the cursor shows and a character does, the cursor's place.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let advance = inp();
    let step = inp();
    let mut top = inp();
    let mut shown = inp();
    let cursor = inp();
    let lines = inp();
    // the cursor's place, just after the last character shown (none yet: -1)
    let mut after = -1;
    let mut after_top = 0;
    for _ in 0..lines {
        let centred = inp();
        let mut x = inp();
        let count = inp();
        if centred > 0 {
            let mut width = 0;
            for _ in 0..count { width += advance; }
            x -= width / 2;
        }
        for _ in 0..count {
            if shown > 0 {
                shown -= 1;
                out(x);
                out(top);
                x += advance;
                after = x;
                after_top = top;
            }
        }
        top += step;
    }
    if cursor > 0 && after >= 0 {
        out(after);
        out(after_top);
    }
}
