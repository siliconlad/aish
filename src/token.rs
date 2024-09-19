use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Meta(String),
    Plain(String),
    DoubleQuoted(String),
    SingleQuoted(String),
}

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
