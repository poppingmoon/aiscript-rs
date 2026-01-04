use thiserror::Error;

use crate::node::Pos;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptError {
    #[error("Internal: {0}")]
    Internal(String),
    #[error("Syntax: {0}")]
    Syntax(#[from] AiScriptSyntaxError),
    #[error("Namespace: {0}")]
    Namespace(#[from] AiScriptNamespaceError),
    #[error("Runtime: {0}")]
    Runtime(#[from] AiScriptRuntimeError),
}

impl AiScriptError {
    pub fn internal(message: impl std::fmt::Display) -> Self {
        AiScriptError::Internal(message.to_string())
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
#[error("{} (Line {}, Column {})", kind, pos.line, pos.column)]
pub struct AiScriptSyntaxError {
    pub kind: AiScriptSyntaxErrorKind,
    pub pos: Pos,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptSyntaxErrorKind {
    #[error("break corresponding to statement cannot include value")]
    BreakToStatementWithValue,
    #[error("continue must be inside for / each / while / do-while / loop")]
    ContinueOutsideLoop,
    #[error("Key {0} is duplicated.")]
    DuplicateKey(String),
    #[error("type parameter name {0} is duplicate")]
    DuplicateTypeParameterName(String),
    #[error("invalid attribute.")]
    InvalidAttribute,
    #[error("invalid character: {0}")]
    InvalidCharacter(char),
    #[error("cannot use label for expression other than eval / if / match")]
    InvalidExpressionWithLabel,
    #[error(transparent)]
    InvalidNumber(#[from] std::num::ParseFloatError),
    #[error(
        "cannot use label for statement other than eval / if / match / for / each / while / \
        do-while / loop"
    )]
    InvalidStatementWithLabel,
    #[error("Multiple statements cannot be placed on a single line.")]
    MultipleStatements,
    #[error(r#"Reserved word "{0}" cannot be used as variable name."#)]
    ReservedWord(String),
    #[error("return must be inside function")]
    ReturnOutsideFunction,
    #[error("separator expected")]
    SeparatorExpected,
    #[error("cannot use spaces in a label")]
    SpaceInLabel,
    #[error("Cannot use spaces in a reference.")]
    SpaceInReference,
    #[error(r#"label "{0}" is not defined"#)]
    UndefinedLabel(String),
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Unknown type: '{0}'")]
    UnknownType(String),
    #[error("unlabeled break must be inside for / each / while / do-while / loop")]
    UnlabeledBreakOutsideLoop,
    #[error("cannot omit label from break if expression is given")]
    UnlabeledBreakWithExpression,
}

#[derive(Error, Debug, PartialEq, Clone)]
#[error("{} (Line {}, Column {})", kind, pos.line, pos.column)]
pub struct AiScriptNamespaceError {
    pub kind: AiScriptNamespaceErrorKind,
    pub pos: Pos,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptNamespaceErrorKind {
    #[error("Destructuring assignment is invalid in namespace declarations.")]
    DestructuringAssignment,
    #[error(r#"No "var" in namespace declaration: {0}"#)]
    Mutable(String),
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum AiScriptRuntimeError {
    #[error("Cannot assign to an immutable variable {0}.")]
    AssignmentToImmutable(String),
    #[error("Expect anything, but got nothing.")]
    ExpectAny,
    #[error("Index out of range. index: {index} max: {max}")]
    IndexOutOfRange { index: f64, max: isize },
    #[error(
        "The left-hand side of an assignment expression must be a variable or a property/index \
        access."
    )]
    InvalidAssignment,
    #[error("The left-hand side of an definition expression must be a variable.")]
    InvalidDefinition,
    #[error("Cannot read prop of {target_type}. (reading {name})")]
    InvalidPrimitiveProperty { name: String, target_type: String },
    #[error("Cannot read prop ({name}) of {target_type}.")]
    InvalidProperty { name: String, target_type: String },
    #[error("`seed` must be either number or string.")]
    InvalidSeed,
    #[error("max step exceeded")]
    MaxStepExceeded,
    #[error("No such prop ({name}) in {target_type}.")]
    NoSuchProperty { name: String, target_type: String },
    #[error("No such variable '{name}' in scope '{scope_name}'")]
    NoSuchVariable { name: String, scope_name: String },
    #[error("Reduce of empty array without initial value")]
    ReduceWithoutInitialValue,
    #[error("{0}")]
    Runtime(String),
    #[error("{0} expected non-negative number, got negative")]
    UnexpectedNegative(String),
    #[error("{0} expected integer, got non-integer")]
    UnexpectedNonInteger(String),
    #[error("Expect {expected}, but got {actual}.")]
    TypeMismatch { expected: String, actual: String },
    #[error("{0}")]
    User(String),
    #[error("Variable '{name}' already exists in scope '{scope_name}'")]
    VariableAlreadyExists { name: String, scope_name: String },
}
