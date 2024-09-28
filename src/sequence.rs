use crate::traits::Runnable;
use std::error::Error;
use std::ops::Index;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Sequence {
    commands: Vec<Box<dyn Runnable>>,
}

impl Sequence {
    pub fn new(commands: Vec<Box<dyn Runnable>>) -> Result<Sequence, Box<dyn Error>> {
        Ok(Sequence { commands })
    }
}

impl Runnable for Sequence {
    fn run(&self, aliases: &mut HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        for command in &self.commands {
            let _ = command.run(aliases); // Ignore error
        }
        Ok("".to_string())
    }
}

impl Index<usize> for Sequence {
    type Output = Box<dyn Runnable>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.commands[index]
    }
}

#[derive(Clone)]
pub struct AndSequence {
    commands: Vec<Box<dyn Runnable>>,
}

impl AndSequence {
    pub fn new(commands: Vec<Box<dyn Runnable>>) -> Result<AndSequence, Box<dyn Error>> {
        Ok(AndSequence { commands })
    }
}

impl Runnable for AndSequence {
    fn run(&self, aliases: &mut HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        for command in &self.commands {
            command.run(aliases)?; // ? will propagate error
        }
        Ok("".to_string())
    }
}

impl Index<usize> for AndSequence {
    type Output = Box<dyn Runnable>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.commands[index]
    }
}
