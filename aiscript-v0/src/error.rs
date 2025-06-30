use peg::{error::ParseError, str::LineCol};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptError {
    #[error("Internal: {0}")]
    Internal(String),
    #[error("Syntax: {0}")]
    Syntax(#[from] AiScriptSyntaxError),
    // Type,
    #[error(transparent)]
    Runtime(#[from] AiScriptRuntimeError),
}

impl AiScriptError {
    pub fn internal(message: impl std::fmt::Display) -> Self {
        AiScriptError::Internal(message.to_string())
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptSyntaxError {
    #[error("Parsing error. (Line {}:{})", .0.location.line, .0.location.column)]
    Parse(#[from] ParseError<LineCol>),
    #[error("invalid attribute.")]
    Attribute,
    #[error(r#"Reserved word "{0}" cannot be used as variable name."#)]
    ReservedWord(String),
    #[error("Unknown type: '{0}'")]
    UnknownType(String),
}

impl AiScriptSyntaxError {
    pub fn parse(error: ParseError<LineCol>) -> Self {
        AiScriptSyntaxError::Parse(error)
    }

    pub fn attribute() -> Self {
        AiScriptSyntaxError::Attribute
    }

    pub fn reserved_word(name: impl std::fmt::Display) -> Self {
        AiScriptSyntaxError::ReservedWord(name.to_string())
    }

    pub fn unknown_type(name: impl std::fmt::Display) -> Self {
        AiScriptSyntaxError::UnknownType(name.to_string())
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptRuntimeError {
    #[error("Runtime: {0}")]
    Runtime(String),
    #[error("Runtime: Index out of range. index: {0} max: {1}")]
    IndexOutOfRange(f64, isize),
    #[error("{0}")]
    User(String),
}

impl AiScriptRuntimeError {
    pub fn runtime(message: impl std::fmt::Display) -> Self {
        AiScriptRuntimeError::Runtime(message.to_string())
    }

    pub fn index_out_of_range(index: impl Into<f64>, max: impl Into<isize>) -> Self {
        AiScriptRuntimeError::IndexOutOfRange(index.into(), max.into())
    }

    pub fn user(message: impl std::fmt::Display) -> Self {
        AiScriptRuntimeError::User(message.to_string())
    }
}
