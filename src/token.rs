use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Meta,
    Plain,
    DoubleQuoted,
    SingleQuoted,
    Variable,
    Tilde,
    Group,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Meta(String),
    Plain(String),
    Variable(String),
    Tilde,
    Group(Vec<Token>),
    DoubleQuoted(Vec<Token>),
    SingleQuoted(Vec<Token>),
}
pub type Tokens = Vec<Token>;

impl Token {
    pub fn resolve(&self) -> String {
        match self {
            Token::Meta(s) => s.clone(),
            Token::Plain(s) => s.clone(),
            Token::Variable(s) => std::env::var(s.clone()).unwrap_or("".to_string()),
            Token::DoubleQuoted(s) => join_tokens(s.to_vec()),
            Token::SingleQuoted(s) => join_tokens(s.to_vec()),
            Token::Tilde => std::env::var("HOME").unwrap_or("".to_string()),
            Token::Group(s) => join_tokens(s.to_vec()),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Meta(s) => write!(f, "{:?}", s),
            Token::Plain(s) => write!(f, "{:?}", s),
            Token::DoubleQuoted(s) => write!(f, "{:?}", s),
            Token::SingleQuoted(s) => write!(f, "{:?}", s),
            Token::Variable(s) => write!(f, "${:?}", s),
            Token::Tilde => write!(f, "~"),
            Token::Group(s) => write!(f, "{:?}", s),
        }
    }
}

pub fn tokenize(value: Vec<Token>, token_type: TokenType) -> Token {
    match token_type {
        TokenType::Meta => Token::Meta(join_tokens(value)),
        TokenType::Plain => Token::Plain(join_tokens(value)),
        TokenType::DoubleQuoted => Token::DoubleQuoted(value),
        TokenType::SingleQuoted => Token::SingleQuoted(value),
        TokenType::Variable => Token::Variable(join_tokens(value)),
        TokenType::Tilde => Token::Tilde,
        TokenType::Group => Token::Group(value),
    }
}

pub fn join_tokens(tokens: Vec<Token>) -> String {
    tokens.iter().map(|t| t.resolve()).collect::<String>()
}
