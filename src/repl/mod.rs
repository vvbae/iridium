use std::{
    io::{self, Write},
    num::ParseIntError,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::vm::VM;

pub static REMOTE_BANNER: &'static str = "Start using Iridium ☂️";
pub static PROMPT: &'static str = ">>> ";

pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    // pub tx_pipe: Option<Box<Sender<String>>>,
    // pub rx_pipe: Option<Box<Receiver<String>>>,
}

impl REPL {
    pub fn new() -> REPL {
        // let (tx, rx) = mpsc::channel::<String>();
        Self {
            command_buffer: Vec::<String>::new(),
            vm: VM::new(),
            // tx_pipe: Some(Box::new(tx)),
            // rx_pipe: Some(Box::new(rx)),
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
                _ => {
                    let results = self.parse_hex(buffer);
                    match results {
                        Ok(bytes) => {
                            for byte in bytes {
                                self.vm.add_byte(byte)
                            }
                        }
                        Err(_e) => {
                            println!("Unable to decode hex string. Please enter 4 groups of 2 hex characters.")
                        }
                    };
                    self.vm.run_once();
                }
            }
        }
    }

    // pub fn send_message(&mut self, msg: String) {
    //     match &self.tx_pipe {
    //         Some(pipe) => match pipe.send(msg) {
    //             Ok(_) => {}
    //             Err(_e) => {}
    //         },
    //         None => {}
    //     }
    // }

    // pub fn send_prompt(&mut self) {
    //     match &self.tx_pipe {
    //         Some(pipe) => match pipe.send(PROMPT.to_owned()) {
    //             Ok(_) => {}
    //             Err(_e) => {}
    //         },
    //         None => {}
    //     }
    // }

    /// Accepts a hexadecimal string WITHOUT a leading `0x` and returns a Vec of u8
    /// Example for a LOAD command: (LOAD)00 (12)0C (1000)03 E8
    pub fn parse_hex(&mut self, i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split = i.split(" ").collect::<Vec<&str>>();
        let mut results = Vec::<u8>::new();
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }
}
