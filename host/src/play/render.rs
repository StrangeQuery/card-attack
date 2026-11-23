//! The film as a video file: `card-attack render OUT.mp4 [--workers N]`.
//!
//! The pictures are the live player's: the same run (live.rs) computes every frame on CARDIAC, and each is written
//! out in order as soon as it is filed, at 1536x768, a 192x96 frame with each pixel filed 8 x 8 times, so nothing is
//! blurred. The sound is the live player's too: lubdub's `Stereo`, the two CARDIACs that decode the soundtrack's
//! tracks, decoded here as fast as they go into a file in the temporary directory, padded with silence (or cut) to
//! the film's length exactly, and deleted afterwards. ffmpeg (on the PATH) encodes both: H.264 (yuv420p, CRF 17)
//! and AAC (320 kb/s, 48 kHz stereo), with the index at the front for streaming.
//!
//! It needs no rendered files on disk, takes longer than the film (no frame is dropped to keep up), and reads the
//! measured costs (costs.txt) without writing them.
use super::{live, sound, Picture, HD};
use crate::fx::cut::Cut;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Samples a frame lasts: 48,000 a second at 12 frames a second.
const PER_FRAME: usize = sound::RATE as usize / 12;

/// Decode the soundtrack on its two CARDIACs into `path` as 16-bit little-endian stereo, `frames` frames long.
fn decode_sound(path: &std::path::Path, frames: usize) -> Result<usize, String> {
    let dir = crate::demo_dir().join("music");
    let open = |side: &str| {
        let p = dir.join(format!("{side}.track"));
        sound::cards(&p).map_err(|e| format!("{}: {e} (unpack segments/music/generated.tar.xz: ./build.sh does)", p.display()))
    };
    let mut stereo = lubdub::Stereo::new(open("left")?, open("right")?);
    let want = frames * PER_FRAME;
    let mut out = BufWriter::new(std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?);
    let mut decoded = 0;
    let mut written = 0;
    loop {
        let block = match stereo.next_block() {
            Ok(Some(block)) => block,
            Ok(None) => break,
            Err((side, e)) => return Err(format!("the {side:?} track {e}")),
        };
        decoded += block.len();
        for [l, r] in block {
            if written < want {
                out.write_all(&l.to_le_bytes()).and_then(|_| out.write_all(&r.to_le_bytes())).map_err(|e| e.to_string())?;
                written += 1;
            }
        }
    }
    // silence to the end of the film's last frame
    for _ in written..want { out.write_all(&[0; 4]).map_err(|e| e.to_string())?; }
    out.flush().map_err(|e| e.to_string())?;
    Ok(decoded)
}

/// A frame at 1536x768: an HD frame as it is, a 192x96 one with each pixel filed 8 x 8 times.
fn full_size(p: &Picture, out: &mut Vec<u8>) {
    out.clear();
    if (p.w, p.h) == HD { out.extend_from_slice(p.rgb()); return }
    let k = HD.0 / p.w;
    assert_eq!((p.w * k, p.h * k), HD);
    let rgb = p.rgb();
    let mut row = Vec::with_capacity(HD.0 * 3);
    for y in 0..p.h {
        row.clear();
        for x in 0..p.w { let px = &rgb[(y * p.w + x) * 3..][..3]; for _ in 0..k { row.extend_from_slice(px) } }
        for _ in 0..k { out.extend_from_slice(&row) }
    }
}

pub fn main(a: &[String]) {
    let Some(out) = a.first().filter(|s| !s.starts_with("--")) else {
        eprintln!("usage: card-attack render OUT.mp4 [--workers N]");
        std::process::exit(2)
    };
    let workers: usize = super::arg(a, "--workers").unwrap_or_else(|| std::thread::available_parallelism().map_or(8, |n| n.get()).saturating_sub(2).max(1));
    if Command::new("ffmpeg").arg("-version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_err() {
        eprintln!("render: ffmpeg isn't on the PATH (brew install ffmpeg)");
        std::process::exit(1)
    }
    let demo = crate::demo_dir();
    let started = Instant::now();
    let cut = Arc::new(Cut::parse(&crate::read(&demo, "cut/cut.txt")));
    let frames = cut.frames.len();
    // the soundtrack first, on its own thread, while the workers start on the pictures
    let wav = std::env::temp_dir().join(format!("card-attack-{}.s16", std::process::id()));
    let sound = { let wav = wav.clone(); std::thread::spawn(move || decode_sound(&wav, frames)) };
    let run = live::Run::start(&demo, cut, "cut", workers, started);
    let decoded = match sound.join().unwrap() {
        Ok(n) => n,
        Err(e) => { let _ = std::fs::remove_file(&wav); eprintln!("render: the soundtrack: {e}"); std::process::exit(1) }
    };
    eprintln!("render: the soundtrack decoded by its CARDIACs in {:.1} s: {decoded} samples, {} more of silence to the film's end",
        started.elapsed().as_secs_f64(), (frames * PER_FRAME).saturating_sub(decoded));
    let size = format!("{}x{}", HD.0, HD.1);
    let mut ff = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", &size, "-r", "12", "-i", "-"])
        .args(["-f", "s16le", "-ar", "48000", "-ch_layout", "stereo", "-i"]).arg(&wav)
        .args(["-vf", &format!("scale={}:{}:flags=area", HD.0, HD.1), "-c:v", "libx264", "-profile:v", "main", "-level", "4.0", "-crf", "17",
               "-bf", "0", "-g", "12", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "320k", "-ar", "48000", "-ac", "2",
               "-movflags", "+faststart", "-r", "12"])
        .arg(out)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| { eprintln!("render: ffmpeg: {e}"); std::process::exit(1) });
    let mut pipe = ff.stdin.take().unwrap();
    let film = run.film.clone();
    let mut frame = Vec::with_capacity(HD.0 * HD.1 * 3);
    for n in 0..frames {
        while !film.filed(n) { std::thread::sleep(Duration::from_millis(2)) }
        full_size(&film.get(n).expect("a frame let go before it was written"), &mut frame);
        if let Err(e) = pipe.write_all(&frame) { eprintln!("render: ffmpeg stopped at frame {n}: {e}"); break }
        film.release(n);
        if (n + 1) % 600 == 0 || n + 1 == frames {
            eprintln!("render: {} of {frames} frames, {:.0} s", n + 1, started.elapsed().as_secs_f64());
        }
    }
    drop(pipe);
    let status = ff.wait();
    let _ = std::fs::remove_file(&wav);
    match status {
        Ok(s) if s.success() => eprintln!("render: {out}, {frames} frames ({:.1} s of film), in {:.0} s", frames as f64 / 12.0, started.elapsed().as_secs_f64()),
        _ => { eprintln!("render: ffmpeg failed"); std::process::exit(1) }
    }
    std::process::exit(0)   // the workers are done with every frame; don't wait for them to wind down
}
