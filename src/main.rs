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
use std::collections::HashMap;

fn main() -> rustyline::Result<()> {
    // Setup logging
    let log_path = home_dir().unwrap().join(".aish_log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    WriteLogger::init(LevelFilter::Debug, Config::default(), log_file).unwrap();
    info!("Starting aish");

    let mut aliases = HashMap::new();

    // Get args
    let args: Vec<String> = env::args().collect();

    // Run aishrc file if it exists
    let aishrc = aishrc_path()?;
    if aishrc.exists() {
        match run_file_mode(&aishrc, &mut aliases) {
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
        1 => match interactive_mode(&mut aliases) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        },
        2 => match run_file_mode(&PathBuf::from(&args[1]), &mut aliases) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        },
        _ => eprintln!("Usage: aish [file]"),
    }

    info!("Exiting aish");
    Ok(())
}

fn interactive_mode(aliases: &mut HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
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

        let expanded_buffer = expand_aliases(aliases, &buffer);

        execute_commands(vec![expanded_buffer], aliases);
    }
    let _ = rl.save_history(history.as_path());
    Ok(())
}

fn run_file_mode(file_path: &PathBuf, aliases: &mut HashMap<String, String>) -> Result<(), std::io::Error> {
    let commands = read_file(file_path)?;
    execute_commands(commands, aliases);
    Ok(())
}

fn execute_commands(commands: Vec<String>, aliases: &mut HashMap<String, String>) {
    for command in commands {
        debug!("Executing command: {}", command);
        let tokenized = match parse(command) {
            Ok(tokenized) => tokenized,
            Err(e) => {
                eprintln!("Error in command: {}", e);
                continue;
            }
        };
        match tokenized.run(aliases) {
            Ok(s) => {
                if !s.is_empty() {
                    println!("{}", s)
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

fn expand_aliases(aliases: &HashMap<String, String>, buffer: &str) -> String {
    let commands: Vec<&str> = buffer.split(';').collect();
    let mut expanded_commands = Vec::new();
    let mut temp_aliases = aliases.clone();

    for command in commands {
        let and_commands: Vec<&str> = command.split("&&").collect();
        let mut expanded_and_commands = Vec::new();

        for and_command in and_commands {
            let pipe_commands: Vec<&str> = and_command.split('|').collect();
            let mut expanded_pipe_commands = Vec::new();

            for pipe_command in pipe_commands {
                let mut words: Vec<&str> = pipe_command.trim().split_whitespace().collect();

                if !words.is_empty() {
                    if words[0] == "alias" && words.len() > 1 {
                        // Handle alias definition
                        let full_command = words[1..].join(" ");
                        if let Some(equals_pos) = full_command.find('=') {
                            let (alias_name, alias_value) = full_command.split_at(equals_pos);
                            temp_aliases.insert(alias_name.to_string(), alias_value[1..].trim_matches('\'').to_string());
                        }
                        expanded_pipe_commands.push(pipe_command.trim().to_string());
                    } else {
                        // Expand aliases
                        let mut expansion_count = 0;
                        const MAX_EXPANSIONS: usize = 10;

                        while let Some(expansion) = temp_aliases.get(words[0]) {
                            let expanded_words: Vec<&str> = expansion.split_whitespace().collect();
                            if expanded_words.is_empty() {
                                break;
                            }
                            words.splice(0..1, expanded_words);
                            expansion_count += 1;
                            if expansion_count >= MAX_EXPANSIONS {
                                break;
                            }
                        }
                        expanded_pipe_commands.push(words.join(" "));
                    }
                }
            }

            expanded_and_commands.push(expanded_pipe_commands.join(" | "));
        }

        expanded_commands.push(expanded_and_commands.join(" && "));
    }

    expanded_commands.join("; ")
}
