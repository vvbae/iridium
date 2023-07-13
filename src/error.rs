use std::{io, sync::mpsc};

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

#[derive(Error, Debug)]
pub enum IridiumError {
    /// IO error
    #[error("Io Error: {0}")]
    Io(#[from] io::Error),
    /// serialization or deserialization error
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Pipe send prompt/message error
    #[error("Pipe send Error: {0}")]
    Send(mpsc::SendError<String>),
    /// Pipe send prompt/message error
    #[error("Pipe receive Error: {0}")]
    Recv(#[from] mpsc::RecvError),
    /// Assemble error
    #[error("Assemble Error")]
    Assemble(Vec<AssemblerError>),
    /// Error with a string message
    #[error("{0}")]
    StringError(String),
}

impl From<mpsc::SendError<String>> for IridiumError {
    fn from(err: mpsc::SendError<String>) -> IridiumError {
        IridiumError::Send(err)
    }
}

impl From<Vec<AssemblerError>> for IridiumError {
    fn from(err: Vec<AssemblerError>) -> IridiumError {
        IridiumError::Assemble(err)
    }
}

pub type Result<T> = std::result::Result<T, IridiumError>;
