//! Tests, for the turn's hidden surfaces: comparisons of the scene's numbers read off their signs (sw_sign's). A test
//! is a pair's sign (1 if its difference is below 0, else 0; 1 if it is 0, else 0) and what it asks of it: whether
//! the difference is below 0, or (1 to count 0 in) at most 0, and (1 to turn the answer over) the opposite of either:
//! at least 0, above 0. Deck: the count; then each item: 0, how many tests and the tests, punching each test's answer
//! (1 yes, 0 no) and then 1 if every answer is yes (else 0); or 1, how many tests and the tests in pairs, punching the
//! second's answer of the first pair whose first is yes (0 if none is).
include!("../../shared/cardiac.rs");
include!("../../shared/prelude.rs");

pub fn main() {
    let count = inp();
    for _ in 0..count {
        let pairs = inp();
        let n = inp();
        let mut answer = 1 - pairs;
        let (mut found, mut when, mut second) = (0, 0, 0);
        for _ in 0..n {
            let mut yes = inp();
            let zero = inp();
            if inp() > 0 { yes += zero; }
            if inp() > 0 { yes = 1 - yes; }
            if pairs == 0 {
                out(yes);
                if yes == 0 { answer = 0; }
            } else if second == 0 {
                when = yes;
                second = 1;
            } else {
                if found == 0 && when == 1 { answer = yes; found = 1; }
                second = 0;
            }
        }
        out(answer);
    }
}
