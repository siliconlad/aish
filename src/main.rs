pub mod builtins;
pub mod command;
pub mod pipeline;
pub mod tokenize;
pub mod traits;

#[macro_use] extern crate log;
extern crate simplelog;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::fs::OpenOptions;
use home::home_dir;
use simplelog::{Config, LevelFilter, WriteLogger};

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
    loop {
        let readline = rl.readline("> ");
        let mut buffer = match readline {
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
        let tokenized = match tokenize::tokenize(&mut buffer) {
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
    Ok(())
}
