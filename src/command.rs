use crate::builtins::builtin;
use crate::builtins::is_builtin;
use crate::errors::SyntaxError;
use crate::openai_client::OpenAIClient;
use crate::redirect;
use crate::token::{join_tokens, Token};
use crate::traits::{Runnable, ShellCommand};

use nix::unistd::{dup2, fork, pipe, ForkResult};
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
            return Err(SyntaxError::ExpectedToken("".to_string()));
        }

        match &tokens[..] {
            [Token::DoubleQuoted(prompt)] => {
                debug!("Detected LLM command with tokens: {:?}", tokens);
                Ok(CommandType::Llm(LlmCommand::new(
                    join_tokens(prompt.to_vec()),
                    OpenAIClient::new(None)?,
                )))
            }
            [Token::Plain(cmd), ..] if is_builtin(cmd) => {
                debug!("Detected builtin command: {:?}", tokens);
                Ok(CommandType::Builtin(BuiltinCommand::new(tokens)?))
            }
            _ => {
                debug!("Detected external command: {:?}", tokens);
                Ok(CommandType::External(ExternalCommand::new(tokens)?))
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
    tokens: Vec<Token>,
}

impl BuiltinCommand {
    pub fn new(tokens: Vec<Token>) -> Result<BuiltinCommand, SyntaxError> {
        if tokens.is_empty() {
            return Err(SyntaxError::InternalError);
        }
        Ok(BuiltinCommand { tokens })
    }

    pub fn run_builtin(&self) -> Result<String, Box<dyn Error>> {
        let output = builtin(self.cmd(), self.args())?;
        Ok(output)
    }
}

impl fmt::Debug for BuiltinCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BuiltinCommand({:?})", self.tokens)
    }
}

impl Runnable for BuiltinCommand {
    fn run(&self) -> Result<String, Box<dyn Error>> {
        debug!("Running builtin: {:?}", self);
        let output = self.run_builtin()?;
        Ok(output)
    }
}

impl ShellCommand for BuiltinCommand {
    fn cmd(&self) -> String {
        self.tokens[0].resolve()
    }

    fn args(&self) -> Vec<String> {
        self.tokens[1..].iter().map(|s| s.resolve()).collect()
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
    tokens: Vec<Token>,
}

impl ExternalCommand {
    pub fn new(tokens: Vec<Token>) -> Result<ExternalCommand, SyntaxError> {
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
        debug!("Running external: {:?}", self);
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
    fn cmd(&self) -> String {
        self.tokens[0].resolve()
    }

    fn args(&self) -> Vec<String> {
        self.tokens[1..].iter().map(|s| s.resolve()).collect()
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
        // TODO: do something more sophisticated
        let context = if let Some(input) = input {
            format!(
                "Given the following input and prompt, decide whether to return a normal response to the input or a suggested new shell command. \
                Example one: input = result of a git diff, prompt = 'Write a commit message in one line', output = 'COMMAND: git commit -m \"<LLM generated response>\"'.\
                Example two: input = '', prompt = 'hello', output = 'Hello, how are you?' \
                Input: {} Prompt: {}",
                input, self.prompt
            )
        } else {
            self.prompt.clone()
        };

        debug!("Context: {}", context);

        let output = match self.openai_client.generate_text(&context, 100).await {
            Ok(text) => text,
            Err(e) => {
                return Err(Box::new(e));
            }
        };
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
        debug!("Running llm: {:?}", self);
        let runtime = Runtime::new().unwrap();
        let output = runtime.block_on(self.generate_response(None))?;
        Ok(output)
    }
}

impl ShellCommand for LlmCommand {
    fn cmd(&self) -> String {
        "llm".to_string()
    }

    fn args(&self) -> Vec<String> {
        vec![self.prompt.clone()]
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
