//! Live rendering: the cut's segments computed on CARDIAC while the film plays, one HEARTBURN a worker thread.
//!
//! Each video the cut takes frames from is made by an effect run as it always is (its `render`, in live mode, so its
//! frames go into memory: see cardiac_host::live), a lower third laid over a segment, or (a placeholder not yet built
//! on CARDIAC) a file read from disk; so is each layer an `over` line lays on (the UI kit's, made by its effect; any
//! other read from disk), due by the first frame it covers. An effect that reads another's video (the flash reads the flight, the vga the plasma, the usenet the flash
//! and the Mode X ray tracer, the bad dive the dive, the titles their segments, the swarm the videos it flies) waits for it.
//!
//! The heavy effects are split into blocks of their frames, each about BLOCK seconds of work, each a job of its own on
//! any worker; the effect's render makes only its block's frames (live::frames) and the parts are joined into its
//! video when every block is in. Each of these effects makes a frame from the frame's number and what it sets up
//! first (which every block sets up again), so a block makes exactly the frames the whole run would; the swarm's
//! blocks start on its groups of 8 frames (its scenes' sums go by groups), the drive's cover is in one block, made
//! from the drive's last frame. Costs are per frame (costs.txt, from the last dry run), so the blocks come out even.
//!
//! A video's deadline is the clock time of the first frame of the cut that needs it (or the latest start of a job
//! that needs it); workers take the ready job whose latest start (deadline less its cost) comes first. Composer
//! threads make each frame of the cut as soon as the videos it needs are in: the cut's own composer (cut.rs), on
//! CARDIAC, so the frames are the cut's exactly. A segment's slack is how long before its first frame is due its
//! video was in.
use super::{Film, Picture, HD};
use crate::fx::cut::{Composer, Cut, Step};
use crate::{fx, live, Host};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

/// Seconds between starting and the film's frame 0, by default: loading and the first segment.
pub const LEAD: f64 = 1.0;

/// Composer threads, beside the workers: the frames made at 1536x768 (un-pixelated, under a layer) are heavy.
pub const COMPOSERS: usize = 4;

/// Seconds of work a block aims at (setting up included).
pub const BLOCK: f64 = 0.5;

/// Ask macOS to keep this thread on the performance cores at full speed (QoS user-initiated), even when the window
/// isn't in front: without it a windowed run's workers were slowed down.
fn full_speed() {
    #[cfg(target_os = "macos")]
    {
        extern "C" { fn pthread_set_qos_class_self_np(qos: u32, relative: i32) -> i32; }
        const QOS_CLASS_USER_INITIATED: u32 = 0x19;
        unsafe { pthread_set_qos_class_self_np(QOS_CLASS_USER_INITIATED, 0); }
    }
}

/// What makes a video.
#[derive(Clone)]
enum Work { Effect(&'static str), Disk, Setup(&'static str) }

/// A maker: an effect (every video it files), a lower third or a file; what it reads; when its videos are due; its
/// jobs (one, or its blocks).
struct Group { name: String, work: Work, makes: Vec<String>, needs: Vec<usize>, deadline: f64, jobs: Vec<usize>, cuts: Option<Vec<usize>>,
    /// The longest a job of it and what waits for it take, one after the other (its chain, in seconds).
    chain: f64 }

/// A job: a maker's block of frames from..to (None: all of it), and its cost in seconds.
struct Job { group: usize, block: Option<(usize, usize)>, cost: f64 }

/// The frames an effect may be split at (its blocks' bounds), 0 to its frame count; None for an effect run whole.
/// The others are light, or carry state from frame to frame (the flight's camera, the zoom's terrain, the opening's
/// glide, the ending's birds), or are read whole by another.
fn cuts(effect: &str) -> Option<Vec<usize>> {
    let all = |n: usize| Some((0..=n).collect());
    match effect {
        "deep" => all(fx::deep::FRAMES), "polys" => all(fx::polys::END),
        "plasma" => all(fx::plasma::FRAMES), "modex" => all(fx::modex::FRAMES), "accel" => all(fx::accel::LOW.max(fx::accel::HD)),
        // the UI kit's layers
        "notify" | "miner" | "videoopts" | "accelbrag" => all(fx::ui::scenes::scene(effect).frames),
        // the party chat's layer
        "partychat" => all(fx::partychat::layout::FRAMES),
        // the die and the Rubik's cube (each block turns the cube's moves again); the BBS (each block draws the pages
        // its frames show)
        "dierubik" => all(fx::dierubik::FRAMES), "bbs" => all(fx::bbs::HD_FRAMES),
        // its blocks share what it sets up (the CGA tunnel's pixels, the last 3D frame's screen)
        "turn" => all(fx::turn::FRAMES),
        // the cold open: each block works out the game's frames, the GIF and the tutorial it shows, and the tunnel's;
        // the hall of mirrors goes on from each frame's picture to the next, so from its start to Resume in one block
        "mac" => Some((0..=fx::mac::boom::HAYWIRE).chain(fx::mac::boom::RESUME..=fx::mac::FRAMES).collect()),
        // the train: each frame from its number and the start (worked out once, see setup)
        "train" => all(fx::train::FRAMES),
        "night" => all(fx::night::FRAMES),
        // the stadium party: each frame from its number and the start (worked out once, see setup)
        "stadium" => all(fx::stadium::FRAMES),
        // the rooms: each frame from its number and the printed tables (read once, see setup)
        "rooms" => all(fx::rooms::FRAMES),
        // the ending's SQ: each frame from its number and the start (the picture's chrome, worked out once)
        "sqend" => all(fx::sqend::FRAMES),
        _ => None,
    }
}

/// What a split effect sets up once for all its blocks, in a job of its own before them (kept by the host, see
/// live::keep), rather than in every block: the deep's ray lengths, the turn's CGA pixels and last screen, the train's
/// tables.
fn setup(effect: &str) -> Option<&'static str> {
    match effect { "deep" => Some("deep"), "turn" => Some("turn"), "train" => Some("train"), "night" => Some("night"), "stadium" => Some("stadium"), "rooms" => Some("rooms"), "sqend" => Some("sqend"), _ => None }
}

/// The effect that makes a video, and the videos it reads; None for a video no effect makes (a placeholder).
fn maker(demo: &Path, video: &str) -> Option<(Work, Vec<String>)> {
    let effect = |e: &'static str| Some((Work::Effect(e), vec![]));
    match video {
        "openers/video.rgb" => effect("openers"),
        "mac/video.rgb" | "mac/video_hd.rgb" => effect("mac"),
        "tumble/video.rgb" => effect("tumble"),
        "roto/video4.rgb" => effect("roto4"),
        "plasma/video.rgb" => effect("plasma"),
        "vga/video.rgb" | "vga/video_hd.rgb" | "vga/brag.rgba" => Some((Work::Effect("vga"), vec!["plasma/video.rgb".into()])),
        "modex/video.rgb" | "modex/video_hd.rgb" => effect("modex"),
        "accel/video.rgb" | "accel/video_hd.rgb" => effect("accel"),
        "dierubik/video.rgb" => effect("dierubik"),
        "bbs/video_hd.rgb" => Some((Work::Effect("bbs"), vec!["tumble/video.rgb".into(), "dierubik/video.rgb".into(), "roto/video4.rgb".into()])),
        "bbs/video.rgb" => Some((Work::Effect("bbsroto"), vec!["dierubik/video.rgb".into(), "roto/video4.rgb".into()])),
        "turn/video_hd.rgb" | "turn/bezel.rgba" => effect("turn"),
        "polys/video.rgb" => effect("polys"),
        "deep/video.rgb" | "deep/video_hd.rgb" => effect("deep"),
        "train/video_hd.rgb" => effect("train"),
        "night/video_hd.rgb" => effect("night"),
        // the stadium party: its first frames dissolve from the night arrival's
        "stadium/video_hd.rgb" => Some((Work::Effect("stadium"), vec!["night/video_hd.rgb".into()])),
        // the rooms: their screens show Mode X, Dope, the drive (and its brag) and the dive
        "rooms/video_hd.rgb" => Some((Work::Effect("rooms"), fx::rooms::READS.iter().map(|r| r.to_string()).collect())),
        // the degauss at the rotozoom's end, into the plasma
        "qr/video_hd.rgb" => effect("qr"),
        "sqend/video_hd.rgb" => effect("sqend"),
        "rotodg/video_hd.rgb" => Some((Work::Effect("rotodg"), fx::rotodg::READS.iter().map(|r| r.to_string()).collect())),
        "ui/notify.rgba" => effect("notify"),
        "ui/miner.rgba" => effect("miner"),
        "ui/videoopts.rgba" => effect("videoopts"),
        "ui/accelbrag.rgba" => effect("accelbrag"),
        "partychat/chat.rgba" => effect("partychat"),
        _ => None,
    }
}

fn run(demo: &Path, work: &Work, makes: &[String]) {
    match work {
        Work::Effect(e) => {
            let render: fn(&Path, &str) = match *e {
                "openers" => fx::openers::render,
                "tumble" => fx::tumble::render, "roto4" => fx::roto4::render, "plasma" => fx::plasma::render,
                "polys" => fx::polys::render,
                "deep" => fx::deep::render,
                "notify" => fx::ui::notify, "miner" => fx::ui::miner, "videoopts" => fx::ui::videoopts, "accelbrag" => fx::ui::accelbrag, "vga" => fx::vga::render, "modex" => fx::modex::render, "accel" => fx::accel::render, "mac" => fx::mac::render,
                "partychat" => fx::partychat::render,
                "dierubik" => fx::dierubik::render, "bbs" => fx::bbs::render, "bbsroto" => fx::bbs::render_roto,
                "turn" => fx::turn::render, "train" => fx::train::render, "night" => fx::night::render, "stadium" => fx::stadium::render, "rooms" => fx::rooms::render, "rotodg" => fx::rotodg::render, "sqend" => fx::sqend::render, "qr" => fx::qr::render,
                e => panic!("no effect {e}"),
            };
            render(demo, "");
        }
        Work::Setup(e) => match *e { "deep" => fx::deep::setup(demo), "turn" => fx::turn::setup(demo), "train" => fx::train::setup(demo), "night" => fx::night::setup(demo), "stadium" => fx::stadium::setup(demo), "rooms" => fx::rooms::setup(demo), "sqend" => fx::sqend::setup(demo), e => panic!("no setup for {e}") },
        Work::Disk => for v in makes { let p = demo.join(v); live::put(&p, std::fs::read(&p).unwrap_or_else(|e| panic!("{v}: {e}"))) },
    }
}

/// A maker's measured cost (costs.txt beside the cut file, written by the dry run): seconds, or for a split effect
/// the seconds a block takes to set up and each frame's.
enum Cost { Whole(f64), Frames { setup: f64, per: Vec<f64> } }

fn costs(demo: &Path, dir: &str) -> HashMap<String, Cost> {
    std::fs::read_to_string(demo.join(dir).join("costs.txt")).unwrap_or_default().lines().filter(|l| !l.starts_with('#')).filter_map(|l| {
        let p: Vec<&str> = l.split_whitespace().collect();
        match p.as_slice() {
            [name, "setup", s, "frames", per @ ..] => Some((name.to_string(), Cost::Frames { setup: s.parse().ok()?, per: per.iter().map(|v| v.parse().unwrap()).collect() })),
            [name, s] => Some((name.to_string(), Cost::Whole(s.parse().ok()?))),
            _ => None,
        }
    }).collect()
}

/// A split effect's blocks (from, to, cost): as many as make each about BLOCK seconds, cut at the allowed frames
/// nearest even shares of the work.
fn blocks(cuts: &[usize], setup: f64, per: &[f64]) -> Vec<(usize, usize, f64)> {
    let n = *cuts.last().unwrap();
    let mut sum = vec![0.0];
    for f in 0..n { sum.push(sum[f] + per[f]); }
    let work = sum[n];
    let k = ((work / (BLOCK - setup).max(BLOCK / 4.0)).ceil() as usize).clamp(1, cuts.len() - 1);
    let mut at = vec![0];
    for i in 1..k {
        let share = work * i as f64 / k as f64;
        let c = *cuts.iter().min_by(|&&a, &&b| (sum[a] - share).abs().total_cmp(&(sum[b] - share).abs())).unwrap();
        if c > *at.last().unwrap() && c < n { at.push(c); }
    }
    at.push(n);
    at.windows(2).map(|w| (w[0], w[1], setup + sum[w[1]] - sum[w[0]])).collect()
}

/// The makers the cut needs and their jobs, with deadlines (seconds from the film's frame 0) and costs; and each
/// video's maker.
fn plan(demo: &Path, cut: &Cut, dir: &str) -> (Vec<Group>, Vec<Job>, HashMap<String, usize>) {
    let cost = costs(demo, dir);
    let (mut groups, mut jobs, mut by_video) = (vec![], vec![], HashMap::new());
    fn add(demo: &Path, video: &str, groups: &mut Vec<Group>, jobs: &mut Vec<Job>, by_video: &mut HashMap<String, usize>, cost: &HashMap<String, Cost>) -> usize {
        if let Some(&g) = by_video.get(video) { return g }
        let (work, reads) = maker(demo, video).unwrap_or((Work::Disk, vec![]));
        let name = match &work { Work::Effect(e) => e.to_string(), Work::Disk => format!("disk:{video}"), Work::Setup(_) => unreachable!() };
        // one maker per effect, whatever videos it files
        if let Some(g) = groups.iter().position(|g| g.name == name) { groups[g].makes.push(video.into()); by_video.insert(video.into(), g); return g }
        let mut needs: Vec<usize> = reads.iter().map(|r| add(demo, r, groups, jobs, by_video, cost)).collect();
        let cuts = match &work { Work::Effect(e) => cuts(e), _ => None };
        if let (Work::Effect(e), Some(_)) = (&work, &cuts) {
            if let Some(s) = setup(e) {
                // the setup: a maker of no video, a job of its own
                let name = format!("{s}:setup");
                let c = match cost.get(&name) { Some(Cost::Whole(c)) => *c, _ => 1.0 };
                jobs.push(Job { group: groups.len(), block: None, cost: c });
                groups.push(Group { name, work: Work::Setup(s), makes: vec![], needs: vec![], deadline: f64::INFINITY, jobs: vec![jobs.len() - 1], cuts: None, chain: 0.0 });
                needs.push(groups.len() - 1);
            }
        }
        let g = groups.len();
        let guess = if matches!(work, Work::Disk) { 0.05 } else { 10.0 };
        let first = jobs.len();
        match &cuts {
            Some(c) => {
                let n = *c.last().unwrap();
                let (setup, per) = match cost.get(&name) {
                    Some(Cost::Frames { setup, per }) if per.len() == n => (*setup, per.clone()),
                    Some(Cost::Frames { setup, per }) => (*setup, vec![per.iter().sum::<f64>() / n as f64; n]),
                    Some(Cost::Whole(s)) => (0.0, vec![s / n as f64; n]),
                    None => (0.0, vec![guess / n as f64; n]),
                };
                for (a, b, c) in blocks(c, setup, &per) { jobs.push(Job { group: g, block: Some((a, b)), cost: c }); }
            }
            None => {
                let c = match cost.get(&name) { Some(Cost::Whole(s)) => *s, Some(Cost::Frames { setup, per }) => setup + per.iter().sum::<f64>(), None => guess };
                jobs.push(Job { group: g, block: None, cost: c });
            }
        }
        groups.push(Group { name, work, makes: vec![video.into()], needs, deadline: f64::INFINITY, jobs: (first..jobs.len()).collect(), cuts, chain: 0.0 });
        by_video.insert(video.into(), g);
        g
    }
    for s in &cut.segs {
        add(demo, &s.video, &mut groups, &mut jobs, &mut by_video, &cost);
        for o in &s.overs { add(demo, &o.video, &mut groups, &mut jobs, &mut by_video, &cost); }
    }
    for n in 0..cut.frames.len() {
        for v in cut.uses(n) { let g = by_video[v]; groups[g].deadline = groups[g].deadline.min(n as f64 / super::present::FPS); }
    }
    // what a maker reads is due by the latest start of its jobs (makers were added after what they read)
    for g in (0..groups.len()).rev() {
        let by = groups[g].deadline - groups[g].jobs.iter().map(|&j| jobs[j].cost).fold(0.0, f64::max);
        for k in groups[g].needs.clone() { groups[k].deadline = groups[k].deadline.min(by); }
    }
    for g in (0..groups.len()).rev() {
        groups[g].chain += groups[g].jobs.iter().map(|&j| jobs[j].cost).fold(0.0, f64::max);
        for k in groups[g].needs.clone() { groups[k].chain = groups[k].chain.max(groups[g].chain); }
    }
    (groups, jobs, by_video)
}

/// Each frame of the cut's share of the jobs it was made from, (job, fraction), for the running count of instructions:
/// a job's instructions are spread over the frames of the cut that show its frames; a job none of whose frames is
/// shown (or a setup, or a maker read by another) over the frames of its maker, or of what reads it.
fn shares(cut: &Cut, groups: &[Group], jobs: &[Job], by_video: &HashMap<String, usize>) -> Vec<Vec<(usize, f64)>> {
    let mut users = vec![vec![]; jobs.len()];
    let mut of_group = vec![vec![]; groups.len()];
    for n in 0..cut.frames.len() {
        for (v, f) in cut.uses_at(n) {
            let g = by_video[v];
            let j = groups[g].jobs.iter().copied().find(|&j| jobs[j].block.is_none_or(|(a, b)| (a..b).contains(&f))).unwrap_or(groups[g].jobs[0]);
            users[j].push(n);
            of_group[g].push(n);
        }
    }
    // what a maker reads is read for the reader's frames (readers were added after what they read)
    for g in (0..groups.len()).rev() {
        for &k in &groups[g].needs { if of_group[k].is_empty() { let f = of_group[g].clone(); of_group[k].extend(f); } }
    }
    let mut out = vec![vec![]; cut.frames.len()];
    for (j, job) in jobs.iter().enumerate() {
        let to = if users[j].is_empty() { &of_group[job.group] } else { &users[j] };
        for &n in to { out[n].push((j, 1.0 / to.len() as f64)); }
    }
    out
}

/// What a job came to: when it ran (seconds from the start), on which worker, and its instructions.
#[derive(Clone, Copy, Default)]
pub struct Ran { pub from: f64, pub to: f64, pub worker: usize, pub steps: u64 }

/// The jobs done (and each one's frames' seconds) and running, the blocks each maker has still to finish, when its
/// videos were in, and the frames not yet taken by a composer.
struct State { done: Vec<Option<Ran>>, times: Vec<Vec<(usize, f64)>>, running: BTreeSet<usize>, togo: Vec<usize>, made: Vec<Option<f64>>, left: BTreeSet<usize>,
    /// For each maker, what still needs its videos: the frames still to compose that use them, and the makers still
    /// to run that read them. When both are none (and it has run), its videos are let go: `let_go`.
    frames_left: Vec<usize>, readers_left: Vec<usize>, gone: Vec<bool> }

impl State {
    /// Take maker g's videos out of the store if nothing will need them again. A frame still showing from one keeps
    /// it in memory until the frame is released.
    fn let_go(&mut self, g: usize, groups: &[Group], demo: &Path) {
        if self.made[g].is_some() && self.frames_left[g] == 0 && self.readers_left[g] == 0 && !self.gone[g] {
            self.gone[g] = true;
            for v in &groups[g].makes { live::remove(&demo.join(v)); }
        }
    }
}

/// A live run: workers computing the jobs, composers filing the film's frames.
pub struct Run { pub film: Arc<Film>, groups: Arc<Vec<Group>>, jobs: Arc<Vec<Job>>, state: Arc<(Mutex<State>, Condvar)>, threads: Vec<std::thread::JoinHandle<()>> }

impl Run {
    pub fn start(demo: &Path, cut: Arc<Cut>, dir: &str, workers: usize, started: Instant) -> Run {
        live::start();
        let (groups, jobs, by_video) = plan(demo, &cut, dir);
        // the horizon: how long the work takes on every worker; a job whose latest start falls within it goes by its
        // deadline, the others by their chains, so that the whole is done soonest
        let horizon = jobs.iter().map(|j| j.cost).sum::<f64>() / workers as f64;
        let (groups, jobs) = (Arc::new(groups), Arc::new(jobs));
        let film = Arc::new(Film::new(cut.frames.len()));
        // the makers whose videos each frame of the cut is made from
        let need: Arc<Vec<Vec<usize>>> = Arc::new((0..cut.frames.len()).map(|n| {
            let mut g: Vec<usize> = cut.uses(n).iter().map(|v| by_video[*v]).collect();
            g.sort(); g.dedup(); g
        }).collect());
        let mut frames_left = vec![0; groups.len()];
        for n in need.iter() { for &g in n { frames_left[g] += 1 } }
        let mut readers_left = vec![0; groups.len()];
        for gr in groups.iter() { let mut r = gr.needs.clone(); r.sort(); r.dedup(); for g in r { readers_left[g] += 1 } }
        let state = Arc::new((Mutex::new(State { done: vec![None; jobs.len()], times: vec![vec![]; jobs.len()], running: BTreeSet::new(),
            togo: groups.iter().map(|g| g.jobs.len()).collect(), made: vec![None; groups.len()], left: (0..cut.frames.len()).collect(),
            frames_left, readers_left, gone: vec![false; groups.len()] }), Condvar::new()));
        let mut threads = vec![];
        for w in 0..workers {
            let (groups, jobs, state, demo) = (groups.clone(), jobs.clone(), state.clone(), demo.to_path_buf());
            threads.push(std::thread::Builder::new().name(format!("heartburn {w}")).stack_size(64 << 20).spawn(move || { full_speed(); loop {
                // the ready job (all its maker reads in): a setup; else of those that must start within the horizon,
                // the earliest latest start; else the longest chain (then the earliest latest start)
                let j = {
                    let (m, cv) = &*state;
                    let mut st = m.lock().unwrap();
                    loop {
                        let soon = started.elapsed().as_secs_f64() + horizon;
                        let start = |j: usize| groups[jobs[j].group].deadline - jobs[j].cost;
                        // a setup first, whatever its own deadline: every block of its effect waits on it
                        let key = |j: usize| if matches!(groups[jobs[j].group].work, Work::Setup(_)) { (-1, start(j), 0.0) }
                            else if start(j) <= soon { (0, start(j), 0.0) } else { (1, -groups[jobs[j].group].chain, start(j)) };
                        let ready = (0..jobs.len()).filter(|&j| st.done[j].is_none() && !st.running.contains(&j) && groups[jobs[j].group].needs.iter().all(|&g| st.made[g].is_some()))
                            .min_by(|&a, &b| { let (x, y) = (key(a), key(b)); x.0.cmp(&y.0).then(x.1.total_cmp(&y.1)).then(x.2.total_cmp(&y.2)) });
                        if let Some(j) = ready { st.running.insert(j); break Some(j) }
                        if (0..jobs.len()).all(|j| st.done[j].is_some() || st.running.contains(&j)) { break None }
                        st = cv.wait(st).unwrap();
                    }
                };
                let Some(j) = j else { return };
                let (job, group) = (&jobs[j], &groups[jobs[j].group]);
                let from = started.elapsed().as_secs_f64();
                live::take_steps();
                live::take_times();
                live::set_block(job.block);
                run(&demo, &group.work, &group.makes);
                live::set_block(None);
                let ran = Ran { from, to: started.elapsed().as_secs_f64(), worker: w, steps: live::take_steps() };
                let times = live::take_times();
                if std::env::var_os("CARD_ATTACK_JOBS").is_some() { eprintln!("{:7.2} s  {}{} done ({:.2} s)", ran.to, group.name, job.block.map(|(a, b)| format!(" {a}..{b}")).unwrap_or_default(), ran.to - ran.from); }
                let (m, cv) = &*state;
                let last = {
                    let mut st = m.lock().unwrap();
                    st.running.remove(&j); st.done[j] = Some(ran); st.times[j] = times;
                    st.togo[job.group] -= 1;
                    st.togo[job.group] == 0
                };
                if last {
                    // the maker's last block: its parts joined into its videos, then they're in
                    if job.block.is_some() { for v in &group.makes { live::join(&demo.join(v)); } }
                    let mut st = m.lock().unwrap();
                    st.made[job.group] = Some(started.elapsed().as_secs_f64());
                    // it has read what it needed: those may be let go now, and so may its own, if nothing needs them
                    let mut read = group.needs.clone(); read.sort(); read.dedup();
                    for h in read { st.readers_left[h] -= 1; st.let_go(h, &groups, &demo); }
                    st.let_go(job.group, &groups, &demo);
                }
                cv.notify_all();
            } }).unwrap());
        }
        // the composers: each frame of the cut as soon as the videos it needs are in, earliest first, several at a time
        let shares = Arc::new(shares(&cut, &groups, &jobs, &by_video));
        for c in 0..COMPOSERS {
            let (film, state, cut, demo, need, shares, groups) = (film.clone(), state.clone(), cut.clone(), demo.to_path_buf(), need.clone(), shares.clone(), groups.clone());
            threads.push(std::thread::Builder::new().name(format!("composer {c}")).spawn(move || {
                full_speed();
                let composer = Composer::new(&demo);
                let mut host = Host::new();
                let video = |v: &str| live::get(&demo.join(v)).unwrap();
                loop {
                    let n = {
                        let (m, cv) = &*state;
                        let mut st = m.lock().unwrap();
                        loop {
                            if st.left.is_empty() { return }
                            let ready = st.left.iter().copied().find(|&n| need[n].iter().all(|&g| st.made[g].is_some()));
                            if let Some(n) = ready { st.left.remove(&n); break n }
                            st = cv.wait(st).unwrap();
                        }
                    };
                    let before = host.steps;
                    let pic = match cut.frames[n] {
                        _ if cut.composed_hd(n) => Picture::new(HD.0, HD.1, Arc::new(composer.sharp(&mut host, &cut, n, &video)), 0),
                        Step::Hd { s, f, .. } => { let vb = video(&cut.segs[s].video); if (f + 1) * HD.0 * HD.1 * 3 > vb.len() { panic!("{}: frame {f} of {} bytes", cut.segs[s].video, vb.len()); } Picture::new(HD.0, HD.1, vb, f * HD.0 * HD.1 * 3) }
                        _ => Picture::new(crate::W2, crate::H2, Arc::new(composer.frame(&mut host, &cut, n, &video)), 0),
                    };
                    // what the frame took: its composing, and its shares of the jobs it was made from (all done)
                    let made: f64 = { let st = state.0.lock().unwrap(); shares[n].iter().map(|&(j, w)| st.done[j].map_or(0.0, |r| r.steps as f64 * w)).sum() };
                    film.set_tally(n, host.steps - before + made.round() as u64);
                    film.set(n, pic);
                    // composed: the videos it was made from may be let go now
                    let mut st = state.0.lock().unwrap();
                    for &g in &need[n] { st.frames_left[g] -= 1; st.let_go(g, &groups, &demo); }
                }
            }).unwrap());
        }
        Run { film, groups, jobs, state, threads }
    }

    fn wait(self) -> (Vec<Ran>, Vec<Vec<(usize, f64)>>, Vec<f64>, Arc<Film>, Arc<Vec<Group>>, Arc<Vec<Job>>) {
        for t in self.threads { t.join().unwrap(); }
        let mut st = self.state.0.lock().unwrap();
        (st.done.iter().map(|d| d.unwrap()).collect(), std::mem::take(&mut st.times), st.made.iter().map(|m| m.unwrap()).collect(), self.film, self.groups, self.jobs)
    }
}

/// Render the film as the player would, with no window: each segment's slack before its first frame is due, the
/// workers' load over time; with `check`, every frame compared with the rendered cut.
pub fn dry_run(demo: &Path, cut: Arc<Cut>, dir: &str, workers: usize, check: bool, as_shown: bool) {
    let started = Instant::now();
    let run = Run::start(demo, cut.clone(), dir, workers, started);
    // as shown: each frame let go when the next would go up on the film's clock, as the presenter does, so that the
    // run's memory is the show's (it then lasts as long as the film)
    let releaser = as_shown.then(|| {
        let film = run.film.clone();
        std::thread::spawn(move || for n in 0..film.len() {
            let up = started + std::time::Duration::from_secs_f64(LEAD + (n + 1) as f64 / super::present::FPS);
            std::thread::sleep(up.saturating_duration_since(Instant::now()));
            film.release(n);
        })
    });
    let (ran, times, made, film, groups, jobs) = run.wait();
    let total = started.elapsed().as_secs_f64();
    let fps = super::present::FPS;
    let ready = |n: usize| (film.ready_at(n).unwrap() - started).as_secs_f64();
    let secs = |j: usize| ran[j].to - ran[j].from;
    let work = |g: usize| groups[g].jobs.iter().map(|&j| secs(j)).sum::<f64>();
    let busy: f64 = (0..jobs.len()).map(secs).sum();
    let longest = (0..jobs.len()).max_by(|&a, &b| secs(a).total_cmp(&secs(b))).unwrap();
    println!("dry run: {} frames ({:.1} s of film), {} jobs ({} makers, {} split into blocks) on {workers} workers, all computed in {total:.1} s, {:.1}G instructions",
        film.len(), film.len() as f64 / fps, jobs.len(), groups.len(), groups.iter().filter(|g| g.cuts.is_some()).count(), ran.iter().map(|r| r.steps).sum::<u64>() as f64 / 1e9);
    let counted: u64 = (0..film.len()).filter_map(|n| film.tally(n)).sum();
    // each frame's tally, for previews of the counter (not in git)
    std::fs::write(demo.join(dir).join("tally.txt"), (0..film.len()).map(|n| format!("{}\n", film.tally(n).unwrap_or(0))).collect::<String>()).unwrap();
    println!("the running count reaches {:.1}G instructions (the jobs' and the composing, over the film's frames)", counted as f64 / 1e9);
    println!("work {busy:.1} s on the workers ({:.1} s each if even); the longest job {} {:.2} s",
        busy / workers as f64, groups[jobs[longest].group].name, secs(longest));
    // per segment of the cut: its maker, when its first frame is due (with no lead), when that frame was filed
    println!("\n{:<4} {:<26} {:>6} {:>7}  {:<16} {:>6} {:>7} {:>7} {:>9}", "seg", "video", "frame", "due s", "maker", "jobs", "work s", "in s", "slack s");
    let mut worst = f64::INFINITY;
    let (_, _, by_video) = plan(demo, &cut, dir);
    for (i, s) in cut.segs.iter().enumerate() {
        let frames: Vec<usize> = (0..cut.frames.len()).filter(|&n| cut.shows(n).contains(&i)).collect();
        let slack = frames.iter().map(|&n| n as f64 / fps - ready(n)).fold(f64::INFINITY, f64::min);
        worst = worst.min(slack);
        let g = by_video[&s.video];
        println!("{i:<4} {:<26} {:>6} {:>7.1}  {:<16} {:>6} {:>7.2} {:>7.2} {:>9.1}{}", s.video, frames[0], frames[0] as f64 / fps, groups[g].name, groups[g].jobs.len(), work(g), made[g], slack,
            if matches!(groups[g].work, Work::Disk) { "  (placeholder, from disk)" } else { "" });
    }
    // the layers the over lines lay on: each one's maker, its instructions, and its slack before the first frame it
    // covers is due
    let layers: Vec<&str> = cut.segs.iter().flat_map(|s| s.overs.iter().map(|o| o.video.as_str())).fold(vec![], |mut v, o| { if !v.contains(&o) { v.push(o) } v });
    if !layers.is_empty() {
        println!("\n{:<26} {:>6} {:>7}  {:<16} {:>6} {:>7} {:>9} {:>9} {:>7} {:>9}", "layer", "frame", "due s", "maker", "jobs", "work s", "instr M", "M/frame", "in s", "slack s");
        for v in &layers {
            let g = by_video[*v];
            let frames: Vec<usize> = (0..cut.frames.len()).filter(|&n| cut.uses(n).contains(v)).collect();
            let slack = frames.iter().map(|&n| n as f64 / fps - ready(n)).fold(f64::INFINITY, f64::min);
            let steps: u64 = groups[g].jobs.iter().map(|&j| ran[j].steps).sum();
            println!("{:<26} {:>6} {:>7.1}  {:<16} {:>6} {:>7.2} {:>9} {:>9.1} {:>7.2} {:>9.1}{}", v, frames[0], frames[0] as f64 / fps, groups[g].name, groups[g].jobs.len(), work(g), steps / 1_000_000,
                steps as f64 / 1e6 / frames.len() as f64, made[g], slack, if matches!(groups[g].work, Work::Disk) { "  (from disk)" } else { "" });
        }
    }
    let extra: Vec<String> = (0..groups.len()).filter(|&g| !cut.segs.iter().any(|s| by_video[&s.video] == g) && !layers.iter().any(|v| by_video[*v] == g))
        .map(|g| format!("{} {:.2} s (in {:.2})", groups[g].name, work(g), made[g])).collect();
    if !extra.is_empty() { println!("\nonly read by other makers: {}", extra.join(", ")); }
    // the machines that took the most instructions, over every job and the composing
    let machines = live::machines();
    let all: u64 = machines.iter().map(|m| m.1).sum();
    println!("\n{:<20} {:>9} {:>6} {:>6} {:>11} {:>9}", "machine", "instr G", "share", "cum", "runs", "per run");
    let mut cum = 0;
    // the top 30 (all of them with CARD_ATTACK_MACHINES set)
    let shown = if std::env::var_os("CARD_ATTACK_MACHINES").is_some() { machines.len() } else { 30 };
    for (name, steps, runs) in machines.iter().take(shown) {
        cum += steps;
        println!("{name:<20} {:>9.1} {:>5.1}% {:>5.1}% {runs:>11} {:>9}", *steps as f64 / 1e9, 100.0 * *steps as f64 / all as f64, 100.0 * cum as f64 / all as f64, steps / runs.max(&1));
    }
    println!("{} machines ran, {:.1}G instructions", machines.len(), all as f64 / 1e9);
    println!("\nworst slack {worst:.2} s with no lead: {}", if worst >= 0.0 { "every frame ahead of the clock".to_string() } else { format!("the film needs {:.2} s of lead (play's default is {LEAD} s)", -worst) });
    // the load: busy workers per second
    let step = 1.0;
    let buckets = (total / step).ceil() as usize;
    print!("load (busy workers of {workers}, per {step} s):");
    for b in 0..buckets {
        let (a, z) = (b as f64 * step, (b + 1) as f64 * step);
        let busy: f64 = ran.iter().map(|r| (r.to.min(z) - r.from.max(a)).max(0.0)).sum::<f64>() / step;
        print!(" {busy:.1}");
    }
    println!();
    let mut per_worker = vec![0.0; workers];
    for r in &ran { per_worker[r.worker] += r.to - r.from; }
    let (lo, hi) = (per_worker.iter().copied().fold(f64::INFINITY, f64::min), per_worker.iter().copied().fold(0.0, f64::max));
    println!("each worker busy {lo:.1} to {hi:.1} s");
    // the measured costs, for the next run's schedule: a split effect's setup (each block's time less its frames',
    // on average) and each frame's seconds
    let mut text = String::from("# seconds per maker, measured by `card-attack play --dry-run` (the player schedules by them); a split effect's\n# seconds to set up a block, then each frame's\n");
    for (g, group) in groups.iter().enumerate() {
        match &group.cuts {
            Some(c) => {
                let mut per = vec![0.0; *c.last().unwrap()];
                let mut setup = 0.0;
                for &j in &group.jobs {
                    for &(f, s) in &times[j] { per[f] = s; }
                    setup += (secs(j) - times[j].iter().map(|t| t.1).sum::<f64>()).max(0.0) / group.jobs.len() as f64;
                }
                text += &format!("{} setup {setup:.4} frames {}\n", group.name, per.iter().map(|s| format!("{s:.4}")).collect::<Vec<_>>().join(" "));
            }
            None => text += &format!("{} {:.2}\n", group.name, work(g)),
        }
    }
    std::fs::write(demo.join(dir).join("costs.txt"), text).unwrap();
    if check { check_frames(demo, dir, &film); }
    if let Some(r) = releaser { r.join().unwrap(); }
}

/// Every frame the player filed against the rendered cut: video.rgb beside the cut file, and for the frames hd.txt
/// lists, the 1536x768 frames it names.
fn check_frames(demo: &Path, dir: &str, film: &Film) {
    let lo = std::fs::read(demo.join(dir).join("video.rgb")).expect("video.rgb: render the cut first");
    let frame = crate::FRAME2;
    let hd_frame = HD.0 * HD.1 * 3;
    let mut hd: HashMap<usize, (String, usize)> = HashMap::new();
    for l in std::fs::read_to_string(demo.join(dir).join("hd.txt")).unwrap().lines() {
        let p: Vec<&str> = l.split(' ').collect();
        hd.insert(p[0].parse().unwrap(), (p[1].to_string(), p[2].parse().unwrap()));
    }
    let mut files: HashMap<String, std::fs::File> = HashMap::new();
    let (mut good, mut bad) = (0, vec![]);
    assert_eq!(lo.len(), film.len() * frame, "video.rgb is {} frames, the cut {}", lo.len() / frame, film.len());
    for n in 0..film.len() {
        let p = film.get(n).unwrap();
        let same = match hd.get(&n) {
            Some((v, f)) => {
                use std::io::{Read, Seek, SeekFrom};
                let fh = files.entry(v.clone()).or_insert_with(|| std::fs::File::open(demo.join(v)).unwrap());
                let mut want = vec![0u8; hd_frame];
                fh.seek(SeekFrom::Start((f * hd_frame) as u64)).unwrap();
                fh.read_exact(&mut want).unwrap();
                p.w == HD.0 && p.rgb() == &want[..]
            }
            None => p.w == crate::W2 && p.rgb() == &lo[n * frame..(n + 1) * frame],
        };
        if same { good += 1 } else { bad.push(n) }
    }
    println!("check: {good}/{} frames identical to {dir}/video.rgb and the 1536x768 frames hd.txt lists{}", film.len(),
        if bad.is_empty() { String::new() } else { format!("; differ: {:?}", &bad[..bad.len().min(20)]) });
    if !bad.is_empty() { std::process::exit(1) }
}
