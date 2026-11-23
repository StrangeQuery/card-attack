// Lists every machine in segments/*/machines for src/machines.rs, each included as a module of its own, and builds
// the crate with --cfg native, so the machines' built-ins (segments/shared/cardiac.rs) read and punch cards on the host.
use std::fmt::Write;
use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(native)");
    println!("cargo:rustc-cfg=native");
    let segments = Path::new(env!("CARGO_MANIFEST_DIR")).join("../segments").canonicalize().unwrap();
    let mut found = vec![];
    for seg in std::fs::read_dir(&segments).unwrap().flatten() {
        let dir = seg.path().join("machines");
        let Ok(files) = std::fs::read_dir(&dir) else { continue };
        println!("cargo:rerun-if-changed={}", dir.display());   // a machine added or taken away
        for f in files.flatten() {
            let p = f.path();
            if p.extension().is_some_and(|x| x == "rs") { found.push((p.file_stem().unwrap().to_string_lossy().to_string(), p)); }
        }
    }
    found.sort();
    let mut out = String::new();
    for (name, path) in &found {
        writeln!(out, "#[allow(unused_attributes, dead_code, unused_imports, unused_macros, clippy::all)]\n#[path = {:?}]\nmod m_{name};", path.display().to_string()).unwrap();
    }
    out.push_str("\n/// The native build of machine NAME (its file's name, without .rs).\npub fn find(name: &str) -> Option<fn()> {\n    match name {\n");
    for (name, _) in &found { writeln!(out, "        {name:?} => Some(m_{name}::main),").unwrap(); }
    out.push_str("        _ => None,\n    }\n}\n");
    std::fs::write(Path::new(&std::env::var("OUT_DIR").unwrap()).join("machines.rs"), out).unwrap();
}
