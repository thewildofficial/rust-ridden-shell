#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    // handle user input here using read_line
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();

    // print message not found
    // xyz: command not found  
    println!("{}: command not found", command.trim());
}
