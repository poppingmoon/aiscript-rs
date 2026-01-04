# aiscript-rs

[![CI](https://github.com/poppingmoon/aiscript-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/poppingmoon/aiscript-rs/actions/workflows/ci.yml)

Rust implementation of [AiScript](https://github.com/aiscript-dev/aiscript).

## Example

```rust
use aiscript::v1::{Interpreter, Parser};

let script = Parser::default().parse("<: 'Hello, world!'")?;
let interpreter = Interpreter::builder().out_sync(|v| println!("{v}")).build();
interpreter.exec(script).await?;
```
