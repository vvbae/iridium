use nom::{error::context, multi::many1};

use crate::parse::Parse;

use super::instruction::AssemblerInstruction;

#[derive(Debug, PartialEq)]
pub struct Program {
    instructions: Vec<AssemblerInstruction>,
}

impl Program {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut program = Vec::new();
        for instruction in &self.instructions {
            program.append(&mut instruction.to_bytes());
        }
        program
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
        let (_, p) = Program::parse("load $0 #100\n").unwrap();

        assert_eq!(1, p.instructions.len());
    }

    #[test]
    fn test_program_to_bytes() {
        let (_, program) = Program::parse("load $0 #100\n").unwrap();
        let bytecode = program.to_bytes();
        assert_eq!(bytecode.len(), 4);
        println!("{:?}", bytecode);
    }
}
