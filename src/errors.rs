use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unclosed quote")]
    UnclosedQuote,
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Expected token")]
    ExpectedToken,
    #[error("Invalid OPENAI_API_KEY")]
    InvalidOpenAIKey,
    #[error("Internal Error")]
    InternalError,
}
