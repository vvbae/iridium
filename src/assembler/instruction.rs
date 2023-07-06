use nom::{
    bytes::complete::tag_no_case, character::complete::multispace1, error::context, sequence::tuple,
};

use crate::parse::{self, Parse};

use super::token::{parse_int_operand, parse_opcode, parse_register, Token};

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcode: Token,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

impl<'a> Parse<'a> for AssemblerInstruction {
    fn parse(input: &'a str) -> parse::ParseResult<'a, Self> {
        let (remaining_input, (opcode, _, reg, _, i, _)) = context(
            "Instruction",
            tuple((
                parse_opcode,
                multispace1,
                parse_register,
                multispace1,
                parse_int_operand,
                tag_no_case("\n"),
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
