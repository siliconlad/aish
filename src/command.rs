use crate::builtins::builtin;
use crate::builtins::is_builtin;
use crate::traits::{Runnable, ShellCommand};
use crate::openai_client::OpenAIClient;
use crate::token::Token;

use nix::unistd::{dup2, fork, pipe, ForkResult};
use std::error::Error;
use std::process::{ChildStdout, Command, Stdio};
use std::io::{BufReader, Read};
use std::os::fd::{FromRawFd, AsRawFd, IntoRawFd};
use tokio::runtime::Runtime;
use std::env;
use std::fs::File;

pub fn cmd(tokens: Vec<Token>) -> Result<Box<dyn ShellCommand>, Box<dyn Error>> {
    if tokens.is_empty() {
        return Err("Tokens cannot be empty".into());
    }

    match &tokens[..] {
        [Token::DoubleQuoted(prompt)] => {
            debug!("Detected LLM command with tokens: {:?}", tokens);
            let openai_client = if let Ok(api_key) = env::var("OPENAI_API_KEY") {
                OpenAIClient::new(api_key)
            } else {
                return Err("OPENAI_API_KEY not set".into());
            };
            Ok(Box::new(LlmCommand::new(prompt.clone(), openai_client)))
        }
        [Token::Plain(cmd), ..] if is_builtin(cmd) => {
            debug!("Detected builtin command: {:?}", tokens);
            let string_tokens: Vec<String> = tokens.into_iter().map(|t| t.to_string()).collect();
            Ok(Box::new(BuiltinCommand::new(string_tokens)?))
        }
        _ => {
            debug!("Detected external command: {:?}", tokens);
            let string_tokens: Vec<String> = tokens.into_iter().map(|t| t.to_string()).collect();
            Ok(Box::new(ExternalCommand::new(string_tokens)?))
        }
    }
}

pub fn runnable(tokens: Vec<Token>) -> Result<Box<dyn Runnable>, Box<dyn Error>> {
    if tokens.is_empty() {
        return Err("Tokens cannot be empty".into());
    }

    match &tokens[..] {
        [Token::DoubleQuoted(prompt)] => {
            debug!("Detected LLM command with tokens: {:?}", tokens);
            let openai_client = if let Ok(api_key) = env::var("OPENAI_API_KEY") {
                OpenAIClient::new(api_key)
            } else {
                return Err("OPENAI_API_KEY not set".into());
            };
            Ok(Box::new(LlmCommand::new(prompt.clone(), openai_client)))
        }
        [Token::Plain(cmd), ..] if is_builtin(cmd) => {
            debug!("Detected builtin command: {:?}", tokens);
            let string_tokens: Vec<String> = tokens.into_iter().map(|t| t.to_string()).collect();
            Ok(Box::new(BuiltinCommand::new(string_tokens)?))
        }
        _ => {
            debug!("Detected external command: {:?}", tokens);
            let string_tokens: Vec<String> = tokens.into_iter().map(|t| t.to_string()).collect();
            Ok(Box::new(ExternalCommand::new(string_tokens)?))
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinCommand {
    tokens: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(tokens: Vec<String>) -> Result<BuiltinCommand, Box<dyn Error>> {
        if tokens.is_empty() {
            return Err("Tokens cannot be empty".into());
        }
        Ok(BuiltinCommand { tokens })
    }

    pub fn run_builtin(&self) -> Result<(), Box<dyn Error>> {
        builtin(self.cmd(), self.args())?;
        Ok(())
    }
}

impl Runnable for BuiltinCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        self.run_builtin()?;
        Ok("".to_string())
    }
}

impl ShellCommand for BuiltinCommand {
    fn cmd(&self) -> &str {
        &self.tokens[0]
    }

    fn args(&self) -> Vec<&str> {
        self.tokens[1..].iter().map(|s| s.as_str()).collect()
    }

    fn pipe(&self, _stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let (pipe_out_r, pipe_out_w) = pipe()?;
        let (pipe_err_r, pipe_err_w) = pipe()?;

        match unsafe { fork() }? {
            ForkResult::Parent { child: _ } => {
                drop(pipe_out_w);
                drop(pipe_err_w);
                Ok(Some(ChildStdout::from(pipe_out_r)))
            }
            ForkResult::Child => {
                drop(pipe_out_r);
                drop(pipe_err_r);
                dup2(pipe_out_w.as_raw_fd(), 1)?;
                dup2(pipe_err_w.as_raw_fd(), 2)?;
                self.run_builtin()?;
                std::process::exit(0);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternalCommand {
    tokens: Vec<String>,
}

impl ExternalCommand {
    pub fn new(tokens: Vec<String>) -> Result<ExternalCommand, Box<dyn Error>> {
        if tokens.is_empty() {
            return Err("Tokens cannot be empty".into());
        }
        Ok(ExternalCommand { tokens })
    }
}

impl Runnable for ExternalCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut child = match Command::new(self.cmd()).args(self.args()).spawn() {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };
        match child.wait() {
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
        Ok("".to_string())
    }
}

impl ShellCommand for ExternalCommand {
    fn cmd(&self) -> &str {
        &self.tokens[0]
    }

    fn args(&self) -> Vec<&str> {
        self.tokens[1..].iter().map(|s| s.as_str()).collect()
    }

    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let input = match stdin {
            Some(input) => Stdio::from(input),
            None => Stdio::inherit(),
        };
        // Spawn the command
        let mut child = match Command::new(self.cmd())
            .args(self.args())
            .stdin(input)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };

        Ok(child.stdout.take())
    }
}

#[derive(Debug, Clone)]
pub struct LlmCommand {
    prompt: String,
    openai_client: OpenAIClient,
}

impl LlmCommand {
    pub fn new(prompt: String, openai_client: OpenAIClient) -> Self {
        LlmCommand {
            prompt,
            openai_client,
        }
    }

    pub async fn generate_response(&self, input: Option<String>) -> Result<String, Box<dyn Error>> {
        debug!("Received input: {:?}", input);
        let context = if let Some(input) = input {
            format!("{}: {}", self.prompt, input)
        } else {
            self.prompt.clone()
        };
        let output = self.openai_client.generate_text(&context, 100).await?; // TODO: make this configurable
        debug!("Generated response: {}", output);
        Ok(output)
    }
}

impl Runnable for LlmCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let runtime = Runtime::new().unwrap();
        let output = runtime.block_on(self.generate_response(None))?;
        Ok(output)
    }
}

impl ShellCommand for LlmCommand {
    fn cmd(&self) -> &str {
        "llm"
    }

    fn args(&self) -> Vec<&str> {
        vec![&self.prompt]
    }

    fn pipe(&self, stdin: Option<Stdio>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        // let mut input = String::new();
        // if let Some(stdinput) = stdin {
        //     let hi: ChildStdout = stdinput.into();
        //     let mut reader =
        //         BufReader::new(unsafe { File::from_raw_fd(hi.into_raw_fd()) });
        //     reader.read_to_string(&mut input)?;
        // }

        // let runtime = Runtime::new().unwrap();
        // let response = runtime.block_on(self.generate_response(Some(input)))?;
        println!("llm said hello");
        let mut child = Command::new("cat").stdin(stdin.unwrap()).stdout(Stdio::piped()).spawn()?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        Ok(Some(stdout))
    }
}
