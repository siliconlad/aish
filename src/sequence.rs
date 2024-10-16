use crate::traits::Runnable;
use std::error::Error;
use std::fmt;
use std::ops::Index;
use std::collections::HashMap;
#[derive(Clone)]
pub struct Sequence {
    commands: Vec<Box<dyn Runnable>>,
}

impl Sequence {
    pub fn new() -> Sequence {
        Sequence {
            commands: Vec::new(),
        }
    }

    pub fn init(commands: Vec<Box<dyn Runnable>>) -> Sequence {
        Sequence { commands }
    }

    pub fn add(&mut self, command: Box<dyn Runnable>) -> &mut Sequence {
        self.commands.push(command);
        self
    }

    pub fn transfer(&mut self) -> Sequence {
        let commands = self.commands.clone();
        self.clear();
        Sequence { commands }
    }

    pub fn clear(&mut self) -> &mut Sequence {
        self.commands.clear();
        self
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

impl Runnable for Sequence {
    fn run(&self, aliases: &mut HashMap<String, String>) -> Result<String, Box<dyn Error>> {
        let mut prev_output: Option<String> = None;
        for command in &self.commands {
            match command.run(aliases) {
                Ok(output) => {
                    if !output.is_empty() {
                        prev_output = Some(output);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(prev_output.unwrap_or_default())
    }
}

impl Index<usize> for Sequence {
    type Output = Box<dyn Runnable>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.commands[index]
    }
}

impl fmt::Debug for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sequence(")?;
        for (i, command) in self.commands.iter().enumerate() {
            write!(f, "{:?}", command)?;
            if i < self.commands.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Clone)]
pub struct AndSequence {
    commands: Vec<Box<dyn Runnable>>,
}

impl AndSequence {
    pub fn new() -> AndSequence {
        AndSequence {
            commands: Vec::new(),
        }
    }

    pub fn init(commands: Vec<Box<dyn Runnable>>) -> AndSequence {
        AndSequence { commands }
    }

    pub fn add(&mut self, command: Box<dyn Runnable>) -> &mut AndSequence {
        self.commands.push(command);
        self
    }

    pub fn transfer(&mut self) -> AndSequence {
        let commands = self.commands.clone();
        self.clear();
        AndSequence { commands }
    }

    pub fn clear(&mut self) -> &mut AndSequence {
        self.commands.clear();
        self
    }
}

impl Default for AndSequence {
    fn default() -> Self {
        Self::new()
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

impl fmt::Debug for AndSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AndSequence(")?;
        for (i, command) in self.commands.iter().enumerate() {
            write!(f, "{:?}", command)?;
            if i < self.commands.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}
