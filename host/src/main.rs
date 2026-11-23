//! card-attack SEGMENT REFERENCE_BINARY [MODE]  render one segment on CARDIAC and check it against its reference
//! card-attack all REFERENCE_DIR                 every segment in the order the cut needs them (REFERENCE_DIR/<segment>_ref)
//! card-attack cut REFERENCE [CUT_FILE]          the cut (cut/cut.txt, or another cut file under segments/)
//! card-attack play [...]                        the player (see play/mod.rs)
//! card-attack render OUT.mp4 [--workers N]      the film as a video file, computed as the player does (play/render.rs)
//! --native first (or CARD_ATTACK_NATIVE=1): every machine written in Rust also runs natively on the same cards, and
//! must punch the same ones. CARD_ATTACK_NATIVE=only: natively in place of CARDIAC. CARD_ATTACK_SHADOW=1: each one-phase
//! Rust machine's hand-written original runs beside it on the same cards.
use card_attack::fx;
use card_attack::native::{self, Native, Settings};

fn render(effect: &str, refbin: &str, mode: Option<&str>) {
    let demo = card_attack::demo_dir();
    match effect {
        "roto4" => fx::roto4::render(&demo, refbin),
        "plasma" => fx::plasma::render(&demo, refbin),
        "ray" => fx::ray::render(&demo, refbin),
        "tumble" => fx::tumble::render(&demo, refbin),
        "openers" => fx::openers::render(&demo, refbin),
        "polys" => fx::polys::render(&demo, refbin),
        "deep" => fx::deep::render(&demo, refbin),
        "notify" | "miner" | "videoopts" | "accelbrag" => fx::ui::render(&demo, refbin, effect),
        "vga" => fx::vga::render(&demo, refbin),
        "modex" => fx::modex::render(&demo, refbin),
        "accel" => fx::accel::render(&demo, refbin),
        "accel_dump" => fx::accel_hd::dump(&demo, refbin.parse().unwrap()),
        "partychat" => fx::partychat::render(&demo, refbin),
        "mac" => fx::mac::render(&demo, refbin),
        "dierubik" => fx::dierubik::render(&demo, refbin),
        "bbs" => fx::bbs::render(&demo, refbin),
        "bbsroto" => fx::bbs::render_roto(&demo, refbin),
        "turn" => fx::turn::render(&demo, refbin),
        "mesh" => fx::mesh::render(&demo, refbin),
        "train" => fx::train::render(&demo, refbin),
        "night" => fx::night::render(&demo, refbin),
        "stadium" => fx::stadium::render(&demo, refbin),
        "rooms" => fx::rooms::render(&demo, refbin),
        "rotodg" => fx::rotodg::render(&demo, refbin),
        "sqend" => fx::sqend::render(&demo, refbin),
        "qr" => fx::qr::render(&demo, refbin),
        "cut" => fx::cut::render(&demo, refbin, mode),
        _ => { eprintln!("unknown effect {effect}"); std::process::exit(2) }
    }
}

fn main() {
    let mut a: Vec<String> = std::env::args().collect();
    let flag = a.get(1).is_some_and(|s| s == "--native");
    if flag { a.remove(1); }
    let mode = match std::env::var("CARD_ATTACK_NATIVE").as_deref() {
        Ok("only") => Native::Only,
        Ok("all") => Native::Report,
        Ok("" | "0") if !flag => Native::Off,
        Ok(_) => Native::Check,
        Err(_) if flag => Native::Check,
        Err(_) => Native::Off,
    };
    native::set(Settings { shadow: std::env::var("CARD_ATTACK_SHADOW").is_ok(), native: mode, find: card_attack::machines::find });
    if a.get(1).is_some_and(|s| s == "play") { card_attack::play::main(&a[2..]); return; }
    if a.get(1).is_some_and(|s| s == "render") { card_attack::play::render::main(&a[2..]); return; }
    if a.len() < 3 { eprintln!("usage: card-attack [--native] EFFECT REFERENCE [MODE] | card-attack [--native] all REFERENCE_DIR"); std::process::exit(2); }
    if a[1] == "all" {
        let r = |e: &str| format!("{}/{e}_ref", a[2]);
        // each segment after the ones whose videos it reads, then the cut
        for e in ["openers", "mac", "turn", "roto4", "tumble", "dierubik", "plasma", "vga", "polys"] { render(e, &r(e), None); }
        for e in ["bbs", "bbsroto"] { render(e, &r("bbs"), None); }
        for e in ["modex", "accel", "deep"] { render(e, &r(e), None); }
        for e in ["notify", "miner", "videoopts", "accelbrag"] { render(e, &r("ui"), None); }
        render("partychat", &r("partychat"), None);
        render("train", &r("train"), None);
        render("night", &r("night"), None);
        render("rooms", &r("rooms"), None);
        render("rotodg", &r("rotodg"), None);
        render("qr", &r("qr"), None);
        render("sqend", &r("sqend"), None);
        render("cut", &r("cut"), None);
    } else { render(&a[1], &a[2], a.get(3).map(|s| s.as_str())); }
    // CARD_ATTACK_NATIVE=all: every machine whose cards differed, whichever segments ran them
    let differed = native::all_differed();
    if !differed.is_empty() {
        eprintln!("these machines' cards differ natively from CARDIAC: {}", differed.join(", "));
        std::process::exit(1);
    }
}
