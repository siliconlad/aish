pub mod builtins;
pub mod command;
pub mod errors;
pub mod openai_client;
pub mod parsing;
pub mod pipeline;
pub mod redirect;
pub mod sequence;
pub mod suggestions;
pub mod token;
pub mod traits;

#[macro_use]
extern crate log;
extern crate simplelog;

use crate::suggestions::ShellHelper;
use crate::traits::Runnable;
use home::home_dir;
use parsing::parse;
use rustyline::error::ReadlineError;
use rustyline::Editor;
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

    let mut llm_output = String::new();

    // Run aishrc file if it exists
    let aishrc = aishrc_path()?;
    if aishrc.exists() {
        match run_file_mode(&aishrc, &mut llm_output) {
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
        1 => match interactive_mode(&mut llm_output) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        },
        2 => match run_file_mode(&PathBuf::from(&args[1]), &mut llm_output) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        },
        _ => eprintln!("Usage: aish [file]"),
    }

    info!("Exiting aish");
    Ok(())
}

fn interactive_mode(llm_output: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    // Setup readline
    let mut rl = Editor::<ShellHelper, rustyline::history::DefaultHistory>::new()?;
    let history = home_dir().unwrap().join(".aish_history");
    let _ = rl.load_history(history.as_path());

    let helper = ShellHelper {
        suggestion: String::new(),
    };
    rl.set_helper(Some(helper));

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                debug!("Added input to history");
                execute_commands(vec![line.to_string()], llm_output);

                if let Some(helper) = rl.helper_mut() {
                    helper.suggestion = llm_output.clone();
                }
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };
    }
    let _ = rl.save_history(history.as_path());
    Ok(())
}

fn run_file_mode(file_path: &PathBuf, llm_output: &mut String) -> Result<(), std::io::Error> {
    let commands = read_file(file_path)?;
    execute_commands(commands, llm_output);
    Ok(())
}

fn execute_commands(commands: Vec<String>, llm_output: &mut String) {
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
                if let Some(stripped) = s.strip_prefix("COMMAND: ") {
                    *llm_output = stripped.to_string();
                } else if !s.is_empty() {
                    println!("{}", s);
                    llm_output.clear();
                } else {
                    llm_output.clear();
                }
            }
            Err(e) => eprintln!("Error in command: {}", e),
        }
    }
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
