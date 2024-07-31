use std::error::Error;
use std::fmt::Display;
use std::process::ChildStdout;

pub trait Runnable {
    fn run(&self) -> Result<String, Box<dyn Error>>;
}

pub trait ShellCommand: Runnable {
    fn cmd(&self) -> &str;
    fn args(&self) -> Vec<&str>;
    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<ChildStdout, Box<dyn Error>>;
}

impl Display for dyn ShellCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.cmd(), self.args().join(" "))
    }
}
