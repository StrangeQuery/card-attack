//! The listing's text machines, used by the cut's captions: a typesetter (ltex) sets
//! characters as glyph rows from the listing font, and a row machine (lrow), the row's glyph bits riding in its
//! program deck, draws it.
use crate::*;
use std::collections::BTreeMap;
use std::path::Path;

pub struct Listing { pub font: BTreeMap<char, Vec<i32>>, pub ltex: Machine, lrow: Compiled, name: &'static str, table: &'static str, rs: bool }

impl Listing {
    pub fn new(demo: &Path) -> Self {
        let font = json::font(&json::parse(&read(demo, "listing/data/font.json")));
        // the row machine written in Rust if it is (its row at resource_tex), else the hand-written one (at TEX)
        let rs = demo.join("listing/machines/lrow.rs").exists();
        let (file, name, table) = if rs { ("", "lrow (Rust)", "resource_tex") } else { ("listing/original/lrow.asm", "lrow", "TEX") };
        let lrow = if rs { compiled(demo, "lrow") } else { Compiled::of(&asm::assemble(3, &format!("{}\n{}", read(demo, "shared/none_data.asm"), read(demo, file))).unwrap()) };
        Listing { font, ltex: one(demo, "listing/original/ltex.asm"), lrow, name, table, rs }
    }
    /// The row machine with this row of glyph bits in its program.
    pub fn row_machine(&self, row: &[i32]) -> Machine {
        let mut cells = self.lrow.cells.clone();
        cells.extend(row.iter().enumerate().map(|(i, &v)| (self.lrow.label(self.table) + i as i32, v)));
        let m = Machine::new(self.name, vec![asm::tape(self.lrow.label("start"), &cells)]);
        if self.rs { m.native("lrow", row) } else { m }
    }
    pub fn cards(&self, s: &str) -> Vec<i32> { s.chars().flat_map(|c| self.font[&c].clone()).collect() }
}
