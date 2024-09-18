use std::fmt;

#[derive(Clone, Debug)]
pub enum Token {
    Plain(String),
    DoubleQuoted(String),
    SingleQuoted(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Plain(s) | Token::DoubleQuoted(s) | Token::SingleQuoted(s) => write!(f, "{}", s),
        }
    }
}
