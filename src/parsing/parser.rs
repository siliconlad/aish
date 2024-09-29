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
        if tokens.peek().is_none() {
            break;
        }

        let command = parse_cmd_impl(tokens)?;

        match tokens.peek().unwrap() {
            Token::Meta(m) if m == ";" => {
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
            break;
        }

        match tokens.peek().unwrap() {
            Token::Meta(m) if m == ";" => {
                break;
            }
            Token::Meta(m) if m == "&" => {
                break;
            }
            Token::Meta(m) if m == "|" => {
                break;
            }
            Token::Meta(m) if m == "<" => {
                break;
            }
            Token::Meta(m) if m == ">" => {
                break;
            }
            Token::Meta(m) if m == ">>" => {
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
                break;
            }
            Token::Meta(m) if m == "&" => {
                break;
            }
            Token::Meta(m) if m == "|" => {
                break;
            }
            Token::Meta(m) if m == "<" => {
                tokens.next();
                let file_name = tokens.next().to_string();
                command = CommandType::InputRedirect(InputRedirect::new(
                    vec![command.unpack_cmd()],
                    file_name,
                )?);
            }
            Token::Meta(m) if m == ">" => {
                tokens.next();
                let file_name = tokens.next().to_string();
                command = CommandType::OutputRedirect(OutputRedirect::new(
                    vec![command.unpack_cmd()],
                    file_name,
                )?);
            }
            Token::Meta(m) if m == ">>" => {
                tokens.next();
                let file_name = tokens.next().to_string();
                command = CommandType::OutputRedirectAppend(OutputRedirectAppend::new(
                    vec![command.unpack_cmd()],
                    file_name,
                )?);
            }
            _ => {
                let token = tokens.next().to_string();
                return Err(SyntaxError::UnexpectedToken(token));
            }
        }
    }

    Ok(command)
}
