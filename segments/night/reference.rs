//! The night arrival (the train's ending), drawn in ordinary Rust with whole numbers: the reference the host's
//! frames are checked against. The train's renderer (segments/train) with the night's own parts (night.rs, path.rs,
//! nframe.rs, nmat.rs, npost.rs), every operation done natively, as its definition says; the host does each with
//! machines on CARDIAC. See README.md.
//!
//!     night_ref SEGMENTS_DIR [FRAMES]     the frames (0 .. 519, or those listed) as RGB on stdout (the world at half
//!                                         resolution, the subjects at full)
//!     night_ref SEGMENTS_DIR --camera      the dissolve's camera (data/camera_dissolve.txt's lines) on stdout
#![allow(clippy::needless_range_loop, clippy::too_many_arguments, dead_code)]
use std::io::Write;

include!("../train/ops.rs");
include!("../train/scene.rs");
include!("../train/render.rs");
include!("../train/frame.rs");
include!("../train/sky.rs");
include!("../train/shade.rs");
include!("../train/materials.rs");
include!("../train/tunnel.rs");
include!("../train/blur.rs");
include!("../train/wires.rs");
include!("../train/lens.rs");
include!("../train/mirror.rs");
include!("../train/shadow.rs");
include!("../train/overlays.rs");
include!("../train/party.rs");
include!("night.rs");
include!("path.rs");
include!("nframe.rs");
include!("nmat.rs");
include!("npost.rs");
include!("nseg.rs");
include!("nparty.rs");

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let mut o = Native::new(read_trig(dir));
    let nw = night_load(&mut o, dir);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    if a.get(2).map(|s| s.as_str()) == Some("--camera") {
        for l in dissolve_camera(&mut o, &nw) { writeln!(out, "{l}").unwrap(); }
        return;
    }
    if a.get(2).map(|s| s.as_str()) == Some("--cameras") {
        for f in 0..F_HAND { let df = design_f(&mut o, &nw, f); let c = night_cam(&mut o, &nw, 10 * df, false); writeln!(out, "{f} {} {} {} {} {} {} {}", c.eye[0], c.eye[1], c.eye[2], c.b[2][0], c.b[2][1], c.b[2][2], c.f2).unwrap(); }
        return;
    }
    let only: Option<Vec<i64>> = a.get(2).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let setup: std::collections::BTreeMap<(&str, &str), u64> = o.by_stage.clone();
    let mut nf = 0u64;
    for k in 0..F_HAND {
        if let Some(l) = &only { if !l.contains(&k) { continue; } }
        let img = night_frame_v(&mut o, &nw, k, std::env::var("NIGHT_FULL").is_err());
        out.write_all(&img).unwrap();
        out.flush().unwrap();
        nf += 1;
    }
    if std::env::var("NIGHT_COUNT").is_ok() {
        // a rough cost an item on CARDIAC (the train's estimates), instructions
        let cost = |op: &str| -> u64 { match op { "lin" => 10, "lin_limbs" => 45, "div" => 150, "sum" => 8, "mul" => 45, "min" | "max" | "lt" | "clamp" => 8,
            "muldiv" => 60, "pct" => 25, "mix" => 30, "walk" => 10, "walka" => 10, "walkc" => 10, "nearest" => 10, "runsum" => 6, "divmod" => 80, "wrap" => 10, "sqrt" => 200, "ldiff" => 10,
            "recip" => 25, "phase" => 60, "crossings" => 3000, "sincos" => 100, "turns" => 20, "light_sum" => 20, "tone_index" => 10, "tone_push" => 30, "up4" => 10, "ends" => 15, "vel" => 10, "absless" => 15, "lit16" => 40, _ => 0 } };
        let mut st: std::collections::BTreeMap<&str, u64> = Default::default();
        let mut tot = 0;
        for ((s_, op), c) in &o.by_stage { let c0 = setup.get(&(*s_, *op)).copied().unwrap_or(0); let x = (c - c0) * cost(op); *st.entry(s_).or_insert(0) += x; tot += x; }
        for (s_, c) in &st { eprintln!("{s_:12} {:>8.1}M a frame", (*c / nf.max(1)) as f64 / 1e6); }
        eprintln!("total {:.2}G a frame (setup {:.2}G)", (tot / nf.max(1)) as f64 / 1e9, setup.iter().map(|(k, c)| c * cost(k.1)).sum::<u64>() as f64 / 1e9);
        let mut ops: Vec<(&(&str, &str), u64)> = o.by_stage.iter().map(|(k, c)| (k, c - setup.get(k).copied().unwrap_or(0))).collect();
        ops.sort_by_key(|(k, c)| std::cmp::Reverse(*c * cost(k.1)));
        for ((s_, op), c) in ops.iter().take(25) { eprintln!("  {s_:10} {op:10} {:>9}k items {:>8.1}M", *c / nf.max(1) / 1000, (*c * cost(op) / nf.max(1)) as f64 / 1e6); }
    }
}
