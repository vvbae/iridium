pub mod command_parser;

use std::{
    fs::File,
    io::{self, Read, Write},
    net::TcpStream,
    path::Path,
    sync::mpsc::{self, Receiver, SendError, Sender},
};

use log::debug;

use crate::{
    assembler::{program::Program, symbols::Symbol, Assembler},
    cluster::cluster_client::ClusterClient,
    error::{IridiumError, Result},
    parse::Parse,
    scheduler::Scheduler,
    vm::VM,
};

use self::command_parser::CommandParser;

const COMMAND_PREFIX: char = '!';
pub static REMOTE_BANNER: &str = "Welcome to Iridium! Let's be productive!";
pub static PROMPT: &str = ">>> ";

#[derive(Default)]
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
    pub tx_pipe: Option<Box<Sender<String>>>,
    pub rx_pipe: Option<Box<Receiver<String>>>,
}

impl REPL {
    pub fn new(vm: VM) -> REPL {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        Self {
            command_buffer: Vec::<String>::new(),
            vm,
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
            tx_pipe: Some(Box::new(tx)),
            rx_pipe: Some(Box::new(rx)),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.send_message(REMOTE_BANNER.to_string())?;
        self.send_prompt()?;
        loop {
            // This allocates a new String in which to store whatever the user types each iteration.
            // TODO: Figure out how allocate this outside of the loop and re-use it every iteration
            let mut buffer = String::new();

            let stdin = io::stdin();

            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");

            let historical_copy = buffer.clone();
            self.command_buffer.push(historical_copy);

            self.run_single(&buffer)?;
        }
    }

    /// Execute single command for remote client
    pub fn run_single(&mut self, buffer: &str) -> Result<()> {
        if buffer.starts_with(COMMAND_PREFIX) {
            self.execute_command(buffer)?;
        } else {
            match Program::parse(buffer) {
                Ok((_, program)) => {
                    let mut bytes = program.to_bytes(&self.asm.symbols);
                    self.vm.program.append(&mut bytes);
                    self.vm.run_once();
                }
                Err(e) => {
                    self.send_message(format!("Unable to parse input: {:?}", e))?;
                    self.send_prompt()?;
                }
            };
        }
        Ok(())
    }

    fn execute_command(&mut self, input: &str) -> Result<()> {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..])?,
            "!history" => self.history(&args[1..])?,
            "!program" => self.program(&args[1..])?,
            "!clear_program" => self.clear_program(&args[1..]),
            "!clear_registers" => self.clear_registers(&args[1..])?,
            "!registers" => self.registers(&args[1..])?,
            "!symbols" => self.symbols(&args[1..])?,
            "!load_file" => self.load_file(&args[1..])?,
            "!spawn" => self.spawn(&args[1..])?,
            "!start_cluster" => self.start_cluster(&args[1..])?,
            "!join_cluster" => self.join_cluster(&args[1..])?,
            "!cluster_members" => self.cluster_members(&args[1..])?,
            _ => {
                self.send_message("Invalid command!".to_string())?;
            }
        };

        Ok(())
    }

    fn quit(&mut self, _args: &[&str]) -> Result<()> {
        self.send_message("Farewell! Have a great day!".to_string())?;
        std::process::exit(0);
    }

    fn history(&mut self, _args: &[&str]) -> Result<()> {
        let mut results = vec![];
        for command in &self.command_buffer {
            results.push(command.clone());
        }
        self.send_message(format!("{:#?}", results))?;

        Ok(())
    }

    fn program(&mut self, _args: &[&str]) -> Result<()> {
        self.send_message("Listing instructions currently in VM's program vector: ".to_string())?;
        let mut results = vec![];
        for instruction in &self.vm.program {
            results.push(*instruction)
        }
        self.send_message(format!("{:#?}", results))?;
        self.send_message("End of Program Listing".to_string())?;

        Ok(())
    }

    fn clear_program(&mut self, _args: &[&str]) {
        self.vm.program.clear();
    }

    fn clear_registers(&mut self, _args: &[&str]) -> Result<()> {
        self.send_message("Setting all registers to 0".to_string())?;
        for i in 0..self.vm.registers.len() {
            self.vm.registers[i] = 0;
        }
        self.send_message("Done!".to_string())?;

        Ok(())
    }

    fn registers(&mut self, _args: &[&str]) -> Result<()> {
        self.send_message("Listing registers and all contents:".to_string())?;
        let mut results = vec![];
        for register in &self.vm.registers {
            results.push(*register);
        }
        self.send_message(format!("{:#?}", results))?;
        self.send_message("End of Register Listing".to_string())?;

        Ok(())
    }

    fn symbols(&mut self, _args: &[&str]) -> Result<()> {
        let mut results = vec![];
        for symbol in &self.asm.symbols.symbols {
            results.push(<&Symbol>::clone(&symbol));
        }
        self.send_message("Listing symbols table:".to_string())?;
        self.send_message(format!("{:#?}", results))?;
        self.send_message("End of Symbols Listing".to_string())?;

        Ok(())
    }

    fn load_file(&mut self, _args: &[&str]) -> Result<()> {
        let contents = self.get_data_from_load();
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    self.send_message("Sending assembled program to VM".to_string())?;
                    self.vm.program.append(&mut assembled_program);
                    self.vm.run();
                }
                Err(errors) => {
                    if let IridiumError::Assemble(e) = errors {
                        for error in e {
                            self.send_message(format!("Unable to parse input: {}", error))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn spawn(&mut self, _args: &[&str]) -> Result<()> {
        let contents = self.get_data_from_load();
        self.send_message(format!("Loaded contents: {:#?}", contents))?;
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    self.send_message("Sending assembled program to VM".to_string())?;
                    self.vm.program.append(&mut assembled_program);
                    self.scheduler.get_thread(self.vm.clone());
                }
                Err(errors) => {
                    if let IridiumError::Assemble(e) = errors {
                        for error in e {
                            self.send_message(format!("Unable to parse input: {}", error))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn start_cluster(&mut self, _args: &[&str]) -> Result<()> {
        self.send_message("Started cluster server!".to_string())?;
        self.vm.bind_cluster_server();

        Ok(())
    }

    fn join_cluster(&mut self, args: &[&str]) -> Result<()> {
        debug!("Joining cluster with VM ID: {:?}", self.vm.alias);
        self.send_message("Attempting to join cluster...".to_string())?;

        let ip = args[0];
        let port = args[1];

        let addr = ip.to_owned() + ":" + port;
        let alias = self.vm.alias.as_ref().unwrap();
        let _addr = addr.clone();

        if let Ok(stream) = TcpStream::connect(addr) {
            self.send_message("Connected to cluster!".to_string())?;
            let mut cc = ClusterClient::new(stream)?.with_alias(alias.to_string());
            cc.send_hello()?;
            self.send_message(format!("Node {} sent hello to server at {}", alias, _addr))?;
            if let Ok(mut lock) = self.vm.conn_manager.write() {
                lock.add_client(alias.to_string(), cc);
            }
        } else {
            self.send_message("Could not connect to cluster!".to_string())?;
        }

        Ok(())
    }

    fn cluster_members(&mut self, args: &[&str]) -> Result<()> {
        self.send_message(format!("Listing Known Nodes:"))?;
        if let Ok(lock) = self.vm.conn_manager.read() {
            let cluster_members = lock.get_client_names();
            self.send_message(format!("{:#?}", cluster_members))?;
        }

        Ok(())
    }

    pub fn send_message(&self, msg: String) -> Result<()> {
        match &self.tx_pipe {
            Some(pipe) => {
                pipe.send(msg + "\n")?;
                Ok(())
            }
            None => Err(IridiumError::Send(SendError(
                "Send pipe not found on repl".to_owned(),
            ))),
        }
    }

    pub fn send_prompt(&mut self) -> Result<()> {
        match &self.tx_pipe {
            Some(pipe) => {
                pipe.send(PROMPT.to_owned())?;
                Ok(())
            }
            None => Err(IridiumError::Send(SendError(
                "Send pipe not found on repl".to_owned(),
            ))),
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
        let mut f = match File::open(filename) {
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
