use crate::traits::Runnable;
use dyn_clone::DynClone;
use std::error::Error;
use std::process::ChildStdout;
use std::collections::HashMap;

pub trait ShellCommand: Runnable + DynClone {
    fn cmd(&self) -> String;
    fn args(&self) -> Vec<String>;
    fn pipe(&self, stdin: Option<ChildStdout>, aliases: &mut HashMap<String, String>) -> Result<Option<ChildStdout>, Box<dyn Error>>;
}
dyn_clone::clone_trait_object!(ShellCommand);
