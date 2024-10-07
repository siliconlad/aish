use crate::errors::RuntimeError;
use crate::openai_client::OpenAIClient;

use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use std::process::ChildStdout;
use tokio::runtime::Runtime;

const BUILTINS: &[&str] = &[
    "cd", "pwd", "exit", "echo", "export", "unset", "llm", "alias",
];

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn builtin(
    cmd: String,
    args: Vec<String>,
    stdin: Option<ChildStdout>,
    aliases: &mut HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    match cmd.as_str() {
        "cd" => cd(args),
        "pwd" => pwd(),
        "exit" => exit(),
        "echo" => echo(args),
        "export" => export(args),
        "unset" => unset(args),
        "llm" => llm(args, stdin),
        "alias" => alias(args, aliases),
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

pub fn llm(args: Vec<String>, stdin: Option<ChildStdout>) -> Result<String, Box<dyn Error>> {
    let openai_client = OpenAIClient::new(None)?;
    let prompt = if args.is_empty() {
        "".to_string()
    } else {
        args.first().unwrap().clone()
    };
    let mut input = String::new();
    if let Some(mut stdin) = stdin {
        stdin.read_to_string(&mut input)?;
    }

    // TODO: do something more sophisticated
    debug!("Received input: {:?}", input);
    let context = if !input.is_empty() {
        format!("{}: {}", prompt, input)
    } else {
        prompt.to_string()
    };
    debug!("Context: {}", context);

    // TODO: make configurable
    let runtime = Runtime::new().unwrap();
    let output = runtime.block_on(openai_client.generate_text(&context, 100))?;
    debug!("Generated response: {}", output);
    println!("{}", output);

    Ok("".to_string())
}

pub fn alias(
    args: Vec<String>,
    aliases: &mut HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    if args.is_empty() {
        // Print all aliases
        for (alias, command) in aliases {
            println!("{}='{}'", alias, command);
        }
        Ok("".to_string())
    } else if args.len() == 1 {
        let parts: Vec<&str> = args[0].splitn(2, '=').collect();
        if parts.len() == 2 {
            let alias = parts[0];
            let command = parts[1];
            aliases.insert(alias.to_string(), command.to_string());
            Ok("".to_string())
        } else {
            match aliases.get(&args[0]) {
                Some(command) => {
                    println!("{}='{}'", args[0], command);
                    Ok("".to_string())
                }
                None => Err(format!("Alias '{}' not found", args[0]).into()),
            }
        }
    } else {
        Err("Usage: alias [name[=value] ...]".into())
    }
}
