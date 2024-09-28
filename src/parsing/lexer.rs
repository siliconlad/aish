use crate::errors::SyntaxError;
use crate::scanner::Scanner;
use crate::token::{tokenize, Token, TokenType, Tokens};

pub fn lex_impl(scanner: &mut Scanner<String>) -> Result<Tokens, SyntaxError> {
    let mut buffer = TokenBuffer::new();
    loop {
        if scanner.peek().is_none() {
            break;
        }

        match scanner.peek().unwrap() {
            '&' | '<' | ';' | '|' => {
                buffer.push(scanner.next()).save(TokenType::Meta);
            }
            '>' => {
                buffer.push(scanner.next());
                if Some('>') == scanner.peek() {
                    buffer.push(scanner.next());
                }
                buffer.save(TokenType::Meta);
            }
            ' ' => {
                scanner.next();
            }
            _ => {
                let mut escaped = false;
                let mut quote_type = QuoteType::None;

                loop {
                    let c = scanner.peek();

                    if c.is_none() {
                        buffer.save(TokenType::Plain);
                        break;
                    }

                    if is_escape(c.unwrap()) {
                        if escaped || quote_type.single() {
                            buffer.push(scanner.next());
                        } else {
                            escaped = true;
                        }
                    }

                    if is_meta(c.unwrap()) {
                        if escaped || quote_type.quoted() {
                            buffer.push(scanner.next());
                        } else {
                            buffer.save(TokenType::Plain);
                            break;
                        }
                    }

                    if is_whitespace(c.unwrap()) {
                        if escaped || quote_type.quoted() {
                            buffer.push(scanner.next());
                        } else {
                            buffer.save(TokenType::Plain);
                            scanner.next(); // Skip to next char
                            break;
                        }
                    }

                    if is_double_quote(c.unwrap()) {
                        if escaped || quote_type.single() {
                            buffer.push(scanner.next());
                        } else if quote_type.double() {
                            buffer.save(TokenType::DoubleQuoted);
                            quote_type = QuoteType::None;
                        } else {
                            buffer.save(TokenType::Plain);
                            quote_type = QuoteType::Double;
                        }
                    }

                    if is_single_quote(c.unwrap()) {
                        if escaped || quote_type.double() {
                            buffer.push(scanner.next());
                        } else if quote_type.single() {
                            buffer.save(TokenType::SingleQuoted);
                            quote_type = QuoteType::None;
                        } else {
                            buffer.save(TokenType::Plain);
                            quote_type = QuoteType::Single;
                        }
                    }
                }

                if quote_type.quoted() {
                    return Err(SyntaxError::UnclosedQuote);
                }
            }
        }
    }
    Ok(buffer.tokens())
}

#[derive(Debug, PartialEq, Eq)]
enum QuoteType {
    None,
    Single,
    Double,
}

impl QuoteType {
    fn quoted(&self) -> bool {
        *self != QuoteType::None
    }

    fn single(&self) -> bool {
        *self == QuoteType::Single
    }

    fn double(&self) -> bool {
        *self == QuoteType::Double
    }
}

struct TokenBuffer {
    token: String,
    tokens: Vec<Token>,
}

impl TokenBuffer {
    fn new() -> Self {
        TokenBuffer {
            token: String::new(),
            tokens: Vec::<Token>::new(),
        }
    }

    fn push(&mut self, token: char) -> &mut Self {
        self.token.push(token);
        self
    }

    fn save(&mut self, token_type: TokenType) -> bool {
        if self.token.is_empty() {
            return false;
        }
        let new_token = tokenize(self.token.clone(), token_type);
        self.tokens.push(new_token);
        self.token.clear();
        true
    }

    fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }
}

fn is_meta(c: char) -> bool {
    c == '&' || c == '<' || c == '>' || c == ';' || c == '|'
}

fn is_whitespace(c: char) -> bool {
    c == ' '
}

fn is_escape(c: char) -> bool {
    c == '\\'
}

fn is_single_quote(c: char) -> bool {
    c == '\''
}

fn is_double_quote(c: char) -> bool {
    c == '"'
}
