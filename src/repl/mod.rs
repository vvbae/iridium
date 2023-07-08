use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use crate::{
    assembler::{program::Program, Assembler},
    parse::Parse,
    scheduler::Scheduler,
    vm::VM,
};

#[derive(Default)]
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
}

impl REPL {
    pub fn new() -> REPL {
        Self {
            command_buffer: Vec::<String>::new(),
            vm: VM::new(),
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to Iridium! Let's be productive!");
        loop {
            // This allocates a new String in which to store whatever the user types each iteration.
            // TODO: Figure out how create this outside of the loop and re-use it every iteration
            let mut buffer = String::new();

            // Blocking call until the user types in a command
            let stdin = io::stdin();

            // Annoyingly, `print!` does not automatically flush stdout like `println!` does, so we
            // have to do that there for the user to see our `>>> ` prompt.
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");

            // Here we'll look at the string the user gave us.
            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");
            let buffer = buffer.trim();

            self.command_buffer.push(buffer.to_string());

            match buffer {
                ".program" => {
                    println!("Listing instructions currently in VM's program vector:");
                    for instruction in &self.vm.program {
                        println!("{}", instruction);
                    }
                    println!("End of Program Listing");
                }
                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", self.vm.registers);
                    println!("End of Register Listing")
                }
                ".history" => {
                    for command in &self.command_buffer {
                        println!("{}", command);
                    }
                }
                ".quit" => {
                    println!("Farewell! Have a great day!");
                    std::process::exit(0);
                }
                ".load_file" => {
                    print!("Please enter the path to the file you wish to load: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    let mut tmp = String::new();
                    stdin
                        .read_line(&mut tmp)
                        .expect("Unable to read line from user");
                    let tmp = tmp.trim();
                    let filename = Path::new(&tmp);
                    let mut f = File::open(Path::new(&filename)).expect("File not found");
                    let mut contents = String::new();
                    f.read_to_string(&mut contents)
                        .expect("There was an error reading from the file");
                    let program = match Program::parse(&contents) {
                        // Rusts pattern matching is pretty powerful an can even be nested
                        Ok((_, program)) => program,
                        Err(e) => {
                            println!("Unable to parse input: {:?}", e);
                            continue;
                        }
                    };
                    // self.vm.program.append(&mut program.to_bytes());
                }
                ".spawn" => {
                    let contents = self.get_data_from_load();
                    if let Some(contents) = contents {
                        match self.asm.assemble(&contents) {
                            Ok(mut assembled_program) => {
                                println!("Sending assembled program to VM");
                                self.vm.program.append(&mut assembled_program);
                                println!("{:#?}", self.vm.program);
                                self.scheduler.get_thread(self.vm.clone());
                            }
                            Err(errors) => {
                                for error in errors {
                                    println!("Unable to parse input: {}", error);
                                }
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                }
                ".clear" => {
                    self.vm.program.clear();
                    println!("Program vector is cleared");
                }
                _ => {
                    let (_, result) = Program::parse(buffer).unwrap();
                    // let bytecode = result.to_bytes();
                    // for byte in bytecode {
                    //     self.vm.add_byte(byte)
                    // }

                    // self.vm.run_once();
                }
            }
        }
    }

    fn get_data_from_load(&mut self) -> Option<String> {
        let stdin = io::stdin();
        print!("Please enter the path to the file you wish to load: ");
        io::stdout().flush().expect("Unable to flush stdout");
        let mut tmp = String::new();

        stdin
            .read_line(&mut tmp)
            .expect("Unable to read line from user");
        println!("Attempting to load program from file...");

        let tmp = tmp.trim();
        let filename = Path::new(&tmp);
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => {
                println!("There was an error opening that file: {:?}", e);
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                println!("there was an error reading that file: {:?}", e);
                None
            }
        }
    }
}
