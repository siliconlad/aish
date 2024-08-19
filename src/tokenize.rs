use std::error::Error;

use crate::command::{cmd, runnable};
use crate::pipeline::Pipeline;
use crate::redirect::{InputRedirect, OutputRedirect, OutputRedirectAppend};
use crate::sequence::Sequence;
use crate::traits::{Runnable, ShellCommand};

// Type aliases for readability
type ShellCommandBox = Box<dyn ShellCommand>;
type RunnableBox = Box<dyn Runnable>;
type ShellCommandBoxes = Vec<ShellCommandBox>;

pub fn clean(input: &mut String) -> &mut String {
    // Remove leading and trailing whitespace
    *input = input.trim().to_string();

    // Remove trailing newline
    if input.ends_with('\n') {
        input.pop();
    }

    // Add semicolon to trigger final command
    if !input.ends_with(';') {
        input.push(';');
    }

    input
}

pub fn tokenize(input: &mut String) -> Result<RunnableBox, Box<dyn Error>> {
    // Flags for state
    let mut in_quotes = false;
    let mut in_double_quotes = false;
    let mut in_pipeline = false;
    let mut in_output_redirect = false;
    let mut in_output_redirect_append = false;
    let mut in_input_redirect = false;
    let mut escaped = false;

    // Accumulators
    let mut current_token = String::new();
    let mut tokens = Vec::<String>::new();
    let mut commands = Vec::<Box<dyn ShellCommand>>::new();
    let mut redirect_cmd: Option<Box<dyn ShellCommand>> = None;
    let mut final_commands = Vec::<Box<dyn Runnable>>::new();

    let cleaned = clean(input);
    for (i, c) in cleaned.chars().enumerate() {
        match c {
            // Input Redirect
            '<' => {
                if in_output_redirect {
                    let cmd = create_oredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    commands.push(Box::new(cmd));
                    in_output_redirect = false;
                } else {
                    redirect_cmd = Some(create_command(&mut tokens)?);
                }
                in_input_redirect = true;
            }
            // Output Redirect
            '>' => {
                // End input redirect and start output redirect
                if in_input_redirect {
                    let cmd = create_iredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    commands.push(Box::new(cmd));
                    in_input_redirect = false;
                }

                if cleaned.chars().nth(i + 1) == Some('>') {
                    in_output_redirect_append = true;
                } else {
                    if !in_output_redirect_append {
                        in_output_redirect = true;
                    }
                    redirect_cmd = Some(create_command(&mut tokens)?);
                }
            }
            ';' => {
                add_token(&mut tokens, &mut current_token);
                if in_pipeline {
                    if in_input_redirect {
                        let cmd = create_iredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                        commands.push(Box::new(cmd));
                        in_input_redirect = false;
                    } else if in_output_redirect {
                        let cmd = create_oredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                        commands.push(Box::new(cmd));
                        in_output_redirect = false;
                    } else if in_output_redirect_append {
                        let cmd = create_aredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                        commands.push(Box::new(cmd));
                        in_output_redirect_append = false;
                    } else {
                        commands.push(create_command(&mut tokens)?);
                    }
                    final_commands.push(create_pipeline(&mut commands)?);
                    in_pipeline = false;
                } else if in_input_redirect {
                    let cmd = create_iredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    final_commands.push(Box::new(cmd));
                    in_input_redirect = false;
                } else if in_output_redirect {
                    let cmd = create_oredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    final_commands.push(Box::new(cmd));
                    in_output_redirect = false;
                } else if in_output_redirect_append {
                    let cmd = create_aredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    final_commands.push(Box::new(cmd));
                    in_output_redirect_append = false;
                } else {
                    final_commands.push(runnable(tokens.clone())?);
                    tokens.clear();
                }
            }
            '|' => {
                if in_output_redirect {
                    let cmd = create_oredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    commands.push(Box::new(cmd));
                    in_output_redirect = false;
                } else if in_output_redirect_append {
                    let cmd = create_aredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    commands.push(Box::new(cmd));
                    in_output_redirect_append = false;
                } else if in_input_redirect {
                    let cmd = create_iredirect(redirect_cmd.take().unwrap(), &mut tokens)?;
                    commands.push(Box::new(cmd));
                    in_input_redirect = false;
                } else {
                    commands.push(create_command(&mut tokens)?);
                }
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
                    add_token(&mut tokens, &mut current_token);
                }
                escaped = false;
            }
            _ => {
                current_token.push(c);
                escaped = false;
            }
        }
    }

    // If there is only one command, return it
    if final_commands.len() == 1 {
        return Ok(final_commands.remove(0));
    }
    Ok(Box::new(Sequence::new(final_commands)?))
}

fn add_token(tokens: &mut Vec<String>, token: &mut String) {
    if !token.is_empty() {
        tokens.push(token.to_string());
        token.clear();
    }
}

fn create_command(tokens: &mut Vec<String>) -> Result<ShellCommandBox, Box<dyn Error>> {
    let new_cmd = cmd(tokens.clone())?;
    tokens.clear();
    Ok(new_cmd)
}

// Output Redirect
fn create_oredirect(
    cmd: ShellCommandBox,
    tokens: &mut Vec<String>,
) -> Result<OutputRedirect, Box<dyn Error>> {
    let new_oredirect = OutputRedirect::new(vec![cmd], tokens.join(""))?;
    tokens.clear();
    Ok(new_oredirect)
}

// Input Redirect
fn create_iredirect(
    cmd: ShellCommandBox,
    tokens: &mut Vec<String>,
) -> Result<InputRedirect, Box<dyn Error>> {
    let new_iredirect = InputRedirect::new(vec![cmd], tokens.join(""))?;
    tokens.clear();
    Ok(new_iredirect)
}

// Output Redirect Append
fn create_aredirect(
    cmd: ShellCommandBox,
    tokens: &mut Vec<String>,
) -> Result<OutputRedirectAppend, Box<dyn Error>> {
    let new_oredirect_append = OutputRedirectAppend::new(vec![cmd], tokens.join(""))?;
    tokens.clear();
    Ok(new_oredirect_append)
}

fn create_pipeline(cmds: &mut ShellCommandBoxes) -> Result<RunnableBox, Box<dyn Error>> {
    let new_pipeline = Pipeline::new(cmds.clone())?;
    cmds.clear();
    Ok(Box::new(new_pipeline))
}
