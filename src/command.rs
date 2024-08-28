use crate::builtins::builtin;
use crate::builtins::is_builtin;
use crate::traits::{Runnable, ShellCommand};
use nix::unistd::{dup2, fork, pipe, ForkResult};
use std::error::Error;
use std::os::fd::AsRawFd;
use std::process::{ChildStdout, Command, Stdio};

pub fn cmd(tokens: Vec<String>) -> Result<Box<dyn ShellCommand>, Box<dyn Error>> {
    if tokens.is_empty() {
        Err("Tokens cannot be empty".into())
    } else if is_builtin(&tokens[0]) {
        Ok(Box::new(BuiltinCommand::new(tokens)?))
    } else {
        Ok(Box::new(ExternalCommand::new(tokens)?))
    }
}

pub fn runnable(tokens: Vec<String>) -> Result<Box<dyn Runnable>, Box<dyn Error>> {
    if tokens.is_empty() {
        Err("Tokens cannot be empty".into())
    } else if is_builtin(&tokens[0]) {
        Ok(Box::new(BuiltinCommand::new(tokens)?))
    } else {
        Ok(Box::new(ExternalCommand::new(tokens)?))
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinCommand {
    tokens: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(tokens: Vec<String>) -> Result<BuiltinCommand, Box<dyn Error>> {
        if tokens.is_empty() {
            return Err("Tokens cannot be empty".into());
        }
        Ok(BuiltinCommand { tokens })
    }

    pub fn run_builtin(&self) -> Result<(), Box<dyn Error>> {
        builtin(self.cmd(), self.args())?;
        Ok(())
    }
}

impl Runnable for BuiltinCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        self.run_builtin()?;
        Ok("".to_string())
    }
}

impl ShellCommand for BuiltinCommand {
    fn cmd(&self) -> &str {
        &self.tokens[0]
    }

    fn args(&self) -> Vec<&str> {
        self.tokens[1..].iter().map(|s| s.as_str()).collect()
    }

    fn pipe(&self, _stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let (pipe_out_r, pipe_out_w) = pipe()?;
        let (pipe_err_r, pipe_err_w) = pipe()?;

        match unsafe { fork() }? {
            ForkResult::Parent { child: _ } => {
                drop(pipe_out_w);
                drop(pipe_err_w);
                Ok(Some(ChildStdout::from(pipe_out_r)))
            }
            ForkResult::Child => {
                drop(pipe_out_r);
                drop(pipe_err_r);
                dup2(pipe_out_w.as_raw_fd(), 1)?;
                dup2(pipe_err_w.as_raw_fd(), 2)?;
                self.run_builtin()?;
                std::process::exit(0);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternalCommand {
    tokens: Vec<String>,
}

impl ExternalCommand {
    pub fn new(tokens: Vec<String>) -> Result<ExternalCommand, Box<dyn Error>> {
        if tokens.is_empty() {
            return Err("Tokens cannot be empty".into());
        }
        Ok(ExternalCommand { tokens })
    }
}

impl Runnable for ExternalCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut child = match Command::new(self.cmd()).args(self.args()).spawn() {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };
        match child.wait() {
            Ok(code) => {
                if code.success() {
                    Ok("".to_string())
                } else {
                    Err(format!("{}", code).into())
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl ShellCommand for ExternalCommand {
    fn cmd(&self) -> &str {
        &self.tokens[0]
    }

    fn args(&self) -> Vec<&str> {
        self.tokens[1..].iter().map(|s| s.as_str()).collect()
    }

    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let input = match stdin {
            Some(input) => Stdio::from(input),
            None => Stdio::inherit(),
        };
        // Spawn the command
        let mut child = match Command::new(self.cmd())
            .args(self.args())
            .stdin(input)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };

        Ok(child.stdout.take())
    }
}
