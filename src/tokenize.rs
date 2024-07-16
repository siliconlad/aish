#[derive(Debug)]
pub struct TokenizedInput<'a> {
  tokens: Vec<&'a str>,
}

pub fn tokenize<'a>(input: &'a mut String) -> TokenizedInput<'a> {
  let tokens: Vec<&'a str> = input.split_whitespace().collect();
  TokenizedInput { tokens }
}
