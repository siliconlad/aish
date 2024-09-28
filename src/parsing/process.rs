pub fn process(input: String) -> String {
    // Remove leading and trailing whitespace
    let mut input = input.trim().to_string();

    // Remove trailing newline
    if input.ends_with('\n') {
        input.pop();
    }

    // Add semicolon to trigger final command
    if !input.ends_with(';') {
        input.push(';');
    }

    input
}

