//! Rust implementation of [AiScript](https://github.com/aiscript-dev/aiscript).
//!
//! # Example
//!
//! ```
//! use aiscript_v1::{Interpreter, Parser};
//! # use aiscript_v1::errors::AiScriptError;
//! use futures::FutureExt;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), AiScriptError> {
//! let script = Parser::default().parse("<: 'Hello, world!'")?;
//! let interpreter = Interpreter::new(
//!     [],
//!     None::<fn(_) -> _>,
//!     Some(|v| {
//!         println!("{v}");
//!         async move {}.boxed()
//!     }),
//!     None::<fn(_) -> _>,
//!     None,
//! );
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

pub mod errors {
    pub use crate::error::*;
}

pub mod utils {
    pub use crate::interpreter::util::*;
}

pub mod values {
    pub use crate::interpreter::value::*;
}

pub use self::{
    constants::AISCRIPT_VERSION,
    interpreter::Interpreter,
    parser::{Parser, ParserPlugin, PluginType},
};
