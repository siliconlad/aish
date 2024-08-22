use std::error::Error;

use crate::command::{cmd, runnable};
use crate::pipeline::Pipeline;
use crate::redirect::{InputRedirect, OutputRedirect, OutputRedirectAppend};
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
    let mut in_output_redirect = false;
    let mut in_output_redirect_append = false;
    let mut in_input_redirect = false;
    let mut escaped = false;
    let mut current_token = String::new();
    let mut tokens = Vec::<String>::new();
    let mut commands = Vec::<Box<dyn ShellCommand>>::new();
    let mut final_commands = Vec::<Box<dyn Runnable>>::new();

    let cleaned = clean(input);

    for (i, c) in cleaned.chars().enumerate() {
        match c {
            // Input Redirect
            '<' => {
                in_input_redirect = true;
                if in_output_redirect {
                    let prev_cmd = commands.remove(commands.len() - 1);
                    commands.push(Box::new(OutputRedirect::new(
                        vec![prev_cmd],
                        tokens.join(""),
                    )?));
                    in_output_redirect = false;
                } else {
                    tokens.retain(|x| !x.is_empty());
                    commands.push(cmd(tokens)?);
                }
                tokens = Vec::<String>::new();
            }
            // Output Redirect
            '>' => {
                // End input redirect and start output redirect
                if in_input_redirect {
                    let prev_cmd = commands.remove(commands.len() - 1);
                    commands.push(Box::new(InputRedirect::new(
                        vec![prev_cmd],
                        tokens.join(""),
                    )?));
                    in_input_redirect = false;
                    tokens = Vec::<String>::new();
                }

                if cleaned.chars().nth(i + 1) == Some('>') {
                    in_output_redirect_append = true;
                } else if in_output_redirect_append {
                    tokens.retain(|x| !x.is_empty());
                    commands.push(cmd(tokens)?);
                    tokens = Vec::<String>::new();
                } else {
                    in_output_redirect = true;
                    tokens.retain(|x| !x.is_empty());
                    commands.push(cmd(tokens)?);
                    tokens = Vec::<String>::new();
                }
            }
            ';' => {
                tokens.push(current_token);
                current_token = String::new();
                if in_pipeline {
                    if in_input_redirect {
                        let prev_cmd = commands.remove(commands.len() - 1);
                        commands.push(Box::new(InputRedirect::new(
                            vec![prev_cmd],
                            tokens.join(""),
                        )?));
                        final_commands.push(Box::new(Pipeline::new(commands)?));
                        tokens = Vec::<String>::new();
                        commands = Vec::<Box<dyn ShellCommand>>::new();
                        in_pipeline = false;
                        in_input_redirect = false;
                    } else if in_output_redirect {
                        tokens.retain(|x| !x.is_empty());
                        let prev_cmd = commands.remove(commands.len() - 1);
                        commands.push(Box::new(OutputRedirect::new(
                            vec![prev_cmd],
                            tokens.join(""),
                        )?));
                        final_commands.push(Box::new(Pipeline::new(commands)?));
                        tokens = Vec::<String>::new();
                        commands = Vec::<Box<dyn ShellCommand>>::new();
                        in_pipeline = false;
                        in_output_redirect = false;
                    } else if in_output_redirect_append {
                        tokens.retain(|x| !x.is_empty());
                        let prev_cmd = commands.remove(commands.len() - 1);
                        commands.push(Box::new(OutputRedirectAppend::new(
                            vec![prev_cmd],
                            tokens.join(""),
                        )?));
                        final_commands.push(Box::new(Pipeline::new(commands)?));
                        tokens = Vec::<String>::new();
                        commands = Vec::<Box<dyn ShellCommand>>::new();
                        in_pipeline = false;
                        in_output_redirect_append = false;
                    } else {
                        tokens.retain(|x| !x.is_empty());
                        commands.push(cmd(tokens)?);
                        tokens = Vec::<String>::new();
                        final_commands.push(Box::new(Pipeline::new(commands)?));
                        commands = Vec::<Box<dyn ShellCommand>>::new();
                        in_pipeline = false;
                    }
                } else if in_output_redirect {
                    tokens.retain(|x| !x.is_empty());
                    let output_redirect = OutputRedirect::new(commands, tokens.join(""))?;
                    final_commands.push(Box::new(output_redirect));
                    commands = Vec::<Box<dyn ShellCommand>>::new();
                    tokens = Vec::<String>::new();
                    in_output_redirect = false;
                } else if in_output_redirect_append {
                    tokens.retain(|x| !x.is_empty());
                    let output_redirect_append =
                        OutputRedirectAppend::new(commands, tokens.join(""))?;
                    final_commands.push(Box::new(output_redirect_append));
                    commands = Vec::<Box<dyn ShellCommand>>::new();
                    tokens = Vec::<String>::new();
                    in_output_redirect_append = false;
                } else if in_input_redirect {
                    let input_redirect = InputRedirect::new(commands, tokens.join(""))?;
                    final_commands.push(Box::new(input_redirect));
                    commands = Vec::<Box<dyn ShellCommand>>::new();
                    tokens = Vec::<String>::new();
                    in_input_redirect = false;
                } else {
                    final_commands.push(runnable(tokens)?);
                    tokens = Vec::<String>::new();
                }
                in_sequence = true;
            }
            '|' => {
                debug!("Found pipe |");
                tokens.retain(|x| !x.is_empty());
                if in_output_redirect {
                    let prev_cmd = commands.remove(commands.len() - 1);
                    commands.push(Box::new(OutputRedirect::new(
                        vec![prev_cmd],
                        tokens.join(""),
                    )?));
                    in_output_redirect = false;
                } else if in_output_redirect_append {
                    let prev_cmd = commands.remove(commands.len() - 1);
                    commands.push(Box::new(OutputRedirectAppend::new(
                        vec![prev_cmd],
                        tokens.join(""),
                    )?));
                    in_output_redirect_append = false;
                } else if in_input_redirect {
                    let prev_cmd = commands.remove(commands.len() - 1);
                    commands.push(Box::new(InputRedirect::new(
                        vec![prev_cmd],
                        tokens.join(""),
                    )?));
                    in_input_redirect = false;
                } else {
                    commands.push(cmd(tokens)?);
                }
                tokens = Vec::<String>::new();
                in_pipeline = true;
            }
            '\\' => {
                debug!("Found backslash \\");
                if !in_quotes {
                    escaped = !escaped;
                } else {
                    current_token.push(c);
                }
            }
            '\'' => {
                debug!("Found single quote \'");
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
                debug!("Found double quote \"");
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
                debug!("Found whitespace");
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
        if in_output_redirect {
            let prev_cmd = commands.remove(commands.len() - 1);
            commands.push(Box::new(OutputRedirect::new(
                vec![prev_cmd],
                tokens.join(""),
            )?));
        } else if in_input_redirect {
            let prev_cmd = commands.remove(commands.len() - 1);
            commands.push(Box::new(InputRedirect::new(
                vec![prev_cmd],
                tokens.join(""),
            )?));
        } else if in_output_redirect_append {
            let prev_cmd = commands.remove(commands.len() - 1);
            commands.push(Box::new(OutputRedirectAppend::new(
                vec![prev_cmd],
                tokens.join(""),
            )?));
        } else if !tokens.is_empty() {
            commands.push(cmd(tokens)?);
        }
        Ok(Box::new(Pipeline::new(commands)?))
    } else if in_pipeline && in_sequence {
        if in_output_redirect {
            let prev_cmd = commands.remove(commands.len() - 1);
            commands.push(Box::new(OutputRedirect::new(
                vec![prev_cmd],
                tokens.join(""),
            )?));
        } else if in_input_redirect {
            let prev_cmd = commands.remove(commands.len() - 1);
            commands.push(Box::new(InputRedirect::new(
                vec![prev_cmd],
                tokens.join(""),
            )?));
        } else if in_output_redirect_append {
            let prev_cmd = commands.remove(commands.len() - 1);
            commands.push(Box::new(OutputRedirectAppend::new(
                vec![prev_cmd],
                tokens.join(""),
            )?));
        } else if !tokens.is_empty() {
            commands.push(cmd(tokens)?);
        }
        final_commands.push(Box::new(Pipeline::new(commands)?));
        Ok(Box::new(Sequence::new(final_commands)?))
    } else if in_sequence {
        if in_output_redirect {
            final_commands.push(Box::new(OutputRedirect::new(commands, tokens.join(""))?));
        } else if in_input_redirect {
            final_commands.push(Box::new(InputRedirect::new(commands, tokens.join(""))?));
        } else if in_output_redirect_append {
            final_commands.push(Box::new(OutputRedirectAppend::new(
                commands,
                tokens.join(""),
            )?));
        } else if !tokens.is_empty() {
            final_commands.push(runnable(tokens)?);
        }
        Ok(Box::new(Sequence::new(final_commands)?))
    } else if in_output_redirect {
        Ok(Box::new(OutputRedirect::new(commands, tokens.join(""))?))
    } else if in_input_redirect {
        Ok(Box::new(InputRedirect::new(commands, tokens.join(""))?))
    } else if in_output_redirect_append {
        Ok(Box::new(OutputRedirectAppend::new(
            commands,
            tokens.join(""),
        )?))
    } else {
        Ok(runnable(tokens)?)
    }
}
