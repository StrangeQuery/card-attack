//! Presenting: a fullscreen window that shows frame n of the film at n/12 s from the clock's start, each frame
//! scaled to the screen at 2:1, letterboxed, nearest neighbour. Escape (or closing the window) quits. The pictures
//! are taken from the film's slots as they fill; a frame not there when its time comes is late: the last one stays
//! up and it is counted. Scaling to the screen is presentation, a blit: no pixel is computed here.
use super::{Film, Picture};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Fullscreen, Window, WindowId};

/// What the showing came to.
#[derive(Default, Debug)]
pub struct Shown { pub frames: usize, pub late: Vec<usize>, pub quit_at: Option<usize>, pub worst_draw_ms: f64,
    /// The most any frame went up after its time: from its time to the end of its draw.
    pub worst_behind_ms: f64, pub worst_behind_at: usize, pub behind_ms: Vec<f64>, pub woke_ms: Vec<f64>, due: Option<(usize, Instant)> }

struct App {
    film: Arc<Film>,
    first: usize,
    end: usize,
    start: Instant,
    windowed: bool,
    /// macOS: fullscreen in a Space of its own (as the green button does) instead of simple fullscreen.
    space: bool,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    /// The frame up now (the film's number), and the picture it showed (a late frame keeps the last picture).
    up: Option<usize>,
    pic: Option<(usize, Picture)>,
    /// The next frame scaled to the screen ahead of its time: (frame, screen width, height, pixels).
    ahead: Option<(usize, usize, usize, Vec<u32>)>,
    shown: Shown,
    /// The running count of CARDIAC instructions: the frames summed so far (0..counted) and their total.
    counted: usize,
    running: u64,
    /// When the last second's frames went up (for the frame rate), and the HUD's font at the screen's height.
    ups: std::collections::VecDeque<Instant>,
    hud: Option<(usize, Hud)>,
}

pub const FPS: f64 = 12.0;

impl App {
    fn due(&self) -> Option<usize> {
        let now = Instant::now();
        let first = self.start + Duration::from_secs_f64(self.first as f64 / FPS);
        if now < first { return None }
        Some((((now - self.start).as_secs_f64() * FPS) as usize).max(self.first))
    }

    fn draw(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else { return };
        let t = Instant::now();
        let size = window.inner_size();
        let (Some(sw), Some(sh)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        surface.resize(sw, sh).unwrap();
        let mut buf = surface.buffer_mut().unwrap();
        let (sw, sh) = (size.width as usize, size.height as usize);
        match (&self.pic, self.ahead.take()) {
            // scaled ahead, after the last frame went up: only copied now
            (Some((n, _)), Some((k, w, h, ready))) if k == *n && (w, h) == (sw, sh) => buf.copy_from_slice(&ready),
            (Some((_, p)), _) => scale(p, sw, sh, &mut buf),
            (None, _) => buf.fill(0),
        }
        // the running count, in the picture's lower right: every instruction its frames took, up to the one up now
        if let Some(up) = self.up {
            while self.counted <= up { self.running += self.film.tally(self.counted).unwrap_or(0); self.counted += 1; }
            // and the rate: the last second of film's frames (12 of them), what they took
            let second: u64 = (up.saturating_sub(FPS as usize - 1)..=up).filter_map(|n| self.film.tally(n)).sum();
            if self.film.tally(up).is_some() {
                // the frame rate: the frames that went up in the last second, as the window put them up
                let now = Instant::now();
                while self.ups.front().is_some_and(|&t| now.duration_since(t).as_secs_f64() > 1.0) { self.ups.pop_front(); }
                let fps = if self.ups.len() > 1 { (self.ups.len() - 1) as f64 / now.duration_since(self.ups[0]).as_secs_f64().max(1e-3) } else { 0.0 };
                if self.hud.as_ref().is_none_or(|h| h.0 != sh) { self.hud = Hud::new(sh).map(|h| (sh, h)); }
                if let Some((_, hud)) = &mut self.hud {
                    hud.draw(&mut buf, sw, sh, &[("FPS", format!("{fps:.1}")), ("INSTRUCTIONS", commas(self.running)), ("PER SECOND", commas(second))]);
                }
            }
        }
        buf.present().unwrap();
        if let Some((n, d)) = self.shown.due.take() {
            let behind = Instant::now().saturating_duration_since(d).as_secs_f64() * 1e3;
            self.shown.behind_ms.push(behind);
            if behind > self.shown.worst_behind_ms { self.shown.worst_behind_ms = behind; self.shown.worst_behind_at = n; }
        }
        self.shown.worst_draw_ms = self.shown.worst_draw_ms.max(t.elapsed().as_secs_f64() * 1e3);
        // the next frame, if it's in, scaled now so that at its time it is only copied
        let next = self.up.map_or(self.first, |u| u + 1);
        if let Some(p) = self.film.get(next) {
            let mut ready = vec![0u32; sw * sh];
            scale(&p, sw, sh, &mut ready);
            self.ahead = Some((next, sw, sh, ready));
        }
    }
}

/// The HUD in the lower right: the frame rate, and the running count of CARDIAC instructions and the last second's,
/// in Menlo at the screen's own size in a translucent box, like a game's performance overlay. Drawn by the host
/// (presentation, like the scaling); the counts are the jobs' own (see live::shares).
struct Hud { font: fontdue::Font, px: f32, glyphs: std::collections::HashMap<char, (fontdue::Metrics, Vec<u8>)> }

impl Hud {
    fn new(sh: usize) -> Option<Hud> {
        let bytes = std::fs::read("/System/Library/Fonts/Menlo.ttc").ok()?;
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings { collection_index: 0, ..Default::default() }).ok()?;
        Some(Hud { font, px: (sh as f32 / 58.0).max(12.0), glyphs: Default::default() })
    }

    fn glyph(&mut self, c: char) -> &(fontdue::Metrics, Vec<u8>) {
        let (font, px) = (&self.font, self.px);
        self.glyphs.entry(c).or_insert_with(|| font.rasterize(c, px))
    }

    fn width(&mut self, text: &str) -> usize { text.chars().map(|c| self.glyph(c).0.advance_width.round() as usize).sum() }

    /// Text with its baseline at (x, y), in colour, laid over by its coverage.
    fn text(&mut self, buf: &mut [u32], sw: usize, sh: usize, text: &str, x: usize, y: usize, colour: u32) {
        let mut pen = x as i32;
        for c in text.chars() {
            let (m, cover) = self.glyph(c).clone();
            let (gx, gy) = (pen + m.xmin, y as i32 - m.height as i32 - m.ymin);
            for row in 0..m.height {
                for col in 0..m.width {
                    let (px, py) = (gx + col as i32, gy + row as i32);
                    if px < 0 || py < 0 || px as usize >= sw || py as usize >= sh { continue }
                    let a = cover[row * m.width + col] as u32;
                    if a > 0 { let i = py as usize * sw + px as usize; buf[i] = blend(buf[i], colour, a); }
                }
            }
            pen += m.advance_width.round() as i32;
        }
    }

    fn draw(&mut self, buf: &mut [u32], sw: usize, sh: usize, rows: &[(&str, String)]) {
        let line = (self.px * 1.4) as usize;
        let pad = (self.px * 0.7) as usize;
        let label = rows.iter().map(|(k, _)| self.width(k)).max().unwrap_or(0);
        let value = rows.iter().map(|(_, v)| self.width(v)).max().unwrap_or(0);
        let (w, h) = (pad * 2 + label + 2 * pad + value, pad * 2 + line * rows.len());
        let (x0, y0) = (sw.saturating_sub(w + pad * 2), sh.saturating_sub(h + pad * 2));
        for y in y0..(y0 + h).min(sh) { for x in x0..(x0 + w).min(sw) { let i = y * sw + x; buf[i] = blend(buf[i], 0, 150); } }
        for (k, (l, v)) in rows.iter().enumerate() {
            let base = y0 + pad + line * k + (self.px * 1.05) as usize;
            self.text(buf, sw, sh, l, x0 + pad, base, 0xa0dca0);
            let vw = self.width(v);
            self.text(buf, sw, sh, v, x0 + w - pad - vw, base, 0xf0f0f0);
        }
    }
}

fn blend(under: u32, over: u32, a: u32) -> u32 {
    let ch = |shift: u32| { let (u, o) = ((under >> shift) & 255, (over >> shift) & 255); (u * (255 - a) + o * a) / 255 << shift };
    ch(16) | ch(8) | ch(0)
}

fn commas(n: u64) -> String {
    let d = n.to_string();
    d.chars().enumerate().flat_map(|(i, c)| (i > 0 && (d.len() - i) % 3 == 0).then_some(',').into_iter().chain([c])).collect()
}

/// The picture on a screen of sw x sh: 2:1 inside it, letterboxed; each screen pixel takes the picture's pixel it
/// falls in.
fn scale(p: &Picture, sw: usize, sh: usize, buf: &mut [u32]) {
    buf.fill(0);
    let (tw, th) = if sw >= 2 * sh { (2 * sh, sh) } else { (sw, sw / 2) };
    let (x0, y0) = ((sw - tw) / 2, (sh - th) / 2);
    let xs: Vec<usize> = (0..tw).map(|x| x * p.w / tw * 3).collect();
    let rgb = p.rgb();
    let mut last_src = usize::MAX;
    for y in 0..th {
        let sy = y * p.h / th;
        let row = (y0 + y) * sw + x0;
        if sy == last_src {
            buf.copy_within(row - sw..row - sw + tw, row);
            continue;
        }
        last_src = sy;
        let src = &rgb[sy * p.w * 3..(sy + 1) * p.w * 3];
        for (x, &sx) in xs.iter().enumerate() {
            buf[row + x] = (src[sx] as u32) << 16 | (src[sx + 1] as u32) << 8 | src[sx + 2] as u32;
        }
    }
}

impl App {
    fn made(&mut self, window: Window) {
        let window = Rc::new(window);
        window.set_cursor_visible(self.windowed);
        let context = Context::new(window.clone()).unwrap();
        self.surface = Some(Surface::new(&context, window.clone()).unwrap());
        self.window = Some(window);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() { return }
        let mut attrs = Window::default_attributes().with_title("Card Attack");
        attrs = if self.windowed { attrs.with_inner_size(winit::dpi::LogicalSize::new(1152.0, 576.0)) } else { attrs.with_fullscreen(Some(Fullscreen::Borderless(None))) };
        #[cfg(target_os = "macos")]
        if !self.windowed && !self.space {
            // macOS: "simple" fullscreen, the window over the whole screen in this Space; in a fullscreen Space of
            // its own the window's loop was seen to stall up to 70 ms at a time
            use winit::platform::macos::WindowExtMacOS;
            attrs = Window::default_attributes().with_title("Card Attack");
            let window = el.create_window(attrs).unwrap();
            window.set_simple_fullscreen(true);
            window.focus_window();
            return self.made(window);
        }
        let window = el.create_window(attrs).unwrap();
        self.made(window);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested | WindowEvent::KeyboardInput { event: KeyEvent { logical_key: Key::Named(NamedKey::Escape), state: ElementState::Pressed, .. }, .. } => {
                self.shown.quit_at = self.up; el.exit();
            }
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(_) => if let Some(w) = &self.window { w.request_redraw() },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        match self.due() {
            None => el.set_control_flow(ControlFlow::WaitUntil(self.start + Duration::from_secs_f64(self.first as f64 / FPS))),
            Some(n) if n >= self.end => { self.shown.frames = self.end; el.exit() }
            Some(n) => {
                if self.up != Some(n) {
                    // every frame passed since the last wake-up that wasn't ready at its time is late
                    for k in self.up.map_or(0, |u| u + 1)..=n { if !self.film.filed(k) { self.shown.late.push(k) } }
                    if let Some(p) = self.film.get(n) { self.pic = Some((n, p)) }
                    // the frames before this one have been shown: let them go (the picture up stays with us)
                    for k in self.up.unwrap_or(n)..n { self.film.release(k) }
                    self.up = Some(n);
                    self.ups.push_back(Instant::now());
                    self.shown.due = Some((n, self.start + Duration::from_secs_f64(n as f64 / FPS)));
                    self.shown.woke_ms.push(Instant::now().saturating_duration_since(self.shown.due.unwrap().1).as_secs_f64() * 1e3);
                    if let Some(w) = &self.window { w.request_redraw() }
                }
                let next = self.start + Duration::from_secs_f64((n + 1) as f64 / FPS);
                el.set_control_flow(ControlFlow::WaitUntil(next));
            }
        }
    }
}

/// Tell macOS the player is a user-interactive, latency-critical activity that keeps the display awake: otherwise
/// a window not in front is put in App Nap and its workers slowed to a crawl.
fn stay_awake() {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::{NSActivityOptions, NSProcessInfo, NSString};
        let token = NSProcessInfo::processInfo().beginActivityWithOptions_reason(NSActivityOptions::UserInteractive | NSActivityOptions::IdleDisplaySleepDisabled, &NSString::from_str("Card Attack plays live"));
        std::mem::forget(token);   // for the whole run
    }
}

/// Show the film on its clock (frame 0 at `start`), fullscreen (or in a window), frames `first` to `end` or until
/// Escape. Must run on the main thread.
pub fn show(film: Arc<Film>, first: usize, end: usize, start: Instant, windowed: bool, space: bool) -> Shown {
    stay_awake();
    let el = EventLoop::new().unwrap();
    let end = end.min(film.len());
    let mut app = App { first, end, film, start, windowed, space, window: None, surface: None, up: first.checked_sub(1), pic: None, ahead: None, shown: Shown::default(), counted: 0, running: 0, ups: Default::default(), hud: None };
    el.run_app(&mut app).unwrap();
    app.shown
}

#[cfg(test)]
mod tests {
    /// The HUD drawn over grey at a MacBook Pro's screen size, written where HUD_PPM says (to look at).
    #[test]
    fn hud() {
        let (sw, sh) = (3024, 1964);
        let mut buf = vec![0x507090u32; sw * sh];
        let Some(mut hud) = super::Hud::new(sh) else { return };
        hud.draw(&mut buf, sw, sh, &[("FPS", "12.0".into()), ("INSTRUCTIONS", super::commas(331482907114)), ("PER SECOND", super::commas(6442018233))]);
        assert!(buf.iter().any(|&p| p != 0x507090));
        if let Some(path) = std::env::var_os("HUD_PPM") {
            let mut out = format!("P6 {sw} {sh} 255\n").into_bytes();
            for p in buf { out.extend([(p >> 16) as u8, (p >> 8) as u8, p as u8]); }
            std::fs::write(path, out).unwrap();
        }
    }
}
