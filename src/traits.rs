use dyn_clone::DynClone;
use std::error::Error;
use std::fmt::Display;
use std::process::ChildStdout;

pub trait Runnable: DynClone {
    fn run(&self) -> Result<String, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(Runnable);

pub trait ShellCommand: Runnable + DynClone {
    fn cmd(&self) -> &str;
    fn args(&self) -> Vec<&str>;
    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(ShellCommand);

impl Display for dyn ShellCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.cmd(), self.args().join(" "))
    }
}
