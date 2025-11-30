use std::io::{prelude::*, stdin, stdout};

use aiscript_v0::{
    Interpreter, Parser,
    values::{V, Value},
};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let parser = Parser::default();
    let aiscript = Interpreter::builder()
        .in_sync(|q| {
            print!("{q}");
            stdout().flush().unwrap();
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            buf
        })
        .out_sync(|v| println!("{}", v.value.repr_value()))
        .err_sync(|e| eprintln!("[Error] {e}"))
        .build();
    let mut input = String::new();
    println!("Welcome to AiScript!");
    println!("https://github.com/aiscript-dev/aiscript");
    println!();
    println!("Type 'exit' to end this session.");
    loop {
        let readline = rl.readline(if input.is_empty() { "> " } else { ". " });
        match readline {
            Ok(line) => {
                if !input.is_empty() {
                    input += "\n";
                }
                input += &line;
                if input == "exit" {
                    println!("Bye.");
                    break;
                }
                let script = parser.parse(&input);
                match script {
                    Ok(script) => {
                        rl.add_history_entry(&input)?;
                        input.clear();
                        let result = aiscript.exec(script).await.unwrap();
                        if let Some(Value { value, .. }) = result
                            && *value != V::Null
                        {
                            println!("{}", value.repr_value());
                        }
                    }
                    Err(err) => {
                        if line.trim().is_empty() {
                            rl.add_history_entry(&input)?;
                            input.clear();
                            eprintln!("Error: {err:?}");
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                input.clear();
                println!("Interrupted.");
            }
            Err(ReadlineError::Eof) => {
                println!("Bye.");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }
    Ok(())
}
