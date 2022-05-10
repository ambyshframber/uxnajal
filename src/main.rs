use thiserror::Error;
use std::fs::{read_to_string, write};
use std::{io, io::{stdin, Read}};
use assembler::{UxnAssembler, UxnAssemblerErr};
use std::env::args;

mod utils;
mod assembler;

fn main() -> Result<(), UxnErr> {
    let a: Vec<String> = args().collect();
    if a.len() == 0 {
        panic!()
    }
    let mut code = String::new();
    if a[1] == "-" {
        stdin().read_to_string(&mut code)?;
    }
    else {
        code = read_to_string(&a[1])?
    };
    let mut asm = UxnAssembler::new(&code);
    asm.assemble()?;
    let rom = asm.output();
    if a.len() < 2 {
        write("out.rom", rom)?
    }
    else {
        write(&a[2], rom)?
    }

    Ok(())
}

#[derive(Debug, Error)]
enum UxnErr {
    #[error("assembler error: ")]
    AssemblerErr(#[from] UxnAssemblerErr),
    #[error("file system error")]
    FsError(#[from] io::Error)
}
