use std::fmt::Display;

#[derive(Debug)]
pub struct TokenizedInput {
    tokens: Vec<String>,
}

impl TokenizedInput {
    pub fn cmd(&self) -> &String {
        &self.tokens[0]
    }

    pub fn args(&self) -> Vec<String> {
        self.tokens[1..].to_vec()
    }
}

pub fn clean(input: &mut String) -> &mut String {
    *input = input.trim().to_string();

    if input.ends_with('\n') {
        input.pop();
    }
    // Replace each whitespace with a single space
    *input = input.split_whitespace().collect::<Vec<&str>>().join(" ");

    input
}

pub fn tokenize(input: &mut String) -> TokenizedInput {
    let mut in_quotes = false;
    let mut in_double_quotes = false;
    let mut escaped = false;
    let mut current_token = String::new();
    let mut tokens = Vec::<String>::new();

    let cleaned = clean(input);

    for c in cleaned.chars() {
        match c {
            '\\' => {
                if !in_quotes {
                    escaped = !escaped;
                } else {
                    current_token.push(c);
                }
            }
            '\'' => {
                if in_double_quotes {
                    current_token.push(c);
                } else if !escaped {
                    in_quotes = !in_quotes;
                } else {
                    current_token.push(c);
                }
                escaped = false;
            }
            '"' => {
                if in_quotes {
                    current_token.push(c);
                } else if !escaped {
                    in_double_quotes = !in_double_quotes;
                } else {
                    current_token.push(c);
                }
                escaped = false;
            }
            ' ' => {
                if in_quotes || in_double_quotes || escaped {
                    current_token.push(c);
                } else {
                    tokens.push(current_token);
                    current_token = String::new();
                }
                escaped = false;
            }
            _ => {
                current_token.push(c);
                escaped = false;
            }
        }
    }
    // Add last token
    tokens.push(current_token);

    // Remove empty strings
    tokens.retain(|x| !x.is_empty());

    TokenizedInput { tokens }
}

impl Display for TokenizedInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.tokens)
    }
}
