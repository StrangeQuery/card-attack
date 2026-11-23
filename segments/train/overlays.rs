// The words over the city: the ray-generation shader top left (from 0.3 s, gone by
// 2.8 s) and the brag bottom left (1.2 s to 3.7 s), each over a dark glass panel, with a shadow. On the frame's
// 0..255 values. Included after shadow.rs.

/// How far in a word is at frame t: min((t - a) / b, (c - t) / d) held to 0..1, percent.
fn level_at<O: Ops>(o: &mut O, t: i64, a: i64, b: i64, c: i64, d: i64) -> i64 {
    let up = { let x = o.lin(&[(K(t), Some(K(100))), (K(a), Some(K(-100)))]); o.div(v(&x), K(b), 0) };
    let down = { let x = o.lin(&[(K(c), Some(K(100))), (K(t), Some(K(-100)))]); o.div(v(&x), K(d), 0) };
    let m = o.min(v(&up), v(&down));
    one(o.clamp(v(&m), 0, 100))
}

pub fn overlays<O: Ops>(o: &mut O, tb: &Tb, k: i64, img: &mut [V; 3]) {
    let lv = [level_at(o, k, 4, 4, 34, 8), level_at(o, k, 14, 3, 44, 4)];
    let names = ["ink_code", "ink_brag"];
    let shadows = ["ink_code_sh", "ink_brag_sh"];
    let boxes = &tb.t("ink_box").data;
    // the panels: each channel times 1 - 0.55 lv
    for (i, &l) in lv.iter().enumerate() {
        if l <= 0 { continue; }
        let b = &boxes[4 * i..4 * i + 4];
        let at: Vec<usize> = (b[1]..b[3]).flat_map(|y| (b[0]..b[2]).map(move |x| (y * W() + x) as usize)).collect();
        let kf = { let m = one(o.muldiv(K(55), K(l), 100)); one(o.sum(&[(K(m), -1)], 100)) };
        for c in 0..3 {
            let cur: V = at.iter().map(|&p| img[c][p]).collect();
            let r = o.pct(v(&cur), K(kf));
            for (j, &p) in at.iter().enumerate() { img[c][p] = r[j]; }
        }
    }
    // the words: their shadow (the ink 2 pixels down and right, laid out at the start; 0.75), then the ink toward white
    for (i, &l) in lv.iter().enumerate() {
        if l <= 0 { continue; }
        let ink = &tb.t(names[i]).data;
        let shm = &tb.t(shadows[i]).data;
        let at: Vec<usize> = (0..N_PIX()).filter(|&p| ink[p] > 0 || shm[p] > 0).collect();
        let sh: V = at.iter().map(|&p| shm[p]).collect();
        let iv: V = at.iter().map(|&p| ink[p]).collect();
        // percent of each: the shadow 0.75 lv, the ink lv
        let shk = { let t = o.muldiv(v(&sh), K(75), 255); o.muldiv(v(&t), K(l), 100) };
        let ik = { let t = o.muldiv(v(&iv), K(100), 255); o.muldiv(v(&t), K(l), 100) };
        let keep = o.sum(&[(v(&shk), -1)], 100);
        for c in 0..3 {
            let cur: V = at.iter().map(|&p| img[c][p]).collect();
            let r = o.pct(v(&cur), v(&keep));
            let r = o.mix(v(&r), K(255), v(&ik));
            for (j, &p) in at.iter().enumerate() { img[c][p] = r[j]; }
        }
    }
}
