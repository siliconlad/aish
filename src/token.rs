#[derive(Clone, Debug)]
pub enum Token {
    Plain(String),
    DoubleQuoted(String),
    SingleQuoted(String),
}

impl Token {
    pub fn to_string(&self) -> String {
        match self {
            Token::Plain(s) | Token::DoubleQuoted(s) | Token::SingleQuoted(s) => s.clone(),
        }
    }
}