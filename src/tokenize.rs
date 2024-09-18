use std::error::Error;
use std::env;

use crate::command::{cmd, runnable};
use crate::pipeline::Pipeline;
use crate::redirect::{
    InputRedirect, OutputRedirect, OutputRedirectAppend, Redirect, RedirectType,
};
use crate::sequence::{AndSequence, Sequence};
use crate::token::Token;
use crate::traits::{Runnable, ShellCommand};

// Type aliases for readability
type ShellCommandBox = Box<dyn ShellCommand>;
type RunnableBox = Box<dyn Runnable>;
type ShellCommandBoxes = Vec<ShellCommandBox>;
type RunnableBoxes = Vec<RunnableBox>;

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
    let mut in_and_sequence = false;
    let mut r_type = RedirectType::None;
    let mut escaped = false;
    let mut in_variable = false;

    // Accumulators
    let mut var_buffer = String::new();
    let mut current_token = String::new();
    let mut tokens = Vec::<Token>::new();
    let mut commands = Vec::<Box<dyn ShellCommand>>::new();
    let mut r_cmd: Option<Box<dyn ShellCommand>> = None;
    let mut sequence_commands = Vec::<Box<dyn Runnable>>::new();
    let mut final_commands = Vec::<Box<dyn Runnable>>::new();

    let cleaned = clean(input);
    for (i, c) in cleaned.chars().enumerate() {
        match c {
            '$' => {
                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes || escaped {
                    current_token.push(c);
                } else {
                    debug!("Found dollar sign $");
                    in_variable = true;
                }
            }
            '&' => {
                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else if cleaned.chars().nth(i + 1) == Some('&') {
                    in_and_sequence = true;
                } else {
                    add_token(&mut tokens, &mut current_token);
                    if in_pipeline {
                        if r_type != RedirectType::None {
                            let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                            commands.push(match cmd {
                                Redirect::Output(oredirect) => Box::new(oredirect),
                                Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                                Redirect::Input(iredirect) => Box::new(iredirect),
                                Redirect::None => unreachable!(),
                            });
                            r_type = RedirectType::None;
                        } else {
                            commands.push(create_command(&mut tokens)?);
                        }
                        sequence_commands.push(create_pipeline(&mut commands)?);
                        in_pipeline = false;
                    } else if r_type != RedirectType::None {
                        let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                        sequence_commands.push(match cmd {
                            Redirect::Output(oredirect) => Box::new(oredirect),
                            Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                            Redirect::Input(iredirect) => Box::new(iredirect),
                            Redirect::None => unreachable!(),
                        });
                        r_type = RedirectType::None;
                    } else {
                        sequence_commands.push(runnable(tokens.clone())?);
                        tokens.clear();
                    }
                }
            }
            // Input Redirect
            '<' => {
                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else if r_type == RedirectType::Output || r_type == RedirectType::OutputAppend {
                    let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                    r_cmd = match cmd {
                        Redirect::Output(oredirect) => Some(Box::new(oredirect)),
                        Redirect::OutputAppend(aredirect) => Some(Box::new(aredirect)),
                        _ => unreachable!(),
                    };
                } else {
                    r_cmd = Some(create_command(&mut tokens)?);
                }
                r_type = RedirectType::Input;
            }
            // Output Redirect
            '>' => {
                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else if r_type == RedirectType::Input {
                    let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                    r_cmd = match cmd {
                        Redirect::Input(iredirect) => Some(Box::new(iredirect)),
                        _ => unreachable!(),
                    };
                } else if r_type != RedirectType::OutputAppend {
                    r_cmd = Some(create_command(&mut tokens)?);
                }

                if cleaned.chars().nth(i + 1) == Some('>') {
                    r_type = RedirectType::OutputAppend;
                } else if r_type != RedirectType::OutputAppend {
                    r_type = RedirectType::Output;
                }
            }
            ';' => {
                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }
                
                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else {
                    add_token(&mut tokens, &mut current_token);
                    if in_pipeline {
                        if r_type != RedirectType::None {
                            let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                            commands.push(match cmd {
                                Redirect::Output(oredirect) => Box::new(oredirect),
                                Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                                Redirect::Input(iredirect) => Box::new(iredirect),
                                Redirect::None => unreachable!(),
                            });
                            r_type = RedirectType::None;
                        } else {
                            commands.push(create_command(&mut tokens)?);
                        }

                        if in_and_sequence {
                            sequence_commands.push(create_pipeline(&mut commands)?);
                            final_commands.push(create_and_sequence(&mut sequence_commands)?);
                            in_and_sequence = false;
                        } else {
                            final_commands.push(create_pipeline(&mut commands)?);
                        }
                        in_pipeline = false;
                    } else if r_type != RedirectType::None {
                        let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                        let r_cmd: Box<dyn Runnable> = match cmd {
                            Redirect::Output(oredirect) => Box::new(oredirect),
                            Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                            Redirect::Input(iredirect) => Box::new(iredirect),
                            Redirect::None => unreachable!(),
                        };
                        if in_and_sequence {
                            sequence_commands.push(r_cmd);
                            final_commands.push(create_and_sequence(&mut sequence_commands)?);
                            in_and_sequence = false;
                        } else {
                            final_commands.push(r_cmd);
                        }
                        r_type = RedirectType::None;
                    } else {
                        if in_and_sequence {
                            sequence_commands.push(runnable(tokens.clone())?);
                            final_commands.push(create_and_sequence(&mut sequence_commands)?);
                            in_and_sequence = false;
                        } else {
                            final_commands.push(runnable(tokens.clone())?);
                        }
                        tokens.clear();
                    }
                }
            }
            '|' => {
                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else if r_type != RedirectType::None {
                    let cmd = create_redirect(r_cmd.take().unwrap(), &mut tokens, &r_type)?;
                    commands.push(match cmd {
                        Redirect::Output(oredirect) => Box::new(oredirect),
                        Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                        Redirect::Input(iredirect) => Box::new(iredirect),
                        Redirect::None => unreachable!(),
                    });
                    r_type = RedirectType::None;
                } else {
                    debug!("Creating command from tokens: {:?}", tokens);
                    commands.push(create_command(&mut tokens)?);
                }
                in_pipeline = true;
            }
            '\\' => {
                debug!("Found backslash \\");

                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if !in_quotes {
                    escaped = !escaped;
                } else {
                    current_token.push(c);
                }
            }
            '\'' => {
                debug!("Found single quote \'");

                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_double_quotes {
                    current_token.push(c);
                } else if !escaped {
                    in_quotes = !in_quotes;
                    if !in_quotes && !current_token.is_empty() {
                        tokens.push(Token::SingleQuoted(current_token.clone()));
                        current_token.clear();
                    }
                } else {
                    current_token.push(c);
                }
                escaped = false;
            }
            '"' => {
                debug!("Found double quote \"");

                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes {
                    current_token.push(c);
                } else if !escaped {
                    in_double_quotes = !in_double_quotes;
                    if !in_double_quotes && !current_token.is_empty() {
                        tokens.push(Token::DoubleQuoted(current_token.clone()));
                        current_token.clear();
                    }
                } else {
                    current_token.push(c);
                }
                escaped = false;
            }
            ' ' => {
                debug!("Found whitespace");

                if in_variable {
                    current_token += &get_var(var_buffer.clone())?;
                    var_buffer.clear();
                    in_variable = false;
                }

                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else {
                    add_token(&mut tokens, &mut current_token);
                }
                escaped = false;
            }
            _ => {
                if in_variable {
                    var_buffer.push(c);
                } else {
                    current_token.push(c);
                }
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

fn add_token(tokens: &mut Vec<Token>, token: &mut String) {
    if !token.is_empty() {
        tokens.push(Token::Plain(token.clone()));
        token.clear();
    }
}

fn get_var(token: String) -> Result<String, Box<dyn Error>> {
    debug!("Getting variable: {}", token);
    let var_value = env::var(&token).unwrap_or("".to_string());
    Ok(var_value)
}

fn create_command(tokens: &mut Vec<Token>) -> Result<ShellCommandBox, Box<dyn Error>> {
    let new_cmd = cmd(tokens.clone())?;
    tokens.clear();
    Ok(new_cmd)
}

fn create_redirect(
    cmd: ShellCommandBox,
    tokens: &mut Vec<Token>,
    r_type: &RedirectType,
) -> Result<Redirect, Box<dyn Error>> {
    let token = tokens.iter().map(|t| t.to_string()).collect::<String>();
    let new_redirect = match r_type {
        RedirectType::Output => Ok(Redirect::Output(OutputRedirect::new(vec![cmd], token)?)),
        RedirectType::OutputAppend => Ok(Redirect::OutputAppend(OutputRedirectAppend::new(
            vec![cmd],
            token,
        )?)),
        RedirectType::Input => Ok(Redirect::Input(InputRedirect::new(vec![cmd], token)?)),
        RedirectType::None => Ok(Redirect::None),
    };
    tokens.clear();
    new_redirect
}

fn create_pipeline(cmds: &mut ShellCommandBoxes) -> Result<RunnableBox, Box<dyn Error>> {
    let new_pipeline = Pipeline::new(cmds.clone())?;
    cmds.clear();
    Ok(Box::new(new_pipeline))
}

fn create_and_sequence(cmds: &mut RunnableBoxes) -> Result<RunnableBox, Box<dyn Error>> {
    let new_and_sequence = AndSequence::new(cmds.clone())?;
    cmds.clear();
    Ok(Box::new(new_and_sequence))
}
