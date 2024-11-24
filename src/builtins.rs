use crate::errors::RuntimeError;

use std::error::Error;

const BUILTINS: &[&str] = &["cd", "pwd", "exit", "echo", "export", "unset"];

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn builtin(cmd: String, args: Vec<String>) -> Result<String, Box<dyn Error>> {
    match cmd.as_str() {
        "cd" => cd(args),
        "pwd" => pwd(),
        "exit" => exit(),
        "echo" => echo(args),
        "export" => export(args),
        "unset" => unset(args),
        _ => Err(format!("{}: command not found", cmd).into()),
    }
}

pub fn echo(msg: Vec<String>) -> Result<String, Box<dyn Error>> {
    println!("{}", msg.join(" "));
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

pub fn cd(args: Vec<String>) -> Result<String, Box<dyn Error>> {
    let home = std::env::var("HOME").unwrap();
    let path = args.first().map_or(home.as_str(), |s| s);
    let path = path.to_string();

    // Check if path exists
    if !std::path::Path::new(&path).exists() {
        debug!("cd: no such file or directory: {}", path);
        return Err(Box::new(RuntimeError::CommandFailed(
            "cd: no such directory".into(),
        )));
    }

    std::env::set_current_dir(path)?;
    Ok("".to_string())
}

pub fn export(args: Vec<String>) -> Result<String, Box<dyn Error>> {
    if args.is_empty() {
        for (key, value) in std::env::vars() {
            println!("{}=\"{}\"", key, value);
        }
        Ok("".to_string())
    } else if args.len() > 1 {
        return Err("export: too many arguments".into());
    } else {
        let (key, value) = args.first().unwrap().split_once("=").unwrap();

        if value.eq("~") || value.starts_with("~/") {
            let home = std::env::var("HOME").unwrap();
            let value = value.replace("~", home.as_str());
            std::env::set_var(key, value);
        } else {
            std::env::set_var(key, value);
        }

        Ok("".to_string())
    }
}

pub fn unset(args: Vec<String>) -> Result<String, Box<dyn Error>> {
    for arg in args {
        std::env::remove_var(arg);
    }
    Ok("".to_string())
}
