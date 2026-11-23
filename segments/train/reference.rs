//! The bullet train, drawn in ordinary Rust with whole numbers: the reference the host's frames are
//! checked against. The renderer itself (render.rs, frame.rs, shade.rs) is written once, against the operations of
//! ops.rs: here each operation is done natively, as its definition says; the host does each with machines on
//! CARDIAC. See README.md.
#![allow(clippy::needless_range_loop, clippy::too_many_arguments)]
use std::io::Write;

include!("ops.rs");
include!("scene.rs");
include!("render.rs");
include!("frame.rs");
include!("sky.rs");
include!("shade.rs");
include!("materials.rs");
include!("tunnel.rs");
include!("blur.rs");
include!("wires.rs");
include!("lens.rs");
include!("mirror.rs");
include!("shadow.rs");
include!("overlays.rs");
include!("party.rs");

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let only: Option<Vec<i64>> = a.get(2).map(|s| s.split(',').map(|v| v.parse().unwrap()).collect());
    let mut o = Native::new(read_trig(dir));
    let world = load(&mut o, dir);
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    // the standing frames are one picture: drawn once
    let mut last: Option<(i64, Vec<u8>)> = None;
    for k in 0..FRAMES {
        if let Some(l) = &only { if !l.contains(&k) { continue; } }
        // the party version (the world at half resolution, the board and the words at full), or with TRAIN_FULL the full one
        if last.as_ref().is_none_or(|(t, _)| *t != ride_t(k)) { last = Some((ride_t(k), render_v(&mut o, &world, k, std::env::var("TRAIN_FULL").is_err()))); }
        out.write_all(&last.as_ref().unwrap().1).unwrap();
    }
    if std::env::var("TRAIN_COUNT").is_ok() {
        let n = only.as_ref().map_or(FRAMES as usize, |l| l.len()) as u64;
        // a rough cost a item on CARDIAC, instructions
        let cost = |op: &str| -> u64 { match op { "lin" => 10, "lin_limbs" => 45, "div" => 150, "sum" => 8, "mul" => 45, "min" | "max" | "lt" | "clamp" => 8,
            "muldiv" => 60, "pct" => 25, "mix" => 30, "walk" => 10, "nearest" => 10, "runsum" => 6, "divmod" => 80, "wrap" => 10, "sqrt" => 200,
            "recip" => 25, "phase" => 60, "crossings" => 3000, "sincos" => 100, _ => 0 } };
        let mut st: std::collections::BTreeMap<&str, u64> = Default::default();
        for ((s, op), c) in &o.by_stage { *st.entry(s).or_insert(0) += c * cost(op); }
        for (s, c) in &st { eprintln!("{s:12} {:>8.1}M a frame", (*c / n) as f64 / 1e6); }
        let mut ops: Vec<(&(&str, &str), &u64)> = o.by_stage.iter().collect();
        ops.sort_by_key(|(k, c)| std::cmp::Reverse(**c * cost(k.1)));
        for ((s, op), c) in ops.iter().take(25) { eprintln!("  {s:10} {op:10} {:>9}k items {:>8.1}M", **c / n / 1000, (**c * cost(op) / n) as f64 / 1e6); }
    }
}
