// The card sorter. Wherever the host puts cards in order by numbers machines punched, it does it as a card sorter
// does, never by comparing two numbers: each pass reads one column of every card's key, drops the card into that
// column's pocket (0 to 9), and gathers the pockets in order, pocket 0's cards first, each pocket's cards in the order
// they went in. The passes run from the key's last column to its first, so after the last pass the deck is in the
// key's order, and cards with equal keys are still in the order they came in (the sort is stable).
//
// A key is one or more fields, each a whole number punched by a machine: a card (three digits and a sign) or several
// cards read as one number (a long is six digits). The deck is sorted by the last field first and the first field
// last, so the first field decides and the later ones break its ties. A field's columns are its digits, the units
// first, then its sign:
//
// - the digits of the number's size, one pass a digit;
// - the sign last, as its own pass into two pockets, minus then plus. On the minus cards the digit passes read each
//   digit's nines' complement (9 - d) instead, so that a bigger size goes into an earlier pocket: -1 (998 on three
//   digits) comes after -999 (000), and the minus cards come out smallest first.
// - A field sorted high to low (`down`) is read the other way throughout: the nines' complement of every digit on
//   the plus cards, the plain digits on the minus cards, and the plus pocket gathered before the minus pocket. That
//   is a sorter operator's way of sorting descending, and it keeps equal keys in the order they came in.
//
// A field a key does not have (keys of different lengths) is a blank column: those cards go into a pocket gathered
// ahead of pocket 0, so a shorter key comes before every longer key it begins.
//
// A pass that would drop every card into one pocket leaves the deck as it was, and is not made.
//
// Keys must fit their fields: the sorter stops on a number with more digits than its field has.
//
// This file is the crate module `sorter` and is also included by segments/train/ops.rs, so the renderers the host
// shares with the references use the same sorter.

/// One field of a key: how many digits it has, and whether it sorts low to high or high to low.
#[derive(Clone, Copy, Debug)]
pub struct Field { pub digits: u32, pub down: bool }

/// A field of `digits` digits, sorted low to high.
pub const fn up(digits: u32) -> Field { Field { digits, down: false } }
/// A field of `digits` digits, sorted high to low.
pub const fn down(digits: u32) -> Field { Field { digits, down: true } }

/// A card, low to high.
pub const CARD: Field = up(3);
/// A long (two cards read as one number, six digits), low to high.
pub const LONG: Field = up(6);

/// The pocket a field's value goes into on column `col` (0 the units, `digits` the sign); 0 is the blank pocket and
/// 1.. the sorter's pockets in the order they are gathered.
fn pocket(value: Option<i64>, f: Field, col: u32) -> usize {
    let Some(v) = value else { return 0 };
    let minus = v < 0;
    if col == f.digits { return if minus != f.down { 1 } else { 2 }; }
    let d = (v.unsigned_abs() / 10u64.pow(col) % 10) as usize;
    1 + if minus != f.down { 9 - d } else { d }
}

/// The order a sorter leaves a deck in: the cards' places in the deck it was given, in the order they come out.
/// `keys[i]` is card i's key, one number a field.
pub fn order<K: AsRef<[i64]>>(keys: &[K], fields: &[Field]) -> Vec<usize> {
    for k in keys {
        let k = k.as_ref();
        assert!(k.len() <= fields.len(), "sorter: a key of {} fields, the sorter has {}", k.len(), fields.len());
        for (f, &v) in fields.iter().zip(k) { assert!(v.unsigned_abs() < 10u64.pow(f.digits), "sorter: {v} does not fit {} digits", f.digits); }
    }
    let mut deck: Vec<usize> = (0..keys.len()).collect();
    let mut next = vec![0; keys.len()];
    for (fi, &f) in fields.iter().enumerate().rev() {
        for col in 0..=f.digits {
            let at = |i: usize| pocket(keys[i].as_ref().get(fi).copied(), f, col);
            let mut count = [0usize; 12];
            for &i in &deck { count[at(i)] += 1; }
            if count.iter().any(|&c| c == deck.len()) { continue; }
            // each pocket's first place in the gathered deck
            let mut start = [0usize; 12];
            for p in 1..12 { start[p] = start[p - 1] + count[p - 1]; }
            for &i in &deck { let p = at(i); next[start[p]] = i; start[p] += 1; }
            std::mem::swap(&mut deck, &mut next);
        }
    }
    deck
}

/// Sort `items` by their keys, as `order` does.
pub fn sort<T, K: AsRef<[i64]>>(items: &mut Vec<T>, fields: &[Field], key: impl Fn(&T) -> K) {
    let keys: Vec<K> = items.iter().map(&key).collect();
    let deck = order(&keys, fields);
    let mut was: Vec<Option<T>> = items.drain(..).map(Some).collect();
    items.extend(deck.into_iter().map(|i| was[i].take().unwrap()));
}

/// The sorted deck in runs of equal keys: each run the places of its cards in the deck given, in their order.
pub fn groups<K: AsRef<[i64]>>(keys: &[K], fields: &[Field]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![];
    for i in order(keys, fields) {
        match out.last_mut() {
            Some(g) if keys[g[0]].as_ref() == keys[i].as_ref() => g.push(i),
            _ => out.push(vec![i]),
        }
    }
    out
}

/// Sort `items` by their keys and drop the repeats as they come out of the pockets, keeping the first card of each
/// key.
pub fn distinct<T, K: AsRef<[i64]>>(items: &mut Vec<T>, fields: &[Field], key: impl Fn(&T) -> K) {
    let keys: Vec<K> = items.iter().map(&key).collect();
    let firsts: Vec<usize> = groups(&keys, fields).into_iter().map(|g| g[0]).collect();
    let mut was: Vec<Option<T>> = items.drain(..).map(Some).collect();
    items.extend(firsts.into_iter().map(|i| was[i].take().unwrap()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(n: usize, seed: u64, span: i64) -> Vec<i64> {
        let mut s = seed;
        (0..n).map(|_| { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 33) as i64 % (2 * span + 1)) - span }).collect()
    }

    #[test]
    fn matches_a_stable_sort() {
        for seed in 0..40 {
            let a = deck(500, seed, 999);
            let b = deck(500, seed + 1000, 3);
            let c = deck(500, seed + 2000, 999_999);
            let idx: Vec<usize> = (0..a.len()).collect();
            // one field each way
            let mut want = idx.clone(); want.sort_by_key(|&i| a[i]);
            assert_eq!(order(&idx.iter().map(|&i| [a[i]]).collect::<Vec<_>>(), &[CARD]), want);
            let mut want = idx.clone(); want.sort_by_key(|&i| -a[i]);
            assert_eq!(order(&idx.iter().map(|&i| [a[i]]).collect::<Vec<_>>(), &[down(3)]), want);
            // two and three fields, mixed directions
            let mut want = idx.clone(); want.sort_by_key(|&i| (b[i], -a[i]));
            assert_eq!(order(&idx.iter().map(|&i| [b[i], a[i]]).collect::<Vec<_>>(), &[up(1), down(3)]), want);
            let mut want = idx.clone(); want.sort_by_key(|&i| (std::cmp::Reverse(b[i]), c[i], a[i]));
            assert_eq!(order(&idx.iter().map(|&i| [b[i], c[i], a[i]]).collect::<Vec<_>>(), &[down(1), LONG, CARD]), want);
            // sort, distinct
            let mut v: Vec<(i64, i64)> = idx.iter().map(|&i| (b[i], a[i] / 100)).collect();
            let mut w = v.clone(); w.sort(); w.dedup();
            distinct(&mut v, &[up(1), CARD], |t| [t.0, t.1]);
            assert_eq!(v, w);
            // keys of different lengths: a shorter one first, as Vec's order
            let vs: Vec<Vec<i64>> = idx.iter().map(|&i| a[i..(i + (b[i].unsigned_abs() as usize)).min(a.len())].iter().map(|x| x / 300).collect()).collect();
            let mut want = idx.clone(); want.sort_by(|&p, &q| vs[p].cmp(&vs[q]));
            assert_eq!(order(&vs, &[CARD; 3]), want);
        }
    }
}
