use std::collections::HashMap;
use thiserror::Error;

use uxn_macro::{UxnMacro, UxnMacroErr};
use crate::utils::*;

mod instr;
mod uxn_macro;

#[derive(Default)]
pub struct UxnAssembler<'a> {
    macros: HashMap<&'a str, UxnMacro<'a>>,

    bytes: Vec<u8>,
    labels: HashMap<String, (bool, usize, Vec<LabelCall>)>,
    // for bootstrapping: lbllen lbl addr calls...

    code: &'a str,

    last_main_lbl: &'a str,
    in_comment: bool,
    in_macro: bool,
    cur_macro_name: &'a str,
    cur_macro_buf: Vec<&'a str>,
    counter: usize,
}
struct LabelCall {
    pub pos: usize,
    pub kind: LabelKind
}
#[derive(PartialEq)]
enum LabelKind {
    Abs, Rel, Zpg
}

impl<'a> UxnAssembler<'a> {
    pub fn new(code: &str) -> UxnAssembler {
        let mut a = UxnAssembler::default();
        a.code = code;
        a
    }
    pub fn output(&'a self) -> &'a [u8] {
        &self.bytes[0x0100..]
    }

    pub fn assemble(&mut self) -> Result<(), UxnAssemblerErr> {
        let lines = self.code.split('\n').enumerate();
        /*let mut last_main_lbl = "";
        let mut in_comment = false;

        let mut in_macro = false;
        let mut cur_macro_name = "";
        let mut cur_macro_buf = Vec::new();

        let mut counter = 0;*/

        for (i, line) in lines {
            let frags = line.split_ascii_whitespace();
            for f in frags {
                self.do_word(f, i)?;
            }
        }

        for m in &self.macros {
            println!("{:?}", m)
        }

        for (lbl, (exists, addr, calls)) in &self.labels {
            if !exists {
                return Err(UxnAssemblerErr::UndefinedSymbol(lbl.into()))
            }
            for c in calls {
                match c.kind {
                    LabelKind::Abs => {
                        let bytes = (*addr as u16).to_be_bytes();
                        set_vec(&mut self.bytes, bytes[0], c.pos);
                        set_vec(&mut self.bytes, bytes[1], c.pos + 1);
                    }
                    LabelKind::Rel => {
                        let mut rel_amt = *addr as i8 - c.pos as i8;
                        rel_amt -= 2; // compensate for following instructions
                        set_vec(&mut self.bytes, rel_amt as u8, c.pos)
                    }
                    LabelKind::Zpg => {
                        if addr > &0xff {
                            return Err(UxnAssemblerErr::NotInZpg(lbl.into()))
                        }
                        set_vec(&mut self.bytes, *addr as u8, c.pos)
                    }
                }
            }
        }

        Ok(())
    }

    pub fn do_word(&mut self, word: &'a str, line: usize) -> Result<(), UxnAssemblerErr> {
        if self.in_comment { // comments
            if word.starts_with(')') {
                self.in_comment = false
            }
        }
        else if word.starts_with('(') {
            self.in_comment = true
        }
        else if self.in_macro { // macro declarations, not invocations
            self.cur_macro_buf.push(word); // push everything into the buf
            if word == "}" {
                self.in_macro = false;
                let mac = UxnMacro::new(&self.cur_macro_buf)?;
                self.macros.insert(self.cur_macro_name, mac);
            }
        }
        else if word.starts_with('%') { // macro
            self.cur_macro_name = &word[1..];
            self.in_macro = true;
            self.cur_macro_buf.clear()
        }
        else if word.starts_with('@') { // parent label
            self.last_main_lbl = &word[1..];
            self.add_label(self.last_main_lbl)
        }
        else if word.starts_with('&') { // child label
            let lbl = &format!("{}/{}", self.last_main_lbl, &word[1..]);
            self.add_label(lbl)
        }
        else if word.starts_with('|') { // abs pad
            let pad = parse_hex(&word[1..])?;
            self.counter = match pad {
                ByteOrShort::Byte(b) => b as usize,
                ByteOrShort::Short(s) => s as usize
            }
        }
        else if word.starts_with('$') {
            let pad = parse_hex(&word[1..])?;
            self.counter += match pad {
                ByteOrShort::Byte(b) => b as usize,
                ByteOrShort::Short(_) => panic!()
            }
        }
        else { // AFTER THIS POINT we are doing real things, not directives
            self.do_word_to_bytes(word, line)?;
            return Ok(())
        }
        println!("{}", word);
        Ok(())
    }

    pub fn do_word_to_bytes(&mut self, word: &str, line: usize) -> Result<(), UxnAssemblerErr> {
        println!("{}", word);
        let wstart = word.chars().next().unwrap();
        match wstart {
            '#' => { // byte/short literal
                match parse_hex(&word[1..])? {
                    ByteOrShort::Byte(b) => {
                        set_vec(&mut self.bytes, 0x80, self.counter); // LIT
                        self.counter += 1;
                        set_vec(&mut self.bytes, b, self.counter);
                    }
                    ByteOrShort::Short(s) => {
                        set_vec(&mut self.bytes, 0xa0, self.counter); // LIT2
                        let s = s.to_be_bytes();
                        self.counter += 1;
                        set_vec(&mut self.bytes, s[0], self.counter);
                        self.counter += 1;
                        set_vec(&mut self.bytes, s[1], self.counter);
                    }
                }
                self.counter += 1;
            }
            ';' | ':' | '.' | ',' => { // some kind of label
                let mut lbl = String::from(&word[1..]);
                if lbl.starts_with('&') { // sub label
                    lbl = format!("{}/{}", self.last_main_lbl, &lbl[1..])
                }
                if wstart == ';' { // abs lit label
                    set_vec(&mut self.bytes, 0xa0, self.counter); // LIT2
                    self.counter += 1;
                    self.add_label_call(&lbl, LabelKind::Abs)
                }
                else if wstart == ':' { // abs raw label
                    self.add_label_call(&lbl, LabelKind::Abs)
                }
                else if wstart == '.' { // zpg lit label
                    set_vec(&mut self.bytes, 0x80, self.counter); // LIT
                    self.counter += 1;
                    self.add_label_call(&lbl, LabelKind::Zpg)
                }
                else if wstart == ',' { // rel lit label
                    set_vec(&mut self.bytes, 0x80, self.counter); // LIT
                    self.counter += 1;
                    self.add_label_call(&lbl, LabelKind::Rel)
                }
            }
            '\'' => { // raw char
                let c = &word[1..2].as_bytes()[0];
                set_vec(&mut self.bytes, *c, self.counter);
                self.counter += 1;
            }
            '"' => { // raw word
                let bytes = &word[1..].as_bytes();
                for (i, b) in bytes.iter().enumerate() {
                    set_vec(&mut self.bytes, *b, self.counter + i)
                }
                self.counter += bytes.len()
            }

            _ => { // opcode or raw byte/short
                match parse_hex(word) {
                    Ok(v) => { // valid hex, raw byte/short
                        match v {
                            ByteOrShort::Byte(b) => {
                                set_vec(&mut self.bytes, b, self.counter);
                            }
                            ByteOrShort::Short(s) => {
                                let s = s.to_be_bytes();
                                set_vec(&mut self.bytes, s[0], self.counter);
                                self.counter += 1;
                                set_vec(&mut self.bytes, s[1], self.counter);
                            }
                        }
                        self.counter += 1;
                    }
                    Err(_) => {
                        match self.macros.get(word) {
                            Some(macr) => {
                                for w in macr.expand(&[]) {
                                    self.do_word_to_bytes(&w, line)?;
                                }
                            }
                            None => {
                                let i = instr::new(word)?;
                                set_vec(&mut self.bytes, i, self.counter);
                                self.counter += 1
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// does counter bumps for you
    fn add_label_call(&mut self, lbl: &str, kind: LabelKind) {
        let is_abs = kind == LabelKind::Abs;
        let call = LabelCall {
            pos: self.counter,
            kind
        };
        if is_abs {
            self.counter += 1
        }
        self.counter += 1;
        match self.labels.get_mut(lbl) {
            Some(v) => {
                v.2.push(call)
            }
            None => {
                self.labels.insert(lbl.into(), (false, 0, vec![call]));
            }
        }
    }
    fn add_label(&mut self, lbl: &str) {
        match self.labels.get_mut(lbl) {
            Some(v) => {
                v.1 = self.counter;
                v.0 = true
            }
            None => {
                self.labels.insert(lbl.into(), (true, self.counter, Vec::new()));
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum UxnAssemblerErr {
    #[error("macro error: ")]
    MacrErr(#[from] UxnMacroErr),
    #[error("hex error: ")]
    HexErr(#[from] UxnHexErr),
    #[error("instruction error: ")]
    InstrErr(#[from] instr::UxnrInstrErr),
    #[error("undefined symbol: ")]
    UndefinedSymbol(String),
    #[error("label not in zpg: ")]
    NotInZpg(String),
}
