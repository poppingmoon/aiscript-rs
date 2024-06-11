use std::{
    fs::File,
    io::{prelude::*, stdin, stdout},
};

use aiscript::{values::Value, Interpreter, Parser};
use futures::FutureExt;

#[tokio::main]
async fn main() {
    let mut file = File::open("test.is").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let script = Parser::default().parse(&s).unwrap();
    let aiscript = Interpreter::new(
        [],
        Some(|q| {
            print!("{q}");
            stdout().flush().unwrap();
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            async move { buf }.boxed()
        }),
        Some(|v: Value| {
            println!("{}", v.value.repr_value());
            async move {}.boxed()
        }),
        Some(|e| {
            eprintln!("{e}");
            async move {}.boxed()
        }),
        None,
    );
    println!(
        "{}",
        aiscript
            .exec(script)
            .await
            .unwrap()
            .unwrap_or_default()
            .value
            .repr_value()
    );
}
