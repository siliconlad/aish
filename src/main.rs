pub mod builtins;
pub mod command;
pub mod errors;
pub mod openai_client;
pub mod parser;
pub mod pipeline;
pub mod redirect;
pub mod sequence;
pub mod token;
pub mod traits;

#[macro_use]
extern crate log;
extern crate simplelog;

use home::home_dir;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::OpenOptions;

fn main() -> Result<()> {
    // Setup logging
    let log_path = home_dir().unwrap().join(".aish_log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    WriteLogger::init(LevelFilter::Debug, Config::default(), log_file).unwrap();

    // Setup readline
    let mut rl = DefaultEditor::new()?;
    let history = home_dir().unwrap().join(".aish_history");
    let _ = rl.load_history(history.as_path());
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

        // Convert the input into a command
        debug!("Tokenizing...");
        let tokenized = match parser::tokenize(buffer) {
            Ok(tokenized) => tokenized,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        debug!("Finished tokenizing...running command(s)");

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
    let _ = rl.save_history(history.as_path());
    Ok(())
}
