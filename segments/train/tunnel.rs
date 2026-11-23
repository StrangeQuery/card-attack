// The tunnel's light: its lamps (on both walls every 12 m) and our headlights, as two maps worked out once at the
// start (the prototype's tunnel_light at places through a lamp's 12 m, and ahead of the cab), looked up by each
// pixel's place. R (250 for 1.0) before the colour and the exposure. Included after materials.rs.

pub struct TunnelMaps { pub lamps: Tab, pub head: Tab }

/// The light at a map's places from lights at (x, y, z) (centimetres; z from the place's own z for the lamps'
/// map, from the cab for the headlights'), each place's normal from the map's grid.
fn light_at<O: Ops>(o: &mut O, tb: &Tb, grid: &str, lights: &[(i64, i64, i64)], head: bool) -> V {
    let g = |k: &str| -> V { tb.t(&format!("{grid}_{k}")).data.clone() };
    let (x, y, z, nx, ny) = (g("x"), g("y"), g("z"), g("nx"), g("ny"));
    let n = x.len();
    let mut acc = vec![0i64; n];
    for &(lx, ly, lz) in lights {
        // the way to the light, decimetres
        let vx = { let t = o.lin(&[(v(&x), Some(K(-1))), (K(lx), None)]); o.div(v(&t), K(10), 0) };
        let vy = { let t = o.lin(&[(v(&y), Some(K(-1))), (K(ly), None)]); o.div(v(&t), K(10), 0) };
        let vz = { let t = o.lin(&[(v(&z), Some(K(-1))), (K(lz), None)]); o.div(v(&t), K(10), 0) };
        let d2 = o.lin(&[(v(&vx), Some(v(&vx))), (v(&vy), Some(v(&vy))), (v(&vz), Some(v(&vz)))]);
        let di = { let t = o.div(v(&d2), K(10), 0); o.clamp(v(&t), 0, 29_999) };
        let a = o.mul(v(&vx), v(&nx), 2);
        let b = o.mul(v(&vy), v(&ny), 2);
        let vn = o.sum(&[(v(&a), 1), (v(&b), 1)], 0);
        let vn = o.clamp(v(&vn), 0, 999);
        let c = if !head {
            let gg = o.tab(tb.t("lamp_g"), v(&di));
            o.mul(v(&vn), v(&gg), 3)
        } else {
            // the cone: -(v.dir) / |v|, dir (0, -0.06, 1) made a unit
            let dist = o.sqrt(v(&d2));
            let dist = o.max(v(&dist), K(1));
            let num = o.lin(&[(v(&vy), Some(K(60))), (v(&vz), Some(K(-1000)))]);
            let num = o.max(v(&num), K(0));
            let den = o.lin(&[(v(&dist), Some(K(1002)))]);
            let den = o.div(v(&den), K(1000), 0);
            let den = o.max(v(&den), K(1));
            let cc = o.div(v(&num), v(&den), 0);
            let cc = o.clamp(v(&cc), 0, 1000);
            let cone = o.tab(tb.t("head_cone"), v(&cc));
            let gg = o.tab(tb.t("head_g"), v(&di));
            let a = o.lin(&[(v(&vn), Some(v(&gg)))]);
            let a = o.div(v(&a), K(1000), 0);
            let a = o.min(v(&a), K(999_999));
            let a = o.lin(&[(v(&a), Some(v(&cone)))]);
            let a = o.div(v(&a), K(1000), 0);
            o.min(v(&a), K(999))
        };
        acc = o.sum(&[(v(&acc), 1), (v(&c), 1)], 0);
        acc = o.min(v(&acc), K(999));
    }
    acc
}

pub fn tunnel_maps<O: Ops>(o: &mut O, tb: &Tb) -> TunnelMaps {
    let mut lamps = vec![];
    for side in [-1i64, 1] { for k in -2..=2i64 { lamps.push((MID + side * 516, 420, 1200 * k)); } }
    // worked out at the places near the surfaces, put in their maps (the rest 0)
    let place = |vals: V, grid: &str, nz: usize| -> V {
        let mut m = vec![0; nz * 48 * 63];
        for (k, &i) in tb.t(&format!("{grid}_at")).data.iter().enumerate() { m[i as usize] = vals[k]; }
        m
    };
    let lm = place(light_at(o, tb, "lampgrid", &lamps, false), "lampgrid", 120);
    let hm = place(light_at(o, tb, "headgrid", &[(-75, 80, 150), (75, 80, 150)], true), "headgrid", 200);
    TunnelMaps { lamps: Tab { name: "lampmap", data: lm, w: 1 }, head: Tab { name: "headmap", data: hm, w: 1 } }
}

/// A map's value between its places across (every 20 cm, from 10 cm in) and, given a fraction along (percent, or
/// none), between its rows z0 and z0 + 1; by the height's row yi.
fn map_look<O: Ops>(o: &mut O, t: &Tab, z0: &V, fz: Option<&V>, x: &V, yi: &V) -> V {
    let xx = o.lin(&[(v(x), None), (K(620 - MID - 10), None)]);
    let xx = o.clamp(v(&xx), 0, 20 * 62 - 1);
    // over 20 with the fraction in percent: 5 xx over 100
    let x5 = o.sum(&[(v(&xx), 5)], 0);
    let (x0, fx) = o.divmod(v(&x5), K(100));
    // the host fetches the map's entries by their three indices (a place away from the tunnel's surfaces was never
    // worked out: no light there)
    let at = |dzi: i64, dxi: i64| -> V { (0..z0.len()).map(|k| t.data[((z0[k] + dzi) * 3024 + yi[k] * 63 + x0[k] + dxi) as usize]).collect() };
    let p = o.mix(v(&at(0, 0)), v(&at(0, 1)), v(&fx));
    match fz {
        None => p,
        Some(fz) => { let q = o.mix(v(&at(1, 0)), v(&at(1, 1)), v(&fx)); o.mix(v(&p), v(&q), v(fz)) }
    }
}

/// The lamps' and the headlights' light (R, exposed) at places in the tunnel.
pub fn tunnel_light<O: Ops>(o: &mut O, tm: &TunnelMaps, cam: &Cam, e: i64, x: &V, y: &V, z: &V) -> [V; 3] {
    let yi = { let t = o.sum(&[(v(y), 5)], 250); let t = o.div(v(&t), K(100), 0); o.clamp(v(&t), 0, 47) };
    // along a lamp's 12 m: decimetres past the lamps' first, md 120 (the map's rows, every 10 cm)
    let zq = { let t = o.lin(&[(v(z), None), (K(2400 - TUN0 - 665), None)]); let t = o.max(v(&t), K(0)); o.div(v(&t), K(10), 0) };
    let (_, zp) = o.divmod(v(&zq), K(120));
    let lamp = map_look(o, &tm.lamps, &zp, None, x, &yi);
    // ahead of the cab: rows every 50 cm from 25 cm, the fraction 2 (u md 50) percent
    let ahead = { let c = kl(o, &[(cam.eye_cm[2], Some(-1)), (-25, None)]); o.lin(&[(v(z), None), (K(c), None)]) };
    let u = o.clamp(v(&ahead), 0, 50 * 199 - 1);
    let u2 = o.sum(&[(v(&u), 2)], 0);
    let (z0, fz) = o.divmod(v(&u2), K(100));
    let xh = { let c = kl(o, &[(cam.eye_cm[0], Some(-1))]); o.lin(&[(v(x), None), (K(c), None)]) };
    let head = map_look(o, &tm.head, &z0, Some(&fz), &xh, &yi);
    // each channel: the lamps' and the headlights' colours and the exposure, by tables worked out for the frame
    let all: V = (0..1000).collect();
    std::array::from_fn(|c| {
        let tl = { let t = o.mul(v(&all), K([999, 700, 380][c]), 3); Tab { name: "lamp_c", data: mulk(o, v(&t), K(e)), w: 0 } };
        let th = { let t = o.mul(v(&all), K([900, 950, 999][c]), 3); Tab { name: "head_c", data: mulk(o, v(&t), K(e)), w: 0 } };
        let a = o.tab(&tl, v(&lamp));
        let b = o.tab(&th, v(&head));
        let s = o.sum(&[(v(&a), 1), (v(&b), 1)], 0);
        o.min(v(&s), K(999))
    })
}
