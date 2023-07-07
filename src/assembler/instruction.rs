use nom::{character::complete::multispace1, error::context, sequence::tuple};

use crate::parse::{self, Parse};

use super::token::{parse_int_operand, parse_opcode, parse_register, Token};

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcode: Token,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

impl AssemblerInstruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut results = Vec::new();
        match &self.opcode {
            Token::Op { code } => results.push(*code as u8),
            _ => {
                println!("Non-opcode found in opcode field");
                std::process::exit(1);
            }
        };

        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            match operand {
                Some(t) => AssemblerInstruction::extract_operand(t, &mut results),
                None => {}
            }
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>) {
        match t {
            Token::Register { reg_num } => results.push(*reg_num),
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        }
    }
}

impl<'a> Parse<'a> for AssemblerInstruction {
    fn parse(input: &'a str) -> parse::ParseResult<'a, Self> {
        let (remaining_input, (opcode, _, reg, _, i)) = context(
            "Instruction",
            tuple((
                parse_opcode,
                multispace1,
                parse_register,
                multispace1,
                parse_int_operand,
            )),
        )(input)?;

        Ok((
            remaining_input,
            AssemblerInstruction {
                opcode,
                operand1: Some(reg),
                operand2: Some(i),
                operand3: None,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::Opcode;

    use super::*;

    #[test]
    fn test_parse_instruction_form_one() {
        let (_, value) = AssemblerInstruction::parse("load $0 #100\n").unwrap();
        let expected = AssemblerInstruction {
            opcode: Token::Op { code: Opcode::LOAD },
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: Some(Token::IntegerOperand { value: 100 }),
            operand3: None,
        };

        assert_eq!(expected, value);
    }
}
