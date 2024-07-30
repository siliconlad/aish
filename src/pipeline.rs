use std::error::Error;
use std::ops::Index;
use std::os::fd::AsRawFd;
use std::os::fd::IntoRawFd;
use std::process::ChildStdout;
use nix::unistd::{fork, ForkResult, pipe, dup2};
use std::fs::File;
use std::io::{Read, BufReader};
use std::os::fd::FromRawFd;
use crate::command::SimpleCommand;

#[derive(Debug)]
pub struct Pipeline {
    commands: Vec<SimpleCommand>,
}

impl Pipeline {
    pub fn new(commands: Vec<SimpleCommand>) -> Result<Pipeline, Box<dyn Error>> {
        if commands.is_empty() {
            return Err("Commands cannot be empty".into());
        }
        Ok(Pipeline { commands })
    }

    pub fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut prev_stdout: Option<ChildStdout> = None;
        for (i, command) in self.commands.iter().enumerate() {
            if command.is_builtin() {
                let (pipe_out_r, pipe_out_w) = pipe()?;
                let (pipe_err_r, pipe_err_w) = pipe()?;

                match unsafe { fork() }? {
                    ForkResult::Parent { child: _ } => {
                        drop(pipe_out_w);
                        drop(pipe_err_w);
                        if i == self.commands.len() - 1 {
                            let mut output = String::new();
                            let mut reader = BufReader::new(unsafe { File::from_raw_fd(pipe_out_r.into_raw_fd()) });
                            reader.read_to_string(&mut output)?;
                            println!("{}", output);
                        } else {
                            prev_stdout = Some(ChildStdout::from(pipe_out_r));
                        }
                    }
                    ForkResult::Child => {
                        drop(pipe_out_r);
                        drop(pipe_err_r);
                        dup2(pipe_out_w.as_raw_fd(), 1)?;
                        dup2(pipe_err_w.as_raw_fd(), 2)?;
                        command.run_builtin()?;
                        std::process::exit(0);
                    }
                }    
            } else {
                let mut child = command.run_cmd(prev_stdout.take())?;
                if i == self.commands.len() - 1 {
                    let output = child.wait_with_output()?;
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    prev_stdout = Some(child.stdout.take().unwrap());
                }
            }
        }
        Ok("".to_string())
    }
}

impl Index<usize> for Pipeline {
    type Output = SimpleCommand;

    fn index(&self, index: usize) -> &Self::Output {
        &self.commands[index]
    }
}
