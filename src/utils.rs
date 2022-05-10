use thiserror::Error;

pub enum ByteOrShort {
    Byte(u8),
    Short(u16)
}
pub fn parse_hex(hex: &str) -> Result<ByteOrShort, UxnHexErr> { // helper method for hex literals
    match hex.len() { // work out if byte or short
        0 => {
            Err(UxnHexErr::EmptyHex)
        }
        2 => {
            match u8::from_str_radix(hex, 16) {
                Ok(v) => Ok(ByteOrShort::Byte(v)),
                Err(_) => Err(UxnHexErr::InvalidHex(String::from(hex)))
            }
        }
        4 => {
            match u16::from_str_radix(hex, 16) {
                Ok(v) => {
                    return Ok(ByteOrShort::Short(v))
                },
                Err(_) => Err(UxnHexErr::InvalidHex(String::from(hex)))
            }
        }
        _ => Err(UxnHexErr::BigHex(String::from(hex)))
    }
}

pub fn until_macro_args(word: &str) -> Option<&str> { // extract macro name, None if not macro
    for (i, c) in word.chars().enumerate() {
        if c == '[' { // match opening delim
            return Some(&word[..i])
        }
    }
    None
}

pub fn set_vec<T: Default + Clone>(vec: &mut Vec<T>, val: T, idx: usize) {
    let l = vec.len();
    if l > idx {
        vec[idx] = val
    }
    else { // if len is 5 and idx is 10, len needs to be 11
        vec.resize(idx + 1, T::default());
        vec[idx] = val
    }
}

/*pub fn collect_until<I, T>(iter: &mut I, mut pred: impl FnMut(T) -> bool) -> (Vec<T>, bool)
where I: Iterator<Item=T> + Sized, T: Clone {
    let mut ret = Vec::new();
    let mut early = false;
    loop {
        let item = iter.next();
        match item {
            Some(v) => {
                ret.push(v.clone());
                if pred(v) == true {
                    break
                }
            }
            None => {
                early = true;
                break
            }
        }
    }
    (ret, early)
}
pub trait IterHelpers {
    fn collect_until<T, F>(&mut self, pred: F) -> Vec<T>
    where F: FnMut(T) -> bool;
}
impl<I: Iterator<Item=T>> IterHelpers for I {
    fn collect_until<T, F>(&mut self, pred: F) -> Vec<T>
    where F: FnMut(T) -> bool {
        let mut ret = Vec::new();
    loop {
        let item = self.next();
        match item {
            Some(v) => {
                ret.push(v);
                if pred(v) == true {
                    break
                }
            }
            None => break
        }
    }
    ret
    }
}*/

#[derive(Error, Debug)]
pub enum UxnHexErr {
    #[error("empty hex literal")]
    EmptyHex,
    #[error("invalid hex literal")]
    InvalidHex(String),
    #[error("wrongly sized hex literal")]
    BigHex(String),
}
