pub mod builtins;
pub mod command;
pub mod pipeline;
pub mod tokenize;

use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        // Convert the input into a command
        let tokenized = match tokenize::tokenize(&mut buffer) {
            Ok(tokenized) => tokenized,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        // Run the command
        match tokenized.run() {
            Ok(s) => {
                if !s.is_empty() {
                    println!("{}", s)
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}
