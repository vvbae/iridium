use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, Utc};
use log::debug;
use std::{
    f64::EPSILON,
    io::Cursor,
    net::SocketAddr,
    sync::{Arc, RwLock},
    thread,
};
use uuid::Uuid;

use crate::{
    assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX},
    cluster::{cluster_server::ClusterServer, manager::Manager},
    error::Result,
    instruction::Opcode,
};

// const DEFAULT_PEER_LISTENING_HOST: &str = "127.0.0.1";
// const DEFAULT_PEER_LISTENING_PORT: &str = "2254";
// const DEFAULT_NODE_ALIAS: &str = "";

#[derive(Clone, Debug)]
pub enum VMEventType {
    Start,
    Stop,
    Crash,
}

#[derive(Clone, Debug)]
pub struct VMEvent {
    event: VMEventType,
    at: DateTime<Utc>,
    app_id: Uuid,
}

/// Read 32-bit data (instruction), execute, repeat
#[derive(Default, Clone)]
pub struct VM {
    pub registers: [i32; 32], // 32-bits is an instruction; first 8-bit->Opcode; remaining->Operands
    pub float_registers: [f64; 32], // Array to store floating point
    pc: usize,                // program counter
    pub program: Vec<u8>,     // The bytecode of the program being run
    remainder: u32,           // Contains the remainder of modulo division ops
    equal_flag: bool,         // Contains the result of the last comparison operation
    heap: Vec<u8>,            // Memory heap
    ro_data: Vec<u8>,         // read-only section data
    id: Uuid,                 // UUID
    events: Vec<VMEvent>,     // events
    pub logical_cores: usize, // number of CPUs
    pub alias: Option<String>, // An alias that can be specified by the user and used to refer to the Node
    peer_host: Option<String>, // Server address that the VM will bind to for server-to-server communications
    pub peer_port: Option<String>, // Port the server will bind to for server-to-server communications
    pub conn_manager: Arc<RwLock<Manager>>, // Data structure to manage remote clients
}

impl VM {
    pub fn new() -> VM {
        Self {
            registers: [0; 32],
            float_registers: [0.0; 32],
            pc: 65,
            program: Vec::new(),
            remainder: 0,
            equal_flag: false,
            heap: Vec::new(),
            ro_data: Vec::new(),
            id: Uuid::new_v4(),
            events: Vec::new(),
            logical_cores: num_cpus::get(),
            alias: None,
            peer_host: None,
            peer_port: None,
            conn_manager: Arc::new(RwLock::new(Manager::new())),
        }
    }

    /// Wraps execution in a loop so it will continue to run until done or there is an error
    /// executing instructions.
    pub fn run(&mut self) -> Vec<VMEvent> {
        self.events.push(VMEvent {
            event: VMEventType::Start,
            at: Utc::now(),
            app_id: self.id.to_owned(),
        });
        // TODO: Should setup custom errors here
        if !self.verify_header() {
            self.events.push(VMEvent {
                event: VMEventType::Crash,
                at: Utc::now(),
                app_id: self.id.to_owned(),
            });
            println!("Header was incorrect");
            return self.events.clone();
        }

        self.pc = 64 + self.get_starting_offset();
        let mut is_done = None;
        while is_done.is_none() {
            is_done = self.execute_instruction();
        }
        self.events.push(VMEvent {
            event: VMEventType::Stop,
            at: Utc::now(),
            app_id: self.id.to_owned(),
        });
        self.events.clone()
    }

    /// Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn execute_instruction(&mut self) -> Option<u32> {
        if self.pc >= self.program.len() {
            return Some(1);
        }
        match self.decode_opcode() {
            // halt
            Opcode::HLT => {
                println!("HLT encountered");
                return None;
            }
            // LOAD $1 #15
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize;
                let number = self.next_16_bits();
                self.registers[register] = number as i32;
            }
            // ADD $0 $1 $2
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            }
            // SUB $0 $1 $2
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            }
            // MUL $0 $1 $2
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            }
            // DIV $0 $1 $2
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as u32;
            }
            // JMP $0
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            // JMPF $0
            Opcode::JMPF => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc += target as usize;
            }
            // JMPB $0
            Opcode::JMPB => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc -= target as usize;
            }
            // EQ $0 $1
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits();
            }
            // NEQ $0 $1
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.next_8_bits();
            }
            // GT $0 $1
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            // GTE $0 $1
            Opcode::GTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            // LT $0 $1
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            // LTE $0 $1
            Opcode::LTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            // ALOC $0
            Opcode::ALOC => {
                let bytes = self.registers[self.next_8_bits() as usize];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0)
            }
            // INC $0
            Opcode::INC => {
                let position = self.next_8_bits() as usize;
                self.registers[position] += 1;
                self.next_8_bits();
                self.next_8_bits();
            }
            // DEC $0
            Opcode::DEC => {
                let position = self.next_8_bits() as usize;
                self.registers[position] -= 1;
                self.next_8_bits();
                self.next_8_bits();
            }
            // JMPE $0
            Opcode::JMPE => {
                if self.equal_flag {
                    let target = self.registers[self.next_8_bits() as usize];
                    self.pc = target as usize;
                } else {
                    // TODO: Fix the bits
                }
            }
            // PRTS @symbol_name/$0
            Opcode::PRTS => {
                let starting_offset = self.next_16_bits() as usize;
                let ending_offset = self.ro_data[starting_offset..]
                    .iter()
                    .position(|&x| x != 0)
                    .unwrap();
                let result = std::str::from_utf8(&self.ro_data[starting_offset..ending_offset]);
                match result {
                    Ok(s) => {
                        print!("{}", s);
                    }
                    Err(e) => {
                        println!("Error decoding string for prts instruction: {:#?}", e)
                    }
                };
            }
            // Begin floating point 64-bit instructions
            Opcode::LOADF64 => {
                let register = self.next_8_bits() as usize;
                let number = f64::from(self.next_16_bits());
                self.float_registers[register] = number;
            }
            Opcode::ADDF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUBF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MULF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIVF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 / register2;
            }
            Opcode::EQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() < EPSILON;
                self.next_8_bits();
            }
            Opcode::NEQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() > EPSILON;
                self.next_8_bits();
            }
            Opcode::GTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            Opcode::GTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            Opcode::LTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            Opcode::LTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            Opcode::NOP => {
                self.next_8_bits();
                self.next_8_bits();
                self.next_8_bits();
            }
            Opcode::SHL => {
                let reg_num = self.next_8_bits() as usize;
                let num_bits = match self.next_8_bits() {
                    0 => 16,
                    other => other,
                };
                self.registers[reg_num] = self.registers[reg_num].wrapping_shl(num_bits.into());
                self.next_8_bits();
            }
            // SHR $<reg_num> #<number of bits> shifts to the right by default 16 bits
            Opcode::SHR => {
                let reg_num = self.next_8_bits() as usize;
                let num_bits = match self.next_8_bits() {
                    0 => 16,
                    other => other,
                };
                self.registers[reg_num] = self.registers[reg_num].wrapping_shr(num_bits.into());
                self.next_8_bits();
            }
            _ => {
                println!("Unrecognized opcode found! Terminating!");
                return Some(1);
            }
        }
        None
    }

    /// Get starting offset of the section after read-only
    fn get_starting_offset(&self) -> usize {
        let mut rdr = Cursor::new(&self.program[4..8]);
        rdr.read_u32::<LittleEndian>().unwrap() as usize
    }

    /// Adds an arbitrary byte to the VM's program
    pub fn add_byte(&mut self, b: u8) {
        self.program.push(b);
    }

    /// Adds a vector of bytes to the VM's program
    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    pub fn get_test_vm() -> VM {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 10;
        test_vm
    }

    /// Add alias to this VM
    pub fn with_alias(mut self, alias: &String) -> Self {
        self.alias = match alias.as_ref() {
            "" => None,
            other => Some(other.to_owned()),
        };
        self
    }

    /// Add cluster binding to this VM
    pub fn with_cluster_bind(mut self, peer_host: &String, peer_port: &String) -> Self {
        self.peer_host = Some(peer_host.to_string());
        self.peer_port = Some(peer_port.to_string());
        self
    }

    /// Listen for peer connections
    pub fn bind_cluster_server(&mut self) {
        let host = self.peer_host.as_ref().unwrap();
        let port = self.peer_port.as_ref().unwrap();
        debug!(
            "Node {:?} is listening for incoming connections on {}:{}",
            self.alias, host, port,
        );
        let socket_addr = (host.to_owned() + ":" + port)
            .parse::<SocketAddr>()
            .unwrap();
        let conn_manager = self.conn_manager.clone();
        let alias = self.alias.clone().unwrap();
        debug!("Spawning listening thread");
        thread::spawn(move || -> Result<()> {
            let mut server = ClusterServer::new(alias, conn_manager);
            server.listen(socket_addr)?;
            Ok(())
        });
    }

    /// Decode current opcode and increment program counter
    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        opcode
    }

    /// Read next 8 bits
    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        result
    }

    /// Read next 16 bits
    fn next_16_bits(&mut self) -> u16 {
        let result = ((self.program[self.pc] as u16) << 8) | self.program[self.pc + 1] as u16;
        self.pc += 2;
        result
    }

    /// Processes the header of bytecode the VM wants to execute
    fn verify_header(&self) -> bool {
        self.program[0..4] == PIE_HEADER_PREFIX
    }

    /// Prepend header to the body
    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte);
        }
        while prepension.len() < PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![3, 0, 1, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 50);
    }

    #[test]
    fn test_prts_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.ro_data.append(&mut vec![72, 101, 108, 108, 111, 0]);
        test_vm.program = vec![21, 0, 0, 0];
        test_vm.run_once();
    }

    #[test]
    fn test_create_new() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0)
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = VM::new();
        let test_bytes = vec![0, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = VM::new();
        let test_bytes = vec![200, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 1;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0, 6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 10;
        test_vm.program = vec![9, 0, 1, 0, 9, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 20;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_jeq_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 7;
        test_vm.equal_flag = true;
        test_vm.program = vec![15, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![16, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024);
    }
}
