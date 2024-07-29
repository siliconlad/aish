pub mod builtins;
pub mod tokenize;

use std::process::Command;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("> ");
        let mut buffer = match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                line
            },
            Err(ReadlineError::Interrupted) => {
                break
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        };

        // Tokenize input
        let tokenized = tokenize::tokenize(&mut buffer);
        
        // Run command
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
    Ok(())
}
