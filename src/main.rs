pub mod builtins;
pub mod command;
pub mod openai_client;
pub mod pipeline;
pub mod redirect;
pub mod sequence;
pub mod token;
pub mod tokenize;
pub mod traits;

#[macro_use]
extern crate log;
extern crate simplelog;

use home::home_dir;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::OpenOptions;
use std::collections::HashMap;

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

    let mut aliases = HashMap::new();

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

        buffer = expand_aliases(&aliases, &buffer);

        debug!("Buffer: {}", buffer);

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
        match tokenized.run(&mut aliases) {
            Ok(s) => {
                if !s.is_empty() {
                    println!("{}", s)
                }
            }
            Err(e) => eprintln!("{}", e),
        }
        debug!("Line {}: Aliases: {:?}", line!(), aliases);
    }
    let _ = rl.save_history(history.as_path());
    Ok(())
}

fn expand_aliases(aliases: &HashMap<String, String>, buffer: &str) -> String {
    let mut buffer = buffer.to_string();
    debug!("Buffer at the start of alias expansion: {}", buffer);
    if buffer.starts_with("alias ") {
        // If setting an alias, don't expand existing aliases
        let parts: Vec<&str> = buffer.splitn(2, '=').collect();
        if parts.len() == 2 {
            let alias_name = parts[0].trim().split_whitespace().nth(1).unwrap_or("");
            if aliases.contains_key(alias_name) {
                return buffer;
            }
        }
    }
    debug!("Line {}: Aliases: {:?}", line!(), aliases);
    for (alias, expansion) in aliases {
        debug!("Expanding alias: {} -> {}", alias, expansion);
        buffer = buffer.replace(alias, expansion);
    }
    buffer
}
