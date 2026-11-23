// The UI kit's text pages: screens of text in a PC's bitmap font, as data. Shared by the host (which works every
// frame out on CARDIAC: fx/text.rs) and the references (which draw them in ordinary Rust: text.rs). Nothing here is
// worked out: a page is lines of runs of text, each run its ink and paper, in a face of the atlas (a bitmap face:
// atlas.txt's faces 10 and on, every glyph its whole cell). The machines place the characters (the advances summed
// along a line, the line step down the page, a line centred on its x if it says so), show as many as have arrived,
// put the cursor after the last, and lay each glyph's ink and paper on. The page is drawn at its own pixels, then
// shown at `sx` x `sy` times at x, y of the 1536x768 frame (a blit). The texts are the BBS's and the VGA's.

/// A colour: red, green, blue.
pub type Rgb = [u8; 3];

/// The sixteen colours of the PC's text modes (CGA, EGA and VGA alike).
pub const BLACK: Rgb = [0, 0, 0];
pub const GREY: Rgb = [170, 170, 170];
pub const WHITE: Rgb = [255, 255, 255];
pub const YELLOW: Rgb = [255, 255, 85];
pub const CYAN: Rgb = [85, 255, 255];
pub const GREEN: Rgb = [85, 255, 85];
pub const BLUE: Rgb = [0, 0, 170];
pub const LIGHT_BLUE: Rgb = [85, 85, 255];
pub const LIGHT_MAGENTA: Rgb = [255, 85, 255];

/// A run of text: what it says, its ink and its paper (None: none, the page shows through between the strokes).
#[derive(Clone, Copy, Debug)]
pub struct Run { pub text: &'static str, pub ink: Rgb, pub paper: Option<Rgb> }

/// A line: its runs, one after another, starting at x (centred: centred on x).
#[derive(Clone, Copy, Debug)]
pub struct Line { pub x: i16, pub centred: bool, pub runs: &'static [Run] }

/// A page: its size in its own pixels, its face (atlas.txt), the advance and the line step, the first line's top,
/// its lines; the cursor's colour (atlas code 1, after the last character shown); where it is shown on the
/// 1536x768 frame and how many times over.
pub struct Page { pub name: &'static str, pub w: usize, pub h: usize, pub face: usize, pub advance: i16, pub step: i16, pub top: i16,
    pub lines: &'static [Line], pub cursor: Rgb, pub sx: usize, pub sy: usize, pub x: usize, pub y: usize }

impl Page {
    /// How many characters the page has.
    pub fn chars(&self) -> usize { self.lines.iter().flat_map(|l| l.runs).map(|r| r.text.chars().count()).sum() }
}

/// The faces (atlas.txt): the IBM CGA's 8x8, the IBM VGA's 8x16.
pub const CGA8: usize = 10;
pub const VGA16: usize = 11;
/// The cursor's code in a bitmap face.
pub const CURSOR: u32 = 1;

const fn run(text: &'static str, ink: Rgb) -> Run { Run { text, ink, paper: Some(BLACK) } }
const fn line(runs: &'static [Run]) -> Line { Line { x: 0, centred: false, runs } }
const BLANK: Line = line(&[]);
const fn ink(text: &'static str, ink: Rgb) -> Run { Run { text, ink, paper: None } }

/// CompuServe, 1990: a forum message with the mode 13h code, in 40-column text (each pixel 4 x 8 on the frame).
pub const COMPUSERVE: Page = Page { name: "compuserve", w: 320, h: 96, face: CGA8, advance: 8, step: 8, top: 0, cursor: GREY, sx: 4, sy: 8, x: 128, y: 0, lines: &[
    line(&[run(" CompuServe                       IBMPRO", GREY)]),
    line(&[run(" #: 31337 S5/Assembly", GREY)]),
    line(&[run(" Sb: check out this code", GREY)]),
    line(&[run(" Fm: Strange Query 70000,1", GREY)]),
    line(&[run(" To: All", GREY)]),
    BLANK,
    line(&[run("   mov  ax,13h    ; 256 colors", WHITE)]),
    line(&[run("   int  10h       ; set the mode", WHITE)]),
    line(&[run("   mov  ax,0A000h", WHITE)]),
    line(&[run("   mov  es,ax     ; es -> screen", WHITE)]),
    BLANK,
    line(&[run(" Press <CR> for more !", GREY)]),
] };

/// trn reading comp.sys.ibm.pc.demos, 30 May 1993: a real reply recommending Future Crew and Witan's Facts of Life
/// (the archive's text, the address masked), 52 columns (each pixel 3 x 6 on the frame).
pub const USENET: Page = Page { name: "usenet", w: 416, h: 96, face: CGA8, advance: 8, step: 8, top: 0, cursor: GREY, sx: 3, sy: 6, x: 144, y: 96, lines: &[
    line(&[run("Article 1281 (3 more) in comp.sys.ibm.pc.demos:", YELLOW)]),
    line(&[run("From: ", CYAN), run("BG*****@MUSICB.MCGILL.CA", WHITE)]),
    line(&[run("Subject: ", CYAN), run("RE: What are top 5 DEMOS? anyone?", WHITE)]),
    line(&[run("Date: ", CYAN), run("Sun, 30 May 1993 06:06:09 GMT", WHITE)]),
    BLANK,
    line(&[run("I can heartily recommend Future Crew's UNREAL", GREY)]),
    line(&[run("and PANIC demos.", GREY)]),
    line(&[run("They are tops along with Witan's FACTS OF LIFE demo.", GREY)]),
    BLANK,
    line(&[run("Tony", GREY)]),
    BLANK,
    line(&[run("End of article 1281 -- what next? [npq] ", GREEN)]),
] };

/// The StrangeQuery BBS's message reader, 1987: a post to the demo coders' area, "check out this code!", in 80-column
/// ANSI text, 16 colours (each pixel 2 x 4 on the frame: 1280x768, centred). The bar's "³" is bbs.py's, drawn as the
/// font draws that character. Beside the message, a crude ASCII cube, the normal vector off its right face.
pub const BBS: Page = Page { name: "bbs", w: 640, h: 192, face: CGA8, advance: 8, step: 8, top: 0, cursor: GREY, sx: 2, sy: 4, x: 128, y: 0, lines: &[
    line(&[Run { text: " StrangeQuery BBS ³ Node 1 ³ 2400 baud                        Time left: 23 min ", ink: WHITE, paper: Some(BLUE) }]),
    BLANK,
    line(&[run("╔══════════════════════════════════════════════════════════════════════════════╗", LIGHT_BLUE)]),
    line(&[run("║", LIGHT_BLUE), run(" Msg #: ", CYAN), run("1987 of 2026", YELLOW), run("        Area: ", CYAN), run("[3] Demo Coders", YELLOW), run("                             ", GREY), run("║", LIGHT_BLUE)]),
    line(&[run("║", LIGHT_BLUE), run(" From : ", CYAN), run("STRANGE QUERY", WHITE), run("           To  : ", CYAN), run("All", WHITE), run("                                     ", GREY), run("║", LIGHT_BLUE)]),
    line(&[run("║", LIGHT_BLUE), run(" Subj : ", CYAN), run("check out this code!", LIGHT_MAGENTA), run("                                                  ", GREY), run("║", LIGHT_BLUE)]),
    line(&[run("║", LIGHT_BLUE), run(" Date : ", CYAN), run("11-21-87  23:58", GREY), run("                                                       ", GREY), run("║", LIGHT_BLUE)]),
    line(&[run("╚══════════════════════════════════════════════════════════════════════════════╝", LIGHT_BLUE)]),
    BLANK,
    line(&[run(" yo all,", GREY)]),
    BLANK,
    line(&[run(" here is some code to do gouraud shading! no math coprocessor, no", GREY)]),
    line(&[run(" nothin. it's in file area 3 as GOURAUD.ZIP.", GREY), run("       +------+", CYAN)]),
    Line { x: 400, centred: false, runs: &[run(" /      /|", CYAN)] },
    line(&[run(" greets to everyone at the party", GREY), run("                  +------+ |", CYAN)]),
    Line { x: 400, centred: false, runs: &[run("|      | |", CYAN), run("-- normal vector", YELLOW)] },
    line(&[run(" -- sq", GREY), run("                                            |      | +", CYAN)]),
    Line { x: 400, centred: false, runs: &[run("|      |/", CYAN)] },
    Line { x: 400, centred: false, runs: &[run("+------+", CYAN)] },
    line(&[run(" [", GREEN), run("R", WHITE), run("]eply, [", GREEN), run("N", WHITE), run("]ext, [", GREEN), run("Q", WHITE), run("]uit? ", GREEN)]),
] };

/// The VGA's brag, in its own 8x16 font, white on a black box, each pixel 4 x 4 on the frame (a layer over the
/// plasma: the box hides it).
pub const VGA_BRAG: Page = Page { name: "vga_brag", w: 176, h: 48, face: VGA16, advance: 8, step: 18, top: 6, cursor: GREY, sx: 4, sy: 4, x: 64, y: 528, lines: &[
    Line { x: 88, centred: true, runs: &[Run { text: "320 x 200", ink: WHITE, paper: None }] },
    Line { x: 88, centred: true, runs: &[Run { text: "256 COLORS", ink: WHITE, paper: None }] },
] };

/// Mode X's setup, the assembly that sets 320x240 (the VGA unchained, its clock and CRT registers), in the VGA's
/// own 8x16 font, white letters with no paper (the ray tracer shows between them), each pixel 2 x 2 on the frame.
/// The drop shadow is the letters again, 3 pixels down and right, in black (fx/modex.rs).
pub const MODEX: Page = Page { name: "modex", w: 384, h: 320, face: VGA16, advance: 8, step: 16, top: 0, cursor: GREY, sx: 2, sy: 2, x: 300, y: 64, lines: &[
    line(&[ink("; MODE X: 320x240, 256 colors", WHITE)]),
    line(&[ink("        mov  ax,13h", WHITE)]),
    line(&[ink("        int  10h          ; mode 13h first", WHITE)]),
    line(&[ink("        mov  dx,3C4h      ; sequencer", WHITE)]),
    line(&[ink("        mov  ax,0604h     ; chain-4 off", WHITE)]),
    line(&[ink("        out  dx,ax", WHITE)]),
    line(&[ink("        mov  dx,3C2h", WHITE)]),
    line(&[ink("        mov  al,0E3h      ; 25 MHz, 480 lines", WHITE)]),
    line(&[ink("        out  dx,al", WHITE)]),
    line(&[ink("        mov  dx,3D4h      ; CRT controller", WHITE)]),
    line(&[ink("        mov  si,offset crtc", WHITE)]),
    line(&[ink("        mov  cx,11", WHITE)]),
    line(&[ink("next:   lodsw", WHITE)]),
    line(&[ink("        out  dx,ax", WHITE)]),
    line(&[ink("        loop next", WHITE)]),
    line(&[ink("        ret", WHITE)]),
    BLANK,
    line(&[ink("crtc    dw 2C11h,0D06h,3E07h,4109h", WHITE)]),
    line(&[ink("        dw 0EA10h,0AC11h,0DF12h,0014h", WHITE)]),
    line(&[ink("        dw 0E715h,0616h,0E317h", WHITE)]),
] };

/// Mode X's sign (in place of the listing): VGA over MODE X in the VGA's 8x16 font, filling the
/// sky behind the mirror ball: each page pixel 6 x 4 on the 320x240 picture at x, y, white where it has ink and black
/// within 3 pixels of one (the outline), where the ball doesn't cover it (modex/reference.rs).
pub const MODEX_SIGN: Page = Page { name: "modex_sign", w: 49, h: 22, face: VGA16, advance: 8, step: 12, top: -2, cursor: GREY, sx: 6, sy: 4, x: 13, y: 6, lines: &[
    Line { x: 24, centred: true, runs: &[Run { text: "VGA", ink: WHITE, paper: None }] },
    Line { x: 0, centred: false, runs: &[Run { text: "MODE X", ink: WHITE, paper: None }] },
] };

/// The monitor's DEGAUSS sign over Mode X (fx/modex.rs): the word in the VGA's own 8x16 font, bright green, drawn
/// twice a pixel apart (bold), each pixel 6 x 6 on the frame, beside the warning sign (modex/data/warning.txt) on a
/// dark box.
pub const DEGAUSS: Page = Page { name: "degauss", w: 57, h: 16, face: VGA16, advance: 8, step: 0, top: 0, cursor: GREY, sx: 6, sy: 6, x: 675, y: 102, lines: &[
    Line { x: 0, centred: false, runs: &[Run { text: "DEGAUSS", ink: GREEN, paper: None }] },
    Line { x: 1, centred: false, runs: &[Run { text: "DEGAUSS", ink: GREEN, paper: None }] },
] };

/// The CGA's brag, in its own 8x8 font, on a black box: laid on the turn's 320x200 CGA picture rather than the
/// 1536x768 frame, each pixel 2 x 2 there, the box's top left at x, y (the design's text doubled about the middle of
/// the picture and moved to the lower left corner, 8 pixels in). The ink is the picture's colour 3, the box colour 0.
pub const CGA_BRAG: Page = Page { name: "cga_brag", w: 79, h: 25, face: CGA8, advance: 8, step: 12, top: 3, cursor: GREY, sx: 2, sy: 2, x: 8, y: 143, lines: &[
    Line { x: 4, centred: false, runs: &[Run { text: "320 x 200", ink: WHITE, paper: None }] },
    Line { x: 8, centred: false, runs: &[Run { text: "4 COLORS", ink: WHITE, paper: None }] },
] };

/// The accelerated drive's calls (accel.py, wording 2), white in the VGA's 8x16 at 2 x 2, 44 pixels a line, over
/// the drive as it goes 3D accelerated.
pub const ACCEL: Page = Page { name: "accel", w: 368, h: 170, face: VGA16, advance: 8, step: 22, top: 0, cursor: GREY, sx: 2, sy: 2, x: 120, y: 150, lines: &[
    line(&[ink("acCreateContext(&ctx, AC_DEVICE_0);", WHITE)]),
    line(&[ink("acSetDisplayMode(ctx, 640, 480, 16, 60);", WHITE)]),
    line(&[ink("acEnable(ctx, AC_DEPTH_TEST | AC_DITHER);", WHITE)]),
    line(&[ink("acTexParameter(ctx, AC_MIN_FILTER, AC_LINEAR);", WHITE)]),
    line(&[ink("acTexParameter(ctx, AC_MAG_FILTER, AC_LINEAR);", WHITE)]),
    line(&[ink("acFogColor(ctx, 0x2C1A3E);", WHITE)]),
    line(&[ink("acFogMode(ctx, AC_FOG_TABLE);", WHITE)]),
    line(&[ink("acSwapBuffers(ctx);", WHITE)]),
] };

pub const PAGES: [&Page; 8] = [&COMPUSERVE, &USENET, &VGA_BRAG, &MODEX, &BBS, &CGA_BRAG, &DEGAUSS, &ACCEL];

pub fn page(name: &str) -> &'static Page { PAGES.iter().find(|p| p.name == name).copied().unwrap_or_else(|| panic!("no page {}", name)) }
