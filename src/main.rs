pub mod builtins;
pub mod command;
pub mod pipeline;
pub mod redirect;
pub mod sequence;
pub mod tokenize;
pub mod traits;

use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use home::home_dir;

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let history = home_dir().unwrap().join(".aish_history");
    let _ = rl.load_history(history.as_path());
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
        let tokenized = match tokenize::tokenize(&mut buffer) {
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
    let _ = rl.save_history(history.as_path());
    Ok(())
}
