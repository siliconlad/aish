pub mod tokenize;
pub mod builtins;

use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        let tokenized = tokenize::tokenize(&mut buffer);
        
        if tokenized.cmd() == "cd" {
            match builtins::cd(tokenized.args()) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        } else if tokenized.cmd() == "pwd" {
            match builtins::pwd() {
                Ok(pwd) => println!("{}", pwd),
                Err(e) => eprintln!("{}", e),
            }
        } else if tokenized.cmd() == "exit" {
            match builtins::exit() {
                Ok(_) => continue,
                Err(e) => eprintln!("{}", e),
            }
        } else {
            println!("{:?}", tokenized);
        }
    }
}
