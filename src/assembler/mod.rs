use std::vec;

use crate::{
    error::{AssemblerError, IridiumError, Result},
    parse::Parse,
};

use self::{
    assem_instruction::AssemblerInstruction,
    program::Program,
    symbols::{Symbol, SymbolTable, SymbolType},
};

pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
pub const PIE_HEADER_LENGTH: usize = 64;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum AssemblerPhase {
    #[default]
    First,
    Second,
}

/// <header> -> header prefix + read-only section len + padding: 64 bytes
/// <read-only data> -> store constants
/// <executable data>
#[derive(Debug, Default)]
pub struct Assembler {
    pub phase: AssemblerPhase,       // Tracks which phase the assember is in
    pub symbols: SymbolTable,        // Symbol table for constants and variables
    pub ro: Vec<u8>,                 // read-only data section constants are put in
    pub bytecode: Vec<u8>,           // compiled bytecode generated from the assembly instructions
    ro_offset: u32,                  // current offset of the read-only section
    sections: Vec<AssemblerSection>, // list of all the sections in the code
    curr_section: Option<AssemblerSection>, // current section the assembler is in
    curr_instruction: u32,           // current instruction the assembler is converting to bytecode
    errors: Vec<AssemblerError>,     // all errors
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            ro: Vec::new(),
            bytecode: Vec::new(),
            ro_offset: 0,
            sections: Vec::new(),
            curr_section: None,
            curr_instruction: 0,
            errors: Vec::new(),
        }
    }

    /// Convert a raw string to bytecode
    /// i.e. LOAD $0 $1
    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>> {
        match Program::parse(raw) {
            Ok((remainder, program)) => {
                assert_eq!(remainder, "");

                self.process_first_phase(&program);

                if !self.errors.is_empty() {
                    return Err(IridiumError::Assemble(self.errors.clone()));
                }

                if self.sections.len() != 2 {
                    self.errors.push(AssemblerError::InsufficientSections);
                    return Err(IridiumError::Assemble(self.errors.clone()));
                }

                let mut body = self.process_second_phase(&program);
                let mut assembled_program = self.write_pie_header();

                assembled_program.append(&mut body);
                Ok(assembled_program)
            }
            Err(e) => {
                eprintln!("There was an error parsing the code: {:?}", e);
                Err(IridiumError::Assemble(vec![AssemblerError::ParsingError]))
            }
        }
    }

    /// Extract program labels
    fn process_first_phase(&mut self, p: &Program) {
        for i in &p.instructions {
            if i.is_directive() {
                self.process_directive(i);
            }

            match self.curr_section {
                None => self.errors.push(AssemblerError::NoSegmentDeclarationFound(
                    self.curr_instruction,
                )),
                Some(_) => match i.is_label_declaration() {
                    true => self.process_label_declaration(i),
                    false => {} // process label usage
                },
            }
            self.curr_instruction += 1;
        }
        self.phase = AssemblerPhase::Second;
    }

    /// Extract program instruction bytes
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        self.curr_instruction = 0;
        let mut program = Vec::new();
        for i in &p.instructions {
            if i.is_opcode() {
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if i.is_directive() {
                self.process_directive(i);
            }
            self.curr_instruction += 1
        }
        program
    }

    /// Handles directives
    fn process_directive(&mut self, i: &AssemblerInstruction) {
        let directive_name = i.get_directive_name().unwrap();
        if i.contain_operands() {
            match directive_name.as_ref() {
                "asciiz" => {
                    self.handle_asciiz(i);
                }
                "integer" => {
                    // TODO: self.handle_integer(i);
                    todo!()
                }
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound(
                        directive_name.clone(),
                    ));
                }
            }
        } else {
            self.process_section_header(&directive_name);
        }
    }

    /// Handles the declaration of a label such as:
    /// hello: .asciiz 'Hello'
    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        let label_name = i.get_label_declaration_name().unwrap();
        if self.symbols.contain_symbol(&label_name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        let symbol = Symbol::new(label_name, SymbolType::Label);
        self.symbols.add_symbol(symbol);
    }

    /// Handles a declaration of a section header, such as:
    /// .code
    fn process_section_header(&mut self, header_name: &str) {
        let section = AssemblerSection::from(header_name);
        if section == AssemblerSection::Unknown {
            println!("Unknow section header encountered: {}", header_name);
            return;
        }
        self.sections.push(section.clone());
        self.curr_section = Some(section);
        assert_ne!(self.sections.len(), 0);
        assert_ne!(self.curr_section, None);
    }

    /// Handles a declaration of a null-terminated string:
    /// hello: .asciiz 'Hello!'
    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        if self.phase != AssemblerPhase::First {
            return;
        }

        if let Some(str) = i.get_string_constant() {
            if let Some(label_name) = i.get_label_declaration_name() {
                self.symbols.set_symbol_offset(&label_name, self.ro_offset);
            };

            for byte in str.as_bytes() {
                self.ro.push(*byte);
                self.ro_offset += 1;
            }

            // null-terminated string
            self.ro.push(0);
            self.ro_offset += 1;
        }
    }

    /// PIE_HEADER_PREFIX(4 bytes) + Read-Only(4 bytes) + padding
    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![0; PIE_HEADER_LENGTH];
        header[..PIE_HEADER_PREFIX.len()].clone_from_slice(&PIE_HEADER_PREFIX);

        let ro_len: Vec<u8> = (self.ro.len() as u32).to_le_bytes().to_vec();
        header[PIE_HEADER_PREFIX.len()..PIE_HEADER_PREFIX.len() + ro_len.len()]
            .clone_from_slice(&ro_len);

        header
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum AssemblerSection {
    Data(Option<u32>),
    Code(Option<u32>),
    #[default]
    Unknown,
}

impl<'a> From<&'a str> for AssemblerSection {
    fn from(name: &str) -> AssemblerSection {
        match name {
            "data" => AssemblerSection::Data(None),
            "code" => AssemblerSection::Code(None),
            _ => AssemblerSection::Unknown,
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
        assert_eq!(program.len(), 92);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 92);
    }

    #[test]
    fn test_code_start_offset_written() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest1: .asciiz 'Hello'\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        assert_eq!(program[4], 6);
    }
}

pub mod assem_instruction;
pub mod program;
pub mod symbols;
pub mod token;
