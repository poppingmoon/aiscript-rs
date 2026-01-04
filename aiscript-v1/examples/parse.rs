use std::{fs::File, io::prelude::*};

use aiscript_v1::Parser;

#[tokio::main]
async fn main() {
    let mut file = File::open("main.ais").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    let script = Parser::default().parse(&s).unwrap();
    println!("{script:?}");
}
