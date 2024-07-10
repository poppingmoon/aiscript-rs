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

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptRuntimeError {
    #[error("Runtime: {0}")]
    Runtime(String),
    #[error("Runtime: Index out of range. index: {0} max: {1}")]
    IndexOutOfRange(f64, isize),
    #[error("{0}")]
    User(String),
}
