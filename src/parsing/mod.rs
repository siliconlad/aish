mod lexer;
mod parser;
mod process;
mod scanner;

use std::error::Error;

use crate::parsing::lexer::lex_impl;
use crate::parsing::parser::parse_impl;
use crate::parsing::process::process;
use crate::parsing::scanner::Scanner;
use crate::sequence::Sequence;

pub fn parse(input: String) -> Result<Sequence, Box<dyn Error>> {
    debug!("Parsing input: {}", input);
    let input = process(input);

    debug!("Lexing input: {}", input);
    let mut scanner = Scanner::new(input);
    let tokens = lex_impl(&mut scanner)?;

    debug!("Parsing tokens: {:?}", tokens);
    // let mut scanner = Scanner::new(tokens);
    // let commands = parse_impl(&mut scanner)?;

    // Ok(commands)
    Ok(Sequence::new())
}
