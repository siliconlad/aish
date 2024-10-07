use crate::traits::Runnable;
use dyn_clone::DynClone;
use std::error::Error;
use std::process::ChildStdout;

pub trait ShellCommand: Runnable + DynClone {
    fn cmd(&self) -> String;
    fn args(&self) -> Vec<String>;
    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(ShellCommand);
