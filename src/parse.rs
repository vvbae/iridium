use nom::IResult;

use crate::error::ParseError;

pub type ParseResult<'a, T> = IResult<&'a str, T, ParseError<'a>>;

pub trait Parse<'a>: Sized {
    /// Parse the given string into self
    fn parse(input: &'a str) -> ParseResult<'a, Self>;
}
