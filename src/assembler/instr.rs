use thiserror::Error;

/// send EVERYTHING that doesnt start with a glyph or parse into 2-char hex, here
/// 
/// DOES NOT deal with macros
pub fn new(word: &str) -> Result<u8, UxnrInstrErr> {
    // parse kind
    let instr = &word[..3];

    let mut kind = match INSTRUCTIONS.iter().position(|i| i == &instr) {
        Some(v) => v,
        None => return Err(UxnrInstrErr::BadInstr(instr.into()))
    };

    let modes = if instr != word { // we have modes
        if kind == 0 {
            return Err(UxnrInstrErr::ModesOnBrk)
        }
        &word[3..]
    }
    else {
        ""
    };
    let mut keep = false;
    let mut rst = false;
    let mut short = false;
    for c in modes.chars() {
        match c {
            'k' => keep = true,
            'r' => rst = true,
            '2' => short = true,
            _ => return Err(UxnrInstrErr::BadMode(c))
        }
    }
    if kind == 33 { // lit
        keep = true;
        kind = 0
    }

    let mut b = kind.clone() as u8;
    b ^= 0b1000_0000 * keep as u8;
    b ^= 0b0100_0000 * rst as u8;
    b ^= 0b0010_0000 * short as u8;
    Ok(b)
}

pub const INSTRUCTIONS: &'static [&'static str] = &[ // 100% kludge but I Don't Care
    "BRK", "INC", "POP", "NIP", "SWP", "ROT", "DUP", "OVR",
    "EQU", "NEQ", "GTH", "LTH", "JMP", "JCN", "JSR", "STH",
    "LDZ", "STZ", "LDR", "STR", "LDA", "STA", "DEI", "DEO",
    "ADD", "SUB", "MUL", "DIV", "AND", "ORA", "EOR", "SFT",
    "LIT"
];

/*pub struct UxnInstruction {
    kind: InstrKind,
    keep: bool,
    rst: bool,
    short: bool,
    orig_line: usize
}
#[repr(u8)]
#[derive(Clone, PartialEq)]
enum InstrKind {
    // stack
    Brk, Inc, Pop, Nip, Swp, Rot, Dup, Ovr, // Brk is also Lit
    // logic
    Equ, Neq, Gth, Lth, Jmp, Jcn, Jsr, Sth,
    // mem
    Ldz, Stz, Ldr, Str, Lda, Sta, Dei, Deo,
    // arith
    Add, Sub, Mul, Div, And, Ora, Eor, Sft,
    // lit (special case)
    Lit
}*/

/*impl UxnInstruction {
    /// send EVERYTHING that doesnt start with a glyph or parse into 2-char hex, here
    /// 
    /// DOES NOT deal with macros
    pub fn new(word: &str, line: usize) -> Result<Self, UxnrInstrErr> {
        // parse kind
        type I = InstrKind;
        let instr = &word[..3];
        let mut kind = match instr.to_lowercase().as_str() {
            "brk" => I::Brk, "inc" => I::Inc, "pop" => I::Pop, "nip" => I::Nip,
            "swp" => I::Swp, "rot" => I::Rot, "dup" => I::Dup, "ovr" => I::Ovr,

            "equ" => I::Equ, "neq" => I::Neq, "gth" => I::Gth, "lth" => I::Lth,
            "jmp" => I::Jmp, "jcn" => I::Jcn, "jsr" => I::Jsr, "sth" => I::Sth,

            "ldz" => I::Ldz, "stz" => I::Stz, "ldr" => I::Ldr, "str" => I::Str,
            "lda" => I::Lda, "sta" => I::Sta, "dei" => I::Dei, "deo" => I::Deo,

            "add" => I::Add, "sub" => I::Sub, "mul" => I::Mul, "div" => I::Div,
            "and" => I::And, "ora" => I::Ora, "eor" => I::Eor, "sft" => I::Sft,

            "lit" => I::Lit,

            _ => return Err(UxnrInstrErr::BadInstr(String::from(instr)))
        };

        let modes = if instr != word { // we have modes
            if kind == I::Brk {
                return Err(UxnrInstrErr::ModesOnBrk)
            }
            &word[3..]
        }
        else {
            ""
        };
        let mut keep = false;
        let mut rst = false;
        let mut short = false;
        for c in modes.chars() {
            match c {
                'k' => keep = true,
                'r' => rst = true,
                '2' => short = true,
                _ => return Err(UxnrInstrErr::BadMode(c))
            }
        }
        if kind == I::Lit {
            keep = true;
            kind = I::Brk
        }

        Ok(UxnInstruction {
            kind, keep, rst, short,
            orig_line: line
        })
    }

    fn into_byte(&self) -> u8 {
        let mut b = self.kind.clone() as u8;
        b ^= 0b1000_0000 * self.keep as u8;
        b ^= 0b0100_0000 * self.rst as u8;
        b ^= 0b0010_0000 * self.short as u8;
        b
    }
}*/

#[derive(Debug, Error)]
pub enum UxnrInstrErr {
    #[error("unrecognised instruction")]
    BadInstr(String),
    #[error("unrecognised mode")]
    BadMode(char),
    #[error("modes are not allowed on BRK")]
    ModesOnBrk
}

