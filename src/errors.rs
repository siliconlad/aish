use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unclosed quote")]
    UnclosedQuote,
}
