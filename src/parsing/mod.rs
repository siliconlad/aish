mod lexer;
mod parser;
mod process;
mod scanner;

use std::error::Error;

use crate::parsing::lexer::lex_impl;
use crate::parsing::parser::parse_impl;
use crate::parsing::process::process;
use crate::parsing::scanner::Scanner;

pub fn parse(input: String) -> Result<(), Box<dyn Error>> {
    debug!("User input: {}", input);

    let input = process(input);
    debug!("Processed input: {}", input);

    let mut scanner = Scanner::new(input);
    let tokens = lex_impl(&mut scanner)?;
    debug!("Lexed tokens: {:?}", tokens);

    let mut scanner = Scanner::new(tokens);
    parse_impl(&mut scanner)?;

    Ok(())
}
