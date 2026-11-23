//! Machines: a Rust machine's tapes as `cargo cardiac build` wrote them, one a phase (a phase that ends with JMP 00
//! hands back to the loader, which reads the next phase's tape over the old code), or a hand-written machine
//! assembled on the empty data block; and which hand-written machines the Rust ones replace one for one.
use crate::asm::{assemble, tape};
use std::path::Path;

/// One machine: its tapes, one per phase, each followed by that phase's data cards when it runs.
pub struct Machine { pub name: String, pub tapes: Vec<Vec<i32>>,
    /// For a machine compiled in phases: how many data cards each phase but the last reads, so one deck can be dealt
    /// out between the phases' tapes. Empty for hand-written machines (one phase each) and for a compiled one whose
    /// host deals each phase its own deck.
    pub split: Vec<usize>,
    /// With shadow runs on: the hand-written machine this one replaces, run beside it on the same cards and
    /// compared (for finding where a port goes wrong).
    pub shadow: Option<Box<Machine>>,
    /// With native runs on: the machine's source built as a native function, run beside it on the same cards.
    pub native: Option<fn()>,
    /// The table the host punched into the program, for `load_resource()` in a native run.
    pub table: Vec<i32> }

impl Machine {
    /// A machine of these tapes, nothing else set.
    pub fn new(name: &str, tapes: Vec<Vec<i32>>) -> Self { Machine { name: name.into(), tapes, split: vec![], shadow: None, native: None, table: vec![] } }
    /// Give it the native build of the Rust machine NAME, if native runs are on, carrying `table`.
    pub fn native(mut self, name: &str, table: &[i32]) -> Self {
        self.native = crate::native::find(name);
        self.table = table.to_vec();
        self
    }
}

pub fn read(demo: &Path, file: &str) -> String {
    std::fs::read_to_string(demo.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

pub fn build(demo: &Path, data_file: &str, phases: &[(&str, &str)]) -> Machine {
    // a machine written in Rust that replaces this one, phase for phase, cards dealt the same way
    if let Some(name) = replacements(demo).get(phases[0].0) {
        let m = rust(demo, name);
        assert_eq!(m.tapes.len(), phases.len(), "{name} replaces {} but has another number of phases", phases[0].0);
        return m;
    }
    build_hand(demo, data_file, phases)
}

/// A hand-written machine of one or more phases (see build).
pub fn build_hand(demo: &Path, data_file: &str, phases: &[(&str, &str)]) -> Machine {
    let data = read(demo, data_file);
    let data_end = assemble(3, &data).unwrap().end;
    let mut tapes = vec![];
    for (i, (file, start)) in phases.iter().enumerate() {
        let src = format!("{}\n{}", data, read(demo, file));
        let p = assemble(3, &src).unwrap_or_else(|e| panic!("{file}: {e}"));
        let cells: Vec<_> = if i == 0 { p.cells.clone() } else { p.cells.iter().copied().filter(|&(a, _)| a >= data_end).collect() };
        tapes.push(tape(p.label(start), &cells));
    }
    Machine::new(phases[0].0, tapes)
}

/// A one-phase machine on the empty data block, started at `start:` (the usual case). If a machine written in Rust
/// says it replaces this file (a line `// cardiac: replaces FILE` in a machine's source, nothing after the name), that
/// one is used instead: same cards in, same cards out.
pub fn one(demo: &Path, file: &str) -> Machine {
    match replacements(demo).get(file) { Some(name) => rust(demo, name), None => hand(demo, file) }
}

/// The hand-written machine itself.
pub fn hand(demo: &Path, file: &str) -> Machine { build_hand(demo, "shared/none_data.asm", &[(file, "start")]) }

/// Every machine written in Rust: NAME -> its folder (PART/machines), from the demo folder's */machines/NAME.rs.
pub fn machines(demo: &Path) -> &'static std::collections::HashMap<String, String> {
    static MAP: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        for seg in std::fs::read_dir(demo).unwrap().flatten() {
            let dir = seg.path().join("machines");
            let Ok(files) = std::fs::read_dir(&dir) else { continue };
            for e in files.flatten() {
                let p = e.path();
                if p.extension().map_or(true, |x| x != "rs") { continue; }
                let name = p.file_stem().unwrap().to_string_lossy().to_string();
                let folder = format!("{}/machines", seg.file_name().to_string_lossy());
                if let Some(other) = m.insert(name.clone(), folder.clone()) { panic!("two machines called {name}: {other} and {folder}"); }
            }
        }
        m
    })
}

/// The Rust machine NAME's file with this ending (".rs", ".asm", "_data.asm" ...), if there is a NAME.
pub fn machine_file(demo: &Path, name: &str, ending: &str) -> Option<String> {
    machines(demo).get(name).map(|dir| format!("{dir}/{name}{ending}"))
}

/// Which hand-written machines the Rust ones replace one for one: file -> NAME.
pub fn replacements(demo: &Path) -> &'static std::collections::HashMap<String, String> {
    static MAP: std::sync::OnceLock<std::collections::HashMap<String, String>> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::HashMap::new();
        for name in machines(demo).keys() {
            let src = read(demo, &machine_file(demo, name, ".rs").unwrap());
            for l in src.lines() { if let Some(f) = l.strip_prefix("// cardiac: replaces ") {
                if !f.contains(' ') { m.insert(f.trim().to_string(), name.clone()); } } }
        }
        m
    })
}

/// A machine with cards patched into its program: `extra` (cell, value) pairs added after the assembled cells,
/// as a host does for a row of text or a table carried in the program deck.
pub fn patched(name: &str, src: &str, start: &str, extra: impl Fn(&crate::asm::Program) -> Vec<(i32, i32)>) -> Machine {
    let p = assemble(3, src).unwrap();
    let mut cells = p.cells.clone(); cells.extend(extra(&p));
    Machine::new(name, vec![tape(p.label(start), &cells)])
}

/// The tapes `cargo cardiac build` wrote for the Rust machine NAME: NAME.cqb, and NAME.2.cqb ... for a machine in
/// phases, with its resources' slots (NAME.cqb.meta).
fn tapes(demo: &Path, name: &str) -> crate::tape::Tape {
    let file = machine_file(demo, name, ".cqb").unwrap_or_else(|| panic!("no machine called {name}"));
    crate::tape::Tape::load(&demo.join(&file)).unwrap_or_else(|e| panic!("{e} (build the machines with cargo cardiac build)"))
}

/// A Rust machine of one phase as it was compiled: the cells its tape loads, and its labels, `start` and
/// `resource_NAME` for each resource's slot, where a host punches a table in (`carrying`). A hand-written machine,
/// assembled, answers the same (`Compiled::of`).
pub struct Compiled { pub cells: Vec<(i32, i32)>, labels: std::collections::HashMap<String, i32> }

impl Compiled {
    pub fn label(&self, label: &str) -> i32 { *self.labels.get(label).unwrap_or_else(|| panic!("no label {label}")) }
    pub fn of(p: &crate::asm::Program) -> Self { Compiled { cells: p.cells.clone(), labels: p.labels.clone() } }
}

/// The Rust machine NAME as it was compiled (it has one phase).
pub fn compiled(demo: &Path, name: &str) -> Compiled {
    let t = tapes(demo, name);
    assert_eq!(t.phases.len(), 1, "{name}: a machine in phases has no one program to punch a table into");
    let l = crate::listing::from_deck(&t.phases[0]).unwrap_or_else(|e| panic!("{name}.cqb: {e}"));
    let mut labels: std::collections::HashMap<String, i32> = t.slots.iter().map(|s| (format!("resource_{}", s.name), s.at)).collect();
    labels.insert("start".into(), l.start.unwrap_or_else(|| panic!("{name}.cqb doesn't start a program")) as i32);
    Compiled { cells: l.cells.iter().map(|&(a, v)| (a as i32, v)).collect(), labels }
}

/// The Rust machine NAME carrying `table` in its program at its resource's slot, punched in after its cells, as a
/// host does for a table its load_resource() reads; with its native build, carrying the same table.
pub fn carrying(demo: &Path, name: &str, resource: &str, table: &[i32]) -> Machine {
    static COMPILED: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<Compiled>>>> = std::sync::OnceLock::new();
    let c = COMPILED.get_or_init(Default::default).lock().unwrap().entry(name.to_string()).or_insert_with(|| std::sync::Arc::new(compiled(demo, name))).clone();
    let at = c.label(&format!("resource_{resource}"));
    let mut cells = c.cells.clone(); cells.extend(table.iter().enumerate().map(|(i, &v)| (at + i as i32, v)));
    Machine::new(&format!("{name} (Rust)"), vec![tape(c.label("start"), &cells)]).native(name, table)
}

/// A machine written in Rust (PART/machines/NAME.rs), compiled by `cargo cardiac build`: its tapes, NAME.cqb for one
/// phase, and NAME.2.cqb ... for several. A line `// cardiac: phase-cards=48,...` in the source gives how many
/// data cards each phase but the last reads.
pub fn rust(demo: &Path, name: &str) -> Machine {
    let file = |ending: &str| machine_file(demo, name, ending).unwrap_or_else(|| panic!("no machine called {name}"));
    // written by hand for speed (PART/hand/NAME.asm): that tape in this one's place, the same deck,
    // and this one's native build beside it for native checks
    let by_hand = file(".rs").replace("/machines/", "/hand/").replace(".rs", ".asm");
    if demo.join(&by_hand).exists() {
        let mut m = hand(demo, &by_hand).native(name, &[]);
        m.name = format!("{name} (hand)");
        return m;
    }
    let src = read(demo, &file(".rs"));
    let split: Vec<usize> = src.lines().find_map(|l| l.strip_prefix("// cardiac: phase-cards=")).map(|v| v.split(',').map(|n| n.trim().parse().unwrap()).collect()).unwrap_or_default();
    let mut m = Machine::new(name, tapes(demo, name).phases);
    // with phase-cards the host's one deck is dealt out between the phases; without, the host deals it per phase
    assert!(split.is_empty() || split.len() + 1 == m.tapes.len(), "{name}: phase-cards needs one count per phase but the last");
    m = m.native(name, &[]);
    m.name = format!("{name} (Rust)"); m.split = split;
    // the shadow runs the hand-written original on one tape, so only one-phase machines have one
    // (`// cardiac: diverged` in the source: it differs from the original on purpose)
    if crate::native::settings().shadow && m.tapes.len() == 1 && !src.contains("// cardiac: diverged") {
        if let Some(f) = src.lines().find_map(|l| l.strip_prefix("// cardiac: replaces ")).filter(|f| !f.contains(' ')) { m.shadow = Some(Box::new(hand(demo, f.trim()))); }
    }
    m
}

/// The machine written in Rust called NAME, if there is one (for a port whose cards are dealt differently, so
/// it is not a one-for-one replacement), else the hand-written FILE.
pub fn rust_or(demo: &Path, name: &str, file: &str) -> Machine {
    if machine_file(demo, name, ".rs").is_some() { rust(demo, name) } else { hand(demo, file) }
}

