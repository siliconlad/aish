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
use rustyline::DefaultEditor;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::result::Result;

fn main() -> rustyline::Result<()> {
    // Setup logging
    let log_path = home_dir().unwrap().join(".aish_log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    WriteLogger::init(LevelFilter::Debug, Config::default(), log_file).unwrap();
    info!("Starting aish");

    // Get args
    let args: Vec<String> = env::args().collect();

    // Run aishrc file if it exists
    let aishrc = aishrc_path()?;
    if aishrc.exists() {
        match run_file_mode(&aishrc) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Ok(());
            }
        }
    } else {
        warn!("No .aishrc file found");
    }

    // Run in interactive mode if no args
    match args.len() {
        1 => match interactive_mode() {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        },
        2 => match run_file_mode(&PathBuf::from(&args[1])) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        },
        _ => eprintln!("Usage: aish [file]"),
    }

    info!("Exiting aish");
    Ok(())
}

fn interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Setup readline
    let mut rl = DefaultEditor::new()?;
    let history = home_dir().unwrap().join(".aish_history");
    let _ = rl.load_history(history.as_path());
    let mut previous_output = String::new();

    loop {
        let readline = if previous_output.is_empty() {
            rl.readline("> ")
        } else {
            rl.readline_with_initial("> ", (&previous_output, ""))
        };

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

        debug!("4321 Going to execute commands");
        match execute_commands(vec![buffer]) {
            Ok(output) => {
                debug!("Received output: {}", output);
                if output.starts_with("COMMAND: ") {
                    previous_output = output[9..].to_string();
                } else {
                    println!("output: {}", output);
                    previous_output.clear();
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                previous_output.clear();
            }
        }
    }
    let _ = rl.save_history(history.as_path());
    Ok(())
}

fn run_file_mode(file_path: &PathBuf) -> Result<(), std::io::Error> {
    let commands = read_file(file_path)?;
    debug!("1234 Going to execute commands");
    let _ = execute_commands(commands)?;
    Ok(())
}

fn execute_commands(commands: Vec<String>) -> Result<String, std::io::Error> {
    debug!("Number of commands: {}", commands.len());
    for command in commands {
        debug!("Executing command: {}", command);
        let tokenized = match parse(command) {
            Ok(tokenized) => tokenized,
            Err(e) => {
                eprintln!("Error in command: {}", e);
                continue;
            }
        };
        debug!("tokenized: {:?}", tokenized);
        match tokenized.run() {
            Ok(s) => {
                if !s.is_empty() {
                    return Ok(s)
                }
            }
            Err(e) => eprintln!("Error in command: {}", e),
        }
    }
    Ok("".to_string())
}

fn read_file(file_path: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let commands: Vec<String> = reader
        .lines()
        .take_while(|r| r.is_ok())
        .filter_map(|r| r.ok())
        .collect();

    Ok(commands)
}

fn aishrc_path() -> Result<PathBuf, std::io::Error> {
    let home = home_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Unable to determine home directory",
    ))?;
    let aishrc_path: PathBuf = home.join(".aishrc");
    Ok(aishrc_path)
}
