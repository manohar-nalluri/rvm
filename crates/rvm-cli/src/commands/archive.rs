use rvm_core::{branch, Workspace};

pub fn execute(name: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    branch::archive(&ws, name)?;
    println!("Archived branch '{}'.", name);
    println!("Use `rvm branch --list-archived` to see archived branches.");
    Ok(())
}
