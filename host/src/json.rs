//! Just enough JSON for the listing font.
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Json { Num(f64), Str(String), Arr(Vec<Json>), Obj(BTreeMap<String, Json>), Bool(bool), Null }

pub fn parse(s: &str) -> Json { let mut p = P { b: s.as_bytes(), i: 0 }; let v = p.value(); v }

struct P<'a> { b: &'a [u8], i: usize }
impl P<'_> {
    fn ws(&mut self) { while self.i < self.b.len() && (self.b[self.i] as char).is_whitespace() { self.i += 1; } }
    fn value(&mut self) -> Json {
        self.ws();
        match self.b[self.i] {
            b'{' => { self.i += 1; let mut m = BTreeMap::new();
                loop { self.ws(); if self.b[self.i] == b'}' { self.i += 1; break; }
                    let k = match self.value() { Json::Str(s) => s, _ => panic!("key") }; self.ws(); assert_eq!(self.b[self.i], b':'); self.i += 1;
                    let v = self.value(); m.insert(k, v); self.ws(); if self.b[self.i] == b',' { self.i += 1; } }
                Json::Obj(m) }
            b'[' => { self.i += 1; let mut a = vec![];
                loop { self.ws(); if self.b[self.i] == b']' { self.i += 1; break; }
                    a.push(self.value()); self.ws(); if self.b[self.i] == b',' { self.i += 1; } }
                Json::Arr(a) }
            b'"' => { self.i += 1; let mut s = String::new();
                loop { let c = self.b[self.i]; self.i += 1;
                    match c { b'"' => break,
                        b'\\' => { let e = self.b[self.i]; self.i += 1; match e {
                            b'n' => s.push('\n'), b't' => s.push('\t'), b'u' => { let h = std::str::from_utf8(&self.b[self.i..self.i + 4]).unwrap(); self.i += 4; s.push(char::from_u32(u32::from_str_radix(h, 16).unwrap()).unwrap()); }
                            o => s.push(o as char) } }
                        _ => { let st = self.i - 1; let mut e = self.i; while e < self.b.len() && self.b[e] != b'"' && self.b[e] != b'\\' { e += 1; }
                            s.push_str(std::str::from_utf8(&self.b[st..e]).unwrap()); self.i = e; } } }
                Json::Str(s) }
            b't' => { self.i += 4; Json::Bool(true) } b'f' => { self.i += 5; Json::Bool(false) } b'n' => { self.i += 4; Json::Null }
            _ => { let st = self.i; while self.i < self.b.len() && matches!(self.b[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') { self.i += 1; }
                Json::Num(std::str::from_utf8(&self.b[st..self.i]).unwrap().parse().unwrap()) }
        }
    }
}

impl Json {
    pub fn get(&self, k: &str) -> &Json { match self { Json::Obj(m) => &m[k], _ => panic!("not an object") } }
    pub fn ints(&self) -> Vec<i32> { match self { Json::Arr(a) => a.iter().map(|v| match v { Json::Num(n) => *n as i32, _ => panic!() }).collect(), _ => panic!() } }
    pub fn strs(&self) -> Vec<String> { match self { Json::Arr(a) => a.iter().map(|v| match v { Json::Str(s) => s.clone(), _ => panic!() }).collect(), _ => panic!() } }
}

/// A font file: character -> its five glyph-row cards.
pub fn font(j: &Json) -> BTreeMap<char, Vec<i32>> {
    match j.get("font") { Json::Obj(m) => m.iter().map(|(k, v)| (k.chars().next().unwrap(), v.ints())).collect(), _ => panic!() }
}
