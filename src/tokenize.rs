#[derive(Debug)]
pub struct TokenizedInput<'a> {
    tokens: Vec<&'a str>,
}

impl<'a> TokenizedInput<'a> {
    pub fn cmd(&self) -> &'a str {
        self.tokens[0]
    }

    pub fn args(&self) -> Vec<&'a str> {
        self.tokens[1..].to_vec()
    }
}

pub fn tokenize<'a>(input: &'a mut str) -> TokenizedInput<'a> {
    let tokens: Vec<&'a str> = input.split_whitespace().collect();
    TokenizedInput { tokens }
}
