use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use clap::{arg, Command};
use iridium::{assembler, error::AssemblerError, repl, vm::VM};

/// Attempts to read a file and return the contents. Exits if unable to read the file for any reason.
fn read_file(tmp: &str) -> Result<String, io::Error> {
    let mut contents = String::new();
    let mut f = File::open(Path::new(tmp))?;
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Starts a REPL that will run until the user kills it
fn start_repl() {
    let mut repl = repl::REPL::new();
    repl.run();
}

fn main() -> Result<(), Vec<AssemblerError>> {
    let matches = Command::new("iridium")
        .version("1.0")
        .author("Vivi W. <polarsatellitest@gmail.com>")
        .about("Interpreter for the Iridium language")
        .arg(arg!([INPUT_FILE] "Path to the .iasm or .ir file to run").index(1))
        .get_matches();

    match matches.args_present() {
        true => {
            let filename = matches.get_one::<String>("INPUT_FILE").unwrap();
            match read_file(filename) {
                Ok(program) => {
                    let mut asm = assembler::Assembler::new();
                    let mut vm = VM::new();
                    let program = asm.assemble(&program)?;
                    vm.add_bytes(program);
                    vm.run();
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("There was an error opening that file: {:?}", e);
                }
            }
        }
        false => start_repl(),
    }

    Ok(())
}
