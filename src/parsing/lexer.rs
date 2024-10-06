use crate::errors::SyntaxError;
use crate::parsing::scanner::Scanner;
use crate::token::{tokenize, Token, TokenType, Tokens};

pub fn lex_impl(scanner: &mut Scanner<String>) -> Result<Tokens, SyntaxError> {
    let mut buffer = TokenBuffer::new();
    loop {
        if scanner.peek().is_none() {
            debug!("Reached EOF");
            break;
        }

        match scanner.peek().unwrap() {
            '<' | ';' | '|' => {
                debug!("Meta: {}", scanner.peek().unwrap());
                buffer.push(scanner.next()).save(TokenType::Meta);
            }
            '>' => {
                buffer.push(scanner.next());
                if Some('>') == scanner.peek() {
                    debug!("Meta: >>");
                    buffer.push(scanner.next());
                } else {
                    debug!("Meta: >");
                }
                buffer.save(TokenType::Meta);
            }
            '&' => {
                let c = scanner.next();
                if Some('&') == scanner.peek() {
                    debug!("Meta: &&");
                    buffer.push(c);
                    buffer.push(scanner.next());
                    buffer.save(TokenType::Meta);
                } else {
                    return Err(SyntaxError::UnexpectedToken("&".to_string()));
                }
            }
            '$' => {
                debug!("Variable");
                scanner.next();
                loop {
                    let c = scanner.peek();
                    if c.is_none() || is_break_point(c.unwrap()) {
                        break;
                    } else {
                        buffer.push(scanner.next());
                    }
                }

                if buffer.is_empty() {
                    return Err(SyntaxError::UnexpectedToken("$".to_string()));
                }

                buffer.save(TokenType::Variable);
            }
            ' ' => {
                debug!("Whitespace");
                scanner.next();
            }
            _ => {
                let mut escaped = false;
                let mut quote_type = QuoteType::None;

                loop {
                    let c = scanner.peek();

                    if c.is_none() {
                        buffer.save(TokenType::Plain);
                        debug!("EOF");
                        break;
                    } else if is_escape(c.unwrap()) {
                        debug!("Escape: {}", c.unwrap());
                        if escaped || quote_type.single() {
                            buffer.push(scanner.next());
                            escaped = false;
                        } else {
                            scanner.next();
                            escaped = true;
                        }
                    } else if c.unwrap() == '$' {
                        if escaped || quote_type.single() {
                            buffer.push(scanner.next());
                            escaped = false;
                            continue;
                        }

                        debug!("Variable");
                        scanner.next();
                        if quote_type.double() {
                            let mut var_buffer = TokenBuffer::new();
                            loop {
                                let c = scanner.peek();
                                if c.is_none()
                                    || is_break_point(c.unwrap())
                                    || is_double_quote(c.unwrap())
                                {
                                    break;
                                } else {
                                    var_buffer.push(scanner.next());
                                }
                            }
                            if var_buffer.is_empty() {
                                return Err(SyntaxError::UnexpectedToken("$".to_string()));
                            }
                            var_buffer.save(TokenType::Variable);
                            buffer.push_token(var_buffer.tokens().get(0).unwrap().clone());
                        }
                    } else if is_meta(c.unwrap()) {
                        debug!("Meta: {}", c.unwrap());
                        if escaped || quote_type.quoted() {
                            buffer.push(scanner.next());
                            escaped = false;
                        } else {
                            buffer.save(TokenType::Plain);
                            break;
                        }
                    } else if is_whitespace(c.unwrap()) {
                        debug!("Whitespace");
                        if escaped || quote_type.quoted() {
                            buffer.push(scanner.next());
                            escaped = false;
                        } else {
                            buffer.save(TokenType::Plain);
                            scanner.next(); // Skip to next char
                            break;
                        }
                    } else if is_double_quote(c.unwrap()) {
                        debug!("Double quote");
                        let c = scanner.next();
                        if escaped || quote_type.single() {
                            buffer.push(c);
                            escaped = false;
                        } else if quote_type.double() {
                            buffer.save(TokenType::DoubleQuoted);
                            quote_type = QuoteType::None;
                            break;
                        } else {
                            buffer.save(TokenType::Plain);
                            quote_type = QuoteType::Double;
                            escaped = false;
                        }
                    } else if is_single_quote(c.unwrap()) {
                        debug!("Single quote");
                        let c = scanner.next();
                        if escaped || quote_type.double() {
                            buffer.push(c);
                            escaped = false;
                        } else if quote_type.single() {
                            buffer.save(TokenType::SingleQuoted);
                            quote_type = QuoteType::None;
                            break;
                        } else {
                            buffer.save(TokenType::Plain);
                            quote_type = QuoteType::Single;
                            escaped = false;
                        }
                    } else {
                        debug!("Char: {}", c.unwrap());
                        buffer.push(scanner.next());
                        escaped = false;
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
    token: Vec<Token>,
    tokens: Vec<Token>,
}

impl TokenBuffer {
    fn new() -> Self {
        TokenBuffer {
            token: Vec::<Token>::new(),
            tokens: Vec::<Token>::new(),
        }
    }

    fn push(&mut self, token: char) -> &mut Self {
        self.token.push(Token::Plain(token.to_string()));
        self
    }

    fn push_token(&mut self, token: Token) -> &mut Self {
        self.token.push(token);
        self
    }

    fn save(&mut self, token_type: TokenType) -> bool {
        if self.token.is_empty() {
            return false;
        }
        let new_token = tokenize(self.token.clone(), token_type);
        self.tokens.push(new_token);
        debug!("Saved token: {:?}", self.token);
        self.token.clear();
        true
    }

    fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }

    fn is_empty(&self) -> bool {
        self.token.is_empty()
    }
}

fn is_break_point(c: char) -> bool {
    is_meta(c) || is_whitespace(c) || c == '$'
}

fn is_meta(c: char) -> bool {
    ['&', '<', '>', ';', '|'].contains(&c)
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
