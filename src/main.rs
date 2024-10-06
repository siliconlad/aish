pub mod builtins;
pub mod command;
pub mod errors;
pub mod openai_client;
pub mod parsing;
pub mod pipeline;
pub mod redirect;
pub mod sequence;
pub mod token;
pub mod traits;

#[macro_use]
extern crate log;
extern crate simplelog;

use crate::traits::Runnable;
use home::home_dir;
use parsing::parse;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::PathBuf;

fn main() -> Result<()> {
    // Setup logging
    let log_path = home_dir().unwrap().join(".aish_log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    WriteLogger::init(LevelFilter::Debug, Config::default(), log_file).unwrap();
    info!("Starting aish");

    // Read .aishrc file
    let aishrc_commands = read_aishrc()
        .map_err(|e| ReadlineError::Io(Error::new(ErrorKind::Other, e.to_string())))?;

    // Execute .aishrc commands
    for command in aishrc_commands {
        debug!("Executing .aishrc command: {}", command);
        let tokenized = match tokenize::tokenize(&mut command.clone()) {
            Ok(tokenized) => tokenized,
            Err(e) => {
                eprintln!("Error in .aishrc: {}", e);
                continue;
            }
        };
        match tokenized.run() {
            Ok(s) => {
                if !s.is_empty() {
                    println!("{}", s)
                }
            }
            Err(e) => eprintln!("Error in .aishrc: {}", e),
        }
    }

    // Setup readline
    let mut rl = DefaultEditor::new()?;
    let history = home_dir().unwrap().join(".aish_history");
    let _ = rl.load_history(history.as_path());
    loop {
        let readline = rl.readline("> ");
        let buffer = match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                debug!("Added input to history");
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
        let tokenized = match parse(buffer) {
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
    info!("Exiting aish");
    Ok(())
}

fn read_aishrc() -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
    let home_dir = home_dir()
        .ok_or_else(|| Box::<dyn std::error::Error>::from("Unable to determine home directory"))?;
    let aishrc_path: PathBuf = home_dir.join(".aishrc");

    if !aishrc_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(aishrc_path)?;
    let reader = BufReader::new(file);
    let commands: Vec<String> = reader
        .lines()
        .take_while(|r| r.is_ok())
        .filter_map(|r| r.ok())
        .collect();

    Ok(commands)
}
