pub mod builtins;
pub mod tokenize;

use std::io::{self, Write};
use std::process::Command;

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
        } else if tokenized.cmd() == "echo" {
            match builtins::echo(tokenized.args()) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        } else {
            // Spawn the command
            let mut child = match Command::new(tokenized.cmd()).args(tokenized.args()).spawn() {
                Ok(child) => child,
                Err(_) => {
                    eprintln!("Failed to execute command");
                    continue;
                }
            };

            // Wait for the command to finish
            match child.wait() {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        }
    }
}
