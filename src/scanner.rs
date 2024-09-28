pub struct Scanner {
    input: String,
    index: usize,
}

impl Scanner {
    pub fn new(input: String) -> Scanner {
        Scanner { input, index: 0 }
    }

    pub fn next(&mut self) -> Option<char> {
        self.index += 1;
        self.input.chars().nth(self.index - 1)
    }

    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.index)
    }
}
