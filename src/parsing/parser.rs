use crate::command::CommandType;
use crate::errors::SyntaxError;
use crate::parsing::scanner::Scanner;
use crate::pipeline::Pipeline;
use crate::redirect::{InputRedirect, OutputRedirect, OutputRedirectAppend};
use crate::sequence::{AndSequence, Sequence};
use crate::token::{Token, Tokens};

pub fn parse_impl(tokens: &mut Scanner<Tokens>) -> Result<Sequence, SyntaxError> {
    // State variables
    let mut in_and_sequence = false;
    let mut in_pipeline = false;

    // Final commands
    let mut final_commands = Sequence::new();
    let mut and_sequence = AndSequence::new();
    let mut pipeline = Pipeline::new();

    loop {
        debug!("Parsing tokens");

        if tokens.peek().is_none() {
            debug!("End of tokens");
            break;
        }

        let command = parse_cmd_impl(tokens)?;
        debug!("Parsed command: {:?}", command);

        match tokens.peek().unwrap() {
            Token::Meta(m) if m == ";" => {
                tokens.next(); // Consume token
                if in_pipeline && in_and_sequence {
                    in_pipeline = false;
                    in_and_sequence = false;
                    pipeline.add(command.unpack_cmd());
                    and_sequence.add(Box::new(pipeline.transfer()));
                    final_commands.add(Box::new(and_sequence.transfer()));
                } else if in_pipeline {
                    in_pipeline = false;
                    pipeline.add(command.unpack_cmd());
                    final_commands.add(Box::new(pipeline.transfer()));
                } else if in_and_sequence {
                    in_and_sequence = false;
                    and_sequence.add(command.unpack_run());
                    final_commands.add(Box::new(and_sequence.transfer()));
                } else {
                    final_commands.add(command.unpack_run());
                }
            }
            Token::Meta(m) if m == "&" => {
                tokens.next(); // Consume token
                in_and_sequence = true;
                if in_pipeline {
                    in_pipeline = false;
                    pipeline.add(command.unpack_cmd());
                    and_sequence.add(Box::new(pipeline.transfer()));
                } else {
                    and_sequence.add(command.unpack_run());
                }
            }
            Token::Meta(m) if m == "|" => {
                tokens.next(); // Consume token
                in_pipeline = true;
                pipeline.add(command.unpack_cmd());
            }
            _ => {
                let token = tokens.next().to_string();
                return Err(SyntaxError::UnexpectedToken(token));
            }
        }
    }

    Ok(final_commands)
}

fn parse_cmd_impl(tokens: &mut Scanner<Tokens>) -> Result<CommandType, SyntaxError> {
    let mut command_tokens = Vec::<Token>::new();

    loop {
        if tokens.peek().is_none() {
            debug!("End of tokens");
            break;
        }

        match tokens.peek().unwrap() {
            Token::Meta(c) => {
                debug!("Tokens: {:?}", command_tokens);
                debug!("Break point ({})", c);
                break;
            }
            _ => {
                command_tokens.push(tokens.next());
            }
        }
    }

    let mut command = CommandType::create(command_tokens)?;

    loop {
        if tokens.peek().is_none() {
            break;
        }

        match tokens.peek().unwrap() {
            Token::Meta(m) if m == ";" => {
                debug!("End of command (;)");
                break;
            }
            Token::Meta(m) if m == "&" => {
                debug!("End of command (&)");
                break;
            }
            Token::Meta(m) if m == "|" => {
                debug!("End of command (|)");
                break;
            }
            Token::Meta(m) if m == "<" => {
                debug!("Input redirect");
                tokens.next();
                let file_name = tokens.next().to_string();
                debug!("Input redirect file name: {}", file_name);
                command = CommandType::InputRedirect(InputRedirect::new(
                    vec![command.unpack_cmd()],
                    file_name,
                )?);
            }
            Token::Meta(m) if m == ">" => {
                debug!("Output redirect");
                tokens.next();
                let file_name = tokens.next().to_string();
                debug!("Output redirect file name: {}", file_name);
                command = CommandType::OutputRedirect(OutputRedirect::new(
                    vec![command.unpack_cmd()],
                    file_name,
                )?);
            }
            Token::Meta(m) if m == ">>" => {
                debug!("Output redirect append");
                tokens.next();
                let file_name = tokens.next().to_string();
                debug!("Output redirect append file name: {}", file_name);
                command = CommandType::OutputRedirectAppend(OutputRedirectAppend::new(
                    vec![command.unpack_cmd()],
                    file_name,
                )?);
            }
            _ => {
                let token = tokens.next().to_string();
                debug!("Unexpected token: {}", token);
                return Err(SyntaxError::UnexpectedToken(token));
            }
        }
    }

    Ok(command)
}
