use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    character::complete::multispace0,
    combinator::map,
    error::context,
    sequence::{preceded, terminated},
};

use crate::{
    instruction::Opcode,
    parse::{Parse, ParseResult},
};

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
}

pub fn parse_opcode(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context(
        "Opcode",
        map(
            alt((tag_no_case("load"), tag_no_case("add"))),
            |op| match op {
                "load" => Token::Op { code: Opcode::LOAD },
                "add" => Token::Op { code: Opcode::ADD },
                _ => unimplemented!(),
            },
        ),
    )(input)?;

    Ok((remaining, token))
}

pub fn parse_register(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context(
        "Register",
        preceded(tag("$"), take_while(|c: char| c.is_numeric())),
    )(input)?;

    Ok((
        remaining,
        Token::Register {
            reg_num: token.parse::<u8>().unwrap(),
        },
    ))
}

pub fn parse_int_operand(input: &str) -> ParseResult<'_, Token> {
    let (remaining, token) = context(
        "Integer Operand",
        preceded(tag("#"), take_while(|c: char| c.is_numeric())),
    )(input)?;

    Ok((
        remaining,
        Token::IntegerOperand {
            value: token.parse::<i32>().unwrap(),
        },
    ))
}

impl<'a> Parse<'a> for Token {
    fn parse(input: &'a str) -> ParseResult<'a, Self> {
        context(
            "Token",
            preceded(
                multispace0,
                terminated(
                    alt((parse_opcode, parse_register, parse_int_operand)),
                    multispace0,
                ),
            ),
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_opcode() {
        let expected = Token::Op { code: Opcode::LOAD };

        let (_, value) = Token::parse("load ").unwrap();

        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_register() {
        let expected = Token::Register { reg_num: 12 };

        let (_, value) = Token::parse(" $12 ").unwrap();

        assert_eq!(value, expected);
    }

    #[test]
    fn test_parse_int_operand() {
        let expected = Token::IntegerOperand { value: 54 };

        let (_, value) = Token::parse("  #54 ").unwrap();

        assert_eq!(value, expected);
    }
}
