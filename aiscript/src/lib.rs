//! Rust implementation of [AiScript](https://github.com/aiscript-dev/aiscript).
//!
//! # Example
//!
//! ```
//! use aiscript::v1::{Interpreter, Parser};
//! # use aiscript::v1::errors::AiScriptError;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), AiScriptError> {
//! let script = Parser::default().parse("<: 'Hello, world!'")?;
//! let interpreter = Interpreter::builder().out_sync(|v| println!("{v}")).build();
//! interpreter.exec(script).await?;
//! # Ok(())
//! # }
//! ```

pub use aiscript_v0 as v0;
pub use aiscript_v1 as v1;
