use crate::errors::SyntaxError;
use crate::traits::{Runnable, ShellCommand};
use std::fmt;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::fd::IntoRawFd;
use std::os::unix::io::FromRawFd;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;

#[derive(Clone, PartialEq)]
pub enum RedirectType {
    Output,
    OutputAppend,
    Input,
    None,
}

pub enum Redirect {
    Output(OutputRedirect),
    OutputAppend(OutputRedirectAppend),
    Input(InputRedirect),
    None,
}

#[derive(Clone)]
pub struct OutputRedirect {
    commands: Vec<Box<dyn ShellCommand>>,
    output_file: String,
}

impl OutputRedirect {
    pub fn new(
        commands: Vec<Box<dyn ShellCommand>>,
        output_file: String,
    ) -> Result<Self, SyntaxError> {
        if commands.len() != 1 {
            return Err(SyntaxError::InternalError);
        }
        Ok(Self {
            commands,
            output_file,
        })
    }

    fn open_file(&self) -> Result<File, Box<dyn Error>> {
        let file = File::create(&self.output_file)?;
        Ok(file)
    }
}

impl fmt::Debug for OutputRedirect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutputRedirect({:?}, {:?})", self.output_file, self.commands)
    }
}

impl Runnable for OutputRedirect {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut file = self.open_file()?;
        let stdout = self.commands[0].pipe(None)?;
        if let Some(mut stdout) = stdout {
            std::io::copy(&mut stdout, &mut file)?;
        }
        Ok("".to_string())
    }
}

impl ShellCommand for OutputRedirect {
    fn cmd(&self) -> &str {
        self.commands[0].cmd()
    }

    fn args(&self) -> Vec<&str> {
        self.commands[0].args()
    }

    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let mut file = self.open_file()?;
        let stdout = self.commands[0].pipe(stdin)?;
        if let Some(mut stdout) = stdout {
            std::io::copy(&mut stdout, &mut file)?;
        }
        Ok(None)
    }
}

#[derive(Clone)]
pub struct OutputRedirectAppend {
    commands: Vec<Box<dyn ShellCommand>>,
    output_file: String,
}

impl OutputRedirectAppend {
    pub fn new(
        commands: Vec<Box<dyn ShellCommand>>,
        output_file: String,
    ) -> Result<Self, SyntaxError> {
        if commands.len() != 1 {
            return Err(SyntaxError::InternalError);
        }
        Ok(Self {
            commands,
            output_file,
        })
    }

    fn open_file(&self) -> Result<File, Box<dyn Error>> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.output_file)?;
        Ok(file)
    }
}

impl fmt::Debug for OutputRedirectAppend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutputRedirectAppend({:?}, {:?})", self.output_file, self.commands)
    }
}

impl Runnable for OutputRedirectAppend {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut file = self.open_file()?;
        let stdout = self.commands[0].pipe(None)?;
        if let Some(mut stdout) = stdout {
            std::io::copy(&mut stdout, &mut file)?;
        }
        Ok("".to_string())
    }
}

impl ShellCommand for OutputRedirectAppend {
    fn cmd(&self) -> &str {
        self.commands[0].cmd()
    }

    fn args(&self) -> Vec<&str> {
        self.commands[0].args()
    }

    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let mut file = self.open_file()?;
        let stdout = self.commands[0].pipe(stdin)?;
        if let Some(mut stdout) = stdout {
            std::io::copy(&mut stdout, &mut file)?;
        }
        Ok(None)
    }
}

#[derive(Clone)]
pub struct InputRedirect {
    commands: Vec<Box<dyn ShellCommand>>,
    input_file: String,
}

impl InputRedirect {
    pub fn new(
        commands: Vec<Box<dyn ShellCommand>>,
        input_file: String,
    ) -> Result<Self, SyntaxError> {
        if commands.len() != 1 {
            return Err(SyntaxError::InternalError);
        }
        Ok(Self {
            commands,
            input_file,
        })
    }
}

impl fmt::Debug for InputRedirect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InputRedirect({:?}, {:?})", self.input_file, self.commands)
    }
}

impl Runnable for InputRedirect {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let file = File::open(&self.input_file)?;
        let file_fd = file.into_raw_fd();

        let mut child = Command::new("cat")
            .stdin(unsafe { Stdio::from_raw_fd(file_fd) })
            .stdout(Stdio::piped())
            .spawn()?;
        let input = child.stdout.take().unwrap();

        let stdout = self.commands[0].pipe(Some(input))?;
        if let Some(mut stdout) = stdout {
            let mut output = String::new();
            stdout.read_to_string(&mut output)?;
            println!("{}", output.trim_end());
        }
        Ok("".to_string())
    }
}

impl ShellCommand for InputRedirect {
    fn cmd(&self) -> &str {
        self.commands[0].cmd()
    }

    fn args(&self) -> Vec<&str> {
        self.commands[0].args()
    }

    fn pipe(&self, _stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let file = File::open(&self.input_file)?;
        let file_fd = file.into_raw_fd();

        let mut child = Command::new("cat")
            .stdin(unsafe { Stdio::from_raw_fd(file_fd) })
            .stdout(Stdio::piped())
            .spawn()?;
        let input = child.stdout.take().unwrap();

        let stdout = self.commands[0].pipe(Some(input))?;
        Ok(stdout)
    }
}
