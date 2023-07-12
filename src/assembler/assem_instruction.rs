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
        parse_directive, parse_int_operand, parse_label_declaration, parse_label_usage,
        parse_opcode, parse_register, parse_str_operand, Token,
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
    /// Convert entire instruction to bytes
    pub fn to_bytes(&self, symbol_table: &SymbolTable) -> Vec<u8> {
        let mut results = Vec::new();
        match &self.opcode {
            Some(Token::Op { code }) => results.push(*code as u8),
            _ => {
                println!("Non-opcode found in opcode field");
                std::process::exit(1);
            }
        };

        for token in [&self.operand1, &self.operand2, &self.operand3]
            .iter()
            .copied()
            .flatten()
        {
            AssemblerInstruction::extract_operand(token, &mut results, symbol_table)
        }

        while results.len() < 4 {
            results.push(0);
        }

        results
    }

    /// Convert a register, operand, label to u8
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

    /// If this instruction contains any operands
    pub fn contain_operands(&self) -> bool {
        self.operand1.is_some() || self.operand2.is_some() || self.operand3.is_some()
    }

    /// If this instruction contains a label declaration
    pub fn is_label_declaration(&self) -> bool {
        matches!(&self.label, Some(Token::LabelDeclaration { name: _ }))
    }

    /// If this instruction contains a label usage
    pub fn is_label_usage(&self) -> bool {
        matches!(&self.label, Some(Token::LabelUsage { name: _ }))
    }

    /// If this instruction contains a directive
    pub fn is_directive(&self) -> bool {
        self.directive.is_some()
    }

    /// If this instruction contains an opcode
    pub fn is_opcode(&self) -> bool {
        self.opcode.is_some()
    }

    /// If contained label declaration, return label; Else None
    pub fn get_label_declaration_name(&self) -> Option<String> {
        assert!(self.label.is_some());
        self.label.as_ref().and_then(|tok| match tok {
            Token::LabelDeclaration { name } => Some(name.to_owned()),
            _ => None,
        })
    }

    /// If contained label usage, return label; Else None
    pub fn get_label_usage_name(&self) -> Option<String> {
        assert!(self.label.is_some());
        self.label.as_ref().and_then(|tok| match tok {
            Token::LabelUsage { name } => Some(name.to_owned()),
            _ => None,
        })
    }

    /// If contained directive, return name; Else None
    pub fn get_directive_name(&self) -> Option<String> {
        assert!(self.directive.is_some());
        self.directive.as_ref().and_then(|tok| match tok {
            Token::Directive { name } => Some(name.to_owned()),
            _ => None,
        })
    }

    /// If contained string constant, return string; Else None
    pub fn get_string_constant(&self) -> Option<String> {
        assert!(self.operand1.is_some());
        self.operand1.as_ref().and_then(|tok| match tok {
            Token::StringOperand { value } => Some(value.to_owned()),
            _ => None,
        })
    }
}

impl<'a> Parse<'a> for AssemblerInstruction {
    fn parse(input: &'a str) -> parse::ParseResult<'a, Self> {
        let (remaining_input, instruction) = context(
            "Instruction",
            alt((
                // <opcode> <label_usage>
                map(
                    tuple((parse_opcode, multispace1, parse_label_usage, opt(tag("\n")))),
                    |(opcode, _, label, _)| AssemblerInstruction {
                        opcode: Some(opcode),
                        label: Some(label),
                        directive: None,
                        operand1: None,
                        operand2: None,
                        operand3: None,
                    },
                ),
                // <label_decl> <directive> <tok1>
                map(
                    tuple((
                        parse_label_declaration,
                        preceded(multispace1, parse_directive),
                        preceded(multispace1, parse_str_operand),
                        opt(tag("\n")),
                    )),
                    |(label, directive, tok, _)| AssemblerInstruction {
                        opcode: None,
                        label: Some(label),
                        directive: Some(directive),
                        operand1: Some(tok),
                        operand2: None,
                        operand3: None,
                    },
                ),
                // [label_decl] <opcode> [tok1] [tok2] [tok3]
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
                // <directive> [tok1] [tok2] [tok3]
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

    #[test]
    fn test_parse_instruction_form_five() {
        let (_, value) = AssemblerInstruction::parse("jmpe @test\n").unwrap();
        let expected = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::JMPE }),
            label: Some(Token::LabelUsage {
                name: "test".to_string(),
            }),
            directive: None,
            operand1: None,
            operand2: None,
            operand3: None,
        };

        assert_eq!(expected, value);
    }

    #[test]
    fn test_string_directive() {
        let (_, value) = AssemblerInstruction::parse("test: .asciiz 'Hello'\n").unwrap();
        let expected = AssemblerInstruction {
            opcode: None,
            label: Some(Token::LabelDeclaration {
                name: "test".to_string(),
            }),
            directive: Some(Token::Directive {
                name: "asciiz".to_string(),
            }),
            operand1: Some(Token::StringOperand {
                value: "Hello".to_string(),
            }),
            operand2: None,
            operand3: None,
        };

        assert_eq!(expected, value);
    }
}
