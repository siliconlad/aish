use crate::builtins::{builtin, is_builtin};
use std::error::Error;
use std::fmt::Display;
use std::process::{Child, ChildStdout, Command, Stdio};

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

    pub fn is_builtin(&self) -> bool {
        self.builtin
    }

    pub fn cmd(&self) -> &String {
        &self.tokens[0]
    }

    pub fn args(&self) -> Vec<String> {
        self.tokens[1..].to_vec()
    }

    pub fn run_builtin(&self) -> Result<(), Box<dyn Error>> {
        builtin(&self.tokens[0], self.tokens[1..].to_vec())?;
        Ok(())
    }

    pub fn run_cmd(&self, stdin: Option<ChildStdout>) -> Result<Child, Box<dyn Error>> {
        let input = match stdin {
            Some(input) => Stdio::from(input),
            None => Stdio::inherit(),
        };
        // Spawn the command
        let child = match Command::new(self.cmd())
            .args(self.args())
            .stdin(input)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };

        Ok(child)
    }

    // pub fn run(&self, stdin: Option<Stdio>) -> Result<(), Box<dyn Error>> {
    //     // Spawn the command
    //     if self.builtin {
    //         builtin(&self.tokens[0], self.tokens[1..].to_vec())?;
    //     }
    //     else {
    //         let child = match Command::new(self.cmd())
    //             .args(self.args())
    //             .stdin(stdin.unwrap_or(Stdio::inherit()))
    //             .stdout(Stdio::piped())
    //             .spawn()
    //         {
    //             Ok(child) => child,
    //             Err(e) => return Err(e.into()),
    //         };
    //     }

    //     Ok(())
    // }
}

impl Display for SimpleCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.cmd(), self.args().join(" "))
    }
}
