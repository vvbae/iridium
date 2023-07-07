use std::{fs::File, io::Read, path::Path};

use clap::{arg, Command};
use iridium::{assembler, repl, vm::VM};

/// Attempts to read a file and return the contents. Exits if unable to read the file for any reason.
fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(Path::new(&filename)) {
        Ok(mut fh) => {
            let mut contents = String::new();
            match fh.read_to_string(&mut contents) {
                Ok(_) => {
                    return contents;
                }
                Err(e) => {
                    println!("There was an error reading file: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}

/// Starts a REPL that will run until the user kills it
fn start_repl() {
    let mut repl = repl::REPL::new();
    repl.run();
}

fn main() {
    let matches = Command::new("iridium")
        .version("1.0")
        .author("Vivi W. <polarsatellitest@gmail.com>")
        .about("Interpreter for the Iridium language")
        .arg(arg!([INPUT_FILE] "Path to the .iasm or .ir file to run").index(1))
        .get_matches();

    match matches.args_present() {
        true => {
            let filename = matches.get_one::<String>("INPUT_FILE").unwrap();
            let program = read_file(filename);
            let mut asm = assembler::Assembler::new();
            let mut vm = VM::new();
            let program = asm.assemble(&program);
            match program {
                Some(p) => {
                    vm.add_bytes(p);
                    vm.run();
                    std::process::exit(0);
                }
                None => {}
            }
        }
        false => start_repl(),
    }
}
