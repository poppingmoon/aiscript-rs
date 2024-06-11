# aiscript-rs

[![CI](https://github.com/poppingmoon/aiscript-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/poppingmoon/aiscript-rs/actions/workflows/ci.yml)

Rust implementation of [AiScript](https://github.com/aiscript-dev/aiscript) (Experimental)

## Example

```rust
use aiscript::{Interpreter, Parser};
use futures::FutureExt;

let script = Parser::default().parse("<: 'Hello, world!'")?;
let interpreter = Interpreter::new(
    [],
    None::<fn(_) -> _>,
    Some(|v| {
        println!("{v}");
        async move {}.boxed()
    }),
    None::<fn(_) -> _>,
    None,
);
interpreter.exec(script).await?;
```
