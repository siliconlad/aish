use std::error::Error;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref ALIASES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

const BUILTINS: &[&str] = &["cd", "pwd", "exit", "echo", "alias"];

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn builtin(cmd: &str, args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    match cmd {
        "cd" => cd(args),
        "pwd" => pwd(),
        "exit" => exit(),
        "echo" => echo(args),
        "alias" => alias(args),
        _ => Err(format!("{}: command not found", cmd).into()),
    }
}

pub fn cd(args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    let home = std::env::var("HOME").unwrap();
    let path = args.first().map_or(home.as_str(), |s| s);
    let mut path = path.to_string();

    // Replace ~ with the home directory
    if path.starts_with("~/") {
        path = path.replace("~", &home);
    }

    std::env::set_current_dir(path)?;
    Ok("".to_string())
}

pub fn pwd() -> Result<String, Box<dyn Error>> {
    let path = std::env::current_dir()?;
    println!("{}", path.display());
    Ok("".to_string())
}

pub fn exit() -> Result<String, Box<dyn Error>> {
    std::process::exit(0);
}

pub fn echo(msg: Vec<&str>) -> Result<String, Box<dyn Error>> {
    println!("{}", msg.join(" "));
    Ok("".to_string())
}

pub fn alias(args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    if args.len() != 1 {
        return Err("alias: not enough arguments or too many arguments".into());
    }

    let alias_input = args[0];
    let parts: Vec<&str> = alias_input.splitn(2, '=').collect();

    if parts.len() != 2 {
        return Err("alias: invalid format, expected name=value".into());
    }

    let alias_name = parts[0].to_string();
    let alias_value = parts[1].to_string();

    ALIASES.lock().unwrap().insert(alias_name, alias_value);
    Ok("".to_string())
}

pub fn expand_aliases(input: &str) -> String {
    let aliases = ALIASES.lock().unwrap();
    let mut words: Vec<&str> = input.split_whitespace().collect();
    if let Some(alias) = aliases.get(words[0]) {
        let alias_words: Vec<&str> = alias.split_whitespace().collect();
        words.splice(0..1, alias_words);
    }
    words.join(" ")
}
