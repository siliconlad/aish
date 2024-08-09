use std::error::Error;

use crate::command::{cmd, runnable};
use crate::pipeline::Pipeline;
use crate::sequence::Sequence;
use crate::traits::{Runnable, ShellCommand};

pub fn clean(input: &mut String) -> &mut String {
    *input = input.trim().to_string();

    if input.ends_with('\n') {
        input.pop();
    }
    // Replace each whitespace with a single space
    *input = input.split_whitespace().collect::<Vec<&str>>().join(" ");

    input
}

pub fn tokenize(input: &mut String) -> Result<Box<dyn Runnable>, Box<dyn Error>> {
    let mut in_quotes = false;
    let mut in_double_quotes = false;
    let mut in_pipeline = false;
    let mut in_sequence = false;
    let mut escaped = false;
    let mut current_token = String::new();
    let mut tokens = Vec::<String>::new();
    let mut commands = Vec::<Box<dyn ShellCommand>>::new();
    let mut final_commands = Vec::<Box<dyn Runnable>>::new();

    let cleaned = clean(input);

    for c in cleaned.chars() {
        match c {
            ';' => {
                tokens.push(current_token);
                current_token = String::new();
                if in_pipeline {
                    tokens.retain(|x| !x.is_empty());
                    commands.push(cmd(tokens)?);
                    tokens = Vec::<String>::new();
                    final_commands.push(Box::new(Pipeline::new(commands)?));
                    commands = Vec::<Box<dyn ShellCommand>>::new();
                    in_pipeline = false;
                } else {
                    tokens.retain(|x| !x.is_empty());
                    final_commands.push(runnable(tokens)?);
                    tokens = Vec::<String>::new();
                }
                in_sequence = true;
            }
            '|' => {
                tokens.retain(|x| !x.is_empty());
                commands.push(cmd(tokens)?);
                tokens = Vec::<String>::new();
                in_pipeline = true;
            }
            '\\' => {
                if !in_quotes {
                    escaped = !escaped;
                } else {
                    current_token.push(c);
                }
            }
            '\'' => {
                if in_double_quotes {
                    current_token.push(c);
                } else if !escaped {
                    in_quotes = !in_quotes;
                } else {
                    current_token.push(c);
                }
                escaped = false;
            }
            '"' => {
                if in_quotes {
                    current_token.push(c);
                } else if !escaped {
                    in_double_quotes = !in_double_quotes;
                } else {
                    current_token.push(c);
                }
                escaped = false;
            }
            ' ' => {
                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else {
                    tokens.push(current_token);
                    current_token = String::new();
                }
                escaped = false;
            }
            _ => {
                current_token.push(c);
                escaped = false;
            }
        }
    }
    // Add last token
    tokens.push(current_token);
    tokens.retain(|x| !x.is_empty());

    // Return appropriate type
    if in_pipeline && !in_sequence {
        if !tokens.is_empty() {
            commands.push(cmd(tokens)?);
        }
        Ok(Box::new(Pipeline::new(commands)?))
    } else if in_pipeline && in_sequence {
        if !tokens.is_empty() {
            commands.push(cmd(tokens)?);
        }
        final_commands.push(Box::new(Pipeline::new(commands)?));
        Ok(Box::new(Sequence::new(final_commands)?))
    } else if in_sequence {
        if !tokens.is_empty() {
            final_commands.push(runnable(tokens)?);
        }
        Ok(Box::new(Sequence::new(final_commands)?))
    } else {
        Ok(runnable(tokens)?)
    }
}
