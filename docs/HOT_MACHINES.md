# Writing the hot machines by hand

> A record of work as it happened, in October and November 2026. The numbers, machines and segments each entry names are as they were that day; many segments and
> machines have since been removed or changed. For the repository as it is now, see the [README](../README.md) and
> [RUST.md](RUST.md).

Card Attack's machines are written in Rust and compiled to CARDIAC tapes by rust-cardiac. In an earlier test,
hand-written CARDIAC for a few machines beat the compiler's instruction counts. The film fits its
five minutes (see [PERFORMANCE.md](PERFORMANCE.md)), but every instruction saved is headroom for the work still
to come. This is the search for the machines worth
writing by hand, what we found in them, and what each bought.

Apple M5 Ultra (10 of its fastest cores, 20 of the next), the live player's 28 workers.

## Where the time goes

A host efficiency sweep (`sample` on a full `card-attack play --check`) put 77.6% of the workers' time
in the interpreter running machine instructions, about 4,200 of 5,370 worker-seconds. The host already counted
instructions per machine, but only for offline renders. So step one was making the live run say which machines
take them: each `Host` adds its per-machine counts into the process's totals as it drops, and `play --check`
prints the top 30.

The film: 3,636G instructions over 427 machines. It is not spread out:

| Machine | Instructions, G | Share | Cumulative | Runs | Per run |
|---|---|---|---|---|---|
| tr_add | 540.1 | 14.9% | 14.9% | 22,319,534 | 24,200 |
| tr_sum | 321.6 | 8.8% | 23.7% | 4,160,387 | 77,298 |
| tr_walka | 266.1 | 7.3% | 31.0% | 614,485 | 433,100 |
| tr_qa | 226.0 | 6.2% | 37.2% | 11,533,609 | 19,591 |
| sw_lsum | 188.8 | 5.2% | 42.4% | 3,678,156 | 51,322 |
| tr_addc | 163.6 | 4.5% | 46.9% | 5,055,439 | 32,358 |
| sw_qis | 147.3 | 4.0% | 51.0% | 4,838,688 | 30,433 |
| sw_divd | 137.9 | 3.8% | 54.8% | 2,772,136 | 49,741 |
| tr_max | 124.1 | 3.4% | 58.2% | 9,187,150 | 13,512 |
| tr_min | 120.8 | 3.3% | 61.5% | 9,454,281 | 12,778 |

Four machines are over a third of the film; fifteen are three quarters. Nearly all are the train's operations
(`tr_*`, the arithmetic the train, the night, the stadium and the rooms are drawn with) and the swarm's calculator
(`sw_*`): small loops over long numbers, two cards with a carry between them, run millions of times. That is the
shape of code a person writing CARDIAC by hand does well.

## tr_add

A long plus a constant, and maybe a second long added or taken away (`segments/train/machines/tr_add.rs`). Deck:
the count, k (1 or 2), the second's sign, the constant's two cards, then the items. A long is two cards, the
thousands and the units (0..999).

### What the compiler made of it

The listing (`cargo cardiac build --bin tr_add --emit listing`): 91 cells, and for the commonest case 26-28
instructions an item. Three things stand out:

- **`999 - cl` is worked out for every item** (`CLA k999; SUB cl; STO t`), though the constant's low card never
  changes in a run. The compiler didn't hoist it out of the loop.
- **`k == 2` is tested for every item** (five instructions), though k is the same for the whole deck.
- **Every partial result goes to a card and back** (`STO hi`, then `CLA hi` to add the carry).

### Which case matters

A machine for each case is the natural hand-written answer, but they had to be worth it. A tally over one live run:

| Case | Items | Share |
|---|---|---|
| one long and the constant | 4.67G | 35% |
| a second long added | 6.84G | 51% |
| a second long taken away | 1.77G | 13% |

13.3G items for 540G instructions: about 41 instructions an item, the loader in.

### By hand

Three machines, one a case, each taking tr_add's deck unchanged (so either can run on the same cards):
`segments/train/hand/` (`tr_add1.asm`, `tr_add2p.asm`, `tr_add2m.asm`; not beside `tr_add.rs`, where the compiler writes its own `.asm`). The host picks the
machine by the terms' count and signs, which are the same for a whole call, and deals it each item whose values are
longs (a comparison of each value with a long's limits).

- `999 - cl` once a run, as `m = cl - 999`: `lo + m` is over 0 exactly when the lows carry.
- No k test in the loop: the case is the machine.
- **The high summed once, after the carries are known**, in the accumulator (four digits): `hi + ch + bh` plus
  0, 1 or 2 carries, one `STO`.
- With no first carry the accumulator already holds `lo + cl - 1000`, so the second carry's test is one `ADD bl`
  away, not a reload and a recompute.

### The bug the bench found

A bench (scratch crate on the host library) ran the compiled tape and the hand ones on the same random decks,
edges weighted (lows of 0 and 999, constants near the limits), and checked both against the true sums. The hand
machines were right on every item. The compiled tr_add was not:

    k1: c -163495, item [-836, 663]: [1, 168] for [-999, 168]

-835,337 + -163,495 = -998,832: a long, [-999, 168]. The compiled code stores the partial high `hi + ch` = -1000
before adding the carry, and a card holds ±999: the -1000 wraps, and the high comes out 1. The operation
guarantees the final sum is a long, not that partial. The native Rust build, on i16, gets it
right; only the tape is wrong, and only at the -1000 edge, in all three cases. The film's 3,419 frames were
identical to the reference before and after, so its decks never hit it: a latent wrong answer, not a visible one.
The first hand k = 2 versions had the same flaw (a partial high on a card) and were rewritten to sum the high
once; that was also cheaper.

### What it bought

The bench, 2.57M items a case (instructions an item, the loader in):

| Case | Compiled | By hand | Change |
|---|---|---|---|
| one long and the constant | 27.3 | 18.7 | -31% |
| a second long added | 49.8 | 25.2 | -49% |
| a second long taken away | 45.1 | 24.7 | -45% |

Weighted by the film's mix: 41 to about 23 an item, about 240G instructions, 6.6% of the film.

Live, `play --check`:

| | Before | tr_add by hand |
|---|---|---|
| tr_add's instructions, G | 540.1 | 300.1 (tr_add2p 166.7, tr_add1 87.8, tr_add2m 45.6) |
| The film's instructions, G | 3,636.4 | 3,396.4 (-6.6%) |
| Each worker busy, s | 184.9-198.7 | 175.6-187.9 |
| All computed in, s | 202.6 | 192.4 |
| Worst slack, s | -0.10 (frame 0) | -0.10 (frame 0) |
| Frames identical | 3419/3419 | 3419/3419 |

One machine, three short hand-written tapes: about ten seconds off every worker.

## tr_sum

Sums of any number of longs, each added or taken away (`tr_sum.rs`; the host's fallback when a sum isn't one
tr_add or tr_addc can take). Deck: the count, how many terms; then each term's sign and long. Compiled: 71 cells,
27-28 instructions an added term, 20-26 a taken one, and every sum starting from 0.

By hand (`segments/train/hand/tr_sum.asm`, tr_sum's deck):

- the sign tested by `TAC` alone (-1 is negative), not `SUB 00; TAC`;
- the low stored before the carry test, so with no carry it is already right;
- the carry or borrow added to the high in the accumulator (never a partial high on a card: the same -1000 wrap as
  tr_add's turned up in the compiled tr_sum on the bench);
- **the first term loaded straight in**: from 0 there is nothing to carry, so a sum's first term is a load, not an
  add with its test.

The bench, 1.33M sums a size, every one right by hand (instructions a sum, the loader in):

| Terms | Compiled | By hand | Change |
|---|---|---|---|
| 1 | 39.7 | 25.2 | -37% |
| 2 | 65.8 | 44.7 | -32% |
| 3 | 91.8 | 64.3 | -30% |
| 4 | 117.8 | 83.8 | -29% |
| 6 | 169.5 | 122.6 | -28% |

The first version, without the first-term load, was 13-22%: one idea, worth more than the rest together for short
sums.

Live, `play --check` (with the scheduler's setups-first fix in):

| | tr_add by hand | and tr_sum |
|---|---|---|
| tr_sum's instructions, G | 321.6 | 226.1 (-30%) |
| The film's instructions, G | 3,396.4 | 3,300.8 |
| Each worker busy, s | 179.4-181.6 | 173.5-175.7 |
| All computed in, s | 185.7 | 179.7 |
| Frames identical | 3419/3419 | 3419/3419 |

## tr_walka

Walks by slopes, for the train's edges and spans: a value carried across n places, each step the slope's whole
part and one more whenever its thousandths carry (`tr_walka.rs`). Deck: the count; each walk n, the start, the
slope (a long and its thousandths), f0. Compiled: 92 cells, about 35 instructions a place, and two loop-invariants
worked out again at every place (`999 - w_lo` and `999 - frac`: rust-cardiac#127 again).

By hand (`segments/train/hand/tr_walka.asm`, tr_walka's deck):

- **the fraction first**, so its carry is just one more on the low's add: `v + w + 1` carries at most once, where
  the compiled code adds w with its carry test and then the fraction's 1 with a second test (`v_lo == 999`);
- each low stored before its carry test (the no-carry path leaves it as it is), the high once in the accumulator;
- the place count a long counted down, the thousands borrowed only when the low is spent.

The bench (instructions a place, the loader in; every place right by hand):

| Walks | Places | Compiled | By hand | Change |
|---|---|---|---|---|
| 0-20 places | 0.21M | 40.5 | 26.7 | -34% |
| 0-400 | 3.8M | 38.1 | 24.6 | -36% |
| 0-2500 | 24.6M | 38.0 | 24.5 | -36% |

The compiled one was wrong on 2 of the 24.6M (a -1000 wrap: a partial high on a card).

Live: tr_walka's 266.1G to 172.9G (-35%); the film 3,207.7G; each worker busy 169.3-171.5 s; all computed in
175.6 s; 3419/3419 identical.

## tr_qa

The first half of a product (`tr_qa.rs`): a b as (x + y)^2 / 4 - (x - y)^2 / 4, x and y the sizes; it punches the
two sizes to look up in the quarter-square table, sum first when the signs agree, difference first when they
don't. Compiled: 90 cells, about 47 instructions a pair. It keeps each sign as a flag in a cell, then compares the
flags (an `==` is two tests), and works out `999 - y` for the sum's carry.

By hand (`segments/train/hand/tr_qa.asm`, tr_qa's deck): the four sign cases go straight to the two orders, no
flags; a size is negated in its own cell only when it is negative; the sum's carry by storing it first.

Its input is two cards, so the bench could try **every pair**, all 3,996,001: identical to the compiled machine on
every one, 46.9 instructions a pair down to 30.9 (-34%).

Live: tr_qa's 226.0G to 140.7G (-38%); the film 3,122.4G; each worker busy 165.7-167.6 s; all computed in 171.5 s;
3419/3419 identical.

## sw_lsum

The swarm calculator's long sums (`segments/swarm/machines/sw_lsum.rs`), three cards a number (millions,
thousands, units), loaded by the swarm, the turn, the mesh and the train's operations alike. So from here on a hand
machine that takes a Rust machine's whole deck lives at `PART/hand/NAME.asm`, and the host's `rust()` loads it in
the Rust machine's place, under the same name, with the Rust machine's native build beside it for native checks.
tr_sum, tr_walka and tr_qa follow that rule (tr_add's three per-case machines are loaded on their own, and the host picks one by the terms' count and signs).

By hand: the first term loaded straight in, each card stored before its carry test, the units' carry one more on
the thousands' add (the compiled code tests `h1 == 999` on its own), the millions' carry in the accumulator.

The bench, 352k sums a size (instructions a sum; every sum right by hand, the compiled one wrong on 69-190 a size,
the same wrap):

| Terms | Compiled | By hand | Change |
|---|---|---|---|
| 0 | 16.8 | 12.7 | -24% |
| 1 | 49.8 | 19.7 | -60% |
| 2 | 85.5 | 45.0 | -47% |
| 3 | 121.4 | 70.5 | -42% |
| 6 | 228.9 | 146.9 | -36% |

Live: sw_lsum's 188.8G to 109.6G (-42%); the film 3,043.1G; each worker busy 164.5-166.4 s; all computed in
170.3 s; 3419/3419 identical.

## tr_addc

Sums of cards, each term added or taken away, and a constant (`tr_addc.rs`): the host's first choice for an item
of at most six cards and a constant card (each value compared with a card's limits). The deck gives the signs once, in its header (up to six); the compiled machine
keeps them in an array and walks it for every sum: an indexed read on CARDIAC is an address worked out and an
instruction built, every term, every sum.

By hand (`segments/train/hand/tr_addc.asm`): **the program writes itself.** Once a run, from the signs, each term's
slot gets `ADD t` or `SUB t` (the `STO` that writes them walks along the slots, its own address bumped by two), and
the slot after the last term becomes a jump to the end. A sum is then straight-line: `CLA c`, `INP t` and the
patched `ADD t` or `SUB t` per term (INP doesn't touch the accumulator), `STO s`, `OUT s`: 9 + 2k instructions for
k terms. The tape is loaded fresh every run, so the patching never leaks from one deck to the next.

(Benching it, the new `PART/hand` rule loaded the hand tape on both sides: both columns came out identical at 9 + 2k
until the file was moved aside.)

| Terms | Compiled | By hand | Change |
|---|---|---|---|
| 0 | 14.3 | 9.2 | -35% |
| 1 | 32.7 | 11.2 | -66% |
| 2 | 51.3 | 13.2 | -74% |
| 3 | 69.8 | 15.3 | -78% |
| 6 | 121.3 | 21.3 | -82% |

Live: tr_addc's 163.6G to 43.1G (-74%); the film 2,922.6G; each worker busy 155.5-157.2 s; all computed in
161.1 s; 3419/3419 identical.

This one isn't for the compiler: it's what CARDIAC's stored program is for, code that is data.

tr_addc punches each sum as two cards, its thousands and the rest, both with the sum's sign: after the last term
`STO s` keeps the last three digits and `SFT 03` leaves the thousands (12 + 2k instructions). Seven cards add up to
at most 6,993, which the accumulator's four digits hold, so every sum of at most six cards and a constant card
comes out whole whatever its partial sums pass on the way, and the host deals it every such item by comparing each
value with a card's limits, item by item. tr_add's machines keep the high's partials in the accumulator too, so
they take every item whose values are longs. Over the film: 18.9G items through `sum`, 7.6G of them all cards; the
adders' 574.0G (tr_sum 226.1, tr_add2p 166.7, tr_add1 87.9, tr_addc 47.3, tr_add2m 46.0) to 536.0G (tr_sum 185.9,
tr_addc 123.8, tr_add2p 103.4, tr_add1 83.1, tr_add2m 39.8); the film 2,679.1G to 2,632.0G; 3419/3419 identical.

## sw_qis

The swarm's signed quarter squares (`sw_qis.rs`): the two sizes to look up for a product, in one order or the other
by the parity of three sign bits. By hand: **the parity as a computed jump**. `JMP table` plus the three signs' sum
(0 to 3) is built in the accumulator and stored into the very next cell, which then runs: no extra jump, no tests of
the sum. The four orders (with or without the sum's thousand, even or odd) are four tails, so the thousand is never
a cell. The input is small enough to try everything: all 8,000,000 (every a, b in 0..999 under each of the eight
sign patterns), identical to the compiled machine, 48.8 instructions an item down to 34.0 (-30%).

Live: 147.3G to 106.8G (-27%).

## sw_divd

The swarm's long division (`sw_divd.rs`): a three-card size by a long, a digit at a time, the long taken away while
it fits. Compiled, each digit's shift is an instruction built at run time (`SFT (k+1)3`), and each subtraction is
worked out beside the remainder and copied over when it fits (17 instructions a fit, 24 with a borrow). By hand:

- the three digits by fixed shifts, `SFT 13`, `SFT 23`, `SFT 33`, each followed by a call of one routine with the
  digit in the accumulator; the routine keeps cell 99 (where every `JMP` leaves its way back) and returns through
  it;
- the remainder's shift as `SFT 13` (its hundreds) and `SFT 21` (the rest's last two digits, times ten);
- the long taken away in place, put back once when it no longer fits: 11 a fit, 13 with a borrow.

Against true division on 250k divisions a size: both right, 23-28% fewer by hand (207 to 159 for one card, 961 to
694 for three). The counter (`f + 1` a fit, three instructions) is now most of what's left.

Live: 137.9G to 102.2G (-26%). With both: the film 2,846.4G; each worker busy 154.0-154.7 s; all computed in
158.5 s; 3419/3419 identical.

tr_max and tr_min (3.4% and 3.3%) were looked at and left: 17 instructions a pair compiled, one wasted `STO`; only
unrolling would buy much (about 12%).

## tr_qb, sw_mag, sw_qp

Three smaller ones, the same ideas:

- **tr_qb** (a product's second half: the difference of two quarter squares, as a long or over 1000 rounded down):
  the rounding `s` is the same for a whole deck, so a loop for each value instead of a test every pair; the
  over-1000 loop never keeps the low card at all. 28.3 instructions an item to 20.2 (-29%) on the bench; live
  113.2G to 85.3G (-25%).
- **sw_mag** (a number's sign and size, one card or three): a one-card number never built into three cards first
  (below 0 its size is just `0 - h0`), each case straight to its own punches. 35.0 to 23.3 (-34%); live 105.4G to
  72.0G (-32%).
- **sw_qp** (a product's difference placed as three cards): the place by a computed jump, as sw_qis's parity. The
  first version gave each place its own copy of the difference and ran past cell 98: 96 cells is the limit that
  bites. Sharing the difference costs a reload an item. 34.9 to 30.1 (-14%); live 104.9G to 94.0G (-10%).

All three identical to the compiled machines on 1.07M items each. With them: the film 2,774.3G; each worker busy
151.6-153.3 s; all computed in 157.2 s; 3419/3419 identical.

## tr_ldiff, tr_mixa, tr_up4, tr_pctb

- **tr_ldiff** (a difference of longs held between two bounds): the high once in the accumulator, borrow and all;
  each bound's punches straight from the constants. 30.5 to 25.4 (-17%); live 103.7G to 89.8G.
- **tr_mixa** (a mix's product, by quarter squares): tr_qa's hand body with `b - a` worked out first. 51.6 to
  35.5 (-31%); live 77.4G to 51.3G.
- **tr_pctb** (a product over 100, toward zero, plus a): by shifts, `hi 10 + lo / 100` as `SFT 10` and `SFT 13`,
  one more when hi is below 0 and `SFT 22` (the last two digits) left a rest. 41.6 to 33.4 (-20%); live 68.2G to
  53.2G.
- **tr_up4** (glow four times as fine: a, a + q, a + h, a + h + q between each value and the next): the compiler
  had already found the halving trick (five times, then `SFT 01`, which cuts toward zero as Rust's `/` does), so
  the first hand version saved only 7%. Counting the steps down once a line, and building `a + q` from the `q`
  still in the accumulator, then `a + h + q` from that and `a + h` from it, took it to 39.5 to 33.6 (-15%); live
  79.7G to 67.5G.

All four identical to the compiled machines on the bench. With them: the film 2,707.2G; each worker busy
148.7-152.1 s; all computed in 156.2 s; 3419/3419 identical.

## tr_max, tr_min, tr_lt

The pair comparisons of longs: the greater, the lesser, and whether a is below b. Compiled they are tight already
(17-20 instructions a pair, one spare `STO`); what's left is the loop. By hand, from one template (a few lines of
Python writing the three): **two pairs a loop**, the count tested once for two, the second pair's "a below b"
falling straight through to the loop's head, an odd last pair on its own. On every deck length 0..40 and long ones:
identical; 18.5 to 15.4 a pair (-17%) for max and min, 19.8 to 14.4 (-28%) for lt. Live: 124.1G to 105.7G, 120.8G
to 97.6G, 95.9G to 68.9G. The film 2,638.6G; each worker busy 149.0-150.8 s; all computed in 154.8 s.

## The film

| | Before | After |
|---|---|---|
| The film's machine instructions, G | 3,636.4 | 2,638.6 (-27.4%) |
| Each worker busy, s | 184.9-198.7 | 138.8-141.5 |
| All computed in, s | 202.6 | 145.6 |
| Frames identical | 3419/3419 | 3419/3419 |

Eighteen machines by hand (tr_add, tr_sum, tr_walka, tr_qa, sw_lsum, tr_addc, sw_qis, sw_divd, tr_qb, sw_mag,
sw_qp, tr_ldiff, tr_mixa, tr_up4, tr_pctb, tr_max, tr_min, tr_lt); the scheduler's setups-first;
and the host's own fixes from the efficiency sweep: the triangles' `whole` test by flag, the deck range check as two
plain scans, the glow's rounds by a kept stamp; `sum` reading its terms straight from their sources; the
machines and tables loaded once a process. One sweep item, punching straight
into the caller's vector, measured no better and was dropped.

The film is 284.9 s; it is now all computed in about half that.

## The compiler and the interpreter

With the hot machines by hand, the film's instructions split 1,995G by hand (75%) and 651G compiled (25%), and
about 93% of the workers' time is the interpreter running them (2,677G at about 1.37 ns each under full load).

- **Loop-invariant code motion in rust-cardiac (rust-cardiac#127, behind CARDIAC_HOIST=1).** Every tape rebuilt with
  it: 3419/3419 identical, 2,647.3G to 2,643.9G (-0.13%). The film's hot loops are no longer compiled. On the way,
  Card Attack's live check caught a bug: the first version hoisted `999 - cl` above the `INP` that reads `cl` (a
  call is a block's terminator in the IR, after its statements), and 317 frames differed.
- **HEARTBURN.** Already tight (a decoded table, `wrap4` by compares): about 1.13 ns an instruction on one core with
  the host's overhead in. A compare before `wrap3`'s remainder was 2.5% slower than the division the compiler makes
  of `% 1000`; earlier, removing the bounds checks was slower too. Superinstructions (common pairs decoded as one
  step that counts two) are the one interpreter idea big enough to matter, and a project of their own.

The proportions at the end: the machines' Rust is 27,047 lines (991 machines, 422 of them in the film); the
hand-written CARDIAC is 1,400 lines (20 machines). The 1,400 lines run 75% of the film's instructions.

In all, 28% off the film's computing time.

## What's left

- umix, tr_runsum, sw_join, tr_tone1, tr_qp are the next candidates (about 8% of the film between them; each 2% or
  less).
- In the compiler: keeping the accumulator after a TAC (rust-cardiac#129); in the interpreter: superinstructions.
- Where a hand version wins by something the compiler could learn (hoisting a loop-invariant, unswitching a loop on
  a deck constant, keeping a sum in the accumulator), the compiler could do it for every machine.
