use nom::{error::context, multi::many1};

use crate::parse::Parse;

use super::{assem_instruction::AssemblerInstruction, symbols::SymbolTable};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>,
}

impl Program {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program = Vec::new();
        for instruction in &self.instructions {
            program.append(&mut instruction.to_bytes(symbols));
        }
        program
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
    }
}

impl<'a> Parse<'a> for Program {
    fn parse(input: &'a str) -> crate::parse::ParseResult<'a, Self> {
        let (remaining_input, instructions) =
            context("Program", many1(AssemblerInstruction::parse))(input)?;
        Ok((remaining_input, Program { instructions }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {
        let (_, p) = Program::parse("jmpe @test\nhlt").unwrap();

        assert_eq!(2, p.instructions.len());
    }

    #[test]
    fn test_program_to_bytes() {
        let (_, program) = Program::parse("load $0 #100\n").unwrap();
        let bytecode = program.to_bytes(&SymbolTable::new());
        assert_eq!(bytecode.len(), 4);
    }

    #[test]
    fn test_complete_program() {
        let (_, p) = Program::parse(".data\nhello: .asciiz 'Hello everyone!'\n.code\nhlt").unwrap();

        assert_eq!(4, p.instructions.len());
    }
}
