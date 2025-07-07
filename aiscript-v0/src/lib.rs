//! Rust implementation of [AiScript](https://github.com/aiscript-dev/aiscript).
//!
//! # Example
//!
//! ```
//! use aiscript_v0::{Interpreter, Parser};
//! # use aiscript_v0::errors::AiScriptError;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), AiScriptError> {
//! let script = Parser::default().parse("<: 'Hello, world!'")?;
//! let interpreter = Interpreter::builder().out_sync(|v| println!("{v}")).build();
//! interpreter.exec(script).await?;
//! # Ok(())
//! # }
//! ```

mod constants;
mod error;
mod interpreter;
mod node;
mod parser;
mod r#type;

pub mod ast {
    pub use crate::node::*;
}

pub mod cst {
    pub use crate::parser::node::*;
}

pub mod errors {
    pub use crate::error::*;
}

pub mod utils {
    pub use crate::interpreter::util::*;
}

pub mod values {
    pub use crate::interpreter::value::*;
}

pub use constants::AISCRIPT_VERSION;
pub use interpreter::Interpreter;
pub use interpreter::scope::Scope;
pub use parser::{Parser, ParserPlugin, PluginType};
