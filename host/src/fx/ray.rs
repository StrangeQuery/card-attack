//! The ray's script: the motion of Mode X's mirror ball (its sway and its bounce) and the floor's scroll, worked
//! out on CARDIAC. The 192x96 ray tracer this folder once held is no longer in the film.
use crate::*;
use std::path::Path;

const HZ: i32 = 18;

pub fn render(demo: &Path, refbin: &str) {
    let m = |f: &str| one(demo, f);
    let (sky, zrow, floor, span, norm, bounce, hit, compose, paint16) = (m("ray/original/sky.asm"), m("ray/original/zrow.asm"), m("ray/original/floor.asm"),
        m("ray/original/span.asm"), m("ray/original/norm.asm"), m("ray/original/bounce.asm"), m("ray/original/hit.asm"), rust_or(demo, "compose", "ray/original/compose.asm"), one(demo, "shared/original/paint16.asm"));
    let palette = tables::flat(&[[40, 60, 150], [80, 120, 200], [150, 180, 230], [170, 180, 200], [200, 40, 40], [235, 225, 200], [110, 25, 25], [130, 125, 110],
        [110, 150, 220], [170, 195, 235], [160, 40, 45], [205, 200, 180], [150, 160, 180], [255, 255, 255], [0, 0, 0], [0, 0, 0]]);
    let script = script(demo);
    let frames = script.len();
    let mut host = Host::new();
    let mut video = Video::new(frames);
    let mut rays = 0;
    for (f, &[bx, by, xs, zs]) in script.iter().enumerate() {
        for y in 0..H as i32 {
            let d = y - HZ;
            let bg = if d < 4 { host.go1(&sky, &[&[y]], W) } else { let z = host.go1(&zrow, &[&[d, zs]], 5); host.go1(&floor, &[&z, &[d, xs, bx]], W) };
            let sp = host.gon(&span, &[&[y, bx, by]]);
            let (x0, n) = (sp[0], sp[1]);
            let mut ball = vec![];
            if n > 0 {
                let b = sp[2];
                for &a in &sp[3..] {
                    let o = host.go1(&norm, &[&[a, b]], 4);
                    let (c, rx, ry, rz) = (o[0], o[1], o[2], o[3]);
                    let ct = host.go1(&bounce, &[&[rx, ry, b, by]], 2); rays += 1;
                    ball.push(if ct[0] >= 0 { ct[0] } else { host.go1(&hit, &[&[ct[1], rx, rz, a, c, bx, xs, zs]], 1)[0] });
                }
            }
            // the compose machine in Rust runs in two phases: start, count and the ball's codes, then the row
            let row = if compose.tapes.len() == 2 { host.go(&compose, &[&[&[x0, n], &ball], &[&bg]], Some(W)) } else { host.go1(&compose, &[&[x0, n], &ball, &bg], W) };
            video.row(f, y as usize, &host.go1(&paint16, &[&palette, &row], W * 3));
        }
    }
    host.done();
    finish("ray", &host, &video, &reference(refbin, &[&demo.join("ray/data/script.txt")]), &demo.join("ray/video.rgb"), &format!(", {rays} reflected rays"));
}

/// Frames in the script.
pub const SCRIPT: usize = 120;

/// The script: per frame the ball's x and height (it bounces every two seconds), and the floor's scroll in x and z.
/// On CARDIAC (the sway and the bounce were the host's, in floats): the sway from the printed sine table
/// (data/sine120.txt, 999 sin of the frame's 120ths of a turn) by rt_sway, the height and the scroll by rt_hop. The
/// references read the same script from ray/data/script.txt, which is kept in the repository; a render that checks
/// its frames (not the live player) stops if what the machines made differs from that file.
pub fn script(demo: &Path) -> Vec<[i32; 4]> {
    let (sway, hop) = (rust(demo, "rt_sway"), rust(demo, "rt_hop"));
    let sine: Vec<i32> = read(demo, "ray/data/sine120.txt").lines().filter(|l| !l.starts_with('#')).map(|l| l.trim().parse().unwrap()).collect();
    let mut host = Host::new();
    let script: Vec<[i32; 4]> = (0..SCRIPT).map(|f| {
        let x = host.go1(&sway, &[&[sine[f]]], 1);
        let h = host.go1(&hop, &[&[f as i32]], 3);
        [x[0], h[0], h[1], h[2]]
    }).collect();
    live::count(host.steps);
    std::fs::write(demo.join("ray/data/script.txt"), script.iter().map(|s| s.map(|v| v.to_string()).join(" ")).collect::<Vec<_>>().join("\n") + "\n").unwrap();
    script
}
