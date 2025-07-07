use std::{
    fs::File,
    io::{prelude::*, stdin, stdout},
};

use aiscript_v0::{Interpreter, Parser};

#[tokio::main]
async fn main() {
    let mut file = File::open("test.is").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let script = Parser::default().parse(&s).unwrap();
    let aiscript = Interpreter::builder()
        .in_sync(|q| {
            print!("{q}");
            stdout().flush().unwrap();
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            buf
        })
        .out_sync(|v| println!("{}", v.value.repr_value()))
        .err_sync(|e| eprintln!("{e}"))
        .build();
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
