use std::error::Error;

pub fn cd(args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    let path = args.get(0).ok_or("expected a path")?;
    let mut path = path.to_string();

    // Replace ~ with the home directory
    let home = std::env::var("HOME").unwrap();
    if path.starts_with("~/") {
      path = path.replace("~", &home);
    }

    std::env::set_current_dir(path)?;
    Ok(())
}
