use crate::builtins::builtin;
use crate::builtins::is_builtin;
use crate::errors::SyntaxError;
use crate::openai_client::OpenAIClient;
use crate::redirect;
use crate::token::Token;
use crate::traits::{Runnable, ShellCommand};

use nix::unistd::{dup2, fork, pipe, ForkResult};
use std::env;
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::process::{ChildStdout, Command, Stdio};
use tokio::runtime::Runtime;

pub enum CommandType {
    Builtin(BuiltinCommand),
    External(ExternalCommand),
    Llm(LlmCommand),
    InputRedirect(redirect::InputRedirect),
    OutputRedirect(redirect::OutputRedirect),
    OutputRedirectAppend(redirect::OutputRedirectAppend),
}

impl CommandType {
    pub fn create(tokens: Vec<Token>) -> Result<CommandType, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::ExpectedToken);
        }

        match &tokens[..] {
            [Token::DoubleQuoted(prompt)] => {
                debug!("Detected LLM command with tokens: {:?}", tokens);
                let openai_client = if let Ok(api_key) = env::var("OPENAI_API_KEY") {
                    OpenAIClient::new(api_key)
                } else {
                    return Err(SyntaxError::InvalidOpenAIKey);
                };
                Ok(CommandType::Llm(LlmCommand::new(
                    prompt.clone(),
                    openai_client,
                )))
            }
            [Token::Plain(cmd), ..] if is_builtin(cmd) => {
                debug!("Detected builtin command: {:?}", tokens);
                let string_tokens: Vec<String> =
                    tokens.into_iter().map(|t| t.to_string()).collect();
                Ok(CommandType::Builtin(BuiltinCommand::new(string_tokens)?))
            }
            _ => {
                debug!("Detected external command: {:?}", tokens);
                let string_tokens: Vec<String> =
                    tokens.into_iter().map(|t| t.to_string()).collect();
                Ok(CommandType::External(ExternalCommand::new(string_tokens)?))
            }
        }
    }

    pub fn unpack_cmd(self) -> Box<dyn ShellCommand> {
        match self {
            CommandType::Builtin(cmd) => Box::new(cmd),
            CommandType::External(cmd) => Box::new(cmd),
            CommandType::Llm(cmd) => Box::new(cmd),
            CommandType::InputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirectAppend(cmd) => Box::new(cmd),
        }
    }

    pub fn unpack_run(self) -> Box<dyn Runnable> {
        match self {
            CommandType::Builtin(cmd) => Box::new(cmd),
            CommandType::External(cmd) => Box::new(cmd),
            CommandType::Llm(cmd) => Box::new(cmd),
            CommandType::InputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirect(cmd) => Box::new(cmd),
            CommandType::OutputRedirectAppend(cmd) => Box::new(cmd),
        }
    }
}

impl fmt::Debug for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandType::Builtin(cmd) => write!(f, "{:?}", cmd),
            CommandType::External(cmd) => write!(f, "{:?}", cmd),
            CommandType::Llm(cmd) => write!(f, "{:?}", cmd),
            CommandType::InputRedirect(cmd) => write!(f, "{:?}", cmd),
            CommandType::OutputRedirect(cmd) => write!(f, "{:?}", cmd),
            CommandType::OutputRedirectAppend(cmd) => write!(f, "{:?}", cmd),
        }
    }
}

#[derive(Clone)]
pub struct BuiltinCommand {
    tokens: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(tokens: Vec<String>) -> Result<BuiltinCommand, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::InternalError);
        }
        Ok(BuiltinCommand { tokens })
    }

    pub fn run_builtin(&self) -> Result<(), Box<dyn Error>> {
        builtin(self.cmd(), self.args())?;
        Ok(())
    }
}

impl fmt::Debug for BuiltinCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BuiltinCommand({:?})", self.tokens)
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

#[derive(Clone)]
pub struct ExternalCommand {
    tokens: Vec<String>,
}

impl ExternalCommand {
    pub fn new(tokens: Vec<String>) -> Result<ExternalCommand, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::InternalError);
        }
        Ok(ExternalCommand { tokens })
    }
}

impl fmt::Debug for ExternalCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExternalCommand({:?})", self.tokens)
    }
}

impl Runnable for ExternalCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        let mut child = match Command::new(self.cmd()).args(self.args()).spawn() {
            Ok(child) => child,
            Err(e) => return Err(e.into()),
        };
        match child.wait() {
            Ok(code) => {
                if code.success() {
                    Ok("".to_string())
                } else {
                    Err(format!("{}", code).into())
                }
            }
            Err(e) => Err(e.into()),
        }
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

#[derive(Clone)]
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

impl fmt::Debug for LlmCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LlmCommand({:?})", self.prompt)
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

    fn pipe(&self, stdin: Option<ChildStdout>) -> Result<Option<ChildStdout>, Box<dyn Error>> {
        let mut input = String::new();
        if let Some(mut stdin) = stdin {
            stdin.read_to_string(&mut input)?;
        }

        let runtime = Runtime::new()?;
        let response = runtime.block_on(self.generate_response(Some(input)))?;

        let mut child = Command::new("echo")
            .arg(response)
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(child.stdout.take())
    }
}
