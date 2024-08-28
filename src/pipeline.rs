use crate::traits::{Runnable, ShellCommand};
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
    pub fn new(commands: Vec<Box<dyn ShellCommand>>) -> Result<Pipeline, Box<dyn Error>> {
        if commands.is_empty() {
            return Err("Commands cannot be empty".into());
        }
        Ok(Pipeline { commands })
    }
}

impl Runnable for Pipeline {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut prev_stdout: Option<ChildStdout> = None;
        for (i, command) in self.commands.iter().enumerate() {
            // let prev_stdout_ = match prev_stdout.take() {
            //     Some(stdout) => Stdio::from(stdout),
            //     None => Stdio::null(),
            // };
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
