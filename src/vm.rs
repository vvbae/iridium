use crate::{assembler::PIE_HEADER_PREFIX, instruction::Opcode};

/// Read 32-bit data (instruction), execute, repeat
#[derive(Default, Clone)]
pub struct VM {
    pub registers: [i32; 32], // 32-bits is an instruction; first 8-bit->Opcode; remaining->Operands
    pc: usize,                // program counter
    pub program: Vec<u8>,     // The bytecode of the program being run
    remainder: u32,           // Contains the remainder of modulo division ops
    equal_flag: bool,         // Contains the result of the last comparison operation
    heap: Vec<u8>,            // Memory heap
    ro_data: Vec<u8>,         // read-only section data
}

impl VM {
    pub fn new() -> VM {
        Self {
            registers: [0; 32],
            pc: 65,
            program: Vec::new(),
            remainder: 0,
            equal_flag: false,
            heap: Vec::new(),
            ro_data: Vec::new(),
        }
    }

    /// Wraps execution in a loop so it will continue to run until done or there is an error
    /// executing instructions.
    pub fn run(&mut self) -> u32 {
        // TODO: Should setup custom errors here
        if !self.verify_header() {
            println!("Header was incorrect");
            return 1;
        }

        // If the header is valid, we need to change the PC to be at bit 64.
        self.pc = 64;
        let mut is_done = false;
        while !is_done {
            is_done = self.execute_instruction();
        }
        0
    }

    /// Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn execute_instruction(&mut self) -> bool {
        if self.pc >= self.program.len() {
            return true;
        }
        match self.decode_opcode() {
            // halt
            Opcode::HLT => {
                println!("HLT encountered");
                return true;
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
            Opcode::NOP => {
                self.next_8_bits();
                self.next_8_bits();
                self.next_8_bits();
            }

            _ => {
                println!("Unrecognized opcode found! Terminating!");
                return true;
            }
        }
        false
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
}

#[cfg(test)]
mod tests {
    use crate::assembler::PIE_HEADER_LENGTH;

    use super::*;

    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }
        while prepension.len() < PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![3, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
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
