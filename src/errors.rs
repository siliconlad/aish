use thiserror::Error;

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
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
}
