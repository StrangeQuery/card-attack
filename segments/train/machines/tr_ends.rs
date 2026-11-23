//! A streak's ends along its line, for the train's blur: from the place k (a long), the half length h (a card) and
//! the line's last place (a long), a = k - h - 1 (at least -1), b = k + h (at most the last) and the count b - a.
//! Deck: the count, then items: k, h, the last. Punches a, b (longs) and the count.
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let kh = inp();
        let kl = inp();
        let h = inp();
        let lh = inp();
        let ll = inp();
        let mut ah = kh;
        let mut al = kl - h - 1;
        if al < 0 { al += 999; al += 1; ah -= 1; }
        if ah < 0 { ah = -1; al = 999; }
        let mut bh = kh;
        let mut bl = kl - (999 - h);
        if bl > 0 { bl -= 1; bh += 1; } else { bl += 999; }
        if bh > lh || (bh == lh && bl > ll) { bh = lh; bl = ll; }
        let mut n = bl - al;
        if bh > ah { n += 999; n += 1; }
        out(ah);
        out(al);
        out(bh);
        out(bl);
        out(n);
    }
}
