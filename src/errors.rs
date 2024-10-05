use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unclosed quote")]
    UnclosedQuote,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Expected token")]
    ExpectedToken,
    #[error("Internal Error")]
    InternalError,
    #[error("Invalid OPENAI_API_KEY")]
    InvalidOpenAIKey,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
}
