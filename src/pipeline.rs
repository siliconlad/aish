use crate::traits::{Runnable, ShellCommand};
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Index;
use std::os::fd::FromRawFd;
use std::os::fd::IntoRawFd;
use std::process::ChildStdout;

#[derive(Clone)]
pub struct Pipeline {
    commands: Vec<Box<dyn ShellCommand>>,
}

impl Pipeline {
    pub fn new() -> Pipeline {
        Pipeline {
            commands: Vec::new(),
        }
    }

    pub fn init(commands: Vec<Box<dyn ShellCommand>>) -> Pipeline {
        Pipeline { commands }
    }

    pub fn add(&mut self, command: Box<dyn ShellCommand>) -> &mut Pipeline {
        self.commands.push(command);
        self
    }

    pub fn transfer(&mut self) -> Pipeline {
        let commands = self.commands.clone();
        self.clear();
        Pipeline { commands }
    }

    pub fn clear(&mut self) -> &mut Pipeline {
        self.commands.clear();
        self
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pipeline(")?;
        for (i, command) in self.commands.iter().enumerate() {
            write!(f, "{:?}", command)?;
            if i < self.commands.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Runnable for Pipeline {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut prev_stdout: Option<ChildStdout> = None;
        for (i, command) in self.commands.iter().enumerate() {
            let cmd_stdout = command.pipe(prev_stdout.take())?;
            if i == self.commands.len() - 1 {
                let mut output = String::new();
                if let Some(stdout) = cmd_stdout {
                    let mut reader =
                        BufReader::new(unsafe { File::from_raw_fd(stdout.into_raw_fd()) });
                    reader.read_to_string(&mut output)?;
                    let trimmed = output.trim_end_matches('\n').to_string();
                    if !trimmed.is_empty() {
                        println!("{}", trimmed);
                    }
                }
                return Ok("".to_string());
            } else {
                prev_stdout = cmd_stdout;
            }
        }
        Ok("".to_string())
    }
}

impl Index<usize> for Pipeline {
    type Output = Box<dyn ShellCommand>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.commands[index]
    }
}
