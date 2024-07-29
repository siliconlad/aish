use std::error::Error;

pub fn cd(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME").unwrap();
    let path = args.first().map_or(home.as_str(), |s| *s);
    let mut path = path.to_string();

    // Replace ~ with the home directory
    if path.starts_with("~/") {
        path = path.replace("~", &home);
    }

    std::env::set_current_dir(path)?;
    Ok(())
}

pub fn pwd() -> Result<String, Box<dyn Error>> {
    let path = std::env::current_dir()?;
    Ok(path.display().to_string())
}

pub fn exit() -> Result<(), Box<dyn Error>> {
    std::process::exit(0);
}

pub fn echo(msg: Vec<&str>) -> Result<(), Box<dyn Error>> {
    println!("{}", msg.join(" "));
    Ok(())
}
