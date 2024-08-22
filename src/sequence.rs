use crate::traits::Runnable;
use std::error::Error;
use std::ops::Index;

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
    fn run(&self) -> Result<String, Box<dyn Error>> {
        for command in &self.commands {
            command.run()?;
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
