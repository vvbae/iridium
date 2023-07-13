use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, digit1},
    combinator::{map, opt},
    error::context,
    sequence::{preceded, terminated, tuple},
};

use crate::{instruction::Opcode, parse::ParseResult};

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    FloatOperand { value: f64 },
    StringOperand { value: String },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
}

pub fn parse_str_operand(input: &str) -> ParseResult<'_, Token> {
    let (remaining, (_, token, _)) = context(
        "String Operand",
        tuple((tag("'"), take_until("'"), tag("'"))),
    )(input)?;

    Ok((
        remaining,
        Token::StringOperand {
            value: token.to_string(),
        },
    ))
}

pub fn parse_label_declaration(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) =
        context("Label Declaration", terminated(alphanumeric1, tag(":")))(input)?;

    Ok((
        remaining,
        Token::LabelDeclaration {
            name: token.to_string(),
        },
    ))
}

pub fn parse_label_usage(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context("Label Usage", preceded(tag("@"), alphanumeric1))(input)?;

    Ok((
        remaining,
        Token::LabelUsage {
            name: token.to_string(),
        },
    ))
}

pub fn parse_directive(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context("Directive", preceded(tag("."), alpha1))(input)?;

    Ok((
        remaining,
        Token::Directive {
            name: token.to_string(),
        },
    ))
}

pub fn parse_opcode(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) =
        context("Opcode", map(alphanumeric1, |op: &str| op.to_lowercase()))(input)?;

    Ok((
        remaining,
        Token::Op {
            code: Opcode::from(token.as_str()),
        },
    ))
}

pub fn parse_register(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context("Register", preceded(tag("$"), digit1))(input)?;

    Ok((
        remaining,
        Token::Register {
            reg_num: token.parse::<u8>().unwrap(),
        },
    ))
}

pub fn parse_int_operand(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context("Integer Operand", preceded(tag("#"), digit1))(input)?;

    Ok((
        remaining,
        Token::IntegerOperand {
            value: token.parse::<i32>().unwrap(),
        },
    ))
}

pub fn parse_float_operand(input: &str) -> ParseResult<'_, Token> {
    let (remaining, value) = context(
        "Float Operand",
        map(
            tuple((opt(tag("-")), digit1, tag("."), digit1)),
            |(sign, left, _, right)| {
                let value = format!("{}.{}", left, right).parse::<f64>().unwrap();
                match sign {
                    Some(_) => -1.0 * value,
                    None => value,
                }
            },
        ),
    )(input)?;

    Ok((remaining, Token::FloatOperand { value }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opcode() {
        let expected = Token::Op { code: Opcode::LOAD };

        let (_, value) = parse_opcode("load $1 #54").unwrap();

        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_register() {
        let expected = Token::Register { reg_num: 12 };

        let (_, value) = parse_register("$12 ").unwrap();

        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_int_operand() {
        let expected = Token::IntegerOperand { value: 54 };

        let (_, value) = parse_int_operand("#54").unwrap();

        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_label_declaration() {
        let expected = Token::LabelDeclaration {
            name: "label".to_string(),
        };

        let (_, value) = parse_label_declaration("label:").unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_label_usage() {
        let expected = Token::LabelUsage {
            name: "label".to_string(),
        };

        let (_, value) = parse_label_usage("@label").unwrap();
        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_directive() {
        let expected = Token::Directive {
            name: "asciiz".to_string(),
        };

        let (_, value) = parse_directive(".asciiz\n").unwrap();

        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_str_operand() {
        let expected = Token::StringOperand {
            value: "hello".to_string(),
        };

        let (_, value) = parse_str_operand("'hello'").unwrap();

        assert_eq!(value, expected);
    }
}
