use crate::builtins::builtin;
use crate::builtins::is_builtin;
use crate::errors::SyntaxError;
use crate::redirect;
use crate::token::Token;
use crate::traits::{Runnable, ShellCommand};

use nix::unistd::{dup2, fork, pipe, ForkResult};
use std::error::Error;
use std::fmt;
use std::os::fd::AsRawFd;
use std::process::{ChildStdout, Command, Stdio};

pub enum CommandType {
    Builtin(BuiltinCommand),
    External(ExternalCommand),
    InputRedirect(redirect::InputRedirect),
    OutputRedirect(redirect::OutputRedirect),
    OutputRedirectAppend(redirect::OutputRedirectAppend),
}

impl CommandType {
    pub fn create(tokens: Vec<Token>) -> Result<CommandType, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::ExpectedToken("".to_string()));
        }

        match &tokens[..] {
            [Token::DoubleQuoted(_)] => {
                debug!("Detected LLM command with tokens: {:?}", tokens);
                let mut new_tokens = vec![Token::Plain("llm".to_string())];
                new_tokens.extend(tokens);
                Ok(CommandType::Builtin(BuiltinCommand::new(new_tokens)?))
            }
            [Token::Plain(cmd), ..] if is_builtin(cmd) => {
                debug!("Detected builtin command: {:?}", tokens);
                Ok(CommandType::Builtin(BuiltinCommand::new(tokens)?))
            }
            _ => {
                debug!("Detected external command: {:?}", tokens);
                Ok(CommandType::External(ExternalCommand::new(tokens)?))
            }
        }
    }

    pub fn unpack_cmd(self) -> Box<dyn ShellCommand> {
        match self {
            CommandType::Builtin(cmd) => Box::new(cmd),
            CommandType::External(cmd) => Box::new(cmd),
            CommandType::InputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirectAppend(cmd) => Box::new(cmd),
        }
    }

    pub fn unpack_run(self) -> Box<dyn Runnable> {
        match self {
            CommandType::Builtin(cmd) => Box::new(cmd),
            CommandType::External(cmd) => Box::new(cmd),
            CommandType::InputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirectAppend(cmd) => Box::new(cmd),
        }
    }
}

impl fmt::Debug for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandType::Builtin(cmd) => write!(f, "{:?}", cmd),
            CommandType::External(cmd) => write!(f, "{:?}", cmd),
            CommandType::InputRedirect(cmd) => write!(f, "{:?}", cmd),
            CommandType::OutputRedirect(cmd) => write!(f, "{:?}", cmd),
            CommandType::OutputRedirectAppend(cmd) => write!(f, "{:?}", cmd),
        }
    }
}

#[derive(Clone)]
pub struct BuiltinCommand {
    tokens: Vec<Token>,
}

impl BuiltinCommand {
    pub fn new(tokens: Vec<Token>) -> Result<BuiltinCommand, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::InternalError);
        }
        Ok(BuiltinCommand { tokens })
    }

    pub fn run_builtin(&self, stdin: Option<ChildStdout>) -> Result<(), Box<dyn Error>> {
        builtin(self.cmd(), self.args(), stdin)?;
        Ok(())
    }
}

impl fmt::Debug for BuiltinCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BuiltinCommand({:?})", self.tokens)
    }
}

impl Runnable for BuiltinCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        debug!("Running builtin: {:?}", self);
        self.run_builtin(None)?;
        Ok("".to_string())
    }
}

impl ShellCommand for BuiltinCommand {
    fn cmd(&self) -> String {
        self.tokens[0].resolve()
    }

    fn args(&self) -> Vec<String> {
        self.tokens[1..].iter().map(|s| s.resolve()).collect()
    }

    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
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
                self.run_builtin(stdin)?;
                std::process::exit(0);
            }
        }
    }
}

#[derive(Clone)]
pub struct ExternalCommand {
    tokens: Vec<Token>,
}

impl ExternalCommand {
    pub fn new(tokens: Vec<Token>) -> Result<ExternalCommand, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::InternalError);
        }
        Ok(ExternalCommand { tokens })
    }
}

impl fmt::Debug for ExternalCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExternalCommand({:?})", self.tokens)
    }
}

impl Runnable for ExternalCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        debug!("Running external: {:?}", self);
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
    fn cmd(&self) -> String {
        self.tokens[0].resolve()
    }

    fn args(&self) -> Vec<String> {
        self.tokens[1..].iter().map(|s| s.resolve()).collect()
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
