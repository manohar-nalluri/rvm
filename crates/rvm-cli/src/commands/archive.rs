use rvm_core::{branch, SystemAuthenticator, Workspace};

pub fn execute(name: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let ws = Workspace::discover(&cwd)?;
    let auth = SystemAuthenticator::new();
    branch::archive(&ws, name, &auth)?;
    println!("Archived branch '{}'.", name);
    println!("Use `rvm branch --list-archived` to see archived branches.");
    Ok(())
}
