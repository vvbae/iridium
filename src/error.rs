use nom_supreme::error::ErrorTree;
use thiserror::Error;

pub type ParseError<'a> = ErrorTree<&'a str>;

#[derive(Debug, Error, Clone)]
pub enum AssemblerError {
    #[error("Insufficient sections")]
    InsufficientSections,
    #[error("Error from parsing")]
    ParsingError,
    #[error("Label found outside segment at: {0}")]
    NoSegmentDeclarationFound(u32),
    #[error("String declared without label at: {0}")]
    StringConstantDeclaredWithoutLabel(u32),
    #[error("Symbol already declared")]
    SymbolAlreadyDeclared,
    #[error("Unknown directive: {0}")]
    UnknownDirectiveFound(String),
}
