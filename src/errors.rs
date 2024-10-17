use thiserror::Error;
use std::error::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unclosed quote")]
    UnclosedQuote,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Expected token: {0}")]
    ExpectedToken(String),
    #[error("Internal Error")]
    InternalError,
    #[error("Invalid OPENAI_API_KEY: {0}")]
    InvalidOpenAIKey(String),
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] Box<dyn Error>),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
}
