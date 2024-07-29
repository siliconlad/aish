use std::error::Error;
use std::ops::Index;
use std::process::{Child, Stdio};

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
        let mut cmd_p: Vec<Child> = Vec::new();
        for (i, command) in self.commands.iter().enumerate() {
            let stdin = if i == 0 {
                None
            } else {
                Some(Stdio::from(cmd_p[i - 1].stdout.take().unwrap()))
            };
            let child = command.run(stdin)?;
            if i == self.commands.len() - 1 {
                let output = child.wait_with_output()?;
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                cmd_p.push(child);
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
