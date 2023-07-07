use crate::parse::Parse;

use self::{
    program::Program,
    symbols::{Symbol, SymbolTable, SymbolType},
};

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerPhase {
    First,
    Second,
}

impl Default for AssemblerPhase {
    fn default() -> Self {
        AssemblerPhase::First
    }
}

#[derive(Debug)]
pub struct Assembler {
    pub phase: AssemblerPhase,
    pub symbols: SymbolTable,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
        }
    }

    /// Process first and second phase of teh program
    /// Return program instructions as bytes from the second phase
    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        match Program::parse(raw) {
            Ok((_, program)) => {
                self.process_first_phase(&program);
                Some(self.process_second_phase(&program))
            }
            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                None
            }
        }
    }

    /// Extract program labels
    fn process_first_phase(&mut self, p: &Program) {
        self.extract_labels(p);
        self.phase = AssemblerPhase::Second;
    }

    /// Extract program instruction bytes
    fn process_second_phase(&self, p: &Program) -> Vec<u8> {
        let mut program = Vec::new();
        for i in &p.instructions {
            let mut bytes = i.to_bytes(&self.symbols);
            program.append(&mut bytes);
        }
        program
    }

    /// Go through every instruction and look for label declarations
    /// Once found, add it to symbol table
    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.label_name() {
                    Some(name) => {
                        let symbol = Symbol::new(name, SymbolType::Label, c);
                        self.symbols.add_symbol(symbol);
                    }
                    None => {}
                }
            }
            c += 4 // an instruction is 32-bit = 4 bytes
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vm::VM;

    use super::*;

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string =
            "load $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        // assert_eq!(program.len(), 21);
        vm.add_bytes(program);
        // assert_eq!(vm.program.len(), 21);
    }
}

pub mod assem_instruction;
pub mod program;
pub mod symbols;
pub mod token;
