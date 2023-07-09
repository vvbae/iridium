pub mod command_parser;

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

use self::command_parser::CommandParser;

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
            // TODO: Figure out how allocate this outside of the loop and re-use it every iteration
            let mut buffer = String::new();

            let stdin = io::stdin();

            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");

            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");

            let historical_copy = buffer.clone();
            self.command_buffer.push(historical_copy);

            if buffer.starts_with("!") {
                self.execute_command(&buffer);
            } else {
                let program = match Program::parse(&buffer) {
                    Ok((_remainder, program)) => program,
                    Err(e) => {
                        println!("Unable to parse input: {:?}", e);
                        continue;
                    }
                };
                self.vm
                    .program
                    .append(&mut program.to_bytes(&self.asm.symbols));
                self.vm.run_once();
            }
        }
    }

    fn execute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!history" => self.history(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!clear_program" => self.clear_program(&args[1..]),
            "!clear_registers" => self.clear_registers(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!symbols" => self.symbols(&args[1..]),
            "!load_file" => self.load_file(&args[1..]),
            "!spawn" => self.spawn(&args[1..]),
            _ => {
                self.send_message("Invalid command!".to_string());
            }
        };
    }

    fn quit(&mut self, _args: &[&str]) {
        self.send_message("Farewell! Have a great day!".to_string());
        std::process::exit(0);
    }

    fn history(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for command in &self.command_buffer {
            results.push(command.clone());
        }
        self.send_message(format!("{:#?}", results));
    }

    fn program(&mut self, _args: &[&str]) {
        self.send_message("Listing instructions currently in VM's program vector: ".to_string());
        let mut results = vec![];
        for instruction in &self.vm.program {
            results.push(instruction.clone())
        }
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Program Listing".to_string());
    }

    fn clear_program(&mut self, _args: &[&str]) {
        self.vm.program.clear();
    }

    fn clear_registers(&mut self, _args: &[&str]) {
        self.send_message("Setting all registers to 0".to_string());
        for i in 0..self.vm.registers.len() {
            self.vm.registers[i] = 0;
        }
        self.send_message("Done!".to_string());
    }

    fn registers(&mut self, _args: &[&str]) {
        self.send_message("Listing registers and all contents:".to_string());
        let mut results = vec![];
        for register in &self.vm.registers {
            results.push(register.clone());
        }
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Register Listing".to_string());
    }

    fn symbols(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for symbol in &self.asm.symbols.symbols {
            results.push(symbol.clone());
        }
        self.send_message("Listing symbols table:".to_string());
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Symbols Listing".to_string());
    }

    fn load_file(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    self.send_message("Sending assembled program to VM".to_string());
                    self.vm.program.append(&mut assembled_program);
                    self.vm.run();
                }
                Err(errors) => {
                    for error in errors {
                        self.send_message(format!("Unable to parse input: {}", error));
                    }
                    return;
                }
            }
        } else {
            return;
        }
    }

    fn spawn(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        self.send_message(format!("Loaded contents: {:#?}", contents));
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    self.send_message("Sending assembled program to VM".to_string());
                    self.vm.program.append(&mut assembled_program);
                    self.scheduler.get_thread(self.vm.clone());
                }
                Err(errors) => {
                    for error in errors {
                        self.send_message(format!("Unable to parse input: {}", error));
                    }
                    return;
                }
            }
        } else {
            return;
        }
    }

    pub fn send_message(&self, msg: String) {
        println!("{}", msg);
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
