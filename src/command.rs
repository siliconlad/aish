use std::error::Error;
use std::fmt::Display;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

use crate::builtins::{builtin, is_builtin};

#[derive(Debug)]
pub struct SimpleCommand {
    builtin: bool,
    tokens: Vec<String>,
}

impl SimpleCommand {
    pub fn new(tokens: Vec<String>) -> Result<SimpleCommand, Box<dyn Error>> {
        if tokens.is_empty() {
            return Err("Tokens cannot be empty".into());
        }
        Ok(SimpleCommand {
            builtin: is_builtin(&tokens[0]),
            tokens,
        })
    }

    pub fn cmd(&self) -> &String {
        &self.tokens[0]
    }

    pub fn args(&self) -> Vec<String> {
        self.tokens[1..].to_vec()
    }

    pub fn run(&self, stdin: Option<Stdio>) -> Result<Child, Box<dyn Error>> {
        // Spawn the command
        let child = match Command::new(self.cmd())
            .args(self.args())
            .stdin(stdin.unwrap_or(Stdio::inherit()))
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };

        Ok(child)
    }
}

impl Display for SimpleCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.cmd(), self.args().join(" "))
    }
}
