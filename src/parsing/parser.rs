use std::error::Error;

use crate::command::{cmd, runnable};
use crate::errors::SyntaxError;
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

pub fn parse_impl(tokens: &Vec<Token>) -> Result<RunnableBox, Box<dyn Error>> {
    let mut in_pipeline = false;
    let mut in_and_sequence = false;
    let mut r_type = RedirectType::None;

    let mut current_command = Vec::<Token>::new();
    let mut commands = Vec::<Box<dyn ShellCommand>>::new();
    let mut r_cmd: Option<Box<dyn ShellCommand>> = None;
    let mut sequence_commands = Vec::<Box<dyn Runnable>>::new();
    let mut final_commands = Vec::<Box<dyn Runnable>>::new();

    for token in tokens {
        match token {
            Token::Meta(m) if m == ";" => {
                if in_pipeline {
                    if r_type != RedirectType::None {
                        let cmd =
                            create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                        commands.push(match cmd {
                            Redirect::Output(oredirect) => Box::new(oredirect),
                            Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                            Redirect::Input(iredirect) => Box::new(iredirect),
                            Redirect::None => unreachable!(),
                        });
                        r_type = RedirectType::None;
                    } else {
                        commands.push(create_command(&mut current_command)?);
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
                    let cmd =
                        create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
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
                        sequence_commands.push(runnable(current_command.clone())?);
                        final_commands.push(create_and_sequence(&mut sequence_commands)?);
                        in_and_sequence = false;
                    } else {
                        final_commands.push(runnable(current_command.clone())?);
                    }
                    current_command.clear();
                }
            }
            Token::Meta(m) if m == "|" => {
                if r_type != RedirectType::None {
                    let cmd =
                        create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                    commands.push(match cmd {
                        Redirect::Output(oredirect) => Box::new(oredirect),
                        Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                        Redirect::Input(iredirect) => Box::new(iredirect),
                        Redirect::None => unreachable!(),
                    });
                    r_type = RedirectType::None;
                } else {
                    commands.push(create_command(&mut current_command)?);
                }
                in_pipeline = true;
            }
            Token::Meta(m) if m == "&" => {
                in_and_sequence = true;
                if in_pipeline {
                    if r_type != RedirectType::None {
                        let cmd =
                            create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                        commands.push(match cmd {
                            Redirect::Output(oredirect) => Box::new(oredirect),
                            Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                            Redirect::Input(iredirect) => Box::new(iredirect),
                            Redirect::None => unreachable!(),
                        });
                        r_type = RedirectType::None;
                    } else {
                        commands.push(create_command(&mut current_command)?);
                    }
                    sequence_commands.push(create_pipeline(&mut commands)?);
                    in_pipeline = false;
                } else if r_type != RedirectType::None {
                    let cmd =
                        create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                    sequence_commands.push(match cmd {
                        Redirect::Output(oredirect) => Box::new(oredirect),
                        Redirect::OutputAppend(aredirect) => Box::new(aredirect),
                        Redirect::Input(iredirect) => Box::new(iredirect),
                        Redirect::None => unreachable!(),
                    });
                    r_type = RedirectType::None;
                } else {
                    sequence_commands.push(runnable(current_command.clone())?);
                    current_command.clear();
                }
            }
            Token::Meta(m) if m == "<" => {
                if r_type == RedirectType::Output || r_type == RedirectType::OutputAppend {
                    let cmd =
                        create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                    r_cmd = match cmd {
                        Redirect::Output(oredirect) => Some(Box::new(oredirect)),
                        Redirect::OutputAppend(aredirect) => Some(Box::new(aredirect)),
                        _ => unreachable!(),
                    };
                } else {
                    r_cmd = Some(create_command(&mut current_command)?);
                }
                r_type = RedirectType::Input;
            }
            Token::Meta(m) if m == ">" => {
                if r_type == RedirectType::Input {
                    let cmd =
                        create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                    r_cmd = match cmd {
                        Redirect::Input(iredirect) => Some(Box::new(iredirect)),
                        _ => unreachable!(),
                    };
                } else if r_type == RedirectType::None {
                    r_cmd = Some(create_command(&mut current_command)?);
                }
                r_type = RedirectType::Output;
            }
            Token::Meta(m) if m == ">>" => {
                if r_type == RedirectType::Input {
                    let cmd =
                        create_redirect(r_cmd.take().unwrap(), &mut current_command, &r_type)?;
                    r_cmd = match cmd {
                        Redirect::Input(iredirect) => Some(Box::new(iredirect)),
                        _ => unreachable!(),
                    };
                } else if r_type == RedirectType::None {
                    r_cmd = Some(create_command(&mut current_command)?);
                }
                r_type = RedirectType::OutputAppend;
            }
            // Not relevant to parsing
            Token::Plain(_) => current_command.push(token.clone()),
            Token::SingleQuoted(_) => current_command.push(token.clone()),
            Token::DoubleQuoted(_) => current_command.push(token.clone()),
            // Should never happen
            _ => unreachable!(),
        }
    }

    Ok(Box::new(Sequence::new(final_commands)?))
}

// fn add_token(tokens: &mut Vec<Token>, token: &mut String) {
//     if !token.is_empty() {
//         tokens.push(Token::Plain(token.clone()));
//         token.clear();
//     }
// }

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
