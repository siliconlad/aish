use crate::traits::{Runnable, ShellCommand};
use std::error::Error;
use std::fs::File;
use std::process::ChildStdout;

pub struct OutputRedirect {
    commands: Vec<Box<dyn ShellCommand>>,
    output_file: String,
}

impl OutputRedirect {
    pub fn new(
        commands: Vec<Box<dyn ShellCommand>>,
        output_file: String,
    ) -> Result<Self, Box<dyn Error>> {
        if commands.len() != 1 {
            return Err("Output redirect must have exactly one command".into());
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
