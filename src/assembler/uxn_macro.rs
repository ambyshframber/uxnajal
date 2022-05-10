use thiserror::Error;
use super::{UxnAssembler, UxnAssemblerErr};

#[derive(Debug)]
pub struct UxnMacro<'a> {
    args: Vec<&'a str>,
    code: Vec<&'a str>,
    times_used: usize
}
impl UxnMacro<'_> {
    /// macros come in like [ arg1 arg2 ] { ins ins }
    /// 
    /// will ALWAYS finish with } because of how the word parser works
    pub fn new<'a>(mac: &[&'a str]) -> Result<UxnMacro<'a>, UxnMacroErr> {
        let mut args = Vec::new();
        let mut code = Vec::new();

        let mut idx = 0; // IM SO SMRT

        if mac[idx] == "[" { // check if the macro args exist
            idx += 1;
            while mac[idx] != "]" { // go until arg end delim
                if mac[idx] == "}" {
                    return Err(UxnMacroErr::NoArgEndDelim)
                }
                args.push(mac[idx]);
                idx += 1;
            }
            idx += 1
        }
        println!("{:?} {}", mac, idx);
        if mac[idx] == "{" {
            idx += 1;
            while mac[idx] != "}" { // will ALWAYS finish in }
                code.push(mac[idx]);
                idx += 1;
            }
        }
        else {
            return Err(UxnMacroErr::NoBody)
        }
        
        Ok(UxnMacro {
            args, code,
            times_used: 0
        })
    }
    pub fn expand<'a>(&'a self, call: &[&str]) -> Vec<String> {
        let mut ret = Vec::new();
        for word in &self.code {
            ret.push(String::from(*word))
        }
        ret
    }
}
#[derive(Debug, Error)]
pub enum UxnMacroErr {
    #[error("macro with no argument end delimiter")]
    NoArgEndDelim,
    #[error("macro with no code body")]
    NoBody
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mac_test() {
        let mac = vec!["[", "beans", "]", "{", "ins", "ins2", "}"];
        println!("{:?}", UxnMacro::new(&mac).unwrap());
        let mac = vec!["{", "ins", "ins2", "}"];
        println!("{:?}", UxnMacro::new(&mac).unwrap())
    }
}
