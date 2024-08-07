use std::error::Error;

const BUILTINS: &[&str] = &["cd", "pwd", "exit", "echo"];

pub fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

pub fn builtin(cmd: &str, args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    match cmd {
        "cd" => cd(args),
        "pwd" => pwd(),
        "exit" => exit(),
        "echo" => echo(args),
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
