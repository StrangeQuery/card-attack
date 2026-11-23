//! The soundtrack, decoded live by two CARDIACs and played on the film's clock.
//!
//! The soundtrack's mix (made outside this repository) writes its two tracks, segments/music/left.track and
//! right.track, with lubdub's stage. Here a decoder thread runs lubdub's `Stereo`: two CARDIACs running
//! the decoder, one per channel, taking turns a block of 64 samples at a time. Their cards are read from the track
//! files as the CARDIACs ask for them, and the samples they decode go into a buffer about two seconds long, which
//! the sound card drains. A sample is dropped as soon as it has been played, so the soundtrack is never all in
//! memory, either as cards or as samples.
//!
//! The sound card's first callback works out where the film's clock will be when its first sample is heard (the
//! callback's playback time, so the output's latency is counted). From there it counts samples: the sound card's
//! clock, which drifts from the film's by some tens of parts per million (about 10 ms over the film), too little to
//! hear. Before the clock starts (or before `--from`), and past the soundtrack's end, it plays silence. If the decoder ever falls behind,
//! the missing samples are played as silence and counted.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use heartburn::tape;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const RATE: u32 = 48000;

/// How far ahead of the sound card the CARDIACs decode, in samples: two seconds.
const AHEAD: usize = 2 * RATE as usize;

/// The decoded samples waiting to be played, and the number of the first of them.
struct Buffer {
    first: i64,
    samples: VecDeque<[i16; 2]>,
}

/// What the decoder thread and the sound card share.
struct Shared {
    buffer: Mutex<Buffer>,
    /// the sample the sound card is playing now
    playing: AtomicI64,
    /// the tracks have ended (or failed): nothing more is coming
    ended: AtomicBool,
    /// samples played as silence because they weren't decoded in time
    missed: AtomicU64,
    /// the CARDIACs' instructions so far, both channels
    steps: AtomicU64,
    /// how many samples the CARDIACs have decoded
    decoded: AtomicI64,
    /// the sample playing starts at (after `--from`); the ones before it are silence
    from: i64,
    stop: AtomicBool,
}

/// The soundtrack playing: it stops when this is dropped, and says if any samples were missed.
pub struct Sound {
    _stream: cpal::Stream,
    shared: Arc<Shared>,
    decoder: Option<std::thread::JoinHandle<()>>,
}

impl Sound {
    /// The CARDIAC instructions the soundtrack has taken so far.
    pub fn steps(&self) -> u64 {
        self.shared.steps.load(Ordering::Relaxed)
    }
}


impl Drop for Sound {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(d) = self.decoder.take() {
            let _ = d.join();
        }
        let missed = self.shared.missed.load(Ordering::Relaxed);
        if missed > 0 {
            eprintln!("sound: {missed} samples weren't decoded in time and played as silence");
        }
    }
}

/// A track's cards, a line at a time, as the CARDIAC asks for them. A line that isn't cards ends the track (the
/// decoder then says it had no end card).
pub(crate) fn cards(path: &std::path::Path) -> std::io::Result<impl Iterator<Item = i32>> {
    let name = path.display().to_string();
    let lines = BufReader::new(File::open(path)?).lines();
    Ok(lines.map_while(Result::ok).enumerate().map_while(move |(n, line)| match tape::values(&line) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("sound: {name}: line {}: {e}", n + 1);
            None
        }
    }).flatten())
}

/// The decoder thread: the two CARDIACs keep the buffer about two seconds ahead of the sound card. Samples the sound
/// card has already passed (after `--from`, say) are decoded and dropped at once, since each sample depends on the
/// ones before it.
fn decode<L: Iterator<Item = i32>, R: Iterator<Item = i32>>(mut stereo: lubdub::Stereo<L, R>, shared: &Shared) {
    let mut next = 0i64;
    while !shared.stop.load(Ordering::Relaxed) {
        let playing = shared.playing.load(Ordering::Relaxed);
        let buffered = shared.buffer.lock().unwrap().samples.len();
        if next >= playing && buffered >= AHEAD {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        let block = match stereo.next_block() {
            Ok(Some(block)) => block,
            Ok(None) => break,
            Err((side, e)) => {
                eprintln!("sound: the {side:?} track {e}");
                break;
            }
        };
        shared.steps.store(stereo.left.steps() + stereo.right.steps(), Ordering::Relaxed);
        shared.decoded.store(next + block.len() as i64, Ordering::Relaxed);
        let n = block.len() as i64;
        let mut b = shared.buffer.lock().unwrap();
        if next + n <= playing {
            // the sound card is already past this block
            b.first = next + n;
            b.samples.clear();
        } else {
            b.samples.extend(block);
        }
        next += n;
    }
    shared.ended.store(true, Ordering::Relaxed);
}

/// The soundtrack ready to play: its decoder thread started, and already decoded up to where playing starts.
pub struct Prepared {
    shared: Arc<Shared>,
    decoder: std::thread::JoinHandle<()>,
}

/// Start decoding the soundtrack, and wait until the CARDIACs have decoded up to `from` seconds into it (for
/// `--from`), so the film's clock can start with the sound ready. None, said on standard error, when there is no
/// soundtrack.
pub fn prepare(from: f64) -> Option<Prepared> {
    let dir = crate::demo_dir().join("music");
    let open = |side: &str| {
        let p = dir.join(format!("{side}.track"));
        cards(&p).map_err(|e| eprintln!("no sound: {}: {e} (unpack segments/music/generated.tar.xz: ./build.sh does)", p.display())).ok()
    };
    let (left, right) = (open("left")?, open("right")?);
    let from = (from * RATE as f64) as i64;
    let shared = Arc::new(Shared {
        buffer: Mutex::new(Buffer { first: 0, samples: VecDeque::with_capacity(AHEAD + lubdub::BLOCK) }),
        playing: AtomicI64::new(from),
        ended: AtomicBool::new(false),
        missed: AtomicU64::new(0),
        steps: AtomicU64::new(0),
        decoded: AtomicI64::new(0),
        from,
        stop: AtomicBool::new(false),
    });
    let decoder = {
        let shared = shared.clone();
        std::thread::Builder::new()
            .name("soundtrack".into())
            .spawn(move || decode(lubdub::Stereo::new(left, right), &shared))
            .ok()?
    };
    while shared.decoded.load(Ordering::Relaxed) < from && !shared.ended.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(5));
    }
    Some(Prepared { shared, decoder })
}

impl Prepared {
    /// Play the soundtrack with the film's frame 0 at `start`, `offset_ms` later still (for a display that shows its
    /// pictures late). None, said on standard error, when there is no output to play it on.
    pub fn play(self, start: Instant, offset_ms: f64) -> Option<Sound> {
        let Prepared { shared, decoder } = self;
        let Some(device) = cpal::default_host().default_output_device() else {
            eprintln!("no sound: no output device");
            shared.stop.store(true, Ordering::Relaxed);
            return None;
        };
    let config = cpal::StreamConfig { channels: 2, sample_rate: cpal::SampleRate(RATE), buffer_size: cpal::BufferSize::Default };
    // the next sample (a stereo pair) to play, counted from frame 0's; before it, negative
    let mut next: Option<i64> = None;
    let card = shared.clone();
    let fill = move |out: &mut [f32], info: &cpal::OutputCallbackInfo| {
        let at = *next.get_or_insert_with(|| {
            let ts = info.timestamp();
            let heard = Instant::now() + ts.playback.duration_since(&ts.callback).unwrap_or_default();
            let t = if heard >= start { (heard - start).as_secs_f64() } else { -(start - heard).as_secs_f64() };
            ((t - offset_ms / 1e3) * RATE as f64).round() as i64
        });
        card.playing.store(at, Ordering::Relaxed);
        let mut b = card.buffer.lock().unwrap();
        for (k, pair) in out.chunks_exact_mut(2).enumerate() {
            let s = at + k as i64;
            // drop what has been passed: played, or skipped
            while b.first < s && !b.samples.is_empty() {
                b.samples.pop_front();
                b.first += 1;
            }
            let lr = if s >= card.from && b.first == s { b.samples.pop_front() } else { None };
            match lr {
                Some(lr) => {
                    b.first += 1;
                    pair[0] = lr[0] as f32 / 32768.0;
                    pair[1] = lr[1] as f32 / 32768.0;
                }
                None => {
                    if s >= card.from && !card.ended.load(Ordering::Relaxed) {
                        card.missed.fetch_add(1, Ordering::Relaxed);
                    }
                    pair[0] = 0.0;
                    pair[1] = 0.0;
                }
            }
        }
        next = Some(at + (out.len() / 2) as i64);
    };
    let stream = device.build_output_stream(&config, fill, |e| eprintln!("sound: {e}"), None).map_err(|e| eprintln!("no sound: {e}")).ok()?;
    stream.play().map_err(|e| eprintln!("no sound: {e}")).ok()?;
    Some(Sound { _stream: stream, shared, decoder: Some(decoder) })
    }
}
