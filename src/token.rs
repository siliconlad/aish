use std::fmt;

pub enum TokenType {
    Meta,
    Plain,
    DoubleQuoted,
    SingleQuoted,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Meta(String),
    Plain(String),
    DoubleQuoted(String),
    SingleQuoted(String),
}
pub type Tokens = Vec<Token>;

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Meta(s) => write!(f, "{}", s),
            Token::Plain(s) => write!(f, "{}", s),
            Token::DoubleQuoted(s) => write!(f, "{}", s),
            Token::SingleQuoted(s) => write!(f, "{}", s),
        }
    }
}

pub fn tokenize(value: String, token_type: TokenType) -> Token {
    match token_type {
        TokenType::Meta => Token::Meta(value),
        TokenType::Plain => Token::Plain(value),
        TokenType::DoubleQuoted => Token::DoubleQuoted(value),
        TokenType::SingleQuoted => Token::SingleQuoted(value),
    }
}
