// Mirrors: what stands above the water, flipped below its level and drawn from the
// same camera, so the paddies and the river can show it, shifted by their ripples, where it is behind the water's
// surface. Included after lens.rs.

/// A mirror's buffer and the water's level (centimetres).
pub struct Mirror1 { pub g: GBuf, pub h: i64 }
pub struct Mirrors { pub paddy: Option<Mirror1>, pub water: Option<Mirror1> }

/// The picked triangles of a list flipped about the level h: their corners' heights 2h - y, their normals' up
/// parts the other way, the planes' distances with them (fd - 2 fy h).
pub fn flip_mesh<O: Ops>(o: &mut O, mesh: &Mesh, corners: &[[i64; 3]], pick: &[usize], h: i64) -> Mesh {
    let mut used: Vec<usize> = pick.iter().flat_map(|&k| mesh.tris[k]).collect();
    used.sort();
    used.dedup();
    let at: std::collections::HashMap<usize, usize> = used.iter().enumerate().map(|(i, &u)| (u, i)).collect();
    let ys: V = used.iter().map(|&u| corners[u][1]).collect();
    let fy = o.lin(&[(v(&ys), Some(K(-1))), (K(2 * h), None)]);
    let cs: Vec<[i64; 3]> = used.iter().enumerate().map(|(i, &u)| [corners[u][0], fy[i], corners[u][2]]).collect();
    let vny: Vec<V> = (0..3).map(|j| { let t: V = pick.iter().map(|&k| mesh.vn[k][3 * j + 1]).collect(); o.sum(&[(v(&t), -1)], 0) }).collect();
    let fny: V = pick.iter().map(|&k| mesh.fnorm[k][1]).collect();
    let fny2 = o.sum(&[(v(&fny), -1)], 0);
    let fd: V = pick.iter().map(|&k| mesh.fd[k]).collect();
    let fd2 = o.lin(&[(v(&fd), None), (v(&fny), Some(K(-2 * h)))]);
    Mesh {
        corners: cs,
        tris: pick.iter().map(|&k| mesh.tris[k].map(|u| at[&u])).collect(),
        vn: pick.iter().enumerate().map(|(i, &k)| { let mut n = mesh.vn[k]; for j in 0..3 { n[3 * j + 1] = vny[j][i]; } n }).collect(),
        fnorm: pick.iter().enumerate().map(|(i, &k)| [mesh.fnorm[k][0], fny2[i], mesh.fnorm[k][2]]).collect(),
        fd: fd2,
        mat: pick.iter().map(|&k| mesh.mat[k]).collect(),
        smooth: pick.iter().map(|&k| mesh.smooth[k]).collect(),
    }
}

/// How much smaller the mirrors' buffers are each way than the screen.
pub const MS: usize = 2;

/// The mirror's buffer for the water at level h, at 1 / MS of the screen's size each way (its pixels at every MS-th
/// row and column): the world's triangles (those the frame considers, above the level, within 450 m) and the other train's,
/// flipped; for the river also the land beyond it, flipped.
pub fn mirror_gbuf<O: Ops>(o: &mut O, w: &World, cam: &Cam, k: i64, h: i64, ray: &[V; 3], vis: &(Vec<usize>, Vec<usize>)) -> GBuf {
    let cands: Vec<usize> = vis.0.iter().chain(vis.1.iter()).copied().collect();
    mirror_gbuf_of(o, w, cam, Some(k), h, ray, &cands, &w.screen().nlq7)
}

/// The same from the world's triangles a frame considers (`cands`), with the other train at ride frame k if given,
/// the screen's ray factors given.
pub fn mirror_gbuf_of<O: Ops>(o: &mut O, w: &World, cam: &Cam, k: Option<i64>, h: i64, ray: &[V; 3], cands: &[usize], nlq7: &[i64]) -> GBuf {
    let sc = &w.sc;
    let mut g = GBuf { iz: vec![0; N_PIX()], mat: vec![-1; N_PIX()], n: [vec![0; N_PIX()], vec![0; N_PIX()], vec![0; N_PIX()]], src: vec![0; N_PIX()], tri: vec![0; N_PIX()] };
    let cz = metres(o, cam.z_cm);
    let above = |m: &Mesh, c: &[[i64; 3]], t: usize| -> bool { m.tris[t].iter().any(|&u| c[u][1] > h + 5) };
    // those above the level, and of them those within 450 m (zlo - 450 < cz, on the machines)
    let up: Vec<usize> = cands.iter().copied().filter(|&t| above(&sc.world, &sc.world.corners, t)).collect();
    let near = o.lt(v(&up.iter().map(|&t| w.cull.mirror[t]).collect()), K(cz));
    let pick: Vec<usize> = (0..up.len()).filter(|&j| near[j] == 1).map(|j| up[j]).collect();
    if h == RIVER {
        // the land beyond the river, upside down in it: the plane at 2h - LAND, where it lies past the river's ends
        let iz0 = ground_depth(o, &w.tb, cam);
        let down = o.lt(K(74), v(&iz0));
        let px: Vec<usize> = (0..N_PIX()).filter(|&p| down[p] == 1 && p % MS == 0 && (p / W() as usize) % MS == 0).collect();
        if !px.is_empty() {
            // (the mirrored plane's height and ratio on the machines)
            let hl = kl(o, &[(h, Some(2)), (-LAND, None)]);
            let dm = kl(o, &[(cam.eye[1], None), (hl, Some(-10))]);
            let ratio = if dm <= 99_999 { let num = kl(o, &[(cam.eye[1], Some(1000))]); one(o.div(K(num), K(dm), 0)) }
                else { let (num, d) = (kl(o, &[(cam.eye_cm[1], Some(1000))]), kl(o, &[(cam.eye_cm[1], None), (hl, Some(-1))])); one(o.div(K(num), K(d), 0)) };
            let izp: V = px.iter().map(|&p| iz0[p]).collect();
            let iz1 = o.lin(&[(v(&izp), Some(K(ratio)))]);
            let iz1 = o.div(v(&iz1), K(1000), 0);
            let iz1 = o.max(v(&iz1), K(1));
            let r: [V; 3] = std::array::from_fn(|i| px.iter().map(|&p| ray[i][p]).collect());
            let pl = places(o, &w.tb, cam, &[&r[0], &r[1], &r[2]], &iz1, false);
            let z = &pl[2];
            let past = o.lt(K(RIV1), v(z));
            let short = o.lt(v(z), K(RIV0));
            let bank_far = o.lt(v(z), K(RIV1 + BANK));
            let bank_near = o.lt(K(RIV0 - BANK), v(z));
            for (j, &p) in px.iter().enumerate() {
                if past[j] == 0 && short[j] == 0 { continue; }
                let bank = (past[j] == 1 && bank_far[j] == 1) || (short[j] == 1 && bank_near[j] == 1);
                g.iz[p] = iz1[j]; g.mat[p] = if bank { kind::BANK } else { kind::PADDY }; g.n[1][p] = -99; g.src[p] = 0;
            }
        }
    }
    let fm = flip_mesh(o, &sc.world, &sc.world.corners, &pick, h);
    let two: Vec<bool> = fm.mat.iter().map(|&m| sc.mats[m].kind == kind::TREE).collect();
    let all: Vec<usize> = (0..fm.tris.len()).collect();
    let d = draw(o, &w.tb, cam, &fm, &fm.corners, &two, &all, false, MS as i64);
    let mut lists: Vec<(u8, Drawn, Mesh)> = vec![(1, d, fm)];
    let (front, tc) = match k { Some(k) => train_corners(o, sc, k), None => (i64::MIN / 4, vec![]) };
    // the other train's nose from the camera, on the machines, against the reach
    let ahead = k.map(|_| kl(o, &[(front, None), (cam.z_cm, Some(-1))]));
    if matches!(ahead, Some(a) if a < 45_000 && a > -500 - CARS * CAR) {
        let tp: Vec<usize> = (0..sc.train.tris.len()).filter(|&t| above(&sc.train, &tc, t)).collect();
        let pt = placed_train(o, sc, &tc);
        let tm = flip_mesh(o, &pt, &tc, &tp, h);
        let none = vec![false; tm.tris.len()];
        let all: Vec<usize> = (0..tm.tris.len()).collect();
        let d = draw(o, &w.tb, cam, &tm, &tm.corners, &none, &all, false, MS as i64);
        lists.push((2, d, tm));
    }
    let refs: Vec<(u8, &Drawn, &Mesh)> = lists.iter().map(|(s, d, m)| (*s, d, m)).collect();
    resolve(o, &mut g, &refs, Some(Cut { w, cam, ray, mirror_h: Some(h), nlq7 }));
    g
}
