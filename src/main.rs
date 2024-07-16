pub mod tokenize;

use std::io::{self, Write};

fn main() -> Result<(), std::io::Error> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        let tokenized = tokenize::tokenize(&mut buffer);
        println!("{:?}", tokenized);
    }
}
