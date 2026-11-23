# Fitting the night and the party into five minutes

> A record of work as it happened, in October and November 2026. The numbers, machines and segments each entry names are as they were that day; many segments and
> machines have since been removed or changed. For the repository as it is now, see the [README](../README.md) and
> [RUST.md](RUST.md).

Card Attack computes every pixel live, on CARDIAC machines emulated by HEARTBURN, on one Mac, while the film plays.
In October 2026 the film's last two segments, the night arrival and the stadium party, were built on CARDIAC, and
the film stopped fitting. This is what we saw, how we found out why, what we ruled out, and what we changed.

Apple M5 Ultra (10 of its fastest cores, 20 of the next), rustc 1.97.1, the live player's 28 workers.

## What we saw

`card-attack play --dry-run` computes the film as the live player would, against the clock, and reports each
segment's slack: how long before its first frame is due its last frame was ready (negative: late). With the night
and the stadium in, the first measured run:

| Run | What changed | Work, s per worker | Film, s | Longest job, s | Worst slack, s |
|---|---|---|---|---|---|
| 1 | night and stadium as first built | 396.6 | 297 | 470.4 | **-483.4** |

The stadium's frames were up to eight minutes late. The film can't wait: there is no video to fall back on, and no
precalc (the film computes live, not after a loading pause).

## Why it was a problem

The live player has one pool of compute: 28 workers for as long as the film runs, about 297 s, so about 297
seconds of the whole Mac. Every frame must be ready by the time it is due. The cheap early segments finish long
before their frames are due, and the player hands their spare cores to later segments' jobs; so the question is
not how fast each segment renders, but whether the whole film's work fits under that line, with each frame's share
done in time.

The night and the stadium, as first built, were most of the film's work. At this Mac's rate they were about 330 s
of the whole Mac between them, against a film of 297 s.

## How we attacked it

### 1. Is it the work, or the scheduling?

Run 1's longest single job was 470 s: one block of the night's frames on one worker. The player splits a heavy
effect into blocks sized by each frame's measured cost (`segments/cut/costs.txt`, written by the dry run); the
night and the stadium had no measured costs yet, so they were cut into a few huge blocks, and most workers sat idle
while a few ground through them. A second run, with the costs the first had measured:

| Run | What changed | Work, s per worker | Longest job, s | Worst slack, s |
|---|---|---|---|---|
| 2 | measured costs, so fine blocks | 432.7 | 40.1 | **-179.7** |

Every worker was now busy to within 1% of the others (429 to 439 s). The scheduling was fixed; what remained was
the work: about 433 s per worker for a 297 s film. (The extra 36 s over run 1 is each small block setting up
again.)

### 2. Less work: party mode

The night and the stadium were given a party version: the world drawn at half resolution (768x384) and made up to full size on CARDIAC (`tr_up4`),
the things the eye locks on (the train, the screens and their text, the near dancers, the lasers, the stars) at
full resolution, joined by a depth test so there are no halos.
The blimp's orbit (7 bars, 147 frames) was cut from the night and given to the stadium as fly-throughs of the
crowd, which cost less a frame. The crowd was halved for the party version (32,768), and 40 near dancers drawn as
meshes, not 64.

| | Party version |
|---|---|
| Night | 1,466G (373 frames) |
| Stadium | 1,480G (575 frames) |

| Run | What changed | Work, s per worker | Longest job, s | Worst slack, s |
|---|---|---|---|---|
| 3 | party versions | 260.8 | 18.9 | **-18.9** |

The total now fits, about 36 s under the film. The stadium's weave through the crowd (up to 5.1G a frame) still
comes in late: the stadium uses compute two to four times faster than real time, and leans on being started early.
All of this is during the film: the player starts at most one second before frame 0, and a segment's frames may be
computed while earlier segments are on screen and held in memory until they are due.
The red pill scene, still to come, makes it about 30 s to find.

(Run 3 first crashed: the night's party version set the shared train renderer's resolution through one global, and
in the live player a train job on another worker read it. The resolution is now kept per thread.)

### 3. Where does the time go?

We sampled the live player itself (`sample`, every millisecond, 25 s, all 28 workers busy on the film's work):

| Where | Samples | Share of busy time |
|---|---|---|
| `Host::go`: HEARTBURN's interpreter, inlined into the host's machine runner | 171,532 | 73.2% |
| Allocating, freeing and copying memory (`malloc`, `free`, `realloc`, vector growth, `memmove`) | 33,729 | 14.4% |
| The effects' own host code (dealing decks, filing pixels) | 29,063 | 12.4% |
| (workers waiting for a job, not counted as busy) | 60,590 | |

HEARTBURN is not a separate process: it is a Rust library compiled into the player, a machine run is a function
call (`cardiac::run`) on the deck's slices, and with link-time optimisation its loop is inlined into the host's.

### 4. Hypotheses ruled out

- **IPC or process overhead.** None: one process, the interpreter a function call, inlined.
- **A slow interpreter.** HEARTBURN keeps every cell's instruction decoded beside it (re-decoded only when a store
  changes it), looks a stored word's decoding up in a table, and wraps sums by comparing, not dividing. On one of
  the fastest cores it runs about 0.72G CARDIAC instructions a second: four or five Mac cycles an instruction, close
  to the floor for an interpreter of this kind. (The one division left in the loop is SFT's: a shift by a power of
  ten that the instruction chooses, so the Mac divides at run time. It is a few cycles, not dozens.)
- **The compiler.** The host builds with LTO and one codegen unit. On a fixed benchmark (the stadium's party
  frames 250 to 255, one core: 22,933M instructions, 1,010,420 machine runs):

  | Build | Seconds |
  |---|---|
  | as built | 31.8 |
  | `-C target-cpu=native` | 31.8 |
  | profile-guided (trained on stadium, night and train frames) | 32.3 |

  No gain from either.
- **The cores.** One worker on a fast core runs 0.72G a second; across all 28 the average is 0.48G, because two
  thirds of the cores are the slower kind. That is the machine, not something to fix.

### 5. What's left: memory allocation pressure

A seventh of the busy time goes on making and throwing away small vectors around each machine run. The stadium alone
makes about 63 million machine runs, most of them short, so each run's setup is paid tens of millions of times:

- `Host::go` (card-attack's runtime) builds a new list of deck parts, a flattened copy of the deck, and a fresh
  vector of punched cards for every run, and returns the cards as a new vector.
- The effects build each run's deck of cards from scratch: several small vectors per run, dropped straight after.

Attributing each allocation sample to the host code that called it (the sample's call stacks) put almost two thirds
of it in one module:

| Caller | Share of the allocation |
|---|---|
| `swarm_calc::Calc` (`deal`, `eval`, `push`, its drops, `lin_within`, `divrem`) | 64% |
| `Host::go` (the runtime's machine runner) | 5% |
| the dive's ray code, the mesh rasteriser, the train's helpers, and the rest | 1 to 3% each |

`Calc` is not the swarm's alone: it is the shared calculator the mesh rasteriser (the stadium's near dancers), the
dive and others use to batch their machine runs. Every evaluation builds maps keyed by level and by machine name, a
fresh vector per node, and fresh decks per run, and drops them all.

None of this is CARDIAC's work: it doesn't change the instruction count, only how long the Mac takes to set the
machines up.

## How we're improving it

1. **Pool the allocations** (done: run 5), chiefly `Calc`'s: flat arrays in place of its per-evaluation maps, its node and deck
   buffers reused between evaluations; and in the runtime, decks passed through without copying. HEARTBURN is not
   touched. Expected: most of the 14%, so roughly 5 to 8% of the film's time, 13 to 20 s. Got: the allocation
   share from 14% to about 4%, and 20% off the film's time (46 s a worker), because the train's operations, which
   the night and the stadium share, were allocating too.
2. **The train's party version** (done: run 4), the same treatment as the night's: 398G, about 25 s saved.
3. **SFT by constant divisions** (a branch per shift, each dividing by a constant the compiler turns into a
   multiply): a few percent at most; only if needed. Not needed.

Every change must leave the pictures byte-identical to the native Rust references and the CARDIAC instruction counts
unchanged.

## Results

The film fits, with room to spare. From the first measured run to the last:

| Run | What changed | Work, s per worker | Worst slack, s (the stadium's) |
|---|---|---|---|
| 1 | night and stadium as first built | 396.6 | -483.4 |
| 2 | fine blocks (measured costs) | 432.7 | -179.7 |
| 3 | night and stadium party versions, the orbit to the crowd | 260.8 | -18.9 |
| 4 | the train's party version | 235.9 | +7.5 |
| 5 | allocation pooling | 189.7 | +53.7 |

(From run 4 on, the film's worst slack is the openers' first frame, -0.1 s, inside the player's 1 s lead.) Of the
237 s per worker saved since run 2, party versions bought about 197 s and changed the pictures (softer worlds,
sharp subjects); the host's allocation bought 46 s and changed nothing.

The red pill scene, still to come, pulls the later deadlines 4.4 s earlier and adds its own work: well inside the
stadium's 53.7 s.

Considered and left alone:

- **Snapshots of loaded machines.** Every run starts a cleared CARDIAC and runs the bootstrap loader over its tape;
  a machine loaded once could be saved and cloned for each later run (the loader never reads the data cards, so the
  state at its hand-off is always the same). It would lower the instruction counts by the loader's share and needs a
  small entry point in HEARTBURN. Not needed for the budget; worth measuring per machine (tape length times runs,
  against total instructions) if a table machine, a sine lookup say, turns out to be mostly loading.
- **A JIT.** Translating each CARDIAC program to native code once could beat the interpreter's four or five cycles an
  instruction several times over, but self-modifying programs make it a project of its own, and it would change what
  HEARTBURN is. HEARTBURN stays an interpreter.

| Run | What changed | Work, s per worker | Longest job, s | Worst slack, s |
|---|---|---|---|---|
| 4 | train party version (398G) | 235.9 | 18.6 | **-0.10** (the openers' frame 0, inside the player's 1 s lead; the stadium 7.5 s early) |
| 5 | allocation pooling | 189.7 | 16.9 | **-0.09** (the openers' frame 0 again; the stadium 53.7 s early) |

### Run 5: allocation pooling

The host now keeps its bookkeeping in flat arrays and reuses them:

- `Calc` holds every node's arguments in one array and every node's cards in another (a node keeps where its own
  start), names its machines by number, and works out a level in one sorted order (level, machine name, node), the
  same order its maps gave. Its buffers are reused between evaluations, and a calculator dropped hands them, cleared,
  to the next one made on the same thread. The machine runner is given the items' cards and where each ends, and
  punches straight into `Calc`'s array.
- The train's operations (`train::Cardiac`, which the train, the night and the stadium all use) were the other
  half. Profiling the stadium benchmark showed it there, not in `Calc`: `sum` built a vector for every item, and
  every run made a new `String` for the per-machine tally. Items are now built flat, decks go to the machine as
  pieces (count, header, items) without being copied, and the tally is keyed by the machine's static name.
- `Host::go` keeps the reader's parts on the stack, the dealt-out deck of a machine in phases in a buffer it keeps,
  and counts each run without cloning the machine's name; `go_into`, `go1_into` and `gon_into` add the punched
  cards to the caller's vector, and `Calc`'s users call them.

Every machine is dealt the same cards in the same order. Each effect touched reports the same frames, instruction
counts and machine runs as before:

| Check | Before | After | |
|---|---|---|---|
| stadium, party frames 250 to 255, one core | 31.6 s | 28.2 s | 6/6, 22,933M, 1,010,420 runs |
| mesh | 10.5 s | 8.9 s | 48/48, 6,147M, 64,178 runs |
| mac | 1.9 s | 1.5 s | 319/319, 995M, 32,014 runs |
| turn | 8.1 s | 6.2 s | 143/143, 3,507M, 56,593 runs |
| accel | 32.0 s | 31.8 s | 118/118, 18,075M, 1,671,758 runs |
| deep, whole | 225.4 s | 208.1 s | 353/353 and 353/353, 74,815M + 95,519M |
| night, frames 50, 200, 300 | 27.9 s | 26.1 s | 3/3, 23,225M, 459,437 runs |
| train, all 256 party frames | | 418 s | 256/256, 398,479M, 9,661,420 runs |

`card-attack cut refs/cut_ref` gives 3135/3135, and `play --check` 3135/3135. (The train's check needed its
reference rebuilt: `refs/train_ref` was older than the party version.)

Busy time in allocation, sampled the same way:

| | Before | After |
|---|---|---|
| the stadium benchmark | 14.8% | 3.0% |
| the dry run (before: 25 s; after: three 20 s windows, at 30, 110 and 190 s) | 14.4% | 4.3%, 3.7%, 3.7% |

The interpreter's share rose to match (73% to 75 to 79% in the dry run, 71% to 82% on the benchmark). `Calc` is no
longer among the top callers. What is left is mostly the train's operations' result vectors and the stadium
renderer's own maps.

The dry run's instruction total moves between runs (3,653G in run 4, 3,660G in run 5) because each run re-measures
`costs.txt`, and so splits the heavy segments into slightly different blocks, each setting up again. On the same
`costs.txt` the two builds make the same 1,633 jobs, run the same 3,663.2G instructions and give the same count for
every frame (`tally.txt` identical). They take 237.9 and 189.3 s per worker, with the stadium 4.8 and 54.6 s early.
