// The night's frames as the film has them: the first XFADE frames dissolving in from the ride's own world, the
// cab's view dissolving to the platform's (F_GO .. F_VIEW), the rest the night's own (its last DISSOLVE frames are
// our layer of the stadium's dissolve, which the stadium segment lays its render over). And the camera the stadium
// takes for that dissolve. Included after npost.rs.

/// Two pictures mixed: a's to b's by t thousandths (as percent), RGB.
pub fn mix_rgb<O: Ops>(o: &mut O, a: &[u8], b: &[u8], t: i64) -> Vec<u8> {
    let tp = one(o.div(K(t), K(10), 0));
    let av: V = a.iter().map(|&x| x as i64).collect();
    let bv: V = b.iter().map(|&x| x as i64).collect();
    let m = o.mix(v(&av), v(&bv), K(tp));
    m.iter().map(|&x| x as u8).collect()
}

/// The night's frame k (0 .. F_HAND - 1), RGB: the full version (the party version: nparty.rs).
pub fn night_frame<O: Ops>(o: &mut O, nw: &NightWorld, k: i64) -> Vec<u8> { night_frame_v(o, nw, k, false) }

/// The camera for the stadium's dissolve, frames F_HAND - DISSOLVE .. F_HAND: frame, the stadium's clock in
/// twelfths, the eye (the stadium's frame, cm), right, up and forward (thousandths, that frame), the focal length
/// (tenths of a pixel at 1536 wide). See README.md.
pub fn dissolve_camera<O: Ops>(o: &mut O, nw: &NightWorld) -> Vec<String> {
    let ac = nw.aud_c;
    (F_HAND - DISSOLVE..=F_HAND).map(|f| {
        let df = design_f(o, nw, f);
        let cam = night_cam(o, nw, 10 * df, false);
        // the eye: (x - BX - C.x, z - C.z, y - C.y), mm to cm
        let ex = one({ let t = o.lin(&[(K(cam.eye[0]), None), (K(10 * BX), Some(K(-1))), (K(ac[0]), Some(K(-10)))]); o.div(v(&t), K(10), 0) });
        let ey = one({ let t = o.lin(&[(K(cam.eye[2]), None), (K(ac[2]), Some(K(-10)))]); o.div(v(&t), K(10), 0) });
        let ez = one({ let t = o.lin(&[(K(cam.eye[1]), None), (K(ac[1]), Some(K(-10)))]); o.div(v(&t), K(10), 0) });
        let fo = one(o.lin(&[(K(cam.f2), Some(K(5)))]));
        let b = cam.b;
        let ax = |r: [i64; 3]| format!("{} {} {}", r[0], r[2], r[1]);
        format!("{f} {} {ex} {ey} {ez} {} {} {} {fo}", f - F_HAND + 36, ax(b[0]), ax(b[1]), ax(b[2]))
    }).collect()
}
