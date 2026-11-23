// The ending's SQ: a hard cut from the CARDIAC on the giant screen to a big SQ in silver chrome,
// extruded and seen from the ground, low and to the left, close, STRANGE QUERY spaced out
// under it materialising: each of its pixels, at its turn, a white sparkle for two frames, then its chrome.
//
// Written once against the train's operations (segments/train/ops.rs). Once, on CARDIAC (the start): every chrome
// pixel's grey, the surround at its argument T (piecewise straight: 60 up high to 245 at the horizon, a dark band,
// 20 to 35 to 120 to 170 below), times M, plus A, held to 0..255; the stars' (A); the name's chrome. Each frame: how
// far the name has come in, which of its pixels are in and which still sparkle, each mixed over the picture by its
// cover. The host copies the picture's bytes and fetches by the indices.

pub const W: usize = 1536;
pub const H: usize = 768;
pub const FRAMES: i64 = 36;
pub const NAME0: i64 = 8;            // the name starts coming in at frame 8
pub const NAME_N: i64 = 14;          // over 14 frames
pub const SPARK: i64 = 143;          // a pixel sparkles for 2 / 14 of the way (thousandths)

pub struct World { pub grey: Vec<u8>, pub name_at: Vec<usize>, pub name_cover: Vec<i64>, pub name_order: Vec<i64>, pub name_grey: Vec<i64> }

/// A number of a data record, stored in 1, 2 or 4 bytes (segments/shared/records.py).
fn stored_number(b: &[u8]) -> i64 {
    match b.len() { 1 => b[0] as i8 as i64, 2 => i16::from_le_bytes([b[0], b[1]]) as i64, _ => i32::from_le_bytes(b.try_into().unwrap()) as i64 }
}

fn records(path: &str) -> std::collections::BTreeMap<String, Vec<i64>> {
    let b = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut out = std::collections::BTreeMap::new();
    let mut at = 0;
    let u = |b: &[u8], at: usize| u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    while at < b.len() {
        let n = u(&b, at); at += 4;
        let name = String::from_utf8(b[at..at + n].to_vec()).unwrap(); at += n;
        // the count of dimensions, with the bytes per number in its second byte (0 for 4)
        let nd = u(&b, at); at += 4;
        let w = match nd >> 8 { 0 => 4, w => w };
        let nd = nd & 255;
        let len: usize = (0..nd).map(|i| u(&b, at + 4 * i)).product(); at += 4 * nd;
        out.insert(name, (0..len).map(|i| stored_number(&b[at + w * i..at + w * (i + 1)])).collect());
        at += w * len;
    }
    out
}

/// The surround's grey at arguments t (thousandths), on CARDIAC: piecewise straight.
fn env<O: Ops>(o: &mut O, t: &V) -> V {
    // the four pieces: (from, to, grey at from, grey at to)
    let piece = |o: &mut O, t0: i64, t1: i64, g0: i64, g1: i64| -> V {
        let d = o.sum(&[(v(t), 1)], -t0);
        let x = o.lin(&[(v(&d), Some(K(g1 - g0)))]);
        let x = o.div(v(&x), K(t1 - t0), 0);
        o.sum(&[(v(&x), 1)], g0)
    };
    let sky = piece(o, -1000, -50, 60, 245);
    let band = piece(o, -50, 0, 20, 35);
    let low = piece(o, 0, 150, 35, 120);
    let ground = piece(o, 150, 1000, 120, 170);
    let up = o.lt(v(t), K(-50));
    let in_band = o.lt(v(t), K(0));
    let in_low = o.lt(v(t), K(150));
    let x = o.sel(&in_low, v(&low), v(&ground));
    let x = o.sel(&in_band, v(&band), v(&x));
    o.sel(&up, v(&sky), v(&x))
}

/// The start: the data read; the picture's grey worked out (the chrome's pixels on CARDIAC, the stars as they are);
/// the name's chrome.
pub fn load<O: Ops>(o: &mut O, dir: &str) -> World {
    let mut r = records(&format!("{dir}/sqend/data/sqend.bin"));
    let (t, m, a) = (r.remove("t").unwrap(), r.remove("m").unwrap(), r.remove("a").unwrap());
    let chrome: Vec<usize> = (0..W * H).filter(|&i| m[i] > 0).collect();
    let mut grey: Vec<u8> = a.iter().map(|&x| x as u8).collect();
    let tv: V = chrome.iter().map(|&i| t[i]).collect();
    let mv: V = chrome.iter().map(|&i| m[i]).collect();
    let av: V = chrome.iter().map(|&i| a[i]).collect();
    let g = env(o, &tv);
    let x = o.lin(&[(v(&g), Some(v(&mv)))]);
    let x = o.div(v(&x), K(1000), 0);
    let x = o.sum(&[(v(&x), 1), (v(&av), 1)], 0);
    let x = o.clamp(v(&x), 0, 255);
    for (k, &i) in chrome.iter().enumerate() { grey[i] = x[k] as u8; }
    let cover = r.remove("name_cover").unwrap();
    let name_at: Vec<usize> = (0..W * H).filter(|&i| cover[i] > 0).collect();
    let nt: V = name_at.iter().map(|&i| r["name_t"][i]).collect();
    let name_grey = env(o, &nt);
    World { grey, name_cover: name_at.iter().map(|&i| cover[i]).collect(), name_order: name_at.iter().map(|&i| r["name_order"][i]).collect(), name_at, name_grey }
}

/// Frame k: RGB 1536 x 768, grey.
pub fn render<O: Ops>(o: &mut O, w: &World, k: i64) -> Vec<u8> {
    let mut g = w.grey.clone();
    // how far the name has come in (thousandths): (k - NAME0) 1000 / NAME_N
    let d = one(o.sum(&[(K(k), 1)], -NAME0));
    let d = one(o.lin(&[(K(d), Some(K(1000)))]));
    let d = one(o.clamp(K(d), 0, 999_000));
    let u = one(o.div(K(d), K(NAME_N), 0));
    if u > 0 {
        let on = o.lt(v(&w.name_order), K(u));
        let lit: Vec<usize> = (0..w.name_at.len()).filter(|&j| on[j] == 1).collect();
        if !lit.is_empty() {
            let ord: V = lit.iter().map(|&j| w.name_order[j]).collect();
            let back = one(o.sum(&[(K(u), 1)], -SPARK));
            let spark = o.lt(K(back), v(&ord));
            let chrome: V = lit.iter().map(|&j| w.name_grey[j]).collect();
            let col = o.sel(&spark, K(255), v(&chrome));
            // over the picture by the cover: b + (col - b) cover / 255
            let base: V = lit.iter().map(|&j| g[w.name_at[j]] as i64).collect();
            let cov: V = lit.iter().map(|&j| w.name_cover[j]).collect();
            let diff = o.sum(&[(v(&col), 1), (v(&base), -1)], 0);
            let x = o.lin(&[(v(&diff), Some(v(&cov)))]);
            let x = o.div(v(&x), K(255), 0);
            let x = o.sum(&[(v(&x), 1), (v(&base), 1)], 0);
            for (k2, &j) in lit.iter().enumerate() { g[w.name_at[j]] = x[k2] as u8; }
        }
    }
    g.iter().flat_map(|&b| [b, b, b]).collect()
}

fn one(x: V) -> i64 { x[0] }
