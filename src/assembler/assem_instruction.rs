use std::error;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::{map, opt},
    error::context,
    sequence::{preceded, tuple},
};

use crate::parse::{self, Parse};

use super::{
    symbols::SymbolTable,
    token::{
        parse_directive, parse_int_operand, parse_label_declaration, parse_opcode, parse_register,
        Token,
    },
};

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    pub opcode: Option<Token>,
    pub label: Option<Token>,
    pub directive: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
}

impl AssemblerInstruction {
    /// Turn entire instruction to bytes
    pub fn to_bytes(&self, symbol_table: &SymbolTable) -> Vec<u8> {
        let mut results = Vec::new();
        match &self.opcode {
            Some(Token::Op { code }) => results.push(*code as u8),
            _ => {
                println!("Non-opcode found in opcode field");
                std::process::exit(1);
            }
        };

        for operand in &[&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results, symbol_table)
            }
        }

        while results.len() < 4 {
            results.push(0);
        }

        results
    }

    /// Turn a register to u8 or operand to u16
    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbol_table: &SymbolTable) {
        match t {
            Token::Register { reg_num } => results.push(*reg_num),
            Token::IntegerOperand { value } => {
                let converted = *value as i16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }
            Token::LabelUsage { name } => {
                if let Some(value) = symbol_table.symbol_value(name) {
                    let converted = value;
                    let byte1 = converted;
                    let byte2 = converted >> 8;
                    results.push(byte2 as u8);
                    results.push(byte1 as u8);
                } else {
                    eprintln!("No value found for {:?}", name);
                }
            }
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        }
    }

    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn label_name(&self) -> Option<String> {
        match &self.label {
            Some(l) => match l {
                Token::LabelDeclaration { name } => Some(name.clone()),
                _ => None,
            },
            None => None,
        }
    }
}

impl<'a> Parse<'a> for AssemblerInstruction {
    fn parse(input: &'a str) -> parse::ParseResult<'a, Self> {
        let (remaining_input, instruction) = context(
            "Instruction",
            alt((
                // <opcode> | <opcode> <tok1> | <opcode> <tok1> <tok2> | <opcode> <tok1> <tok2> <tok3> | <label> <opcode> <tok1> <tok2> <tok3>
                map(
                    tuple((
                        opt(parse_label_declaration),
                        multispace0,
                        parse_opcode,
                        opt(preceded(
                            multispace1,
                            alt((parse_register, parse_int_operand)),
                        )),
                        opt(preceded(
                            multispace1,
                            alt((parse_register, parse_int_operand)),
                        )),
                        opt(preceded(
                            multispace1,
                            alt((parse_register, parse_int_operand)),
                        )),
                        opt(tag("\n")),
                    )),
                    |(label, _, opcode, tok1, tok2, tok3, _)| AssemblerInstruction {
                        opcode: Some(opcode),
                        label,
                        directive: None,
                        operand1: tok1,
                        operand2: tok2,
                        operand3: tok3,
                    },
                ),
                // <directive> | <directive> <tok1> | <directive> <tok1> <tok2> | <directive> <tok1> <tok2> <tok3>
                map(
                    tuple((
                        parse_directive,
                        opt(preceded(
                            multispace1,
                            alt((parse_register, parse_int_operand)),
                        )),
                        opt(preceded(
                            multispace1,
                            alt((parse_register, parse_int_operand)),
                        )),
                        opt(preceded(
                            multispace1,
                            alt((parse_register, parse_int_operand)),
                        )),
                        opt(tag("\n")),
                    )),
                    |(directive, tok1, tok2, tok3, _)| AssemblerInstruction {
                        opcode: None,
                        label: None,
                        directive: Some(directive),
                        operand1: tok1,
                        operand2: tok2,
                        operand3: tok3,
                    },
                ),
            )),
        )(input)?;

        Ok((remaining_input, instruction))
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
            opcode: Some(Token::Op { code: Opcode::LOAD }),
            label: None,
            directive: None,
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: Some(Token::IntegerOperand { value: 100 }),
            operand3: None,
        };

        assert_eq!(expected, value);
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let (_, value) = AssemblerInstruction::parse("hlt\n").unwrap();
        let expected = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::HLT }),
            label: None,
            directive: None,
            operand1: None,
            operand2: None,
            operand3: None,
        };

        assert_eq!(expected, value);
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let (_, value) = AssemblerInstruction::parse("test: inc $0\n").unwrap();
        let expected = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::INC }),
            label: Some(Token::LabelDeclaration {
                name: "test".to_string(),
            }),
            directive: None,
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: None,
            operand3: None,
        };

        assert_eq!(expected, value);
    }

    #[test]
    fn test_parse_instruction_form_four() {
        let (_, value) = AssemblerInstruction::parse(".asciiz\n").unwrap();
        let expected = AssemblerInstruction {
            opcode: None,
            label: None,
            directive: Some(Token::Directive {
                name: "asciiz".to_string(),
            }),
            operand1: None,
            operand2: None,
            operand3: None,
        };

        assert_eq!(expected, value);
    }
}
