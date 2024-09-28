use std::collections::HashMap;
use std::error::Error;

const BUILTINS: &[&str] = &["cd", "pwd", "exit", "echo", "export", "unset", "alias"];

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn builtin(cmd: &str, args: Vec<&str>, aliases: &mut HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    match cmd {
        "cd" => cd(args),
        "pwd" => pwd(),
        "exit" => exit(),
        "echo" => echo(args),
        "export" => export(args),
        "unset" => unset(args),
        "alias" => alias(args, aliases),
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

pub fn export(args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    if args.is_empty() {
        for (key, value) in std::env::vars() {
            println!("{}=\"{}\"", key, value);
        }
        Ok("".to_string())
    } else if args.len() > 1 {
        return Err("export: too many arguments".into());
    } else {
        let (key, value) = args.first().unwrap().split_once("=").unwrap();
        std::env::set_var(key, value);
        Ok("".to_string())
    }
}

pub fn unset(args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    for arg in args {
        std::env::remove_var(arg);
    }
    Ok("".to_string())
}

pub fn alias(args: Vec<&str>, aliases: &mut HashMap<String, String>) -> Result<String, Box<dyn Error>> {
    debug!("Alias command: {:?}", args);
    if args.is_empty() {
        // Print all aliases
        for (alias, command) in aliases {
            println!("{}='{}'", alias, command);
        }
        Ok("".to_string())
    } else if args.len() == 1 {
        debug!("Alias command: {:?}", args);
        // Print specific alias if it exists
        let parts: Vec<&str> = args[0].splitn(2, '=').collect();
        debug!("Alias parts: {:?}", parts);
        if parts.len() == 2 {
            let alias = parts[0];
            let command = parts[1];
            debug!("Alias: {} -> {}", alias, command);
            aliases.insert(alias.to_string(), command.to_string());
            debug!("Aliases: {:?}", aliases);
            Ok(format!("Alias '{}' created for '{}'", alias, command))
        } else {
            match aliases.get(args[0]) {
                Some(command) => Ok(format!("{}='{}'", args[0], command)),
                None => Err(format!("Alias '{}' not found", args[0]).into()),
            }
        }
    } else {
        Err("Usage: alias [name[=value] ...]".into())
    }
}
