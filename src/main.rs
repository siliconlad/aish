#[macro_use]
extern crate lazy_static;

pub mod builtins;
pub mod command;
pub mod pipeline;
pub mod tokenize;
pub mod traits;

use crate::builtins::expand_aliases;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline("> ");
        let buffer = match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                line
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        // Expand aliases
        let mut expanded_buffer = expand_aliases(&buffer);

        // Convert the input into a command
        let tokenized = match tokenize::tokenize(&mut expanded_buffer) {
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
    Ok(())
}
