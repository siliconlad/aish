use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Context;
use rustyline_derive::Helper;

#[derive(Helper)]
pub struct ShellHelper {
    pub suggestion: String,
}

impl Completer for ShellHelper {
    type Candidate = String;
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos == line.len() && !self.suggestion.is_empty() && self.suggestion.starts_with(line) {
            Some(self.suggestion[line.len()..].to_owned())
        } else {
            None
        }
    }
}

impl Highlighter for ShellHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        // Use ANSI escape codes to color the hint gray
        format!("\x1b[38;5;240m{}\x1b[0m", hint).into()
    }
}

impl Validator for ShellHelper {}
