use crate::token::{Token, Tokens};

pub struct Scanner<T> {
    input: T,
    index: usize,
}

impl<T> Scanner<T> {
    pub fn new(input: T) -> Scanner<T> {
        Scanner { input, index: 0 }
    }
}

impl Scanner<String> {
    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.index)
    }

    pub fn peek_next(&self, index: usize) -> Option<char> {
        self.input.chars().nth(self.index + index)
    }

    pub fn next(&mut self) -> char {
        self.index += 1;
        self.input.chars().nth(self.index - 1).unwrap()
    }
}

impl Scanner<Tokens> {
    pub fn peek(&self) -> Option<Token> {
        self.input.get(self.index).cloned()
    }

    pub fn next(&mut self) -> Token {
        self.index += 1;
        self.input.get(self.index - 1).unwrap().clone()
    }
}
