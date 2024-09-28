mod lexer;
mod parser;
mod process;

use std::error::Error;

use crate::parsing::lexer::lex_impl;
use crate::parsing::parser::parse_impl;
use crate::parsing::process::process;
use crate::scanner::Scanner;
use crate::traits::Runnable;

pub fn parse(input: String) -> Result<Box<dyn Runnable>, Box<dyn Error>> {
    let input = process(input);

    let mut scanner = Scanner::new(input);
    let tokens = lex_impl(&mut scanner)?;
    let commands = parse_impl(&tokens)?;
    Ok(commands)
}
