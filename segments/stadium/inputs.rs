// What the stadium reads from outside: the night arrival's camera for the dissolve (night/data/camera_dissolve.txt,
// as the night segment publishes it) and its frames before the hand-off. Included after frame.rs.

/// The night arrival's camera for its frames F_HAND - 44 .. F_HAND - 1 (our 0..43): the night segment's file, one
/// line a frame, `frame t12 ex ey ez rx ry rz ux uy uz fx fy fz focal` (the eye in our frame in centimetres, its axes
/// in thousandths; the focal length within a quarter of a percent of ours, which is taken). Without it, the test file
/// here (eye and the point looked at, from the design's NightCam: `k ex ey ez tx ty tz`).
pub fn night_cams(dir: &str) -> NightCams {
    let theirs = format!("{dir}/night/data/camera_dissolve.txt");
    let path = if std::path::Path::new(&theirs).exists() { theirs } else { format!("{dir}/stadium/data/camera_dissolve.txt") };
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut out = vec![];
    let mut frames = vec![];
    for l in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let x: Vec<i64> = l.split_whitespace().map(|v| v.parse().unwrap()).collect();
        if x.len() == 15 {
            frames.push(x[0]);
            out.push(NightCam::Axes(Cam { eye: [x[2], x[3], x[4]], b: [[x[5], x[6], x[7]], [x[8], x[9], x[10]], [x[11], x[12], x[13]]] }));
        } else {
            let n = x.len();
            out.push(NightCam::Look([x[n - 6], x[n - 5], x[n - 4]], [x[n - 3], x[n - 2], x[n - 1]]));
        }
    }
    // its last 44 (the frames before its hand-off)
    assert!(out.len() >= DISSOLVE as usize, "{path}: {} cameras", out.len());
    let out = out.split_off(out.len() - DISSOLVE as usize);
    let frames = if frames.len() >= DISSOLVE as usize { frames.split_off(frames.len() - DISSOLVE as usize) } else { vec![] };
    NightCams(out, frames)
}

/// The night arrival's frames for the dissolve: the frames its camera file numbers (its last 44 before the
/// hand-off), else its video's last 44 (its video holds its frames 0 .. F_HAND - 1; NIGHT_F_HAND says otherwise).
pub fn night_dissolve_frames(night: &[u8], cams: &NightCams) -> Vec<u8> {
    let f = FULL_NPIX * 3;
    let n = night.len() / f;
    let hand = match std::env::var("NIGHT_F_HAND") {
        Ok(s) => s.parse::<usize>().unwrap(),
        Err(_) => match cams.1.last() { Some(&last) if (last as usize) < n && cams.1.windows(2).all(|w| w[1] == w[0] + 1) => last as usize + 1, _ => n },
    };
    assert!(hand >= DISSOLVE as usize && hand <= n, "the night's video: {n} frames, its hand-off at {hand}");
    night[(hand - DISSOLVE as usize) * f..hand * f].to_vec()
}
